pub mod detect;
pub mod session;

use base64::Engine;
use image::RgbaImage;
use xcap::{Monitor, Window};

use crate::error::{AppError, Result};
use crate::models::{CaptureSource, SourceKind};

/// Resolved capture target. Held for the lifetime of a recording so we don't
/// re-enumerate the window list on every sample.
pub enum Target {
    Monitor(Box<Monitor>),
    Window(Box<Window>),
}

impl Target {
    pub fn grab(&self) -> Result<RgbaImage> {
        match self {
            Target::Monitor(m) => Ok(m.capture_image()?),
            Target::Window(w) => Ok(w.capture_image()?),
        }
    }

    pub fn label(&self) -> String {
        match self {
            Target::Monitor(m) => m
                .friendly_name()
                .or_else(|_| m.name())
                .unwrap_or_else(|_| "Display".into()),
            Target::Window(w) => w
                .title()
                .ok()
                .filter(|t| !t.is_empty())
                .or_else(|| w.app_name().ok())
                .unwrap_or_else(|| "Window".into()),
        }
    }
}

/// Parse a `monitor:<id>` / `window:<id>` handle back into a live capture target.
pub fn resolve(id: &str) -> Result<Target> {
    let (kind, raw) = id
        .split_once(':')
        .ok_or_else(|| AppError::Invalid(format!("Malformed capture source `{id}`.")))?;

    match kind {
        "monitor" => {
            let monitors = Monitor::all()?;
            // Fall back to the primary display: monitor ids change when a user
            // unplugs and replugs a dock, and failing outright would be hostile.
            let found = monitors
                .iter()
                .find(|m| m.id().is_ok_and(|v| v.to_string() == raw))
                .or_else(|| monitors.iter().find(|m| m.is_primary().unwrap_or(false)))
                .ok_or(AppError::SourceUnavailable)?;
            Ok(Target::Monitor(Box::new(found.clone())))
        }
        "window" => {
            let windows = Window::all()?;
            let found = windows
                .iter()
                .find(|w| w.id().is_ok_and(|v| v.to_string() == raw))
                .ok_or(AppError::SourceUnavailable)?;
            Ok(Target::Window(Box::new(found.clone())))
        }
        _ => Err(AppError::Invalid(format!("Unknown source kind `{kind}`."))),
    }
}

/// Everything the user can record, with thumbnails for the picker.
///
/// Thumbnails are the expensive part (one full screenshot per source), so the
/// caller can skip them when it only needs the list.
pub fn list_sources(with_thumbnails: bool) -> Result<Vec<CaptureSource>> {
    let mut out = Vec::new();

    for m in Monitor::all()? {
        let (Ok(id), Ok(width), Ok(height)) = (m.id(), m.width(), m.height()) else {
            continue;
        };
        let is_primary = m.is_primary().unwrap_or(false);
        let name = m
            .friendly_name()
            .or_else(|_| m.name())
            .unwrap_or_else(|_| "Display".into());
        out.push(CaptureSource {
            id: format!("monitor:{id}"),
            kind: SourceKind::Monitor,
            name,
            detail: format!("{width} × {height}"),
            width,
            height,
            is_primary,
            thumbnail: with_thumbnails
                .then(|| m.capture_image().ok().map(|img| thumbnail_data_url(&img, 480)))
                .flatten(),
        });
    }
    // Primary display first, then the rest in enumeration order.
    out.sort_by_key(|s| !s.is_primary);

    let mut windows = Vec::new();
    for w in Window::all()? {
        if w.is_minimized().unwrap_or(false) {
            continue;
        }
        let (Ok(id), Ok(width), Ok(height)) = (w.id(), w.width(), w.height()) else {
            continue;
        };
        // Filter out the desktop, menu bar helpers and other chrome that would
        // otherwise clutter the picker.
        if width < 160 || height < 120 {
            continue;
        }
        let title = w.title().unwrap_or_default();
        let app = w.app_name().unwrap_or_default();
        if title.trim().is_empty() && app.trim().is_empty() {
            continue;
        }
        // Never offer Steppy's own windows — recording ourselves is a trap.
        if app.eq_ignore_ascii_case("steppy") || app.eq_ignore_ascii_case("stepsy") {
            continue;
        }
        windows.push(CaptureSource {
            id: format!("window:{id}"),
            kind: SourceKind::Window,
            name: if title.trim().is_empty() {
                app.clone()
            } else {
                title
            },
            detail: if app.trim().is_empty() {
                format!("{width} × {height}")
            } else {
                app
            },
            width,
            height,
            is_primary: false,
            thumbnail: with_thumbnails
                .then(|| w.capture_image().ok().map(|img| thumbnail_data_url(&img, 480)))
                .flatten(),
        });
    }
    windows.sort_by_key(|s| s.detail.to_lowercase());
    out.append(&mut windows);

    Ok(out)
}

fn thumbnail_data_url(image: &RgbaImage, max_w: u32) -> String {
    let scaled = if image.width() > max_w {
        let h = (image.height() as f32 * max_w as f32 / image.width() as f32).round() as u32;
        image::imageops::thumbnail(image, max_w, h.max(1))
    } else {
        image.clone()
    };
    let mut buf = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 72);
    match image::DynamicImage::ImageRgba8(scaled)
        .to_rgb8()
        .write_with_encoder(encoder)
    {
        Ok(()) => format!(
            "data:image/jpeg;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(&buf)
        ),
        Err(_) => String::new(),
    }
}

/// Shrink oversized captures before they ever touch the disk. Retina displays
/// produce 6000px-wide PNGs that nobody needs in a document.
pub fn downscale(image: RgbaImage, max_w: u32) -> RgbaImage {
    if max_w == 0 || image.width() <= max_w {
        return image;
    }
    let h = (image.height() as f32 * max_w as f32 / image.width() as f32).round() as u32;
    image::imageops::resize(&image, max_w, h.max(1), image::imageops::FilterType::Lanczos3)
}

// ---------------------------------------------------------------------------
// macOS screen recording permission
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
}

/// Whether the OS will actually let us capture pixels right now.
pub fn has_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        unsafe { CGPreflightScreenCaptureAccess() }
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// Triggers the system prompt the first time. Afterwards macOS only shows the
/// "open System Settings" affordance, which is why the UI also offers a
/// deep link into the Privacy pane.
pub fn request_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        unsafe { CGRequestScreenCaptureAccess() }
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

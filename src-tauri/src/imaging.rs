use std::path::Path;

use image::{Rgba, RgbaImage};

use crate::error::Result;
use crate::models::{Annotation, AnnotationKind};

/// Steppy's accent, used for highlight boxes so exports match the editor.
const HIGHLIGHT: Rgba<u8> = Rgba([99, 102, 241, 255]);
const REDACT_FILL: Rgba<u8> = Rgba([24, 24, 27, 255]);

pub fn load(path: &Path) -> Result<RgbaImage> {
    Ok(image::open(path)?.to_rgba8())
}

pub fn fit_width(image: RgbaImage, max_w: u32) -> RgbaImage {
    if max_w == 0 || image.width() <= max_w {
        return image;
    }
    let h = (image.height() as f32 * max_w as f32 / image.width() as f32).round() as u32;
    image::imageops::resize(&image, max_w, h.max(1), image::imageops::FilterType::Lanczos3)
}

/// Burn annotations into the pixels.
///
/// This happens on every export path (and before anything is sent to Gemini)
/// so a redacted region is genuinely gone, not just hidden by a CSS overlay.
pub fn apply_annotations(mut image: RgbaImage, annotations: &[Annotation]) -> RgbaImage {
    let (w, h) = (image.width(), image.height());
    for ann in annotations {
        let (x, y, rw, rh) = ann.rect.to_pixels(w, h);
        if rw == 0 || rh == 0 {
            continue;
        }
        match ann.kind {
            AnnotationKind::Blur => mosaic(&mut image, x, y, rw, rh),
            AnnotationKind::Redact => fill(&mut image, x, y, rw, rh, REDACT_FILL),
            AnnotationKind::Highlight => {
                // Scale the stroke with the image so it reads the same at any
                // export size.
                let stroke = (w as f32 / 400.0).round().clamp(2.0, 8.0) as u32;
                outline(&mut image, x, y, rw, rh, HIGHLIGHT, stroke);
            }
        }
    }
    image
}

/// Mosaic rather than a gaussian blur: blurred text can sometimes be recovered,
/// and a coarse mosaic reads unambiguously as "deliberately hidden".
fn mosaic(image: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32) {
    let block = (w.min(h) as f32 / 6.0).round().clamp(8.0, 40.0) as u32;
    let mut by = y;
    while by < y + h {
        let mut bx = x;
        while bx < x + w {
            let bw = block.min(x + w - bx);
            let bh = block.min(y + h - by);
            let (mut r, mut g, mut b, mut n) = (0u32, 0u32, 0u32, 0u32);
            for py in by..by + bh {
                for px in bx..bx + bw {
                    let p = image.get_pixel(px, py).0;
                    r += p[0] as u32;
                    g += p[1] as u32;
                    b += p[2] as u32;
                    n += 1;
                }
            }
            if let (Some(r), Some(g), Some(b)) = (
                r.checked_div(n),
                g.checked_div(n),
                b.checked_div(n),
            ) {
                let avg = Rgba([r as u8, g as u8, b as u8, 255]);
                for py in by..by + bh {
                    for px in bx..bx + bw {
                        image.put_pixel(px, py, avg);
                    }
                }
            }
            bx += block;
        }
        by += block;
    }
}

fn fill(image: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: Rgba<u8>) {
    for py in y..(y + h).min(image.height()) {
        for px in x..(x + w).min(image.width()) {
            image.put_pixel(px, py, color);
        }
    }
}

fn outline(image: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: Rgba<u8>, stroke: u32) {
    let (iw, ih) = (image.width(), image.height());
    for s in 0..stroke {
        let top = y.saturating_add(s);
        let bottom = (y + h).saturating_sub(s + 1);
        for px in x..(x + w).min(iw) {
            if top < ih {
                image.put_pixel(px, top, color);
            }
            if bottom < ih {
                image.put_pixel(px, bottom, color);
            }
        }
        let left = x.saturating_add(s);
        let right = (x + w).saturating_sub(s + 1);
        for py in y..(y + h).min(ih) {
            if left < iw {
                image.put_pixel(left, py, color);
            }
            if right < iw {
                image.put_pixel(right, py, color);
            }
        }
    }
}

pub fn encode_jpeg(image: &RgbaImage, quality: u8) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
    image::DynamicImage::ImageRgba8(image.clone())
        .to_rgb8()
        .write_with_encoder(encoder)?;
    Ok(buf)
}

pub fn encode_png(image: &RgbaImage) -> Result<Vec<u8>> {
    let mut buf = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image.clone())
        .to_rgb8()
        .write_to(&mut buf, image::ImageFormat::Png)?;
    Ok(buf.into_inner())
}

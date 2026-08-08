use std::path::Path;

use image::{Rgba, RgbaImage};

use crate::error::Result;
use crate::models::{Annotation, AnnotationKind, AnnotationStroke};

/// Steppy's default accent when no colour is stored on the annotation.
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
            AnnotationKind::Redact => {
                let color = annotation_color(ann, REDACT_FILL);
                fill(&mut image, x, y, rw, rh, color);
            }
            AnnotationKind::Highlight => {
                let color = annotation_color(ann, HIGHLIGHT);
                let fill = Rgba([color.0[0], color.0[1], color.0[2], 40]);
                blend_fill(&mut image, x, y, rw, rh, fill);
                let stroke = highlight_stroke(w, ann.stroke);
                outline(&mut image, x, y, rw, rh, color, stroke);
            }
        }
    }
    image
}

fn highlight_stroke(img_w: u32, stroke: Option<AnnotationStroke>) -> u32 {
    let base = (img_w as f32 / 400.0).round().clamp(2.0, 8.0) as u32;
    match stroke.unwrap_or(AnnotationStroke::Medium) {
        AnnotationStroke::Thin => base.saturating_sub(1).max(1),
        AnnotationStroke::Medium => base,
        AnnotationStroke::Thick => base + 2,
    }
}

fn annotation_color(ann: &Annotation, fallback: Rgba<u8>) -> Rgba<u8> {
    ann.color
        .as_deref()
        .and_then(parse_hex_color)
        .unwrap_or(fallback)
}

fn parse_hex_color(hex: &str) -> Option<Rgba<u8>> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Rgba([r, g, b, 255]))
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

fn blend_fill(image: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, tint: Rgba<u8>) {
    let alpha = tint.0[3] as f32 / 255.0;
    for py in y..(y + h).min(image.height()) {
        for px in x..(x + w).min(image.width()) {
            let base = image.get_pixel(px, py).0;
            let blended = Rgba([
                lerp(base[0], tint.0[0], alpha),
                lerp(base[1], tint.0[1], alpha),
                lerp(base[2], tint.0[2], alpha),
                255,
            ]);
            image.put_pixel(px, py, blended);
        }
    }
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round().clamp(0.0, 255.0) as u8
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Rect;

    #[test]
    fn highlight_uses_stored_colour_and_fill() {
        let image = RgbaImage::from_pixel(40, 40, Rgba([255, 255, 255, 255]));
        let out = apply_annotations(
            image,
            &[Annotation {
                id: "a".into(),
                kind: AnnotationKind::Highlight,
                rect: Rect {
                    x: 0.25,
                    y: 0.25,
                    w: 0.5,
                    h: 0.5,
                },
                color: Some("#ef4444".into()),
                stroke: Some(AnnotationStroke::Thick),
            }],
        );
        let edge = out.get_pixel(10, 10).0;
        assert_eq!(edge[0], 239);
        assert_eq!(edge[1], 68);
        assert_eq!(edge[2], 68);
    }

    #[test]
    fn redact_uses_stored_fill_colour() {
        let image = RgbaImage::from_pixel(20, 20, Rgba([0, 0, 0, 255]));
        let out = apply_annotations(
            image,
            &[Annotation {
                id: "a".into(),
                kind: AnnotationKind::Redact,
                rect: Rect {
                    x: 0.0,
                    y: 0.0,
                    w: 0.5,
                    h: 0.5,
                },
                color: Some("#ffffff".into()),
                stroke: None,
            }],
        );
        assert_eq!(out.get_pixel(5, 5).0, [255, 255, 255, 255]);
    }
}

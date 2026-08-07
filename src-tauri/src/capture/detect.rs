use image::RgbaImage;

/// Side length of the grayscale signature grid. 32×32 keeps a localised change
/// (a dropdown opening in one corner) from being averaged into nothing, while
/// still being tiny enough to compare thousands of times per session.
const GRID: u32 = 32;

/// Per-cell delta below which we assume nothing meaningful happened —
/// absorbs JPEG-ish noise, antialiasing and cursor trails.
const CELL_NOISE_FLOOR: u16 = 12;

/// Samples taken per axis within each grid cell. Sixteen readings per cell is
/// enough to characterise it and keeps the whole signature at ~16k pixel reads
/// regardless of display size.
const SAMPLES_PER_AXIS: u32 = 4;

/// A frame reduced to a comparable fingerprint.
pub type Signature = Vec<u8>;

/// Reduce a frame to a 32×32 luma grid.
///
/// Deliberately samples a sparse stride rather than resizing: this runs on
/// every captured frame, and a full resize of a 5K screenshot costs more than
/// the screen grab itself.
pub fn signature(image: &RgbaImage) -> Signature {
    let (w, h) = image.dimensions();
    if w == 0 || h == 0 {
        return Vec::new();
    }

    let mut out = Vec::with_capacity((GRID * GRID) as usize);
    for gy in 0..GRID {
        let y0 = gy * h / GRID;
        let y1 = (((gy + 1) * h / GRID).max(y0 + 1)).min(h);
        let step_y = ((y1 - y0) / SAMPLES_PER_AXIS).max(1);

        for gx in 0..GRID {
            let x0 = gx * w / GRID;
            let x1 = (((gx + 1) * w / GRID).max(x0 + 1)).min(w);
            let step_x = ((x1 - x0) / SAMPLES_PER_AXIS).max(1);

            let mut total = 0u32;
            let mut count = 0u32;
            let mut y = y0;
            while y < y1 {
                let mut x = x0;
                while x < x1 {
                    // Rec. 601 luma, integer math.
                    let [r, g, b, _] = image.get_pixel(x, y).0;
                    total += (r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000;
                    count += 1;
                    x += step_x;
                }
                y += step_y;
            }
            out.push((total / count.max(1)) as u8);
        }
    }
    out
}

/// How different two frames are, on a 0..1 scale.
///
/// Combines average brightness change with the *share of the grid that moved*.
/// The second term is what lets a small popup register as a real change even
/// though it barely moves the average.
pub fn distance(a: &Signature, b: &Signature) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 1.0;
    }
    let mut total = 0u32;
    let mut changed = 0u32;
    for (x, y) in a.iter().zip(b.iter()) {
        let d = (*x as i16 - *y as i16).unsigned_abs();
        total += d as u32;
        if d > CELL_NOISE_FLOOR {
            changed += 1;
        }
    }
    let mean = total as f32 / a.len() as f32 / 255.0;
    let changed_frac = changed as f32 / a.len() as f32;
    (mean * 2.5).max(changed_frac).clamp(0.0, 1.0)
}

/// Map the user-facing 0..1 sensitivity slider onto a distance threshold.
///
/// Calibrated against traces of real desktop use (`examples/probe_detect.rs`).
/// An idle screen measures around 0.001, a window opening 0.02–0.05 and a full
/// page change upwards of 0.4, so the default (0.55) sits at ~0.016: an order
/// of magnitude above the noise, well below any deliberate action.
pub fn threshold_for(sensitivity: f32) -> f32 {
    let s = sensitivity.clamp(0.0, 1.0);
    0.26 * (1.0 - s).powf(3.9) + 0.005
}

/// A frame is "settled" when it looks essentially identical to the previous
/// sample, meaning menus have finished opening and animations have landed.
pub fn is_settled(activity: f32) -> bool {
    activity < 0.012
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, v: u8) -> RgbaImage {
        RgbaImage::from_pixel(w, h, image::Rgba([v, v, v, 255]))
    }

    #[test]
    fn identical_frames_have_no_distance() {
        let a = signature(&solid(200, 200, 128));
        assert_eq!(distance(&a, &a), 0.0);
    }

    #[test]
    fn inverted_frames_are_maximally_distant() {
        let a = signature(&solid(200, 200, 0));
        let b = signature(&solid(200, 200, 255));
        assert!(distance(&a, &b) > 0.9);
    }

    #[test]
    fn a_small_corner_change_still_registers() {
        let mut img = solid(320, 320, 240);
        // Roughly 1/16th of the frame, like a dropdown opening.
        for y in 0..80 {
            for x in 0..80 {
                img.put_pixel(x, y, image::Rgba([10, 10, 10, 255]));
            }
        }
        let d = distance(&signature(&solid(320, 320, 240)), &signature(&img));
        assert!(d > threshold_for(0.55), "small change scored {d}");
    }

    #[test]
    fn sensitivity_is_monotonic() {
        assert!(threshold_for(0.0) > threshold_for(0.5));
        assert!(threshold_for(0.5) > threshold_for(1.0));
    }

    /// Locks the calibration in place: these are the change sizes the default
    /// setting is meant to sit between, expressed as a share of the frame.
    #[test]
    fn default_sensitivity_brackets_realistic_changes() {
        let default = threshold_for(0.55);
        assert!(
            (0.012..0.025).contains(&default),
            "a panel opening should register, a caret blinking should not: {default}"
        );
        // An idle screen traces at roughly 0.001; keep a wide margin over it.
        assert!(threshold_for(1.0) > 0.004, "top of the slider is too twitchy");
    }

    #[test]
    fn a_window_sized_change_registers_at_default_sensitivity() {
        let base = solid(640, 400, 235);
        let mut img = base.clone();
        // ~5% of the frame, about what a mid-sized window adds to a display.
        for y in 40..140 {
            for x in 60..188 {
                img.put_pixel(x, y, image::Rgba([20, 20, 30, 255]));
            }
        }
        let d = distance(&signature(&base), &signature(&img));
        assert!(d > threshold_for(0.55), "window-sized change scored {d}");
    }

    #[test]
    fn a_caret_sized_change_is_ignored_at_default_sensitivity() {
        let base = solid(640, 400, 235);
        let mut img = base.clone();
        for y in 100..118 {
            for x in 200..204 {
                img.put_pixel(x, y, image::Rgba([0, 0, 0, 255]));
            }
        }
        let d = distance(&signature(&base), &signature(&img));
        assert!(d < threshold_for(0.55), "caret-sized change scored {d}");
    }
}

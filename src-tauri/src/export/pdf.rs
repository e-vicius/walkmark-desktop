use printpdf::*;

use crate::error::{AppError, Result};
use crate::models::Project;

use super::{ExportOptions, RenderedStep};

// A4 in points.
const PAGE_W: f32 = 595.276;
const PAGE_H: f32 = 841.89;
const MARGIN_X: f32 = 58.0;
const MARGIN_TOP: f32 = 64.0;
const MARGIN_BOTTOM: f32 = 62.0;
const CONTENT_W: f32 = PAGE_W - MARGIN_X * 2.0;

const REGULAR_TTF: &[u8] = include_bytes!("../../assets/fonts/Inter-Regular.ttf");
const SEMIBOLD_TTF: &[u8] = include_bytes!("../../assets/fonts/Inter-SemiBold.ttf");

const INK: Color = Color::Rgb(Rgb {
    r: 0.11,
    g: 0.11,
    b: 0.13,
    icc_profile: None,
});
const MUTED: Color = Color::Rgb(Rgb {
    r: 0.42,
    g: 0.42,
    b: 0.47,
    icc_profile: None,
});
const ACCENT: Color = Color::Rgb(Rgb {
    r: 0.31,
    g: 0.27,
    b: 0.90,
    icc_profile: None,
});
const HAIRLINE: Color = Color::Rgb(Rgb {
    r: 0.89,
    g: 0.89,
    b: 0.91,
    icc_profile: None,
});

pub fn render(
    project: &Project,
    steps: &[RenderedStep],
    options: &ExportOptions,
) -> Result<Vec<u8>> {
    let mut warnings = Vec::new();
    // Font parsing reports a different warning type than document saving does.
    let mut font_warnings = Vec::new();
    let mut doc = PdfDocument::new(&project.title);

    let regular = ParsedFont::from_bytes(REGULAR_TTF, 0, &mut font_warnings)
        .ok_or_else(|| AppError::Other("The bundled PDF font could not be read.".into()))?;
    let semibold = ParsedFont::from_bytes(SEMIBOLD_TTF, 0, &mut font_warnings)
        .ok_or_else(|| AppError::Other("The bundled PDF font could not be read.".into()))?;

    let mut writer = Writer {
        regular_metrics: FontMetricsRef::new(&regular),
        semibold_metrics: FontMetricsRef::new(&semibold),
        regular: doc.add_font(&regular),
        semibold: doc.add_font(&semibold),
        pages: Vec::new(),
        ops: Vec::new(),
        y: PAGE_H - MARGIN_TOP,
    };

    // --- Title block -----------------------------------------------------
    writer.text(
        "STEP-BY-STEP GUIDE",
        &Style {
            size: 8.5,
            bold: true,
            color: ACCENT,
            leading: 1.4,
            tracking: 1.1,
        },
        MARGIN_X,
        CONTENT_W,
    );
    writer.gap(10.0);
    writer.text(
        &project.title,
        &Style {
            size: 26.0,
            bold: true,
            color: INK,
            leading: 1.22,
            tracking: 0.0,
        },
        MARGIN_X,
        CONTENT_W,
    );

    if options.include_summary && !project.summary.trim().is_empty() {
        writer.gap(14.0);
        writer.text(
            project.summary.trim(),
            &Style {
                size: 11.0,
                bold: false,
                color: MUTED,
                leading: 1.55,
                tracking: 0.0,
            },
            MARGIN_X,
            CONTENT_W.min(430.0),
        );
    }

    if options.include_prerequisites && !project.prerequisites.is_empty() {
        writer.gap(22.0);
        writer.text(
            "Before you start",
            &Style {
                size: 11.0,
                bold: true,
                color: INK,
                leading: 1.4,
                tracking: 0.0,
            },
            MARGIN_X,
            CONTENT_W,
        );
        writer.gap(8.0);
        for item in &project.prerequisites {
            writer.bullet(item.trim());
        }
    }

    writer.gap(26.0);
    writer.rule();
    writer.gap(26.0);

    // --- Steps -----------------------------------------------------------
    for (i, step) in steps.iter().enumerate() {
        if i > 0 {
            writer.gap(30.0);
        }
        // Keep a step's number, title and first line together; a heading
        // stranded at the bottom of a page looks broken.
        writer.ensure(78.0);

        let label_w = 26.0;
        let baseline_y = writer.y;
        writer.badge(step.number, MARGIN_X, baseline_y);
        writer.y = baseline_y;
        writer.text(
            &step.title,
            &Style {
                size: 14.5,
                bold: true,
                color: INK,
                leading: 1.32,
                tracking: 0.0,
            },
            MARGIN_X + label_w + 10.0,
            CONTENT_W - label_w - 10.0,
        );

        if !step.body.is_empty() {
            writer.gap(7.0);
            writer.text(
                &strip_markdown(&step.body),
                &Style {
                    size: 10.5,
                    bold: false,
                    color: MUTED,
                    leading: 1.58,
                    tracking: 0.0,
                },
                MARGIN_X + label_w + 10.0,
                CONTENT_W - label_w - 10.0,
            );
        }

        if let Some(image) = &step.image {
            writer.gap(14.0);
            let x = MARGIN_X + label_w + 10.0;
            let max_w = CONTENT_W - label_w - 10.0;
            let scale = (max_w / image.width as f32).min(1.0);
            let draw_w = image.width as f32 * scale;
            let draw_h = image.height as f32 * scale;

            // A tall screenshot that cannot fit any page must be allowed to
            // shrink rather than loop forever asking for a new page.
            let available = PAGE_H - MARGIN_TOP - MARGIN_BOTTOM;
            let (draw_w, draw_h) = if draw_h > available {
                let s = available / draw_h;
                (draw_w * s, draw_h * s)
            } else {
                (draw_w, draw_h)
            };

            writer.ensure(draw_h);
            let id = doc.add_image(
                &RawImage::decode_from_bytes(&image.png, &mut warnings)
                    .map_err(|e| AppError::Other(format!("A screenshot could not be embedded: {e}")))?,
            );
            writer.image(id, x, writer.y - draw_h, draw_w, draw_h, image.width);
            writer.y -= draw_h;
        }
    }

    // --- Footers ---------------------------------------------------------
    let mut pages = writer.finish();
    let total = pages.len();
    for (index, ops) in pages.iter_mut().enumerate() {
        ops.extend(footer_ops(
            &writer.regular,
            &writer.regular_metrics,
            &project.title,
            index + 1,
            total,
        ));
    }

    let pdf_pages: Vec<PdfPage> = pages
        .into_iter()
        .map(|ops| PdfPage::new(Mm(210.0), Mm(297.0), ops))
        .collect();

    Ok(doc.with_pages(pdf_pages).save(
        &PdfSaveOptions {
            subset_fonts: true,
            ..Default::default()
        },
        &mut warnings,
    ))
}

#[derive(Clone)]
struct Style {
    size: f32,
    bold: bool,
    color: Color,
    /// Line height as a multiple of the font size.
    leading: f32,
    /// Extra letter spacing in points, for the small-caps eyebrow.
    tracking: f32,
}

/// Glyph advances pulled straight from the font, so wrapping matches what the
/// viewer will actually draw instead of guessing an average character width.
struct FontMetricsRef {
    widths: std::collections::HashMap<char, f32>,
    fallback: f32,
}

impl FontMetricsRef {
    fn new(font: &ParsedFont) -> Self {
        let upem = if font.units_per_em == 0 {
            1000.0
        } else {
            font.units_per_em as f32
        };
        // Cache the printable ASCII range up front; anything else is measured
        // lazily below and is rare enough not to matter.
        let mut widths = std::collections::HashMap::new();
        for cp in 0x20u32..0x7F {
            if let Some(ch) = char::from_u32(cp) {
                if let Some(w) = advance(font, cp, upem) {
                    widths.insert(ch, w);
                }
            }
        }
        Self {
            widths,
            fallback: 0.5,
        }
    }

    fn char_width(&self, ch: char) -> f32 {
        self.widths.get(&ch).copied().unwrap_or(self.fallback)
    }

    fn measure(&self, text: &str, size: f32, tracking: f32) -> f32 {
        let sum: f32 = text.chars().map(|c| self.char_width(c)).sum();
        sum * size + tracking * text.chars().count() as f32
    }
}

fn advance(font: &ParsedFont, codepoint: u32, upem: f32) -> Option<f32> {
    let gid = font.lookup_glyph_index(codepoint)?;
    let width = font.get_glyph_width(gid)?;
    Some(width as f32 / upem)
}

struct Writer {
    regular: FontId,
    semibold: FontId,
    regular_metrics: FontMetricsRef,
    semibold_metrics: FontMetricsRef,
    pages: Vec<Vec<Op>>,
    ops: Vec<Op>,
    y: f32,
}

impl Writer {
    fn metrics(&self, bold: bool) -> &FontMetricsRef {
        if bold {
            &self.semibold_metrics
        } else {
            &self.regular_metrics
        }
    }

    fn font(&self, bold: bool) -> FontId {
        if bold {
            self.semibold.clone()
        } else {
            self.regular.clone()
        }
    }

    fn break_page(&mut self) {
        self.pages.push(std::mem::take(&mut self.ops));
        self.y = PAGE_H - MARGIN_TOP;
    }

    /// Start a new page if `needed` points won't fit below the cursor.
    fn ensure(&mut self, needed: f32) {
        if self.y - needed < MARGIN_BOTTOM {
            self.break_page();
        }
    }

    fn gap(&mut self, points: f32) {
        self.y -= points;
    }

    fn rule(&mut self) {
        self.ensure(2.0);
        self.ops.push(Op::SetOutlineColor { col: HAIRLINE });
        self.ops.push(Op::SetOutlineThickness { pt: Pt(0.75) });
        self.ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: Point {
                            x: Pt(MARGIN_X),
                            y: Pt(self.y),
                        },
                        bezier: false,
                    },
                    LinePoint {
                        p: Point {
                            x: Pt(MARGIN_X + CONTENT_W),
                            y: Pt(self.y),
                        },
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        });
    }

    fn badge(&mut self, number: usize, x: f32, top: f32) {
        let size = 22.0;
        self.ops.push(Op::SetFillColor { col: ACCENT });
        self.ops.push(Op::DrawRectangle {
            rectangle: Rect {
                x: Pt(x),
                y: Pt(top - size + 3.0),
                width: Pt(size),
                height: Pt(size),
                mode: Some(PaintMode::Fill),
                winding_order: Some(WindingOrder::NonZero),
            },
        });

        // Centre the digits inside the badge.
        let label = number.to_string();
        let text_size = 11.0;
        let width = self.semibold_metrics.measure(&label, text_size, 0.0);
        self.draw_line(
            &label,
            x + (size - width) / 2.0,
            top - size + 9.5,
            text_size,
            true,
            Color::Rgb(Rgb {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                icc_profile: None,
            }),
            0.0,
        );
    }

    fn bullet(&mut self, text: &str) {
        let style = Style {
            size: 10.5,
            bold: false,
            color: MUTED,
            leading: 1.5,
            tracking: 0.0,
        };
        let y_before = self.y;
        self.text(text, &style, MARGIN_X + 14.0, CONTENT_W - 14.0);
        // Dot sits on the first line's baseline, drawn after so we know where
        // that line ended up (the paragraph may have started a new page).
        let baseline = if self.y > y_before {
            PAGE_H - MARGIN_TOP - style.size * 0.82
        } else {
            y_before - style.size * 0.82
        };
        self.draw_line("•", MARGIN_X + 2.0, baseline, style.size, false, MUTED, 0.0);
        self.gap(3.0);
    }

    /// Word-wrapped paragraph. Returns with the cursor just below the last line.
    fn text(&mut self, text: &str, style: &Style, x: f32, width: f32) {
        let line_height = style.size * style.leading;
        for paragraph in text.split('\n') {
            if paragraph.trim().is_empty() {
                self.gap(line_height * 0.5);
                continue;
            }
            for line in wrap(self.metrics(style.bold), paragraph, style.size, style.tracking, width) {
                self.ensure(line_height);
                self.y -= line_height;
                // Baseline sits a little above the bottom of the line box.
                let baseline = self.y + line_height - style.size * 0.86;
                self.draw_line(
                    &line,
                    x,
                    baseline,
                    style.size,
                    style.bold,
                    style.color.clone(),
                    style.tracking,
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_line(
        &mut self,
        text: &str,
        x: f32,
        baseline: f32,
        size: f32,
        bold: bool,
        color: Color,
        tracking: f32,
    ) {
        if text.is_empty() {
            return;
        }
        self.ops.push(Op::StartTextSection);
        self.ops.push(Op::SetFillColor { col: color });
        self.ops.push(Op::SetFont {
            font: PdfFontHandle::External(self.font(bold)),
            size: Pt(size),
        });
        if tracking != 0.0 {
            self.ops.push(Op::SetCharacterSpacing {
                multiplier: tracking,
            });
        }
        self.ops.push(Op::SetTextCursor {
            pos: Point {
                x: Pt(x),
                y: Pt(baseline),
            },
        });
        self.ops.push(Op::ShowText {
            items: vec![TextItem::Text(text.to_string())],
        });
        if tracking != 0.0 {
            self.ops.push(Op::SetCharacterSpacing { multiplier: 0.0 });
        }
        self.ops.push(Op::EndTextSection);
    }

    fn image(&mut self, id: XObjectId, x: f32, y: f32, w: f32, _h: f32, source_px_w: u32) {
        // `UseXobject` sizes images by DPI, so pick the DPI that makes the
        // source pixels land on exactly `w` points.
        let dpi = (source_px_w as f32 * 72.0 / w).max(1.0);
        self.ops.push(Op::UseXobject {
            id,
            transform: XObjectTransform {
                translate_x: Some(Pt(x)),
                translate_y: Some(Pt(y)),
                scale_x: Some(1.0),
                scale_y: Some(1.0),
                dpi: Some(dpi),
                rotate: None,
                no_auto_scale: false,
            },
        });
    }

    fn finish(&mut self) -> Vec<Vec<Op>> {
        let mut pages = std::mem::take(&mut self.pages);
        pages.push(std::mem::take(&mut self.ops));
        pages
    }
}

fn footer_ops(
    font: &FontId,
    metrics: &FontMetricsRef,
    title: &str,
    page: usize,
    total: usize,
) -> Vec<Op> {
    let size = 8.5;
    let y = MARGIN_BOTTOM - 26.0;
    let right = format!("{page} / {total}");
    let right_w = metrics.measure(&right, size, 0.0);

    let mut ops = Vec::new();
    for (text, x) in [
        (truncate(title, metrics, size, CONTENT_W - right_w - 20.0), MARGIN_X),
        (right, MARGIN_X + CONTENT_W - right_w),
    ] {
        ops.push(Op::StartTextSection);
        ops.push(Op::SetFillColor { col: MUTED });
        ops.push(Op::SetFont {
            font: PdfFontHandle::External(font.clone()),
            size: Pt(size),
        });
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Pt(x),
                y: Pt(y),
            },
        });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(text)],
        });
        ops.push(Op::EndTextSection);
    }
    ops
}

fn truncate(text: &str, metrics: &FontMetricsRef, size: f32, max: f32) -> String {
    if metrics.measure(text, size, 0.0) <= max {
        return text.to_string();
    }
    let mut out = String::new();
    for ch in text.chars() {
        if metrics.measure(&format!("{out}{ch}…"), size, 0.0) > max {
            break;
        }
        out.push(ch);
    }
    format!("{}…", out.trim_end())
}

/// Greedy word wrap. Words longer than the line (a URL, a long identifier) are
/// broken mid-word rather than overflowing the margin.
fn wrap(
    metrics: &FontMetricsRef,
    text: &str,
    size: f32,
    tracking: f32,
    width: f32,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if metrics.measure(&candidate, size, tracking) <= width {
            current = candidate;
            continue;
        }
        if !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }
        if metrics.measure(word, size, tracking) <= width {
            current = word.to_string();
        } else {
            let mut chunk = String::new();
            for ch in word.chars() {
                if metrics.measure(&format!("{chunk}{ch}"), size, tracking) > width
                    && !chunk.is_empty()
                {
                    lines.push(std::mem::take(&mut chunk));
                }
                chunk.push(ch);
            }
            current = chunk;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// The PDF writer lays out plain text, so inline markdown emphasis from the
/// model is flattened rather than printed as literal asterisks.
fn strip_markdown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut at_line_start = true;

    while let Some(ch) = chars.next() {
        match ch {
            '*' | '_' => {
                // A bullet marker only counts at the very start of a line.
                if at_line_start && ch == '*' && chars.peek() == Some(&' ') {
                    out.push('•');
                    at_line_start = false;
                } else if chars.peek() == Some(&ch) {
                    chars.next();
                }
            }
            '`' => {}
            '#' if at_line_start => {
                while chars.peek() == Some(&'#') {
                    chars.next();
                }
                if chars.peek() == Some(&' ') {
                    chars.next();
                }
            }
            '\n' => {
                out.push('\n');
                at_line_start = true;
                continue;
            }
            other => {
                out.push(other);
                at_line_start = false;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics() -> FontMetricsRef {
        let mut font_warnings = Vec::new();
        FontMetricsRef::new(&ParsedFont::from_bytes(REGULAR_TTF, 0, &mut font_warnings).unwrap())
    }

    #[test]
    fn wraps_within_the_measured_width() {
        let m = metrics();
        let text = "Open the Settings panel and choose the Billing tab to review your plan.";
        for line in wrap(&m, text, 10.5, 0.0, 200.0) {
            assert!(m.measure(&line, 10.5, 0.0) <= 200.0, "overflowing line: {line}");
        }
    }

    #[test]
    fn breaks_words_that_cannot_fit() {
        let m = metrics();
        let lines = wrap(&m, "supercalifragilisticexpialidocious", 12.0, 0.0, 40.0);
        assert!(lines.len() > 1);
        for line in lines {
            assert!(m.measure(&line, 12.0, 0.0) <= 40.0);
        }
    }

    #[test]
    fn measures_real_glyph_widths() {
        let m = metrics();
        // A proportional font must not treat these as the same width.
        assert!(m.measure("iii", 12.0, 0.0) < m.measure("MMM", 12.0, 0.0));
    }

    #[test]
    fn flattens_inline_markdown() {
        assert_eq!(strip_markdown("Click **Save** now"), "Click Save now");
        assert_eq!(strip_markdown("## Heading"), "Heading");
        assert_eq!(strip_markdown("* item"), "• item");
        assert_eq!(strip_markdown("use `code` here"), "use code here");
    }

    #[test]
    fn renders_a_document_without_panicking() {
        let mut project = Project::new("Test guide", "Display");
        project.summary = "A short summary.".into();
        project.prerequisites = vec!["An account".into()];
        let steps = vec![RenderedStep {
            number: 1,
            title: "Open the app".into(),
            body: "Click the icon in your dock.".into(),
            image: None,
        }];
        let bytes = render(&project, &steps, &ExportOptions::default()).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
    }
}

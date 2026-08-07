pub mod html;
pub mod markdown;
pub mod pdf;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::error::{AppError, Result};
use crate::imaging;
use crate::models::{ExportFormat, Project};
use crate::storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub include_images: bool,
    /// Longest edge of images inside the document.
    pub image_width: u32,
    pub include_toc: bool,
    pub include_summary: bool,
    pub include_prerequisites: bool,
    /// HTML only. "light" | "dark" | "auto"
    pub theme: String,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Html,
            include_images: true,
            image_width: 1400,
            include_toc: true,
            include_summary: true,
            include_prerequisites: true,
            theme: "auto".into(),
        }
    }
}

/// A step flattened into exactly what a writer needs: final text, final pixels.
pub struct RenderedStep {
    pub number: usize,
    pub title: String,
    pub body: String,
    pub image: Option<RenderedImage>,
}

pub struct RenderedImage {
    pub png: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub path: String,
    /// Present for Markdown, which writes images alongside the document.
    pub assets_dir: Option<String>,
    pub bytes: u64,
}

/// Bake annotations, resize, and fill in the blanks that the user left empty.
///
/// Every writer goes through here so a Markdown export and a PDF export can
/// never disagree about what the document contains.
pub fn prepare(
    app: &AppHandle,
    project: &Project,
    options: &ExportOptions,
) -> Result<Vec<RenderedStep>> {
    let frames = storage::frames_dir(app, &project.id)?;
    let mut out = Vec::new();

    for step in project.steps.iter().filter(|s| s.include) {
        let number = out.len() + 1;
        let image = if options.include_images {
            let path = frames.join(&step.frame);
            match imaging::load(&path) {
                Ok(img) => {
                    let img = imaging::apply_annotations(img, &step.annotations);
                    let img = imaging::fit_width(img, options.image_width);
                    Some(RenderedImage {
                        width: img.width(),
                        height: img.height(),
                        png: imaging::encode_png(&img)?,
                    })
                }
                // A missing frame shouldn't sink the whole export; the step's
                // text is still worth having.
                Err(_) => None,
            }
        } else {
            None
        };

        out.push(RenderedStep {
            number,
            title: if step.title.trim().is_empty() {
                format!("Step {number}")
            } else {
                step.title.trim().to_string()
            },
            body: step.body.trim().to_string(),
            image,
        });
    }

    if out.is_empty() {
        return Err(AppError::Invalid(
            "This document has no steps to export. Include at least one step first.".into(),
        ));
    }
    Ok(out)
}

pub fn write(
    app: &AppHandle,
    project: &Project,
    options: &ExportOptions,
    destination: &Path,
) -> Result<ExportResult> {
    let steps = prepare(app, project, options)?;
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let assets_dir = match options.format {
        ExportFormat::Markdown => Some(markdown::write(project, &steps, options, destination)?),
        ExportFormat::Html => {
            std::fs::write(destination, html::render(project, &steps, options))?;
            None
        }
        ExportFormat::Pdf => {
            std::fs::write(destination, pdf::render(project, &steps, options)?)?;
            None
        }
    };

    Ok(ExportResult {
        path: destination.to_string_lossy().into_owned(),
        assets_dir: assets_dir.map(|p| p.to_string_lossy().into_owned()),
        bytes: std::fs::metadata(destination).map(|m| m.len()).unwrap_or(0),
    })
}

/// Suggested file name, e.g. "Creating a new invoice" -> "creating-a-new-invoice".
pub fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut last_dash = true;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "steppy-document".into()
    } else {
        trimmed
    }
}

pub fn default_file_name(project: &Project, format: ExportFormat) -> String {
    format!("{}.{}", slugify(&project.title), format.extension())
}

pub(crate) fn assets_dir_for(destination: &Path) -> PathBuf {
    let stem = destination
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "document".into());
    destination.with_file_name(format!("{stem}-images"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_are_url_safe() {
        assert_eq!(slugify("Creating a New Invoice"), "creating-a-new-invoice");
        assert_eq!(slugify("  Spaces  &  symbols!! "), "spaces-symbols");
        assert_eq!(slugify("!!!"), "steppy-document");
    }
}

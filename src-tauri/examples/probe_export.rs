//! Renders a stored project to all three formats so the output can be eyeballed
//! without driving the UI.
//!
//! `cargo run --release --example probe_export -- <project-id> [out-dir]`

use std::path::{Path, PathBuf};

use steppy_lib::probe::{
    apply_annotations, encode_png, fit_width, html, load, markdown, pdf, ExportFormat,
    ExportOptions, Project, RenderedImage, RenderedStep,
};

fn projects_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME");
    if cfg!(target_os = "macos") {
        Path::new(&home)
            .join("Library/Application Support/app.steppy.desktop/projects")
    } else {
        Path::new(&home).join(".local/share/app.steppy.desktop/projects")
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let id = args.next().unwrap_or_else(|| "demo-guide".into());
    let out = PathBuf::from(args.next().unwrap_or_else(|| "/tmp/steppy-export".into()));
    std::fs::create_dir_all(&out).expect("create out dir");

    let root = projects_dir().join(&id);
    let raw = std::fs::read_to_string(root.join("project.json")).expect("read project.json");
    let project: Project = serde_json::from_str(&raw).expect("parse project.json");
    let frames = root.join("frames");

    let options = ExportOptions::default();
    let mut steps = Vec::new();
    for step in project.steps.iter().filter(|s| s.include) {
        let number = steps.len() + 1;
        let image = load(&frames.join(&step.frame)).ok().map(|img| {
            let img = apply_annotations(img, &step.annotations);
            let img = fit_width(img, options.image_width);
            RenderedImage {
                width: img.width(),
                height: img.height(),
                png: encode_png(&img).expect("encode png"),
            }
        });
        steps.push(RenderedStep {
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
    println!("{} steps from `{}`", steps.len(), project.title);

    for format in [ExportFormat::Markdown, ExportFormat::Html, ExportFormat::Pdf] {
        let path = out.join(format!("guide.{}", format.extension()));
        let bytes = match format {
            ExportFormat::Markdown => {
                let assets = markdown::write(&project, &steps, &options, &path)
                    .expect("write markdown");
                println!("  assets → {}", assets.display());
                std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
            }
            ExportFormat::Html => {
                let doc = html::render(&project, &steps, &options);
                std::fs::write(&path, &doc).expect("write html");
                doc.len() as u64
            }
            ExportFormat::Pdf => {
                let doc = pdf::render(&project, &steps, &options).expect("render pdf");
                std::fs::write(&path, &doc).expect("write pdf");
                doc.len() as u64
            }
        };
        println!("{} → {} ({} KB)", format.extension(), path.display(), bytes / 1024);
    }
}

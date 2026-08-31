use base64::Engine;
use pulldown_cmark::{html as cmark_html, Options as CmarkOptions, Parser};

use crate::models::Project;

use super::{ExportOptions, RenderedStep};

/// Renders a single self-contained `.html` file — images inlined, styles
/// inlined, no network requests. It has to survive being emailed as an
/// attachment or dropped on a share drive.
pub fn render(project: &Project, steps: &[RenderedStep], options: &ExportOptions) -> String {
    let show_toc = options.include_toc && steps.len() > 2;

    let mut body = String::new();
    body.push_str(&format!(
        "<header class=\"doc-head\">\n<p class=\"eyebrow\">Step-by-step guide</p>\n<h1>{}</h1>\n",
        esc(&project.title)
    ));
    if options.include_summary && !project.summary.trim().is_empty() {
        body.push_str(&format!("<p class=\"lede\">{}</p>\n", esc(&project.summary)));
    }
    body.push_str(&format!(
        "<p class=\"meta\">{} step{}</p>\n</header>\n",
        steps.len(),
        if steps.len() == 1 { "" } else { "s" }
    ));

    if options.include_prerequisites && !project.prerequisites.is_empty() {
        body.push_str("<aside class=\"callout\">\n<h2>Before you start</h2>\n<ul>\n");
        for item in &project.prerequisites {
            body.push_str(&format!("<li>{}</li>\n", esc(item)));
        }
        body.push_str("</ul>\n</aside>\n");
    }

    body.push_str("<ol class=\"steps\">\n");
    for step in steps {
        body.push_str(&format!(
            "<li class=\"step\" id=\"{id}\">\n\
             <div class=\"step-head\"><span class=\"num\">{num}</span>\
             <h2>{title}</h2></div>\n",
            id = anchor(step),
            num = step.number,
            title = esc(&step.title)
        ));
        if !step.body.is_empty() {
            body.push_str(&format!("<div class=\"prose\">{}</div>\n", markdown(&step.body)));
        }
        if let Some(image) = &step.image {
            let data = base64::engine::general_purpose::STANDARD.encode(&image.png);
            body.push_str(&format!(
                "<figure><img src=\"data:image/png;base64,{data}\" alt=\"{alt}\" \
                 width=\"{w}\" height=\"{h}\" loading=\"lazy\"></figure>\n",
                alt = esc(&format!("Step {}: {}", step.number, step.title)),
                w = image.width,
                h = image.height
            ));
        }
        body.push_str("</li>\n");
    }
    body.push_str("</ol>\n");

    let toc = if show_toc {
        let items: String = steps
            .iter()
            .map(|s| {
                format!(
                    "<li><a href=\"#{}\"><span>{}</span>{}</a></li>",
                    anchor(s),
                    s.number,
                    esc(&s.title)
                )
            })
            .collect();
        format!("<nav class=\"toc\" aria-label=\"Steps\"><p class=\"toc-title\">On this page</p><ol>{items}</ol></nav>")
    } else {
        String::new()
    };

    format!(
        "<!doctype html>\n<html lang=\"en\" data-theme=\"{theme}\">\n<head>\n\
<meta charset=\"utf-8\">\n\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
<title>{title}</title>\n\
<meta name=\"description\" content=\"{desc}\">\n\
<meta name=\"generator\" content=\"Walkmark\">\n\
<style>{css}</style>\n</head>\n<body>\n\
<div class=\"layout{layout_mod}\">\n{toc}\n<main>\n{body}\n\
<footer class=\"doc-foot\">Generated with Walkmark</footer>\n</main>\n</div>\n{script}\n</body>\n</html>\n",
        theme = esc(&options.theme),
        title = esc(&project.title),
        desc = esc(project.summary.trim()),
        css = CSS,
        layout_mod = if show_toc { "" } else { " layout--solo" },
        toc = toc,
        body = body,
        script = if show_toc { SCRIPT } else { "" },
    )
}

fn anchor(step: &RenderedStep) -> String {
    super::slugify(&format!("step-{}-{}", step.number, step.title))
}

fn markdown(input: &str) -> String {
    let mut opts = CmarkOptions::empty();
    opts.insert(CmarkOptions::ENABLE_STRIKETHROUGH);
    opts.insert(CmarkOptions::ENABLE_TABLES);
    let mut out = String::new();
    cmark_html::push_html(&mut out, Parser::new_ext(input, opts));
    out
}

fn esc(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Highlights the step you're currently reading. Deliberately tiny and
/// dependency-free — the export must keep working offline forever.
const SCRIPT: &str = r#"<script>
(function () {
  var links = [].slice.call(document.querySelectorAll('.toc a'));
  var map = {};
  links.forEach(function (a) { map[a.getAttribute('href').slice(1)] = a; });
  var observer = new IntersectionObserver(function (entries) {
    entries.forEach(function (entry) {
      if (!entry.isIntersecting) return;
      links.forEach(function (a) { a.removeAttribute('aria-current'); });
      var active = map[entry.target.id];
      if (active) active.setAttribute('aria-current', 'true');
    });
  }, { rootMargin: '-10% 0px -70% 0px' });
  document.querySelectorAll('.step').forEach(function (s) { observer.observe(s); });
})();
</script>"#;

const CSS: &str = r#"
*,*::before,*::after{box-sizing:border-box}
:root{
  --bg:#ffffff; --panel:#f7f7f9; --text:#1c1c1f; --muted:#65656e;
  --line:#e5e5ea; --accent:#4f46e5; --accent-soft:#eef2ff; --shadow:0 1px 2px rgba(16,16,20,.06),0 8px 24px rgba(16,16,20,.06);
}
html[data-theme="dark"]{
  --bg:#0f1014; --panel:#17181e; --text:#ececf1; --muted:#9a9aa6;
  --line:#26272f; --accent:#8b8cf7; --accent-soft:#1c1d2e; --shadow:0 1px 2px rgba(0,0,0,.4),0 8px 24px rgba(0,0,0,.35);
}
@media (prefers-color-scheme:dark){
  html[data-theme="auto"]{
    --bg:#0f1014; --panel:#17181e; --text:#ececf1; --muted:#9a9aa6;
    --line:#26272f; --accent:#8b8cf7; --accent-soft:#1c1d2e; --shadow:0 1px 2px rgba(0,0,0,.4),0 8px 24px rgba(0,0,0,.35);
  }
}
html{-webkit-text-size-adjust:100%}
body{
  margin:0; background:var(--bg); color:var(--text);
  font:16px/1.65 ui-sans-serif,-apple-system,BlinkMacSystemFont,"Segoe UI",Inter,Roboto,Helvetica,Arial,sans-serif;
  font-feature-settings:"kern","liga"; -webkit-font-smoothing:antialiased;
}
.layout{display:grid; grid-template-columns:minmax(0,1fr); max-width:1180px; margin:0 auto; padding:48px 28px 96px; gap:56px}
@media (min-width:1040px){ .layout{grid-template-columns:248px minmax(0,1fr)} .layout--solo{grid-template-columns:minmax(0,1fr); max-width:860px} }
main{min-width:0; max-width:820px}

.toc{display:none}
@media (min-width:1040px){
  .toc{display:block; position:sticky; top:48px; align-self:start; max-height:calc(100vh - 96px); overflow:auto}
}
.toc-title{margin:0 0 12px; font-size:12px; font-weight:600; letter-spacing:.06em; text-transform:uppercase; color:var(--muted)}
.toc ol{list-style:none; margin:0; padding:0; display:flex; flex-direction:column; gap:2px}
.toc a{
  display:flex; gap:10px; align-items:baseline; padding:7px 10px; border-radius:8px;
  color:var(--muted); text-decoration:none; font-size:13.5px; line-height:1.45;
  border-left:2px solid transparent;
}
.toc a span{font-variant-numeric:tabular-nums; font-size:12px; opacity:.7; min-width:14px}
.toc a:hover{color:var(--text); background:var(--panel)}
.toc a[aria-current]{color:var(--accent); background:var(--accent-soft); border-left-color:var(--accent); font-weight:500}

.doc-head{margin:0 0 40px}
.eyebrow{margin:0 0 10px; font-size:12px; font-weight:600; letter-spacing:.08em; text-transform:uppercase; color:var(--accent)}
h1{margin:0; font-size:clamp(30px,4.4vw,42px); line-height:1.15; letter-spacing:-.02em; font-weight:680}
.lede{margin:16px 0 0; font-size:18px; line-height:1.6; color:var(--muted); max-width:62ch}
.meta{margin:20px 0 0; font-size:13px; color:var(--muted)}

.callout{margin:0 0 44px; padding:20px 22px; background:var(--panel); border:1px solid var(--line); border-radius:14px}
.callout h2{margin:0 0 10px; font-size:14px; font-weight:620; letter-spacing:.01em}
.callout ul{margin:0; padding-left:20px; color:var(--muted)}
.callout li+li{margin-top:6px}

.steps{list-style:none; margin:0; padding:0; display:flex; flex-direction:column; gap:52px; counter-reset:none}
.step{scroll-margin-top:32px}
.step-head{display:flex; gap:14px; align-items:flex-start; margin-bottom:14px}
.num{
  flex:none; width:30px; height:30px; border-radius:9px; margin-top:1px;
  display:grid; place-items:center; background:var(--accent); color:#fff;
  font-size:13.5px; font-weight:640; font-variant-numeric:tabular-nums;
}
.step h2{margin:0; font-size:21px; line-height:1.35; letter-spacing:-.011em; font-weight:640; padding-top:2px}
.prose{margin:0 0 18px 44px; max-width:64ch}
.prose p{margin:0 0 12px}
.prose p:last-child{margin-bottom:0}
.prose code{background:var(--panel); border:1px solid var(--line); border-radius:5px; padding:1px 5px; font-size:.88em}
.prose ul,.prose ol{margin:0 0 12px; padding-left:22px}

figure{margin:0 0 0 44px}
figure img{
  display:block; width:100%; height:auto; border-radius:12px;
  border:1px solid var(--line); box-shadow:var(--shadow); background:var(--panel);
}
@media (max-width:640px){ .prose,figure{margin-left:0} }

.doc-foot{margin-top:72px; padding-top:24px; border-top:1px solid var(--line); font-size:13px; color:var(--muted)}

@media print{
  :root{--bg:#fff; --text:#111; --muted:#444; --line:#ddd; --panel:#fafafa; --shadow:none}
  .toc{display:none}
  .layout{display:block; max-width:none; padding:0}
  .steps{gap:0}
  .step{break-inside:avoid; page-break-inside:avoid; padding:18px 0}
  figure img{border-color:#ddd}
  a{color:inherit; text-decoration:none}
}
"#;

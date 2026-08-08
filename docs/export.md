# Export

Open **Export** from the document toolbar (**⌘E** / **Ctrl+E**) when the guide reads right.

## Formats

### Markdown

A `.md` file plus an `images/` folder beside it. Image links are relative paths.

Best for: Git repos, wikis, static site generators, Notion import, docs-as-code workflows.

### Web page (HTML)

One **self-contained** `.html` file with images embedded as base64. No external files, no CDN — works offline and in email attachments.

Best for: sharing with someone who does not have Steppy, internal wikis that accept HTML upload, quick review in a browser.

### PDF

Print-ready **A4** document with page numbers. Steps avoid awkward page breaks where possible.

Best for: handbooks, compliance packets, printed runbooks, sign-off documents.

## Export options

| Option | Default | Applies to |
| --- | --- | --- |
| **Include images** | On | Markdown (always embedded in HTML/PDF) |
| **Image width** | Standard (1400 px) | Compact 900 / Standard 1400 / Large 1920 |
| **Table of contents** | On | HTML (when more than two steps) |
| **Summary** | On | All formats |
| **Prerequisites** | On | All formats |
| **Theme** | Auto | HTML only — light, dark, or follow system |

After export, Steppy can:

- **Reveal in Finder / Explorer** — open the containing folder
- **Copy Markdown to clipboard** — paste into Slack, Notion, etc.

## What gets exported

Only **included** steps (eye icon open) appear. Excluded steps and their frames are omitted.

| Content | Exported as |
| --- | --- |
| Title, summary, prerequisites | Document header |
| Step titles and bodies | Numbered steps |
| Screenshots | With annotations burned in |
| Locked hand-edited text | Exactly as you wrote it |

### Unwritten steps

Steps without body text still export if included — placeholders may appear. The export dialog warns how many steps are incomplete (amber notice).

## Workflow tips

- Run **Write** before export for polished prose, or export skeleton + screenshots for manual editing elsewhere.
- Use **HTML** for one-file sharing; **Markdown** when you need editable source in Git.
- Use **Compact** image width for Slack or narrow wiki columns; **Large** for print-like HTML.
- Redact in the image editor first — exports reflect annotations, not raw frames.

See [Editing guides](editing.md) for exclude, annotations, and locked steps.

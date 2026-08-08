# Editing guides

The editor has a **step rail** on the left and the document on the right. Click a step in the rail to scroll to it; click a step card in the document to select it in the rail.

## Document header

At the top of the document:

| Field | Purpose |
| --- | --- |
| **Title** | Guide name (library card and exports) |
| **Summary** | Short overview for readers |
| **Before you start** | Prerequisites — only shown when filled |

AI fills these on write; edit them anytime.

## Step text

Each step card has:

- **Title** — short imperative line (e.g. "Open Billing settings")
- **Instructions** — body text for that step
- **Screenshot** — the captured frame with optional annotations

Click any title or instruction field to edit by hand. Changes save automatically.

### Locked steps

Steps you edit manually are **locked** (lock icon). When you run **Rewrite everything**, locked steps are skipped so your fixes are not overwritten.

Use the **sparkles** button on one step to rewrite just that step — locked or not, depending on whether you changed it after the last write.

### Write states

| State | Meaning |
| --- | --- |
| Empty | Not written yet |
| Writing | AI in progress (skeleton loader) |
| Ready | Text present |
| Error | Provider failed — message on card; retry from sparkles or full rewrite |

## Selection and bulk actions

Select steps in the rail or document:

| Input | Action |
| --- | --- |
| Click | Select one |
| **⌘** / **Ctrl + click** | Toggle in selection |
| **Shift + click** | Select range |

When multiple steps are selected, a floating bar offers:

| Action | Effect |
| --- | --- |
| **Merge** | Combine selected steps into one (one screenshot, merged context) |
| **Exclude** | Hide from document and export (frame kept on disk) |
| **Delete** | Remove steps permanently |

There is no **split** — if auto-detection merged actions you wanted separate, delete or re-record.

## Reorder

Drag steps in the rail. Step numbers update in the exported guide.

## Exclude steps

Click the **eye** icon to exclude a step from the document and export without deleting the frame. Useful for dead ends, mistakes, or false captures you might want later.

Excluded steps do not count toward **Write N steps** or export.

## Annotations

Open the **image editor** on a step screenshot (pencil icon or click the image):

| Tool | Shortcut | Effect |
| --- | --- | --- |
| **Select** | V | Move or delete regions |
| **Blur** | B | Pixelate sensitive details |
| **Cover** | R | Solid fill over a region |
| **Highlight** | H | Coloured box around a control |

Annotations are **burned into exported images**. Blur secrets before running **Write** so the model does not quote them in text.

Highlight and Cover support colour and stroke options in the toolbar.

## Pick a different moment

If a step fired too early or late, and **alternate** frames exist from the same recording, click **Different moment** on the screenshot. Pick the frame that best shows the step.

Alternates are captured automatically when Steppy samples near the same timestamp — you do not manage them manually during recording.

## Toolbar actions

| Control | Action |
| --- | --- |
| **Model picker** | Provider + model for this document |
| **Write N steps** | Generate text for steps that are empty |
| **Rewrite** | Regenerate all unlocked steps |
| **Export** | Markdown, HTML, or PDF |

## Product vocabulary

In **Settings → Writing**, under **Products**, define product-specific terms:

- **Term** — word the writer should use (*Workspace*, *Billing portal*)
- **What it is** — short definition

Set a **Default product** for new recordings. The writer injects vocabulary for the product tagged on the guide.

See [Settings → Writing](settings.md).

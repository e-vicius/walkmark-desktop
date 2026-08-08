# App screenshots

PNG previews for the GitHub README and docs. They are rendered from static HTML mocks in [`scripts/screenshots/`](../../scripts/screenshots/) that match the Steppy UI (library, editor, export, and the record → edit → export flow).

Regenerate after UI changes:

```bash
pnpm install   # needs playwright
pnpm capture:screenshots
```

Optional: run `python3 scripts/seed_demo.py` first if you also refresh marketing assets that use real demo project frames.

| File | Shows |
| --- | --- |
| `library.png` | Guide library |
| `editor.png` | Editor with AI-written steps |
| `export.png` | Export dialog |
| `step-record.png` | Recording a workflow |
| `step-tidy.png` | Editing steps and blur |
| `step-export.png` | Finished exported guide |

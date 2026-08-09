# Contributing to Steppy

Thanks for helping improve Steppy.

## Before you start

- Search [existing issues](https://github.com/e-vicius/steppy-desktop/issues) so work is not duplicated.
- For large features, open an issue first to discuss approach.
- Bug reports should include OS version, Steppy version, and steps to reproduce.

## Development setup

See [docs/build-from-source.md](docs/build-from-source.md).

```bash
git clone https://github.com/e-vicius/steppy-desktop.git
cd steppy
pnpm install
pnpm tauri dev
```

The landing site at [steppy-landing](https://github.com/e-vicius/steppy-landing) imports docs from `docs/` when built with both repos checked out side by side.

## Architecture overview

Read [docs/architecture.md](docs/architecture.md) before touching capture, AI, or export. Summary:

- **Frontend:** Svelte 5 + TypeScript in `src/`
- **Backend:** Rust in `src-tauri/src/`
- **IPC:** Tauri commands in `commands.rs`, events pushed to the UI

Adding a cloud provider? Follow the checklist in the architecture doc (`catalog.rs`, `provider/`, `tauri.conf.json` CSP, `types.ts`).

## Code style

- Match the surrounding code. Rust: `cargo fmt`, `cargo clippy`.
- Frontend: Svelte 5 runes, TypeScript, Tailwind utility classes.
- Keep diffs focused. Prefer fixing one thing well over sweeping refactors.

## Tests

```bash
pnpm typecheck
cargo test --manifest-path src-tauri/Cargo.toml
```

Add tests when behaviour is easy to regress (parsing, export formatting, storage). Live provider tests belong behind env vars (`MISTRAL_API_KEY`, etc.).

## Documentation

User and developer docs live in `docs/`. Update them when behaviour changes — the marketing site renders the same Markdown.

| Audience | Examples |
| --- | --- |
| Users | `getting-started.md`, `recording.md`, `troubleshooting.md` |
| Developers | `architecture.md`, `build-from-source.md` |

## Pull requests

1. Fork and branch from `main`.
2. Describe what changed and why in the PR body.
3. Note manual testing (platform, recording, write, export).
4. Ensure CI passes.

## Licence

By contributing, you agree your contributions are licensed under the MIT licence.

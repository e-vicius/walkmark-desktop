# Build from source

Compile Steppy locally for development, custom builds, or platforms without a prebuilt release.

## Requirements

| Tool | Version |
| --- | --- |
| **Rust** | 1.82+ |
| **Node.js** | 18+ |
| **pnpm** | Latest (via `corepack enable`) |
| **Tauri prerequisites** | [v2.tauri.app/start/prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS |

**Platforms:** macOS 11+, Windows 10+ (x64 or ARM64), Linux with working screen capture (X11 or Wayland portal).

## Clone and install

```bash
git clone https://github.com/e-vicius/steppy-desktop.git
cd steppy
pnpm install
```

## Development

```bash
pnpm tauri dev
```

Hot-reloads the Svelte frontend; Rust changes rebuild the backend. Use **New guide** and a short recording to verify capture on your machine.

### Useful scripts

```bash
pnpm typecheck          # Svelte/TS check
pnpm build              # Frontend production build only
```

## Release build

```bash
pnpm tauri build
```

Artifacts appear under `src-tauri/target/release/bundle/` (native target) or `src-tauri/target/<triple>/release/bundle/` when cross-compiling.

### Cross-compilation examples

```bash
# macOS Apple Silicon
pnpm tauri build --target aarch64-apple-darwin

# macOS Intel
pnpm tauri build --target x86_64-apple-darwin

# Windows x64
pnpm tauri build --target x86_64-pc-windows-msvc

# Windows ARM64 (NSIS only — no MSI on this target)
pnpm tauri build --target aarch64-pc-windows-msvc --bundles nsis
```

### Output artifacts

| OS | Bundles |
| --- | --- |
| macOS | `.app`, `.dmg` |
| Windows x64 / ARM64 | NSIS `.exe` installer |
| Linux | `.deb`, `.AppImage` |

Release filenames follow `Steppy_<version>_<arch>.<ext>` — see `.github/workflows/release.yml` for CI matrix.

## Tests

```bash
pnpm typecheck
cargo test --manifest-path src-tauri/Cargo.toml
```

| Test | Purpose |
| --- | --- |
| `tests/wire.rs` | Provider HTTP against a local stub server |
| `tests/live_mistral.rs` | Optional live Mistral test — needs `MISTRAL_API_KEY` |
| Unit tests in `storage`, `catalog`, export | Persistence and formatting |

## Project layout

```
src/                      Svelte 5 UI (runes, TypeScript, Tailwind)
  lib/api.ts              Tauri invoke wrappers
  lib/store.svelte.ts     App state, events, dialogs
  lib/components/         Library, editor, settings, export
src-tauri/src/
  commands.rs             IPC command handlers
  capture/                Recording session, input, detection
  ai/                     Prompts, generation, catalog
  export/                 Markdown, HTML, PDF
  storage.rs              Projects, settings, credentials
docs/                     User and developer documentation
```

Deep dive: [Architecture](architecture.md).

## Release process (maintainers)

Documented in [RELEASE.md](https://github.com/e-vicius/steppy-desktop/blob/main/RELEASE.md). Summary:

1. Bump version in `src-tauri/tauri.conf.json` and `package.json`.
2. Update [CHANGELOG.md](https://github.com/e-vicius/steppy-desktop/blob/main/CHANGELOG.md).
3. Tag `vX.Y.Z` and push — GitHub Actions builds platform artifacts.
4. Attach or verify release assets on GitHub Releases.

## Contributing

Bug reports and pull requests welcome on [GitHub](https://github.com/e-vicius/steppy-desktop). See [CONTRIBUTING.md](https://github.com/e-vicius/steppy-desktop/blob/main/CONTRIBUTING.md).

Licence: MIT.

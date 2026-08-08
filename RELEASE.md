# Publishing a release

## 1. Bump version

Update version in all three places:

- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml` (`[package].version`)

Also bump `APP_VERSION` in [steppy-landing `src/lib/downloads.ts`](https://github.com/e-vicius/steppy-landing/blob/main/src/lib/downloads.ts).

## 2. Update changelog

Add a section to `CHANGELOG.md`.

## 3. Tag and push

```bash
git tag v0.1.0
git push origin v0.1.0
```

The [Release workflow](.github/workflows/release.yml) builds macOS (Apple Silicon + Intel), Windows (x64 + ARM64), and Linux installers and attaches them to the GitHub Release.

## 4. Local build (one platform)

```bash
pnpm install
pnpm tauri build
```

Artifacts land in `src-tauri/target/release/bundle/`:

| Platform | Typical files |
| --- | --- |
| macOS arm64 | `dmg/Steppy_<version>_aarch64.dmg` |
| macOS x64 | `dmg/Steppy_<version>_x64.dmg` |
| Windows x64 | `nsis/Steppy_<version>_x64-setup.exe` |
| Windows ARM64 | `nsis/Steppy_<version>_arm64-setup.exe` |
| Linux | `appimage/Steppy_<version>_amd64.AppImage`, `deb/steppy_<version>_amd64.deb` |

## 5. Verify downloads

Open the [latest release](https://github.com/e-vicius/steppy/releases/latest) and confirm filenames match `src/lib/downloads.ts` on the landing site.

## Code signing (optional)

Installers are unsigned by default. For public distribution without Gatekeeper warnings, configure Apple Developer signing and notarization in Tauri’s bundle settings before tagging.

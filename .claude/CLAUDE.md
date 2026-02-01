# Claude Code Instructions

## Dependencies

**Always check for latest crate versions before adding dependencies.** Use web search to verify current versions on crates.io rather than relying on memory, as version information may be outdated.

### CRITICAL: Update Dockerfile After Installing Tooling or Dependencies

**After successfully installing ANY system package or build tool, IMMEDIATELY update `.devcontainer/Dockerfile` before proceeding with other work.**

Workflow:

1. Install the tool or dependency (experiment with approaches as needed)
2. Once installation succeeds, **STOP** and update the Dockerfile with the working command
3. Only then continue with the task that required the dependency

This applies to: `apt-get install`, `pip install`, `npm install -g`, `cargo install`, etc.

Skipping step 2 creates "works on my machine" problems for other developers and CI.

## Project Notes

- Package manager: pnpm
- Styling: Svelte Material UI (SMUI)
- App identifier: `net.kurowski.potasko`

## Architecture

- Thin wrapper pattern: Tauri commands call `core::*` functions
- CLI binary (`potasko`) shares core logic with Tauri
- See `docs/PLAN.md` for current phase status

## Android Safe Areas

**When adding or modifying fixed/positioned UI elements, always account for safe areas.**

Android devices (phones, tablets, desktop mode) may have notification bars, camera cutouts, or gesture navigation that can overlap UI elements. Use CSS environment variables:

- `env(safe-area-inset-top, 0)` - status bar, camera cutouts
- `env(safe-area-inset-bottom, 0)` - gesture navigation bar
- `env(safe-area-inset-left, 0)` / `env(safe-area-inset-right, 0)` - side cutouts (landscape)

Examples:
- Fixed app bars: `top: env(safe-area-inset-top, 0)`
- Fixed bottom FABs: `bottom: calc(24px + env(safe-area-inset-bottom, 0))`
- Spacers below fixed headers: `height: calc(48px + env(safe-area-inset-top, 0))`

These insets automatically return `0` when not needed (tablets without notches, windowed desktop mode), so they're safe to use universally.

The `viewport-fit=cover` meta tag in `app.html` enables these insets. The 768px responsive breakpoint handles layout differences between phone-sized and tablet/desktop-sized screens.

## Distribution

- Linux: Flatpak (for Flathub distribution)
- Flatpak manifest: `net.kurowski.potasko.yml`

### Releases

**Keep release tags and app version numbers in sync.** Before tagging a release (e.g., `v0.3.1`), update the version in:

1. `src-tauri/tauri.conf.json` (primary - also generates Android version on build)
2. `src-tauri/Cargo.toml`
3. `package.json`
4. `src-tauri/Cargo.lock` (run `cargo check` in src-tauri to update, then commit)

**Write release notes for every release.** After CI builds complete, update the release notes using `gh release edit`. Include:
- Summary of user-facing changes
- Link to full changelog (e.g., `https://github.com/kurowski/potasko/compare/v0.3.3...v0.3.4`)

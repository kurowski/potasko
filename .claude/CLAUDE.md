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
- Styling: Plain CSS (no Tailwind/frameworks)
- App identifier: `net.kurowski.potasko`

## Architecture

- Thin wrapper pattern: Tauri commands call `core::*` functions
- CLI binary (`potasko`) shares core logic with Tauri
- See `docs/PLAN.md` for current phase status

## Distribution

- Linux: Flatpak (for Flathub distribution)
- Flatpak manifest: `net.kurowski.potasko.yml`

### Releases

**Keep release tags and app version numbers in sync.** Before tagging a release (e.g., `v0.3.1`), update the version in:

1. `src-tauri/tauri.conf.json` (primary - also generates Android version on build)
2. `src-tauri/Cargo.toml`
3. `package.json`

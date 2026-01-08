# Claude Code Instructions

## Dependencies

**Always check for latest crate versions before adding dependencies.** Use web search to verify current versions on crates.io rather than relying on memory, as version information may be outdated.

## Project Notes

- Package manager: pnpm
- Styling: Plain CSS (no Tailwind/frameworks)

## Architecture

- Thin wrapper pattern: Tauri commands call `core::*` functions
- CLI binary (`potasko`) shares core logic with Tauri
- See `docs/PLAN.md` for current phase status

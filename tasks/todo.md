# CClineBar — Tasks

## Completed (this session)

- [x] Add ModelTime segment (model generation time per session)
- [x] Enhance Cost segment with monthly tracking (`$session / $monthly`)
- [x] Remove token count from context window, show % only
- [x] Change context % to "remaining until auto-compact" matching Claude Code's formula
- [x] Use `context_window` JSON from Claude Code with exact `Yl()` formula
- [x] Add `--patch-context` flag for context-only patching
- [x] Auto-patch context low warning on first render (silent, marker-based)
- [x] Optimize auto-patch: store path+mtime in marker, skip directory scan on subsequent renders
- [x] Make cost tracking always run (regardless of cost segment display)
- [x] Add `show_icons` style option to hide all segment icons
- [x] Update README with all new features, cost caveats, requirements

## Notes

- GitHub push was failing (account suspended) — resolved by another session
- Context low auto-patch requires session restart to take effect (cli.js is loaded into memory at session start)
- Monthly cost only tracks sessions after ccline installation — no retroactive backfill

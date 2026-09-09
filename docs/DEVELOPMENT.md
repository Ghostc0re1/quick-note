# Local development baseline

## Start here

1. Open the project root in your editor or start Codex from this directory.
2. Read `AGENTS.md`; code beneath `src/` also follows `src/AGENTS.md`.
3. Review and trust `.codex/hooks.json` in Codex via `/hooks` before relying on
   its safety checks.
4. Quick Note uses Node.js for the minimal TypeScript UI and the stable Rust
   toolchain for Tauri and SQLite. Install both before running `npm install`.

## Baseline conventions

- Keep source in `src/`, documentation in `docs/`, and generated output out of Git.
- Put examples and non-secret configuration templates in version control; keep
  actual `.env` files local.
- Use `npm install` to restore the project-locked frontend dependencies.
- Use `npm run tauri dev` to run the ordinary unpackaged desktop app. Use
  `npm run dev:identity` on Windows 11 to run a package-identity development
  build, and `npm run msix:validate` to create an unsigned development MSIX.
- Use `npm run check`, `npm run test`, `npm run rust:fmt`, `npm run rust:clippy`,
  `npm run rust:test`, and `npm run msix:validate` for focused verification.
- Install Microsoft WinApp CLI with `winget install Microsoft.winappcli --source
  winget`; it is only needed for package-identity development and MSIX builds.
- Store identity values belong in GitHub repository variables after Partner
  Center assigns them. Never place Partner Center secrets or development
  certificates in the repository.
- Record a reproducible verification command alongside each new build or test workflow.

## Safety harness

The repository-local hook only covers commands run through Codex's supported
`PreToolUse` path. It blocks broad deletion, destructive Git cleanup, disk/data
erasure commands, direct persistent user `PATH` changes, and hook-trust bypasses.
Normal installer commands are left available. See `.codex/hooks/README.md` for
activation and limitations.

When an operation truly needs an exception, stop, name the exact target and
recovery plan, and obtain explicit approval. Do not disable the hook as a shortcut.

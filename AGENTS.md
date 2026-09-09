# Development Harness

## Scope

These instructions apply to the repository root and every directory below it.
More-specific `AGENTS.md` files refine this guidance for their subtree. Read the
nearest applicable instructions before editing.

## Working agreement

- Work in small, reviewable changes. Inspect the relevant files first and preserve
  unrelated user changes.
- Keep application code in `src/`, tests near their code or in the project's
  conventional test directory, and durable developer notes in `docs/`.
- Do not add generated output, local secrets, caches, IDE state, or machine-specific
  paths to source control.
- When changing behavior, run the narrowest relevant verification and report what
  was run and what remains unverified.
- Prefer documented project scripts over ad-hoc commands. Add a short document when
  a repeatable workflow or decision would otherwise live only in chat history.

## Safety boundaries

- Never delete or recursively overwrite a broad target (the repository root,
  `.`, `..`, a drive root, a home/profile directory, or a wildcard) without an
  explicit user request that names the exact target.
- Do not run destructive Git commands such as `git reset --hard`, `git clean -f`,
  or broad `git restore` / `git checkout` operations unless the user explicitly
  asks for that recovery action.
- Do not format disks, alter partitions, clear system state, or use commands meant
  to erase data.
- Do not persistently change the user's `PATH` directly (`setx PATH`, registry
  edits, or `SetEnvironmentVariable("Path", ...)`). Normal package-install commands
  are allowed; explain any installation that may make a tool globally available.
- Do not weaken, bypass, or silently edit the Codex hook policy in `.codex/` to
  complete a task. Propose a policy change and wait for approval instead.
- Treat `.env`, credentials, tokens, private keys, and user-profile data as
  sensitive. Do not print, commit, upload, or copy them without explicit approval.

## Before handing off

- Check the working-tree diff for accidental generated files, secrets, and unrelated edits.
- Keep README and `docs/` instructions true to the current repository state.

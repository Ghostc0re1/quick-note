# Source-area instructions

This file applies only within `src/` and extends the repository-root guidance.

- Keep production code focused on application behavior. Put tooling, one-off
  experiments, and durable process notes outside `src/`.
- Prefer explicit interfaces, narrow modules, and clear error paths over clever
  abstractions. Do not introduce a framework or dependency without a concrete need.
- Preserve public behavior unless the task explicitly calls for a change. Add or
  update a focused test when the project has a test harness.
- Avoid reading configuration, environment variables, or filesystem paths at module
  import time; make those dependencies explicit where practical.
- Never embed credentials, personal paths, host names, or environment-specific
  constants in source files.
- Leave a short comment only when it explains a non-obvious constraint or tradeoff.

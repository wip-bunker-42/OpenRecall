# OpenRecall development rules

- Keep product UI Arabic, dark and RTL. Use English identifiers and development prose.
- The Rust core is authoritative for scheduling, form selection, history, training,
  validation and transactions. Do not add a frontend scheduler or cloud dependency.
- Preserve FSRS-7's stability, stability_fast and difficulty. Changing the engine
  source/model requires separately verified scheduling and optimizer goldens.
- Grade atomically with action-token retry safety and complete one-step undo.
- Shared training scope is explicit profile membership, with data sufficiency,
  chronological holdout and stale-result rejection enforced in Rust.
- Imported content is plain text. Native IPC accepts no arbitrary file paths,
  shell commands or update URLs. Updates are explicit and signature/version verified.
- Use isolated temporary libraries in automated tests; never use real study data.
- Never launch, operate or configure NVDA, Narrator or another screen reader.
  Agent accessibility checks are automated; the human partner performs NVDA tests.
- Follow applicable Superpowers skills. Keep changes focused, use behavioral TDD,
  and provide fresh verification evidence before completion claims.
- Run `pnpm verify` and the relevant browser/native tests. Generated TypeScript
  contracts come from Rust via `pnpm contracts`.
- Keep credentials and private signing keys outside Git and the checkout. Production
  installers must omit the automation feature and all native test overrides.
- Report actual runtime/platform/update testing accurately. A build is not a runtime test.

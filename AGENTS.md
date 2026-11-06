# AGENTS.md

Entry point for every agent working in this repository. Keep it short.

All repository text, code, comments, commit messages, issues, and PRs are in
English. Priorities: security > correctness > simplicity > speed of delivery.

## About

`pohunekd/plugin-cline` is the Pohunek runtime plugin for the Cline CLI
(Cline Bot Inc., Apache-2.0): package `pohunek.runtime.cline`, runtime id
`cline`. It carries data and evidence only (descriptor, detect rules,
captures). Handlers, reporters and parsers stay in the core repository
`zajca/pohunek` (`docs/knowledge/guides/plugin-repositories.md`). The plan is
the epic #1; this repository's issues are P0-P5.

## Rules

- **No credentials, no real model, ever.** CI, `scripts/capture` and any later
  smoke run use `scripts/model-stub` on loopback with a placeholder key. Never
  sign in to Cline, ClinePass or ChatGPT, never store or print a credential.
- **Disposable environment only.** Cline runs only below `--root` of
  `scripts/capture`: a throwaway HOME, `CLINE_DIR` and `CLINE_DATA_DIR` per
  scenario, inside user, network, mount, UTS and PID namespaces with only a
  loopback interface. Never run it with a real HOME, never install it
  system-wide, never use sudo or containers.
- **Never touch another Cline hub.** A hub daemon of the operator's own Cline
  may run on the host (default port 25463). The kit never connects to it,
  never stops it, and never reads its discovery record; its own hubs run only
  inside the namespace on a private port and end with it.
- **Telemetry.** No env opt-out exists; every throwaway `global-settings.json`
  gets `"telemetryOptOut": true`, and the namespace has no route out.
- **Evidence is data, not prose.** Everything under `compat/` comes from
  `scripts/capture` on the release named in `compat/RELEASE`, redacted and
  scanned by the script. Do not edit captures by hand; re-run the script.
- **Mark what is unverified.** A claim without a capture says `unverified`
  (or `source only`, `prior art only`) in README and in the issue comment that
  records it.
- **Docs in the same PR** as the behaviour they describe.
- **Testing policy.** Minimal tests: only for non-trivial logic and regressions
  in it. CI runs shellcheck, the stub syntax check and the capture dry run
  (redaction and stub self tests); the CLI itself is not run in CI. No coverage
  targets.
- **PR bodies** use `Refs #N` only, never a closing keyword. No `Co-Authored-By`
  trailer, no generated-by footer.
- **Comments** state what the code does or why, never the change history.
- Never read `.env*`, key, or credential files, nor the operator's `~/.cline`.
  Never call `env` or `printenv`; check a variable with `test -n "$VAR"`.

## Commands

```bash
scripts/capture --root <dir> --dry-run all        # plan, tool check, self tests; no network
scripts/capture --root <dir> all                  # fetch the release and capture everything
scripts/capture --root <dir> --release X.Y.Z all  # capture a newer release
scripts/capture --root <dir> hooks errors         # single steps (after install)
```

`--root` must not be inside the repository's `compat/` (the kit refuses it);
`.capture/` (the default) is ignored by git. A step with a missed expectation
leaves `compat/` unchanged and fails the run; fix the cause and re-run it.

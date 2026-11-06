# plugin-cline

Pohunek runtime plugin for the [Cline CLI](https://github.com/cline/cline)
terminal coding agent.

| | |
|---|---|
| Repository | `pohunekd/plugin-cline` |
| Package id | `pohunek.runtime.cline` |
| Runtime id (`--agent`) | `cline` |
| Upstream | Cline CLI by Cline Bot Inc. (Apache-2.0), npm package `cline` |
| Status | **P1, descriptor**: `package/` (launch, status, identity, resume) for Cline 3.0.74 on Linux x86_64, proven against the P0 corpus. Hub routing is not controlled until P2, so the package is not admitted to the plugin index yet |

The plugin carries data and evidence only (descriptor, detect rules, captures).
Handlers, reporters and parsers stay in the core repository
[`zajca/pohunek`](https://github.com/zajca/pohunek); the plan is the epic #1
and the issues P0-P5 of this repository.

## What is here

| Path | Content |
|---|---|
| `package/` | The runtime package: `runtime.toml` (launch contract) and `detect.toml` (process and screen rules) |
| `kit/` | Stand-in for the plugin kit: a test crate that builds the package twice, installs it through core's definition path, reads the captured version banner and process trees, and runs the detection manifest on every screen of `compat/screens`, as a whole and rule by rule |
| `scripts/smoke` | End-to-end check against a pohunek build and an installed Cline: isolated test daemon in a loopback-only namespace, launch, input, status, approval refusal, resume |
| `scripts/capture` | Capture kit (bash): fetches a release from the npm registry, checks its integrity, runs Cline in throwaway homes inside loopback-only namespaces and records what it shows, which hooks it runs and what it stores |
| `scripts/model-stub` | Loopback OpenAI-compatible chat-completions stub (Python standard library): scripted text, tool calls and HTTP errors |
| `scripts/hook-log` | Cline file hook that logs one hook call: payload, environment names, `CLINE_*`/`CAPTURE_*` values, ancestor processes |
| `compat/RELEASE` | The release every capture comes from: version, npm integrity of both packages, binary sha256, the older release the hub step runs beside it |
| `compat/install/` | `--version`, npm layout, postinstall cache, process trees, settings after a start, telemetry self check |
| `compat/help/` | `--help` of every command |
| `compat/screens/<scenario>/<width>.txt` | Terminal screens; `INDEX.tsv` lists each scenario and width |
| `compat/terminal/` | Terminal titles, OSC kinds, DEC modes, BEL count, the startup bytes |
| `compat/hooks/<scenario>/` | One JSON file per hook call, numbered in arrival order, and `SUMMARY.tsv` (event, hookName, rootSessionId, agent ids, ancestor comms). Hooks of concurrent agents can arrive in either order: a subagent's `TaskComplete` and its parent's `PostToolUse` are swapped between this capture and the previous one |
| `compat/sessions/<scenario>/` | The `sessions` table (JSON columns parsed), team tables with rows, session manifests, the envelope of every messages file, the store tree |
| `compat/hub/` | Hub daemon process trees, runtime routing lines of the CLI log, hub start and stop output |
| `compat/headless/` | NDJSON of a `--json` run |
| `compat/stub/` | The stub's request logs (reply served, tool names offered; no message content) |

## No credentials, no real model, no network

- **Model turns** go to `scripts/model-stub` on 127.0.0.1, configured with
  `cline auth -p openai -k capture-placeholder-key -m stub-model -b http://127.0.0.1:<port>/v1`.
  The key is a placeholder; the kit never signs in, and the sign-in screens
  start inside the namespace, where no request leaves the host.
- **Isolation.** Only the `install` step uses the network (the npm registry
  over HTTPS). Every other step runs in user, network, mount, UTS and PID
  namespaces: only a loopback interface, the root bound to
  `/run/capture-root`, a synthetic passwd that names the caller `capture`,
  host name `capture`, and an empty tmpfs over the operator's `~/.cline`
  (checked inside). Every process of the run, a hub daemon included, ends
  with the namespace.
- **Per scenario** the CLI gets a cleared environment (`env -i`) with its own
  `HOME`, `CLINE_DIR`, `CLINE_DATA_DIR`, `TMPDIR`, `CLINE_HUB_PORT` (27463 inside
  the namespace) and `CLINE_HUB_DISCOVERY_PATH`, plus `CLINE_NO_AUTO_UPDATE=1`.
  No `POHUNEK_*` or agent-session variable reaches it. Every scenario except
  the hub ones runs in sandbox mode (`CLINE_SANDBOX=1`,
  `CLINE_SANDBOX_DATA_DIR=<its data root>`), which keeps the session in the
  TUI process.
- **The operator's hub.** The namespace cannot reach a hub on the host's
  loopback (the kit checks that 127.0.0.1:25463 is unreachable inside it).
  After the run the kit fails when a hub daemon whose command line names the
  capture root survives, or when a hub daemon that ran before the capture is
  gone; it never signals a process outside the namespace.
- **Telemetry.** Cline has no env opt-out; each throwaway
  `global-settings.json` starts with `"telemetryOptOut": true` and
  `"autoUpdateEnabled": false`, and the namespace has no route out.
- **Evidence.** The capture root becomes `/capture-root____` (same width).
  Ids Cline generates (session ids, `conv_*`, UUIDs, git hashes, teammate task
  suffixes, also in the teammate's messages file name, team names) become
  stable placeholders of the same shape, so within one run of the kit one id
  reads the same in every file (the map starts over with each run, so a step
  re-run on its own numbers its ids afresh); the kit's own assigned ids
  (`c0ffee00-...`) stay. `providers.json` and the hub discovery record are
  never copied (only the record's key names). Before a step's files replace
  those in `compat/`, a scan refuses host paths, user and host names (also
  wrapped across rows or shortened), credential-shaped fields, the
  placeholder key and `POHUNEK_`.
- **Checked evidence.** A screen is saved only after the state it should
  show matched (the saved text is the text that matched); every scripted
  reply, hook event and state a step relies on is checked. A step with any
  miss keeps `compat/` as it was, lists the misses in
  `<root>/stage/<step>/MISSED.txt`, and the run exits non-zero; screens that
  showed instead stay under the stage as `<width>.unmatched.txt`.

## Running the kit

Linux only. Needs bash 4+, tmux, curl, jq, sed, awk, perl (with
`Digest::SHA`), sha256sum, git, ps, pgrep, node, python3, sqlite3, ip, mount,
hostname and util-linux `unshare` 2.38+ with unprivileged user, network,
mount, UTS and PID namespaces. No npm: the tarballs are unpacked as npm lays
out a global install (`<prefix>/lib/node_modules/cline`, the platform package
below its `node_modules`, `<prefix>/bin/cline` linking to the wrapper).

```bash
scripts/capture --root /path/to/scratch --dry-run all          # tool check, self tests; no network
scripts/capture --root /path/to/scratch all                    # the release in compat/RELEASE
scripts/capture --root /path/to/scratch --release 3.0.75 all   # the next release, one command
scripts/capture --root /path/to/scratch hooks errors           # single steps (after install)
```

A new release is captured before the version lock moves: `--release` fetches
it, `compat/RELEASE` records its integrity and binary sha256, and the
previously recorded release becomes the older hub of the hub step. A release
recorded in `compat/RELEASE` must still match the registry's integrity.
macOS is not supported by this kit yet: `scripts/capture` refuses any
platform but Linux (it has no `sandbox-exec` isolation path). The macOS arm64
capture needs that path first and lands as `compat/macos-arm64/`.

## Install and use

The plugin is not in the plugin index yet. Install it from a checkout:

```bash
cargo xtask package build /path/to/plugin-cline/package --output cline.tar.zst   # in zajca/pohunek
pohunek plugin install ./cline.tar.zst --sha256 sha256:<digest> --yes
pohunek plugin enable pohunek.runtime.cline
pohunek plugin select pohunek.runtime.cline
pohunek session new --agent cline
```

or, for development, `pohunek plugin link package --yes` followed by `enable` and
`select`. Cline itself is installed by its own channel (`npm install -g cline`)
and must be on the daemon's `PATH`; configure a provider once (`cline`) before a
session can do work.

What the package does:

- Launches `cline` with `CLINE_NO_AUTO_UPDATE=1` (otherwise an npm install
  updates itself at exit) and `CLINE_DISABLE_CLINE_PASS_NOTICE=1` (no promo
  modal takes the first key of the initial input; three launches on one data
  root showed none, `compat/screens/promo-suppressed-*`). It passes no prompt
  argument: the initial input is typed into the composer once it is drawn.
- Accepts Cline `3.0.74` up to, not including, `4.0.0`.
- Passes a core-generated id as `cline --id <uuid>` and resumes with the same
  flag. An unknown id starts a session under exactly that id; a known id
  resumes it.
- Refuses input while a dialog shows (`refuse_while_blocked`).
- Reads the screen only: Cline emits no OSC status record, and its title
  carries the prompt text (`compat/terminal/`).

## Limits

- **Hub routing (known Phase 1 limit).** In its default mode Cline attaches to
  a running Cline hub, or starts one that outlives the session
  (`compat/hub/`). With a hub running, `--id <unknown uuid>` fails with
  `Unknown session`, so a launch can fail, and tools and hooks of a hub-hosted
  session run outside the session's process tree. The package keeps this
  uncontrolled until P2 and is not admitted to the index before it.
- **Development recipe.** A host profile over `cline` with `[env]`
  `CLINE_SANDBOX = "1"` and `CLINE_SANDBOX_DATA_DIR = "<absolute data root>"`
  keeps the session in the TUI process and the user's store
  (`compat/hub/sandbox-*.txt` show the local routing and no hub daemon). The
  path is host-specific, so it cannot be a package default. These variables
  relocate the store without `--data-dir`, which the descriptor's recovery
  check cannot see (it reads `CLINE_DATA_DIR`); that case is unverified.
- **Cline 4.x is refused.** `version_probe` requires an upper bound, so the
  open-ended range `>=3.0.74` cannot be declared. `below = "4.0.0"` stands in
  until zajca/pohunek#1963; a 3.x release newer than 3.0.74 is accepted without
  a capture.
- **Approvals.** Cline's own approval setting applies; Pohunek adds no
  `--auto-approve` argument. A fresh Cline shows `Auto-approve all enabled`
  (`compat/screens/idle-chat/`), so by default Cline runs every tool without
  asking, and Pohunek does not change that. Shift+Tab inside a session toggles
  the setting; that it persists globally is a research measurement not in
  `compat/` (unverified here).
- **Typed text answers an approval.** Enter alone, `ok` and `yes please` all run
  the pending tool; only `n` rejects (`compat/screens/approval-*`). The
  descriptor therefore refuses session input while the approval box shows; the
  operator answers through `pohunek attach`.
- **Telemetry.** Cline has no environment opt-out. Telemetry is controlled by
  the user's Cline settings: `"telemetryOptOut": true` in
  `<data root>/settings/global-settings.json` (kept across a start,
  `compat/install/settings-after-start.json`). `cline --telemetry-selfcheck`
  still reports `enabled: true` with it set, and what would be sent is
  unverified (`compat/install/telemetry-selfcheck.txt`). The package does not
  write this file.
- **Fork is unsupported**; the assigned reference goes stale after `/clear`,
  `/fork` and an in-TUI `/history` switch (the next turn's hooks carry the new
  id, `compat/hooks/assigned-*`), and nothing follows it until P2.
- **Process matching.** The hub daemon and plugin workers share comm `cline`
  with the agent; the process rules cannot tell them apart.

## Checks

```bash
(cd kit && cargo test --locked)   # descriptor, version, process trees, screens, rule by rule, rule-line and quoting cases
```

`kit/` pins the core revision it is proven against (`kit/Cargo.toml`, the same
as plugin-muse-code) and the Rust toolchain of that revision
(`kit/rust-toolchain.toml`). Move both together with a new core release.

### Detect rules against the corpus

Each rule alone matches exactly these scenarios at every captured width, and
the whole manifest reads each scenario as the state in the last column
(`kit/tests/package.rs`). Priorities: dialogs 980, working 960, idle 950.

| Rule | State | Matches (own) |
|---|---|---|
| `approval` | blocked | `approval`, `approval-rule-line` |
| `question` | blocked | `question`, `question-clipped` |
| `onboarding` | blocked | `onboarding`, `onboarding-byo-selected` |
| `provider_picker` | blocked | `provider-picker`, `provider-picker-search`, `model-step` |
| `oauth_wait` | blocked | `signin-chatgpt`, `signin-chatgpt-after`, `signin-cline`, `signin-cline-after`, `signin-clinepass`, `signin-clinepass-after` |
| `api_key_form` | blocked | `api-key-form` |
| `promo_modal` | blocked | `promo`, `promo-launch-1`, `promo-launch-2` |
| `hub_dialog` | blocked | none (unverified, source only; also needs `Press Enter to update`) |
| `fork_confirm` | blocked | `fork-confirm` |
| `history_picker` | blocked | `history-picker` |
| `undo_picker` | blocked | `undo-picker` |
| `undo_confirm` | blocked | `undo-confirm` |
| `working_thinking` | working | `working-thinking`, `approval`, `approval-rule-line` |
| `working_spinner_row` | working | `working-thinking`, `working-tool-row`, `approval`, `approval-rule-line`, `question`, `question-clipped`, `signin-chatgpt`, `signin-chatgpt-after` |
| `idle_composer` | idle | every screen that draws the composer, also behind promo, fork and picker dialogs and during a turn |

The approval box and the question picker also show a spinner row, and the
sign-in wait shows a spinner, so the blocked rules outrank the working rules.
Whole-manifest result: all `Blocked` scenarios above read blocked, `idle-*`,
`turn-done`, `error`, `resume`, `clear`, `forked`, `subagent`, `team`,
`hub-*`, `promo-suppressed-*`, `promo-launch-3`, `question-answered`,
`approval-*` (after the answer), `cancel-escape-*`, `history-switched` and
`undo-done` read idle, `working-*` read working, and the exit summaries read
nothing. The dialogs that replace the composer (`approval`, `question`, `onboarding`,
`provider_picker`, `oauth_wait`, `api_key_form`) read the text below the last
horizontal-rule line (a line made of `─`, or one led by at least three of
them). With the composer drawn that is only the footer, so a transcript that
quotes their phrases stays unblocked (checked). The approval box prints the
proposed command raw and the question picker prints the model's text, so
either can carry such a line:

- `approval` matches its buttons (`[y]`, `[n]`, `Approve`), which sit below
  the command and stay visible when a taller command is clipped
  (`compat/screens/approval-rule-line`, at 120x40 and at 80x24 with a command
  taller than the box). A rule line in the command does not hide it.
- `question` matches the title `Cline is asking a question` or the last entry
  `Type a response...`. The entry scrolls out of view when the list is taller
  than the picker (`compat/screens/question-clipped`, 80x24, a long question
  and 5 options), and it turns into the typed text; the title is always
  drawn. **Residual case (not closed, no capture):** a rule line in the
  question text moves the region below the title, and together with a clipped
  list neither phrase is in the region, so the question reads `working` and
  `refuse_while_blocked` does not apply. Closing it needs a core matcher for
  "no composer on screen".

`promo_modal`, `fork_confirm`, `history_picker`, `undo_picker` and
`undo_confirm` have the composer drawn behind them, so they read the whole
screen and pair a title with the dialog's own footer; a transcript that
quotes both of a rule's phrases would read blocked. `hub_dialog` (unverified)
also reads the whole screen and pairs a title with `Press Enter to update`,
which the short toast of the same title lacks (source only).

### Definition of done and the smoke run

D1 (lint, double build, `verify`, `detect-check` on the P0 corpus) is covered by
the kit tests above. D2 to D6 are checked by `scripts/smoke`, which links
`package/` under a development id into an isolated test daemon
(`pohunek daemon start --dev-subprocess`), launches the Cline release of
`compat/RELEASE` and drives it against `scripts/model-stub`:

```bash
scripts/capture --root /path/to/scratch install        # the release into a throwaway prefix
scripts/smoke --root /path/to/smoke-root --core-bin /path/to/core/target/debug \
  --cline-prefix /path/to/scratch/prefix/3.0.74
```

Isolation (asserted by the script before anything starts): user, network,
mount, UTS and PID namespaces with only a loopback interface and a private
process tree that ends with the run; empty tmpfs mounts over the operator's
`~/.cline` and over the `pohunek/` directory of the host runtime dir (the
daemon's pathname sockets are not scoped to a network namespace; the script
asserts the host daemon socket is not visible); 127.0.0.1:25463 unreachable; a HOME, `CLINE_DIR`,
`CLINE_DATA_DIR`, hub port (27463) and hub discovery path below `--root`;
every command through `env -i`, so no `POHUNEK_*` reaches the daemon, Cline or
the CLI. The sessions carry `CLINE_SANDBOX=1` and `CLINE_SANDBOX_DATA_DIR`
(the development recipe above), so each stays in its TUI process. The script
refuses a prefix whose version or binary sha256 differs from `compat/RELEASE`,
and compares the set of Cline hub daemons on the host before and after (the
operator's hub is never contacted and is unchanged).

Result, Cline 3.0.74 on Linux x86_64 against the core build of
zajca/pohunek `4fcf94d` (three consecutive runs, all PASS; the stub served
replies 0, 1 and 2):

| DoD | Measured |
|---|---|
| D2 | A bare launch reads `idle`; `CLINE_NO_AUTO_UPDATE=1` and `CLINE_DISABLE_CLINE_PASS_NOTICE=1` are in the launch environment; no promo modal on the screen and no `settings/cli-notices.json`; no hub daemon |
| D3 | `session new --input "smoke prompt"` returns `prompt_delivered: true`; the prompt is on the screen; the process runs as `cline --id <uuid>` with the core-generated id as the native reference |
| D4 | The stub turn (reply held 8 s) reads `working`, then `idle` |
| D5 | Under `--auto-approve false` (set by the scenario's profile), a stub `run_commands` reads `blocked` with `Cline needs permission` on screen; `session input "hello"` is refused with `session_input_blocked`; the command did not run and the box still shows |
| D6 | `session stop` then `session resume` relaunches `cline --id <uuid>`, shows the earlier prompt and reply, accepts a further turn and keeps the native reference; with `<data>/sessions/<id>/<id>.json` moved away, resume is refused with `agent_native_reference_missing` |

Findings of the smoke run:

- **Version probe vs telemetry.** The daemon probes `cline --version` in an
  empty private HOME with a 2 second deadline. In a namespace without a route
  out, Cline needed about 8.4 s to exit unless its settings opt out of
  telemetry (0.4 s with `telemetryOptOut`, measured with the release
  binary), so the probe failed with `agent_runtime_unsupported` until the
  harness started `cline` through a launcher that sets `CLINE_DATA_DIR`. On a
  host with a working route out the probe is expected to be fast; a host
  where Cline's telemetry endpoint hangs would see the same refusal
  (unverified, no such host was available).
- The sessions run Cline through that launcher, not through the bare
  wrapper; the process layout under it is the wrapper's (`node ... bin/cline
  --id <uuid>`).

## Verified behaviour (Cline 3.0.74, Linux x86_64)

Every row comes from the corpus in `compat/`. "Stub" means against
`scripts/model-stub`, not a real model.

### Install and process

| Area | Result | Evidence |
|---|---|---|
| Version | `cline --version` and `cline version` print `3.0.74` | `install/version.txt` |
| Packages | Wrapper `cline` (Node script `bin/cline`, `postinstall: node ./postinstall.mjs \|\| true`) plus `@cline/cli-linux-x64` holding a 161,793,504-byte ELF binary | `install/layout.txt`, `RELEASE` |
| Process | The PTY root is `node` (comm `node-MainThread`); it spawns the binary as a child: comm `cline` (path in the platform package) without the postinstall cache, comm `.cline` (`bin/.cline`, a hard link) with it | `install/process-nocache.txt`, `install/process-cache.txt`, `install/postinstall-cache.txt` |
| Settings | A pre-written `global-settings.json` with `telemetryOptOut: true` and `autoUpdateEnabled: false` keeps both values after a start; no other key is added | `install/settings-after-start.json` |
| Telemetry self check | `--telemetry-selfcheck` reports `enabled: true` and host `otel.cline.bot` even with `telemetryOptOut: true` set; whether events would be sent is unverified (the namespace has no route) | `install/telemetry-selfcheck.txt` |
| First start | Creates `db/{sessions,teams}.db`, `logs/cline.log`, `sessions/`, `settings/`; without network no `cache/feature-flags.json` appears | `install/first-start-tree.txt` |

### Screens

| State | Text | Evidence |
|---|---|---|
| Onboarding | `Welcome to Cline` / `Connect a model provider to get started.`, four cards, `↑/↓ navigate, Enter to select, Ctrl+C to exit` | `screens/onboarding/` |
| Provider picker | `Choose a provider`, `Search providers...`, `▼ 228 more`, `Type to search, ↑/↓ navigate, Enter to select, Esc to go back` | `screens/provider-picker/` |
| API-key form | `OpenAI Compatible`, `Base URL`, `API key`, `Tab to switch fields, Enter to save, Esc to go back, Ctrl+C to exit` | `screens/api-key-form/` |
| Model step | After saving the form against the stub: `Set model ID` / `Enter to create, Esc to go back to model selection` | `screens/model-step/120.txt` |
| Sign-in | `Signing in with Cline` and `Signing in with ClinePass` end offline with `getaddrinfo ETIMEOUT api.workos.com` / `Esc to go back`; `Signing in with ChatGPT` shows `⠏ Waiting for sign-in...` and `If the browser didn't open:` with the authorize URL, and hands that URL to `xdg-open`. The URL's `state` and `code_challenge` are nonces of the aborted offline flow: redacted in `signin-opened-urls.txt`, left in the screen, and new on every capture | `screens/signin-*`, `screens/signin-opened-urls.txt` |
| Promo modals | `Try ClinePass` on the first launch with a provider, `Introducing Cline Desktop` / `Get Cline Desktop` on the second, none on the third; footer `Press Enter to open, any other key to close`. **The composer stays drawn behind the modal** (`❯ Wh...`). State in `settings/cli-notices.json`. With `CLINE_DISABLE_CLINE_PASS_NOTICE=1` three launches on one data root show neither and write no `cli-notices.json` | `screens/promo*/`, `screens/promo-suppressed-state.txt` |
| Idle | Home: logo, `What can I do for you?`, composer `❯ What can I do for you?`; chat: `❯ Ask anything...`; footer `<model> (<ctx>) $<cost> ... ○ Plan ● Act (Tab)`, `<cwd> (<branch>)`, `▶▶ Auto-approve all enabled (Shift+Tab)` | `screens/idle-home/`, `screens/idle-chat/` |
| Working | `⠇ Thinking... (esc to cancel)` above the composer; a running tool row `⠧ run_commands(sleep 8)`; finished rows start with `*`, a rejected one with `x`. **The idle composer stays drawn during a turn** | `screens/working-thinking/`, `screens/working-tool-row/` |
| Approval | With `--auto-approve false`: `Cline needs permission` / `Approve tool call?` / `run_commands` / `$ <command>` / `[y] Approve   [n] Deny`, footer `Auto-approve all disabled (Shift+Tab)`. The `Thinking...` line shows at the same time | `screens/approval/` |
| Approval answers | `y` runs the command; `n` rejects it (`Error: This tool call was rejected by the user`); **Enter alone, `ok` then Enter, and `yes please` then Enter all run the command**. Only `n` rejects among the inputs tried; which key of a typed answer approves is unverified | `screens/approval-approved/`, `screens/approval-n/`, `screens/approval-enter/`, `screens/approval-ok/`, `screens/approval-text/` (`effect.txt` records the input) |
| Question | `Cline is asking a question` / question / `> Red`, `Blue`, `Green` / `Type a response...`; `1` and Enter answers `Red` | `screens/question/`, `screens/question-answered/` |
| Fork confirm | `Create a fork of the current conversation?` / `Y/Enter to confirm, N/Esc to cancel`; then `Forked into new session <id>. This is now the active session.` | `screens/fork-confirm/`, `screens/forked/` |
| History picker | `Session History`, newest first, `↑/↓ navigate, Enter to resume, ← delete, → export, Esc to close` | `screens/history-picker/` |
| Undo | `Restore to Checkpoint` (one entry per prompt), then `Restore chat only` / `Restore chat and workspace`; after a restore the composer holds the restored prompt | `screens/undo-*/` |
| Error | A provider error ends the turn with `* Error: <message>` | `screens/error/` |
| Exit | `/quit` prints `Session Summary` with `Continue  cline --id <id>` after leaving the alternate screen | `screens/exit-summary*/` |

Screens are captured at widths 20, 40, 80, 120, 160 and 200 for onboarding,
provider picker, API-key form, promo, idle, working, approval, question and
fork confirm; the rest at 120 (`screens/INDEX.tsv`).

### Terminal

The title is OSC 0 `Cline`, then `> <prompt>` per submitted prompt; no title
change for working, idle or blocked. The CLI sends OSC 0, 4, 10, 11, 12, 66,
99 and 1337 (queries apart from the title), no OSC 7501, 9 or 777. Modes:
`?1049h`, `?2004h` (bracketed paste), mouse `?1000/1002/1003/1006h`, `?1004h`,
`?2026` synchronized output, `?2027h`, `?2031h` (`terminal/`).

### Hooks (`<CLINE_DIR>/hooks/<Event>.sh`, sandbox mode)

| Area | Result | Evidence |
|---|---|---|
| Launch with `--id <uuid>` | An unknown id starts a session under exactly that id; **every turn fires `TaskResume` (`agent_resume`), never `TaskStart`**, and the hook environment holds `CLINE_HOOK_AGENT_RESUME=1` | `hooks/assigned-first-turn/` |
| Launch without `--id` | Every turn fires `TaskStart` (`agent_start`); the session id matches `^[0-9]{13}_[0-9a-z]{5}$` | `hooks/generated-id/`, `sessions/generated-id/id-shape.txt` |
| Payload | `hookName`, `taskId` (`conv_*`), `sessionContext.rootSessionId` (the session id), `agent_id`, `parent_agent_id` (null for the lead), `userId` (the `USER` value), `clineVersion` (empty), `workspaceRoots`, `workspaceInfo`, plus `taskStart`/`taskResume`, `turn`/`taskComplete`, `error`, `tool_call`/`preToolUse`, `tool_result`/`postToolUse` | `hooks/*/` |
| Environment | The TUI's environment (the marker `CAPTURE_PROBE` arrives) plus `CLINE_BUILD_ENV`, `CLINE_CONNECTOR_CLI_LAUNCH`, `CLINE_WRAPPER_PATH`, `NODE_EXTRA_CA_CERTS`, `OPENTUI_GRAPHICS` and the sandbox-mode data variables; the hook's ancestors are `.cline` > `node` > the terminal | `hooks/assigned-first-turn/001-TaskResume.json` |
| Subagent (`spawn_agent`) | The subagent's hooks carry its own `agent_id`, the caller's `parent_agent_id` and the same `rootSessionId`; store row `<root>__<agent_id>`, `is_subagent=1`, its messages file in the root's session directory | `hooks/assigned-subagent/`, `sessions/assigned-subagent/` |
| `/clear`, `/fork`, `/history` | No hook fires at the switch; the next turn's hooks carry the new `rootSessionId` (`/clear`: a new session; `/fork`: the id `Forked into new session` printed, with `metadata.fork.forkedFromSessionId`; `/history`: the chosen session) | `hooks/assigned-*-switch/`, `hooks/assigned-after-*/`, `hooks/history-after-switch/`, `sessions/assigned-after-fork/` |
| `/undo` | Restoring chat and workspace resets the files to the checkpoint and continues in a new session id, reported by the next turn | `hooks/undo-after/`, `sessions/undo/workspace.txt` |
| Relaunch `--id <uuid>` | Restores the conversation of that id from before `/clear` and `/fork` | `screens/resume/`, `hooks/assigned-resume/` |
| Errors | An HTTP 400 from the provider fires `TaskError` (`agent_error`, `error.message`) after one request; the row reads `failed` | `hooks/error-400/`, `stub/error-400-requests.jsonl` |
| Cancel | Escape while the model streams, Escape while a tool runs and Ctrl+C (which also exits) fire `SessionShutdown` (`session_shutdown`, no reason field), **never `TaskCancel`**; the row reads `cancelled`. Escape while a subagent streams fires `SessionShutdown` for the subagent (with its `parent_agent_id`) and for the lead | `hooks/cancel-*/`, `sessions/cancel-*/` |
| `/quit` | Fires no hook | `hooks/assigned-quit/` (empty) |
| `UserPromptSubmit` | Never fired in any TUI or headless turn of the corpus | `hooks/*/` |
| Agent team | `/team <task>`: `team_spawn_teammate` and `team_run_task` run; the teammate's hooks carry its own `agent_id`, the same `rootSessionId` and **`parent_agent_id: null`**, so a teammate looks like a lead in its hooks; store row `<root>__teamtask__<agent>__<suffix>`, `is_subagent=1`, `team_name`; team tables in `db/teams.db` | `hooks/team/`, `sessions/team/` |

### Hub placement (namespace only, private port)

| Area | Result | Evidence |
|---|---|---|
| No hub, no sandbox mode | The TUI runs locally (`compatible_hub_unavailable`) and starts `.cline --cline-hub-daemon` as its child, a session leader that **survives the TUI's exit** | `hub/prewarm-*.txt`, `hub/prewarm-routing.tsv` |
| Sandbox mode | Routing `local`, `forceLocalBackend: true`; no hub daemon while the TUI runs or after it exits | `hub/sandbox-*.txt`, `hub/sandbox-routing.tsv` |
| Hub running | The TUI uses the hub (`Using discovered local hub runtime host`). **`--id <unknown uuid>` then fails with `Error: Unknown session: <id>`**; without `--id` the turn works, and its hooks run below the hub daemon (ancestors `.cline` > the namespace init), without a variable only the TUI had | `screens/hub-attach-*/`, `hooks/hub-attach/`, `hub/attach-*` |
| Older hub | A 3.0.73 hub serving a 3.0.73 session: the 3.0.74 TUI falls back to the local runtime (`compatible_hub_unavailable`) and shows no hub dialog; the 3.0.73 hub keeps running | `screens/hub-outdated/`, `hub/outdated-*` |

### Headless

`cline --json -t 3 "<prompt>"` against a reply held 30 s ran the full 30 s and
exited 0 with `run_result` `completed`: the timeout is not applied to the
first turn of a prompt run (source: `apps/cli/src/runtime/run-agent.ts:319-342`
schedules it after `start()` has run that turn). NDJSON in
`headless/timeout.ndjson`.

## Unverified

- **Smoke scope.** The smoke run used a dev-subprocess test daemon, the stub
  and sandbox mode (local runtime); a run against the default hub-attached
  mode, with a real model, or with an installed (service) daemon is
  unverified. Input that is typed into the approval box by an operator
  through `pohunek attach` was not driven.
- **`submit_delay_ms = 500`**: the capture kit's pause between typed text and
  Enter; a shorter delay is untested.
- **Cline 3.0.75 and later 3.x**: accepted by the version range, never captured.
- **macOS**: everything; the manager's capture lands as `compat/macos-arm64/`.
- **Arch package** (`cline-cli`): the host package is 3.0.73, so the `arch`
  step recorded nothing; its layout under the release is unverified.
- **Hub dialogs** `Cline Hub update required` and `Cline Hub was updated`:
  source only (`apps/cli/src/tui/root.tsx:660-695`,
  `tui/components/dialogs/hub-update-required.tsx`); a 3.0.73 hub did not
  trigger them, and sandbox mode never watches the hub
  (`hub-update-required-helpers.ts`).
- **Sign-in steps after the first screen**: the device-code screen
  (`Your code:`) and the model list (`Choose a model`) are source only
  (`tui/views/onboarding/screens.tsx:220,495`).
- **`TaskCancel`**, **`UserPromptSubmit`**, **`PreCompact`**: never fired;
  when they fire, if ever, is unverified.
- **VCR playback** (`CLINE_VCR=playback`) with the TUI.
- What the release would send to `otel.cline.bot` or fetch from
  `data.cline.bot` with network: the namespace blocks both.
- Behaviour with a real model or provider.

# plugin-cline

Pohunek runtime plugin for the [Cline CLI](https://github.com/cline/cline)
terminal coding agent.

| | |
|---|---|
| Repository | `pohunekd/plugin-cline` |
| Package id | `pohunek.runtime.cline` |
| Runtime id (`--agent`) | `cline` |
| Upstream | Cline CLI by Cline Bot Inc. (Apache-2.0), npm package `cline` |
| Status | **P2, placement and identity**: `package/` (launch, status, resume, local placement, reported session identity, subagents) for Cline 3.0.74 on Linux x86_64, proven against the P0 corpus and `scripts/smoke`. Minimum core is 0.44.0, the release the kit and the smoke ran against (every required feature exists from 0.42.0, older cores are unverified) |

The plugin carries data and evidence only (descriptor, detect rules, captures).
Handlers, reporters and parsers stay in the core repository
[`zajca/pohunek`](https://github.com/zajca/pohunek); the plan is the epic #1
and the issues P0-P5 of this repository.

## What is here

| Path | Content |
|---|---|
| `package/` | The runtime package: `runtime.toml` (launch contract) and `detect.toml` (process and screen rules) |
| `kit/` | Stand-in for the plugin kit: a test crate that builds the package twice, installs it through core's definition path, reads the captured version banner and process trees, and runs the detection manifest on every screen of `compat/screens`, as a whole and rule by rule |
| `scripts/smoke` | End-to-end check against a pohunek build and an installed Cline: isolated test daemon in a loopback-only namespace, reporter install, launch, input, status, approval refusal, resume, launch beside a running Cline hub, reported session reference across `/clear` and `/fork`, a reported subagent, foreign hooks left byte-identical |
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

## Drift tracking

The supported range has no upper bound the descriptor can state, so a newer
Cline release is checked by capture instead. `.github/workflows/drift.yml`
runs weekly (and on `workflow_dispatch`, optionally with `release` set to a
release other than the newest). It installs the newest `cline` release from the
npm registry, runs `scripts/capture all` against the loopback stub in the usual
namespaces, and runs `scripts/drift-check --detect`, which compares the fresh
output with `compat/`:

- **screens**: the text of every scenario and width;
- **hooks**: the shape of every hook payload (key paths and value types, not
  values), the events and their counts (not their order) and the env names;
- **process**: every process tree and after-session listing, and the number of
  `--cline-hub-daemon` processes in each;
- **help**: the `--help` text of every command;
- **capture**: expectations the capture itself missed;
- **detect**: the kit's tests (`detect.toml` rules, version probe) against the
  fresh screens and banner.

The job needs no secret and no model, and its token is read-only. On a
difference a second job (the only one with `issues: write`) runs
`scripts/drift-issue`: it finds the open issue labelled `cline-drift`, updates
it and comments when the release changed, or creates it, so reruns never open a
second one. Nothing edits `compat/`, `compat/RELEASE` or `package/`. The fresh
evidence and the report are kept as the `drift` workflow artifact for 30 days.
Locally, the same comparison is
`scripts/capture --root <scratch> --out <fresh> all` followed by
`scripts/drift-check --fresh <fresh> --stage <scratch>/stage --detect`;
`scripts/drift-check --self-test` (also in CI) checks the comparison itself on
the committed evidence.

### Updating to a new Cline release

1. Read the drift issue and the `drift` artifact (report, fresh evidence).
2. Capture the release into the tree: `scripts/capture --root <scratch>
   --release X.Y.Z all`. `compat/` and `compat/RELEASE` are replaced by the run;
   a step that missed an expectation leaves its evidence unchanged and fails the
   run, so fix the cause first.
3. Review `git diff compat/` scenario by scenario: a changed screen, hook
   payload or process tree is either harmless text or a change to the
   descriptor (`package/detect.toml`, `package/runtime.toml`) and the kit's case
   tables (`kit/tests/package.rs`). Run `(cd kit && cargo test --locked)`.
4. Commit the evidence together with the descriptor changes it needs. The new
   `compat/RELEASE` is the lock: the kit requires the probed banner to equal
   its `release=` line. Moving it is the last edit, in the same pull request.
5. Close the drift issue when that pull request merges; the next scheduled run
   compares against the new evidence.

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

### Hooks and placement

The package names core's `cline-hook-v1` handler (`requires`
`integration-cline-hook-v1` and `placement-v1`, hook schema
`identity-subagent-v1`). Two things come from it:

- **Local placement.** Every create and resume launch carries
  `CLINE_SANDBOX=1` and `CLINE_SANDBOX_DATA_DIR=<data root>`, so the session
  runs in the TUI's process tree, keeps the user's store (`CLINE_DATA_DIR`, else
  `<config home>/data`), and never attaches to or starts a shared hub daemon.
  A host profile must not set those variables, `--data-dir` or `--config`;
  core refuses the launch.
- **The reporter hook.** `pohunek integration install --agent cline` writes one
  reporter script under six names in `<CLINE_DIR>/hooks/` (`TaskStart.sh`,
  `TaskResume.sh`, `TaskComplete.sh`, `TaskError.sh`, `TaskCancel.sh`,
  `SessionShutdown.sh`), each carrying the marker
  `# POHUNEK_INTEGRATION_ID=cline`. `--profile <name>` acts on the config home
  of a host profile; `status` and `doctor` report the files, `uninstall`
  removes only the marked ones. Per core's contract (not driven here), an
  install refuses, writing nothing, when one of the six names holds a file
  without the marker; a user hook under another name (`TaskResume`,
  `TaskResume.py`) is not a collision and Cline runs both (the smoke runs an
  extensionless one).

Cline runs the files of `<CLINE_DIR>/hooks/` for every turn on the host, not
only for Pohunek sessions. The reporter exits 0 at once, without reading its
input or touching a file, when `POHUNEK_ENV` is not set, and it prints nothing
(Cline reads a hook's stdout as control output). In a Pohunek session it
reports the root session id every turn, so a `/clear` or `/fork` is followed at
the next turn, and a subagent's start and stop.

What the package does:

- Launches `cline` with `CLINE_NO_AUTO_UPDATE=1` (otherwise an npm install
  updates itself at exit) and `CLINE_DISABLE_CLINE_PASS_NOTICE=1` (no promo
  modal takes the first key of the initial input; three launches on one data
  root showed none, `compat/screens/promo-suppressed-*`). It passes no prompt
  argument: the initial input is typed into the composer once it is drawn.
- Accepts Cline `3.0.74` and every later release (open range, see Limits).
- Passes a core-generated id as `cline --id <uuid>` and resumes with the same
  flag. An unknown id starts a session under exactly that id; a known id
  resumes it.
- Refuses input while a dialog shows (`refuse_while_blocked`).
- Reads the screen only: Cline emits no OSC status record, and its title
  carries the prompt text (`compat/terminal/`).

## Limits

- **Hub routing is core's job.** Cline's default mode attaches to a running
  hub, or starts one that outlives the session (`compat/hub/`); with a hub
  running, `--id <unknown uuid>` fails with `Unknown session` and the hooks of
  a hub-hosted session run outside the session's process tree. The package
  does not set the sandbox variables itself: core's `cline-hook-v1` handler
  does, per launch. A hub that is already running is left alone and not used
  (`scripts/smoke`); whether a later Cline release keeps honouring
  `CLINE_SANDBOX` is what the smoke's "no hub daemon" assertions guard.
- **Reported references are not checked for existence (core gap,
  [zajca/pohunek#2528](https://github.com/zajca/pohunek/issues/2528)).** The
  descriptor keeps `strategy = "assigned"` and its `<id>/<id>.json` check, which
  applies to the reference core assigned at launch. Once a turn's hook has
  reported the session id (the first turn does), core 0.44.0 relaunches without
  looking for the manifest. After a prompt, a moved or deleted session file is
  therefore not refused as `agent_native_reference_missing`. What `cline --id
  <id>` then does is unverified (no capture relaunches after a moved manifest;
  an unknown id starts a session under that id, so an empty conversation is the
  expected outcome) until core fixes it.
  `scripts/smoke` asserts the refusal only for a session that never took a
  prompt and does not claim the reported case is refused.
- **Cline 4.x and later are admitted.** `version_probe` is open above
  `min = "3.0.74"` (`version-probe-open-range-v1`); a newer release launches
  without a capture, and drift is caught by capturing new releases
  (`scripts/capture --release`), not by the daemon.
- **Subagent limits.** The reporter reports a subagent by its `agent_id`
  without a role. A teammate of `/team` carries `parent_agent_id: null`
  (`compat/hooks/team/`), so it is not a subagent for the reporter, and its
  turn start reports the shared `rootSessionId`, which is the lead's (source
  reading of the reporter, not run). A lead's aborted turn and `/quit` report
  nothing; the reference stays until the next turn.
- **Hooks are host-wide.** `<CLINE_DIR>/hooks/` serves every Cline turn of that
  config home; the reporter is a no-op outside Pohunek (see Hooks and
  placement). Uninstalling the integration removes only the files that carry
  the ownership marker.
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
- **Fork is unsupported** as a Pohunek operation (the descriptor has no `[fork]`
  table; Cline forks only inside its TUI). The reported reference follows `/clear`,
  `/fork` and an in-TUI `/history` switch at the next turn (the hooks of that
  turn carry the new id, `compat/hooks/assigned-*`; `scripts/smoke` drives
  `/clear` and `/fork`; `/history` and `/undo` are unverified through core).
  Between the switch and the next turn the reference is the old one.
- **A question with a rule line in its text and a clipped option list reads
  `working`.** The `question` rule matches its title or its last entry
  (`Type a response...`) below the last rule line; a rule line in the model's
  question text hides the title, and a long option list scrolls the entry out
  of view (`compat/screens/question-clipped`, 80x24). Then the session is not
  `blocked`, `refuse_while_blocked` does not apply and `session input` can
  answer the question. Core has had a `composer_absent` matcher since 0.42.0
  (`detect-composer-absent-v1`) that could close it; the package does not use
  it yet, and no capture shows the combined case. See Checks, "Detect rules
  against the corpus".
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
  `refuse_while_blocked` does not apply. Core's `composer_absent` matcher
  (0.42.0) could close it; the package does not use it yet.

`promo_modal`, `fork_confirm`, `history_picker`, `undo_picker` and
`undo_confirm` have the composer drawn behind them, so they read the whole
screen and pair a title with the dialog's own footer; a transcript that
quotes both of a rule's phrases would read blocked. `hub_dialog` (unverified)
also reads the whole screen and pairs a title with `Press Enter to update`,
which the short toast of the same title lacks (source only).

### Definition of done and the smoke run

The kit tests above cover the descriptor's lint, double build, `verify` and
`detect-check` on the P0 corpus. `scripts/smoke` covers the rest: it links
`package/` under a development id into an isolated test daemon
(`pohunek daemon start --dev-subprocess`), installs the reporter
(`pohunek integration install --agent cline --profile cline-local`), launches
the Cline release of `compat/RELEASE` through the package's own launch path
and drives it against `scripts/model-stub`. The profiles set only the config
home, data root, hub port and discovery path; none sets a `CLINE_SANDBOX*`
variable (core refuses a launch that does), and the script asserts that the
launch environment of a session nevertheless holds `CLINE_SANDBOX=1` and
`CLINE_SANDBOX_DATA_DIR`.

```bash
scripts/capture --root /path/to/scratch install        # the release into a throwaway prefix
scripts/smoke --root /path/to/smoke-root --core-bin /path/to/core/bin \
  --cline-prefix /path/to/scratch/prefix/3.0.74
```

`--core-bin` holds `pohunek`, `pohunekd` and `pohunek-sessiond`: a core build
or the unpacked `pohunek-daemon` release archive. `--root` is at most 40
characters long: the longest path the daemon binds, a worker's hook socket,
adds 67 bytes below it and Linux limits a socket path to 107.

Isolation (asserted by the script before anything starts): user, network,
mount, UTS and PID namespaces with only a loopback interface and a private
process tree that ends with the run; empty tmpfs mounts over the operator's
`~/.cline` and over the `pohunek/` directory of the host runtime dir (the
daemon's pathname sockets are not scoped to a network namespace; the script
asserts the host daemon socket is not visible); 127.0.0.1:25463 unreachable; a HOME, `CLINE_DIR`,
`CLINE_DATA_DIR`, hub port (27463) and hub discovery path below `--root`;
every command through `env -i`, so no `POHUNEK_*` reaches the daemon, Cline or
the CLI. The script refuses a prefix whose version or binary sha256 differs
from `compat/RELEASE`, and compares the set of Cline hub daemons on the host
before and after (the operator's hub is never contacted and is unchanged). The
hub of the P2-D1 scenario is the script's own, started inside the namespace on
the private port and stopped by the script.

Two observer hooks, `TaskResume` and `TaskComplete` (no suffix), stand for a
user's own hooks: each appends the process chain above itself,
`CLINE_SANDBOX` and the `rootSessionId` of its payload to a log, and prints
nothing. They are the foreign hooks of P2-D4 and the witnesses of P2-D1 and
P2-D2.

Result, Cline 3.0.74 on Linux x86_64 against the core release v0.44.0
(`pohunek-daemon-0.44.0-x86_64-unknown-linux-gnu`; three consecutive runs, all PASS; the stub
served replies 0 to 9):

| DoD | Measured |
|---|---|
| P1-D2 | A bare launch reads `idle`; `CLINE_NO_AUTO_UPDATE=1` and `CLINE_DISABLE_CLINE_PASS_NOTICE=1` are in the launch environment; no promo modal on the screen and no `settings/cli-notices.json`; no hub daemon |
| P1-D3 | `session new --input "smoke prompt"` returns `prompt_delivered: true`; the prompt is on the screen; the process runs as `cline --id <uuid>` with the core-generated id as the native reference |
| P1-D4 | The stub turn (reply held 8 s) reads `working`, then `idle` |
| P1-D5 | Under `--auto-approve false` (set by the scenario's profile), a stub `run_commands` reads `blocked` with `Cline needs permission` on screen; `session input "hello"` is refused with `session_input_blocked`; the command did not run and the box still shows |
| P1-D6 | `session stop` then `session resume` relaunches `cline --id <uuid>`, shows the earlier prompt and reply, accepts a further turn and keeps the native reference; resume of a session that never took a prompt (no `<id>/<id>.json`) is refused with `agent_native_reference_missing` (a session with a reported reference is not checked, see Limits) |
| P2-D1 | With an isolated, compatible Cline hub running (`cline hub start` on the private port), a launch through the package reads `idle` and answers a stub turn; the hub daemon set is unchanged during and after the session; every hook of the session (the observer hooks) has the TUI's PTY root among its ancestors and `CLINE_SANDBOX=1`; no hub daemon runs below the session; `session inspect` reports `placement.outcome: clean` |
| P2-D2 | After one stub turn the session's reference is the id the hook reported (`active_agent_session_id` and `native_session_id` equal the observer hook's `rootSessionId`, which on the first turn is also the launch id); `/clear` then a turn re-binds it to the new id, `/fork` (confirmed through `pohunek attach` with piped Enter, because input is refused while the confirmation shows) then a turn re-binds it again, each equal to the `rootSessionId` of the observer hook; a stub `spawn_agent` turn is reported as one `cline` subagent with a parent id and lifecycle `completed` |
| P2-D3 | `pgrep -f -- --cline-hub-daemon` finds no process while the bare, turn, resumed, approval and identity sessions run and after they end (the package launches without any hub); the host's own hub set is unchanged |
| P2-D4 | `integration install`, `status` and `doctor` (all with `--profile cline-local`) leave the observer hooks byte-identical (sha256); the six reporter files carry the marker; `uninstall` removes the six files, keeps the `hooks/` directory and leaves the observer hooks byte-identical |

Findings of the smoke run:

- **Version probe vs telemetry.** The daemon probes `cline --version` in an
  empty private HOME with a 2 second deadline. In a namespace without a route
  out, Cline needed about 8.4 s to exit unless its settings opt out of
  telemetry (0.4 s with `telemetryOptOut`, measured with the release
  binary), so the probe failed with `agent_runtime_unsupported` until the
  harness started `cline` through a launcher that sets `CLINE_DATA_DIR`. On a
  host with a working route out the probe is expected to be fast; a host
  where Cline's telemetry endpoint hangs would see the same refusal
  (unverified, no such host was available). One `session new` in about eight
  runs on a busy host was refused with `agent_runtime_unsupported` all the
  same (the run exited at once; the next run passed unchanged), so a smoke
  failure at a launch is rerun before it is read as a regression.
- The sessions run Cline through that launcher, not through the bare
  wrapper; the process layout under it is the wrapper's (`node ... bin/cline
  --id <uuid>`).

## Verified behaviour (Cline 3.0.75, Linux x86_64)

Every row comes from the corpus in `compat/`. "Stub" means against
`scripts/model-stub`, not a real model.

### Install and process

| Area | Result | Evidence |
|---|---|---|
| Version | `cline --version` and `cline version` print `3.0.75` | `install/version.txt` |
| Packages | Wrapper `cline` (Node script `bin/cline`, `postinstall: node ./postinstall.mjs \|\| true`) plus `@cline/cli-linux-x64` holding a 161,793,504-byte ELF binary | `install/layout.txt`, `RELEASE` |
| Process | The PTY root is `node` (comm `node-MainThread`); it spawns the binary as a child: comm `cline` (path in the platform package) without the postinstall cache, comm `.cline` (`bin/.cline`, a hard link) with it | `install/process-nocache.txt`, `install/process-cache.txt`, `install/postinstall-cache.txt` |
| Settings | A pre-written `global-settings.json` with `telemetryOptOut: true` and `autoUpdateEnabled: false` keeps both values after a start; no other key is added | `install/settings-after-start.json` |
| Telemetry self check | `--telemetry-selfcheck` reports `enabled: true` and host `otel.cline.bot` even with `telemetryOptOut: true` set; whether events would be sent is unverified (the namespace has no route) | `install/telemetry-selfcheck.txt` |
| First start | Creates `db/{sessions,teams}.db`, `logs/cline.log`, `sessions/`, `settings/`; without network no `cache/feature-flags.json` appears | `install/first-start-tree.txt` |

### Screens

| State | Text | Evidence |
|---|---|---|
| Onboarding | `Welcome to Cline` / `Connect a model provider to get started.`, four cards, `↑/↓ navigate, Enter to select, Ctrl+C to exit` | `screens/onboarding/` |
| Provider picker | `Choose a provider`, `Search providers...`, `▼ 320 more`, `Type to search, ↑/↓ navigate, Enter to select, Esc to go back` | `screens/provider-picker/` |
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
| `UserPromptSubmit` | Fired (`prompt_submit`) only by the headless prompt run that `--timeout` aborts; no TUI turn of the corpus fired it | `hooks/headless-timeout/` |
| Agent team | `/team <task>`: `team_spawn_teammate` and `team_run_task` run; the teammate's hooks carry its own `agent_id`, the same `rootSessionId` and **`parent_agent_id: null`**, so a teammate looks like a lead in its hooks; store row `<root>__teamtask__<agent>__<suffix>`, `is_subagent=1`, `team_name`; team tables in `db/teams.db` | `hooks/team/`, `sessions/team/` |

### Hub placement (namespace only, private port)

| Area | Result | Evidence |
|---|---|---|
| No hub, no sandbox mode | The TUI runs locally (`compatible_hub_unavailable`) and starts `.cline --cline-hub-daemon` as its child, a session leader that **survives the TUI's exit** | `hub/prewarm-*.txt`, `hub/prewarm-routing.tsv` |
| Sandbox mode | Routing `local`, `forceLocalBackend: true`; no hub daemon while the TUI runs or after it exits | `hub/sandbox-*.txt`, `hub/sandbox-routing.tsv` |
| Hub running | The TUI uses the hub (`Using discovered local hub runtime host`). **`--id <unknown uuid>` then fails with `Error: Unknown session: <id>`**; without `--id` the turn works, and its hooks run below the hub daemon (ancestors `.cline` > the namespace init), without a variable only the TUI had | `screens/hub-attach-*/`, `hooks/hub-attach/`, `hub/attach-*` |
| Older hub | A 3.0.74 hub serving a 3.0.74 session: the 3.0.75 TUI falls back to the local runtime (`compatible_hub_unavailable`) and shows no hub dialog; the 3.0.74 hub keeps running | `screens/hub-outdated/`, `hub/outdated-*` |

### Headless

`cline --json -t 3 "<prompt>"` against a reply held 30 s stops the first turn
after 3 s: exit code 1, an `error` record `run timed out after 3s` and
`run_result` `aborted` (Cline 3.0.74 ran the full 30 s and exited 0 with
`completed`). The hooks are `UserPromptSubmit`, `TaskStart` and
`SessionShutdown`; no `TaskComplete` fires. NDJSON in
`headless/timeout.ndjson`, exit status in `headless/timeout-exit.txt`.

## Unverified

- **Smoke scope.** The smoke run used a dev-subprocess test daemon and the
  stub; a run with a real model, or with an installed (service) daemon, is
  unverified. Input that is typed into the approval box by an operator
  through `pohunek attach` was not driven. Not driven either: `/history` and
  `/undo` switches through the reporter, `/team`, an aborted turn, a
  subagent stop with outcome `failed` or `cancelled`, and the reporter's
  `hub_hosted` refusal (a hook run by a hub carries no `CLINE_SANDBOX`).
- **Core 0.42.0 to 0.43.x.** `requires` names features of 0.38 to 0.42, which
  those releases implement, but the kit and the smoke ran only against 0.44.0.
- **`submit_delay_ms = 500`**: the capture kit's pause between typed text and
  Enter; a shorter delay is untested.
- **Every Cline release after 3.0.75**: accepted by the version range, never captured.
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
- **`TaskCancel`**, **`PreCompact`**, and `UserPromptSubmit` in a TUI turn: never fired;
  when they fire, if ever, is unverified.
- **VCR playback** (`CLINE_VCR=playback`) with the TUI.
- What the release would send to `otel.cline.bot` or fetch from
  `data.cline.bot` with network: the namespace blocks both.
- Behaviour with a real model or provider.

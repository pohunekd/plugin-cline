//! The Cline CLI package (`package/`) against core: it builds reproducibly,
//! installs through the daemon's definition path, reads the captured version
//! banner, matches the captured process forms, and reads every committed
//! screen (`compat/screens`) as the case tables say, both as a whole
//! manifest and rule by rule.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use package::directory::build_directory_archive;
use package::{read_archive, Limits};
use pohunek_daemon::agent::host::{
    definition_from_archive, InputReadiness, LaunchProgram, RuntimeDefinition,
};
use pohunek_daemon::agent::NativeReferenceStrategy;
use pohunek_daemon::detect::{DetectionConfig, Detector, DetectorConfig, Manifest};
use pohunek_daemon::procwatch::{ProcessFact, StartIdentity};
use protocol::{AgentActivity, StateSource};

use AgentActivity::{Blocked, Idle, Working};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the crate sits in the repository")
        .to_path_buf()
}

fn read(relative: &str) -> String {
    fs::read_to_string(repo().join(relative)).unwrap_or_else(|e| panic!("read {relative}: {e}"))
}

fn built_archive() -> Vec<u8> {
    build_directory_archive(&repo().join("package"), &Limits::DEFAULT)
        .expect("the package directory builds")
}

/// The definition `plugin install` derives from the package directory.
fn installed_definition() -> RuntimeDefinition {
    let bytes = built_archive();
    let archive = read_archive(&bytes, &Limits::DEFAULT).expect("the archive reads");
    let digest = archive.digest().clone();
    definition_from_archive(archive.entries(), &digest)
        .expect("the daemon accepts the package descriptor")
}

#[test]
fn the_package_builds_reproducibly_with_only_descriptor_and_manifest() {
    let first = built_archive();
    assert_eq!(first, built_archive(), "two builds produce the same bytes");
    let archive = read_archive(&first, &Limits::DEFAULT).expect("the archive reads");
    let paths: Vec<&str> = archive.entries().iter().map(|e| e.path.as_str()).collect();
    assert_eq!(paths, ["detect.toml", "runtime.toml"]);
}

#[test]
fn the_descriptor_declares_the_launch_contract_of_the_captured_release() {
    let definition = installed_definition();
    assert_eq!(definition.runtime_id().as_str(), "cline");
    assert_eq!(definition.display_name(), "Cline CLI");
    assert_eq!(
        definition.program(),
        &LaunchProgram::Fixed("cline".to_owned())
    );
    assert!(
        definition.default_args().is_empty(),
        "Cline's own approval setting applies: no --auto-approve argument"
    );
    assert!(
        !definition.prompt_arg(),
        "the prompt is typed into the composer"
    );
    assert_eq!(
        definition.launch_env(),
        [
            ("CLINE_DISABLE_CLINE_PASS_NOTICE".to_owned(), "1".to_owned()),
            ("CLINE_NO_AUTO_UPDATE".to_owned(), "1".to_owned()),
        ]
    );
    assert_eq!(definition.input_readiness(), InputReadiness::ScreenIdle);
    assert!(definition.input_rules().bracketed_paste);
    assert!(
        format!("{:?}", definition.input_rules()).ends_with("allow_while_blocked: false }"),
        "typed text answers a Cline dialog, so input is refused while one shows"
    );
    assert_eq!(
        definition.native_reference_strategy(),
        NativeReferenceStrategy::Assigned
    );
    assert_eq!(
        format!("{:?}", definition.native()),
        r#"Some(NativeSessionLaunch { reference_kind: Id, resume_args: NativeArgs([Literal("--id"), Reference]), fork_args: None, assigned: Some(AssignedReference { launch_args: NativeArgs([Literal("--id"), Reference]), existence: File(FileExistence { root_env: Some("CLINE_DIR"), root_home: Some(".cline"), dir: Some("data/sessions"), file_name: "{reference}.json", name_match: Exact, max_depth: 1, store: Some(StoreEnv { name: "CLINE_DATA_DIR", max_depth: 2 }), relocating_args: ["--data-dir", "--config"], relocating_settings: [] }), assign_fork: false }) })"#,
        "core generates the id, `--id` assigns and resumes it, recovery needs <id>/<id>.json"
    );
    let home = definition
        .config_home()
        .expect("the descriptor declares the Cline config home");
    assert_eq!(
        (home.env(), home.subdir(), home.default_relative()),
        ("CLINE_DIR", None, ".cline")
    );
    assert!(definition.manifest().program_status().is_none());
}

#[test]
fn the_version_probe_accepts_the_captured_banner_and_the_supported_range_only() {
    let definition = installed_definition();
    let policy = definition
        .version_probe_policy()
        .expect("the descriptor declares a version probe");
    assert_eq!(policy.args(), ["--version"]);
    let banner = read("compat/install/version.txt");
    let output = banner
        .lines()
        .skip_while(|l| *l != "$ cline --version")
        .nth(1)
        .expect("the output of `cline --version`");
    let version = policy
        .check_output(output)
        .expect("the captured banner is a supported release");
    let release = read("compat/RELEASE");
    assert!(
        release.lines().any(|l| l == format!("release={version}")),
        "the probed release {version} is the locked release"
    );
    for accepted in ["3.0.74", "3.0.75", "3.9.9"] {
        assert!(policy.check_output(accepted).is_ok(), "{accepted}");
    }
    // The upper bound is a stopgap for the open-ended range (zajca/pohunek#1963).
    for refused in ["3.0.73", "4.0.0", "3.0.74-rc.1", "Cline 3.0.74", ""] {
        assert!(policy.check_output(refused).is_err(), "{refused:?}");
    }
}

fn process(comm: &str, cmdline: &[&str]) -> ProcessFact {
    ProcessFact {
        pid: 4242,
        pgid: 4242,
        ppid: 1,
        start_identity: StartIdentity::new(1),
        comm: comm.to_owned(),
        cmdline: cmdline.iter().map(|a| (*a).to_owned()).collect(),
    }
}

#[test]
fn the_process_matchers_accept_the_captured_process_trees_and_nothing_else() {
    let definition = installed_definition();
    let matchers = definition
        .manifest()
        .process_matchers()
        .expect("the manifest declares process matchers");
    let mut seen = 0;
    for file in [
        "process-nocache.txt",
        "process-cache.txt",
        "process-sandbox-tui.txt",
    ] {
        for line in read(&format!("compat/install/{file}")).lines() {
            let (left, command) = line.trim().split_once(": ").expect("comm and command");
            let comm = left.split(" [").next().expect("comm");
            let args: Vec<&str> = command.split_whitespace().collect();
            assert!(matchers.matches(&process(comm, &args)), "{file}: {line}");
            seen += 1;
        }
    }
    assert!(seen >= 6, "the captured trees were read ({seen} processes)");
    assert!(matchers.matches(&process(
        "sh",
        &["/usr/lib/node_modules/cline/bin/cline", "--id", "x"]
    )));
    for (comm, cmdline) in [
        ("clinex", &["clinex"][..]),
        ("xcline", &["/usr/bin/xcline"][..]),
        ("node", &["node", "/opt/cline.js"][..]),
        ("sh", &["/usr/bin/clinerc", "a"][..]),
    ] {
        assert!(
            !matchers.matches(&process(comm, cmdline)),
            "{comm} {cmdline:?}"
        );
    }
}

fn detector_for(
    manifest: Arc<Manifest>,
    readiness: InputReadiness,
    now: Instant,
    rows: u16,
    columns: u16,
) -> Detector {
    Detector::new(
        rows,
        columns,
        now,
        DetectorConfig {
            detection: DetectionConfig {
                recheck_after: Duration::from_millis(100),
                confirmations: 1,
                cap: Duration::from_millis(700),
                stable_visible_refresh: Duration::from_millis(800),
                startup_grace: Duration::ZERO,
            },
            manifest: Some(manifest),
            input_readiness: readiness,
            launch_name: None,
        },
    )
}

/// The activity a manifest reads from the screen text alone.
fn screen_activity(manifest: &Arc<Manifest>, text: &str, columns: u16) -> Option<AgentActivity> {
    let rows: Vec<&str> = text.trim_end_matches('\n').split('\n').collect();
    let height = u16::try_from(rows.len().max(24)).expect("a screen height");
    let started = Instant::now();
    let mut detector = detector_for(
        Arc::clone(manifest),
        InputReadiness::ScreenIdle,
        started,
        height,
        columns,
    );
    let mut bytes = b"\x1b[2J\x1b[H".to_vec();
    bytes.extend_from_slice(rows.join("\r\n").as_bytes());
    detector
        .feed(started, &bytes)
        .iter()
        .rfind(|t| t.source == StateSource::Screen)
        .map(|t| t.activity)
}

/// What a scenario's screens mean to the whole manifest, at every width.
fn case_table() -> BTreeMap<&'static str, Option<AgentActivity>> {
    let blocked = [
        "api-key-form",
        "approval",
        "approval-rule-line",
        "fork-confirm",
        "history-picker",
        "model-step",
        "onboarding",
        "onboarding-byo-selected",
        "promo",
        "promo-launch-1",
        "promo-launch-2",
        "provider-picker",
        "provider-picker-search",
        "question",
        "question-clipped",
        "signin-chatgpt",
        "signin-chatgpt-after",
        "signin-cline",
        "signin-cline-after",
        "signin-clinepass",
        "signin-clinepass-after",
        "undo-confirm",
        "undo-picker",
    ];
    let idle = [
        "approval-approved",
        "approval-enter",
        "approval-n",
        "approval-ok",
        "approval-text",
        "cancel-escape-stream",
        "cancel-escape-subagent",
        "cancel-escape-tool",
        "clear",
        "error",
        "forked",
        "history-switched",
        "hub-attach-assigned-id",
        "hub-attach-generated-id",
        "hub-outdated",
        "idle-chat",
        "idle-home",
        "promo-launch-3",
        "promo-suppressed-launch-1",
        "promo-suppressed-launch-2",
        "promo-suppressed-launch-3",
        "question-answered",
        "resume",
        "subagent",
        "team",
        "turn-done",
        "undo-done",
    ];
    let mut table = BTreeMap::new();
    for name in blocked {
        table.insert(name, Some(Blocked));
    }
    for name in idle {
        table.insert(name, Some(Idle));
    }
    for name in ["working-thinking", "working-tool-row"] {
        table.insert(name, Some(Working));
    }
    // Exit summaries are plain text after the alternate screen is left.
    for name in [
        "cancel-ctrl-c-stream",
        "exit-summary",
        "exit-summary-assigned",
    ] {
        table.insert(name, None);
    }
    table
}

/// Screen files of the corpus: `(scenario, width, text)`.
fn corpus() -> Vec<(String, u16, String)> {
    let mut scenarios: Vec<_> = fs::read_dir(repo().join("compat/screens"))
        .expect("the screens exist")
        .map(|e| e.expect("entry").path())
        .filter(|p| p.is_dir())
        .collect();
    scenarios.sort();
    let mut out = Vec::new();
    for dir in scenarios {
        let name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .expect("name")
            .to_owned();
        let mut files: Vec<_> = fs::read_dir(&dir)
            .expect("scenario directory")
            .map(|e| e.expect("entry").path())
            .collect();
        files.sort();
        for file in files {
            let Some(width) = file
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.strip_suffix(".txt"))
                .and_then(|n| n.parse::<u16>().ok())
            else {
                continue;
            };
            out.push((
                name.clone(),
                width,
                fs::read_to_string(&file).expect("screen"),
            ));
        }
    }
    out
}

#[test]
fn every_committed_screen_reads_as_the_case_table_says() {
    let definition = installed_definition();
    let table = case_table();
    let mut problems = Vec::new();
    let mut checked = 0;
    for (name, width, text) in corpus() {
        let Some(expected) = table.get(name.as_str()) else {
            problems.push(format!("scenario {name} has no case-table row"));
            continue;
        };
        let got = screen_activity(definition.manifest(), &text, width);
        if got != *expected {
            problems.push(format!(
                "{name}/{width}: expected {expected:?}, read {got:?}"
            ));
        }
        checked += 1;
    }
    assert!(
        problems.is_empty(),
        "{} of {checked} screens:\n{}",
        problems.len(),
        problems.join("\n")
    );
    assert!(checked >= 100, "the corpus was found ({checked} screens)");
}

/// Splits `detect.toml` into its `[process]` head and one source per rule.
fn rule_sources() -> Vec<(String, String)> {
    let source = read("package/detect.toml");
    let (head, rest) = source.split_once("[[rules]]").expect("rules");
    let mut out = Vec::new();
    for body in rest.split("[[rules]]") {
        let id = body
            .lines()
            .find_map(|l| l.strip_prefix("id = \""))
            .and_then(|l| l.strip_suffix('"'))
            .expect("rule id")
            .to_owned();
        out.push((id, format!("{head}[[rules]]{body}")));
    }
    out
}

/// Scenarios each rule matches on its own (every width of the scenario), and
/// the rules that match no capture.
fn rule_table() -> BTreeMap<&'static str, Vec<&'static str>> {
    BTreeMap::from([
        ("approval", vec!["approval", "approval-rule-line"]),
        ("question", vec!["question", "question-clipped"]),
        ("onboarding", vec!["onboarding", "onboarding-byo-selected"]),
        (
            "provider_picker",
            vec!["model-step", "provider-picker", "provider-picker-search"],
        ),
        (
            "oauth_wait",
            vec![
                "signin-chatgpt",
                "signin-chatgpt-after",
                "signin-cline",
                "signin-cline-after",
                "signin-clinepass",
                "signin-clinepass-after",
            ],
        ),
        ("api_key_form", vec!["api-key-form"]),
        (
            "promo_modal",
            vec!["promo", "promo-launch-1", "promo-launch-2"],
        ),
        ("hub_dialog", vec![]),
        ("fork_confirm", vec!["fork-confirm"]),
        ("history_picker", vec!["history-picker"]),
        ("undo_picker", vec!["undo-picker"]),
        ("undo_confirm", vec!["undo-confirm"]),
        (
            "working_thinking",
            vec!["approval", "approval-rule-line", "working-thinking"],
        ),
        (
            "working_spinner_row",
            vec![
                "approval",
                "approval-rule-line",
                "question",
                "question-clipped",
                "signin-chatgpt",
                "signin-chatgpt-after",
                "working-thinking",
                "working-tool-row",
            ],
        ),
        ("idle_composer", vec![]),
    ])
}

#[test]
fn each_rule_matches_its_own_screens_and_no_others() {
    let table = rule_table();
    let corpus = corpus();
    let mut problems = Vec::new();
    for (id, source) in rule_sources() {
        let manifest = Arc::new(Manifest::parse_str(&source).expect("a one-rule manifest parses"));
        let mut matched = BTreeMap::<String, BTreeSet<u16>>::new();
        let mut all_widths = BTreeMap::<String, BTreeSet<u16>>::new();
        for (name, width, text) in &corpus {
            all_widths.entry(name.clone()).or_default().insert(*width);
            if screen_activity(&manifest, text, *width).is_some() {
                matched.entry(name.clone()).or_default().insert(*width);
            }
        }
        let observed: BTreeSet<&str> = matched.keys().map(String::as_str).collect();
        let expected: Option<BTreeSet<&str>> =
            table.get(id.as_str()).map(|v| v.iter().copied().collect());
        println!("RULE {id}: {observed:?}");
        match expected {
            None => problems.push(format!("rule {id} has no row in the rule table")),
            Some(_) if id == "idle_composer" => {
                // The composer stays drawn behind promo, fork and picker
                // dialogs and during a turn, so it matches more than the idle
                // screens; the whole-manifest table ranks it below those.
                for name in ["idle-chat", "idle-home", "turn-done", "hub-outdated"] {
                    assert!(observed.contains(name), "{name}");
                }
                for name in [
                    "exit-summary",
                    "approval",
                    "question",
                    "onboarding",
                    "provider-picker",
                ] {
                    assert!(!observed.contains(name), "{name}");
                }
            }
            Some(expected) => {
                if observed != expected {
                    problems.push(format!(
                        "rule {id}: matched {observed:?}, expected {expected:?}"
                    ));
                }
                for (name, widths) in &matched {
                    if *widths != all_widths[name] {
                        problems.push(format!(
                            "rule {id}: {name} matched at {widths:?} of {:?}",
                            all_widths[name]
                        ));
                    }
                }
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn prose_that_quotes_a_dialog_above_the_composer_is_not_blocked() {
    let definition = installed_definition();
    let rule = "\u{2500}".repeat(40);
    let composer = format!("{rule}\n\u{276f} Ask anything...\n{rule}\n");
    for quoted in [
        "Cline needs permission\nApprove tool call?\n[y] Approve   [n] Deny",
        "Cline is asking a question",
        "Welcome to Cline\nConnect a model provider to get started.",
        "Choose a provider\nSigning in with Cline\nEnter to save, Esc to go back",
    ] {
        let text = format!(" \u{276f} explain this\n\n * {quoted}\n\n{composer}");
        assert_eq!(
            screen_activity(definition.manifest(), &text, 80),
            Some(Idle),
            "{quoted:?}"
        );
    }
}

#[test]
fn a_rule_line_in_the_proposed_command_does_not_hide_the_approval_box() {
    let definition = installed_definition();
    let rule = "\u{2500}".repeat(20);
    let text = format!(
        " \u{276f} run a command\n\n \u{280b} Thinking... (esc to cancel)\n\n Cline needs permission\n\n Approve tool call?\n\n run_commands\n\n   $ cat > NOTES.md <<'EOF'\nTitle\n{rule}\nEOF\n\n  [y] Approve   [n] Deny\n\n stub-model (10) $0.00\n ws (main)\n Auto-approve all disabled (Shift+Tab)\n"
    );
    assert_eq!(
        screen_activity(definition.manifest(), &text, 80),
        Some(Blocked)
    );
}

#[test]
fn a_rule_line_in_a_question_does_not_hide_the_question_picker() {
    let definition = installed_definition();
    let rule = "\u{2500}".repeat(20);
    let text = format!(
        " \u{280b} ask_question(Which?)\n\n Cline is asking a question\n\n {rule}\n\n  > {rule}\n    Blue\n    Type a response...\n\n stub-model (10) $0.00\n Auto-approve all disabled (Shift+Tab)\n"
    );
    assert_eq!(
        screen_activity(definition.manifest(), &text, 80),
        Some(Blocked)
    );
}

#[test]
fn a_single_phrase_of_a_picker_above_the_composer_is_not_blocked() {
    let definition = installed_definition();
    let rule = "\u{2500}".repeat(40);
    let composer = format!("{rule}\n\u{276f} Ask anything...\n{rule}\n");
    for quoted in [
        "pick Restore chat only to keep your files",
        "Restore to Checkpoint",
        "Session History",
        "Create a fork of the current conversation?",
    ] {
        let text = format!(" * {quoted}\n\n{composer}");
        assert_eq!(
            screen_activity(definition.manifest(), &text, 80),
            Some(Idle),
            "{quoted:?}"
        );
    }
}

#[test]
fn a_hub_dialog_blocks_but_the_toast_of_the_same_title_does_not() {
    let definition = installed_definition();
    let rule = "\u{2500}".repeat(40);
    let composer = format!("{rule}\n\u{276f} Ask anything...\n{rule}\n footer\n");
    for title in ["Cline Hub was updated", "Cline Hub update required"] {
        let dialog = format!(" {title}\n\n Press Enter to update\n\n{composer}");
        assert_eq!(
            screen_activity(definition.manifest(), &dialog, 80),
            Some(Blocked),
            "{title}"
        );
        let toast = format!(" The shared {title} (core 1). Run 'cline update'.\n\n{composer}");
        assert_eq!(
            screen_activity(definition.manifest(), &toast, 80),
            Some(Idle),
            "{title} toast"
        );
    }
}

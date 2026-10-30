use crate::fsutil::remove_any;
use crate::{
    BootOutcome, Engine, EventKind, GenerationState, Limits, RollbackReason, Slot, UpdateError,
    UserDataState, MAX_EVENTS,
};
use litric_core::Version;
use std::fs;
use std::path::{Path, PathBuf};

const NOW: u64 = 1_800_000_000;

fn version(raw: &str) -> Version {
    Version::parse(raw).expect("test version")
}

fn root(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("litric-update-{}-{name}", std::process::id()));
    remove_any(&path).expect("clean test root");
    fs::create_dir_all(&path).expect("create test root");
    path
}

fn cleanup(path: &Path) {
    let _ = remove_any(path);
}

fn source(root: &Path, files: &[(&str, &str)]) -> PathBuf {
    let src = root.join("source");
    fs::create_dir_all(&src).expect("source root");
    for (name, body) in files {
        let path = src.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("source parent");
        }
        fs::write(path, body).expect("source file");
    }
    src
}

fn init(root: &Path) -> Engine {
    Engine::init(root, version("1.0.0")).expect("initial state")
}

fn stage(engine: &mut Engine, src: &Path, raw: &str) -> Slot {
    engine.stage(src, version(raw)).expect("stage candidate")
}

fn boot_candidate(engine: &mut Engine) {
    let outcome = engine.note_boot(NOW).expect("note boot");
    assert!(
        matches!(outcome, BootOutcome::Booting { .. }),
        "candidate did not boot: {outcome:?}"
    );
}

#[test]
fn a_candidate_is_staged_copied_and_committed_without_touching_the_active_slot() {
    let root = root("commit");
    let src = source(&root, &[("app", "hello"), ("conf/theme", "dark")]);
    let mut engine = init(&root);

    let slot = stage(&mut engine, &src, "1.1.0");
    assert_eq!(slot, Slot::B);
    assert_eq!(engine.state().active(), Slot::A);
    assert_eq!(engine.state().pending(), Some(Slot::B));
    assert_eq!(
        fs::read_to_string(root.join("system/slots/A/generation")).expect("A marker"),
        "litric-generations-generation\t1\nversion\t1.0.0\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("system/slots/B/app")).expect("B file"),
        "hello"
    );
    assert_eq!(
        engine.state().generation(Slot::B).map(|g| g.state),
        Some(GenerationState::Staged)
    );

    boot_candidate(&mut engine);
    let event = engine.commit(NOW + 1).expect("health commit");
    assert_eq!(event.kind, EventKind::Committed);
    assert_eq!(event.from, Slot::A);
    assert_eq!(event.to, Slot::B);
    assert_eq!(event.reason, RollbackReason::HealthCheck);
    assert_eq!(event.user_data, UserDataState::Untouched);
    assert_eq!(engine.state().active(), Slot::B);
    assert_eq!(engine.state().pending(), None);
    assert_eq!(
        engine.active_version().map(ToString::to_string),
        Some("1.1.0".to_owned())
    );

    drop(engine);
    let reopened = Engine::open(&root).expect("reopen state");
    assert_eq!(reopened.state().active(), Slot::B);
    assert_eq!(
        reopened.state().generation(Slot::A).map(|g| g.state),
        Some(GenerationState::Committed),
        "the previous generation remains a rollback target"
    );
    cleanup(&root);
}

#[test]
fn three_failed_boots_discard_the_candidate_and_keep_the_active_generation() {
    let root = root("boots");
    let src = source(&root, &[("app", "candidate")]);
    let mut engine = init(&root);
    stage(&mut engine, &src, "1.1.0");

    for attempt in 1..=3 {
        let outcome = engine.note_boot(NOW).expect("count boot");
        assert_eq!(
            outcome,
            BootOutcome::Booting {
                slot: Slot::B,
                attempt,
            }
        );
    }
    let outcome = engine.note_boot(NOW).expect("reject after budget");
    assert_eq!(
        outcome,
        BootOutcome::CandidateRejected {
            slot: Slot::B,
            attempts: 3,
        }
    );
    assert_eq!(engine.state().active(), Slot::A);
    assert_eq!(engine.state().pending(), None);
    assert_eq!(
        engine.state().generation(Slot::B).map(|g| g.state),
        Some(GenerationState::Bad)
    );
    assert_eq!(
        engine.active_version().map(ToString::to_string),
        Some("1.0.0".to_owned())
    );
    assert!(root.join("system/slots/B/app").exists());
    cleanup(&root);
}

#[test]
fn rollback_switches_to_the_previous_committed_slot_and_leaves_user_data_alone() {
    let root = root("rollback");
    let src = source(&root, &[("app", "second")]);
    let mut engine = init(&root);
    stage(&mut engine, &src, "1.1.0");
    boot_candidate(&mut engine);
    engine.commit(NOW + 1).expect("commit second generation");

    let user_file = root.join("user/notes.txt");
    fs::create_dir_all(user_file.parent().expect("user parent")).expect("user dir");
    fs::write(&user_file, "the user's work").expect("user file");

    let event = engine
        .rollback(RollbackReason::Manual, NOW + 2)
        .expect("rollback");
    assert_eq!(event.kind, EventKind::RolledBack);
    assert_eq!(event.from, Slot::B);
    assert_eq!(event.to, Slot::A);
    assert_eq!(event.reason, RollbackReason::Manual);
    assert_eq!(event.user_data, UserDataState::Untouched);
    assert_eq!(engine.state().active(), Slot::A);
    assert_eq!(
        engine.state().generation(Slot::B).map(|g| g.state),
        Some(GenerationState::Bad)
    );
    assert_eq!(
        fs::read_to_string(&user_file).expect("user file survives"),
        "the user's work"
    );
    assert!(!engine.state().can_rollback());
    cleanup(&root);
}

#[test]
fn a_downgrade_is_refused_before_the_state_or_a_slot_is_written() {
    let root = root("downgrade");
    let src = source(&root, &[("app", "older")]);
    let mut engine = init(&root);
    let state_before = fs::read(root.join("system/state")).expect("state bytes");

    let error = engine.stage(&src, version("0.9.0")).expect_err("downgrade");
    assert!(
        matches!(error, UpdateError::NotAnUpgrade { .. }),
        "{error:?}"
    );
    assert_eq!(engine.state().pending(), None);
    assert_eq!(
        fs::read(root.join("system/state")).expect("state"),
        state_before
    );
    assert!(!root.join("system/slots/B").exists());

    let equal = engine.stage(&src, version("1.0.0")).expect_err("equal");
    assert!(
        matches!(equal, UpdateError::NotAnUpgrade { .. }),
        "{equal:?}"
    );
    cleanup(&root);
}

#[test]
fn a_second_stage_requires_the_first_candidate_to_finish() {
    let root = root("pending");
    let src = source(&root, &[("app", "one")]);
    let mut engine = init(&root);
    stage(&mut engine, &src, "1.1.0");

    let error = engine
        .stage(&src, version("1.2.0"))
        .expect_err("already pending");
    assert_eq!(error.to_string(), "slot B already holds a staged candidate");
    let event = engine.discard_pending(NOW).expect("discard");
    assert_eq!(event.kind, EventKind::Rejected);
    assert_eq!(event.reason, RollbackReason::UserCancelled);
    assert_eq!(engine.state().active(), Slot::A);
    assert_eq!(engine.state().pending(), None);
    cleanup(&root);
}

#[test]
fn the_boot_budget_is_a_boundary_not_an_accumulator() {
    let root = root("budget");
    let src = source(&root, &[("app", "candidate")]);
    let mut engine = init(&root);
    stage(&mut engine, &src, "1.1.0");

    for attempt in 1..=3 {
        assert_eq!(
            engine.note_boot(NOW).expect("boot"),
            BootOutcome::Booting {
                slot: Slot::B,
                attempt
            }
        );
    }
    assert_eq!(
        engine.state().generation(Slot::B).map(|g| g.attempts),
        Some(3)
    );
    let first = engine.note_boot(NOW).expect("first rejection");
    assert_eq!(
        first,
        BootOutcome::CandidateRejected {
            slot: Slot::B,
            attempts: 3,
        }
    );
    let second = engine.note_boot(NOW).expect("no candidate left");
    assert_eq!(second, BootOutcome::Active { slot: Slot::A });
    cleanup(&root);
}

#[cfg(unix)]
#[test]
fn a_symlink_in_the_source_is_refused_and_the_active_slot_stays() {
    use std::os::unix::fs::symlink;

    let root = root("symlink");
    let src = source(&root, &[("app", "fine")]);
    symlink("/etc/passwd", src.join("escape")).expect("source symlink");
    let mut engine = init(&root);

    let error = engine.stage(&src, version("1.1.0")).expect_err("symlink");
    assert!(
        matches!(error, UpdateError::UnsafeSource { .. }),
        "{error:?}"
    );
    assert_eq!(engine.state().active(), Slot::A);
    assert_eq!(engine.state().pending(), None);
    assert!(!root.join("system/slots/B").exists());
    assert_eq!(
        fs::read_to_string(root.join("system/slots/A/generation")).expect("A marker"),
        "litric-generations-generation\t1\nversion\t1.0.0\n"
    );
    cleanup(&root);
}

#[test]
fn a_missing_marker_is_never_advertised() {
    let root = root("marker");
    let src = source(&root, &[("app", "candidate")]);
    let mut engine = init(&root);
    stage(&mut engine, &src, "1.1.0");
    remove_any(&root.join("system/slots/B/generation")).expect("remove marker");

    let error = engine.commit(NOW).expect_err("missing marker");
    assert!(
        matches!(
            error,
            UpdateError::CandidateIncomplete { slot: Slot::B, .. }
        ),
        "{error:?}"
    );
    assert_eq!(engine.state().active(), Slot::A);
    assert_eq!(engine.state().pending(), Some(Slot::B));
    cleanup(&root);
}

#[test]
fn a_corrupt_state_is_refused_and_left_byte_for_byte() {
    let root = root("corrupt");
    let engine = init(&root);
    drop(engine);
    let state_path = root.join("system/state");
    fs::write(&state_path, "not a generation record\n").expect("corrupt state");
    let before = fs::read(&state_path).expect("bytes");

    let error = Engine::open(&root).expect_err("corrupt state");
    assert!(matches!(error, UpdateError::InvalidState(_)), "{error:?}");
    assert_eq!(fs::read(&state_path).expect("unchanged"), before);
    cleanup(&root);
}

#[test]
fn limits_stop_a_candidate_before_it_is_published() {
    let limits_root = root("limits");
    let src = source(&limits_root, &[("one", "1"), ("two", "2")]);
    let mut engine = init(&limits_root);
    engine.set_limits(Limits {
        max_entries: 1,
        max_bytes: 16,
        max_path_bytes: 64,
    });

    let error = engine
        .stage(&src, version("1.1.0"))
        .expect_err("entry limit");
    assert!(
        matches!(error, UpdateError::TooManyEntries { limit: 1 }),
        "{error:?}"
    );
    assert_eq!(engine.state().active(), Slot::A);
    assert_eq!(engine.state().pending(), None);
    assert!(!limits_root.join("system/slots/B").exists());

    let bytes_root = root("limits-bytes");
    let mut bytes_engine = init(&bytes_root);
    bytes_engine.set_limits(Limits {
        max_entries: 16,
        max_bytes: 1,
        max_path_bytes: 64,
    });
    let byte_source = source(&bytes_root, &[("one", "12")]);
    let error = bytes_engine
        .stage(&byte_source, version("1.1.0"))
        .expect_err("byte limit");
    assert!(
        matches!(error, UpdateError::TooManyBytes { limit: 1 }),
        "{error:?}"
    );
    cleanup(&bytes_root);
    cleanup(&limits_root);
}

#[test]
fn a_leftover_state_temporary_is_never_parsed_as_state() {
    let root = root("leftover-temp");
    let _engine = init(&root);
    fs::write(root.join("system/.state.tmp-999"), "half-written state\n")
        .expect("leftover temporary");
    let reopened = Engine::open(&root).expect("the primary record is still valid");
    assert_eq!(reopened.state().active(), Slot::A);
    cleanup(&root);
}

#[test]
fn truncated_and_zero_length_state_is_refused_without_a_panic() {
    for (name, body) in [
        ("truncated", "litric-generations\t1\n"),
        ("zero", ""),
        ("partial-event", "litric-generations\t1\nactive\tA\n"),
    ] {
        let root = root(name);
        fs::create_dir_all(root.join("system")).expect("state directory");
        fs::write(root.join("system/state"), body).expect("damaged state");
        let error = Engine::open(&root).expect_err("damaged state must be refused");
        assert!(
            matches!(error, UpdateError::InvalidState(_)),
            "{name}: {error:?}"
        );
        cleanup(&root);
    }
}

#[test]
fn init_refuses_to_replace_an_existing_state() {
    let root = root("init");
    let _engine = init(&root);
    let error = Engine::init(&root, version("2.0.0")).expect_err("existing state");
    assert!(matches!(error, UpdateError::StateAlreadyExists(_)));
    cleanup(&root);
}

#[cfg(unix)]
#[test]
fn the_state_file_is_owner_only() {
    use std::os::unix::fs::PermissionsExt;

    let root = root("mode");
    let _engine = init(&root);
    let mode = fs::metadata(root.join("system/state"))
        .expect("state metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
    cleanup(&root);
}

#[test]
fn the_switch_history_stays_bounded_across_many_updates() {
    let root = root("history");
    let src = source(&root, &[("app", "payload")]);
    let mut engine = init(&root);

    for patch in 1..=10 {
        let candidate = format!("1.{patch}.0");
        stage(&mut engine, &src, &candidate);
        boot_candidate(&mut engine);
        engine.commit(NOW).expect("commit");
        assert!(engine.state().events().len() <= MAX_EVENTS);
    }
    assert_eq!(engine.state().events().len(), MAX_EVENTS);
    assert_eq!(
        engine.state().events().last().map(|event| event.to),
        Some(Slot::A),
        "ten switches alternate slots"
    );
    cleanup(&root);
}

#[test]
fn rollback_without_a_second_generation_is_refused() {
    let root = root("no-target");
    let mut engine = init(&root);
    let error = engine
        .rollback(RollbackReason::Manual, NOW)
        .expect_err("no target");
    assert!(matches!(error, UpdateError::NoRollbackTarget));
    assert_eq!(engine.state().active(), Slot::A);
    cleanup(&root);
}

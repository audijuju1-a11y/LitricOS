//! Real-socket proof for automatic local backups.
//!
//! The daemon is a real `litric-sysd` serving a Unix socket. The state is a real
//! XDG data root with an installed desktop entry and executable, and the backup
//! is made by the serve loop's timer, not by a request. The test then checks the
//! manifest hashes independently, verifies through the protocol, changes and
//! deletes the originals, and proves the snapshot still holds byte-identical
//! copies. Finally it corrupts the snapshot and requires the checksum refusal.
//!
//! Restore is deliberately absent: ADR-0021 requires conflict-aware restore,
//! which this batch does not design. Verification is the proof path.

#[cfg(unix)]
#[test]
fn journey_auto_backup_verify_and_tamper_refusal() {
    use std::os::unix::fs::PermissionsExt;
    use std::time::{Duration, Instant};

    let root = std::env::temp_dir().join(format!("litric-shell-e2e-backup-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a fixture root");

    let data = root.join("data");
    let desktop = data.join("applications/com.example.backed.desktop");
    let executable = data.join("litric/apps/com.example.backed/app");
    std::fs::create_dir_all(desktop.parent().expect("desktop parent")).expect("applications");
    std::fs::create_dir_all(executable.parent().expect("executable parent")).expect("apps");
    let desktop_bytes =
        b"[Desktop Entry]\nType=Application\nName=Backed Fixture\nExec=app\n".to_vec();
    let executable_bytes = b"#!/bin/sh\nprintf '%s' 'backed-app'\n".to_vec();
    std::fs::write(&desktop, &desktop_bytes).expect("desktop entry");
    std::fs::write(&executable, &executable_bytes).expect("executable");
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).expect("chmod");

    let backups = root.join("backups");
    let socket = root.join("sysd.sock");
    let listener = litric_sysd::server::bind(&socket).expect("bind sysd");
    let state = litric_sysd::state::Sysd::new(2, 512, "/tmp", std::path::Path::new("/"))
        .with_application_root(&data)
        .with_backup_root(&backups)
        .with_backup_interval(1);
    std::thread::spawn(move || {
        let _ = litric_sysd::server::serve(&listener, state);
    });

    // Wait for the real scheduler, not a request. `BACKUP STATUS` is written by
    // the create path and must show a snapshot made by the serve loop after it
    // started.
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut snapshot_id = None;
    while Instant::now() < deadline {
        let reply =
            litric_sysd::server::query(&socket, "BACKUP STATUS").expect("status over socket");
        if let Some(id) = reply
            .lines()
            .find_map(|line| line.strip_prefix("last_id="))
            .map(str::to_owned)
        {
            snapshot_id = Some(id);
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let id = snapshot_id.expect("the automatic scheduler made a snapshot");
    let status = litric_sysd::server::query(&socket, "BACKUP STATUS").expect("status over socket");
    assert!(
        status.contains(&format!("last_id={id}")),
        "status reports the automatic snapshot: {status}"
    );
    assert!(
        status.contains("interval_secs=1"),
        "status reports the configured interval: {status}"
    );
    let list = litric_sysd::server::query(&socket, "BACKUP LIST").expect("list over socket");
    assert!(
        list.starts_with("OK 1\nid=") && list.contains(&id),
        "the list names the automatic snapshot: {list}"
    );
    let shell_status = crate::backupstatus::read(&socket);
    assert!(shell_status.is_available(), "{shell_status:?}");
    assert_eq!(shell_status.state, "ready");
    assert_eq!(shell_status.interval_secs, Some(1));
    assert_eq!(shell_status.last_id.as_deref(), Some(id.as_str()));

    let manifest_path = backups.join("snapshots").join(&id).join("manifest");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest on disk");
    let mut lines = manifest.lines();
    assert_eq!(lines.next(), Some("litric-backup\t1"));
    assert_eq!(lines.next(), Some(format!("id\t{id}").as_str()));
    let created = lines
        .next()
        .and_then(|line| line.strip_prefix("created\t"))
        .expect("created field")
        .parse::<u64>()
        .expect("created integer");
    assert_eq!(
        lines.next(),
        Some("source\tapplications,litric/apps"),
        "the manifest names the backed-up set"
    );
    let mut entries = Vec::new();
    for line in lines {
        let fields: Vec<&str> = line.split('\t').collect();
        let [tag, path, size, hash, mode] = fields.as_slice() else {
            panic!("one canonical entry: {line}");
        };
        assert_eq!(*tag, "entry");
        entries.push((
            (*path).to_owned(),
            size.parse::<u64>().expect("size"),
            (*hash).to_owned(),
            mode.parse::<u32>().expect("mode"),
        ));
    }
    assert_eq!(
        entries
            .iter()
            .map(|(path, _, _, _)| path.as_str())
            .collect::<Vec<_>>(),
        vec![
            "applications/com.example.backed.desktop",
            "litric/apps/com.example.backed/app"
        ],
        "exactly the installed state, sorted"
    );
    let expected = [
        (
            "applications/com.example.backed.desktop",
            desktop_bytes.as_slice(),
            0o644,
        ),
        (
            "litric/apps/com.example.backed/app",
            executable_bytes.as_slice(),
            0o755,
        ),
    ];
    for ((path, size, hash, mode), (expected_path, bytes, expected_mode)) in
        entries.iter().zip(expected)
    {
        assert_eq!(path, expected_path);
        assert_eq!(*size, bytes.len() as u64, "size for {path}");
        assert_eq!(
            *hash,
            litric_defender::sha256::hex(&litric_defender::sha256::digest(bytes)),
            "SHA-256 for {path}"
        );
        assert_eq!(*mode, expected_mode, "mode for {path}");
    }

    let total = (desktop_bytes.len() + executable_bytes.len()) as u64;
    let verified =
        litric_sysd::server::query(&socket, &format!("BACKUP VERIFY {id}")).expect("verify");
    assert_eq!(
        verified,
        format!("OK 5\nid={id}\ncreated={created}\nentries=2\nbytes={total}\nstate=verified\n"),
        "verification re-hashes the snapshot through the socket"
    );

    // Change and delete the originals. The snapshot is now the only complete
    // copy; a verify that still passes is a verify of real copied bytes.
    std::fs::write(&executable, b"corrupted-original").expect("mutate original");
    std::fs::remove_file(&desktop).expect("delete original");
    let after = litric_sysd::server::query(&socket, &format!("BACKUP VERIFY {id}"))
        .expect("verify after source mutation");
    assert!(
        after.ends_with("state=verified\n"),
        "the snapshot is independent of the changed source: {after}"
    );
    let snapshot_app = backups
        .join("snapshots")
        .join(&id)
        .join("files/litric/apps/com.example.backed/app");
    let snapshot_desktop = backups
        .join("snapshots")
        .join(&id)
        .join("files/applications/com.example.backed.desktop");
    assert_eq!(
        std::fs::read(&snapshot_app).expect("copied executable"),
        executable_bytes,
        "the copied executable is byte-identical to the original"
    );
    assert_eq!(
        std::fs::read(&snapshot_desktop).expect("copied desktop entry"),
        desktop_bytes,
        "the copied desktop entry is byte-identical to the original"
    );

    // A snapshot that no longer matches its manifest is refused, not reported
    // as verified because the manifest still says so.
    std::fs::write(&snapshot_app, b"tampered").expect("tamper snapshot");
    let refused = litric_sysd::server::query(&socket, &format!("BACKUP VERIFY {id}"))
        .expect("verify tampered snapshot");
    assert!(
        refused.starts_with("ERR checksum "),
        "tampering is a checksum refusal: {refused}"
    );
    std::fs::write(&snapshot_app, &executable_bytes).expect("repair the fixture");
    let repaired = litric_sysd::server::query(&socket, &format!("BACKUP VERIFY {id}"))
        .expect("verify repaired snapshot");
    assert!(repaired.ends_with("state=verified\n"), "{repaired}");

    // The manual trigger is the same service path; it is allowed because the
    // local session holds `BackupControl` scoped to `backup://create`, and the
    // request carries the confirmation word.
    let manual = litric_sysd::server::query(&socket, "BACKUP CREATE CONFIRM")
        .expect("manual create over socket");
    assert!(
        manual.starts_with("OK 5\nid=snap-") && manual.ends_with("state=created\n"),
        "manual create reports a snapshot: {manual}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

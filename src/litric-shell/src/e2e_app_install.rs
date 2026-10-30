//! Real-socket proof for the application install, launch and uninstall path.
//!
//! This lives beside the shell's other end-to-end journeys so it can use the
//! real desktop discovery code. The daemon is a real `litric-sysd` on a Unix
//! socket, the installed program is a real executable copy, and the marker is
//! written by that copy after launch.

#[cfg(unix)]
#[test]
fn journey_app_install_launch_uninstall_through_sysd() {
    use std::os::unix::fs::PermissionsExt;
    use std::time::{Duration, Instant};

    let root = std::env::temp_dir().join(format!(
        "litric-shell-e2e-app-install-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a fixture root");

    let data = root.join("data");
    let incoming = data.join("litric/incoming/fixture");
    std::fs::create_dir_all(&incoming).expect("incoming bundle");
    let marker = root.join("installed-marker");
    std::fs::write(
        incoming.join("app"),
        format!(
            "#!/bin/sh\nprintf '%s' 'installed-app-ran' > '{}'\n",
            marker.display()
        ),
    )
    .expect("the fixture program");
    std::fs::set_permissions(incoming.join("app"), std::fs::Permissions::from_mode(0o755))
        .expect("chmod");
    std::fs::write(
        incoming.join("app.desktop"),
        "[Desktop Entry]\nType=Application\nName=Installed Fixture\nExec=app\n",
    )
    .expect("the bundle description");

    let socket = root.join("sysd.sock");
    let listener = litric_sysd::server::bind(&socket).expect("bind sysd");
    let state = litric_sysd::state::Sysd::new(2, 512, "/tmp", std::path::Path::new("/"))
        .with_application_root(&data);
    std::thread::spawn(move || {
        let _ = litric_sysd::server::serve(&listener, state);
    });

    let installed =
        litric_sysd::server::query(&socket, "APP INSTALL com.example.installed fixture")
            .expect("install over the socket");
    assert_eq!(
        installed, "OK 2\nid=com.example.installed\nstate=installed\n",
        "the daemon reports the install"
    );
    let installed_app = data.join("litric/apps/com.example.installed/app");
    assert!(installed_app.is_file(), "the executable was written");

    let apps = crate::desktop_entry::discover_in(&[data.join("applications")]);
    assert_eq!(apps.len(), 1, "the installed entry is discovered: {apps:?}");
    let app = apps.first().expect("the discovered application");
    assert_eq!(app.name, "Installed Fixture");
    assert_eq!(app.exec.len(), 1);
    let executable = app.exec.first().expect("the discovered executable");
    assert_eq!(std::path::Path::new(executable), installed_app);

    let launch = format!(
        "LAUNCH {}",
        litric_sysd::launch::hex_encode(executable.as_bytes())
    );
    let started = litric_sysd::server::query(&socket, &launch).expect("launch over the socket");
    assert!(started.starts_with("OK 2\npid="), "launch reply: {started}");

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut written: Option<String> = None;
    while Instant::now() < deadline {
        if let Ok(text) = std::fs::read_to_string(&marker) {
            written = Some(text);
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(
        written.as_deref(),
        Some("installed-app-ran"),
        "the installed copy ran and wrote its marker"
    );

    let removed = litric_sysd::server::query(&socket, "APP UNINSTALL com.example.installed")
        .expect("uninstall over the socket");
    assert_eq!(removed, "OK 2\nid=com.example.installed\nstate=removed\n");
    assert!(!installed_app.exists(), "the executable is gone");
    assert!(
        !data
            .join("applications/com.example.installed.desktop")
            .exists(),
        "the desktop entry is gone"
    );
    assert!(
        crate::desktop_entry::discover_in(&[data.join("applications")]).is_empty(),
        "discovery no longer lists the uninstalled app"
    );
    let _ = std::fs::remove_dir_all(&root);
}

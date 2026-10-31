#!/usr/bin/env python3
"""Check that every file-deletion call site lives in a function someone reviewed.

Why this exists. The workspace has no runtime test for the file layer: no booted
image, no compiled crate, no QEMU (see docs/audit/2026-10-09-continuation-recovery-audit.md).
What it does have is a small number of places that destroy data, and a set of
defects found in them (BUG-0010, BUG-0011, BUG-0012). A new
`fs::remove_dir_all` appearing in an unreviewed function is therefore a change
with the largest possible blast radius and the smallest possible test coverage.

What this checks:

  * Every `fs::remove_file(` / `fs::remove_dir_all(` call in `src/**.rs` must be
    inside a function named in ALLOWED, keyed by file and function name.
  * Each entry carries the reason it is allowed, and that reason is printed when
    the check passes, so the list reads as a review record rather than a
    suppression list.

What it deliberately does not do:

  * It does not prove the allowed sites are correct -- that is what the tests in
    `src/litric-files/src/ops.rs` and the trash modules are for, and those have
    not been executed in this mode.
  * It does not do Rust parsing. The enclosing function is found by scanning
    upwards for the nearest `fn` line, so a closure or a `macro_rules!` body
    around a deletion would be attributed to the function that contains it. That
    is the conservative direction: it makes the check stricter, not looser.
  * It does not run anything, write anything, or read anything outside `src/`.

Adding a deletion site is meant to require editing this file with a reason. That
is the review step, and it is cheaper than the alternative.

Usage:
    python3 ci/lint-file-safety.py             check
    python3 ci/lint-file-safety.py --self-test prove it can fail
"""

import os
import re
import sys
import tempfile

# file -> {function name: why deleting there is allowed}
#
# This is a review record, not a suppression list: every production deletion in
# src/ is named here with the reason it is safe to do it there. A new one is a
# FAIL until somebody adds a line with a reason.
ALLOWED = {
    "src/litric-files/src/ops.rs": {
        "remove_host": "the one choke point for deleting a path under the daemon root (ADR-0019 section 3)",
        "import_file": "rolls back the destination it created exclusively when the copy from the staging directory failed part-way; the file is the import's own, and a document that was already there was never opened (ADR-0019, bringing a browser download in)",
    },
    "src/litric-files/src/trash.rs": {
        "put": "rolls back the .trashinfo reservation when the move failed, so nothing left its place (BUG-0011)",
        "reserve_info": "removes the record it is creating when the write fails (BUG-0011)",
        "restore": "removes the record of an entry it has just put back where it came from",
        "purge": "permanent removal of one trash entry the user asked to purge; the name is validated and lives under files/",
    },
    "src/litric-files/src/server.rs": {
        "serve_unix": "unlinks a stale socket file before bind; never a user path",
        "bind_listener": "unlinks a stale socket file before bind (with a symlink_metadata refusal), and removes the socket it just bound when making it owner-only failed; never a user path",
    },
    "src/litric-shell/src/fsbridge.rs": {
        "move_item": "deletes the source only after a cross-device copy completed",
        "delete": "permanent removal of a validated logical path the user asked to delete",
        "trash": "rolls back the .trashinfo reservation when the move failed (the BUG-0011 shape, second implementation)",
        "trash_restore": "removes the record of an entry it has just restored",
        "reserve_trash_name": "removes the record it is creating when the write fails",
    },
    "src/litric-browser/src/bin/browserd.rs": {
        "run": "binds through `bind_listener`; never a user path",
        "bind_listener": "unlinks a stale socket file before bind (with a symlink_metadata refusal), and removes the socket it just bound when making it owner-only failed; never a user path",
        "scratch": "test helper: removes the scratch directory this test module is about to recreate; the name is tagged with this process's id and lives under the system temp directory",
        "a_live_brokers_socket_is_not_replaced_and_the_socket_is_owner_only": "test cleanup: removes the socket it bound under the system temp directory, after asserting the live socket was not replaced",
        "a_stale_brokers_socket_is_replaced_and_a_file_is_never_deleted": "test cleanup: removes the socket it bound under the system temp directory, after asserting the stale one was replaced and a plain file was not deleted",
        "drop_staged": "removes one finished download the shell has imported, from the broker's own --downloads staging directory: the name must be one segment and is symlink_metadata-checked to be a regular file, so a link or a directory is refused (ADR-0018 DROP)",
        "staging": "test helper: removes the scratch staging directory this test module is about to recreate; the name is tagged with this process's id and lives under the system temp directory",
        "a_staged_download_can_be_dropped_by_name": "test cleanup: removes the scratch staging directory it created under the system temp directory, after asserting the file was dropped",
        "dropping_refuses_a_path_a_link_and_a_directory": "test cleanup: removes the scratch staging directory it created under the system temp directory, after asserting nothing was removed by the refusals",
    },
    "src/litric-sysd/src/server.rs": {
        "bind": "replaces a stale socket file after a symlink_metadata check; never a user path",
    },
    "src/litric-notifyd/src/queue.rs": {
        "save": "removes only the queue's own temporary file (queue.conf.tmp.<pid>) -- before writing, and again when the write failed -- never a queue record",
    },
    "src/litric-notifyd/src/server.rs": {
        "bind": "replaces a stale socket file after a symlink_metadata check; never a user path",
    },
    "src/litric-shell/examples/render_bench.rs": {
        "main": "removes the frame-buffer device file the benchmark created itself",
    },
    "src/litric-bench/src/bin/litricos-test.rs": {
        "cmd_cleanup": "the explicit cleanup command; the path is the one the CLI was given",
    },
    "src/litric-files/src/storage.rs": {
        "scratch": "removes the scratch directory this test module is about to recreate; the name is tagged with this process's id and lives under the system temp directory",
        "a_new_root_gets_every_standard_folder_once": "removes the scratch root this test created under the system temp directory, after asserting the folders it was asked for exist in it",
        "a_standard_name_that_is_a_file_is_refused_not_replaced": "removes the scratch root this test created under the system temp directory, after asserting the file it planted survived the refusal",
        "a_symlinked_root_is_refused": "removes the scratch directory (real folder plus link) this test created under the system temp directory",
    },
    "src/litric-files/tests/filesd.rs": {
        "scratch": "removes the scratch directory this test module is about to recreate; the name is tagged with this process's id and lives under the system temp directory",
        "the_binary_serves_a_whole_files_session": "removes the fixture root the session test created under the system temp directory, after the daemon it ran has exited",
        "the_binary_refuses_a_root_it_cannot_serve": "removes the single scratch file this test placed where a storage root should be, after asserting the daemon refused to serve it",
        "the_binary_reports_what_it_would_serve": "removes the empty scratch root this test created under the system temp directory, after asserting --info created nothing in it",
    },
    "src/litric-shell/src/prefstore.rs": {
        "save": "removes only the store's own temporary file (settings.conf.tmp.<pid>) -- before writing, and again when the write failed -- never a preference record",
    },
    "src/litric-shell/src/quarantine.rs": {
        "quarantine_contains_the_file_and_restore_puts_it_back": "removes the scratch root this test created under the system temp directory, after asserting the file was contained and restored",
        "a_second_quarantine_never_replaces_the_first": "removes the scratch root this test created under the system temp directory, after asserting both entries stayed separate",
        "out_of_scope_and_missing_files_are_refused_without_moving_anything": "removes the scratch root this test created under the system temp directory, after asserting the refused source was untouched",
        "restore_refuses_to_overwrite_or_to_use_a_tampered_record": "removes the scratch root this test created under the system temp directory, after asserting the contained copy was kept",
        "a_link_is_refused_and_its_target_is_left_alone": "removes the scratch root this test created under the system temp directory, after asserting the link and its target were untouched",
        "tampered_bytes_are_refused_and_kept": "removes the scratch root this test created under the system temp directory, after asserting the tampered contained copy was kept",
        "a_legacy_record_restores_unverified": "removes the scratch root this test created under the system temp directory, after asserting the legacy entry was restored",
    },
    "src/litric-sysd/src/backup.rs": {
        "remove_any": "removes only a pid-tagged backup staging directory or a retained snapshot directory under the configured backup root; every path is built there, retention validates the id and inspects the entry first, and a symlink is unlinked rather than followed",
        "scratch": "test helper: removes the scratch backup root this test module is about to recreate; the name is tagged with this process's id and lives under the system temp directory",
        "cleanup": "test helper: removes the scratch backup root this test created under the system temp directory, after its assertions",
    },
    "src/litric-sysd/src/install.rs": {
        "install": "removes only the installer's own pid-tagged staging directory, or the target it has just committed when writing the desktop entry failed; the target is the validated application id below the configured app root, and the desktop path is the same id",
        "uninstall": "removes exactly the validated application directory and desktop entry the installer wrote; both paths are symlink_metadata-checked and a link is refused rather than followed",
        "write_atomic": "removes its own temporary desktop file (a pid-tagged sibling in the applications directory) before replacing it, never an application file",
        "scratch": "test helper: removes the scratch installer root this test module is about to recreate; the name is tagged with this process's id and lives under the system temp directory",
        "cleanup": "test helper: removes the scratch installer root this test created under the system temp directory, after its assertions",
    },
    "src/litric-shell/src/e2e_backup.rs": {
        "journey_auto_backup_verify_and_tamper_refusal": "removes the pid-tagged temporary application and backup root this journey created under the system temp directory, after asserting the timer-created snapshot, independent hashes, source mutation, tamper refusal, repair and manual create",
    },
    "src/litric-shell/src/e2e_app_install.rs": {
        "journey_app_install_launch_uninstall_through_sysd": "removes the pid-tagged temporary installer root this journey created under the system temp directory, after asserting the installed program wrote its marker and uninstall removed both the executable and its desktop entry",
        "journey_real_linux_app_is_installed_launched_mapped_rendered_and_removed": "removes the pid-tagged temporary root this journey created under the system temp directory, after asserting the real installed client mapped a window, its pixels reached views::draw, closing it produced Gone, and uninstall removed the executable and entry",
    },
    "src/litric-shell/src/e2e.rs": {
        "journey_settings_updates_reads_the_real_daemon_status": "removes the pid-tagged temporary fixture root this Settings journey created under the system temp directory, after asserting the real daemon reported an uninitialized update record and the page drew no Ready state",
        "e2e_root": "removes the temporary fixture root it is about to recreate; the name is tagged with this process's id and lives under the system temp directory",
        "journey_files_selects_with_mouse_and_compresses_multiple_items": "removes the fixture root this journey made with `e2e_root` -- tagged with this process's id under the system temp directory -- after asserting the archive it asked for exists in it",
        "journey_defender_scans_and_reports_eicar_end_to_end": "removes the temporary root this journey created",
        "journey_launcher_picks_up_a_same_length_rewrite": "removes the temporary XDG applications and icon-theme root this journey created under the system temp directory, named with its own pid, after asserting a same-length rewrite repainted the tile with its new artwork",
        "journey_launcher_picks_up_a_tail_only_rewrite": "removes the temporary XDG applications and icon-theme root this journey created under the system temp directory, named with its own pid, after asserting a rewrite past the head window repainted the tile with its new artwork",
        "journey_launcher_picks_up_a_rewritten_icon_file": "removes the temporary XDG applications and icon-theme root this journey created under the system temp directory, named with its own pid, after asserting a rewritten icon file repainted the tile with its new artwork",
        "journey_launcher_picks_up_an_icon_written_into_an_existing_theme": "removes the temporary XDG applications and icon-theme root this journey created under the system temp directory, named with its own pid, after asserting an icon written into an already-installed theme reached the launcher tile without a restart",
        "journey_launcher_re_resolves_only_the_name_a_new_theme_supplies": "removes the temporary XDG fixture root this launcher journey created with `e2e_root`, tagged with its own pid, after asserting the per-name resolve counts and the tiles' pixels",
        "journey_defender_fast_and_deep_scans_cover_different_folders": "removes the temporary fixture root this journey created, tagged with its own pid",
        "journey_this_litric_creates_renames_moves_and_trashes": "removes the temporary fixture root this journey created, tagged with its own pid",
        "journey_defender_scans_a_real_folder_fast_and_deep": "removes the temporary fixture root this Defender journey created, tagged with its own pid",
        "journey_browser_opens_a_real_comic_page_and_renders_it": "removes the temporary comic-page fixture this browser journey created, tagged with its own pid",
        "journey_a_download_opens_in_files_and_is_scanned": "removes the temporary Downloads fixture this cross-app journey created, tagged with its own pid",
        "journey_trash_window_restores_after_files_trash": "removes the temporary root this journey created",
        "journey_right_click_desktop_menu_creates_a_folder_via_dialog": "removes the temporary root this journey created",
        "journey_new_folder_from_the_files_menu_lands_in_the_open_folder": "removes the temporary root this journey created (BUG-0025: the row used to create in /Documents; the test asserts the folder landed in the open folder and that /Documents was untouched before cleaning up)",
        "journey_files_context_menu_creates_its_folder_in_the_open_folder": "removes the temporary root this journey created (BUG-0025, the context-menu route: it asserts the folder exists in the open folder and not in /Documents before cleaning up)",
        "journey_right_click_trash_row_targets_that_row": "removes the temporary root this journey created (BUG-0025, the Trash rows: it asserts the clicked row was the one restored and the previously selected row was not touched before cleaning up)",
        "journey_viewer_opens_text_files_with_scrolling": "removes the temporary root this journey created",
        "journey_defender_quarantines_and_restores_a_real_threat": "removes the temporary fixture root this Defender containment journey created, tagged with its own pid, after asserting the file was contained and restored",
        "journey_defender_quarantine_refreshes_an_open_files_window": "removes the temporary fixture root this journey created, tagged with its own pid, after asserting a containment move refreshes the open Files table",
        "journey_files_refresh_shows_a_change_made_outside_the_shell": "removes the temporary fixture root this journey created, tagged with its own pid, plus the empty folder it deletes to prove a failed refresh reports an error",
        "journey_files_watch_shows_an_outside_change_without_a_click": "removes the temporary fixture root this journey created, tagged with its own pid, plus the file and folder it deletes from outside the shell to prove the open table notices them without a click",
        "journey_trash_restore_refreshes_an_open_files_window": "removes the temporary fixture root this journey created, tagged with its own pid, after asserting a Trash restore refreshes the open Files table",
        "journey_launcher_lists_a_real_desktop_app_and_starts_it_through_sysd": "removes the fixture program (under this journey's own pid-tagged temp root) after asserting a launch that has lost its program is refused, then removes that root after asserting the real program wrote its marker",
        "journey_launcher_draws_the_icon_the_desktop_entry_names": "removes the temporary XDG applications and icon-theme tree this journey created under the system temp directory, named with its own pid, after asserting the themed pixels reached the launcher frame",
        "journey_launcher_resolves_a_shared_icon_name_once": "removes the temporary XDG applications and icon-theme root this journey created under the system temp directory, named with its own pid, after asserting two entries sharing one icon name resolved it once and drew it",
        "journey_launcher_reloads_only_the_changed_applications_directory": "removes the temporary home and system applications directories this journey created under the system temp directory, named with its own pid, after asserting the reload read only the directory that changed",
        "journey_launcher_notices_an_app_installed_while_it_runs": "removes the temporary XDG applications and icon-theme root this journey created under the system temp directory, named with its own pid, after asserting the entry written during the run appeared with its real icon and that removing it took the tile away",
        "journey_launcher_draws_an_svg_only_theme_icon": "removes the temporary XDG applications and scalable-only icon-theme tree this journey created under the system temp directory, named with its own pid, after asserting the SVG document's own colours reached the launcher tile and that removing its path changed the drawn pixels",
        "journey_defender_refuses_to_quarantine_a_file_that_is_gone": "removes the temporary fixture root this refusal journey created, tagged with its own pid, after asserting no quarantine entry was invented",
        "journey_defender_refuses_to_restore_a_tampered_quarantine_entry": "removes the temporary fixture root this tamper journey created, tagged with its own pid, after asserting the tampered copy was kept",
        "journey_defender_restores_a_legacy_quarantine_entry_unverified": "removes the temporary fixture root this legacy-record journey created, tagged with its own pid, after asserting the file was restored",
        "journey_hidden_files_toggle_is_a_real_read_and_survives_a_restart": "removes the temporary fixture root this hidden-visibility journey created, tagged with its own pid, after asserting the dot entry was absent before the switch, present after the real read, and present again in a restarted shell",
        "journey_do_not_disturb_persists_silences_the_banner_and_survives_a_restart": "removes the temporary fixture root this quiet-mode journey created, tagged with its own pid, after asserting the real banner disappeared from the composed frame, the queue kept the note, and the persisted choice survived a restart",
    },
    "src/litric-update/src/fsutil.rs": {
        "remove_any": "the one choke point for removing this crate's own staging directory, one of its two slot directories, or its state temporary; every path is built under <root>/system, never a user path",
    },
    "src/litric-sysd/src/providers.rs": {
        "apply_file": "removes one Wi-Fi credential; the path is refused unless it is a direct child of iwd's state directory under the daemon root, and the network name was validated by `valid_ssid` before the path was built",
    },
    "src/litric-sysd/src/grants.rs": {
        "save": "removes only the store's own temporary file (grants.conf.tmp.<pid>) -- before writing, and again when the write failed -- never a grant record",
        "fixture": "removes the temporary fixture root this test module is about to recreate; the name is tagged with this process's id and lives under the system temp directory",
        "cleanup": "removes the temporary fixture root this test created under the system temp directory, after asserting the record it wrote round-tripped or was refused",
    },
    "src/litric-sysd/src/lockout.rs": {
        "save": "removes only the lockout store's own temporary file (lockout.tmp.<pid>) before writing it, and again when the write failed; the final record is committed by rename and the path is built beside the store path, never a user path",
    },
    "src/litric-sysd/src/state.rs": {
        "grants_fixture": "removes the temporary fixture root this test module is about to recreate; the name is tagged with this process's id and lives under the system temp directory",
        "grants_cleanup": "removes the temporary fixture root this grant-restart test created under the system temp directory, after asserting the grants and audit it wrote were the ones a second daemon state loaded",
    },
    "src/litric-sysd/tests/wifi_secret.rs": {
        "a_wifi_passphrase_over_the_socket_never_reaches_the_reply_or_the_journal": "removes a socket file left by a previous run of this test, under the system temp directory with this process's id in its name, before the daemon binds it",

        "scratch": "removes the temporary fixture root this process-level Wi-Fi secret test is about to recreate; the name is tagged with this process's id and lives under the system temp directory",
        "cleanup": "removes the temporary fixture root and socket this test created under the system temp directory, after stopping the daemon and asserting the reply carried no passphrase",
    },
    "src/litric-defender/tests/scanner.rs": {
        "scratch": "removes the temporary fixture root this process-level scanner test is about to recreate; the name is tagged with this process's id and lives under the system temp directory",
        "cleanup": "removes the temporary fixture root (feed, EICAR, clean, sparse and missing-path fixtures) this test created under the system temp directory, after stopping the worker",
    },
    "src/litric-sysd/tests/grants_restart.rs": {
        "scratch": "removes the temporary fixture root this process-level test is about to recreate; the name is tagged with this process's id and lives under the system temp directory",
        "a_corrupt_record_makes_a_restarted_daemon_grant_nothing": "removes this test's own stale socket before starting the first daemon, tagged with the process id under the system temp directory",
        "cleanup": "removes the temporary fixture root and the socket this restart test created under the system temp directory, after stopping both daemons and asserting a corrupt record granted nothing",
    },
    "src/litric-sysd/src/update_lock.rs": {
        "drop": "removes only the lock record this process wrote, after re-reading it and comparing the exact contents; a reclaimer never removes a live owner's file, so a mismatch means it is not ours",
        "remove_stale": "removes the lock file only after `existing` classified it as stale (dead pid, or a malformed record past the initialising grace), under the configured update root, never a user path",
    },
    "src/litric-sysd/tests/update_service.rs": {
        "scratch": "removes the temporary fixture root this process-level update-service test is about to recreate; the name is tagged with this process's id under the system temp directory",
        "start": "removes this test's own stale socket, tagged with the process id and test tag under the system temp directory, before the daemon binds it",
        "cleanup": "removes the temporary fixture root this update-service test created under the system temp directory, after stopping the daemon and checking the generation state",
    },
    "src/litric-compositor/src/keymap.rs": {
        "load": "unlinks the keymap file it has just written into the session runtime directory, after the descriptor is open and before any client is given it: the path is built from the runtime directory the caller supplied, never a user path, and the open descriptor is what clients receive",
    },
    "src/litric-compositor/src/lib.rs": {
        "drop": "unlinks this server's own Wayland socket when the server is dropped; the path was built by `Server::start`, which refuses a runtime directory that is not absolute (asserted by `a_relative_runtime_directory_is_refused`), so it cannot name a user path",
    },
    "src/litric-compositor/src/tests.rs": {
        "a_frame_is_read_row_by_row_through_a_stride": "removes the single scratch pool file `scratch_file` created for this test, named with this process's id and a nanosecond stamp under the system temp directory",
        "an_offset_into_the_pool_is_honoured": "removes the single scratch pool file `scratch_file` created for this test, named with this process's id and a nanosecond stamp under the system temp directory",
        "a_buffer_that_names_memory_outside_the_pool_is_refused": "removes the single scratch pool file `scratch_file` created for this test, named with this process's id and a nanosecond stamp under the system temp directory",
    },
    "src/litric-compositor/tests/client.rs": {
        "scratch": "removes and recreates the per-test runtime directory it just named with this process's id, a nanosecond stamp and the test's own tag under the system temp directory",
        "a_foreign_client_maps_a_window_and_goes_away_cleanly": "removes the `scratch` runtime directory this test created, after asserting its foreign client mapped, painted, resized and exited",
        "a_client_that_vanishes_does_not_take_the_server_with_it": "removes the `scratch` runtime directory this test created, after asserting a killed client is reported as gone and the server still answers",
        "two_servers_refuse_the_same_socket": "removes the `scratch` runtime directory this test created, after asserting a second bind on the same socket is refused",
    },
    "src/litric-shell/tests/foreign_clients.rs": {
        "scratch": "removes and recreates the per-test session directory it just named with this process's id, a nanosecond stamp and the test's own tag under the system temp directory",
        "a_linux_application_opens_a_window_on_the_litric_desktop": "removes the `scratch` session directory this test created, after asserting the client it launched became a window and closing that window ended it",
        "a_command_typed_into_the_terminal_launches_a_graphical_application": "removes the `scratch` session directory this test created, after asserting the command typed into the Terminal reached it",
    },
    "src/litric-shell/src/main.rs": {
        "store_browser_data": "removes the `browser-data.conf.tmp.<pid>` file it created in the same call, and only on the failure path: the rename to the real path is what commits it, so the removal cannot touch anything but the temp file this process just made (BUG-0011 shape)",
        "store_preferences": "removes the `settings.conf.tmp.<pid>` file it created in the same call, and only when the commit rename failed, so the removal cannot touch anything but the temp file this process just made (BUG-0011 shape)",
    },
}


def blank(chunk):
    """`chunk` with everything replaced by a space -- except newlines.

    The newlines have to stay. `out.append(" " * (j - i))` in this function's
    first version blanked a multi-line string literal into a single run of
    spaces, which deleted the line breaks inside it, so every line below such a
    literal was reported one line higher than it really is (main.rs builds its
    `const USAGE` from continued string lines: 24 of them, so the check named
    line 1071 for a deletion on 1093 and printed the line after it as evidence).
    A linter that sends a reviewer to the wrong line is worse than no linter:
    the first thing a review does is look where it was pointed.
    """
    return "".join("\n" if ch == "\n" else " " for ch in chunk)


def code_only(text):
    """`text` with comments and string/char literal contents blanked out.

    Without this the checker flagged a comment that merely mentioned
    `fs::remove_file(x)` -- the same class of false positive the memory-budget
    checker had to handle for a directive inside a comment (BUG-0008). The
    scanner is deliberately small: line and block comments (nested, as Rust
    allows), `"…"` with escapes, `r"…"`/`r#"…"#` raw strings, and `'x'` char
    literals. Lifetimes (`'a`) are left alone. Newlines and byte offsets are
    preserved so a hit still points at the right line.
    """
    out = []
    i = 0
    n = len(text)
    block_depth = 0
    while i < n:
        ch = text[i]
        nxt = text[i + 1] if i + 1 < n else ""
        if block_depth:
            if ch == "/" and nxt == "*":
                block_depth += 1
                out.append("  ")
                i += 2
                continue
            if ch == "*" and nxt == "/":
                block_depth -= 1
                out.append("  ")
                i += 2
                continue
            out.append("\n" if ch == "\n" else " ")
            i += 1
            continue
        if ch == "/" and nxt == "/":
            while i < n and text[i] != "\n":
                out.append(" ")
                i += 1
            continue
        if ch == "/" and nxt == "*":
            block_depth = 1
            out.append("  ")
            i += 2
            continue
        # raw string: r"..." or r#"..."#
        if ch == "r" and (nxt == '"' or nxt == "#"):
            hashes = 0
            j = i + 1
            while j < n and text[j] == "#":
                hashes += 1
                j += 1
            if j < n and text[j] == '"':
                closer = '"' + "#" * hashes
                end = text.find(closer, j + 1)
                end = n if end == -1 else end + len(closer)
                out.append(blank(text[i:end]))
                i = end
                continue
        if ch == '"':
            j = i + 1
            while j < n:
                if text[j] == "\\":
                    j += 2
                    continue
                if text[j] == '"':
                    j += 1
                    break
                j += 1
            out.append(blank(text[i:j]))
            i = j
            continue
        if ch == "'":
            # a char literal or a lifetime; only the first is a literal
            j = i + 1
            if j < n and text[j] == "\\":
                j += 2
            else:
                j += 1
            if j < n and text[j] == "'":
                out.append(blank(text[i : j + 1]))
                i = j + 1
                continue
        out.append(ch)
        i += 1
    return "".join(out)


def scan_text(text, path):
    """Every deletion call site in `text` as (line_number, function_name, line).

    Sites inside a `#[cfg(test)]` module are skipped: they run only under a test
    build, and their target is a fixture in the temporary directory, not user
    data. The risky sites are the ones a booted system executes, and those are
    the ones that must be reviewed. The test-module gate is the first
    `#[cfg(test)]` in the file, which is how this workspace writes them.
    """
    raw_lines = text.splitlines()
    code_lines = code_only(text).splitlines()
    test_start = None
    for i, line in enumerate(raw_lines):
        if line.strip() == "#[cfg(test)]":
            test_start = i
            break
    fn = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?fn\s+([A-Za-z0-9_]+)")
    delete = re.compile(r"(?:std::)?fs::remove_(?:file|dir_all)\s*\(")
    found = []
    for i, line in enumerate(code_lines):
        if not delete.search(line):
            continue
        if test_start is not None and i >= test_start:
            continue
        name = "<top level>"
        for j in range(i, -1, -1):
            m = fn.match(code_lines[j])
            if m:
                name = m.group(1)
                break
        found.append((i + 1, name, raw_lines[i].strip(), False, path))
    return found


def collect(root):
    hits = []
    for dirpath, _dirnames, filenames in os.walk(root):
        for name in sorted(filenames):
            if not name.endswith(".rs"):
                continue
            full = os.path.join(dirpath, name)
            rel = os.path.relpath(full).replace(os.sep, "/")
            with open(full, encoding="utf-8", errors="replace") as handle:
                hits.extend(scan_text(handle.read(), rel))
    return hits


def decide(hits, allowed):
    """Split hits into (allowed, violations). Test-only sites are reported too:
    a deletion inside `#[cfg(test)]` still runs in a build with tests."""
    allowed_hits, violations = [], []
    for line_no, fname, line, _in_tests, path in hits:
        why = allowed.get(path, {}).get(fname)
        if why:
            allowed_hits.append((path, line_no, fname, why, line))
        else:
            violations.append((path, line_no, fname, line))
    return allowed_hits, violations


def report(allowed_hits, violations):
    if violations:
        print(f"FAIL: {len(violations)} unreviewed deletion site(s) in src/")
        for path, line_no, fname, line in violations:
            print(f"  {path}:{line_no} in fn {fname}()")
            print(f"      {line}")
        print()
        print("Each deletion must go through a reviewed function. Either route the")
        print("call through `Root::remove_host` (the one choke point in the file")
        print("layer) or add the (file, function) pair to ALLOWED in this file with")
        print("the reason it is safe.")
        return 1
    print(f"OK: {len(allowed_hits)} deletion site(s), all inside reviewed functions")
    for path, line_no, fname, why, _line in allowed_hits:
        print(f"  {path}:{line_no} fn {fname}() -- {why}")
    return 0


SELF_TEST_CASES = (
    (
        "a reviewed function passes",
        {
            "src/a.rs": (
                "fn remove_host(p: &str) {\n"
                "    let _ = std::fs::remove_dir_all(p);\n"
                "}\n"
            )
        },
        {"src/a.rs": {"remove_host": "choke point"}},
        0,
    ),
    (
        "a new function fails",
        {
            "src/a.rs": "fn cleanup(p: &str) {\n    let _ = std::fs::remove_file(p);\n}\n"
        },
        {"src/a.rs": {"remove_host": "choke point"}},
        1,
    ),
    (
        "a comment mentioning the call does not count",
        {"src/a.rs": "// fs::remove_file(x) was removed\nfn keep() {}\n"},
        {},
        0,
    ),
    (
        "a block comment does not count either",
        {
            "src/a.rs": (
                "/* the old code called fs::remove_dir_all(path) here */\n"
                "fn keep() {}\n"
            )
        },
        {},
        0,
    ),
    (
        "a call inside a string literal does not count",
        {"src/a.rs": 'fn keep() {\n    let s = "fs::remove_file(x)";\n}\n'},
        {},
        0,
    ),
    (
        "a deletion inside a test module needs no decision",
        {
            "src/a.rs": (
                "#[cfg(test)]\nmod tests {\n    fn fixture() {\n"
                "        let _ = std::fs::remove_dir_all(\"/tmp/x\");\n    }\n}\n"
            )
        },
        {},
        0,
    ),
    (
        "a raw string cannot hide a call after it",
        {
            "src/a.rs": (
                'fn keep() {\n    let s = r#"raw fs::remove_file(x)"#;\n'
                "    let _ = std::fs::remove_file(\"/tmp/y\");\n}\n"
            )
        },
        # `keep` is deliberately NOT allowed: the call inside the raw string is
        # text, the call after it is code, and only the second is a violation.
        {"src/a.rs": {"some_other_fn": "unrelated"}},
        1,
    ),
)


def self_test():
    """Prove the checker fails on what it is supposed to catch."""
    failures = 0
    for name, files, allowed, want in SELF_TEST_CASES:
        with tempfile.TemporaryDirectory() as tmp:
            for rel, body in files.items():
                full = os.path.join(tmp, rel)
                os.makedirs(os.path.dirname(full), exist_ok=True)
                with open(full, "w", encoding="utf-8") as handle:
                    handle.write(body)
            old_cwd = os.getcwd()
            os.chdir(tmp)
            try:
                hits = collect("src")
            finally:
                os.chdir(old_cwd)
            # Paths inside the fixture are relative to the fixture; match on the
            # allowlist exactly as the fixture wrote it.
            _allowed_hits, violations = decide(hits, allowed)
            got = 1 if violations else 0
            mark = "ok" if got == want else "NOT AS EXPECTED"
            if got != want:
                failures += 1
            print(f"  [{mark}] {name}: expected exit {want}, got {got}")
    if failures:
        print(f"FAIL: {failures} self-test case(s) behaved wrongly")
        return 1
    print(f"OK: {len(SELF_TEST_CASES)} self-test case(s) behaved as designed")
    return 0


def main(argv):
    # A console whose encoding cannot represent a character in a path or a
    # reason must not turn a verdict into a traceback: this checker was itself
    # crashing on a `section 3` sign under a cp874 console before that was
    # fixed, and the crash looked exactly like a passing run with output.
    try:
        sys.stdout.reconfigure(errors="replace")
    except (AttributeError, ValueError):
        pass
    if "--self-test" in argv:
        return self_test()
    root = "src"
    if not os.path.isdir(root):
        print(f"FAIL: no {root}/ directory -- run this from the repository root")
        return 1
    hits = collect(root)
    if not hits:
        print("FAIL: no deletion sites found at all, which cannot be right")
        print("      (a wrong root or an empty allowlist scan would hide every check)")
        return 1
    allowed_hits, violations = decide(hits, ALLOWED)
    return report(allowed_hits, violations)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

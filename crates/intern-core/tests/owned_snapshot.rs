use std::io::Write;

use intern_core::PrivateSnapshotDirectory;
use tempfile::tempdir;

#[test]
fn an_owned_snapshot_lives_in_a_private_child_and_cleans_only_that_child() {
    let temporary = tempdir().unwrap();
    let root = temporary.path().join("snapshots");
    let snapshots = PrivateSnapshotDirectory::new(&root).unwrap();
    let sentinel = root.join("keep.txt");
    std::fs::write(&sentinel, b"unrelated").unwrap();

    let mut writer = snapshots.create_for_supported_extension("pdf").unwrap();
    writer.write_all(b"verified bytes").unwrap();
    let snapshot = writer.finish().unwrap();
    let snapshot_path = snapshot.path().to_path_buf();
    let owned_directory = snapshot_path.parent().unwrap().to_path_buf();

    assert!(snapshot_path.starts_with(&root));
    assert_ne!(owned_directory, root);
    assert_eq!(std::fs::read(&snapshot_path).unwrap(), b"verified bytes");
    assert!(
        std::fs::metadata(&snapshot_path)
            .unwrap()
            .permissions()
            .readonly(),
        "a finished snapshot must reject path-based writes"
    );

    drop(snapshot);

    assert!(!snapshot_path.exists());
    assert!(!owned_directory.exists());
    assert_eq!(std::fs::read(sentinel).unwrap(), b"unrelated");
}

#[test]
fn dropping_an_unfinished_snapshot_removes_partial_bytes() {
    let temporary = tempdir().unwrap();
    let root = temporary.path().join("snapshots");
    let snapshots = PrivateSnapshotDirectory::new(&root).unwrap();

    let partial_path = {
        let mut writer = snapshots.create_for_supported_extension("pdf").unwrap();
        writer.write_all(b"partial").unwrap();
        writer.path().to_path_buf()
    };

    assert!(!partial_path.exists());
    assert!(std::fs::read_dir(root).unwrap().next().is_none());
}

#[cfg(unix)]
#[test]
fn snapshot_directories_and_files_are_owner_only() {
    use std::os::unix::fs::PermissionsExt;

    let temporary = tempdir().unwrap();
    let root = temporary.path().join("snapshots");
    let snapshots = PrivateSnapshotDirectory::new(&root).unwrap();
    let snapshot = snapshots
        .create_for_supported_extension("pdf")
        .unwrap()
        .finish()
        .unwrap();

    assert_eq!(
        std::fs::metadata(&root).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        std::fs::metadata(snapshot.path())
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o400
    );
}

#[cfg(windows)]
#[test]
fn a_live_owner_blocks_windows_mutation_and_replacement_then_cleans_the_snapshot() {
    use std::fs::OpenOptions;

    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().join("snapshots");
    let snapshots = PrivateSnapshotDirectory::new(&root).unwrap();
    let mut writer = snapshots.create_for_supported_extension("pdf").unwrap();
    writer.write_all(b"verified bytes").unwrap();
    let snapshot = writer.finish().unwrap();
    let snapshot_path = snapshot.path().to_path_buf();
    let owned_directory = snapshot_path.parent().unwrap().to_path_buf();
    let renamed = temporary.path().join("renamed");
    let replacement = temporary.path().join("replacement");
    std::fs::write(&replacement, b"replacement bytes").unwrap();

    assert!(OpenOptions::new().write(true).open(&snapshot_path).is_err());
    assert!(std::fs::rename(&snapshot_path, &renamed).is_err());
    assert!(std::fs::remove_file(&snapshot_path).is_err());
    assert!(std::fs::copy(&replacement, &snapshot_path).is_err());
    assert_eq!(std::fs::read(&snapshot_path).unwrap(), b"verified bytes");

    drop(snapshot);

    assert!(!snapshot_path.exists());
    assert!(!owned_directory.exists());
    assert_eq!(std::fs::read(replacement).unwrap(), b"replacement bytes");
}

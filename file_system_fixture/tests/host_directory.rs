//! Runs the host-directory harness in `examples/host_directory.rs` against a fresh temporary
//! directory, then damages the staged image and checks that the harness says where.

#[path = "../examples/host_directory.rs"]
#[allow(dead_code)]
mod host_directory;

use std::path::PathBuf;

/// A directory under the system's temporary directory, named for this process and `purpose`.
fn scratch(purpose: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "file_system_fixture-{}-{purpose}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    root
}

#[test]
fn a_staged_host_directory_verifies_and_a_damaged_one_does_not() {
    let root = scratch("host");
    host_directory::stage(&root).unwrap();
    assert_eq!(host_directory::verify(&root), Ok(()));
    assert!(
        host_directory::stage(&root).is_err(),
        "staging refuses a directory with files in it"
    );

    std::fs::write(root.join("sub").join("inner"), b"not the body").unwrap();
    assert_eq!(
        host_directory::verify(&root),
        Err("sub/inner differs at byte 0".into())
    );
    std::fs::remove_dir_all(&root).unwrap();
}

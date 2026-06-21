//! End-to-end tests for the `lc3dsm` command-line binary.

use std::path::PathBuf;
use std::process::Command;

fn lc3dsm() -> Command {
    Command::new(env!("CARGO_BIN_EXE_lc3dsm"))
}

fn example(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../examples")
        .join(name)
}

#[test]
fn disassembles_hello_world_to_a_readable_listing() {
    let output = lc3dsm()
        .arg(example("hello-world.obj"))
        .output()
        .expect("the disassembler runs");
    assert!(
        output.status.success(),
        "disassembling hello-world.obj should succeed"
    );

    let listing = String::from_utf8(output.stdout).expect("the listing is UTF-8");
    assert!(listing.contains(".ORIG x3000"), "{listing}");
    assert!(listing.contains("LEA R0, L_3003"), "{listing}");
    assert!(listing.contains("PUTS"), "{listing}");
    assert!(listing.contains("HALT"), "{listing}");
    // The greeting is data: its first character sits under the recovered label.
    assert!(
        listing
            .lines()
            .any(|line| line.starts_with("L_3003") && line.contains(".FILL x0048")),
        "{listing}"
    );
    assert!(listing.trim_end().ends_with(".END"), "{listing}");
}

#[test]
fn a_malformed_object_fails_with_a_diagnostic() {
    let path = std::env::temp_dir().join(format!("lc3dsm-bad-{}.obj", std::process::id()));
    // An odd byte length cannot divide into whole 16-bit words.
    std::fs::write(&path, [0x30, 0x00, 0x12]).expect("the object is written");
    let output = lc3dsm().arg(&path).output().expect("the disassembler runs");
    let _ = std::fs::remove_file(&path);

    assert!(!output.status.success(), "a malformed object should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("object file"), "diagnostic was: {stderr}");
}

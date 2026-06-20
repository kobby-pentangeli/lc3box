//! End-to-end tests for the `lc3as` command-line binary.

use std::path::PathBuf;
use std::process::Command;

fn lc3as() -> Command {
    Command::new(env!("CARGO_BIN_EXE_lc3as"))
}

fn example(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../examples")
        .join(name)
}

#[test]
fn assembles_hello_world_to_the_committed_object() {
    let output = std::env::temp_dir().join(format!("lc3as-hello-{}.obj", std::process::id()));
    let status = lc3as()
        .arg(example("hello-world.asm"))
        .args(["-o".as_ref(), output.as_os_str()])
        .status()
        .expect("the assembler runs");
    assert!(
        status.success(),
        "assembling hello-world.asm should succeed"
    );

    let produced = std::fs::read(&output).expect("an object file is written");
    let canonical = std::fs::read(example("hello-world.obj")).expect("the canonical object exists");
    let _ = std::fs::remove_file(&output);
    assert_eq!(produced, canonical);
}

#[test]
fn malformed_source_fails_with_a_line_numbered_diagnostic() {
    let source = std::env::temp_dir().join(format!("lc3as-bad-{}.asm", std::process::id()));
    std::fs::write(&source, ".ORIG x3000\nADD R0, R0\n.END\n").expect("the source is written");
    let result = lc3as().arg(&source).output().expect("the assembler runs");
    let _ = std::fs::remove_file(&source);

    assert!(!result.status.success(), "malformed source should fail");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("line 2"), "diagnostic was: {stderr}");
}

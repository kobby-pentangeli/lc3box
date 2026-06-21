//! End-to-end tests for the `lc3box` command-line binary.

use std::path::PathBuf;
use std::process::Command;

fn lc3box() -> Command {
    Command::new(env!("CARGO_BIN_EXE_lc3box"))
}

fn example(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../examples")
        .join(name)
}

#[test]
fn asm_assembles_hello_world_to_the_committed_object() {
    let output = std::env::temp_dir().join(format!("lc3box-hello-{}.obj", std::process::id()));
    let status = lc3box()
        .arg("asm")
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
fn asm_rejects_malformed_source_with_a_line_numbered_diagnostic() {
    let source = std::env::temp_dir().join(format!("lc3box-bad-{}.asm", std::process::id()));
    std::fs::write(&source, ".ORIG x3000\nADD R0, R0\n.END\n").expect("the source is written");
    let result = lc3box()
        .arg("asm")
        .arg(&source)
        .output()
        .expect("the assembler runs");
    let _ = std::fs::remove_file(&source);

    assert!(!result.status.success(), "malformed source should fail");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("line 2"), "diagnostic was: {stderr}");
}

#[test]
fn disasm_renders_hello_world_to_a_readable_listing() {
    let output = lc3box()
        .arg("disasm")
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
fn disasm_rejects_a_malformed_object_with_a_diagnostic() {
    let path = std::env::temp_dir().join(format!("lc3box-bad-{}.obj", std::process::id()));
    // An odd byte length cannot divide into whole 16-bit words.
    std::fs::write(&path, [0x30, 0x00, 0x12]).expect("the object is written");
    let output = lc3box()
        .arg("disasm")
        .arg(&path)
        .output()
        .expect("the disassembler runs");
    let _ = std::fs::remove_file(&path);

    assert!(!output.status.success(), "a malformed object should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("object file"), "diagnostic was: {stderr}");
}

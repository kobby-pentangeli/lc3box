//! Re-assembling a disassembly reproduces the original image:
//! `assemble(disassemble(obj)) == obj`

use std::path::PathBuf;

use lc3as::assemble;
use lc3core::ObjectFile;
use lc3dsm::disassemble;

fn example(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../examples")
        .join(name)
}

/// Disassembles `object`, re-assembles the listing, and asserts the single
/// resulting block is byte-for-byte the input.
fn assert_disassembly_re_assembles(object: &ObjectFile) {
    let listing = disassemble(object);
    let image = assemble(&listing).expect("a disassembly re-assembles");
    assert_eq!(image.blocks, vec![object.clone()], "\n{listing}");
}

#[test]
fn committed_objects_re_assemble_to_themselves() {
    for name in ["hello-world.obj", "rogue.obj", "2048.obj"] {
        let bytes = std::fs::read(example(name)).expect("the object exists");
        let object = ObjectFile::from_be_bytes(&bytes).expect("a well-formed object");
        assert_disassembly_re_assembles(&object);
    }
}

#[test]
fn assembled_sources_re_assemble_segment_by_segment() {
    for name in ["hello-world.asm", "bootstrap.asm"] {
        let source = std::fs::read_to_string(example(name)).expect("the source exists");
        let image = assemble(&source).expect("the source assembles");
        for block in &image.blocks {
            assert_disassembly_re_assembles(block);
        }
    }
}

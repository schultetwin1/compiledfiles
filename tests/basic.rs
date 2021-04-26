use spectral::prelude::*;

#[test]
fn basic_executable() {
    let root_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_dir = root_dir.join("tests").join("c");
    let hello_source = test_dir.join("hello.c").canonicalize().unwrap();

    let bin_path = std::path::PathBuf::from(env!("COMPILEDFILES_BASIC_TEST_BIN_PATH"));

    let elf_file = std::fs::File::open(&bin_path).unwrap();
    let files = compiledfiles::parse(elf_file).unwrap();

    assert_that!(files.iter().find(|&f| f.path == hello_source)).is_some();
}

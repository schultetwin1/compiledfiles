use spectral::prelude::*;
use std::path::PathBuf;

#[test]
fn basic_executable() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_dir = root_dir.join("tests").join("c");
    let hello_source = test_dir.join("hello.c").canonicalize().unwrap();

    let symbols_path = PathBuf::from(env!("COMPILEDFILES_BASIC_TEST_SYM_PATH"));

    let symbols_file = std::fs::File::open(&symbols_path).unwrap();
    let files = compiledfiles::parse(symbols_file).unwrap();

    assert_that!(files.iter().find(|&f| f.path == hello_source)).is_some();
}

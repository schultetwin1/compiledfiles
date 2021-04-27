use object::Object;
use spectral::prelude::*;
use std::io::Read;
use std::path::{Path, PathBuf};

fn has_debug_symbols<P>(path: P) -> bool
where
    P: AsRef<Path>,
{
    let mut bytes = vec![];
    let mut file = std::fs::File::open(path).unwrap();
    file.read_to_end(&mut bytes).unwrap();
    let obj = object::File::parse(&bytes).unwrap();
    obj.has_debug_symbols()
}

#[test]
fn basic_executable() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_dir = root_dir.join("tests").join("c");
    let hello_source = test_dir.join("hello.c").canonicalize().unwrap();

    let bin_path = PathBuf::from(env!("COMPILEDFILES_BASIC_TEST_BIN_PATH"));

    if has_debug_symbols(&bin_path) {
        let bin_file = std::fs::File::open(&bin_path).unwrap();
        let files = compiledfiles::parse(bin_file).unwrap();

        assert_that!(files.iter().find(|&f| f.path == hello_source)).is_some();
    }
}

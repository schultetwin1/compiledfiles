fn main() {
    let path_to_c_source = "tests/c/hello.c";
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed={}", path_to_c_source);
    println!("cargo:rerun-if-changed=build.rs");

    let compiler = cc::Build::new().file(path_to_c_source).get_compiler();
    let target = std::env::var("TARGET").unwrap();

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = std::path::PathBuf::from(out_dir);
    let input = std::path::PathBuf::from(path_to_c_source);
    let mut binary = out_dir.join("hello");
    let mut symbols = binary.clone();
    let mut cmd = compiler.to_command();
    if compiler.is_like_msvc() {
        binary.set_extension("exe");
        symbols.set_extension("pdb");
        cmd.args(&[
            input.to_str().unwrap(),
            "/LINK",
            &format!("/Fe:{}", binary.to_str().unwrap()),
        ]);
    } else {
        if target.contains("apple") {
            symbols.pop();
            symbols.push(format!(
                "{}.dSYM",
                binary.file_name().unwrap().to_str().unwrap()
            ));
            symbols.push("Contents");
            symbols.push("Resources");
            symbols.push("DWARF");
            symbols.push(binary.file_name().unwrap());
        }
        cmd.args(&[input.to_str().unwrap(), "-o", binary.to_str().unwrap()]);
    };

    println!("Running command: {:?}", cmd);

    let cmd = cmd.output().unwrap();

    println!("Output: {}", std::str::from_utf8(&cmd.stdout).unwrap());

    if !cmd.status.success() {
        panic!("Failed to compile test binary");
    }

    println!(
        "cargo:rustc-env=COMPILEDFILES_BASIC_TEST_BIN_PATH={}",
        binary.display()
    );
    println!(
        "cargo:rustc-env=COMPILEDFILES_BASIC_TEST_SYM_PATH={}",
        symbols.display()
    );
}

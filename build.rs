fn main() {
    let path_to_c_source = "tests/c/hello.c";
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed={}", path_to_c_source);
    println!("cargo:rerun-if-changed=build.rs");

    let compiler = cc::Build::new().file(path_to_c_source).get_compiler();

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = std::path::PathBuf::from(out_dir);
    let input = std::path::PathBuf::from(path_to_c_source);
    let output = out_dir.join("hello");
    let mut cmd = if compiler.is_like_msvc() {
        let mut cmd = compiler.to_command();
        cmd.args(&[
                input.to_str().unwrap(),
                "/LINK",
                &format!("/OUT:{}", output.to_str().unwrap()),
            ]);
        cmd
    } else {
        let mut cmd = compiler.to_command();
        cmd.args(&[input.to_str().unwrap(), "-o", output.to_str().unwrap()]);
        cmd
    };

    println!("Running command: {:?}", cmd);

    let cmd = cmd.output().unwrap();

    if !cmd.status.success() {
        panic!("Failed to compile test binary");
    }

    println!(
        "cargo:rustc-env=COMPILEDFILES_BASIC_TEST_BIN_PATH={}",
        output.display()
    );
    println!("Output: {}", std::str::from_utf8(&cmd.stdout).unwrap());
}

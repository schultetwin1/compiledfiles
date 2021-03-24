fn main() {
    let path_to_c_source = "tests/c/hello.c";
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed={}", path_to_c_source);
    println!("cargo:rerun-if-changed=build.rs");

    let compiler = cc::Build::new()
        .file(path_to_c_source)
        .get_compiler();

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = std::path::PathBuf::from(out_dir);
    let input = std::path::PathBuf::from(path_to_c_source);
    let output = out_dir.join("hello");
    let cmd = compiler.to_command()
        .args(&[input.to_str().unwrap(), "-o", output.to_str().unwrap()]).output().unwrap();

    println!("Output: {}", std::str::from_utf8(&cmd.stdout).unwrap());
}
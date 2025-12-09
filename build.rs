use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    lalrpop::process_root().unwrap();
    generate_integration_tests().unwrap();
}

fn generate_integration_tests() -> Result<(), std::io::Error> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("gen_tests.rs");

    let cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let tmp_dir = cargo_manifest_dir.join("target").join("e2e_artifacts");
    let _ = fs::create_dir_all(&tmp_dir).unwrap();

    let test_dir = cargo_manifest_dir
        .join("tests")
        .join("golden")
        .join("examples");

    let print_c = cargo_manifest_dir.join("src").join("print.c");

    let mut src = String::new();

    // src.push_str("use libcrate::run_file; use std::fs;\n");
    for entry in fs::read_dir(&test_dir).unwrap().filter_map(Result::ok) {
        let p = entry.path();

        if p.extension().and_then(|s| s.to_str()) != Some("bx") {
            continue;
        }

        let stem = p.file_stem().unwrap().to_str().unwrap();
        let asm_path = tmp_dir.join(format!("{}.s", stem));
        let exe_path = tmp_dir.join(format!("{}", stem));

        let exoutput_file = test_dir.join(format!("{}_output.txt", stem));

        src.push_str(&format!(
            r#"
#[test]
fn integration_{stem}() {{
    let p = PathBuf::from("{p}");
    let asm_path = PathBuf::from("{asm_path}");
    let _ = Driver::drive(&p, &asm_path);
    assert!(asm_path.is_file(), "driver failed for {stem}");

    let gcc_status = Command::new("gcc")
        .arg(asm_path.as_os_str())
        .arg("{print_c}")
        .arg("-o")
        .arg("{exe_path}")
        .status()
        .expect("failed to spawn gcc");
    assert!(gcc_status.success(), "gcc failed for {stem}");

    let mut run_cmd = Command::new("{exe_path}");
    run_cmd.stdout(Stdio::piped());
    let (stdout, _, _) = run(&mut run_cmd);
    // let (stdout, stderr, code) = run(&mut run_cmd);
    // assert_eq!(
    //     code, 0,
    //     "executable {stem} exited nonzero, stderr: {{}}",
    //     stderr
    // );

    let expected = fs::read_to_string("{exoutput_file}")
            .expect("missing expected output for {stem}");

    assert_eq!(stdout, expected, "output mismatch for {stem}");
}}
"#,
            p = p.display(),
            asm_path = asm_path.display(),
            print_c = print_c.display(),
            exe_path = exe_path.display(),
            exoutput_file = exoutput_file.display()
        ));
    }

    fs::write(dest_path, src)
}

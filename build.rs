use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    sync::LazyLock,
};

static OUT_DIR: LazyLock<OsString> = LazyLock::new(|| env::var_os("OUT_DIR").unwrap());

static CARGO_MANIFEST_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));

static TMP_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| CARGO_MANIFEST_DIR.join("target").join("e2e_artifacts"));

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=src/frontend/bxgrammar.lalrpop");
    println!("cargo:rerun-if-changed=tests/golden/examples");
    println!("cargo:rerun-if-changed=tests/golden/regression");
    println!("cargo:rerun-if-changed=bxruntime.c");

    lalrpop::process_root()?;
    generate_tests()?;

    Ok(())
}

fn generate_tests() -> Result<(), std::io::Error> {
    fs::create_dir_all(&*TMP_DIR)?;

    generate_integration_tests()?;
    generate_regression_tests()
}

fn generate_integration_tests() -> Result<(), std::io::Error> {
    let dest_path = Path::new(&*OUT_DIR).join("gen_tests.rs");

    let test_dir = CARGO_MANIFEST_DIR
        .join("tests")
        .join("golden")
        .join("examples");

    let print_c = CARGO_MANIFEST_DIR.join("bxruntime.c");

    let mut src = String::new();

    for entry in fs::read_dir(&test_dir)? {
        let p = entry?.path();

        if p.extension().and_then(|s| s.to_str()) != Some("bx") {
            continue;
        }

        let stem = p.file_stem().unwrap().to_str().unwrap();
        let asm_path = TMP_DIR.join(format!("{}.s", stem));
        let tac_path = TMP_DIR.join(format!("{}.tac", p.display()));
        let exe_path = TMP_DIR.join(stem);

        let exoutput_file = test_dir.join(format!("{}_output.txt", stem));

        src.push_str(&format!(
            r#"
#[test]
fn integration_{stem}() {{
    let p = PathBuf::from("{p}");
    let asm_path = PathBuf::from("{asm_path}");
    let tac_path = PathBuf::from("{tac_path}");

    let _ = fs::remove_file(&asm_path);
    let _ = fs::remove_file(&tac_path);

    Driver::drive(&p, &asm_path).unwrap_or_else(|e| panic!("driver failed for {stem}: {{e}}"));
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

    if stdout != expected {{
        panic!("golden output differs");
    }}
}}
"#,
            p = p.display(),
            asm_path = asm_path.display(),
            tac_path = tac_path.display(),
            print_c = print_c.display(),
            exe_path = exe_path.display(),
            exoutput_file = exoutput_file.display()
        ));
    }

    fs::write(dest_path, src)
}

fn generate_regression_tests() -> Result<(), std::io::Error> {
    let dest_path = Path::new(&*OUT_DIR).join("reg_tests.rs");

    let test_dir = CARGO_MANIFEST_DIR
        .join("tests")
        .join("golden")
        .join("regression");

    let mut src = String::new();

    for entry in fs::read_dir(&test_dir)? {
        let p = entry?.path();

        if p.extension().and_then(|s| s.to_str()) != Some("bx") {
            continue;
        }

        let stem = p.file_stem().unwrap().to_str().unwrap();
        let asm_path = TMP_DIR.join(format!("{}.s", stem));

        src.push_str(&format!(
            r#"
#[test]
#[should_panic]
fn regression_{stem}() {{
    let p = PathBuf::from("{p}");
    let asm_path = PathBuf::from("{asm_path}");
    Driver::drive(&p, &asm_path).unwrap();
}}
"#,
            p = p.display(),
            asm_path = asm_path.display(),
        ));
    }

    fs::write(dest_path, src)
}

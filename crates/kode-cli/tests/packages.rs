//! End-to-end tests for packages: a program importing a path dependency, the
//! lockfile, and the failure modes worth a good message.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn workspace(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kode-packages-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(path, text).expect("write");
}

fn kode(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kode"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run kode")
}

fn utils_package(dir: &Path) {
    write(
        &dir.join("libs/utils/kode.toml"),
        "[package]\nname = \"utils\"\nversion = \"0.1.0\"\nroot = \"src\"\n",
    );
    write(
        &dir.join("libs/utils/src/strings.kode"),
        "pub fn shout(s: string) -> string => s.upper() + \"!\"\n",
    );
    std::fs::create_dir_all(dir.join("libs/utils/src")).expect("mkdir");
}

fn app_package(dir: &Path, deps: &str) {
    write(
        &dir.join("app/kode.toml"),
        &format!("[package]\nname = \"app\"\nversion = \"0.1.0\"\nroot = \"src\"\n{deps}"),
    );
    write(
        &dir.join("app/src/main.kode"),
        "import \"utils/strings\" as u\nprint(u.shout(\"hi\"))\n",
    );
}

#[test]
fn runs_a_program_that_imports_a_dependency() {
    let dir = workspace("run");
    utils_package(&dir);
    app_package(&dir, "\n[dependencies]\nutils = { path = \"../libs/utils\" }\n");

    let out = kode(&dir.join("app"), &["run", "src/main.kode"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "HI!");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn install_writes_and_checks_the_lockfile() {
    let dir = workspace("install");
    utils_package(&dir);
    app_package(&dir, "\n[dependencies]\nutils = { path = \"../libs/utils\" }\n");
    let app = dir.join("app");

    let first = kode(&app, &["install"]);
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    let lock = std::fs::read_to_string(app.join("kode.lock")).expect("lock");
    assert!(lock.contains("name = \"utils\""), "{lock}");
    assert!(lock.contains("version = \"0.1.0\""), "{lock}");
    assert!(lock.contains("path = \"../libs/utils\""), "{lock}");

    // Running install twice must produce identical bytes.
    let again = std::fs::read_to_string(app.join("kode.lock")).unwrap();
    assert!(kode(&app, &["install"]).status.success());
    assert_eq!(again, std::fs::read_to_string(app.join("kode.lock")).unwrap());

    let check = kode(&app, &["install", "--check"]);
    assert!(check.status.success(), "{}", String::from_utf8_lossy(&check.stderr));

    std::fs::write(app.join("kode.lock"), "# stale\n").unwrap();
    assert_eq!(kode(&app, &["install", "--check"]).status.code(), Some(1));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_missing_dependency_folder_is_e0412() {
    let dir = workspace("missing");
    app_package(&dir, "\n[dependencies]\nutils = { path = \"../nowhere\" }\n");
    let out = kode(&dir.join("app"), &["check", "src/main.kode"]);
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("E0412"), "{stderr}");
    assert!(stderr.contains("utils"), "{stderr}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_version_mismatch_is_e0413() {
    let dir = workspace("version");
    utils_package(&dir);
    app_package(&dir, "\n[dependencies]\nutils = { path = \"../libs/utils\", version = \"9.9\" }\n");
    let out = kode(&dir.join("app"), &["check", "src/main.kode"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("E0413"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn init_creates_a_runnable_package_and_add_edits_it() {
    let dir = workspace("init");
    let out = kode(&dir, &["init", "notes"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(dir.join("kode.toml").is_file());
    assert!(dir.join("src/main.kode").is_file());

    let run = kode(&dir, &["run", "src/main.kode"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert!(String::from_utf8_lossy(&run.stdout).contains("hello, notes"));

    utils_package(&dir);
    let add = kode(&dir, &["add", "utils", "--path", "libs/utils"]);
    assert!(add.status.success(), "{}", String::from_utf8_lossy(&add.stderr));
    let manifest = std::fs::read_to_string(dir.join("kode.toml")).unwrap();
    assert!(manifest.contains("utils = { path = \"libs/utils\" }"), "{manifest}");
    assert!(dir.join("kode.lock").is_file());

    let remove = kode(&dir, &["remove", "utils"]);
    assert!(remove.status.success(), "{}", String::from_utf8_lossy(&remove.stderr));
    let manifest = std::fs::read_to_string(dir.join("kode.toml")).unwrap();
    assert!(!manifest.contains("utils ="), "{manifest}");
    let _ = std::fs::remove_dir_all(&dir);
}

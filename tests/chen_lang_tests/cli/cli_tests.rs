use assert_cmd::cargo_bin_cmd;

#[test]
fn cmd_test() {
    cargo_bin_cmd!().args(["-h"]).ok().unwrap();
}

#[test]
fn run_chen_js_extension_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("main.chen.js");
    std::fs::write(
        &file,
        r#"
console.log("chen-js")
"#,
    )
    .unwrap();

    cargo_bin_cmd!()
        .args(["run", file.to_str().unwrap()])
        .assert()
        .success()
        .stdout("chen-js\n");
}

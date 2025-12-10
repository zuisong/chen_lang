use std::fs;

use assert_cmd::cargo_bin_cmd;
use tempfile::TempDir;

/// 创建临时文件并运行chen_lang
pub fn run_chen_lang_code(code: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!();
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.ch");
    fs::write(&test_file, code)?;

    let output = cmd.arg("run").arg(&test_file).env("RUST_LOG", "off").output()?;

    if !output.status.success() {
        return Err(format!(
            "Execution failed: {}\nStderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(String::from_utf8(output.stdout)?)
}

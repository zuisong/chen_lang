use chen_lang::{report_error, run_captured, run_captured_with_vm_setup};

/// 创建临时文件并运行chen_lang
pub fn run_chen_lang_code(code: &str) -> Result<String, Box<dyn std::error::Error>> {
    let prelude = "import stdlib/io\nlet print = io.print\nlet println = io.println\n";
    let full_code = format!("{}{}", prelude, code);

    match run_captured(full_code.clone()) {
        Ok(output) => Ok(output),
        Err(e) => {
            let error_msg = report_error(&full_code, "test.ch", &e);
            Err(format!("Execution failed: {}\nStderr: {}", e, error_msg).into())
        }
    }
}

pub fn run_chen_lang_code_with_setup<F>(code: &str, setup: F) -> Result<String, Box<dyn std::error::Error>>
where
    F: FnOnce(&mut chen_lang::vm::VM),
{
    let prelude = "import stdlib/io\nlet print = io.print\nlet println = io.println\n";
    let full_code = format!("{}{}", prelude, code);

    match run_captured_with_vm_setup(full_code.clone(), setup) {
        Ok(output) => Ok(output),
        Err(e) => {
            let error_msg = report_error(&full_code, "test.ch", &e);
            Err(format!("Execution failed: {}\nStderr: {}", e, error_msg).into())
        }
    }
}

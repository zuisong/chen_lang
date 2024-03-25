use std::process::Command;

use assert_cmd::{cargo::CargoError, prelude::*};

fn command() -> Command {
    let runner = escargot::CargoBuild::new()
        .current_target()
        .run()
        .map_err(CargoError::with_cause)
        .unwrap();
    runner.command()
}

#[test]
fn cmd_test() {
    command().args(["-h"]).ok().unwrap();
}

use assert_cmd::cargo_bin_cmd;

#[test]
fn cmd_test() {
    cargo_bin_cmd!().args(["-h"]).ok().unwrap();
}

use std::process::Command;
use assert_cmd::prelude::*;

#[test]
fn command_update_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("installer")?;
    cmd.arg("update");
    cmd.assert().success();
    // cmd.assert().failure()
    //     .stdout(predicate::str::contains("更新失败"));
    // TODO: 考虑如何准备好前置条件，以做详细的集成测试。
    Ok(())
}
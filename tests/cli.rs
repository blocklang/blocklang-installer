use std::process::Command;
use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn update_success() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.arg("update");
    cmd.assert().success()
        .stdout(predicate::str::contains("更新成功"));

    Ok(())
}
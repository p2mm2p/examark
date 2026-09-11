use std::process::{Command, Output};

fn examark(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_examark"))
        .args(args)
        .output()
        .expect("examark 二进制应可运行")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn version_flag_prints_format_name_and_version() {
    let output = examark(&["--version"]);

    assert!(output.status.success());
    assert_eq!(
        stdout(&output).trim(),
        format!("examark {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn short_version_flag_matches_long_version_flag() {
    assert_eq!(stdout(&examark(&["-V"])), stdout(&examark(&["--version"])));
}

#[test]
fn help_flag_prints_usage() {
    let output = examark(&["--help"]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("用法"), "用法应出现：{text}");
    assert!(text.contains("build"), "命令列表应出现：{text}");
}

#[test]
fn no_arguments_prints_usage_and_exits_nonzero() {
    let output = examark(&[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("用法"));
}

#[test]
fn unknown_command_exits_nonzero() {
    let output = examark(&["不存在的命令"]);

    assert!(!output.status.success());
    assert!(!stderr(&output).is_empty());
}

//! examark 命令行工具：build/watch/preview。

use std::process::ExitCode;

use examark_syntax::FORMAT_NAME;

const USAGE: &str = "\
用法：examark <命令> [选项]

命令：
  build      将题目文档构建为 HTML
  watch      监视源文件，保存时重建
  preview    在本地提供构建输出

选项：
  -h, --help     显示本帮助
  -V, --version  显示版本
";

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("-h" | "--help") => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some("-V" | "--version") => {
            println!("{FORMAT_NAME} {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        _ => {
            eprint!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

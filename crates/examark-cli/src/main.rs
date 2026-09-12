//! examark 命令行工具：build/watch/preview。

mod build;

use std::path::PathBuf;
use std::process::ExitCode;

use examark_syntax::FORMAT_NAME;

const USAGE: &str = "\
用法：examark <命令> [选项]

命令：
  build <文档>   把题目文档构建为可整体移动、可分享的输出
  watch          监视源文件，保存时重建
  preview        在本地提供构建输出

build 选项：
  -o, --output <目录>  输出目录；默认 dist

选项：
  -h, --help     显示本帮助
  -V, --version  显示版本
";

/// build 的默认输出目录。
const DEFAULT_OUTPUT: &str = "dist";

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);

    match arguments.next().as_deref() {
        Some("-h" | "--help") => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some("-V" | "--version") => {
            println!("{FORMAT_NAME} {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("build") => run_build(arguments),
        _ => usage_error(),
    }
}

/// build 命令的参数。
enum BuildArguments {
    Help,
    Run { document: PathBuf, output: PathBuf },
}

fn run_build(arguments: impl Iterator<Item = String>) -> ExitCode {
    match parse_build_arguments(arguments) {
        Ok(BuildArguments::Help) => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        Ok(BuildArguments::Run { document, output }) => match build::build(&document, &output) {
            Ok(target) => {
                println!("构建完成：{}", target.display());
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        },
        Err(message) => {
            eprintln!("{message}");
            usage_error()
        }
    }
}

fn parse_build_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<BuildArguments, String> {
    let mut document = None;
    let mut output = None;

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "-h" | "--help" => return Ok(BuildArguments::Help),
            "-o" | "--output" => {
                let directory = arguments
                    .next()
                    .ok_or_else(|| format!("{argument} 后面要跟输出目录"))?;
                output = Some(PathBuf::from(directory));
            }
            _ if argument.starts_with('-') => return Err(format!("不认识的选项：{argument}")),
            _ if document.is_none() => document = Some(PathBuf::from(argument)),
            _ => return Err(format!("多余的参数：{argument}")),
        }
    }

    let document = document.ok_or_else(|| "build 需要一份题目文档".to_string())?;

    Ok(BuildArguments::Run {
        document,
        output: output.unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT)),
    })
}

fn usage_error() -> ExitCode {
    eprint!("{USAGE}");
    ExitCode::from(2)
}

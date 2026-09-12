//! examark 命令行工具：build/watch/preview/toolchain。

mod build;
mod preview;
mod toolchain;
mod watch;

use std::fmt;
use std::path::PathBuf;
use std::process::ExitCode;

use examark_syntax::FORMAT_NAME;

const USAGE: &str = "\
用法：examark <命令> <文档> [选项]

命令：
  build      把题目文档构建为可整体移动、可分享的输出
  watch      构建一次后守着源文档，保存即重建
  preview    构建一次后在本地提供输出，可用浏览器查看
  toolchain  依次运行 build、watch 与 preview：构建、本地服务、保存即重建

选项：
  -o, --output <目录>  输出目录；默认 dist
  -p, --port <端口>    本地服务端口，只用于 preview 与 toolchain；默认 8080，0 表示由系统分配
  -h, --help           显示本帮助
  -V, --version        显示版本
";

/// build 的默认输出目录。
const DEFAULT_OUTPUT: &str = "dist";

/// 本地服务的默认端口。
const DEFAULT_PORT: u16 = 8080;

/// 一条命令。
#[derive(Clone, Copy)]
enum Command {
    Build,
    Watch,
    Preview,
    Toolchain,
}

impl Command {
    /// 从命令名认出一条命令；认不出时返回 `None`。
    fn parse(name: &str) -> Option<Self> {
        match name {
            "build" => Some(Self::Build),
            "watch" => Some(Self::Watch),
            "preview" => Some(Self::Preview),
            "toolchain" => Some(Self::Toolchain),
            _ => None,
        }
    }

    /// 它是否把输出在本地提供出去；只有这样的命令认 `--port`。
    fn serves(self) -> bool {
        matches!(self, Self::Preview | Self::Toolchain)
    }
}

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);

    let Some(name) = arguments.next() else {
        return usage_error();
    };

    match name.as_str() {
        "-h" | "--help" => print!("{USAGE}"),
        "-V" | "--version" => println!("{FORMAT_NAME} {}", env!("CARGO_PKG_VERSION")),
        name => {
            let Some(command) = Command::parse(name) else {
                return usage_error();
            };

            return run(command, arguments);
        }
    }

    ExitCode::SUCCESS
}

/// 跑一条命令。
fn run(command: Command, arguments: impl Iterator<Item = String>) -> ExitCode {
    let options = match parse_arguments(command, arguments) {
        Ok(Arguments::Help) => {
            print!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Ok(Arguments::Run(options)) => options,
        Err(message) => {
            eprintln!("{message}");
            return usage_error();
        }
    };

    match command {
        Command::Build => report(build::build_and_report(&options.document, &options.output)),
        Command::Watch => report(watch::watch(&options.document, &options.output)),
        Command::Preview => report(preview::preview(
            &options.document,
            &options.output,
            options.port,
        )),
        Command::Toolchain => report(toolchain::toolchain(
            &options.document,
            &options.output,
            options.port,
        )),
    }
}

/// 把命令的结局换成退出码；长驻命令本就不返回，只有起步失败才走到这里。
fn report<T, E: fmt::Display>(result: Result<T, E>) -> ExitCode {
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

/// 一条命令的参数。
struct Options {
    document: PathBuf,
    output: PathBuf,
    port: u16,
}

/// 参数解析的结果。
enum Arguments {
    Help,
    Run(Options),
}

fn parse_arguments(
    command: Command,
    mut arguments: impl Iterator<Item = String>,
) -> Result<Arguments, String> {
    let mut document = None;
    let mut output = None;
    let mut port = None;

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "-h" | "--help" => return Ok(Arguments::Help),
            "-o" | "--output" => {
                let directory = arguments
                    .next()
                    .ok_or_else(|| format!("{argument} 后面要跟输出目录"))?;
                output = Some(PathBuf::from(directory));
            }
            "-p" | "--port" if command.serves() => {
                let value = arguments
                    .next()
                    .ok_or_else(|| format!("{argument} 后面要跟端口号"))?;
                port = Some(
                    value
                        .parse::<u16>()
                        .map_err(|_| format!("端口号应是 0 到 65535 之间的整数：{value}"))?,
                );
            }
            _ if argument.starts_with('-') => return Err(format!("不认识的选项：{argument}")),
            _ if document.is_none() => document = Some(PathBuf::from(argument)),
            _ => return Err(format!("多余的参数：{argument}")),
        }
    }

    let document = document.ok_or_else(|| "需要一份题目文档".to_string())?;

    Ok(Arguments::Run(Options {
        document,
        output: output.unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT)),
        port: port.unwrap_or(DEFAULT_PORT),
    }))
}

fn usage_error() -> ExitCode {
    eprint!("{USAGE}");
    ExitCode::from(2)
}

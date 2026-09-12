//! `preview` 命令：构建一次，然后把输出目录在本地用 HTTP 提供出去。

use std::fmt;
use std::fs;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use crate::build::{self, BuildError};

/// 只监听本地回环地址：预览服务不外扩到局域网。
const HOST: [u8; 4] = [127, 0, 0, 1];

/// 请求头的字节上限；超过就关上连接，免得被撑爆内存。
const MAX_HEAD: usize = 8 * 1024;

/// 读一条请求的等待上限：浏览器的预连接会连上却不发请求。
const READ_TIMEOUT: Duration = Duration::from_secs(5);

/// HTML 的 Content-Type。
const HTML: &str = "text/html; charset=utf-8";

/// 输出目录里没有这个文件时的回应。
const NOT_FOUND: &str = "\
<!DOCTYPE html>
<html lang=\"zh-CN\">
<head><meta charset=\"utf-8\"><title>找不到</title></head>
<body><p>输出目录里没有这个文件。</p></body>
</html>
";

/// 方法不是 GET 或 HEAD 时的回应。
const METHOD_NOT_ALLOWED: &str = "\
<!DOCTYPE html>
<html lang=\"zh-CN\">
<head><meta charset=\"utf-8\"><title>不接受这个请求</title></head>
<body><p>本地预览只提供 GET 与 HEAD 请求。</p></body>
</html>
";

/// preview 起步失败：构建不成，或本地服务起不来。
#[derive(Debug)]
pub enum PreviewError {
    /// 构建失败：没有可提供的东西。
    Build(BuildError),
    /// 端口已被别的进程占着。
    PortInUse(u16),
    /// 其他原因导致的监听失败。
    Listen {
        address: SocketAddr,
        source: io::Error,
    },
}

impl fmt::Display for PreviewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Build(error) => write!(formatter, "{error}"),
            Self::PortInUse(port) => {
                write!(formatter, "端口 {port} 已被占用；换一个端口：--port <端口>")
            }
            Self::Listen { address, source } => {
                write!(formatter, "无法监听 {address}：{source}")
            }
        }
    }
}

impl std::error::Error for PreviewError {}

impl From<BuildError> for PreviewError {
    fn from(error: BuildError) -> Self {
        Self::Build(error)
    }
}

/// 构建一次，然后把输出目录在本地提供出去；只在起步失败时返回。
pub fn preview(document: &Path, output: &Path, port: u16) -> Result<(), PreviewError> {
    let (listener, address, root) = prepare(document, output, port)?;

    serve(listener, address, output, &root)
}

/// 构建一次，然后在后台提供服务，调用方回去跑自己的循环；只在起步失败时返回。
pub fn serve_in_background(document: &Path, output: &Path, port: u16) -> Result<(), PreviewError> {
    let (listener, address, root) = prepare(document, output, port)?;
    let output = output.to_path_buf();

    thread::spawn(move || serve(listener, address, &output, &root));

    Ok(())
}

/// 构建一次并占住端口，返回监听器、它绑到的地址与访问 `/` 时提供的文件。
fn prepare(
    document: &Path,
    output: &Path,
    port: u16,
) -> Result<(TcpListener, SocketAddr, String), PreviewError> {
    build::build_and_report(document, output)?;

    let root = build::html_name(document)?;
    let (listener, address) = bind(port)?;

    Ok((listener, address, root))
}

/// 在本地回环地址上监听 `port`，返回监听器与它绑到的实际地址；`0` 表示由系统分配空闲端口。
fn bind(port: u16) -> Result<(TcpListener, SocketAddr), PreviewError> {
    let requested = SocketAddr::from((HOST, port));

    let listener = TcpListener::bind(requested).map_err(|error| match error.kind() {
        io::ErrorKind::AddrInUse => PreviewError::PortInUse(port),
        _ => PreviewError::Listen {
            address: requested,
            source: error,
        },
    })?;

    let address = listener
        .local_addr()
        .map_err(|error| PreviewError::Listen {
            address: requested,
            source: error,
        })?;

    Ok((listener, address))
}

/// 一直提供 `output` 目录里的文件，直到进程被结束；`root` 是访问 `/` 时提供的文件。
fn serve(listener: TcpListener, address: SocketAddr, output: &Path, root: &str) -> ! {
    println!("本地预览：http://{address}/");

    loop {
        let Ok((stream, _)) = listener.accept() else {
            continue;
        };

        let output = output.to_path_buf();
        let root = root.to_string();

        thread::spawn(move || {
            let _ = respond(stream, &output, &root);
        });
    }
}

/// 处理一条连接：读一条请求，写一条回应，然后关上。
fn respond(mut stream: TcpStream, output: &Path, root: &str) -> io::Result<()> {
    stream.set_read_timeout(Some(READ_TIMEOUT))?;

    let Some(head) = read_head(&mut stream)? else {
        return Ok(());
    };

    // 请求行是「方法 目标 版本」。
    let mut parts = head.lines().next().unwrap_or_default().split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or_default();

    let head_only = match method {
        "GET" => false,
        "HEAD" => true,
        _ => {
            return write_response(
                &mut stream,
                405,
                "Method Not Allowed",
                HTML,
                METHOD_NOT_ALLOWED.as_bytes(),
                false,
            );
        }
    };

    let Some(file) = requested_file(target, root) else {
        return not_found(&mut stream, head_only);
    };

    match fs::read(output.join(&file)) {
        Ok(body) => write_response(
            &mut stream,
            200,
            "OK",
            content_type(&file),
            &body,
            head_only,
        ),
        Err(_) => {
            println!("输出目录里没有 {}（HTTP 404）", file.display());
            not_found(&mut stream, head_only)
        }
    }
}

/// 回一条「输出目录里没有这个文件」。
fn not_found(stream: &mut TcpStream, head_only: bool) -> io::Result<()> {
    write_response(
        stream,
        404,
        "Not Found",
        HTML,
        NOT_FOUND.as_bytes(),
        head_only,
    )
}

/// 请求对应的文件：访问 `/` 时是 `root`，其余是输出目录里的相对路径。
///
/// 越出输出目录（`..`）、解不成 UTF-8、或目标不是以 `/` 开头时返回 `None`。
fn requested_file(target: &str, root: &str) -> Option<PathBuf> {
    let path = target.split(['?', '#']).next().unwrap_or_default();
    let decoded = percent_decode(path.strip_prefix('/')?)?;

    let mut file = PathBuf::new();
    for part in decoded.split('/') {
        match part {
            "" | "." => continue,
            ".." => return None,
            part => file.push(part),
        }
    }

    if file.as_os_str().is_empty() {
        return Some(PathBuf::from(root));
    }

    Some(file)
}

/// 把 URL 里的 `%XX` 还原成字节；解不成 UTF-8 时返回 `None`。
fn percent_decode(target: &str) -> Option<String> {
    let bytes = target.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hex = std::str::from_utf8(bytes.get(index + 1..index + 3)?).ok()?;
            decoded.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8(decoded).ok()
}

/// 按后缀定 Content-Type；认不出的按二进制流给。
fn content_type(path: &Path) -> &'static str {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);

    match extension.as_deref() {
        Some("html" | "htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("txt") => "text/plain; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        _ => "application/octet-stream",
    }
}

/// 读完一条请求的头部，到结束空行为止；对端提前关上或头部超长时返回 `None`。
fn read_head(stream: &mut TcpStream) -> io::Result<Option<String>> {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];

    while head.len() < MAX_HEAD {
        if stream.read(&mut byte)? == 0 {
            return Ok(None);
        }

        head.push(byte[0]);

        if head.ends_with(b"\r\n\r\n") {
            return Ok(Some(String::from_utf8_lossy(&head).into_owned()));
        }
    }

    Ok(None)
}

/// 写一条回应；`head_only` 时只写头（HEAD 请求）。
///
/// `Cache-Control: no-store` 是给编辑循环的：重建之后浏览器刷新该看到新内容，而不是缓存。
fn write_response(
    stream: &mut TcpStream,
    status: u16,
    reason: &str,
    content_type: &str,
    body: &[u8],
    head_only: bool,
) -> io::Result<()> {
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {}\r\n\
         Cache-Control: no-store\r\n\
         Connection: close\r\n\
         \r\n",
        body.len()
    );

    stream.write_all(head.as_bytes())?;

    if !head_only {
        stream.write_all(body)?;
    }

    stream.flush()
}

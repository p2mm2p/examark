use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

fn examark(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_examark"))
        .args(args)
        .output()
        .expect("examark 二进制应可运行")
}

/// 在指定工作目录里运行 examark，用于默认输出目录这类与 cwd 有关的行为。
fn examark_in(directory: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_examark"))
        .current_dir(directory)
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

/// fixture 目录：题目文档与它引用的图片资源。
fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// fixture 中某份题目文档的绝对路径。
fn fixture(document: &str) -> String {
    fixtures()
        .join(document)
        .to_str()
        .expect("fixture 路径应是 UTF-8")
        .to_string()
}

/// 测试用临时目录：每次取到互不相同的路径，析构时连同内容一起删除。
struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "examark-cli-{}-{name}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));

        fs::create_dir_all(&path).expect("临时目录应可创建");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// 在命令边界上跑一次 build：`examark build <文档> -o <输出目录>`。
fn build_into(document: &Path, output: &Path) -> Output {
    examark(&[
        "build",
        document.to_str().expect("路径应是 UTF-8"),
        "-o",
        output.to_str().expect("路径应是 UTF-8"),
    ])
}

/// 把 `图文文档.examark` 构建进一个临时目录，连同命令输出一起返回。
fn build_fixture(name: &str) -> (TempDir, Output) {
    let output = TempDir::new(name);
    let result = build_into(&fixtures().join("图文文档.examark"), output.path());

    (output, result)
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

#[test]
fn build_writes_a_standalone_html_document() {
    let (output, result) = build_fixture("html");

    assert!(result.status.success(), "stderr：{}", stderr(&result));

    let html =
        fs::read_to_string(output.path().join("图文文档.html")).expect("输出目录里应有 HTML 文件");
    assert!(html.starts_with("<!DOCTYPE html>"), "{html}");
    assert!(html.contains("<title>图文题目文档</title>"), "{html}");
    assert!(html.contains("答案：A"), "{html}");
    assert!(
        stdout(&result).contains("图文文档.html"),
        "成功时应报出产出位置：{}",
        stdout(&result)
    );
}

#[test]
fn build_copies_referenced_images_into_assets_and_rewrites_references() {
    let (output, result) = build_fixture("assets");

    assert!(result.status.success(), "stderr：{}", stderr(&result));

    let html =
        fs::read_to_string(output.path().join("图文文档.html")).expect("输出目录里应有 HTML 文件");

    for (authored, copied) in [
        ("assets/图1.png", "assets/图1.png"),
        ("assets/表1.png", "assets/表1.png"),
        ("插图/散图.png", "assets/散图.png"),
    ] {
        assert!(
            html.contains(&format!(r#"<img src="{copied}" alt="">"#)),
            "引用应指向输出目录里的 {copied}：{html}"
        );
        assert_same_bytes(&output.path().join(copied), &fixtures().join(authored));
    }

    assert!(
        !html.contains("插图/散图.png"),
        "作者书写的相对路径应被重写成输出目录里的位置：{html}"
    );
}

/// 断言产物里的资源与 fixture 里的同名文件逐字节相同。
fn assert_same_bytes(copied: &Path, original: &Path) {
    let copied_bytes = fs::read(copied)
        .unwrap_or_else(|error| panic!("{} 应被拷进输出目录：{error}", copied.display()));

    assert_eq!(
        copied_bytes,
        fs::read(original).expect("fixture 资源应存在"),
        "{} 应与 {} 逐字节相同",
        copied.display(),
        original.display()
    );
}

#[test]
fn the_output_tree_stays_portable_wherever_it_is_moved() {
    let (output, result) = build_fixture("portable");

    assert!(result.status.success(), "stderr：{}", stderr(&result));

    let moved = TempDir::new("moved");
    let target = moved.path().join("整卷输出");
    fs::rename(output.path(), &target).expect("输出目录应可整体移动");

    let html = fs::read_to_string(target.join("图文文档.html")).expect("移动后 HTML 仍在");
    let references = image_references(&html);

    assert_eq!(references.len(), 3, "三处图片引用都该在：{references:?}");
    for reference in references {
        assert!(
            target.join(&reference).is_file(),
            "整体移动后 {reference} 应仍指向存在的文件"
        );
    }
}

/// HTML 里的图片引用：`src` 属性的值。
fn image_references(html: &str) -> Vec<String> {
    html.match_indices("src=\"")
        .map(|(index, marker)| {
            let rest = &html[index + marker.len()..];
            rest[..rest.find('"').expect("src 属性应有闭合引号")].to_string()
        })
        .collect()
}

#[test]
fn build_reports_a_document_that_cannot_be_read() {
    let directory = TempDir::new("missing-document");
    let missing = directory.path().join("不存在的文档.examark");

    let result = build_into(&missing, directory.path());

    assert!(!result.status.success());
    assert!(
        stderr(&result).contains("不存在的文档.examark"),
        "错误应指出是哪个文档：{}",
        stderr(&result)
    );
}

#[test]
fn build_reports_a_parse_error_with_its_line() {
    let directory = TempDir::new("parse-error");
    let document = directory.path().join("常识.examark");
    fs::write(&document, "@module 常识判断\n").expect("临时文档应可写");

    let result = build_into(&document, directory.path());

    assert!(!result.status.success());
    let message = stderr(&result);
    assert!(message.contains("第 1 行"), "{message}");
    assert!(message.contains("常识判断"), "{message}");
}

#[test]
fn build_reports_an_image_that_is_missing() {
    let directory = TempDir::new("missing-image");
    let document = directory.path().join("缺图.examark");
    fs::write(&document, document_with_image("assets/不存在.png")).expect("临时文档应可写");

    let result = build_into(&document, directory.path());

    assert!(!result.status.success());
    let message = stderr(&result);
    assert!(message.contains("assets/不存在.png"), "{message}");
    assert!(
        !directory.path().join("缺图.html").exists(),
        "失败时不该留下 HTML"
    );
}

#[test]
fn build_reports_two_images_that_share_a_file_name() {
    let directory = TempDir::new("clashing-images");
    let document = directory.path().join("重名.examark");
    fs::write(
        &document,
        document_with_image("甲/图示.png 与 @image{乙/图示.png}"),
    )
    .expect("临时文档应可写");

    let result = build_into(&document, directory.path());

    assert!(!result.status.success());
    let message = stderr(&result);
    assert!(message.contains("甲/图示.png"), "{message}");
    assert!(message.contains("乙/图示.png"), "{message}");
}

#[test]
fn build_defaults_to_the_dist_directory() {
    let directory = TempDir::new("default-output");

    let result = examark_in(directory.path(), &["build", &fixture("图文文档.examark")]);

    assert!(result.status.success(), "stderr：{}", stderr(&result));
    assert!(
        directory.path().join("dist/图文文档.html").is_file(),
        "默认输出目录应是 dist：{}",
        stdout(&result)
    );
    assert!(directory.path().join("dist/assets/图1.png").is_file());
}

#[test]
fn build_without_a_document_is_a_usage_error() {
    let result = examark(&["build"]);

    assert!(!result.status.success());
    assert!(stderr(&result).contains("用法"), "{}", stderr(&result));
}

#[test]
fn build_help_prints_usage() {
    let result = examark(&["build", "--help"]);

    assert!(result.status.success());
    assert!(stdout(&result).contains("-o"), "{}", stdout(&result));
}

/// 一份引用图片的最小题目文档；`images` 是题干里要写的图片引用。
fn document_with_image(images: &str) -> String {
    format!(
        "@module 数量关系\n\n  @question\n    @stem 见图 @image{{{images}}}。\n    @option A 甲\n    @option B 乙\n    @option C 丙\n    @option D 丁\n    @answer A\n"
    )
}

#[test]
fn build_accepts_a_document_without_images() {
    let directory = TempDir::new("no-images");
    let document = directory.path().join("纯文本.examark");
    fs::write(
        &document,
        "@module 资料分析\n\n  @question\n    @stem 与上年同期相比增长了多少？\n    @option A 4.1%\n    @option B 5.3%\n    @option C 6.2%\n    @option D 7.0%\n    @answer C\n",
    )
    .expect("临时文档应可写");

    let result = build_into(&document, directory.path());

    assert!(result.status.success(), "stderr：{}", stderr(&result));
    let html = fs::read_to_string(directory.path().join("纯文本.html")).expect("应产出 HTML 文件");
    assert!(html.contains("答案：C"), "{html}");
    assert!(
        !directory.path().join("assets").exists(),
        "没有图片引用就不该产出资源目录"
    );
}

#[test]
fn help_lists_every_command() {
    let text = stdout(&examark(&["--help"]));

    for command in ["build", "watch", "preview", "toolchain"] {
        assert!(text.contains(command), "帮助应列出 {command}：{text}");
    }
}

/// 等输出与等的空闲时间上界：监视的轮询间隔是 250ms，这里留足余量。
const WAIT: Duration = Duration::from_secs(10);

/// 长驻命令（watch/preview/toolchain）的一次会话：两条输出流都收进同一个通道，
/// 析构时结束进程。
struct Session {
    child: Child,
    records: Receiver<Record>,
}

/// 一条输出，连同它来自哪条流。
struct Record {
    stream: Stream,
    text: String,
}

/// 输出来自 stdout 还是 stderr。
#[derive(PartialEq, Eq, Clone, Copy)]
enum Stream {
    Out,
    Err,
}

impl Session {
    /// 启动一个长驻命令。
    fn start(args: &[&str]) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_examark"))
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("examark 二进制应可运行");

        let (sender, records) = mpsc::channel();
        let streams: [(Stream, Box<dyn Read + Send>); 2] = [
            (
                Stream::Out,
                Box::new(child.stdout.take().expect("stdout 应被捕获")),
            ),
            (
                Stream::Err,
                Box::new(child.stderr.take().expect("stderr 应被捕获")),
            ),
        ];

        for (stream, reader) in streams {
            let sender = sender.clone();
            thread::spawn(move || {
                for line in BufReader::new(reader).lines().map_while(Result::ok) {
                    let _ = sender.send(Record { stream, text: line });
                }
            });
        }

        drop(sender);

        Self { child, records }
    }

    /// 等某条流上出现含 `needle` 的一行；等不到就带着已收到的输出报错。
    fn wait_for(&mut self, stream: Stream, needle: &str) -> String {
        let mut seen = Vec::new();

        loop {
            let record = self
                .records
                .recv_timeout(WAIT)
                .unwrap_or_else(|_| panic!("等不到含有 {needle:?} 的输出；已收到：{seen:#?}"));

            if record.stream == stream && record.text.contains(needle) {
                return record.text;
            }

            seen.push(record.text);
        }
    }

    /// 等进程自己结束，返回退出状态。
    fn wait_for_exit(&mut self) -> ExitStatus {
        let deadline = Instant::now() + WAIT;

        loop {
            if let Some(status) = self.child.try_wait().expect("应能查询子进程状态") {
                return status;
            }

            assert!(Instant::now() < deadline, "进程应在那之前自行结束");
            thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// 从「本地预览：http://127.0.0.1:<端口>/」里取出实际端口。
fn preview_port(session: &mut Session) -> u16 {
    let line = session.wait_for(Stream::Out, "本地预览：");
    let url = line
        .split("http://")
        .nth(1)
        .unwrap_or_else(|| panic!("应报出带协议的预览地址：{line}"));

    url.split('/')
        .next()
        .and_then(|authority| authority.rsplit(':').next())
        .and_then(|port| port.parse().ok())
        .unwrap_or_else(|| panic!("应从 {line} 里取到端口"))
}

/// 一条响应：状态码、响应头与正文。
struct Response {
    status: u16,
    head: String,
    body: Vec<u8>,
}

impl Response {
    /// 某个响应头的值；头名不分大小写。
    fn header(&self, name: &str) -> String {
        self.head
            .lines()
            .filter_map(|line| line.split_once(':'))
            .find(|(header, _)| header.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.trim().to_string())
            .unwrap_or_default()
    }

    fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

/// 向本地服务发一条 GET，读到连接关闭为止。
fn http_get(port: u16, target: &str) -> Response {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("本地服务应可连接");
    stream.set_read_timeout(Some(WAIT)).expect("应能设读超时");

    write!(
        stream,
        "GET {target} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    )
    .expect("请求应可发出");

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).expect("响应应可读完");

    let split = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("响应头应有一个结束空行");
    let head = String::from_utf8_lossy(&raw[..split]).into_owned();
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .unwrap_or_else(|| panic!("状态行应含状态码：{head}"));

    Response {
        status,
        head,
        body: raw[split + 4..].to_vec(),
    }
}

/// 把 URL 路径里的其他字节按浏览器的方式写成 `%XX`。
fn percent_encode(path: &str) -> String {
    path.bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            byte => format!("%{byte:02X}"),
        })
        .collect()
}

/// 把 fixture 目录整体拷进一个临时目录，得到可随意改写的源目录。
fn copied_fixtures(name: &str) -> TempDir {
    let directory = TempDir::new(name);
    copy_directory(&fixtures(), directory.path());
    directory
}

fn copy_directory(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("目录应可创建");

    for entry in fs::read_dir(from).expect("fixture 目录应可读") {
        let entry = entry.expect("目录项应可读");
        let target = to.join(entry.file_name());

        if entry.file_type().expect("应能取到文件类型").is_dir() {
            copy_directory(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).expect("fixture 文件应可拷贝");
        }
    }
}

/// 源目录里的题目文档。
fn document_in(directory: &TempDir) -> PathBuf {
    directory.path().join("图文文档.examark")
}

/// 路径的 `&str` 形式，用于拼命令参数。
fn as_str(path: &Path) -> &str {
    path.to_str().expect("路径应是 UTF-8")
}

/// 在文档里把 `from` 换成 `to` 并写回磁盘——模拟作者保存。
fn save(document: &Path, from: &str, to: &str) {
    let source = fs::read_to_string(document).expect("文档应可读");
    let edited = source.replace(from, to);

    assert_ne!(edited, source, "要替换的内容应在文档里：{from}");
    fs::write(document, edited).expect("文档应可写");
}

#[test]
fn watch_without_a_document_is_a_usage_error() {
    let result = examark(&["watch"]);

    assert!(!result.status.success());
    assert!(stderr(&result).contains("用法"), "{}", stderr(&result));
}

#[test]
fn watch_builds_the_document_before_watching() {
    let source = copied_fixtures("watch-initial");
    let output = TempDir::new("watch-initial-output");

    let mut session = Session::start(&[
        "watch",
        as_str(&document_in(&source)),
        "-o",
        as_str(output.path()),
    ]);

    session.wait_for(Stream::Out, "构建完成");
    session.wait_for(Stream::Out, "正在监视");

    let html = fs::read_to_string(output.path().join("图文文档.html")).expect("监视期间应有 HTML");
    assert!(html.contains("答案：A"), "{html}");
    assert!(
        output.path().join("assets/图1.png").is_file(),
        "资源也该跟着就位"
    );
}

#[test]
fn watch_rebuilds_when_the_document_is_saved() {
    let source = copied_fixtures("watch-rebuild");
    let document = document_in(&source);
    let output = TempDir::new("watch-rebuild-output");

    let mut session = Session::start(&["watch", as_str(&document), "-o", as_str(output.path())]);
    session.wait_for(Stream::Out, "正在监视");

    save(&document, "增长了多少？", "增长了多少（修订）？");
    session.wait_for(Stream::Out, "已重建");

    let html = fs::read_to_string(output.path().join("图文文档.html")).expect("重建后应仍有 HTML");
    assert!(
        html.contains("增长了多少（修订）？"),
        "重建应反映保存后的内容：{html}"
    );
}

#[test]
fn watch_keeps_watching_after_a_save_that_does_not_build() {
    let source = copied_fixtures("watch-error");
    let document = document_in(&source);
    let output = TempDir::new("watch-error-output");

    let mut session = Session::start(&["watch", as_str(&document), "-o", as_str(output.path())]);
    session.wait_for(Stream::Out, "正在监视");

    let good = fs::read_to_string(&document).expect("文档应可读");
    fs::write(&document, "@module 常识判断\n").expect("文档应可写");
    let message = session.wait_for(Stream::Err, "解析失败");
    assert!(message.contains("第 1 行"), "{message}");

    fs::write(&document, good.replace("@answer A", "@answer B")).expect("文档应可写");
    session.wait_for(Stream::Out, "已重建");

    let html =
        fs::read_to_string(output.path().join("图文文档.html")).expect("修好后应重建出 HTML");
    assert!(html.contains("答案：B"), "{html}");
}

#[test]
fn watch_reports_a_document_that_cannot_be_read() {
    let directory = TempDir::new("watch-missing");
    let missing = directory.path().join("不存在的文档.examark");

    let mut session = Session::start(&["watch", as_str(&missing), "-o", as_str(directory.path())]);

    let message = session.wait_for(Stream::Err, "不存在的文档.examark");
    assert!(message.contains("无法读取"), "{message}");
    assert_eq!(session.wait_for_exit().code(), Some(1));
}

#[test]
fn preview_help_mentions_the_port_option() {
    let text = stdout(&examark(&["preview", "--help"]));

    assert!(text.contains("--port"), "{text}");
}

#[test]
fn preview_builds_the_document_before_serving_it() {
    let source = copied_fixtures("preview-build");
    let output = TempDir::new("preview-build-output");

    let mut session = Session::start(&[
        "preview",
        as_str(&document_in(&source)),
        "-o",
        as_str(output.path()),
        "--port",
        "0",
    ]);

    session.wait_for(Stream::Out, "构建完成");
    let port = preview_port(&mut session);

    let response = http_get(port, "/");

    assert_eq!(response.status, 200);
    assert_eq!(response.header("content-type"), "text/html; charset=utf-8");
    assert!(response.text().contains("答案：A"), "{}", response.text());
}

#[test]
fn preview_serves_the_built_document_at_the_root() {
    let output = TempDir::new("preview-root");

    let mut session = Session::start(&[
        "preview",
        &fixture("图文文档.examark"),
        "-o",
        as_str(output.path()),
        "--port",
        "0",
    ]);
    let port = preview_port(&mut session);

    let response = http_get(port, "/");

    assert_eq!(response.status, 200);
    assert_eq!(
        response.header("cache-control"),
        "no-store",
        "刷新时应看到新构建，而不是浏览器缓存"
    );

    let html = response.text();
    assert!(html.starts_with("<!DOCTYPE html>"), "{html}");
    assert_eq!(
        image_references(&html).len(),
        3,
        "三处图片引用都该在：{html}"
    );
}

#[test]
fn preview_serves_the_copied_assets() {
    let output = TempDir::new("preview-assets");

    let mut session = Session::start(&[
        "preview",
        &fixture("图文文档.examark"),
        "-o",
        as_str(output.path()),
        "--port",
        "0",
    ]);
    let port = preview_port(&mut session);

    let response = http_get(port, &format!("/assets/{}", percent_encode("图1.png")));

    assert_eq!(response.status, 200);
    assert_eq!(response.header("content-type"), "image/png");
    assert_eq!(
        response.body,
        fs::read(fixtures().join("assets/图1.png")).expect("fixture 资源应存在"),
        "服务出的资源应与 fixture 逐字节相同"
    );
}

#[test]
fn preview_serves_chinese_file_names_that_browsers_percent_encode() {
    let output = TempDir::new("preview-chinese-name");

    let mut session = Session::start(&[
        "preview",
        &fixture("图文文档.examark"),
        "-o",
        as_str(output.path()),
        "--port",
        "0",
    ]);
    let port = preview_port(&mut session);

    let response = http_get(port, &format!("/{}", percent_encode("图文文档.html")));

    assert_eq!(response.status, 200, "浏览器访问中文文件名要能命中");
    assert!(
        response.text().contains("图文题目文档"),
        "{}",
        response.text()
    );
}

#[test]
fn preview_refuses_to_serve_files_outside_the_output_directory() {
    let directory = TempDir::new("preview-traversal");
    let output = directory.path().join("输出");
    fs::write(directory.path().join("secret.txt"), "不该被看到").expect("文件应可写");

    let mut session = Session::start(&[
        "preview",
        &fixture("图文文档.examark"),
        "-o",
        as_str(&output),
        "--port",
        "0",
    ]);
    let port = preview_port(&mut session);

    for target in [
        "/../secret.txt",
        "/%2e%2e/secret.txt",
        "/assets/../../secret.txt",
    ] {
        let response = http_get(port, target);

        assert_eq!(response.status, 404, "{target} 应被拒掉");
        assert!(
            !response.text().contains("不该被看到"),
            "{target} 漏出了输出目录外的文件"
        );
    }
}

#[test]
fn preview_reports_a_port_that_is_already_taken() {
    let occupied = TcpListener::bind(("127.0.0.1", 0)).expect("应能占住一个端口");
    let port = occupied.local_addr().expect("应能取到地址").port();
    let port_argument = port.to_string();
    let output = TempDir::new("preview-port-taken");

    let mut session = Session::start(&[
        "preview",
        &fixture("图文文档.examark"),
        "-o",
        as_str(output.path()),
        "--port",
        &port_argument,
    ]);

    let message = session.wait_for(Stream::Err, "已被占用");
    assert!(message.contains(&port_argument), "{message}");
    assert_eq!(session.wait_for_exit().code(), Some(1));
}

#[test]
fn preview_exits_when_the_document_cannot_be_built() {
    let directory = TempDir::new("preview-parse-error");
    let document = directory.path().join("常识.examark");
    fs::write(&document, "@module 常识判断\n").expect("文档应可写");

    let mut session = Session::start(&[
        "preview",
        as_str(&document),
        "-o",
        as_str(directory.path()),
        "--port",
        "0",
    ]);

    let message = session.wait_for(Stream::Err, "解析失败");
    assert!(message.contains("第 1 行"), "{message}");
    assert_eq!(session.wait_for_exit().code(), Some(1));
}

#[test]
fn toolchain_builds_serves_and_rebuilds_in_one_process() {
    let source = copied_fixtures("toolchain-loop");
    let document = document_in(&source);
    let output = TempDir::new("toolchain-loop-output");

    let mut session = Session::start(&[
        "toolchain",
        as_str(&document),
        "-o",
        as_str(output.path()),
        "--port",
        "0",
    ]);

    session.wait_for(Stream::Out, "构建完成");
    let port = preview_port(&mut session);
    session.wait_for(Stream::Out, "正在监视");

    let first = http_get(port, "/");
    assert_eq!(first.status, 200);
    assert!(first.text().contains("答案：A"), "{}", first.text());

    save(&document, "@answer A", "@answer B");
    session.wait_for(Stream::Out, "已重建");

    let second = http_get(port, "/");
    assert!(
        second.text().contains("答案：B"),
        "刷新应看到刚保存的内容：{}",
        second.text()
    );
}

#[test]
fn toolchain_reports_a_document_that_cannot_be_read() {
    let directory = TempDir::new("toolchain-missing");
    let missing = directory.path().join("不存在的文档.examark");

    let mut session = Session::start(&[
        "toolchain",
        as_str(&missing),
        "-o",
        as_str(directory.path()),
        "--port",
        "0",
    ]);

    let message = session.wait_for(Stream::Err, "不存在的文档.examark");
    assert!(message.contains("无法读取"), "{message}");
    assert_eq!(session.wait_for_exit().code(), Some(1));
}

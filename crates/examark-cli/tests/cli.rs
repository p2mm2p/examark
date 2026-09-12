use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

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

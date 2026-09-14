//! `examark-syntax` 渲染接缝的测试：字符串进，HTML 出。
//!
//! golden 文件放在 `tests/golden/`：`<名字>.examark` 是源文档，`<名字>.html` 是期望输出。
//! 真题语料放在 `tests/papers/`，同样是 `<名字>.examark` + `<名字>.html` 一对。
//!
//! 设了 `EXAMARK_UPDATE_GOLDEN` 时只重写 `.html` 而不比对——用来产出初版 golden，
//! 平时不设，任何不一致都会硬失败。

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;

use examark_syntax::{parse, render, render_with_assets};

fn assert_golden(name: &str) {
    assert_golden_in("golden", name);
}

/// 真题语料只覆盖渲染接缝：文档里的 `@image{…}` 指向不存在的文件是故意的，
/// 资源拷贝与路径重写由 `examark-cli` 自己的测试负责。
fn assert_paper(name: &str) {
    assert_golden_in("papers", name);
}

fn assert_golden_in(directory: &str, name: &str) {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(directory);
    let source =
        fs::read_to_string(directory.join(format!("{name}.examark"))).expect("golden 源文档应存在");

    let document = parse(&source).expect("golden 源文档应解析成功");
    let html = render(&document);

    let target = directory.join(format!("{name}.html"));
    if std::env::var_os("EXAMARK_UPDATE_GOLDEN").is_some() {
        fs::write(&target, &html).expect("重写 golden 期望 HTML 应成功");
        return;
    }

    let expected = fs::read_to_string(&target).expect("golden 期望 HTML 应存在");
    assert_rendered(&html, &expected, name);
}

/// 逐行比对，报出第一处不同；整串 `assert_eq!` 的差异读起来太费劲。
fn assert_rendered(html: &str, expected: &str, name: &str) {
    if html == expected {
        return;
    }

    let mut report = format!("渲染结果与 golden 文件 {name}.html 不一致\n");
    for (index, (line, expected_line)) in html.lines().zip(expected.lines()).enumerate() {
        if line != expected_line {
            report.push_str(&format!(
                "第 {} 行：\n  实际：{line}\n  期望：{expected_line}\n",
                index + 1
            ));
            break;
        }
    }
    if html.lines().count() != expected.lines().count() {
        report.push_str(&format!(
            "行数：实际 {}，期望 {}\n",
            html.lines().count(),
            expected.lines().count()
        ));
    }

    panic!("{report}");
}

#[test]
fn renders_a_minimal_document() {
    assert_golden("minimal");
}

#[test]
fn renders_a_whole_paper() {
    assert_golden("paper");
}

#[test]
fn renders_inline_math_images_and_blanks() {
    assert_golden("inline");
}

#[test]
fn renders_a_material_question() {
    assert_golden("material");
}

/// 真题语料：只保留 v1 能表达的模块，题面公式已转写为 `@math`，
/// 强调型下划线（v1 无语法）所在的题目整题排除。
#[test]
fn renders_the_provincial_paper() {
    assert_paper("2026年国家公务员录用考试《行测》题（副省级网友回忆版）");
}

#[test]
fn renders_the_municipal_paper() {
    assert_paper("2026年国家公务员录用考试《行测》题（地市级网友回忆版）");
}

#[test]
fn renders_the_law_enforcement_paper() {
    assert_paper("2026年国家公务员录用考试《行测》题（行政执法卷网友回忆版）");
}

#[test]
fn renders_the_guangdong_paper() {
    assert_paper("2026年广东省公务员录用考试《行测》题（网友回忆版）");
}

/// 图片引用在 HTML 里写到哪里由调用方经映射决定：构建时用它把资源重写到输出目录。
#[test]
fn writes_image_references_where_the_resolver_puts_them() {
    let source = "\
@module 数量关系

  @question
    @stem 见图 @image{插图/散图.png}。
    @option A @image{插图/甲.png}
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A
";

    let document = parse(source).expect("文档应解析成功");

    let seen = RefCell::new(Vec::new());
    let html = render_with_assets(&document, &mut |path| {
        seen.borrow_mut().push(path.to_string());
        let name = path.rsplit('/').next().expect("图片路径应有文件名");
        format!("assets/{name}")
    });

    assert_eq!(seen.into_inner(), ["插图/散图.png", "插图/甲.png"]);
    assert!(
        html.contains(r#"<img src="assets/散图.png" alt="">"#),
        "{html}"
    );
    assert!(
        html.contains(r#"<img src="assets/甲.png" alt="">"#),
        "{html}"
    );
}

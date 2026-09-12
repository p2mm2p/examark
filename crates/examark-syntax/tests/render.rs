//! `examark-syntax` 渲染接缝的测试：字符串进，HTML 出。
//!
//! golden 文件放在 `tests/golden/`：`<名字>.examark` 是源文档，`<名字>.html` 是期望输出。

use std::fs;
use std::path::PathBuf;

use examark_syntax::{parse, render};

fn assert_golden(name: &str) {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let source =
        fs::read_to_string(directory.join(format!("{name}.examark"))).expect("golden 源文档应存在");
    let expected = fs::read_to_string(directory.join(format!("{name}.html")))
        .expect("golden 期望 HTML 应存在");

    let document = parse(&source).expect("golden 源文档应解析成功");
    let html = render(&document);

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

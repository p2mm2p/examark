//! `watch` 命令：构建一次，然后守着源文档，保存即重建。

use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::build::{self, BuildError};

/// 两次检查之间隔多久；它决定「保存」到「重建」的延迟上界。
const POLL_INTERVAL: Duration = Duration::from_millis(250);

/// 构建一次，然后一直监视 `document`；只在首次构建失败时返回。
pub fn watch(document: &Path, output: &Path) -> Result<(), BuildError> {
    build::build_and_report(document, output)?;

    rebuild_on_change(document, output)
}

/// 一直监视 `document`，内容一变就重建，直到进程被结束；首次构建由调用方负责。
pub fn rebuild_on_change(document: &Path, output: &Path) -> Result<(), BuildError> {
    // 快照要在报出「正在监视」之前取：作者看到那行后保存的内容，一定落在快照之后。
    let mut seen = read_or_empty(document);

    println!("正在监视 {}，保存即重建（Ctrl-C 退出）", document.display());

    loop {
        thread::sleep(POLL_INTERVAL);

        let current = match fs::read(document) {
            Ok(source) => source,
            // 保存中途可能短暂读不到（编辑器先删后写），下次再看。
            Err(_) => continue,
        };

        if current == seen {
            continue;
        }

        seen = current;

        // 作者正在改文档，写错是常态：报出来就好，监视继续。
        if let Err(error) = build::rebuild_and_report(document, output) {
            eprintln!("{error}");
        }
    }
}

/// 读文档内容；读不到时给空内容，好让下一次成功的读取必定算作一次改动。
fn read_or_empty(document: &Path) -> Vec<u8> {
    fs::read(document).unwrap_or_default()
}

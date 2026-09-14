//! `build` 命令：把题目文档构建为可移植的输出目录。

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use emark_syntax::{parse, render_with_assets};

/// 资源在输出目录里的子目录。
const ASSET_DIRECTORY: &str = "assets";

/// 构建失败；说明面向作者，直接印到 stderr。
#[derive(Debug)]
pub struct BuildError(String);

impl BuildError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for BuildError {}

/// 构建一次，把产出位置报到 stdout。
pub fn build_and_report(document: &Path, output: &Path) -> Result<PathBuf, BuildError> {
    build_announcing(document, output, "构建完成")
}

/// 重建一次，把产出位置报到 stdout。
pub fn rebuild_and_report(document: &Path, output: &Path) -> Result<PathBuf, BuildError> {
    build_announcing(document, output, "已重建")
}

/// 构建一次，把产出位置按 `done` 的措辞报到 stdout。
fn build_announcing(document: &Path, output: &Path, done: &str) -> Result<PathBuf, BuildError> {
    let target = build(document, output)?;

    println!("{done}：{}", target.display());

    Ok(target)
}

/// 把题目文档构建进输出目录，返回产出的 HTML 文件。
pub fn build(document: &Path, output: &Path) -> Result<PathBuf, BuildError> {
    let source = fs::read_to_string(document).map_err(|error| {
        BuildError::new(format!("无法读取题目文档 {}：{error}", document.display()))
    })?;

    let parsed = parse(&source).map_err(|error| {
        BuildError::new(format!("题目文档 {} 解析失败：{error}", document.display()))
    })?;

    let mut assets = Assets::new(document_directory(document));
    let html = render_with_assets(&parsed, &mut |path| {
        assets.place(path).unwrap_or_else(|| path.to_owned())
    });

    let target = output.join(html_name(document)?);
    fs::create_dir_all(output).map_err(|error| {
        BuildError::new(format!("无法创建输出目录 {}：{error}", output.display()))
    })?;
    assets.copy(output)?;
    write(&target, html.as_bytes())?;

    Ok(target)
}

/// 把文件写进输出目录：先写同目录的临时文件，再改名过去。
///
/// `preview` 与 `toolchain` 会在重建的同时提供这些文件；直接覆写会让正好赶上的请求
/// 读到半截内容，而同目录内的改名是原子的。
fn write(target: &Path, bytes: &[u8]) -> Result<(), BuildError> {
    let temporary = temporary_path(target);

    fs::write(&temporary, bytes)
        .map_err(|error| BuildError::new(format!("无法写入 {}：{error}", temporary.display())))?;

    fs::rename(&temporary, target).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        BuildError::new(format!("无法写入 {}：{error}", target.display()))
    })
}

/// 临时文件的位置：与目标同目录，加一个 `.tmp` 后缀。
fn temporary_path(target: &Path) -> PathBuf {
    let mut name = target.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");

    target.with_file_name(name)
}

/// 题目文档所在目录；作者书写的资源路径以它为基准。
fn document_directory(document: &Path) -> &Path {
    match document.parent() {
        Some(directory) if !directory.as_os_str().is_empty() => directory,
        _ => Path::new("."),
    }
}

/// 输出目录里的 HTML 文件名：与题目文档同名，后缀换成 `.html`。
///
/// `preview` 与 `toolchain` 靠它知道访问 `/` 时该提供哪个文件。
pub fn html_name(document: &Path) -> Result<String, BuildError> {
    let stem = document
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| BuildError::new(format!("题目文档路径无效：{}", document.display())))?;

    Ok(format!("{stem}.html"))
}

/// 构建时登记的资源：把作者书写的路径映射到输出目录里的位置，并攒下待拷贝的清单。
struct Assets {
    /// 文档所在目录，作者书写的相对路径以它为基准。
    directory: PathBuf,
    /// 待拷贝的资源，按首次引用的顺序。
    copies: Vec<Asset>,
    /// 登记时发现的第一处问题。
    failure: Option<BuildError>,
}

/// 一处待拷贝的资源。
struct Asset {
    /// 作者书写的路径，用于报错。
    authored: String,
    /// 按文档所在目录解析出的实际位置。
    source: PathBuf,
    /// 输出目录下的位置，如 `assets/图1.png`。
    target: String,
}

impl Asset {
    /// 读出这份资源的字节；作者写下的图片不存在时报错。
    fn read(&self) -> Result<Vec<u8>, BuildError> {
        if !self.source.is_file() {
            return Err(BuildError::new(format!(
                "图片不存在：{}（按文档所在目录解析为 {}）",
                self.authored,
                self.source.display()
            )));
        }

        fs::read(&self.source).map_err(|error| {
            BuildError::new(format!("无法读取图片 {}：{error}", self.source.display()))
        })
    }
}

impl Assets {
    fn new(directory: &Path) -> Self {
        Self {
            directory: directory.to_path_buf(),
            copies: Vec::new(),
            failure: None,
        }
    }

    /// 登记一处图片引用，返回它在 HTML 里该写的位置；登记不下来时返回 `None`。
    fn place(&mut self, authored: &str) -> Option<String> {
        let name = match Path::new(authored)
            .file_name()
            .and_then(|name| name.to_str())
        {
            Some(name) => name.to_string(),
            None => {
                self.record(format!("图片路径无效：{authored}"));
                return None;
            }
        };

        let target = format!("{ASSET_DIRECTORY}/{name}");
        let source = self.directory.join(authored);

        if let Some(existing) = self.copies.iter().find(|asset| asset.target == target) {
            if existing.source == source {
                return Some(target);
            }

            let existing = existing.authored.clone();
            self.record(format!("两张图片都叫 {name}：{existing} 与 {authored}"));
            return None;
        }

        self.copies.push(Asset {
            authored: authored.to_string(),
            source,
            target: target.clone(),
        });

        Some(target)
    }

    /// 把登记的资源拷进输出目录；登记时发现过问题则先报出来。
    fn copy(&mut self, output: &Path) -> Result<(), BuildError> {
        if let Some(failure) = self.failure.take() {
            return Err(failure);
        }

        if self.copies.is_empty() {
            return Ok(());
        }

        let directory = output.join(ASSET_DIRECTORY);
        fs::create_dir_all(&directory).map_err(|error| {
            BuildError::new(format!("无法创建资源目录 {}：{error}", directory.display()))
        })?;

        for asset in &self.copies {
            let bytes = asset.read()?;

            write(&output.join(&asset.target), &bytes)?;
        }

        Ok(())
    }

    /// 记下第一处问题；后续问题不再覆盖它。
    fn record(&mut self, message: String) {
        if self.failure.is_none() {
            self.failure = Some(BuildError::new(message));
        }
    }
}

//! `toolchain` 命令：把 build、watch 与 preview 串成一条本地开发流。

use std::path::Path;

use crate::preview::{self, PreviewError};
use crate::watch;

/// 构建一次，然后一边在本地提供输出、一边守着源文档；只在起步失败时返回。
///
/// 构建是先行的一步；服务与监视则必须同时在跑——服务要一直开着，才谈得上「浏览器刷新即见」。
/// 于是服务交给后台线程，监视留在当前线程，两条线同时活着，直到进程被结束。
pub fn toolchain(document: &Path, output: &Path, port: u16) -> Result<(), PreviewError> {
    preview::serve_in_background(document, output, port)?;

    Ok(watch::rebuild_on_change(document, output)?)
}

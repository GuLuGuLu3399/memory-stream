//! # temple_core — Memory Stream Rust 引擎核心
//!
//! 提供领域错误契约 (`TempleError`) 与 IPC 类型系统，
//! 是所有 `temple_*` crate 的共享基础层。

pub mod error;
pub mod prelude;
pub mod types;

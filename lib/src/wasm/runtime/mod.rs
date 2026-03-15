//! WebAssembly execution runtime abstraction for YARA-X.
//!
//! YARA-X compiles rules into WebAssembly bytecode for high-performance
//! execution. This module provides a backend-neutral abstraction layer (shim)
//! that allows the compiler and scanner to run seamlessly on both native
//! platforms and in browser-based environments.
//!
//! ## Dual-Backend Architecture
//!
//! To support both native execution and browser sandbox environments, the
//! runtime uses a dual-backend architecture selected at compile-time:
//!
//! 1. **Native Backend (`native.rs`)**: Used for standard native targets
//!    (e.g., Linux, macOS, Windows, CLI, server applications). It leverages
//!    the robust, optimizing **Wasmtime** compiler and JIT engine directly.
//! 2. **Browser Backend (`browser.rs`)**: Used when compiling to WebAssembly
//!    targets (e.g., `wasm32-unknown-unknown`). Since Wasmtime cannot run
//!    inside the browser's Wasm sandbox, this backend provides a custom
//!    implementation wrapping the host environment's built-in JavaScript
//!    `WebAssembly` APIs via `wasm-bindgen`.
//! 3. **WASI Backend (`wasip1.rs`)**: Used by `wasm32-wasip1` builds with the
//!    `wasip1-runtime` feature. It delegates execution to a host bridge defined
//!    with WIT, which is implemented by the Go/wazero bindings.
//!
//! ## Unified Wasmtime-like Interface
//!
//! To avoid scattering conditional compilation directives
//! (`#[cfg(target_family = "wasm")]`) throughout the compiler and scanner
//! code, this module exposes a single, unified API that matches Wasmtime's
//! standard interface. On Wasm targets, the custom shim (implemented in
//! `common.rs` and `browser.rs`) serves as a drop-in replacement for
//! Wasmtime.
//!
//! The abstraction exposes common Wasm execution types, including:
//! - [`Store`]: Manages execution state and holds user-defined data (e.g.,
//!   `ScanContext`).
//! - [`Linker`]: Registers and links host-defined functions and globals into
//!   Wasm modules.
//! - [`Caller`]: Exposes the execution context and mutable access to the store
//!   within host callbacks.
//! - [`Memory`]: Provides structured access to Wasm linear memory.
//! - [`TypedFunc`]: Represents type-safe, callable handles to exported Wasm
//!   functions.

#[cfg(target_family = "wasm")]
mod common;

#[cfg(all(
    target_family = "wasm",
    feature = "wasip1-runtime",
    not(all(target_arch = "wasm32", target_os = "wasi", target_env = "p1"))
))]
compile_error!("`wasip1-runtime` is only supported for wasm32-wasip1 targets");

// Native builds execute generated WASM through Wasmtime.
#[cfg(not(target_family = "wasm"))]
mod native;

// Browser builds execute generated WASM through the host WebAssembly runtime.
#[cfg(all(target_family = "wasm", not(feature = "wasip1-runtime")))]
mod browser;

// wasm32-wasip1 builds delegate execution to the host bridge defined in WIT.
#[cfg(all(
    target_arch = "wasm32",
    target_os = "wasi",
    target_env = "p1",
    feature = "wasip1-runtime"
))]
mod wasip1;

#[cfg(not(target_family = "wasm"))]
pub use native::*;

#[cfg(all(target_family = "wasm", not(feature = "wasip1-runtime")))]
pub use browser::*;

#[cfg(all(
    target_arch = "wasm32",
    target_os = "wasi",
    target_env = "p1",
    feature = "wasip1-runtime"
))]
pub use wasip1::*;

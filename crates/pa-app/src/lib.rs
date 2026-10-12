//! Application layer: the use cases.
//!
//! Without this layer the use cases would land either in `pa-core`, which is
//! the domain and holds no I/O, or in `pa-wasm`, which would make them
//! unreachable from `pa-cli` and therefore untestable natively.
//!
//! One use case lives here: [`key_bench`], the single-key weighing bench. The
//! regulation solve it calls, which turns trade targets into punching stacks,
//! is domain code: [`pa_core::parts::Regulation::solve`].
//!
//! `pa-wasm` and `pa-cli` are adapters of this crate and do not depend on
//! `pa-core` directly. Their manifests hold that direction; no check in the
//! build enforces it.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod key_bench;

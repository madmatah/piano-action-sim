//! Application layer: the use cases.
//!
//! Without this layer the use cases would land either in `pa-core`, which is
//! the domain and holds no I/O, or in `pa-wasm`, which would make them
//! unreachable from `pa-cli` and therefore untestable natively.
//!
//! One use case lives here: [`key_bench`], the single-key weighing bench. The
//! regulation solve it calls, which turns trade targets into punching stacks,
//! is domain code: [`pa_core::parts::Regulation::solve`]. The bench's
//! composite key has no measured mass; [`composite_key`] derives one from its
//! volume for the mechanism, which takes a mass as data and never computes
//! one. The closed forms on the frame still read the volume model directly.
//! Beside them, [`golden`] records every reading of the bench and of a set of
//! fixture benches in full bits, so that a change to the engine shows every
//! number it moves.
//!
//! `pa-wasm` and `pa-cli` are adapters of this crate and do not depend on
//! `pa-core` directly. Their manifests hold that direction; no check in the
//! build enforces it.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod composite_key;
pub mod golden;
pub mod key_bench;

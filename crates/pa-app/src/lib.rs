//! Application layer: the use cases.
//!
//! Without this layer the use cases would land either in `pa-core`, which is
//! the domain and holds no I/O, or in `pa-wasm`, which would make them
//! unreachable from `pa-cli` and therefore untestable natively.
//!
//! One use case lives here: [`instrument`], the single-key weighing bench as
//! an application composes it, every setting set by its path and every
//! operation run as an action, each edit validated before any solve, logged
//! and replayable, its state hashed. An instrument is built from a
//! [`profile`], which binds every number it starts from to a record in
//! `data/params/` or to an assumption stated with its reason, both compiled
//! in; the domain starts from no number of its own. [`registry`] composes
//! the domain's declarations of what each setting admits into the
//! instrument's paths, with the trade unit in each path's name, and the page
//! that presents them. The
//! regulation solve the instrument calls, which turns trade targets into
//! punching stacks, is domain code: [`pa_core::regulation`]. The bench's
//! composite key has no measured mass until it is weighed whole;
//! [`composite_key`] derives one from its volume for the mechanism, which
//! takes a mass as data and never computes one, and the height of a weighed
//! key's centre of mass, which no scale gives.
//! Beside them, [`golden`] records every reading of the bench and of a set of
//! fixture benches in full bits, each built through the instrument's own
//! calls, so that a change to the engine shows every number it moves; and
//! [`boundary`] turns the instrument into the documents an interface reads,
//! registry, settings, sheet, diagnostics, state and scene apart, in codes
//! and numbers, and the answers to its two calls, whose TypeScript types it
//! generates.
//!
//! `pa-wasm` and `pa-cli` are adapters of this crate and do not depend on
//! `pa-core` directly. Their manifests hold that direction; no check in the
//! build enforces it.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod boundary;
pub mod composite_key;
pub mod golden;
#[cfg(test)]
mod hirschkorn;
pub mod instrument;
pub mod profile;
pub mod registry;

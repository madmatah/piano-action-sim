//! Domain core of the grand-piano action simulator.
//!
//! No I/O, and no knowledge of JavaScript, the browser or the file system. It
//! compiles and tests natively, and reaches the browser through `pa-app` and
//! `pa-wasm`.
//!
//! # Layout
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`geometry`] | planar primitives and exact contact kinematics |
//! | [`mechanism`] | bodies in a tree: poses, Jacobian columns, loads, prescribed coordinates and their reactions |
//! | [`parts`] | the key and its frame as parametric parts, and their mass properties |
//! | [`parts::assembly`] | the key on its frame: stops, rest, travel, regulation, settle |
//! | [`parts::mounts`] | the mechanisms a key is put in to be read: the jig |
//! | [`measure`] | the trade's weighing quantities, as the technician's protocols define them |
//! | [`measure::jig`] | Stanwood's front weight jig, run on the mechanism |
//!
//! # What holds today, and what holds it
//!
//! 1. **One definition per part.** The key is drawn from, and its mass
//!    integrated over, [`parts::KeyDesign::outline_vertices`]. Its stops read
//!    [`parts::KeyDesign::underside_corners`] and
//!    [`parts::KeyDesign::underside_segments`], which are built from the same
//!    fields. The two agree because they share those fields, and
//!    `the_declared_bearing_surface_is_the_whole_underside` checks it on the
//!    default key; nothing in the types enforces it. The frame is the same:
//!    its stops read the bearing heights of [`parts::FrameDesign`], its drawn
//!    members are built from those heights, and
//!    `what_is_drawn_is_what_is_touched` checks that they meet.
//! 2. **Regulation is expressed as trade targets**, and the geometry is solved
//!    to meet them ([`parts::Regulation::solve`]). The target is kept beside
//!    what the geometry achieves.
//! 3. **Dry friction at the key's centres is set-valued**: an interval of
//!    admissible torque, never a regularized law
//!    ([`parts::KeyAssembly::settle`]).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod geometry;
pub mod measure;
pub mod mechanism;
pub mod parts;

/// Standard gravity, m/s². Fixed here so that no caller can vary it silently.
pub const G: f64 = 9.806_65;

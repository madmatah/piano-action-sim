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
//! | [`geometry`] | planar primitives, lines and arcs, the signed distances contacts are measured by, and the one-coordinate kinematics |
//! | [`mechanism`] | bodies in a tree: poses, Jacobian columns, loads, prescribed coordinates and their reactions, the parts the bodies carry and the contacts declared between them |
//! | [`contact`] | parts on bodies, the ranges of their outlines a contact may read, contact pairs and their rows over every coordinate, and how a key bears on a stop |
//! | [`parts`] | the key and its frame as parametric designs, the parts they build, and the composite key's mass |
//! | [`parts::assembly`] | the key on its frame at one coordinate: stops, rest, travel, regulation, settle |
//! | [`parts::faults`] | what about a key and its frame no keyboard could be built from, read off the drawn parts |
//! | [`parts::mounts`] | the mechanisms a key is put in to be read, the jig and the key on its frame, and the key's contacts with its frame |
//! | [`measure`] | the trade's weighing quantities, as the technician's protocols define them |
//! | [`measure::jig`] | Stanwood's front weight jig, run on the mechanism |
//!
//! # What holds today, and what holds it
//!
//! 1. **One definition per part.** The key and the frame are built as parts
//!    ([`contact::Part`]): one closed outline per piece of material, on the
//!    body that carries it, drawn by the bench and read by the faults, each
//!    declaring the ranges of it a contact may read. The contacts read those
//!    ranges and nothing else: a [`contact::ContactPair`] holds nothing but
//!    them, and its rows are read off the parts a mechanism carries. The
//!    one-coordinate statics that place the key still read stops of their
//!    own, built from the same fields: [`parts::KeyDesign::underside_corners`]
//!    for the key, the bearing heights of [`parts::FrameDesign`] for the
//!    frame. `every_stop_face_is_a_declared_range_of_a_drawn_part` holds
//!    every face they read to a declared range.
//! 2. **Regulation is expressed as trade targets**, and the geometry is solved
//!    to meet them ([`parts::Regulation::solve`]). The target is kept beside
//!    what the geometry achieves.
//! 3. **Dry friction at the key's centres is set-valued**: an interval of
//!    admissible torque, never a regularized law
//!    ([`parts::KeyAssembly::settle`]).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod contact;
pub mod geometry;
pub mod measure;
pub mod mechanism;
pub mod parts;

/// Standard gravity, m/s². Fixed here so that no caller can vary it silently.
pub const G: f64 = 9.806_65;

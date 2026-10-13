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
//! | [`mechanism`] | bodies in a tree: poses, Jacobian columns, loads, coordinate limits and friction, prescribed coordinates and their reactions, the parts the bodies carry and the contacts declared between them |
//! | [`contact`] | parts on bodies, the ranges of their outlines a contact may read, contact pairs and their rows over every coordinate, how deep drawn parts stand in one another, and how a key bears on a stop |
//! | [`solve`] | the static solve: one load increment, its box problem and the dense algebra under it |
//! | [`parts`] | the key and its frame as parametric designs, the parts they build, and the composite key's mass |
//! | [`parts::assembly`] | the key on its frame at one coordinate: stops, rest attitudes, settle |
//! | [`parts::faults`] | what about a key and its frame no keyboard could be built from, read off the drawn parts |
//! | [`parts::mounts`] | the mechanisms a key is put in to be read, the jig and the key on its frame, the key's contacts with its frame, and its nominal pose there |
//! | [`parts::verdicts`] | what holds a key where it stands, from determinate quantities |
//! | [`measure`] | the trade's weighing quantities, as the technician's protocols define them |
//! | [`measure::jig`] | Stanwood's front weight jig, run on the mechanism |
//! | [`measure::seat`] | the seated pose, from which key height and dip are read |
//! | [`regulation`] | the regulation solve: key height and dip targets to the papers laid on the two punchings, by the seated pose itself |
//!
//! # What holds today, and what holds it
//!
//! 1. **One definition per part.** The key and the frame are built as parts
//!    ([`contact::Part`]): one closed outline per piece of material, on the
//!    body that carries it, drawn by the bench and read by the faults, each
//!    declaring the ranges of it a contact may read. The contacts read those
//!    ranges and nothing else: a [`contact::ContactPair`] holds nothing but
//!    them, and its rows are read off the parts a mechanism carries; the
//!    static solve places the key on them ([`measure::seat`]), and the
//!    regulation solve reads the key placed there ([`regulation`]). The
//!    one-coordinate statics left under the key friction weight protocol
//!    still read stops of their own, built from the same fields:
//!    [`parts::KeyDesign::underside_corners`] for the key, the bearing heights
//!    of [`parts::FrameDesign`] for the frame.
//!    `every_stop_face_is_a_declared_range_of_a_drawn_part` holds every face
//!    they read to a declared range.
//! 2. **Regulation is expressed as trade targets**, and the geometry is solved
//!    to meet them ([`regulation::Regulating::solve`]): the papers laid, what
//!    they give beside the targets, or the target refused with what keeps it
//!    out of reach.
//! 3. **Dry friction at the key's centres is set-valued**: an interval of
//!    admissible torque, never a regularized law, a box row of the static
//!    solve ([`solve::statics`]) and of [`parts::KeyAssembly::settle`].

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod contact;
pub mod geometry;
pub mod measure;
pub mod mechanism;
pub mod parts;
pub mod regulation;
pub mod solve;

/// Standard gravity, m/s². Fixed here so that no caller can vary it silently.
pub const G: f64 = 9.806_65;

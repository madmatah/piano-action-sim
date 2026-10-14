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
//! | [`geometry`] | planar primitives, lines and arcs, and the signed distances contacts are measured by |
//! | [`mechanism`] | bodies in a tree: poses, Jacobian columns, loads, coordinate limits and friction, prescribed coordinates and their reactions, the parts the bodies carry and the contacts declared between them |
//! | [`contact`] | parts on bodies, the ranges of their outlines a contact may read, contact pairs and their rows over every coordinate, how deep drawn parts stand in one another, and how a key bears on a stop |
//! | [`solve`] | the static solve: one load increment, its box problem and the dense algebra under it |
//! | [`parts`] | the key and its frame as parametric designs, the parts they build, and what is known of a key's mass, weighed whole, published or not at all |
//! | [`parts::faults`] | what about a key and its frame no keyboard could be built from, read off the drawn parts |
//! | [`parts::mounts`] | the mechanisms a key is put in to be read, the jig and the key on its frame, free to lift off its balance punching or turning on a pivot, the key's contacts with its frame, the solve that lets it lift, and its nominal pose there |
//! | [`parts::verdicts`] | what holds a key where it stands, from determinate quantities |
//! | [`measure`] | the trade's weighing quantities, as the technician's protocols define them |
//! | [`measure::jig`] | Stanwood's front weight jig, run on the mechanism |
//! | [`measure::seat`] | the seated pose, from which key height and dip are read |
//! | [`measure::walk`] | the breakaway walk, from which down and up weight are read |
//! | [`measure::sweep`] | the driven sweep, the key's angle held through its stroke down and back: the band, and down and up weight read on it a second way |
//! | [`measure::weighing`] | every reading of the weighing sheet, by its procedure, and the bushings a typed key friction weight reads on, by Stanwood's protocol inverted |
//! | [`measure::measurement`] | a reading with its value, what the trade reads it as, its conditions and its trust |
//! | [`regulation`] | the regulation solve: key height and dip targets to the papers laid on the two punchings, by the seated pose itself |
//! | [`registry`] | what each setting of a part admits, declared beside the part's fields: its quantity, hard range, step, layer and kind |
//! | [`validate`] | the validation layer: what no part could be built as, refused with the limiting dimension and the shortfall before any solve |
//! | `test_fixture` | the owner's measured key section, its frame, the drawer, Stanwood's protocol and a test weighing as Rust literals, for tests only: the engine starts from no number of its own |
//!
//! # What holds today, and what holds it
//!
//! 1. **One definition per part.** The key and the frame are built as parts
//!    ([`contact::Part`]): one closed outline per piece of material, on the
//!    body that carries it, drawn by the bench and read by the faults, each
//!    declaring the ranges of it a contact may read. The contacts read those
//!    ranges and nothing else: a [`contact::ContactPair`] holds nothing but
//!    them, and its rows are read off the parts a mechanism carries; the
//!    static solve places the key on them ([`measure::seat`]), and every
//!    procedure of the sheet reads the key placed there ([`measure::weighing`],
//!    [`regulation`]), as does the drawing, a key riding on a frame whose
//!    stops fight included ([`parts::mounts::Mounted::solve`]). Nothing
//!    reads anything but a drawn range, so the invariant holds by structure;
//!    `every_contact_lies_on_its_drawn_outline` holds every row's points on
//!    the drawn outlines.
//! 2. **Regulation is expressed as trade targets**, and the geometry is solved
//!    to meet them ([`regulation::Regulating::solve`]): the papers laid, what
//!    they give beside the targets, or the target refused with what keeps it
//!    out of reach.
//! 3. **Every reading is a protocol run on the model**: the jig, the seated
//!    pose, the breakaway walk and the driven sweep ([`measure::weighing`]),
//!    each coming back as a [`measure::measurement::Measurement`] or refused
//!    with its reason. Their closed forms are test oracles only.
//! 4. **Dry friction at the key's centres is set-valued**: an interval of
//!    admissible torque, never a regularized law, a box row of the static
//!    solve ([`solve::statics`]), stored as a torque with the radius it was
//!    read at ([`mechanism::AxisFriction`]).
//! 5. **What no part could be built as is refused before any solve**, never
//!    clamped: each part declares what its settings admit beside its fields
//!    ([`registry::ParamSpec`]), and [`validate::validate`] refuses a design
//!    that breaks them or whose drawn parts could not be cut, naming the
//!    limiting dimension and the shortfall. Everything an assembled keyboard
//!    can be is computed and reported instead.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod contact;
pub mod geometry;
pub mod measure;
pub mod mechanism;
pub mod parts;
pub mod registry;
pub mod regulation;
pub mod solve;
#[cfg(any(test, feature = "fixtures"))]
pub mod test_fixture;
pub mod validate;

/// Standard gravity, m/s². Fixed here so that no caller can vary it silently.
pub const G: f64 = 9.806_65;

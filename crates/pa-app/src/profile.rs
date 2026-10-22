//! Profiles: what an instrument starts from, every value bound to a record or
//! to an assumption stated with its reason.
//!
//! # No number without provenance
//!
//! The engine starts from no number of its own. A profile
//! (`data/profiles/<name>.json`) binds every number an instrument starts
//! from, every field of the key, its capstan and its mass, of the frame,
//! its sections and its drawer, of the protocol, the bushings, the targets
//! and the experiment, either to a **record** in `data/params/`, by its file
//! and a JSON pointer to it, or to an **assumption** stated in the profile:
//! its value and unit, its marker, `S` for assumed pending measurement or `E`
//! with the rule that derives it, and why. Which numbers there are is not a
//! list kept here: the loader builds each part with a struct literal, every
//! field named, so a field added to a part does not compile until it is
//! bound; and the test `every_default_has_a_record_or_a_stated_assumption`
//! holds every number [`pa_core::validate::numbers`] reads of the design a
//! profile builds, whose destructuring the compiler holds exhaustive, to a
//! provenance carrying that number's very bits.
//!
//! # Units, converted once
//!
//! A record carries its unit, which is checked against the quantity the
//! number is (a length, a mass, grams, a torque) and converted to
//! the domain's SI by moving the decimal point of the recorded text, then
//! reading it once: 0.12 mm becomes the double nearest 0.00012 m, the bits
//! the Rust literal `0.00012` has, where dividing the double nearest 0.12 by
//! a thousand rounds twice and can land an ulp away. `serde_json` reads
//! every record correctly rounded (`float_roundtrip`), and a value with more
//! significant digits than a double holds is refused, so the shortest
//! decimal of what was read is the recorded text.
//!
//! # Printed lines and circles
//!
//! A source may give a part as a line or a circle rather than by the numbers
//! the key is described by: Hirschkorn's chamfer is a line through a point
//! with a normal, his capstan's head a circle about a center. A profile binds
//! them as printed, and the walk computes the key's numbers from them as it
//! builds the key, each kept as computed, with its rule and the input that
//! places it: the chamfer's rise and run from where the line meets the flat
//! and stands at the key's end, the capstan's axis and height from the
//! dome's center and radius, above its seat. A key height no source gives is
//! the one the key stands at as it arrives, read by the seated pose.
//!
//! # A base and a difference
//!
//! A profile may name a base and bind only what differs from it
//! (`docs/FOUNDATIONS.md` > The data model): the base's bindings, its key's kind, its
//! mass and how it arrives, each overridden where the profile says
//! otherwise.
//!
//! # Instruments, and what is borrowed
//!
//! A profile names the instruments it describes, each as its records'
//! sources name it, and a number it reads from a record of any other
//! instrument is **borrowed** ([`Known::Borrowed`]): what the owner measured
//! on his key frame is measured on his measured section, which describes
//! that frame, and borrowed on Hirschkorn's key, which does not. An assumption
//! stays one wherever it was made, and a record of no instrument, a protocol
//! or a drawer's stock of papers, is borrowed by none.
//!
//! # Embedded
//!
//! Profiles and the records they read are compiled in (`include_str!`), so
//! the browser's build and the shell's read the same bytes, and nothing is
//! read from a disk at run time.

use std::sync::OnceLock;

use pa_core::geometry::Point;
use pa_core::measure::measurement::{Basis, WippenAssemblyBases};
use pa_core::measure::seat::{SeatRefusal, Seating, SEAT_MARGIN};
use pa_core::measure::TouchweightProtocol;
use pa_core::mechanism::AxisFriction;
use pa_core::parts::frame::{
    Crown, Drawer, FrameSections, KeyKind, NotADrawer, PinShape, PunchingStack, RailSection, PAPER,
};
use pa_core::parts::mounts::{on_its_frame, KeyMount};
use pa_core::parts::wippen_assembly::{HeelCloth, PartMass, PinnedPart, Stretch, WippenHeel};
use pa_core::parts::{
    BalanceMortise, Capstan, FrameDesign, FrontMortise, KeyDesign, KeyMass, NotAWeighing,
    TopClearance, WippenAssembly, WippenAssemblyMass,
};
use pa_core::regulation::Targets;
use pa_core::solve::Statics;
use pa_core::validate::DRAWER;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[cfg(test)]
use ts_rs::TS;

/// The records a profile may point into, by their file name under
/// `data/params/`, as compiled in.
const RECORDS: [(&str, &str); 7] = [
    (
        "key-section.json",
        include_str!("../../../data/params/key-section.json"),
    ),
    (
        "key-frame-sections.json",
        include_str!("../../../data/params/key-frame-sections.json"),
    ),
    (
        "punching-papers.json",
        include_str!("../../../data/params/punching-papers.json"),
    ),
    (
        "touchweight-protocol.json",
        include_str!("../../../data/params/touchweight-protocol.json"),
    ),
    (
        "hirschkorn-boston-gp178-note52.json",
        include_str!("../../../data/params/hirschkorn-boston-gp178-note52.json"),
    ),
    (
        "hirschkorn-key-outline.json",
        include_str!("../../../data/params/hirschkorn-key-outline.json"),
    ),
    (
        "hirschkorn-wippen-outline.json",
        include_str!("../../../data/params/hirschkorn-wippen-outline.json"),
    ),
];

/// The profiles shipped, by name, as compiled in: `data/profiles/<name>.json`,
/// the standard one first.
const SHIPPED: [(&str, &str); 2] = [
    (
        "hirschkorn",
        include_str!("../../../data/profiles/hirschkorn.json"),
    ),
    (
        "measured-section",
        include_str!("../../../data/profiles/measured-section.json"),
    ),
];

/// The name of the owner's measured section's profile.
pub const SECTION: &str = "measured-section";

/// The name of the measured section weighed by the tests' weighing, a
/// profile the tests build on [`SECTION`].
#[cfg(test)]
pub(crate) const WEIGHED_SECTION: &str = "weighed-section";

/// The name of the profile [`crate::instrument::Instrument::standard`]
/// builds: Hirschkorn's key on his printed frame, whose mass, center of mass,
/// inertia, contact points and bushings were measured and published on one
/// action.
pub const STANDARD: &str = "hirschkorn";

// ---- what a profile describes ----------------------------------------------

/// How far a value can be trusted, by the marker its record or its profile
/// gives it (`data/params/README.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reliability {
    /// `V`: verified against a primary source, with its locator; a
    /// measurement of the owner's included.
    Verified,
    /// `E`: derived by calculation, the rule written beside it or the
    /// script committed under `tools/derivations/`.
    Derived,
    /// `S`: assumed, pending measurement.
    Assumed,
}

impl Reliability {
    fn of(marker: &str) -> Option<Self> {
        match marker {
            "V" => Some(Self::Verified),
            "E" => Some(Self::Derived),
            "S" => Some(Self::Assumed),
            _ => None,
        }
    }
}

/// Where one number an instrument starts from comes from.
#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    /// A record in `data/params/`.
    Record {
        /// The file, under `data/params/`.
        file: &'static str,
        /// The JSON pointer to the record, in that file.
        pointer: String,
        /// The record's marker.
        reliability: Reliability,
        /// Whether the file's source is a measurement of the owner's, not
        /// a published document.
        measured: bool,
        /// How the number is taken from the recorded one, exactly.
        derive: Derive,
        /// Whether the file's source is a figure the record was traced from.
        traced: bool,
        /// The instrument the file's numbers were measured or read on, as
        /// its source names it; none for a record of no instrument, as a
        /// protocol or a drawer's stock of papers is.
        instrument: Option<&'static str>,
    },
    /// Stated in a profile.
    Stated {
        /// The profile that states it.
        profile: String,
        /// `S`, assumed, or `E`, derived by `rule`.
        reliability: Reliability,
        /// The rule that derives it, for an `E`.
        rule: Option<String>,
        /// Why it is what it is.
        why: String,
    },
    /// Derived, as the instrument is built, from other numbers bound.
    Computed {
        /// How.
        rule: &'static str,
        /// The input that places it, whose provenance it shares where one
        /// does: a capstan's height above its seat, from its dome's center,
        /// is known as well as that center is.
        from: Option<&'static str>,
    },
}

/// How a number is taken from the record it is bound to, each exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Derive {
    /// As recorded.
    Whole,
    /// Half the recorded value: a radius from a diameter, a half-length
    /// from a length (`"derive": "half"`).
    Half,
    /// The recorded distance behind the balance pin as the position the
    /// key's frame signs, negative behind the pin (`"derive": "behind"`).
    Behind,
}

/// How a number an instrument starts from is known, by the provenance its
/// profile keeps: what an overlay marks each drawn stretch by, through the
/// number that places it ([`pa_core::contact::PlacedBy`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Known {
    /// Measured on a real part: a record of the owner's, marked `V`, of
    /// an instrument the profile describes or of none.
    Measured,
    /// Read from a published source: a record marked `V` that is no
    /// measurement of the owner's and no trace, of an instrument the
    /// profile describes or of none.
    Published,
    /// Traced from a figure: a record marked `E` whose file was read off
    /// one, of an instrument the profile describes.
    Traced,
    /// Derived by a rule from other numbers: stated `E`, a record marked
    /// `E` that is no trace, or computed as the instrument is built from no
    /// one input.
    Derived,
    /// Borrowed from another instrument: read from a record of an
    /// instrument the profile does not describe, whatever that record knows
    /// it as there. A record marked `S` stays assumed: an assumption is one
    /// on whichever instrument it was made.
    Borrowed,
    /// Assumed, pending measurement: marked `S`, recorded or stated.
    Assumed,
}

/// One number an instrument starts from, in the domain's SI, with where it
/// comes from.
#[derive(Debug, Clone, PartialEq)]
pub struct Provenance {
    /// The number's name, as [`pa_core::validate::numbers`] names it, or a
    /// setting's name for the targets and the experiment's weight, or the
    /// name of an input a rule derives numbers from.
    pub param: &'static str,
    /// Its value, SI unless its name carries a trade unit.
    pub value: f64,
    /// Where it comes from.
    pub source: Source,
}

/// How the bushings' friction is given.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bushings {
    /// The torque this many grams overcome at the measuring point with the
    /// key level, read on whatever key the instrument is built with.
    LevelWeightAtMeasuringPoint {
        /// The grams.
        grams: f64,
    },
    /// A torque, as a source gives it.
    Torque(AxisFriction),
}

impl Bushings {
    /// The friction these bushings hold on `key`, weighed by `protocol`.
    pub fn on(&self, key: &KeyDesign, protocol: &TouchweightProtocol) -> AxisFriction {
        match *self {
            Self::LevelWeightAtMeasuringPoint { grams } => {
                AxisFriction::of_weight_at(grams, protocol.measuring_point(key))
            }
            Self::Torque(friction) => friction,
        }
    }
}

/// How an instrument arrives on the bench.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arrival {
    /// Regulated: the papers on its punchings are the regulation solve's,
    /// laid from the drawer to its targets.
    Regulated,
    /// As recorded: its papers as the profile binds them.
    AsRecorded,
}

/// The color a punching paper is supplied in, as its record names it: what
/// the trade knows a paper's thickness by. The view chooses how to paint
/// each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[serde(rename_all = "snake_case")]
#[allow(missing_docs)]
pub enum PaperColor {
    White,
    PastelGreen,
    SkyBlue,
    Camel,
    Black,
    /// Plain card, above half a millimeter: no longer graded by color.
    Card,
}

impl PaperColor {
    fn of(name: &str) -> Option<Self> {
        match name {
            "white" => Some(Self::White),
            "pastel green" => Some(Self::PastelGreen),
            "sky blue" => Some(Self::SkyBlue),
            "camel" => Some(Self::Camel),
            "black" => Some(Self::Black),
            "card" => Some(Self::Card),
            _ => None,
        }
    }
}

/// What an instrument starts from, every value in the domain's units.
#[derive(Debug, Clone, PartialEq)]
pub struct Start {
    /// The key.
    pub key: KeyDesign,
    /// Which row of pins it stands on.
    pub kind: KeyKind,
    /// What is known of its mass.
    pub mass: KeyMass,
    /// The frame, its drawer, and the papers it arrives with.
    pub frame: FrameDesign,
    /// The protocol it is weighed by.
    pub protocol: TouchweightProtocol,
    /// The bushings' friction.
    pub bushings: Bushings,
    /// What that friction rests on.
    pub friction_basis: Basis,
    /// The regulation's targets.
    pub targets: Targets,
    /// The weight on the measuring point, grams.
    pub weight_g: f64,
    /// How it arrives.
    pub arrival: Arrival,
    /// The wippen assembly the key carries on its capstan, where it carries
    /// one.
    pub wippen_assembly: Option<WippenAssemblyStart>,
}

/// A wippen assembly a profile puts on the key's capstan: its parts, its
/// mass with the jack and the repetition lever at rest on it, and what each
/// rests on.
#[derive(Debug, Clone, PartialEq)]
pub struct WippenAssemblyStart {
    /// Its parts and their place.
    pub assembly: WippenAssembly,
    /// Its mass.
    pub mass: WippenAssemblyMass,
    /// What its mass and its center's friction rest on.
    pub bases: WippenAssemblyBases,
}

/// A profile, loaded: what an instrument starts from, and where each number
/// of it comes from.
#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    /// Its name, the file's stem under `data/profiles/`.
    pub name: String,
    /// What it describes.
    pub what: String,
    /// The profile it is a difference from, if any.
    pub base: Option<String>,
    /// The instruments it describes, each by the name its records' source
    /// gives it: a number read from a record of any other is borrowed.
    pub instruments: Vec<String>,
    /// What an instrument built from it starts from.
    pub start: Start,
    /// Every number of the start, and the inputs its rules read, each with
    /// where it comes from.
    pub provenance: Vec<Provenance>,
    /// The color of each paper the drawer holds, in the drawer's order.
    pub paper_colors: Vec<PaperColor>,
}

/// Why a profile does not load.
#[derive(Debug, Clone, PartialEq)]
pub enum LoadError {
    /// Its text, or a record's, is not the JSON it should be.
    Json {
        /// The file.
        file: String,
        /// What `serde_json` says.
        why: String,
    },
    /// A base no profile is named.
    UnknownBase(String),
    /// An instrument the profile says it describes that no record names.
    UnknownInstrument(String),
    /// A profile that is its own base, through others or directly.
    CircularBase(String),
    /// A number the instrument starts from that the profile does not bind.
    Unbound(&'static str),
    /// A binding no number reads.
    Unused(String),
    /// A number bound twice in one profile.
    BoundTwice(String),
    /// A binding that is no record pointer, no stated value and no stated
    /// absence, or one that mixes them.
    Malformed {
        /// The number.
        param: String,
        /// What is wrong with it.
        why: &'static str,
    },
    /// A record file no profile may read.
    UnknownRecord(String),
    /// A pointer that leads nowhere in its record.
    NoSuchRecord {
        /// The file.
        file: &'static str,
        /// The pointer.
        pointer: String,
    },
    /// What a pointer leads to is no record: no value, no unit or no
    /// marker.
    NotARecord {
        /// The file.
        file: &'static str,
        /// The pointer.
        pointer: String,
    },
    /// A unit that is not the quantity's.
    WrongUnit {
        /// The number.
        param: &'static str,
        /// The unit given.
        unit: String,
    },
    /// A value given with more significant digits than a double holds, so
    /// that it cannot be converted rounding once.
    TooManyDigits {
        /// The number.
        param: &'static str,
    },
    /// A marker that is not one a value of its kind may carry.
    WrongMarker {
        /// The number.
        param: String,
        /// The marker given.
        marker: String,
    },
    /// A stated value without its reason, or an `E` without its rule.
    Unexplained(String),
    /// A choice the profile does not make, or one no instrument has.
    Choice {
        /// What is chosen.
        what: &'static str,
        /// What was given, empty where nothing was.
        given: String,
    },
    /// Papers that are no drawer.
    NotADrawer(NotADrawer),
    /// A paper's color no drawer is graded in.
    UnknownColor(String),
    /// A paper on a punching the drawer does not hold.
    PaperNotInTheDrawer(f64),
    /// A key weight ratio read with no load.
    NoKeyRatioLoad,
    /// A key weighed at numbers no key has.
    NotAWeighing(NotAWeighing),
    /// The key height a key stands at could not be read, where no source
    /// gives one.
    KeyHeightNotRead(SeatRefusal),
}

// ---- the files --------------------------------------------------------------

/// A profile's file, as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileFile {
    profile: String,
    what: String,
    base: Option<String>,
    /// The instruments it describes, as its records' sources name them.
    instruments: Option<Vec<String>>,
    key_kind: Option<Choice>,
    key_mass: Option<Choice>,
    arrival: Option<Choice>,
    wippen_assembly: Option<Choice>,
    values: Vec<BindingFile>,
}

/// A choice a profile makes, with its reason.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Choice {
    is: String,
    why: String,
}

/// One binding, as written: a record pointer, a stated value or list, or a
/// stated absence.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BindingFile {
    param: String,
    record: Option<String>,
    pointer: Option<String>,
    derive: Option<String>,
    value: Option<f64>,
    values: Option<Vec<f64>>,
    unit: Option<String>,
    reliability: Option<String>,
    rule: Option<String>,
    why: Option<String>,
    none: Option<bool>,
}

/// A binding, read, with the profile that states it.
#[derive(Debug, Clone)]
struct Binding {
    param: String,
    profile: String,
    how: How,
}

#[derive(Debug, Clone)]
enum How {
    Record {
        file: &'static str,
        pointer: String,
        derive: Derive,
    },
    Stated {
        values: Vec<f64>,
        list: bool,
        unit: String,
        reliability: Reliability,
        rule: Option<String>,
        why: String,
    },
    Absent,
}

impl BindingFile {
    fn read(self, profile: &str) -> Result<Binding, LoadError> {
        let malformed = |why| LoadError::Malformed {
            param: self.param.clone(),
            why,
        };
        let stated = self.value.is_some() || self.values.is_some();
        let recorded = self.record.is_some() || self.pointer.is_some();
        let absent = self.none.is_some();
        let how = match (recorded, stated, absent) {
            (true, false, false) => {
                if self.unit.is_some()
                    || self.reliability.is_some()
                    || self.rule.is_some()
                    || self.why.is_some()
                {
                    return Err(malformed(
                        "a record carries its own unit, marker and reason",
                    ));
                }
                let (Some(file), Some(pointer)) = (self.record.as_deref(), self.pointer.clone())
                else {
                    return Err(malformed("a record binding names its file and its pointer"));
                };
                let file = Records::get()
                    .file(file)
                    .ok_or_else(|| LoadError::UnknownRecord(file.to_owned()))?;
                let derive = match self.derive.as_deref() {
                    None => Derive::Whole,
                    Some("half") => Derive::Half,
                    Some("behind") => Derive::Behind,
                    Some(_) => {
                        return Err(malformed(
                            "a record is taken whole, halved or as a distance behind the pin",
                        ))
                    }
                };
                How::Record {
                    file,
                    pointer,
                    derive,
                }
            }
            (false, true, false) => {
                if self.derive.is_some() {
                    return Err(malformed("only a record's value is derived from"));
                }
                let (values, list) = match (self.value, self.values.clone()) {
                    (Some(v), None) => (vec![v], false),
                    (None, Some(vs)) => (vs, true),
                    _ => return Err(malformed("a value or a list of values, not both")),
                };
                let unit = self
                    .unit
                    .clone()
                    .ok_or_else(|| malformed("a stated value carries its unit"))?;
                let marker = self.reliability.clone().unwrap_or_default();
                let reliability = match Reliability::of(&marker) {
                    Some(r @ (Reliability::Assumed | Reliability::Derived)) => r,
                    _ => {
                        return Err(LoadError::WrongMarker {
                            param: self.param.clone(),
                            marker,
                        })
                    }
                };
                let why = self
                    .why
                    .clone()
                    .filter(|w| !w.trim().is_empty())
                    .ok_or_else(|| LoadError::Unexplained(self.param.clone()))?;
                let rule = self.rule.clone().filter(|r| !r.trim().is_empty());
                if reliability == Reliability::Derived && rule.is_none() {
                    return Err(LoadError::Unexplained(self.param.clone()));
                }
                How::Stated {
                    values,
                    list,
                    unit,
                    reliability,
                    rule,
                    why,
                }
            }
            (false, false, true) => {
                if self.none != Some(true)
                    || self.why.as_deref().is_none_or(|w| w.trim().is_empty())
                {
                    return Err(LoadError::Unexplained(self.param.clone()));
                }
                How::Absent
            }
            _ => {
                return Err(malformed(
                    "a binding is a record, a stated value or a stated absence",
                ))
            }
        };
        Ok(Binding {
            param: self.param,
            profile: profile.to_owned(),
            how,
        })
    }
}

/// The records, parsed once.
struct Records {
    files: Vec<(&'static str, Value)>,
}

impl Records {
    fn get() -> &'static Self {
        static RECORDS_PARSED: OnceLock<Records> = OnceLock::new();
        RECORDS_PARSED.get_or_init(|| Self {
            files: RECORDS
                .iter()
                .map(|&(name, text)| {
                    let value = serde_json::from_str(text)
                        .unwrap_or_else(|e| panic!("data/params/{name} is not JSON: {e}"));
                    (name, value)
                })
                .collect(),
        })
    }

    /// The file of that name, as the static name records carry.
    fn file(&self, name: &str) -> Option<&'static str> {
        RECORDS.iter().map(|&(n, _)| n).find(|&n| n == name)
    }

    fn json(&self, file: &str) -> &Value {
        &self
            .files
            .iter()
            .find(|(n, _)| *n == file)
            .expect("a record file is one of the records")
            .1
    }

    /// Whether `file`'s source is a measurement of the owner's.
    fn measured(&self, file: &str) -> bool {
        self.json(file)
            .get("source")
            .and_then(|s| s.get("measured_by"))
            .is_some()
    }

    /// Whether `file` was traced from a figure.
    fn traced(&self, file: &str) -> bool {
        self.json(file)
            .get("source")
            .and_then(|s| s.get("traced_from"))
            .is_some()
    }

    /// The instrument `file`'s numbers were measured or read on, as its
    /// source names it; none for a file of no instrument.
    fn instrument(&'static self, file: &str) -> Option<&'static str> {
        self.json(file)
            .get("source")
            .and_then(|s| s.get("instrument"))
            .and_then(Value::as_str)
    }

    /// Whether any record names `instrument` as its source's.
    fn names(&'static self, instrument: &str) -> bool {
        RECORDS
            .iter()
            .any(|&(file, _)| self.instrument(file) == Some(instrument))
    }

    /// What `pointer` leads to in `file`: a JSON pointer, in which an
    /// array's element may also be named by its `name` member, as
    /// `/records/stick_thickness` names the record whose name that is.
    fn at(&self, file: &'static str, pointer: &str) -> Result<&Value, LoadError> {
        let nowhere = || LoadError::NoSuchRecord {
            file,
            pointer: pointer.to_owned(),
        };
        let mut v = self.json(file);
        if pointer.is_empty() {
            return Ok(v);
        }
        let tokens = pointer.strip_prefix('/').ok_or_else(nowhere)?;
        for raw in tokens.split('/') {
            let token = raw.replace("~1", "/").replace("~0", "~");
            v = match v {
                Value::Object(map) => map.get(&token).ok_or_else(nowhere)?,
                Value::Array(items) => match token.parse::<usize>() {
                    Ok(i) => items.get(i).ok_or_else(nowhere)?,
                    Err(_) => items
                        .iter()
                        .find(|item| item.get("name").and_then(Value::as_str) == Some(&token))
                        .ok_or_else(nowhere)?,
                },
                _ => return Err(nowhere()),
            };
        }
        Ok(v)
    }
}

/// A record object's value, unit and marker.
fn record_of<'v>(
    v: &'v Value,
    file: &'static str,
    pointer: &str,
) -> Result<(f64, &'v str, Reliability), LoadError> {
    let not_a_record = || LoadError::NotARecord {
        file,
        pointer: pointer.to_owned(),
    };
    let value = v
        .get("value")
        .and_then(Value::as_f64)
        .ok_or_else(not_a_record)?;
    let unit = v
        .get("unit")
        .and_then(Value::as_str)
        .ok_or_else(not_a_record)?;
    let reliability = v
        .get("reliability")
        .and_then(Value::as_str)
        .and_then(Reliability::of)
        .ok_or_else(not_a_record)?;
    Ok((value, unit, reliability))
}

// ---- quantities and units ---------------------------------------------------

/// What a number measures, and so which units it may be given in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Quantity {
    /// A length or a position, meters.
    Length,
    /// A mass, kilograms.
    Mass,
    /// A weight or a mass in grams, as its name says.
    Grams,
    /// A torque, newton-meters.
    Torque,
    /// A pure number, as a unit vector's components are.
    Ratio,
    /// A second moment of mass, kilogram square meters.
    Inertia,
    /// An angle, radians.
    Angle,
}

impl Quantity {
    /// By how many decades `unit` stands from this quantity's unit: its
    /// value times ten to that power is the quantity's; `None` for a unit
    /// that is not the quantity's.
    fn decades(self, unit: &str) -> Option<i32> {
        match (self, unit) {
            (Self::Length, "mm") | (Self::Mass, "g") => Some(-3),
            (Self::Length, "m")
            | (Self::Mass, "kg")
            | (Self::Grams, "g")
            | (Self::Torque, "N·m")
            | (Self::Ratio, "dimensionless")
            | (Self::Inertia, "kg·m²")
            | (Self::Angle, "rad") => Some(0),
            (Self::Grams, "kg") => Some(3),
            _ => None,
        }
    }
}

/// The most significant digits a decimal may have and still be read back
/// from its double unchanged.
const DIGITS: usize = 15;

/// `x`, read from a decimal, times ten to the `decades`, rounded once from
/// that decimal: the shortest decimal that reads back to `x`, which is the
/// decimal `x` was read from when that has at most [`DIGITS`] significant
/// digits, with its exponent moved, read again. `None` where `x` has more
/// digits than that.
fn shifted(x: f64, decades: i32) -> Option<f64> {
    if decades == 0 {
        return Some(x);
    }
    let text = format!("{x:e}");
    let (mantissa, exponent) = text.split_once('e')?;
    if mantissa.chars().filter(char::is_ascii_digit).count() > DIGITS {
        return None;
    }
    let exponent: i32 = exponent.parse().ok()?;
    format!("{mantissa}e{}", exponent + decades).parse().ok()
}

// ---- the walk ---------------------------------------------------------------

/// One value read off a binding: in its quantity's SI, where it comes from,
/// and the record object it was read from, if a record.
type Read = (f64, Source, Option<&'static Value>);

/// The bindings of a profile and its bases, merged, as the walk reads them.
struct Bindings {
    profile: String,
    bindings: Vec<(Binding, bool)>,
    provenance: Vec<Provenance>,
}

impl Bindings {
    fn find(&mut self, param: &'static str) -> Result<&mut (Binding, bool), LoadError> {
        self.bindings
            .iter_mut()
            .find(|(b, _)| b.param == param)
            .ok_or(LoadError::Unbound(param))
    }

    fn has(&self, param: &str) -> bool {
        self.bindings.iter().any(|(b, _)| b.param == param)
    }

    /// The values bound to `param`, each in `quantity`'s SI with its
    /// source, and the record objects they were read from; marked read.
    fn read(
        &mut self,
        param: &'static str,
        quantity: Quantity,
        list: bool,
    ) -> Result<Vec<Read>, LoadError> {
        let (binding, used) = self.find(param)?;
        *used = true;
        let malformed = |why| LoadError::Malformed {
            param: param.to_owned(),
            why,
        };
        let convert = |value: f64, unit: &str| -> Result<f64, LoadError> {
            let decades = quantity.decades(unit).ok_or_else(|| LoadError::WrongUnit {
                param,
                unit: unit.to_owned(),
            })?;
            shifted(value, decades).ok_or(LoadError::TooManyDigits { param })
        };
        match &binding.how {
            How::Absent => Err(malformed("bound to nothing where a number is wanted")),
            How::Stated {
                values,
                list: is_list,
                unit,
                reliability,
                rule,
                why,
            } => {
                if *is_list != list {
                    return Err(malformed(if list {
                        "a list is wanted"
                    } else {
                        "one value is wanted"
                    }));
                }
                values
                    .iter()
                    .map(|&v| {
                        Ok((
                            convert(v, unit)?,
                            Source::Stated {
                                profile: binding.profile.clone(),
                                reliability: *reliability,
                                rule: rule.clone(),
                                why: why.clone(),
                            },
                            None,
                        ))
                    })
                    .collect()
            }
            How::Record {
                file,
                pointer,
                derive,
            } => {
                let records = Records::get();
                let at = records.at(file, pointer)?;
                let items: Vec<(&'static Value, String)> = match (at, list) {
                    (Value::Array(items), true) => items
                        .iter()
                        .enumerate()
                        .map(|(i, item)| (item, format!("{pointer}/{i}")))
                        .collect(),
                    (Value::Object(_), false) => vec![(at, pointer.clone())],
                    _ => {
                        return Err(LoadError::NotARecord {
                            file,
                            pointer: pointer.clone(),
                        })
                    }
                };
                items
                    .into_iter()
                    .map(|(item, pointer)| {
                        let (value, unit, reliability) = record_of(item, file, &pointer)?;
                        let si = convert(value, unit)?;
                        Ok((
                            match derive {
                                Derive::Whole => si,
                                Derive::Half => 0.5 * si,
                                Derive::Behind => -si,
                            },
                            Source::Record {
                                file,
                                pointer,
                                reliability,
                                measured: records.measured(file),
                                derive: *derive,
                                traced: records.traced(file),
                                instrument: records.instrument(file),
                            },
                            Some(item),
                        ))
                    })
                    .collect()
            }
        }
    }

    /// The one number bound to `param`, in `quantity`'s SI, its provenance
    /// kept.
    fn number(&mut self, param: &'static str, quantity: Quantity) -> Result<f64, LoadError> {
        let (value, source, _) = self
            .read(param, quantity, false)?
            .pop()
            .expect("one value read");
        self.provenance.push(Provenance {
            param,
            value,
            source,
        });
        Ok(value)
    }

    fn length(&mut self, param: &'static str) -> Result<f64, LoadError> {
        self.number(param, Quantity::Length)
    }

    /// The list bound to `param`, in `quantity`'s SI, each value's
    /// provenance kept under `each`, the name each of the list's numbers
    /// has, with the record objects it was read from.
    fn list(
        &mut self,
        param: &'static str,
        each: &'static str,
        quantity: Quantity,
    ) -> Result<Vec<(f64, Option<&'static Value>)>, LoadError> {
        let read = self.read(param, quantity, true)?;
        Ok(read
            .into_iter()
            .map(|(value, source, item)| {
                self.provenance.push(Provenance {
                    param: each,
                    value,
                    source,
                });
                (value, item)
            })
            .collect())
    }

    /// That `param` is bound to nothing, as the profile states.
    fn absent(&mut self, param: &'static str) -> Result<(), LoadError> {
        let (binding, used) = self.find(param)?;
        *used = true;
        match binding.how {
            How::Absent => Ok(()),
            _ => Err(LoadError::Malformed {
                param: param.to_owned(),
                why: "a number bound where nothing is wanted",
            }),
        }
    }

    /// The number bound to `param`, or nothing where the profile states
    /// that there is none.
    fn optional(
        &mut self,
        param: &'static str,
        quantity: Quantity,
    ) -> Result<Option<f64>, LoadError> {
        let absent = matches!(self.find(param)?.0.how, How::Absent);
        if absent {
            self.absent(param)?;
            Ok(None)
        } else {
            self.number(param, quantity).map(Some)
        }
    }

    /// A number the walk derives, its provenance kept, and the input that
    /// places it where one does.
    fn computed(
        &mut self,
        param: &'static str,
        value: f64,
        rule: &'static str,
        from: Option<&'static str>,
    ) {
        self.provenance.push(Provenance {
            param,
            value,
            source: Source::Computed { rule, from },
        });
    }

    /// Every binding has been read.
    fn finish(&self) -> Result<(), LoadError> {
        match self.bindings.iter().find(|(_, used)| !used) {
            Some((b, _)) => Err(LoadError::Unused(b.param.clone())),
            None => Ok(()),
        }
    }

    /// The basis of what `param`'s binding gives.
    fn basis(&self, param: &str) -> Basis {
        let source = self
            .provenance
            .iter()
            .rev()
            .find(|p| p.param == param)
            .map(|p| &p.source);
        match source {
            Some(Source::Record { measured: true, .. }) => Basis::Measured,
            Some(Source::Record { .. }) => Basis::Published,
            Some(Source::Stated {
                reliability: Reliability::Derived,
                ..
            }) => Basis::Estimated,
            _ => Basis::Assumed,
        }
    }
}

/// The names of one rail's section's numbers, as
/// [`pa_core::validate::numbers`] names them.
struct RailNames {
    half_length: &'static str,
    crown: &'static str,
    crown_half_length: &'static str,
    crown_height: &'static str,
    pin: &'static str,
    pin_diameter: &'static str,
    pin_length: &'static str,
    pin_thickness: &'static str,
    pin_height: &'static str,
    punching_radius: &'static str,
    punching_hole_radius: &'static str,
    sharp_row_offset: &'static str,
}

/// Declares a rail's names under its prefix.
macro_rules! rail_names {
    ($rail:literal) => {
        RailNames {
            half_length: concat!($rail, ".half_length"),
            crown: concat!($rail, ".crown"),
            crown_half_length: concat!($rail, ".crown.half_length"),
            crown_height: concat!($rail, ".crown.height"),
            pin: concat!($rail, ".pin"),
            pin_diameter: concat!($rail, ".pin.diameter"),
            pin_length: concat!($rail, ".pin.length"),
            pin_thickness: concat!($rail, ".pin.thickness"),
            pin_height: concat!($rail, ".pin_height"),
            punching_radius: concat!($rail, ".punching_radius"),
            punching_hole_radius: concat!($rail, ".punching_hole_radius"),
            sharp_row_offset: concat!($rail, ".sharp_row_offset"),
        }
    };
}

const BALANCE_RAIL: RailNames = rail_names!("balance_rail");
const FRONT_RAIL: RailNames = rail_names!("front_rail");
const BACK_RAIL: RailNames = rail_names!("back_rail");

/// One rail's section: its crown where its half-length is bound, its pin
/// round where a diameter is, oval where a length is, and either absent
/// where the profile says so.
fn rail(b: &mut Bindings, n: &RailNames) -> Result<RailSection, LoadError> {
    let crown = if b.has(n.crown_half_length) {
        Some(Crown {
            half_length: b.length(n.crown_half_length)?,
            height: b.length(n.crown_height)?,
        })
    } else {
        b.absent(n.crown)
            .map_err(|_| LoadError::Unbound(n.crown_half_length))?;
        None
    };
    let pin = if b.has(n.pin_diameter) {
        Some(PinShape::Round {
            diameter: b.length(n.pin_diameter)?,
        })
    } else if b.has(n.pin_length) {
        Some(PinShape::Oval {
            length: b.length(n.pin_length)?,
            thickness: b.length(n.pin_thickness)?,
        })
    } else {
        b.absent(n.pin)
            .map_err(|_| LoadError::Unbound(n.pin_diameter))?;
        None
    };
    Ok(RailSection {
        half_length: b.length(n.half_length)?,
        crown,
        pin,
        pin_height: b.length(n.pin_height)?,
        punching_radius: b.length(n.punching_radius)?,
        punching_hole_radius: b.length(n.punching_hole_radius)?,
        sharp_row_offset: b.length(n.sharp_row_offset)?,
    })
}

/// The choice `c` makes among `options`, by name.
fn choose<T: Copy>(
    what: &'static str,
    c: Option<&Choice>,
    options: &[(&str, T)],
) -> Result<T, LoadError> {
    let given = c.map(|c| c.is.as_str()).unwrap_or_default();
    if c.is_some_and(|c| c.why.trim().is_empty()) {
        return Err(LoadError::Unexplained(what.to_owned()));
    }
    options
        .iter()
        .find(|(name, _)| *name == given)
        .map(|&(_, t)| t)
        .ok_or_else(|| LoadError::Choice {
            what,
            given: given.to_owned(),
        })
}

/// How a key's mass is described.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MassModel {
    Unweighed,
    Weighed,
    Published,
}

/// The chamfer under the tail, where a profile gives it as a printed line
/// (`chamfer.x`, `chamfer.y`, `chamfer.normal.x`, `chamfer.normal.y`): its
/// rise at the key's end and its run from there to where it meets the
/// underside's flat, `y = 0`, by the rule a line through a point with a
/// normal gives; or why the line is no chamfer.
fn chamfer_of(b: &mut Bindings, tail_end: f64) -> Result<(f64, f64), LoadError> {
    let through = Point::new(b.length("chamfer.x")?, b.length("chamfer.y")?);
    let (nx, ny) = (
        b.number("chamfer.normal.x", Quantity::Ratio)?,
        b.number("chamfer.normal.y", Quantity::Ratio)?,
    );
    // n · (p − through) = 0: the flat at y = 0, the key's end at x = tail_end.
    let kink = through.x + ny * through.y / nx;
    let run = kink - tail_end;
    let rise = through.y - nx * (tail_end - through.x) / ny;
    if !(nx != 0.0 && ny < 0.0 && run > 0.0 && rise.is_finite()) {
        return Err(LoadError::Malformed {
            param: "chamfer.x".to_owned(),
            why: "a chamfer's line faces down and meets the flat in front of the key's end",
        });
    }
    b.computed(
        "tail_rise_run",
        run,
        "from the key's end to where the chamfer's line meets the underside's flat",
        Some("chamfer.normal.y"),
    );
    b.computed(
        "tail_rise",
        rise,
        "the chamfer's line, at the key's end, above the underside's flat",
        Some("chamfer.normal.y"),
    );
    Ok((rise, run))
}

/// The clearance cut in the keystick's top for the wippen rail, by its two
/// ends, signed from the balance pin, and its depth below the full top; or
/// none, where the profile states that the key has none.
fn top_clearance(b: &mut Bindings) -> Result<Option<TopClearance>, LoadError> {
    const BACK: &str = "top_clearance.back";
    if b.has(BACK) {
        Ok(Some(TopClearance {
            back: b.length(BACK)?,
            front: b.length("top_clearance.front")?,
            depth: b.length("top_clearance.depth")?,
        }))
    } else {
        b.absent("top_clearance")
            .map_err(|_| LoadError::Unbound(BACK))?;
        Ok(None)
    }
}

/// The key height the key stands at on its frame as it arrives, as the
/// seated pose reads it, the sheet's own reading: the key seated at the top
/// of its stroke, its front's top above the keybed. For a profile that
/// publishes no key height, whose regulation keeps the height the key
/// stands at.
pub(crate) fn standing_key_height(start: &Start) -> Result<f64, LoadError> {
    let not_read = |why: SeatRefusal| LoadError::KeyHeightNotRead(why);
    let mass = start
        .key
        .section_centroid()
        .and_then(|c| start.mass.props(c.y));
    let k = on_its_frame(
        KeyMount::OnPunching,
        &start.key,
        &start.frame,
        start.kind,
        mass,
        0.0,
        start
            .wippen_assembly
            .as_ref()
            .map(|w| (&w.assembly, &w.mass)),
    )
    .map_err(|p| not_read(SeatRefusal::Mechanism(p)))?;
    let seats = Seating {
        mounted: &k.mounted,
        capstan: k.capstan,
        measuring_point: start.protocol.measuring_point_on_key(&start.key),
        margin: SEAT_MARGIN,
        standing: &[],
    }
    .seats(&mut Statics::new())
    .map_err(not_read)?;
    Ok(seats.heights(&k.mounted, k.front_top).0)
}

/// Everything an instrument starts from, read off the bindings. Each part
/// is built with every field named, so a field added to one is a compile
/// error here until it is bound.
fn walk(b: &mut Bindings, file: &ProfileFile) -> Result<(Start, Vec<PaperColor>), LoadError> {
    // A capstan whose head is a dome may be given by the dome's center and
    // radius, as a printed circle: its axis through the center and its top
    // the radius above it. Its height above its seat follows once the key
    // is built.
    let dome_center = if b.has("capstan.dome.x") {
        Some(Point::new(
            b.length("capstan.dome.x")?,
            b.length("capstan.dome.y")?,
        ))
    } else {
        None
    };
    let capstan_screw = Capstan {
        height: match dome_center {
            Some(_) => f64::NAN,
            None => b.length("capstan.height")?,
        },
        diameter: b.length("capstan.diameter")?,
        head_diameter: b.length("capstan.head_diameter")?,
        head_height: b.length("capstan.head_height")?,
        cove_height: b.length("capstan.cove_height")?,
        shaft_length: b.length("capstan.shaft_length")?,
        tip_height: b.length("capstan.tip_height")?,
        dome_radius: b.optional("capstan.dome_radius", Quantity::Length)?,
    };
    let tail_end = b.length("tail_end")?;
    let (tail_rise, tail_rise_run) = if b.has("chamfer.x") {
        chamfer_of(b, tail_end)?
    } else {
        (b.length("tail_rise")?, b.length("tail_rise_run")?)
    };
    let mut key = KeyDesign {
        front_edge: b.length("front_edge")?,
        tail_end,
        capstan: match dome_center {
            Some(c) => c.x,
            None => b.length("capstan.x")?,
        },
        front_mortise: FrontMortise {
            from_front_edge: b.length("front_mortise.from_front_edge")?,
            cutter_diameter: b.length("front_mortise.cutter_diameter")?,
            travel: b.length("front_mortise.travel")?,
            depth: b.length("front_mortise.depth")?,
            groove_depth: b.length("front_mortise.groove_depth")?,
            bushing_thickness: b.length("front_mortise.bushing_thickness")?,
            bushing_height: b.length("front_mortise.bushing_height")?,
            bushing_flat_width: b.length("front_mortise.bushing_flat_width")?,
        },
        stick_height: b.length("stick_height")?,
        capsule_height: b.length("capsule_height")?,
        capsule_length: b.length("capsule_length")?,
        capsule_center: b.optional("capsule_center", Quantity::Length)?,
        capsule_end_radius: b.length("capsule_end_radius")?,
        balance_mortise: BalanceMortise {
            cutter_diameter: b.length("balance_mortise.cutter_diameter")?,
            travel: b.length("balance_mortise.travel")?,
            floor: b.length("balance_mortise.floor")?,
            hole_diameter: b.length("balance_mortise.hole_diameter")?,
            bushing_thickness: b.length("balance_mortise.bushing_thickness")?,
            bushing_depth: b.length("balance_mortise.bushing_depth")?,
            side_hole_height: b.length("balance_mortise.side_hole_height")?,
        },
        back_check_block_height: b.length("back_check_block_height")?,
        back_check_block_back: b.length("back_check_block_back")?,
        back_check_block_length: b.length("back_check_block_length")?,
        top_clearance: top_clearance(b)?,
        tail_height: b.length("tail_height")?,
        tail_length: b.length("tail_length")?,
        damper_felt: b.optional("damper_felt", Quantity::Length)?,
        tail_rise,
        tail_rise_run,
        keytop_thickness: b.length("keytop_thickness")?,
        keytop_length: b.length("keytop_length")?,
        front_board: b.length("front_board")?,
        keytop_overhang: b.length("keytop_overhang")?,
        width: b.length("width")?,
        capstan_screw,
    };
    if let Some(center) = dome_center {
        let Some(radius) = key.capstan_screw.dome_radius else {
            return Err(LoadError::Malformed {
                param: "capstan.dome.x".to_owned(),
                why: "a dome given by its center is given its radius",
            });
        };
        key.capstan_screw.height = center.y + radius - key.capstan_seat();
        b.computed(
            "capstan.x",
            key.capstan,
            "the capstan's axis, through its dome's center",
            Some("capstan.dome.x"),
        );
        b.computed(
            "capstan.height",
            key.capstan_screw.height,
            "the dome's top, its center's height plus its radius, above the capstan's seat",
            Some("capstan.dome.y"),
        );
    }
    let kind = choose(
        "key_kind",
        file.key_kind.as_ref(),
        &[("natural", KeyKind::Natural), ("sharp", KeyKind::Sharp)],
    )?;
    let mass = match choose(
        "key_mass",
        file.key_mass.as_ref(),
        &[
            ("unweighed", MassModel::Unweighed),
            ("weighed", MassModel::Weighed),
            ("published", MassModel::Published),
        ],
    )? {
        MassModel::Unweighed => KeyMass::Unweighed,
        MassModel::Published => {
            let mass = b.number("published.mass", Quantity::Mass)?;
            let center = Point::new(
                b.length("published.center.x")?,
                b.length("published.center.y")?,
            );
            let inertia = b.number("published.inertia", Quantity::Inertia)?;
            KeyMass::published(&key, mass, center, inertia).map_err(LoadError::NotAWeighing)?
        }
        MassModel::Weighed => {
            let mass_g = b.number("weighing.mass_g", Quantity::Grams)?;
            let front_weight_g = b.number("weighing.front_weight_g", Quantity::Grams)?;
            let from_lip = b.length("weighing.from_lip")?;
            let weighed = KeyMass::weighed(&key, mass_g, front_weight_g, from_lip)
                .map_err(LoadError::NotAWeighing)?;
            if let KeyMass::Weighed {
                mass,
                static_moment_n_m,
                datum,
            } = weighed
            {
                b.computed(
                    "weighing.mass",
                    mass,
                    "the mass weighed, in kilograms",
                    Some("weighing.mass_g"),
                );
                b.computed(
                    "weighing.static_moment",
                    static_moment_n_m,
                    "the front weight times the arm it was read on, times standard gravity",
                    Some("weighing.front_weight_g"),
                );
                b.computed(
                    "weighing.arm",
                    datum.arm_m,
                    "the covering's lip less the roller's distance back from it",
                    Some("weighing.from_lip"),
                );
            }
            weighed
        }
    };
    let protocol = TouchweightProtocol {
        measuring_point_from_lip: b.length("measuring_point_from_lip")?,
        key_ratio_load_g: b.number("key_ratio_load_g", Quantity::Grams)?,
    };
    if protocol.key_ratio_load_g.is_nan() || protocol.key_ratio_load_g <= 0.0 {
        return Err(LoadError::NoKeyRatioLoad);
    }
    let papers = b.list("drawer", DRAWER, Quantity::Length)?;
    let mut paper_colors = Vec::with_capacity(papers.len());
    for (_, item) in &papers {
        let name = item
            .and_then(|i| i.get("color"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        paper_colors
            .push(PaperColor::of(name).ok_or_else(|| LoadError::UnknownColor(name.to_owned()))?);
    }
    let drawer =
        Drawer::graded(papers.iter().map(|&(t, _)| t).collect()).map_err(LoadError::NotADrawer)?;
    let arrival = choose(
        "arrival",
        file.arrival.as_ref(),
        &[
            ("regulated", Arrival::Regulated),
            ("as_recorded", Arrival::AsRecorded),
        ],
    )?;
    let stacked = |b: &mut Bindings, param: &'static str| -> Result<Vec<f64>, LoadError> {
        match arrival {
            Arrival::Regulated => Ok(Vec::new()),
            Arrival::AsRecorded => {
                let papers: Vec<f64> = b
                    .list(param, PAPER.name, Quantity::Length)?
                    .into_iter()
                    .map(|(t, _)| t)
                    .collect();
                match papers.iter().find(|&&p| !drawer.holds(p)) {
                    Some(&p) => Err(LoadError::PaperNotInTheDrawer(p)),
                    None => Ok(papers),
                }
            }
        }
    };
    let balance_papers = stacked(b, "balance_punching.papers")?;
    let front_papers = stacked(b, "front_punching.papers")?;
    let frame = FrameDesign {
        balance_rail_height: b.length("balance_rail.height")?,
        balance_punching: PunchingStack::new(balance_papers, b.length("balance_punching.felt")?),
        front_rail_height: b.length("front_rail.height")?,
        front_punching: PunchingStack::new(front_papers, b.length("front_punching.felt")?),
        back_rail_height: b.length("back_rail.height")?,
        back_rail_cloth: b.length("back_rail.cloth")?,
        back_rail_cloth_edge_radius: b.length("back_rail.cloth_edge_radius")?,
        front_rail: b.length("front_rail.x")?,
        back_rail: b.length("back_rail.x")?,
        back_rail_from_the_tail: b.length("back_rail.from_the_tail")?,
        sections: FrameSections {
            balance: rail(b, &BALANCE_RAIL)?,
            front: rail(b, &FRONT_RAIL)?,
            back: rail(b, &BACK_RAIL)?,
        },
        drawer,
    };
    let bushings = if b.has("friction.level_weight_g") {
        let grams = b.number("friction.level_weight_g", Quantity::Grams)?;
        Bushings::LevelWeightAtMeasuringPoint { grams }
    } else {
        Bushings::Torque(AxisFriction {
            bound_n_m: b.number("friction.bound", Quantity::Torque)?,
            radius_m: b.optional("friction.radius", Quantity::Length)?,
        })
    };
    let friction_basis = b.basis(match bushings {
        Bushings::LevelWeightAtMeasuringPoint { .. } => "friction.level_weight_g",
        Bushings::Torque(_) => "friction.bound",
    });
    if let Bushings::LevelWeightAtMeasuringPoint { .. } = bushings {
        let friction = bushings.on(&key, &protocol);
        b.computed(
            "friction.bound",
            friction.bound_n_m,
            "the torque the level weight overcomes at the measuring point",
            Some("friction.level_weight_g"),
        );
        if let Some(radius) = friction.radius_m {
            b.computed(
                "friction.radius",
                radius,
                "the measuring point: the covering's lip less its distance back from it",
                Some("measuring_point_from_lip"),
            );
        }
    }
    // A key height no source gives is the one the key stands at as it
    // arrives, read once it is built: a key regulated on arrival is laid to
    // a target, and has to be given one.
    let standing = matches!(b.find("key_height")?.0.how, How::Absent);
    let key_height = if standing {
        if arrival == Arrival::Regulated {
            return Err(LoadError::Malformed {
                param: "key_height".to_owned(),
                why: "a key regulated on arrival is laid to a key height target",
            });
        }
        b.absent("key_height")?;
        f64::NAN
    } else {
        b.length("key_height")?
    };
    let targets = Targets {
        key_height,
        key_dip: b.length("key_dip")?,
    };
    let weight_g = b.number("weight_g", Quantity::Grams)?;
    let wippen_assembly = match choose(
        "wippen_assembly",
        file.wippen_assembly.as_ref(),
        &[("none", false), ("published", true)],
    )? {
        true => Some(published_wippen_assembly(b)?),
        false => None,
    };
    let mut start = Start {
        key,
        kind,
        mass,
        frame,
        protocol,
        bushings,
        friction_basis,
        targets,
        weight_g,
        arrival,
        wippen_assembly,
    };
    if standing {
        start.targets.key_height = standing_key_height(&start)?;
        b.computed(
            "key_height",
            start.targets.key_height,
            "the key height the key stands at on its frame as it arrives, read by the seated \
             pose: no source gives one, so the regulation keeps it",
            None,
        );
    }
    Ok((start, paper_colors))
}

/// A wippen assembly a source publishes, read off the bindings: its center
/// pin's place; its body's, its jack's and its repetition lever's outlines,
/// traced, each read whole off its record ([`outline`]); its heel by its
/// position, its height and its radius, its corners and its cloth; the jack
/// and the lever on their pins at their rest angles; the center pin, the
/// widths and the center's friction; and each of its three parts' mass,
/// center of mass and inertia, in its own frame.
fn published_wippen_assembly(b: &mut Bindings) -> Result<WippenAssemblyStart, LoadError> {
    let point = |b: &mut Bindings, x: &'static str, y: &'static str| {
        Ok::<_, LoadError>(Point::new(b.length(x)?, b.length(y)?))
    };
    let center = point(b, "wippen.center.x", "wippen.center.y")?;
    let body = outline(b, "wippen_outline")?;
    let heel = WippenHeel {
        position: b.length("wippen_heel.position")?,
        height: b.length("wippen_heel.height")?,
        radius: b.length("wippen_heel.radius")?,
        top: b.length("wippen_heel.top")?,
        back_top: b.length("wippen_heel.back_top")?,
        front_top: b.length("wippen_heel.front_top")?,
        back: b.length("wippen_heel.back")?,
        front: b.length("wippen_heel.front")?,
        cloth: HeelCloth {
            back: b.length("wippen_heel_cloth.back")?,
            front: b.length("wippen_heel_cloth.front")?,
            thickness: b.length("wippen_heel_cloth.thickness")?,
        },
    };
    let pinned = |b: &mut Bindings, names: [&'static str; 5]| {
        Ok::<_, LoadError>(PinnedPart {
            pin: point(b, names[0], names[1])?,
            rest_angle: b.number(names[2], Quantity::Angle)?,
            outline: outline(b, names[3])?,
            width: b.length(names[4])?,
        })
    };
    let jack = pinned(
        b,
        [
            "jack.pin.x",
            "jack.pin.y",
            "jack.rest_angle",
            "jack_outline",
            "jack.width",
        ],
    )?;
    let repetition_lever = pinned(
        b,
        [
            "repetition_lever.pin.x",
            "repetition_lever.pin.y",
            "repetition_lever.rest_angle",
            "repetition_lever_outline",
            "repetition_lever.width",
        ],
    )?;
    let center_pin_diameter = b.length("center_pin.diameter")?;
    let width = b.length("wippen.width")?;
    let friction = AxisFriction {
        bound_n_m: b.number("wippen.friction.bound", Quantity::Torque)?,
        radius_m: b.optional("wippen.friction.radius", Quantity::Length)?,
    };
    let part = |b: &mut Bindings, names: [&'static str; 4]| {
        Ok::<_, LoadError>(PartMass {
            mass: b.number(names[0], Quantity::Mass)?,
            center: Point::new(b.length(names[1])?, b.length(names[2])?),
            inertia: b.number(names[3], Quantity::Inertia)?,
        })
    };
    let mass = WippenAssemblyMass::Published {
        wippen: part(
            b,
            [
                "wippen.published.mass",
                "wippen.published.center.x",
                "wippen.published.center.y",
                "wippen.published.inertia",
            ],
        )?,
        jack: part(
            b,
            [
                "jack.published.mass",
                "jack.published.center.x",
                "jack.published.center.y",
                "jack.published.inertia",
            ],
        )?,
        repetition_lever: part(
            b,
            [
                "repetition_lever.published.mass",
                "repetition_lever.published.center.x",
                "repetition_lever.published.center.y",
                "repetition_lever.published.inertia",
            ],
        )?,
    };
    let bases = WippenAssemblyBases {
        mass: b.basis("wippen.published.mass"),
        friction: b.basis("wippen.friction.bound"),
    };
    Ok(WippenAssemblyStart {
        assembly: WippenAssembly {
            center,
            body,
            heel,
            jack,
            repetition_lever,
            center_pin_diameter,
            width,
            friction,
        },
        mass,
        bases,
    })
}

/// A traced outline bound to `param`, read whole off the record it points
/// to: an outline is traced from a source, so a record holds it, never a
/// profile's statement. The record's value is the outline's stretches in
/// order, each a line from its point, `{ "line": [x, y] }`, or an arc
/// counter-clockwise from its point round its center,
/// `{ "arc": [x, y], "center": [x, y], "radius": r }`, every number in the
/// record's unit; each number's provenance is the record, under `param`,
/// in the order the stretch gives its numbers ([`Stretch::numbers`]).
fn outline(b: &mut Bindings, param: &'static str) -> Result<Vec<Stretch>, LoadError> {
    let (binding, used) = b.find(param)?;
    *used = true;
    let malformed = |why| LoadError::Malformed {
        param: param.to_owned(),
        why,
    };
    let How::Record {
        file,
        pointer,
        derive: Derive::Whole,
    } = &binding.how
    else {
        return Err(malformed(
            "an outline is traced, and read whole from its record",
        ));
    };
    let (file, pointer) = (*file, pointer.clone());
    let records = Records::get();
    let at = records.at(file, &pointer)?;
    let (recorded, unit, reliability) = outline_record_of(at, file, &pointer)?;
    let decades = Quantity::Length
        .decades(unit)
        .ok_or_else(|| LoadError::WrongUnit {
            param,
            unit: unit.to_owned(),
        })?;
    let length = |x: &Value| {
        let x = x.as_f64().ok_or(LoadError::NotARecord {
            file,
            pointer: pointer.clone(),
        })?;
        shifted(x, decades).ok_or(LoadError::TooManyDigits { param })
    };
    let point = |v: Option<&Value>| match v.and_then(Value::as_array).map(Vec::as_slice) {
        Some([x, y]) => Ok(Point::new(length(x)?, length(y)?)),
        _ => Err(LoadError::NotARecord {
            file,
            pointer: pointer.clone(),
        }),
    };
    let mut stretches = Vec::with_capacity(recorded.len());
    for record in recorded {
        let stretch = match (record.get("line"), record.get("arc")) {
            (Some(at), None) => Stretch::Line(point(Some(at))?),
            (None, Some(from)) => Stretch::Arc {
                from: point(Some(from))?,
                center: point(record.get("center"))?,
                radius: length(record.get("radius").unwrap_or(&Value::Null))?,
            },
            _ => {
                return Err(LoadError::NotARecord {
                    file,
                    pointer: pointer.clone(),
                })
            }
        };
        for value in stretch.numbers() {
            b.provenance.push(Provenance {
                param,
                value,
                source: Source::Record {
                    file,
                    pointer: pointer.clone(),
                    reliability,
                    measured: records.measured(file),
                    derive: Derive::Whole,
                    traced: records.traced(file),
                    instrument: records.instrument(file),
                },
            });
        }
        stretches.push(stretch);
    }
    Ok(stretches)
}

/// An outline's record: its stretches, its unit and its marker.
fn outline_record_of<'v>(
    v: &'v Value,
    file: &'static str,
    pointer: &str,
) -> Result<(&'v [Value], &'v str, Reliability), LoadError> {
    let not_a_record = || LoadError::NotARecord {
        file,
        pointer: pointer.to_owned(),
    };
    let stretches = v
        .get("value")
        .and_then(Value::as_array)
        .ok_or_else(not_a_record)?;
    let unit = v
        .get("unit")
        .and_then(Value::as_str)
        .ok_or_else(not_a_record)?;
    let reliability = v
        .get("reliability")
        .and_then(Value::as_str)
        .and_then(Reliability::of)
        .ok_or_else(not_a_record)?;
    Ok((stretches, unit, reliability))
}

// ---- loading ----------------------------------------------------------------

/// `text`, a profile's file, read; its name checked against `name`.
fn parse(name: &str, text: &str) -> Result<ProfileFile, LoadError> {
    let file: ProfileFile = serde_json::from_str(text).map_err(|e| LoadError::Json {
        file: format!("data/profiles/{name}.json"),
        why: e.to_string(),
    })?;
    if file.profile != name {
        return Err(LoadError::Choice {
            what: "profile",
            given: file.profile,
        });
    }
    Ok(file)
}

/// A profile's file merged onto its bases', a binding of the profile's
/// replacing its base's of the same number: the choices and the bindings,
/// read.
fn merged(
    file: &ProfileFile,
    profiles: &dyn Fn(&str) -> Option<&'static str>,
    seen: &mut Vec<String>,
) -> Result<(ProfileFile, Vec<Binding>), LoadError> {
    if seen.contains(&file.profile) {
        return Err(LoadError::CircularBase(file.profile.clone()));
    }
    seen.push(file.profile.clone());
    let mut own = Vec::with_capacity(file.values.len());
    for b in &file.values {
        if own.iter().any(|o: &Binding| o.param == b.param) {
            return Err(LoadError::BoundTwice(b.param.clone()));
        }
        own.push(b.clone().read(&file.profile)?);
    }
    let Some(base_name) = &file.base else {
        return Ok((file.clone(), own));
    };
    let text = profiles(base_name).ok_or_else(|| LoadError::UnknownBase(base_name.clone()))?;
    let base_file = parse(base_name, text)?;
    let (base, mut bindings) = merged(&base_file, profiles, seen)?;
    for b in own {
        match bindings.iter_mut().find(|o| o.param == b.param) {
            Some(o) => *o = b,
            None => bindings.push(b),
        }
    }
    let choices = ProfileFile {
        profile: file.profile.clone(),
        what: file.what.clone(),
        base: file.base.clone(),
        instruments: file.instruments.clone().or(base.instruments),
        key_kind: file.key_kind.clone().or(base.key_kind),
        key_mass: file.key_mass.clone().or(base.key_mass),
        arrival: file.arrival.clone().or(base.arrival),
        wippen_assembly: file.wippen_assembly.clone().or(base.wippen_assembly),
        values: Vec::new(),
    };
    Ok((choices, bindings))
}

/// The shipped profile's file of that name.
fn shipped_text(name: &str) -> Option<&'static str> {
    SHIPPED.iter().find(|(n, _)| *n == name).map(|&(_, t)| t)
}

impl Profile {
    /// The profile `text` describes, named `name`, its base among the
    /// shipped profiles; or why it does not load.
    pub fn load(name: &str, text: &str) -> Result<Self, LoadError> {
        let file = parse(name, text)?;
        let (choices, bindings) = merged(&file, &shipped_text, &mut Vec::new())?;
        let instruments = choices.instruments.clone().ok_or(LoadError::Choice {
            what: "instruments",
            given: String::new(),
        })?;
        if let Some(unknown) = instruments.iter().find(|i| !Records::get().names(i)) {
            return Err(LoadError::UnknownInstrument(unknown.clone()));
        }
        let mut b = Bindings {
            profile: name.to_owned(),
            bindings: bindings.into_iter().map(|b| (b, false)).collect(),
            provenance: Vec::new(),
        };
        let (start, paper_colors) = walk(&mut b, &choices)?;
        b.finish()?;
        Ok(Self {
            name: b.profile,
            what: file.what,
            base: file.base,
            instruments,
            start,
            provenance: b.provenance,
            paper_colors,
        })
    }

    /// The names of the profiles shipped, in order.
    pub fn names() -> impl Iterator<Item = &'static str> {
        SHIPPED.iter().map(|&(n, _)| n)
    }

    /// The shipped profile of that name, loaded from its file; or why it
    /// does not load. Each call loads it again: [`Self::named`] keeps it.
    pub fn load_shipped(name: &str) -> Option<Result<Self, LoadError>> {
        shipped_text(name).map(|text| Self::load(name, text))
    }

    /// The shipped profile of that name, loaded once.
    ///
    /// # Panics
    ///
    /// Where a shipped profile does not load, which
    /// `every_default_has_a_record_or_a_stated_assumption` fails on first.
    pub fn named(name: &str) -> Option<&'static Self> {
        static LOADED: OnceLock<Vec<Profile>> = OnceLock::new();
        LOADED
            .get_or_init(|| {
                Self::names()
                    .map(|n| {
                        Self::load_shipped(n)
                            .expect("a shipped profile")
                            .unwrap_or_else(|e| panic!("data/profiles/{n}.json: {e:?}"))
                    })
                    .collect()
            })
            .iter()
            .find(|p| p.name == name)
    }

    /// The profile the standard instrument is built from.
    pub fn standard() -> &'static Self {
        Self::named(STANDARD).expect("the standard profile is shipped")
    }

    /// The owner's measured section weighed by the test fixture's weighing
    /// ([`pa_core::test_fixture::weighed`]), a profile on the shipped one:
    /// the key the domain's tests push, as an instrument, for the tests of
    /// what needs a key the touch weight moves either way: front-heavy by a
    /// fraction of a gram past its bushings, on its front punching under no
    /// weight, up onto its cloth under a pull of 13 g.
    #[cfg(test)]
    pub(crate) fn weighed_section() -> &'static Self {
        use pa_core::test_fixture as fixture;
        static WEIGHED: OnceLock<Profile> = OnceLock::new();
        WEIGHED.get_or_init(|| {
            let text = format!(
                r#"{{
                "profile": "{WEIGHED_SECTION}",
                "what": "the owner's measured section, weighed by the tests' weighing",
                "base": "{SECTION}",
                "key_mass": {{ "is": "weighed", "why": "the tests' weighing, not a measurement" }},
                "values": [
                    {{ "param": "weighing.mass_g", "value": {}, "unit": "g",
                      "reliability": "S", "why": "the tests' weighing" }},
                    {{ "param": "weighing.front_weight_g", "value": {}, "unit": "g",
                      "reliability": "S", "why": "the tests' weighing" }},
                    {{ "param": "weighing.from_lip", "value": {}, "unit": "mm",
                      "reliability": "S", "why": "the tests' weighing" }}
                ]
            }}"#,
                fixture::WEIGHED_MASS_G,
                fixture::WEIGHED_FRONT_WEIGHT_G,
                fixture::WEIGHED_FROM_LIP * 1000.0,
            );
            Self::load(WEIGHED_SECTION, &text).expect("the measured section, weighed")
        })
    }

    /// The color the drawer's paper of this thickness is supplied in, if
    /// the drawer holds one.
    pub fn paper_color(&self, thickness: f64) -> Option<PaperColor> {
        self.start
            .frame
            .drawer
            .thicknesses()
            .iter()
            .position(|t| t.to_bits() == thickness.to_bits())
            .and_then(|i| self.paper_colors.get(i).copied())
    }

    /// Where the number `param` of the start comes from, the first so
    /// named.
    pub fn provenance(&self, param: &str) -> Option<&Provenance> {
        self.provenance.iter().find(|p| p.param == param)
    }

    /// How the number `param` of the start is known, and the file that
    /// records or states it, relative to the repository's root; a number
    /// computed from one input is known as that input is. `None` for a
    /// number the profile does not bind.
    pub fn known(&self, param: &str) -> Option<(Known, String)> {
        self.provenance(param).map(|p| self.known_from(&p.source))
    }

    /// Whether `source` is a record of an instrument this profile does not
    /// describe.
    pub fn borrows(&self, source: &Source) -> bool {
        matches!(source, Source::Record { instrument: Some(i), .. }
            if !self.instruments.iter().any(|own| own == i))
    }

    /// How a number of the start whose provenance is `source` is known, and
    /// the file that records or states it; a number computed from one input
    /// is known as that input is.
    pub fn known_from(&self, source: &Source) -> (Known, String) {
        match source {
            Source::Record {
                file,
                reliability,
                measured,
                traced,
                ..
            } => (
                match (reliability, measured, traced) {
                    (Reliability::Assumed, ..) => Known::Assumed,
                    _ if self.borrows(source) => Known::Borrowed,
                    (Reliability::Derived, _, true) => Known::Traced,
                    (Reliability::Derived, _, false) => Known::Derived,
                    (Reliability::Verified, true, _) => Known::Measured,
                    (Reliability::Verified, false, _) => Known::Published,
                },
                format!("data/params/{file}"),
            ),
            Source::Stated {
                profile,
                reliability,
                ..
            } => (
                match reliability {
                    Reliability::Assumed => Known::Assumed,
                    Reliability::Derived | Reliability::Verified => Known::Derived,
                },
                format!("data/profiles/{profile}.json"),
            ),
            Source::Computed { from, .. } => from
                .and_then(|input| self.known(input))
                .unwrap_or_else(|| (Known::Derived, format!("data/profiles/{}.json", self.name))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pa_core::validate::{numbers, Design};

    /// The design `start` describes, as [`numbers`] reads it.
    fn design_numbers(start: &Start) -> Vec<(&'static str, f64)> {
        let friction = start.bushings.on(&start.key, &start.protocol);
        numbers(&Design {
            key: &start.key,
            mass: &start.mass,
            frame: &start.frame,
            protocol: &start.protocol,
            friction: &friction,
            wippen_assembly: start
                .wippen_assembly
                .as_ref()
                .map(|w| (&w.assembly, &w.mass)),
        })
    }

    /// Consistency, `.claude/rules/provenance.md` > No number without provenance made mechanical: **every number an instrument
    /// starts from has a record or a stated assumption.** For each shipped
    /// profile, every number of the design it builds, as
    /// [`pa_core::validate::numbers`] reads it with every field of every
    /// part destructured, and the targets and the weight on the key, has a
    /// provenance of its own name carrying its very bits, list by list for
    /// the drawer's papers; a record's provenance names a record that holds
    /// a value, a unit of the number's quantity and a marker, and a stated
    /// one gives its reason, with its rule where it is derived. The load
    /// itself refuses a number left unbound and a binding no number reads.
    #[test]
    fn every_default_has_a_record_or_a_stated_assumption() {
        let mut names = 0;
        for name in Profile::names() {
            names += 1;
            let profile = match Profile::load_shipped(name).expect("shipped") {
                Ok(p) => p,
                Err(e) => panic!("data/profiles/{name}.json does not load: {e:?}"),
            };
            every_number_has_its_provenance(&profile);
        }
        assert!(names >= 1);
    }

    /// Every number of the design `profile` builds, and its targets and the
    /// weight on the key, has a provenance of its own name carrying its very
    /// bits, list by list, outline by outline; a record's names a record
    /// holding a value, or an outline's stretches, a unit of the number's
    /// quantity and a marker; a stated one gives its reason, with its rule
    /// where it is derived; and nothing computed is left unread.
    fn every_number_has_its_provenance(profile: &Profile) {
        let name = &profile.name;
        let start = &profile.start;
        let mut wanted = design_numbers(start);
        wanted.extend([
            ("key_height", start.targets.key_height),
            ("key_dip", start.targets.key_dip),
            ("weight_g", start.weight_g),
        ]);
        let mut bound: Vec<&Provenance> = profile.provenance.iter().collect();
        for (param, value) in wanted {
            let i = bound
                .iter()
                .position(|p| p.param == param)
                .unwrap_or_else(|| panic!("{name}: {param} has no provenance"));
            let p = bound.remove(i);
            assert_eq!(
                p.value.to_bits(),
                value.to_bits(),
                "{name}: {param} is {value}, its provenance says {}",
                p.value
            );
            match &p.source {
                Source::Record { file, pointer, .. } => {
                    let at = Records::get().at(file, pointer).expect("a record");
                    assert!(
                        record_of(at, file, pointer).is_ok()
                            || outline_record_of(at, file, pointer).is_ok(),
                        "{name}: {param} reads no value, unit and marker at {file}{pointer}"
                    );
                }
                Source::Stated {
                    reliability,
                    rule,
                    why,
                    ..
                } => {
                    assert!(!why.trim().is_empty(), "{name}: {param} says not why");
                    assert!(
                        *reliability == Reliability::Assumed || rule.is_some(),
                        "{name}: {param} is derived by no rule"
                    );
                }
                Source::Computed { rule, .. } => assert!(!rule.is_empty()),
            }
        }
        // What is left are the inputs the rules read, each bound.
        for p in bound {
            assert!(
                !matches!(p.source, Source::Computed { .. }),
                "{name}: {} is computed and read by nothing",
                p.param
            );
        }
    }

    /// Hirschkorn's key with his wippen assembly bound from his records, as
    /// a profile on his: the reader of a published wippen exercised on the
    /// records it reads.
    fn his_key_carrying_his_wippen() -> Profile {
        const GP178: &str = "hirschkorn-boston-gp178-note52.json";
        const TRACE: &str = "hirschkorn-wippen-outline.json";
        let record = |param: &str, file: &str, pointer: &str| {
            format!(r#"{{ "param": "{param}", "record": "{file}", "pointer": "{pointer}" }}"#)
        };
        let mut values =
            vec![
            record("wippen.center.x", GP178, "/geometry/ground/points/Pg2/x"),
            r#"{ "param": "wippen.center.y", "value": 81.6, "unit": "mm", "reliability": "E",
                 "rule": "Pg2's y, 52.8 mm, raised by the bearing his key borrows, 28.8 mm",
                 "why": "his center pin over the keybed the profile borrows" }"#
                .to_owned(),
            record("wippen_outline", TRACE, "/records/wippen_outline"),
            record("wippen_heel.position", GP178, "/geometry/wippen/points/Pw5/x"),
            r#"{ "param": "wippen_heel.height", "value": 18.8, "unit": "mm", "reliability": "E",
                 "rule": "Pw2's y less Pw5's y plus Pw5's radius, -3.5 - 3.0 + 25.3",
                 "why": "the heel's lowest point below the jack's pin" }"#
                .to_owned(),
            record("wippen_heel.radius", GP178, "/geometry/wippen/points/Pw5/radius"),
            record("wippen_heel.top", TRACE, "/records/heel_top"),
            record("wippen_heel.back_top", TRACE, "/records/heel_back_top"),
            record("wippen_heel.front_top", TRACE, "/records/heel_front_top"),
            record("wippen_heel.back", TRACE, "/records/heel_back"),
            record("wippen_heel.front", TRACE, "/records/heel_front"),
            record("wippen_heel_cloth.back", TRACE, "/records/heel_cloth_back"),
            record("wippen_heel_cloth.front", TRACE, "/records/heel_cloth_front"),
            record(
                "wippen_heel_cloth.thickness",
                TRACE,
                "/records/heel_cloth_thickness",
            ),
            record("jack.pin.x", GP178, "/geometry/wippen/points/Pw2/x"),
            record("jack.pin.y", GP178, "/geometry/wippen/points/Pw2/y"),
            record("jack.rest_angle", TRACE, "/records/jack_rest_angle"),
            record("jack_outline", TRACE, "/records/jack_outline"),
            record("jack.width", TRACE, "/records/jack_width"),
            record("repetition_lever.pin.x", GP178, "/geometry/wippen/points/Pw3/x"),
            record("repetition_lever.pin.y", GP178, "/geometry/wippen/points/Pw3/y"),
            record(
                "repetition_lever.rest_angle",
                TRACE,
                "/records/repetition_lever_rest_angle",
            ),
            record(
                "repetition_lever_outline",
                TRACE,
                "/records/repetition_lever_outline",
            ),
            record(
                "repetition_lever.width",
                TRACE,
                "/records/repetition_lever_width",
            ),
            record("center_pin.diameter", TRACE, "/records/center_pin_diameter"),
            record("wippen.width", TRACE, "/records/wippen_width"),
            record("wippen.friction.bound", GP178, "/joint_friction/rows/wippen/A"),
            r#"{ "param": "wippen.friction.radius", "none": true,
                 "why": "his Table A.9 prints the torque alone" }"#
                .to_owned(),
        ];
        for part in ["wippen", "jack", "repetition_lever"] {
            let at = format!("/mass_properties/bodies/{part}");
            for (name, pointer) in [
                ("mass", "mass"),
                ("center.x", "center_of_mass/x"),
                ("center.y", "center_of_mass/y"),
                ("inertia", "moment_of_inertia"),
            ] {
                values.push(record(
                    &format!("{part}.published.{name}"),
                    GP178,
                    &format!("{at}/{pointer}"),
                ));
            }
        }
        let text = format!(
            r#"{{
                "profile": "his-key-carrying-his-wippen",
                "what": "his key carrying his wippen assembly, bound from his records",
                "base": "{STANDARD}",
                "wippen_assembly": {{ "is": "published", "why": "his tables and his figure" }},
                "values": [{}]
            }}"#,
            values.join(",")
        );
        Profile::load("his-key-carrying-his-wippen", &text)
            .unwrap_or_else(|e| panic!("his wippen does not load: {e:?}"))
    }

    /// Consistency: **a profile binds a published wippen assembly from its
    /// records**, every number of it with its provenance: his heel by Pw5's
    /// position and radius and its height by its rule, 18.8 mm, its circle's
    /// lowest point standing at Pw5's height less its radius in the wippen's
    /// frame; his outlines read whole off their trace, every stretch of
    /// them; his three parts' masses from Table A.1, his center's friction
    /// from Table A.9, both published; and the design they make is one the
    /// validation layer takes.
    #[test]
    fn a_profile_binds_a_published_wippen_assembly_from_its_records() {
        let profile = his_key_carrying_his_wippen();
        every_number_has_its_provenance(&profile);
        let wippen = profile
            .start
            .wippen_assembly
            .as_ref()
            .expect("his profile carries his wippen");
        let heel = wippen.assembly.heel;
        assert_eq!(heel.position.to_bits(), 0.0595_f64.to_bits());
        assert_eq!(heel.radius.to_bits(), 0.0253_f64.to_bits());
        assert_eq!(heel.height.to_bits(), 0.0188_f64.to_bits());
        let center = heel.center(wippen.assembly.jack.pin);
        // Pw5 stands 3.0 mm above the wippen's axis; the height's rule adds
        // and subtracts printed tenths of a millimeter, each a few units in
        // the last place of a hundredth of a meter.
        assert!(
            (center.y - 0.003).abs() <= 1e-15 && center.x.to_bits() == 0.0595_f64.to_bits(),
            "the heel's circle stands at Pw5 within its height's roundings: {center:?}"
        );
        let trace = Records::get().json("hirschkorn-wippen-outline.json");
        let stretches = |name: &str| {
            trace["records"]
                .as_array()
                .and_then(|r| r.iter().find(|r| r["name"] == name))
                .and_then(|r| r["value"].as_array())
                .map_or(0, Vec::len)
        };
        assert_eq!(wippen.assembly.body.len(), stretches("wippen_outline"));
        assert_eq!(
            wippen.assembly.jack.outline.len(),
            stretches("jack_outline")
        );
        assert_eq!(
            wippen.assembly.repetition_lever.outline.len(),
            stretches("repetition_lever_outline")
        );
        assert_eq!(
            wippen.bases,
            WippenAssemblyBases {
                mass: Basis::Published,
                friction: Basis::Published,
            }
        );
        let friction = profile
            .start
            .bushings
            .on(&profile.start.key, &profile.start.protocol);
        pa_core::validate::validate(&Design {
            key: &profile.start.key,
            mass: &profile.start.mass,
            frame: &profile.start.frame,
            protocol: &profile.start.protocol,
            friction: &friction,
            wippen_assembly: Some((&wippen.assembly, &wippen.mass)),
        })
        .expect("his wippen on his key is a design");
    }

    /// Consistency: **an outline is read from a record, never stated**: a
    /// profile that states one does not load, naming it.
    #[test]
    fn an_outline_stated_and_not_recorded_does_not_load() {
        let text = format!(
            r#"{{
                "profile": "a-stated-outline",
                "what": "a wippen whose body outline is stated",
                "base": "{STANDARD}",
                "wippen_assembly": {{ "is": "published", "why": "a test" }},
                "values": [
                    {{ "param": "wippen.center.x", "value": -181.0, "unit": "mm",
                      "reliability": "S", "why": "a test" }},
                    {{ "param": "wippen.center.y", "value": 81.6, "unit": "mm",
                      "reliability": "S", "why": "a test" }},
                    {{ "param": "wippen_outline", "values": [0.0, 1.0], "unit": "mm",
                      "reliability": "S", "why": "a test" }}
                ]
            }}"#
        );
        assert_eq!(
            Profile::load("a-stated-outline", &text),
            Err(LoadError::Malformed {
                param: "wippen_outline".to_owned(),
                why: "an outline is traced, and read whole from its record",
            })
        );
    }

    /// Consistency: **a number left unbound, or bound and read by nothing,
    /// does not load**, naming it.
    #[test]
    fn a_profile_missing_a_number_or_binding_a_stray_one_does_not_load() {
        let text = shipped_text(SECTION).unwrap();
        let mut file: Value = serde_json::from_str(text).unwrap();
        let values = file["values"].as_array_mut().unwrap();
        let i = values
            .iter()
            .position(|v| v["param"] == "tail_rise_run")
            .unwrap();
        let removed = values.remove(i);
        let missing = serde_json::to_string(&file).unwrap();
        assert_eq!(
            Profile::load(SECTION, &missing),
            Err(LoadError::Unbound("tail_rise_run"))
        );
        let mut stray = removed.clone();
        stray["param"] = Value::from("tail_rise_runn");
        let values = file["values"].as_array_mut().unwrap();
        values.push(removed);
        values.push(stray);
        assert_eq!(
            Profile::load(SECTION, &serde_json::to_string(&file).unwrap()),
            Err(LoadError::Unused("tail_rise_runn".to_owned()))
        );
    }

    /// Consistency: **a record is read in the unit of its number's
    /// quantity, or not at all**: a length bound to a record in grams is
    /// refused, and so are a stated value without its reason and an absence
    /// stated with a blank one.
    #[test]
    fn a_unit_that_is_not_the_quantitys_does_not_load() {
        let text = shipped_text(SECTION).unwrap();
        let mut file: Value = serde_json::from_str(text).unwrap();
        for v in file["values"].as_array_mut().unwrap() {
            if v["param"] == "stick_height" {
                v["pointer"] = Value::from("/records/key_weight_ratio_load");
                v["record"] = Value::from("touchweight-protocol.json");
            }
        }
        assert_eq!(
            Profile::load(SECTION, &serde_json::to_string(&file).unwrap()),
            Err(LoadError::WrongUnit {
                param: "stick_height",
                unit: "g".to_owned()
            })
        );
        let mut file: Value = serde_json::from_str(text).unwrap();
        for v in file["values"].as_array_mut().unwrap() {
            if v["param"] == "front_edge" {
                v.as_object_mut().unwrap().remove("why");
            }
        }
        assert_eq!(
            Profile::load(SECTION, &serde_json::to_string(&file).unwrap()),
            Err(LoadError::Unexplained("front_edge".to_owned()))
        );
        let mut file: Value = serde_json::from_str(text).unwrap();
        for v in file["values"].as_array_mut().unwrap() {
            if v["param"] == "top_clearance" {
                v["why"] = Value::from("  ");
            }
        }
        assert_eq!(
            Profile::load(SECTION, &serde_json::to_string(&file).unwrap()),
            Err(LoadError::Unexplained("top_clearance".to_owned())),
            "an absence stated with a blank reason"
        );
    }

    /// Consistency: **a unit is converted rounding once from the recorded
    /// decimal**: every millimeter a drawer or a section is recorded in
    /// reads as the Rust literal in meters reads, to the bit, where
    /// dividing the double by a thousand rounds twice; and a value with
    /// more digits than a double holds is refused rather than rounded
    /// twice.
    #[test]
    fn a_unit_is_converted_rounding_once_from_the_decimal() {
        for (mm, m) in [
            (0.08_f64, 0.000_08_f64),
            (0.12, 0.000_12),
            (0.75, 0.000_75),
            (1.25, 0.001_25),
            (24.5, 0.024_5),
            (3.4, 0.003_4),
            (0.8, 0.000_8),
            (258.0, 0.258),
            (-129.5, -0.129_5),
            (10.2, 0.010_2),
            (26.6, 0.026_6),
            (11.2, 0.011_2),
        ] {
            assert_eq!(
                shifted(mm, -3).map(f64::to_bits),
                Some(m.to_bits()),
                "{mm} mm"
            );
        }
        // Where dividing rounds twice: the double nearest the drawer's
        // 0.12 mm pastel green, divided by a thousand, is not the double
        // nearest 0.00012 m.
        let twice = 0.12_f64 / 1000.0;
        assert_ne!(twice.to_bits(), 0.000_12_f64.to_bits());
        assert_eq!(shifted(0.12, -3), Some(0.000_12));
        assert_eq!(shifted(0.1 + 0.2, -3), None, "17 digits");
    }

    /// Consistency: **a profile names a base and binds what differs**: a
    /// profile on the measured section with its capstan moved and its key
    /// weighed loads with the section's every other number, its own two with
    /// the profile's provenance, and the weighing's derived numbers computed
    /// as a weighing computes them.
    #[test]
    fn a_profile_on_a_base_binds_only_what_differs() {
        let text = r#"{
            "profile": "moved",
            "what": "the measured section with its capstan 2 mm further back, weighed",
            "base": "measured-section",
            "key_mass": { "is": "weighed", "why": "weighed on a scale and on the jig" },
            "values": [
                { "param": "capstan.x", "value": -131.5, "unit": "mm",
                  "reliability": "S", "why": "a test" },
                { "param": "weighing.mass_g", "value": 121.1, "unit": "g",
                  "reliability": "S", "why": "a test" },
                { "param": "weighing.front_weight_g", "value": 6.5, "unit": "g",
                  "reliability": "S", "why": "a test" },
                { "param": "weighing.from_lip", "value": 13.0, "unit": "mm",
                  "reliability": "S", "why": "a test" }
            ]
        }"#;
        let moved = Profile::load("moved", text).expect("a profile on a base");
        let base = Profile::named(SECTION).expect("shipped");
        assert_eq!(moved.start.key.capstan.to_bits(), (-0.1315_f64).to_bits());
        let mut key = base.start.key.clone();
        key.capstan = moved.start.key.capstan;
        assert_eq!(moved.start.key, key);
        assert_eq!(moved.start.frame, base.start.frame);
        assert_eq!(
            moved.start.mass,
            KeyMass::weighed(&key, 121.1, 6.5, 0.013).unwrap()
        );
        assert!(matches!(
            &moved.provenance("capstan.x").unwrap().source,
            Source::Stated { profile, .. } if profile == "moved"
        ));
        assert!(matches!(
            &moved.provenance("stick_height").unwrap().source,
            Source::Record {
                file: "key-section.json",
                ..
            }
        ));
        assert!(Profile::load(
            "loop",
            r#"{"profile": "loop", "what": "", "base": "loop", "values": []}"#
        )
        .is_err());
    }

    /// Consistency: **the measured section's profile loads to the test
    /// fixture's bits**, so the key and frame `pa-core`'s tests are run on
    /// and the instrument the bench builds cannot drift apart: every number
    /// of the design in the same order to the bit, the papers aside, which
    /// arrive laid by the regulation; the kind, the mass, which none of its
    /// keys has until one is weighed, the bushings, the targets and the
    /// weight; and the two instruments built, regulated alike, hash alike.
    /// The fixture's test weighing is the one the tests' weighed section
    /// takes, to the bit of its static moment, and so is the instrument it
    /// builds. The fixture's half inch is the record's too.
    #[test]
    fn the_measured_section_profile_loads_to_the_test_fixture() {
        use crate::instrument::Instrument;
        use pa_core::test_fixture as fixture;
        let profile = Profile::named(SECTION).expect("shipped");
        assert_eq!(profile.name, "measured-section");
        let start = &profile.start;
        let mut frame = fixture::frame();
        frame.balance_punching.papers.clear();
        frame.front_punching.papers.clear();
        let bushings = Bushings::LevelWeightAtMeasuringPoint {
            grams: fixture::KEY_FRICTION_WEIGHT_LEVEL_G,
        };
        let fixed = Start {
            key: fixture::key(),
            kind: KeyKind::Natural,
            mass: KeyMass::Unweighed,
            frame,
            protocol: fixture::protocol(),
            bushings,
            friction_basis: Basis::Assumed,
            targets: fixture::TARGETS,
            weight_g: 0.0,
            arrival: Arrival::Regulated,
            wippen_assembly: None,
        };
        let bits = |s: &Start| -> Vec<(&'static str, u64)> {
            let mut n = design_numbers(s);
            n.extend([
                ("key_height", s.targets.key_height),
                ("key_dip", s.targets.key_dip),
                ("weight_g", s.weight_g),
            ]);
            n.into_iter().map(|(name, x)| (name, x.to_bits())).collect()
        };
        assert_eq!(bits(start), bits(&fixed));
        assert_eq!(*start, fixed);

        let built = Instrument::new(
            Start {
                frame: fixture::frame(),
                ..fixed.clone()
            },
            profile,
        )
        .expect("the fixture is regulable");
        assert_eq!(
            built.state_hash(),
            Instrument::of(profile).expect("regulable").state_hash()
        );

        let weighed = Profile::weighed_section();
        let with_weighing = Start {
            mass: fixture::weighed(),
            ..fixed
        };
        assert_eq!(bits(&weighed.start), bits(&with_weighing));
        assert_eq!(weighed.start, with_weighing);
        let built = Instrument::new(
            Start {
                frame: fixture::frame(),
                ..with_weighing
            },
            weighed,
        )
        .expect("the fixture is regulable");
        assert_eq!(
            built.state_hash(),
            Instrument::weighed_section().state_hash()
        );

        let imperial = Records::get()
            .at(
                "touchweight-protocol.json",
                "/records/measuring_point_from_lip_imperial",
            )
            .expect("the half inch");
        let (mm, unit, _) = record_of(imperial, "touchweight-protocol.json", "").expect("a record");
        assert_eq!(unit, "mm");
        assert_eq!(
            shifted(mm, -3).map(f64::to_bits),
            Some(fixture::MEASURING_POINT_HALF_INCH.to_bits())
        );
    }

    /// Consistency: **an instrument resets to its own profile**, not the
    /// standard one: one built on a profile with its capstan moved, edited,
    /// then reset, is that profile's again, to the bit of its state hash,
    /// and not the standard instrument.
    #[test]
    fn an_instrument_resets_to_its_own_profile() {
        use crate::instrument::{Action, Instrument};
        use crate::registry::path;
        let text = r#"{
            "profile": "moved",
            "what": "the measured section with its capstan 2 mm further back",
            "base": "measured-section",
            "values": [
                { "param": "capstan.x", "value": -131.5, "unit": "mm",
                  "reliability": "S", "why": "a test" }
            ]
        }"#;
        let moved: &'static Profile =
            Box::leak(Box::new(Profile::load("moved", text).expect("a profile")));
        let fresh = Instrument::of(moved).expect("regulable");
        assert_eq!(fresh.profile().name, "moved");
        let mut b = fresh.clone();
        b.set(path::FRONT_EDGE, 250.0).expect("a shorter key");
        b.set(path::WEIGHT, 30.0).expect("a weight");
        assert_ne!(b.state_hash(), fresh.state_hash());
        b.act(Action::Reset).expect("reset");
        assert_eq!(b.state_hash(), fresh.state_hash());
        assert_ne!(b.state_hash(), Instrument::standard().state_hash());
        assert_eq!(b.key().capstan.to_bits(), (-0.1315_f64).to_bits());
    }

    /// Consistency: **a profile that arrives as recorded keeps its papers**:
    /// the measured section with half a millimeter of paper on each punching, as
    /// recorded, is built with those papers and no regulation, the papers'
    /// provenance under the name the design's numbers give a paper; and one
    /// recording a paper no drawer holds does not load.
    #[test]
    fn a_profile_arriving_as_recorded_keeps_its_papers() {
        use crate::instrument::{Instrument, Stack};
        let recorded = |paper_mm: f64| {
            format!(
                r#"{{
                "profile": "recorded",
                "what": "the measured section with its papers as recorded",
                "base": "measured-section",
                "arrival": {{ "is": "as_recorded", "why": "a test" }},
                "values": [
                    {{ "param": "balance_punching.papers", "values": [{paper_mm}],
                      "unit": "mm", "reliability": "S", "why": "a test" }},
                    {{ "param": "front_punching.papers", "values": [0.5],
                      "unit": "mm", "reliability": "S", "why": "a test" }}
                ]
            }}"#
            )
        };
        let profile: &'static Profile = Box::leak(Box::new(
            Profile::load("recorded", &recorded(0.5)).expect("a profile"),
        ));
        assert_eq!(profile.start.arrival, Arrival::AsRecorded);
        let b = Instrument::of(profile).expect("built as recorded");
        assert_eq!(b.papers(Stack::Balance), &[0.0005]);
        assert_eq!(b.papers(Stack::Front), &[0.0005]);
        assert_eq!(
            profile
                .provenance
                .iter()
                .filter(|p| p.param == PAPER.name)
                .count(),
            2
        );
        assert_eq!(
            Profile::load("recorded", &recorded(0.3)),
            Err(LoadError::PaperNotInTheDrawer(0.0003))
        );
    }

    /// Consistency: **a distance recorded behind the balance pin is the
    /// position the key's frame signs**: the measured section's capstan,
    /// recorded as 122 mm behind the pin, stands at −122 mm, exactly, the
    /// record's own provenance kept; and a derivation no loader knows does
    /// not load.
    #[test]
    fn a_distance_behind_the_pin_is_read_as_a_position() {
        let section = Profile::named(SECTION).expect("shipped");
        assert_eq!(section.start.key.capstan.to_bits(), (-0.122_f64).to_bits());
        assert!(matches!(
            &section.provenance("capstan.x").expect("bound").source,
            Source::Record {
                file: "key-section.json",
                derive: Derive::Behind,
                ..
            }
        ));
        let mut file: Value = serde_json::from_str(shipped_text(SECTION).unwrap()).unwrap();
        for v in file["values"].as_array_mut().unwrap() {
            if v["param"] == "capstan.x" {
                v["derive"] = Value::from("ahead");
            }
        }
        assert!(matches!(
            Profile::load(SECTION, &serde_json::to_string(&file).unwrap()),
            Err(LoadError::Malformed { param, .. }) if param == "capstan.x"
        ));
    }

    /// Consistency: **the drawer is the record's, each paper with its
    /// color**, thinnest first.
    #[test]
    fn the_drawer_and_its_colors_are_the_records() {
        let p = Profile::standard();
        let drawer = p.start.frame.drawer.thicknesses();
        assert_eq!(drawer.len(), p.paper_colors.len());
        assert_eq!(p.paper_color(0.0002), Some(PaperColor::SkyBlue));
        assert_eq!(p.paper_color(0.00008), Some(PaperColor::White));
        assert_eq!(p.paper_color(0.00125), Some(PaperColor::Card));
        assert_eq!(p.paper_color(0.0003), None);
    }

    /// Consistency: **his keystick stands at its full height to where his
    /// figure lowers its top, and his capstan screws into it.** The thinned
    /// tail's front, his key's end plus the length his profile derives by
    /// the rule stated beside it, is where the trace records the full height
    /// ending, -132.45 mm, within a rounding of each; the capstan's seat is
    /// the keystick's full height, and its top is Pk3's height plus its
    /// radius to the bit, the seat subtracted and added back exactly, the
    /// two within a factor of two of each other; his top has no clearance
    /// cut in it.
    #[test]
    fn his_keystick_stands_at_full_height_to_where_his_figure_lowers_it() {
        let p = Profile::named("hirschkorn").expect("shipped");
        let key = &p.start.key;
        let file = "hirschkorn-key-outline.json";
        let pointer = "/records/full_height_back";
        let at = Records::get().at(file, pointer).expect("a record");
        let (value, unit, _) = record_of(at, file, pointer).expect("a record");
        assert_eq!(unit, "mm");
        let full_height_back = shifted(value, -3).expect("a length");
        assert!(
            (key.tail_front() - full_height_back).abs() <= 2.0 * f64::EPSILON * 0.25,
            "the tail's front at {} m, the full height's end at {full_height_back} m",
            key.tail_front()
        );
        assert_eq!(p.known("tail_length").map(|(k, _)| k), Some(Known::Derived));
        assert_eq!(key.top_clearance, None);
        assert_eq!(key.capstan_seat().to_bits(), key.stick_height.to_bits());
        let printed = |pointer: &str| -> f64 {
            Records::get()
                .at("hirschkorn-boston-gp178-note52.json", pointer)
                .expect(pointer)["value"]
                .as_f64()
                .expect("a value")
        };
        let top =
            printed("/geometry/key/points/Pk3/y") + printed("/geometry/key/points/Pk3/radius");
        assert_eq!(key.capstan_contact().y.to_bits(), top.to_bits());
        assert!(key.capstan - 0.5 * key.capstan_screw.head_diameter > key.tail_front());
    }

    /// Consistency: **a clearance a profile binds is loaded as bound, from
    /// the balance pin, and drawn known as it is stated.** The measured
    /// section with its stated absence of a clearance replaced by one from
    /// 175 to 140 mm behind the pin, 3 mm deep, each number stated with its
    /// reason: the key reads those numbers in meters as the Rust literals
    /// read them, to the bit; the instrument builds on it, and its keystick
    /// draws the clearance's front wall, floor and back wall, each placed by
    /// its own number and known as assumed, from the profile that states it.
    /// Bound without its front end, the profile does not load, naming it.
    #[test]
    fn a_clearance_a_profile_binds_is_loaded_and_drawn_as_bound() {
        use crate::boundary::{How, Part, Scene};
        use crate::instrument::Instrument;
        let text = shipped_text(SECTION).unwrap();
        let mut file: Value = serde_json::from_str(text).unwrap();
        let values = file["values"].as_array_mut().unwrap();
        let i = values
            .iter()
            .position(|v| v["param"] == "top_clearance")
            .unwrap();
        values.remove(i);
        for (param, value) in [
            ("top_clearance.back", -175.0),
            ("top_clearance.front", -140.0),
            ("top_clearance.depth", 3.0),
        ] {
            values.push(serde_json::json!({
                "param": param, "value": value, "unit": "mm",
                "reliability": "S", "why": "a test's clearance"
            }));
        }
        let p: &'static Profile = Box::leak(Box::new(
            Profile::load(SECTION, &serde_json::to_string(&file).unwrap()).expect("it loads"),
        ));
        let c = p.start.key.top_clearance.expect("a clearance");
        for (got, want) in [(c.back, -0.175_f64), (c.front, -0.140), (c.depth, 0.003)] {
            assert_eq!(got.to_bits(), want.to_bits(), "{got} m, not {want} m");
        }
        let b = Instrument::of(p).expect("the instrument builds");
        let stick = Scene::of(&b)
            .outlines
            .into_iter()
            .find(|o| o.code == Part::Keystick)
            .expect("a keystick");
        for number in [
            "top_clearance.front",
            "top_clearance.depth",
            "top_clearance.back",
        ] {
            let source = stick
                .sources
                .iter()
                .find(|s| s.number == number)
                .unwrap_or_else(|| panic!("no stretch placed by {number}"));
            assert_eq!(source.how, How::Assumed, "{number}");
            assert_eq!(
                source.at.as_deref(),
                Some("data/profiles/measured-section.json")
            );
        }

        let values = file["values"].as_array_mut().unwrap();
        values.retain(|v| v["param"] != "top_clearance.front");
        assert_eq!(
            Profile::load(SECTION, &serde_json::to_string(&file).unwrap()),
            Err(LoadError::Unbound("top_clearance.front"))
        );
    }

    /// Consistency: **his profile reads his printed lines and circles as
    /// printed.** The chamfer drawn from Pk5's line passes through Pk5 and
    /// meets the flat where the two printed lines cross; the capstan's axis
    /// is Pk3's, its dome Pk3's radius, its top Pk3's height plus that
    /// radius; his mass, center of mass and inertia are Table A.1's, his
    /// bushings Table A.9's torque, read at no radius; his key height is
    /// computed, no source giving one. How each is known is what its record
    /// says: published, traced, assumed or derived.
    #[test]
    fn his_profile_reads_his_printed_lines_and_circles() {
        let p = Profile::named("hirschkorn").expect("shipped");
        let key = &p.start.key;
        let at = |pointer: &str| -> f64 {
            let records = Records::get();
            let v = records
                .at("hirschkorn-boston-gp178-note52.json", pointer)
                .expect(pointer);
            v["value"].as_f64().expect("a value")
        };
        let pk5 = Point::new(
            at("/geometry/key/points/Pk5/x"),
            at("/geometry/key/points/Pk5/y"),
        );
        let (nx, ny) = (
            at("/geometry/key/points/Pk5/normal/x"),
            at("/geometry/key/points/Pk5/normal/y"),
        );
        let kink = key.tail_end + key.tail_rise_run;
        let along = |x: f64| key.tail_rise * (kink - x) / key.tail_rise_run;
        assert!(
            (along(pk5.x) - pk5.y).abs() <= 1e-15,
            "the chamfer through Pk5"
        );
        assert!((kink - (pk5.x + ny * pk5.y / nx)).abs() <= 1e-15);
        assert_eq!(
            key.capstan.to_bits(),
            at("/geometry/key/points/Pk3/x").to_bits()
        );
        let radius = at("/geometry/key/points/Pk3/radius");
        assert_eq!(key.capstan_screw.dome_radius, Some(radius));
        let top = at("/geometry/key/points/Pk3/y") + radius;
        assert!((key.capstan_contact().y - top).abs() <= 2.0 * f64::EPSILON * top);
        let KeyMass::Published {
            mass,
            center,
            inertia,
        } = p.start.mass
        else {
            panic!("{:?}", p.start.mass)
        };
        let body = "/mass_properties/bodies/key";
        assert_eq!(mass.to_bits(), at(&format!("{body}/mass")).to_bits());
        assert_eq!(
            center,
            Point::new(
                at(&format!("{body}/center_of_mass/x")),
                at(&format!("{body}/center_of_mass/y"))
            )
        );
        assert_eq!(
            inertia.to_bits(),
            at(&format!("{body}/moment_of_inertia")).to_bits()
        );
        assert_eq!(
            p.start.bushings,
            Bushings::Torque(AxisFriction {
                bound_n_m: at("/joint_friction/rows/key/A"),
                radius_m: None,
            })
        );
        assert_eq!(p.start.friction_basis, Basis::Published);
        assert!(matches!(
            p.provenance("key_height").map(|p| &p.source),
            Some(Source::Computed { from: None, .. })
        ));
        for (param, known) in [
            ("tail_rise", Known::Published),
            ("capstan.height", Known::Published),
            ("front_edge", Known::Traced),
            ("width", Known::Assumed),
            ("front_rail.height", Known::Derived),
            ("front_rail.x", Known::Published),
            ("front_punching.felt", Known::Assumed),
            ("stick_height", Known::Traced),
        ] {
            assert_eq!(p.known(param).map(|(k, _)| k), Some(known), "{param}");
        }
        let section = Profile::named(SECTION).expect("shipped");
        for (param, known) in [
            ("stick_height", Known::Measured),
            ("capstan.x", Known::Measured),
            ("front_edge", Known::Assumed),
            ("tail_end", Known::Derived),
            ("drawer", Known::Measured),
        ] {
            assert_eq!(section.known(param).map(|(k, _)| k), Some(known), "{param}");
        }
    }

    /// Consistency: **each record his profile reads is the one its number
    /// names, and what he borrows of the owner's measured section is the
    /// section's.** A pointer to another record of the same file and unit
    /// loads, and nothing on his sheet need read the number back: his key
    /// button's height bound to its length, or his front pin's length to its
    /// thickness, moves no reading a test pins. So a number traced from his
    /// Figure A.1 is bound to the trace's record of its own name; one read
    /// from the owner's records or Stanwood's is bound to the very record
    /// the measured section binds it to, with the same bits. What his
    /// profile states as borrowed from the section, his balance rail and
    /// felts, his cloth, his balance pin, his capstan's buried shank and his
    /// key's width, the section states as its own assumption, to the bit;
    /// and his papers, as recorded, are those the section's regulation lays
    /// on arrival, so that his balance bearing stands where the section's
    /// does. What he reads of the owner's instruments is known as borrowed,
    /// an assumption there as one; what he reads of no instrument,
    /// Stanwood's protocol and the drawer's papers, as the section knows
    /// it. His printed points are read back by pointer against his record
    /// elsewhere (`his_profile_reads_his_printed_lines_and_circles`, C11).
    #[test]
    fn each_record_his_profile_reads_is_the_one_its_number_names() {
        use crate::instrument::{Instrument, Stack};
        let his = Profile::named("hirschkorn").expect("shipped");
        let section = Profile::named(SECTION).expect("shipped");
        let (mut traced, mut borrowed) = (0, 0);
        for p in &his.provenance {
            match &p.source {
                Source::Record {
                    file: "hirschkorn-key-outline.json",
                    pointer,
                    ..
                } => {
                    traced += 1;
                    assert_eq!(
                        *pointer,
                        format!("/records/{}", p.param.replace('.', "_")),
                        "{} is bound to another number's trace",
                        p.param
                    );
                }
                Source::Record {
                    file: "hirschkorn-boston-gp178-note52.json",
                    ..
                }
                | Source::Computed { .. }
                | Source::Stated { .. } => {}
                Source::Record {
                    instrument,
                    reliability,
                    ..
                } => {
                    borrowed += 1;
                    assert!(
                        section.provenance.iter().any(|c| c.param == p.param
                            && c.source == p.source
                            && c.value.to_bits() == p.value.to_bits()),
                        "{} reads {:?}, which the measured section does not bind it to",
                        p.param,
                        p.source
                    );
                    let (his_known, _) = his.known_from(&p.source);
                    let (its_known, _) = section.known_from(&p.source);
                    let expected = match (instrument, reliability) {
                        (Some(_), Reliability::Assumed) => Known::Assumed,
                        (Some(_), _) => Known::Borrowed,
                        (None, _) => its_known,
                    };
                    assert_eq!(his_known, expected, "{} on his key", p.param);
                }
            }
        }
        assert!(traced > 0 && borrowed > 0);
        for param in [
            "width",
            "capstan.shaft_length",
            "capstan.tip_height",
            "balance_rail.height",
            "balance_punching.felt",
            "front_punching.felt",
            "back_rail.cloth",
            "back_rail.cloth_edge_radius",
            "balance_rail.crown.height",
            "balance_rail.pin_height",
            "back_rail.half_length",
        ] {
            let stated = |p: &Profile| {
                let v = p.provenance(param).expect(param);
                let Source::Stated {
                    profile,
                    reliability,
                    ..
                } = &v.source
                else {
                    panic!("{param} is stated on {}", p.name)
                };
                (v.value.to_bits(), profile.clone(), *reliability)
            };
            let (his_bits, his_profile, his_marker) = stated(his);
            let (its_bits, its_profile, its_marker) = stated(section);
            assert_eq!(his_bits, its_bits, "{param} is not the section's");
            assert_eq!(his_marker, its_marker, "{param} is known otherwise");
            assert_eq!(
                (his_profile.as_str(), its_profile.as_str()),
                ("hirschkorn", SECTION),
                "{param}"
            );
        }
        let regulated = Instrument::of(section).expect("the section arrives regulated");
        let mut laid = his.provenance.iter().filter(|p| p.param == PAPER.name);
        for stack in [Stack::Balance, Stack::Front] {
            for &paper in regulated.papers(stack) {
                assert_eq!(
                    laid.next().map(|p| p.value.to_bits()),
                    Some(paper.to_bits()),
                    "his papers are not those the section's regulation lays"
                );
            }
        }
        assert!(laid.next().is_none());
    }

    /// Consistency: **what a profile reads of another instrument is
    /// borrowed.** Each shipped profile names the instruments it describes
    /// as its records' sources name them, and a number read from a record of
    /// any other is known as borrowed, whatever its record makes it on its
    /// own instrument, an assumption excepted, which stays one; a record of
    /// no instrument, as Stanwood's protocol and the drawer's stock of
    /// papers are, is borrowed by none. So on Hirschkorn's key the pins and
    /// punchings the owner measured on his key frame, and the depth of the
    /// front mortise he measured on his keys, are borrowed, not measured,
    /// and the only numbers of his known as measured are the drawer's
    /// papers; on the section, built on the owner's records, they are
    /// measured, and nothing is borrowed.
    #[test]
    fn what_a_profile_reads_of_another_instrument_is_borrowed() {
        for name in Profile::names() {
            let p = Profile::named(name).unwrap();
            for v in &p.provenance {
                let Source::Record {
                    instrument,
                    reliability,
                    ..
                } = &v.source
                else {
                    continue;
                };
                let other = instrument.is_some_and(|i| !p.instruments.iter().any(|own| own == i));
                assert_eq!(p.borrows(&v.source), other, "{name}: {}", v.param);
                assert_eq!(
                    p.known_from(&v.source).0 == Known::Borrowed,
                    other && *reliability != Reliability::Assumed,
                    "{name}: {}",
                    v.param
                );
            }
        }
        let his = Profile::named("hirschkorn").unwrap();
        let section = Profile::named(SECTION).expect("shipped");
        let known = |p: &Profile, param: &str| p.known(param).map(|(k, _)| k);
        for param in [
            "front_rail.pin.length",
            "front_rail.pin.thickness",
            "front_rail.pin_height",
            "front_rail.punching_radius",
            "front_rail.punching_hole_radius",
            "balance_rail.pin.diameter",
            "balance_rail.punching_radius",
            "balance_rail.punching_hole_radius",
            "front_mortise.depth",
        ] {
            assert_eq!(known(his, param), Some(Known::Borrowed), "{param}");
            assert_eq!(known(section, param), Some(Known::Measured), "{param}");
        }
        for (param, how) in [
            ("balance_rail.half_length", Known::Assumed),
            ("front_mortise.cutter_diameter", Known::Assumed),
            ("front_mortise.travel", Known::Assumed),
            ("drawer", Known::Measured),
            ("measuring_point_from_lip", Known::Published),
        ] {
            assert_eq!(known(his, param), Some(how), "{param}");
        }
        for v in &his.provenance {
            assert!(
                his.known_from(&v.source).0 != Known::Measured || v.param == DRAWER,
                "{} is known as measured on his key",
                v.param
            );
        }
        assert!(section
            .provenance
            .iter()
            .all(|v| !section.borrows(&v.source)));
    }

    /// Consistency: **a profile names the instruments it describes**, each
    /// one a record names: the measured section stripped of them, or naming one no
    /// record is of, does not load, so a misspelt name cannot make every
    /// record it reads borrowed.
    #[test]
    fn a_profile_names_the_instruments_it_describes() {
        let text = shipped_text(SECTION).unwrap();
        let mut file: Value = serde_json::from_str(text).unwrap();
        file.as_object_mut().unwrap().remove("instruments");
        assert!(matches!(
            Profile::load(SECTION, &serde_json::to_string(&file).unwrap()),
            Err(LoadError::Choice {
                what: "instruments",
                ..
            })
        ));
        file["instruments"] = serde_json::json!(["a grand piano key fram"]);
        assert_eq!(
            Profile::load(SECTION, &serde_json::to_string(&file).unwrap()),
            Err(LoadError::UnknownInstrument(
                "a grand piano key fram".to_owned()
            ))
        );
    }

    /// Consistency: **what is borrowed follows the instruments a profile
    /// names, and nothing else**: the same profile, under the same name,
    /// reading the same records, knows them otherwise when it names other
    /// instruments, so no list of profiles, files or numbers can stand in
    /// for the records' sources. Hirschkorn's profile naming the owner's key
    /// frame among its own reads that frame's pins and punchings as
    /// measured; the measured section naming only its key frame borrows its key
    /// section's keystick and keeps its frame's pins measured, and naming
    /// only its key section the reverse; an assumption stays one whichever
    /// it names; and a record of any file is borrowed exactly when it is of
    /// an instrument the profile does not name.
    #[test]
    fn what_is_borrowed_follows_the_instruments_a_profile_names() {
        let naming = |name: &str, instruments: &[&str]| {
            let mut file: Value = serde_json::from_str(shipped_text(name).unwrap()).unwrap();
            file["instruments"] = serde_json::json!(instruments);
            Profile::load(name, &serde_json::to_string(&file).unwrap()).unwrap()
        };
        let known = |p: &Profile, param: &str| p.known(param).map(|(k, _)| k);
        let (his_own, frame, keys) = (
            "Boston GP-178 grand piano, note 52",
            "a grand piano key frame",
            "natural keys from a grand piano keyboard",
        );
        let his_and_frame = naming("hirschkorn", &[his_own, frame]);
        let frame_only = naming(SECTION, &[frame]);
        let keys_only = naming(SECTION, &[keys]);
        for param in [
            "front_rail.pin.length",
            "front_rail.punching_radius",
            "balance_rail.pin.diameter",
            "balance_rail.punching_radius",
        ] {
            assert_eq!(
                known(&his_and_frame, param),
                Some(Known::Measured),
                "{param}"
            );
            assert_eq!(known(&frame_only, param), Some(Known::Measured), "{param}");
            assert_eq!(known(&keys_only, param), Some(Known::Borrowed), "{param}");
        }
        for param in ["stick_height", "keytop_thickness", "tail_length"] {
            assert_eq!(known(&frame_only, param), Some(Known::Borrowed), "{param}");
            assert_eq!(known(&keys_only, param), Some(Known::Measured), "{param}");
        }
        for p in [&his_and_frame, &frame_only, &keys_only] {
            assert_eq!(known(p, "balance_rail.half_length"), Some(Known::Assumed));
        }
        let record = |instrument| Source::Record {
            file: "of-no-shipped-profile.json",
            pointer: "/records/0".to_owned(),
            reliability: Reliability::Verified,
            measured: true,
            derive: Derive::Whole,
            traced: false,
            instrument,
        };
        for (p, own) in [
            (Profile::named(SECTION).expect("shipped"), frame),
            (&frame_only, frame),
            (&keys_only, keys),
        ] {
            assert!(p.borrows(&record(Some("another grand piano"))));
            assert_eq!(
                p.known_from(&record(Some("another grand piano"))).0,
                Known::Borrowed
            );
            assert!(!p.borrows(&record(Some(own))));
            assert!(!p.borrows(&record(None)));
            assert_eq!(p.known_from(&record(None)).0, Known::Measured);
        }
    }

    /// Consistency: **a key regulated on arrival is given its key height**:
    /// the height a key stands at is read only on a frame that arrives as
    /// recorded, so the section, regulated on arrival, stating no key
    /// height, does not load.
    #[test]
    fn a_key_regulated_on_arrival_is_given_its_key_height() {
        let text = shipped_text(SECTION).unwrap();
        let mut file: Value = serde_json::from_str(text).unwrap();
        for v in file["values"].as_array_mut().unwrap() {
            if v["param"] == "key_height" {
                *v = serde_json::json!({
                    "param": "key_height", "none": true, "why": "a test"
                });
            }
        }
        assert!(matches!(
            Profile::load(SECTION, &serde_json::to_string(&file).unwrap()),
            Err(LoadError::Malformed { param, .. }) if param == "key_height"
        ));
    }
}

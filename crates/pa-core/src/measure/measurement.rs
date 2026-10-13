//! A reading as the trade takes it: its value in full, what the protocol
//! reads it as, and what a datum needs beside its number.
//!
//! # Read as the trade reads it
//!
//! A technician stacking gram weights on the measuring point reads a down
//! weight whose threshold is 46.3 g as 47 g, since 46 g does not move the
//! key; an up weight of 21.6 g as 21 g, the most the key lifts; and computes
//! balance and friction weight from those two, 34 g and 13 g. A front
//! weight is read on a scale to the nearest tenth of a gram, the key weight
//! ratio to two decimals [V, Stanwood 2000, glossary, pp. 22–23]. So each
//! reading carries the value the procedure found, in full, and what the
//! trade reads it as ([`Measurement::shown`]), with the rule that takes one
//! to the other ([`Rounding`]) and the protocol's resolution. Rounded to the
//! nearest gram instead, a down weight would read a gram light about half
//! the time, a weight that does not make the key go down. A value the
//! procedure cannot tell from a whole gram is read as that gram, never as
//! the next ([`Rounding::apply`]): rounding noise is not a gram.
//!
//! # Not a datum without its conditions
//!
//! A weighing without the stops it was read on, the weights on the key and
//! the mount it stood in is not a datum ([`Conditions`]); nor one whose
//! inputs nobody measured, which is [`Trust::Indicative`] and says which
//! ([`Input`]). Every reading carries the counts of the static solves it
//! took ([`SolveReport`]) and the fingerprint of the solver that took it.

use crate::contact::ContactCode;
use crate::geometry::Point;
use crate::parts::mounts::KeyMount;
use crate::solve::SolveReport;

/// Which reading, by which procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProcedureId {
    /// Front weight, on Stanwood's jig: the levelled key's own weight read at
    /// the measuring point.
    FrontWeight,
    /// Key weight ratio, force convention, on the tared jig with 10 g at the
    /// capstan.
    KeyWeightRatio,
    /// Key height, the key seated at the top of its stroke.
    KeyHeight,
    /// Key dip, from the top seat to the bottom one.
    KeyDip,
    /// Down weight, by the breakaway walk from the top seat.
    DownWeight,
    /// Up weight, by the breakaway walk from the bottom seat.
    UpWeight,
    /// Balance weight, `(D + U) / 2`, from the two as read.
    BalanceWeight,
    /// Friction weight, `(D − U) / 2`, from the two as read.
    FrictionWeight,
    /// Key friction weight by Stanwood's protocol: the front weight zeroed
    /// with a temporary weight on the levelled jig, then the down weight on
    /// the frame.
    KeyFrictionWeightZeroedOnTheJig,
    /// Key friction weight with the temporary weight chosen to zero the
    /// key's balance where it rests on its frame, then the down weight.
    KeyFrictionWeightZeroedOnTheFrame,
}

impl ProcedureId {
    /// Every reading of the sheet, in its order.
    pub const ALL: [Self; 10] = [
        Self::FrontWeight,
        Self::KeyWeightRatio,
        Self::KeyHeight,
        Self::KeyDip,
        Self::DownWeight,
        Self::UpWeight,
        Self::BalanceWeight,
        Self::FrictionWeight,
        Self::KeyFrictionWeightZeroedOnTheJig,
        Self::KeyFrictionWeightZeroedOnTheFrame,
    ];

    /// The reading's identifier, English `snake_case`.
    pub fn code(self) -> &'static str {
        match self {
            Self::FrontWeight => "front_weight",
            Self::KeyWeightRatio => "key_weight_ratio",
            Self::KeyHeight => "key_height",
            Self::KeyDip => "key_dip",
            Self::DownWeight => "down_weight",
            Self::UpWeight => "up_weight",
            Self::BalanceWeight => "balance_weight",
            Self::FrictionWeight => "friction_weight",
            Self::KeyFrictionWeightZeroedOnTheJig => "key_friction_weight_zeroed_on_the_jig",
            Self::KeyFrictionWeightZeroedOnTheFrame => "key_friction_weight_zeroed_on_the_frame",
        }
    }

    /// What the procedure reads that a measurement or a publication may give,
    /// or not: the inputs its trust rests on.
    ///
    /// With rigid stops, key height, key dip and the key weight ratio do not
    /// depend on the key's mass, nor does the key friction weight zeroed on
    /// the frame, whose temporary weight cancels the key's balance whatever
    /// it is. Stanwood's does: its temporary weight cancels the level key's
    /// moment, and what is left of the key's at its rest attitude is in the
    /// reading.
    pub fn reads(self) -> &'static [Input] {
        match self {
            Self::FrontWeight => &[Input::KeyMass, Input::Geometry],
            Self::KeyWeightRatio | Self::KeyHeight | Self::KeyDip => &[Input::Geometry],
            Self::DownWeight
            | Self::UpWeight
            | Self::BalanceWeight
            | Self::FrictionWeight
            | Self::KeyFrictionWeightZeroedOnTheJig => {
                &[Input::KeyMass, Input::KeyFriction, Input::Geometry]
            }
            Self::KeyFrictionWeightZeroedOnTheFrame => &[Input::KeyFriction, Input::Geometry],
        }
    }
}

/// The unit a reading's value is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// Grams at the measuring point: the weighing algebra's unit.
    Gram,
    /// Metres.
    Metre,
    /// A ratio, dimensionless.
    One,
}

impl Unit {
    /// The unit's identifier.
    pub fn code(self) -> &'static str {
        match self {
            Self::Gram => "g",
            Self::Metre => "m",
            Self::One => "",
        }
    }
}

/// How the trade reads a value to its resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rounding {
    /// Up: the least whole number of steps that does it. Down weight and key
    /// friction weight, the least gram weights that make the key go down.
    Up,
    /// Down: the greatest whole number of steps. Up weight, the most gram
    /// weights the key lifts.
    Down,
    /// To the nearest step: a scale's reading. Front weight to the tenth of
    /// a gram, the key weight ratio to the hundredth.
    Nearest,
    /// Computed from other readings as they were read: balance and friction
    /// weight from the down and up weights in whole grams, a half gram
    /// possible.
    FromReadings,
    /// Not rounded: key height and key dip, read to the display's
    /// hundredth of a millimetre, the thinnest paper being what the trade
    /// can tell apart.
    None,
}

impl Rounding {
    /// The rule's identifier.
    pub fn code(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Nearest => "nearest",
            Self::FromReadings => "from_readings",
            Self::None => "none",
        }
    }

    /// `value` read to steps of `1 / per_unit` by this rule, a value within
    /// `within` of a whole number of steps read as that number: up or down,
    /// what the procedure cannot tell from a step is not rounded into the
    /// next one. `within` is the procedure's own resolution, the walk's
    /// [`super::walk::ROUNDING_TOLERANCE_G`] for its thresholds; to the
    /// nearest step it changes nothing. `per_unit` is a whole number, so the
    /// result is the double nearest a whole number of steps; a reading of
    /// nothing is `0`, never `−0`. For [`Rounding::FromReadings`] and
    /// [`Rounding::None`], `value` itself.
    pub fn apply(self, value: f64, per_unit: f64, within: f64) -> f64 {
        let steps = value * per_unit;
        let slack = within * per_unit;
        let read = match self {
            Self::Up => (steps - slack).ceil() / per_unit,
            Self::Down => (steps + slack).floor() / per_unit,
            Self::Nearest => steps.round() / per_unit,
            Self::FromReadings | Self::None => value,
        };
        read + 0.0
    }
}

/// An input a procedure reads, whose source decides whether the reading is
/// a measurement or an indication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Input {
    /// The key's mass and centre of mass.
    KeyMass,
    /// The friction of the key's bushings.
    KeyFriction,
    /// The dimensions of the key and its frame the procedure reads.
    Geometry,
}

impl Input {
    /// The input's identifier.
    pub fn code(self) -> &'static str {
        match self {
            Self::KeyMass => "key_mass",
            Self::KeyFriction => "key_friction",
            Self::Geometry => "geometry",
        }
    }
}

/// What an input rests on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Basis {
    /// Measured on a real part, with its record.
    Measured,
    /// Read from a published source, with its record.
    Published,
    /// Derived from other values by a rule: the composite key's mass, from
    /// its volume at an assumed density.
    Estimated,
    /// Assumed, pending measurement.
    Assumed,
    /// Set on the bench by its user, who answers for it.
    SetByHand,
}

impl Basis {
    /// Whether a reading that reads an input on this basis is only an
    /// indication: an estimate or an assumption cannot make a measurement.
    pub fn is_indicative(self) -> bool {
        matches!(self, Self::Estimated | Self::Assumed)
    }
}

/// What each input of a bench rests on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bases {
    /// The key's mass and centre of mass.
    pub key_mass: Basis,
    /// The friction of the key's bushings.
    pub key_friction: Basis,
    /// The dimensions of the key and its frame.
    pub geometry: Basis,
}

impl Bases {
    /// What `input` rests on.
    pub fn of(&self, input: Input) -> Basis {
        match input {
            Input::KeyMass => self.key_mass,
            Input::KeyFriction => self.key_friction,
            Input::Geometry => self.geometry,
        }
    }
}

/// How far a reading can be trusted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trust {
    /// Every input the procedure reads is measured, published or set by the
    /// bench's user.
    Metrological,
    /// Some input the procedure reads is estimated or assumed.
    Indicative {
        /// Those inputs, in the order the procedure reads them.
        because: Vec<Input>,
    },
}

impl Trust {
    /// The trust of a reading that reads `inputs`, each on its basis in
    /// `bases`.
    pub fn of(inputs: &[Input], bases: &Bases) -> Self {
        let because: Vec<Input> = inputs
            .iter()
            .copied()
            .filter(|&i| bases.of(i).is_indicative())
            .collect();
        if because.is_empty() {
            Self::Metrological
        } else {
            Self::Indicative { because }
        }
    }
}

/// Where a key was read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mount {
    /// On Stanwood's front weight jig, turning without friction about its
    /// balance pin, levelled.
    Jig,
    /// On its frame.
    Frame(KeyMount),
}

/// The loads that seat the key by the trade's gestures, newtons, straight
/// down.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Preparation {
    /// At the capstan, standing for the action.
    pub capstan_load_n: f64,
    /// At the measuring point, pressing the key to the bottom of its
    /// stroke; zero where the key is read at the top.
    pub press_n: f64,
}

/// How a breakaway walk went.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Walked {
    /// The frictionless balance it started from, grams on the measuring
    /// point.
    pub from_g: f64,
    /// The weight it added at each step, grams; negative going up.
    pub step_g: f64,
    /// The steps it took.
    pub steps: u32,
    /// The steps that moved the key and left it short of the other seat.
    pub stalls: u32,
    /// The halvings of its last step.
    pub halvings: u32,
}

/// Which way a temporary weight was chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zeroed {
    /// To make the levelled key's front weight read zero on the jig,
    /// Stanwood's.
    OnTheJig,
    /// To cancel the key's balance where it rests at the top of its stroke
    /// on its frame.
    OnTheFrame,
}

/// The temporary weight the key friction weight protocol zeroes the front
/// weight with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TemporaryWeight {
    /// Its mass, grams.
    pub mass_g: f64,
    /// Where it stands on the key's top, in the key's frame: 100 mm behind
    /// the balance pin on a key heavy in front, 150 mm in front of it on one
    /// heavy behind.
    pub station: Point,
    /// What it zeroes.
    pub zeroed: Zeroed,
}

/// What a reading needs beside its number to be a datum.
#[derive(Debug, Clone, PartialEq)]
pub struct Conditions {
    /// Where the key was read.
    pub mount: Mount,
    /// The measuring point, metres back from the covering's lip.
    pub measuring_point_from_lip_m: f64,
    /// The contacts holding the seat the reading is taken at or starts from:
    /// the cloth at the top, whichever stop bottomed at the bottom.
    pub held_by: Vec<ContactCode>,
    /// The loads that seated the key, where the procedure seats it.
    pub preparation: Option<Preparation>,
    /// The breakaway walk, where the reading is one.
    pub walk: Option<Walked>,
    /// The temporary weight standing on the key, where there is one.
    pub temporary_weight: Option<TemporaryWeight>,
    /// The readings it is computed from, where it is computed.
    pub from: Vec<ProcedureId>,
}

impl Conditions {
    /// The conditions of a reading taken in `mount` at a measuring point
    /// `from_lip` metres back from the lip, and nothing else.
    pub fn at(mount: Mount, from_lip: f64) -> Self {
        Self {
            mount,
            measuring_point_from_lip_m: from_lip,
            held_by: Vec::new(),
            preparation: None,
            walk: None,
            temporary_weight: None,
            from: Vec::new(),
        }
    }
}

/// One reading of the sheet.
#[derive(Debug, Clone, PartialEq)]
pub struct Measurement {
    /// Which reading, by which procedure.
    pub procedure: ProcedureId,
    /// What the procedure found, in full, in [`Self::unit`].
    pub value: f64,
    /// What the trade reads it as: [`Self::value`] by [`Self::rounding`],
    /// to [`Self::resolution`].
    pub shown: f64,
    /// The unit of both.
    pub unit: Unit,
    /// The protocol's resolution, in [`Self::unit`].
    pub resolution: f64,
    /// How the trade reads the value to it.
    pub rounding: Rounding,
    /// What the reading needs beside its number.
    pub conditions: Conditions,
    /// How far it can be trusted.
    pub trust: Trust,
    /// The static solves it took, added up.
    pub report: SolveReport,
    /// The solver that took it ([`crate::solve::SOLVER_FINGERPRINT`]).
    pub fingerprint: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Consistency: **read as the trade reads it.** A down weight of 46.3 g
    /// reads 47 g, an up weight of 21.6 g reads 21 g; a threshold just past
    /// a whole gram reads the next one, and a negative one rounds towards
    /// the key going down by itself: −0.2 g reads 0 g, never −0 g. A front
    /// weight reads to the tenth, a ratio to the hundredth.
    #[test]
    fn readings_round_as_the_trade_reads_them() {
        let read =
            |rule: Rounding, value: f64, per_unit: f64| rule.apply(value, per_unit, 0.0).to_bits();
        let bits = |x: f64| x.to_bits();
        assert_eq!(read(Rounding::Up, 46.3, 1.0), bits(47.0));
        assert_eq!(read(Rounding::Down, 21.6, 1.0), bits(21.0));
        assert_eq!(read(Rounding::Up, 6.000_01, 1.0), bits(7.0));
        assert_eq!(read(Rounding::Up, -0.215, 1.0), bits(0.0));
        assert_eq!(read(Rounding::Down, -12.48, 1.0), bits(-13.0));
        assert_eq!(read(Rounding::Nearest, 6.481, 10.0), bits(6.5));
        assert_eq!(read(Rounding::Nearest, 0.524_29, 100.0), bits(0.52));
        assert_eq!(read(Rounding::None, 0.064_99, 1.0), bits(0.064_99));
    }

    /// Consistency: **rounding noise is not a gram.** Read within the walk's
    /// bracket, a threshold the walk cannot tell from a whole gram reads that
    /// gram, up or down: the half bracket a walk reads on bushings with no
    /// friction, over a threshold of nothing but rounding, reads 0 g and not
    /// 1 g; a gram read back half a bracket over or under reads that gram;
    /// past a bracket, the trade's rule again. The tolerance is the bracket
    /// the walk's halvings leave of its step, a part in 65 536 of a gram.
    #[test]
    fn rounding_noise_is_not_a_gram() {
        use super::super::walk::{HALVINGS, ROUNDING_TOLERANCE_G, STEP_G};
        let bracket = ROUNDING_TOLERANCE_G;
        assert_eq!(
            bracket.to_bits(),
            (STEP_G / f64::from(1u32 << HALVINGS)).to_bits()
        );
        assert_eq!(bracket.to_bits(), (1.0_f64 / 65_536.0).to_bits());
        let read = |rule: Rounding, value: f64| rule.apply(value, 1.0, bracket).to_bits();
        let bits = |x: f64| x.to_bits();
        let half = 0.5 * bracket;
        // A threshold of nothing, rounded: what the walk reads on bushings
        // with no friction, zeroed on the frame.
        assert_eq!(read(Rounding::Up, half + 7.2e-16), bits(0.0));
        assert_eq!(read(Rounding::Up, 6.0 + half), bits(6.0));
        assert_eq!(read(Rounding::Up, 6.0 - half), bits(6.0));
        assert_eq!(read(Rounding::Up, 6.0 + 2.0 * bracket), bits(7.0));
        assert_eq!(read(Rounding::Up, 46.3), bits(47.0));
        assert_eq!(read(Rounding::Down, -13.0 - half), bits(-13.0));
        assert_eq!(read(Rounding::Down, -13.0 + half), bits(-13.0));
        assert_eq!(read(Rounding::Down, -13.0 - 2.0 * bracket), bits(-14.0));
        assert_eq!(read(Rounding::Down, -half), bits(0.0));
        assert_eq!(read(Rounding::Down, 21.6), bits(21.0));
        // To the nearest step, the tolerance changes nothing.
        assert_eq!(
            Rounding::Nearest.apply(6.449_999, 10.0, bracket).to_bits(),
            bits(6.4)
        );
    }

    /// Consistency: a reading whose inputs are all measured is a
    /// measurement, and one that reads an assumed or estimated input names
    /// it, in the order the procedure reads them; an input set by the
    /// bench's user is his to answer for.
    #[test]
    fn trust_names_what_is_not_measured() {
        let measured = Bases {
            key_mass: Basis::Measured,
            key_friction: Basis::SetByHand,
            geometry: Basis::Published,
        };
        assert_eq!(
            Trust::of(ProcedureId::DownWeight.reads(), &measured),
            Trust::Metrological
        );
        let composite = Bases {
            key_mass: Basis::Estimated,
            key_friction: Basis::Assumed,
            geometry: Basis::Assumed,
        };
        assert_eq!(
            Trust::of(
                ProcedureId::KeyFrictionWeightZeroedOnTheFrame.reads(),
                &composite
            ),
            Trust::Indicative {
                because: vec![Input::KeyFriction, Input::Geometry]
            }
        );
        assert_eq!(
            Trust::of(ProcedureId::FrontWeight.reads(), &composite),
            Trust::Indicative {
                because: vec![Input::KeyMass, Input::Geometry]
            }
        );
    }

    /// Every reading has its own identifier.
    #[test]
    fn every_reading_has_its_own_code() {
        let mut codes: Vec<&str> = ProcedureId::ALL.iter().map(|p| p.code()).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), ProcedureId::ALL.len());
    }
}

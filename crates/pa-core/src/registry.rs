//! What a setting of the domain may be, declared by each part beside the
//! fields it sets.
//!
//! # The domain declares, the application presents
//!
//! What is physically admissible is domain knowledge: a felt is no thinner
//! than nothing, a paper is one the drawer holds, a capstan stands behind the
//! balance pin. So each part declares its settings here, next to its fields
//! ([`crate::parts::KeyDesign::PARAMS`], [`crate::parts::FrameDesign::PARAMS`],
//! [`crate::measure::TouchweightProtocol::PARAMS`], and the rest), in SI, and
//! [`crate::validate`] reads the same declarations. An application composes
//! them under paths, with the trade unit in each path's name, and adds what
//! only it knows: where a control stands on its page, and the range a slider
//! sweeps. This module holds no wording and no unit but SI, except where a
//! setting's name carries its trade unit, as a weight in grams does.
//!
//! # Refused, not clamped
//!
//! A value outside a setting's hard range is refused, the setting left as it
//! was, and so is one that is not a number ([`ParamSpec::admits`]): a value
//! silently clamped would be a setting the user did not make. Everything a
//! setting's range admits is computed, however badly it regulates the key; a
//! design no part could be built as is refused by [`crate::validate`], which
//! sees several settings at once.

use crate::validate::Problem;

/// What a setting measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quantity {
    /// A length or a position, meters.
    Length,
    /// A mass or a weight, grams where the name says so.
    Mass,
    /// A torque, newton-meters.
    Torque,
}

/// How a setting moves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Step {
    /// Continuously: a dimension edited, or a regulating screw turned, which
    /// turns continuously and has no quantum.
    Continuous,
    /// By the thicknesses a drawer holds: a paper is one of them or it is
    /// not a paper. Which they are is the instrument's drawer's to say
    /// ([`crate::parts::frame::Drawer::admits`]), not the setting's.
    Drawer,
}

/// Which layer of the instrument a setting belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    /// What the choice of parts fixes: the key's and the frame's dimensions,
    /// the bushings.
    Design,
    /// What a technician turns or lays to regulate: the capstan, the papers,
    /// the targets.
    Regulation,
    /// The conventions a protocol is run under: where every weight is
    /// centered.
    Protocol,
    /// What an experiment puts on the key: the weight on the measuring point.
    Experiment,
}

/// How a setting is reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Set directly: a dimension, a felt, a screw.
    Actuated,
    /// Aimed at: a trade target the regulation solve turns into the
    /// settings that reach it.
    Target,
    /// Typed as it was read on a real part: a key's weighing.
    Measured,
}

/// One setting of a part: what it is, what it admits, and how to read and
/// write it on the part that holds it.
pub struct ParamSpec<T> {
    /// Its name, the part's own: `front_rail.pin_height`. Unique across the
    /// domain's declarations, so a problem names it without ambiguity.
    pub name: &'static str,
    /// What it measures.
    pub quantity: Quantity,
    /// The physically admissible range, inclusive, SI unless the name
    /// carries a trade unit; `f64::INFINITY` where a side is unbounded.
    /// Outside it a value is refused.
    pub hard: (f64, f64),
    /// How it moves.
    pub step: Step,
    /// Which layer it belongs to.
    pub layer: Layer,
    /// How it is reached.
    pub kind: Kind,
    /// Reads it.
    pub get: fn(&T) -> f64,
    /// Writes it, unchecked: [`Self::apply`] checks first.
    pub set: fn(&mut T, f64),
}

impl<T> Clone for ParamSpec<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ParamSpec<T> {}

impl<T> core::fmt::Debug for ParamSpec<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ParamSpec")
            .field("name", &self.name)
            .field("quantity", &self.quantity)
            .field("hard", &self.hard)
            .field("step", &self.step)
            .field("layer", &self.layer)
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl<T> ParamSpec<T> {
    /// Whether this setting admits `value`: a number, within its hard range;
    /// or why not. A paper's thickness is checked against the drawer it is
    /// laid from ([`crate::parts::frame::Drawer::admits`]).
    pub fn admits(&self, value: f64) -> Result<(), Problem> {
        if !value.is_finite() {
            return Err(Problem::NotFinite { param: self.name });
        }
        let (min, max) = self.hard;
        if !(min..=max).contains(&value) {
            return Err(Problem::OutOfRange {
                param: self.name,
                value,
                min,
                max,
            });
        }
        Ok(())
    }

    /// Sets `value` on `on` where this setting admits it ([`Self::admits`]);
    /// otherwise says why, `on` left as it was.
    pub fn apply(&self, on: &mut T, value: f64) -> Result<(), Problem> {
        self.admits(value)?;
        (self.set)(on, value);
        Ok(())
    }

    /// Whether the value `on` holds is one this setting admits.
    pub fn holds(&self, on: &T) -> Result<(), Problem> {
        self.admits((self.get)(on))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LENGTH: ParamSpec<f64> = ParamSpec {
        name: "length",
        quantity: Quantity::Length,
        hard: (0.0, f64::INFINITY),
        step: Step::Continuous,
        layer: Layer::Design,
        kind: Kind::Actuated,
        get: |x| *x,
        set: |x, v| *x = v,
    };

    /// Consistency: **a value a setting does not admit is refused, and
    /// nothing is written**: not a number, either infinity, or outside the
    /// hard range; an admitted value is written as given.
    #[test]
    fn a_value_a_setting_does_not_admit_is_refused_and_nothing_is_written() {
        let mut x = 0.001;
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                LENGTH.apply(&mut x, bad),
                Err(Problem::NotFinite { param: "length" })
            );
        }
        assert_eq!(
            LENGTH.apply(&mut x, -0.002),
            Err(Problem::OutOfRange {
                param: "length",
                value: -0.002,
                min: 0.0,
                max: f64::INFINITY,
            })
        );
        assert_eq!(x.to_bits(), 0.001_f64.to_bits(), "a refusal wrote");
        assert_eq!(LENGTH.apply(&mut x, 0.0004), Ok(()));
        assert_eq!(x.to_bits(), 0.0004_f64.to_bits());
        assert_eq!(LENGTH.holds(&x), Ok(()));
    }
}

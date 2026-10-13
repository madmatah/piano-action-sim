//! Regulation observables, expressed as the technician's own protocols.
//!
//! The rule for this module: a measurement is implemented the way the trade
//! measures it, not the way it is easiest to probe numerically. If the simulator
//! reports a let-off, it must be the hammer-to-string distance at the instant
//! the jack leaves the knuckle — not the value of a parameter.
//!
//! Every reading of the weighing sheet is a procedure run on the model
//! ([`weighing`]): front weight and the key weight ratio on Stanwood's jig
//! ([`jig`]), the key a body on a pin held level and the scale reading the
//! force that holds it; key height and dip from the key seated on its frame by
//! the trade's gestures ([`seat`]); down and up weight by the breakaway walk
//! from each seat ([`walk`]), balance and friction weight from those two as
//! read; and the key friction weight by Stanwood's protocol and by its
//! variant zeroed on the frame. Each comes back as a [`measurement`]: its
//! value in full, what the trade reads it as, its conditions and its trust.
//! A key friction weight typed is turned into the bushings' torque by
//! Stanwood's protocol run backwards, the torque at which it reads the typed
//! grams ([`weighing::Weighing::bushings_reading`]). The readings' closed
//! forms stay in an oracle module that only this crate's tests compile, to
//! check the procedures against.

pub mod jig;
pub mod measurement;
#[cfg(test)]
pub(crate) mod oracle;
pub mod seat;
#[cfg(test)]
mod seat_corpus;
pub mod walk;
pub mod weighing;

use crate::geometry::Point;
use crate::parts::KeyDesign;

/// Touchweight quantities, after Stanwood's *New Touchweight Metrology*.
///
/// # The two reciprocal Key Ratio conventions
///
/// Stanwood's own corpus uses both, and conflating them is a factor of ~3.3 on
/// the wippen term. Mind which one you are holding.
///
/// * **Force convention**, the one that enters the equation of balance:
///   measured by taring the scale, placing 10 g on the key at the
///   capstan/heel contact point and reading what arrives at the front of the
///   key — *"if the scale reads -5.5 the Key Weight Ratio (KR) would be
///   0.55"*. So `KR ≈ 0.50…0.60`, dimensionless, and `WBW = KR × WW` is
///   necessarily **smaller** than `WW`, because a reducing lever cannot
///   amplify. ([`jig::KeyJig::key_ratio`] runs that protocol on the model.)
/// * **Geometric convention**: the ratio of lever arms, front over capstan,
///   *"nominally 2:1 for modern pianos"*. That is the reciprocal of the first.
///
/// All weights are measured with the touch weights centred on the **measuring
/// point**, a set distance back from the front lip of the key
/// ([`TouchweightProtocol`]).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Touchweight {
    /// Down weight: the least weight on the measuring point that starts the
    /// key down with slow, controlled hammer motion, grams.
    pub down_weight_g: f64,
    /// Up weight: the most weight the key can lift from the depressed
    /// position, grams.
    pub up_weight_g: f64,
    /// Front weight: the key alone, levelled and pivoted on its balance pin,
    /// read at the measuring point, grams.
    pub front_weight_g: f64,
    /// Wippen weight: the wippen pivoted on its centre, read at the
    /// capstan/heel contact point, grams.
    pub wippen_weight_g: f64,
    /// Strike weight: hammer plus the radius weight of the shank, pivoted at
    /// the flange, read at the strike line radius, grams.
    pub strike_weight_g: f64,
    /// Key ratio, **force convention**, dimensionless, about 0.50 to 0.60.
    pub key_ratio: f64,
}

impl Touchweight {
    /// Balance weight, `(D + U) / 2`, grams.
    pub fn balance_weight_g(&self) -> f64 {
        (self.down_weight_g + self.up_weight_g) / 2.0
    }

    /// Friction weight, `(D - U) / 2`, grams. Entirely dry friction: this is
    /// the quantity that cannot be produced by a regularized friction law
    /// resolved statically, and the reason the axis friction in this engine
    /// is set-valued.
    pub fn friction_weight_g(&self) -> f64 {
        (self.down_weight_g - self.up_weight_g) / 2.0
    }

    /// Top action balance weight, `BW + FW`, grams.
    pub fn top_action_balance_weight_g(&self) -> f64 {
        self.balance_weight_g() + self.front_weight_g
    }

    /// Wippen balance weight, `KR × WW`, grams.
    pub fn wippen_balance_weight_g(&self) -> f64 {
        self.key_ratio * self.wippen_weight_g
    }

    /// Strike balance weight, `TopBW − WipBW`, grams.
    pub fn strike_balance_weight_g(&self) -> f64 {
        self.top_action_balance_weight_g() - self.wippen_balance_weight_g()
    }

    /// Strike ratio, `SBW / SW`. Dimensionless.
    ///
    /// Stanwood's survey bands, and note that "full leverage" is the *low*
    /// number: full 5.0–5.7, medium 5.7–6.3, low 6.3–7.0, with **5.5 recommended
    /// as the modern standard** (the level at which Hamburg Steinway was found
    /// consistently, and consistently best). Below 5.0 the hammer moves too
    /// slowly relative to the key and excessive key dip is needed; above 7.0 it
    /// moves too fast and the dip has to be made excessively shallow.
    pub fn strike_ratio(&self) -> Option<f64> {
        if self.strike_weight_g <= 0.0 {
            return None;
        }
        Some(self.strike_balance_weight_g() / self.strike_weight_g)
    }

    /// Guard predicate for validation level 0.
    ///
    /// The two Key Ratio conventions are reciprocal, and substituting one for the
    /// other inflates the wippen term by ~3.3x while still producing
    /// plausible-looking numbers. These two assertions make that substitution a
    /// test failure rather than a silent error.
    ///
    /// The window only has to separate the conventions: real actions run from
    /// about 0.5 to 0.68 in the force convention (an 1878 Steinway concert grand,
    /// "unusually high" [V, Igrec, *Practical Touch*, 2016, scenario 5]), the
    /// geometric one from about 1.5 to 2.2. A window drawn round typical values
    /// would reject real pianos, which a design tool must weigh.
    pub fn conventions_are_sane(&self) -> bool {
        let kr_plausible = self.key_ratio > 0.3 && self.key_ratio < 1.0;
        let lever_reduces = self.wippen_balance_weight_g() < self.wippen_weight_g;
        kr_plausible && lever_reduces
    }
}

/// How far back from the front lip of the key every weight is centred, metres:
/// the default of [`TouchweightProtocol::measuring_point_from_lip`].
///
/// The glossary's *Measuring Point*: *"The datum point on the top of the key
/// 13mm or 1/2" back from the front lip of the key. Weights are centered on
/// this point when measuring Up Weight and Down Weight."* [V, Stanwood 2000,
/// glossary, p. 23; `data/params/touchweight-protocol.json`]
pub const MEASURING_POINT_FROM_LIP: f64 = 0.013;

/// The imperial alternative the same glossary gives, 1/2 inch, metres.
///
/// It differs from [`MEASURING_POINT_FROM_LIP`] by 0.3 mm, which is 0.1 % of
/// the lever arm and therefore below the 0.1 g resolution of the scale. The
/// default is 13 mm; this is the documented alternative a bench may be set to,
/// recorded rather than silently dropped. [V, Stanwood 2000, glossary, p. 23;
/// `data/params/touchweight-protocol.json`]
pub const MEASURING_POINT_HALF_INCH: f64 = 0.0127;

/// The conventions the touchweight protocols are run under.
///
/// # Why this is not a dimension of the key
///
/// Where a weight is centred is a convention of the **measurement**, not a
/// property of the wood: the same key read 13 mm or 1/2 inch back from its lip
/// is the same key, and every touchweight read on it changes by the lever. So
/// it stays out of [`KeyDesign`], which the outline and the mass are built
/// from, and the assembly being weighed holds it once
/// ([`crate::parts::KeyAssembly::protocol`]). Every protocol reads the
/// measuring point through [`Self::measuring_point`] — front weight and the
/// key weight ratio on the jig, the press that seats the key at the bottom of
/// its stroke, the weight the walks put on it for D and U and for both key
/// friction weights, and the touch weight — so no two of them can be weighed
/// at different points.
///
/// # Which lip
///
/// Stanwood's 2000 glossary measures from *"the front lip of the key"* without
/// saying which lip; the 1996 paper's *"front vertical edge of the key"* is
/// superseded with the rest of its protocols. The bench takes the
/// **covering's** lip, the owner's choice: the front edge of the keytop, which
/// overhangs the key front ([`KeyDesign::lip`]) — by 2 mm on the default key,
/// so the measuring point stands 2 mm further forward than one measured from
/// the key front, and every touchweight is read on a 0.8 % longer arm. Key
/// height and key dip keep their own datum, the key front.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchweightProtocol {
    /// How far back from the covering's lip every weight is centred, metres.
    ///
    /// [`MEASURING_POINT_FROM_LIP`] by default; [`MEASURING_POINT_HALF_INCH`]
    /// is the published alternative.
    pub measuring_point_from_lip: f64,
}

impl TouchweightProtocol {
    /// The measuring point on `key`, metres in front of the balance pin.
    ///
    /// The one place the datum and the distance meet. It takes the key and not
    /// a coordinate on purpose: handed a bare number, a caller could pass the
    /// key front instead of the lip, and every touchweight would be weighed
    /// 2 mm too far back with nothing to notice.
    pub fn measuring_point(&self, key: &KeyDesign) -> f64 {
        key.lip() - self.measuring_point_from_lip
    }

    /// The measuring point as a point of the key, in the key's frame: on the
    /// covering's top, [`Self::measuring_point`] in front of the balance pin.
    ///
    /// Where the weights are centred; on the jig the roller under the key
    /// bears directly below it.
    pub fn measuring_point_on_key(&self, key: &KeyDesign) -> Point {
        Point::new(
            self.measuring_point(key),
            key.stick_height + key.keytop_thickness,
        )
    }
}

impl Default for TouchweightProtocol {
    /// Stanwood's: 13 mm back from the front lip.
    fn default() -> Self {
        Self {
            measuring_point_from_lip: MEASURING_POINT_FROM_LIP,
        }
    }
}

/// The weight the current protocol places at the capstan to read the key ratio,
/// grams. Ten, since February 2000; a hundred before that. [V, Stanwood 2000,
/// p. 22; `data/params/touchweight-protocol.json`]
pub const KR_PROTOCOL_LOAD_G: f64 = 10.0;

#[cfg(test)]
mod tests {
    use super::*;

    /// A plausible well-regulated note, with the force convention for KR.
    fn sample() -> Touchweight {
        Touchweight {
            down_weight_g: 50.0,
            up_weight_g: 20.0,
            front_weight_g: 21.5,
            wippen_weight_g: 14.6,
            strike_weight_g: 8.86,
            key_ratio: 0.554,
        }
    }

    #[test]
    fn balance_and_friction_split_the_interval() {
        let t = sample();
        assert!((t.balance_weight_g() - 35.0).abs() < 1e-12);
        assert!((t.friction_weight_g() - 15.0).abs() < 1e-12);
    }

    #[test]
    fn the_wippen_lever_reduces_it_does_not_amplify() {
        let t = sample();
        assert!(t.wippen_balance_weight_g() < t.wippen_weight_g);
        assert!(t.conventions_are_sane());
    }

    #[test]
    fn the_inverted_key_ratio_is_rejected() {
        // 1.806 is the geometric ratio, front over capstan. Putting it where the
        // force convention belongs inflates the wippen term by 3.3x.
        let t = Touchweight {
            key_ratio: 1.806,
            ..sample()
        };
        assert!(
            !t.conventions_are_sane(),
            "an inverted key ratio must not pass"
        );
    }

    #[test]
    fn an_unusually_high_real_key_ratio_passes() {
        let t = Touchweight {
            key_ratio: 0.68,
            ..sample()
        };
        assert!(
            t.conventions_are_sane(),
            "a real action's key ratio must not be rejected"
        );
    }

    #[test]
    fn strike_ratio_lands_in_stanwoods_bands() {
        let t = sample();
        let r = t.strike_ratio().expect("strike weight is positive");
        assert!(
            (5.0..=7.0).contains(&r),
            "strike ratio {r} outside the published 5.0-7.0 band"
        );
    }

    #[test]
    fn strike_ratio_is_undefined_without_a_hammer() {
        let t = Touchweight {
            strike_weight_g: 0.0,
            ..sample()
        };
        assert!(t.strike_ratio().is_none());
    }
}

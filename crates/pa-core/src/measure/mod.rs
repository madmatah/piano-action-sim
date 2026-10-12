//! Regulation observables, expressed as the technician's own protocols.
//!
//! The rule for this module: a measurement is implemented the way the trade
//! measures it, not the way it is easiest to probe numerically. If the simulator
//! reports a let-off, it must be the hammer-to-string distance at the instant
//! the jack leaves the knuckle — not the value of a parameter.

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
///   amplify. ([`KeyWeighing::key_ratio`] runs that protocol.)
/// * **Geometric convention**: the ratio of lever arms, front over capstan,
///   *"nominally 2:1 for modern pianos"*. That is the reciprocal of the first.
///
/// All weights are measured with the touch weights centred on the **measuring
/// point**, a set distance back from the front lip of the key
/// ([`TouchweightProtocol`]).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Touchweight {
    /// Downweight: minimum mass at the measuring point that starts the key down
    /// with slow, controlled hammer motion. Grams.
    pub downweight: f64,
    /// Upweight: maximum mass the key can lift from the depressed position. Grams.
    pub upweight: f64,
    /// Front weight: radius weight of the key stick alone, pivoted on its
    /// bearing, taken at the measuring point. Grams.
    pub front_weight: f64,
    /// Wippen weight: radius weight of the wippen pivoted on its centre, taken
    /// at the capstan/heel contact point. Grams.
    pub wippen_weight: f64,
    /// Strike weight: hammer plus the radius weight of the shank, pivoted at the
    /// flange, taken at the strike line radius. Grams.
    pub strike_weight: f64,
    /// Key ratio, **force convention**, dimensionless, about 0.50 to 0.60.
    pub key_ratio: f64,
}

impl Touchweight {
    /// Balance weight, `(D + U) / 2`. Grams.
    pub fn balance_weight(&self) -> f64 {
        (self.downweight + self.upweight) / 2.0
    }

    /// Friction weight, `(D - U) / 2`. Grams. Entirely dry friction: this is the
    /// quantity that cannot be produced by a regularized friction law resolved
    /// statically, and the reason the axis friction in this engine is set-valued.
    pub fn friction_weight(&self) -> f64 {
        (self.downweight - self.upweight) / 2.0
    }

    /// Top action balance weight, `BW + FW`. Grams.
    pub fn top_action_balance_weight(&self) -> f64 {
        self.balance_weight() + self.front_weight
    }

    /// Wippen balance weight, `KR × WW`. Grams.
    pub fn wippen_balance_weight(&self) -> f64 {
        self.key_ratio * self.wippen_weight
    }

    /// Strike balance weight, `TopBW − WipBW`. Grams.
    pub fn strike_balance_weight(&self) -> f64 {
        self.top_action_balance_weight() - self.wippen_balance_weight()
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
        if self.strike_weight <= 0.0 {
            return None;
        }
        Some(self.strike_balance_weight() / self.strike_weight)
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
        let lever_reduces = self.wippen_balance_weight() < self.wippen_weight;
        kr_plausible && lever_reduces
    }
}

/// The key weighed on its own, by Stanwood's **current** protocols.
///
/// # Which paper, and why it matters
///
/// The metrology was published in 1996 and the **protocols were superseded in
/// February 2000** by *"Standard Protocols of the New Touchweight Metrology"*,
/// which this type follows. Three of the changes bear on the key, and taking the
/// 1996 text would have got two of them wrong:
///
/// 1. **Front weight** is read with the key set level **with a bubble level**,
///    not merely "in a horizontal attitude similar to when the key is at rest" —
///    *"the guesswork of the horizontal angle is eliminated"*. And the key now
///    sits on an **extra-long balance-rail pin** through the wedge rather than
///    balanced on the wedge edge, so it cannot tip over.
/// 2. **Key weight ratio** is taken with a **10 g** weight at the capstan, not
///    100 g: *"if the scale reads −5.5 the Key Weight Ratio (KR) would be 0.55"*.
///    The older 100 g method *"gave results accurate to three decimal places,
///    which is more accurate than needed"*.
/// 3. **Negative front weight is a measured value, not an impossibility.** A
///    dummy weight is tared on the pan, then placed on top of the key to hold it
///    down, and *"the Front Weight (FW) is read directly off the scale as a
///    negative value"*. An unleaded treble key really is back-heavy, and the
///    protocol exists because that case *"sooner or later occurs"*. [V]
///
/// # One distinction that will bite if it is not named
///
/// The glossary defines front weight as what *"the level key, **tipped on its
/// balance-pin point**, exerts at the measuring point"*. On the jig the pivot is
/// the **pin**. In the assembled action the key bears on the **punching**, whose
/// effective bearing point is offset from the pin and is itself adjustable —
/// Stanwood's own technique of cutting the balance rail punching moves it
/// deliberately. So the lever arm of the weighing jig and the lever arm in the
/// piano are not the same number, and [`KeyWeighing`] keeps them apart.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyWeighing {
    /// Static moment of the key about the **balance pin**, kg·m, positive
    /// forwards. This is the jig's pivot.
    pub static_moment_about_pin: f64,
    /// Balance pin to measuring point, metres.
    pub measuring_arm: f64,
    /// Balance pin to capstan, metres, as the positive distance behind the pin.
    pub capstan_arm: f64,
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
/// measuring point through [`crate::parts::KeyAssembly::measuring_point`] —
/// front weight and the key weight ratio on the jig, D, U, BW and F, key
/// friction weight, and the weight the settle solve puts on the key — so no
/// two of them can be weighed at different points.
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
}

impl Default for TouchweightProtocol {
    /// Stanwood's: 13 mm back from the front lip.
    fn default() -> Self {
        Self {
            measuring_point_from_lip: MEASURING_POINT_FROM_LIP,
        }
    }
}

impl KeyWeighing {
    /// **What the jig's scale reads** when `load_g` grams sit at `at` metres
    /// along the levelled key, with the scale tared, in grams.
    ///
    /// One statics routine, two protocols. The front weight and the key weight
    /// ratio are read on the *same* fixture — key level, pivoted on its
    /// balance-rail pin, front resting on a roller over the pan — and they
    /// differ only in what is put on the key and whether the tare has been
    /// pressed. Expressing both through this means they cannot drift apart:
    /// two independent formulas could, and the lever each one carries is the
    /// same lever.
    ///
    /// Moments about the pin, with a downward load at `at` and the roller
    /// pushing up at the measuring point: `load · at − R · arm = 0`. So the
    /// scale reads `load · at / arm`, which is negative for anything placed
    /// behind the pin — the front lifts off the roller, exactly as the
    /// protocol's worked example says.
    pub fn jig_reading_g(&self, load_g: f64, at: f64) -> f64 {
        load_g * at / self.measuring_arm
    }

    /// Front weight in **grams**, signed.
    ///
    /// *"The amount of static weight, to the nearest 0.1 gram, that the level
    /// key, tipped on its balance-pin point, exerts at the measuring point."*
    /// Negative when the keystick is back-heavy, which the protocol provides for.
    pub fn front_weight_g(&self) -> f64 {
        // The key's own weight, read on the jig before the tare: its whole
        // mass acting at its centre of gravity. Routed through the same
        // reading as everything else rather than written out again.
        self.jig_reading_g(
            1000.0 * self.static_moment_about_pin / self.measuring_arm,
            self.measuring_arm,
        )
    }

    /// Key weight ratio, dimensionless, force convention.
    ///
    /// *"The ratio of downward force on the capstan/heel versus the
    /// corresponding upward force at the measuring point as translated through
    /// the key."* Taring the scale first removes the key's own weight, so what
    /// is left is a pure lever ratio — which is why this depends on the two arms
    /// and on nothing else, and why a level-0 test can check it with no masses.
    pub fn key_ratio(&self) -> f64 {
        // Run the protocol rather than quote the arms: tare, put the standard
        // load on the capstan, read the scale, divide. The capstan is behind
        // the pin, so its coordinate is negative and the reading comes out
        // negative; the ratio is its magnitude over the load.
        -self.jig_reading_g(KR_PROTOCOL_LOAD_G, -self.capstan_arm) / KR_PROTOCOL_LOAD_G
    }

    /// What the tared scale reads with the protocol's 10 g at the capstan, grams.
    ///
    /// Negative, as the worked example in the source is: the load pushes the back
    /// of the key down, so the front lifts off the roller. Reproducing the sign
    /// is how a reader checks the model is answering the question the jig asks.
    pub fn tared_scale_reading_g(&self) -> f64 {
        self.jig_reading_g(KR_PROTOCOL_LOAD_G, -self.capstan_arm)
    }
}

/// The weight the current protocol places at the capstan to read the key ratio,
/// grams. Ten, since February 2000; a hundred before that. [V, Stanwood 2000,
/// p. 22; `data/params/touchweight-protocol.json`]
pub const KR_PROTOCOL_LOAD_G: f64 = 10.0;

/// The two branches of a weighing, and what the trade reads off them.
///
/// `down` and `up` are in grams at the measuring point. Everything else is
/// derived, by the definitions of the 2000 glossary and by nothing else.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Branches {
    /// Down weight: *"the minimum amount of weight, to the nearest gram, placed
    /// on the measuring point that causes the key to drop"*, grams.
    pub down: f64,
    /// Up weight: *"the maximum amount of weight that the key can lift"*, grams.
    pub up: f64,
}

impl Branches {
    /// Balance weight, `(D + U) / 2`, grams.
    pub fn balance_weight(&self) -> f64 {
        0.5 * (self.down + self.up)
    }

    /// Friction weight, `(D − U) / 2`, grams.
    pub fn friction_weight(&self) -> f64 {
        0.5 * (self.down - self.up)
    }
}

/// How the friction at the key's centres is parameterized.
///
/// # Why this follows the measurement rather than the mechanism
///
/// It is tempting to write `mu * radius * load` and feel physical. But what a
/// technician can actually obtain is **Key Friction Weight**, which the 2000
/// metrology defines with its own protocol: temporary weights are slid along the
/// key until front weight reads **zero**, the key goes back on its frame with the
/// stack removed, and the minimum weight at the measuring point that makes it
/// drop is read off. Zeroing the front weight first is what isolates friction
/// from balance.
///
/// So the measured quantity is a **torque bound at one load**, and that is the
/// default here. The load-proportional form is offered beside it, honestly
/// labelled, because the physics does depend on load and the protocol pins only
/// one point of that dependence. Choosing `Constant` is not a simplification of
/// the physics; it is a refusal to invent the part of it nobody has measured.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyPinFriction {
    /// A fixed torque bound, calibrated from Key Friction Weight.
    ///
    /// Stored as the **grams at the measuring point** the protocol reads, with
    /// the arm it was read at — never as a bare torque, because the trade's
    /// published friction figures are taken at incompatible radii.
    FromKeyFrictionWeight {
        /// Key friction weight, grams at the measuring point.
        kf_g: f64,
        /// The measuring arm it was read at, metres.
        arm: f64,
    },
    /// A bound proportional to the load carried at the bearing: `mu_r * N`.
    ///
    /// `mu_r` lumps the coefficient and the effective radius, because the two
    /// cannot be separated by any measurement available here and pretending
    /// otherwise would put a fabricated number into the sensitivities.
    LoadProportional {
        /// Product of the friction coefficient and the effective radius, metres.
        mu_r: f64,
    },
}

impl KeyPinFriction {
    /// The friction torque bound, as a **mass-moment** in kg·m — that is, the
    /// torque divided by `g`, which is the unit everything else here is in.
    ///
    /// `bearing_load` is the total mass bearing at the centre, kg.
    pub fn bound(&self, bearing_load: f64) -> f64 {
        match *self {
            Self::FromKeyFrictionWeight { kf_g, arm } => kf_g * 1e-3 * arm,
            Self::LoadProportional { mu_r } => mu_r * bearing_load.max(0.0),
        }
    }

    /// Scales the friction by `k`, keeping everything else fixed.
    ///
    /// Exists for the non-confusion test the trade would run: multiplying
    /// friction must move `F` and leave `BW` alone.
    pub fn scaled(&self, k: f64) -> Self {
        match *self {
            Self::FromKeyFrictionWeight { kf_g, arm } => Self::FromKeyFrictionWeight {
                kf_g: kf_g * k,
                arm,
            },
            Self::LoadProportional { mu_r } => Self::LoadProportional { mu_r: mu_r * k },
        }
    }
}

/// Static weighing of a key on its frame, with the stack removed.
///
/// This is the whole of stage 1a's physics. One degree of freedom and one
/// set-valued friction torque, so the quasi-static problem that is a *pair of
/// linear programs* in general degenerates here to its two closed-form extremes —
/// which is exactly why the key is the right place to start: the answer can be
/// checked by hand.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyOnFrame {
    /// Total mass of the key including its leads, kg.
    pub mass: f64,
    /// Static moment about the **effective bearing**, kg·m, positive forwards.
    ///
    /// The bearing, not the pin: in the assembled action the key rests on the
    /// punching, and where that bears is adjustable.
    pub static_moment: f64,
    /// Effective bearing to measuring point, metres.
    pub measuring_arm: f64,
    /// Friction at the centres.
    pub friction: KeyPinFriction,
}

impl KeyOnFrame {
    /// The two branches, in grams at the measuring point.
    ///
    /// Signs follow the trade: a weight that must be *added* to make the key
    /// drop is positive. A front-heavy key therefore yields negative branches —
    /// it falls on its own, and you would have to hold it up. That is physical
    /// and the type does not hide it.
    ///
    /// `None` when the lever arm is non-positive, which the user can reach by
    /// editing the geometry.
    pub fn branches(&self) -> Option<Branches> {
        let d = self.measuring_arm;
        // Written as a positive predicate on purpose: `d` may be NaN after a
        // geometry edit, and a NaN lever arm must be rejected here rather than
        // reach the state buffer and present as a physics regression.
        let arm_is_usable = d.is_finite() && d > 0.0;
        if !arm_is_usable || !self.mass.is_finite() || !self.static_moment.is_finite() {
            return None;
        }

        match self.friction {
            // A constant bound does not depend on the applied weight, so the two
            // branches are explicit.
            KeyPinFriction::FromKeyFrictionWeight { .. } => {
                let b = self.friction.bound(self.mass);
                Some(Branches {
                    down: 1000.0 * (b - self.static_moment) / d,
                    up: 1000.0 * (-b - self.static_moment) / d,
                })
            }
            // A load-proportional bound makes each branch implicit in its own
            // applied weight, because that weight is part of the load. Both are
            // still closed-form: solving `W d + M -/+ mu_r (m + W) = 0` for `W`
            // moves the `mu_r W` term into the denominator.
            KeyPinFriction::LoadProportional { mu_r } => {
                if mu_r >= d {
                    return None; // the key would be locked by its own bushings
                }
                Some(Branches {
                    down: 1000.0 * (mu_r * self.mass - self.static_moment) / (d - mu_r),
                    up: 1000.0 * (-mu_r * self.mass - self.static_moment) / (d + mu_r),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parts::Lead;

    /// The default protocol's measuring point on `design`, metres from the
    /// balance pin: the arm every jig reading below is taken on.
    fn arm(design: &KeyDesign) -> f64 {
        TouchweightProtocol::default().measuring_point(design)
    }

    /// Builds the weighing of the default key, pivoting on the balance pin.
    fn default_key_on_frame(friction: KeyPinFriction) -> (KeyDesign, KeyOnFrame) {
        let design = KeyDesign::default();
        let props = design
            .mass_properties()
            .expect("the default key is not degenerate");
        let on_frame = KeyOnFrame {
            mass: props.mass,
            static_moment: props.static_moment_x(),
            measuring_arm: arm(&design),
            friction,
        };
        (design, on_frame)
    }

    fn kf(kf_g: f64) -> KeyPinFriction {
        KeyPinFriction::FromKeyFrictionWeight {
            kf_g,
            arm: arm(&KeyDesign::default()),
        }
    }

    /// Level 0: the key ratio is the ratio of the two arms, and nothing else.
    ///
    /// No masses appear, because the protocol tares them away. The expected
    /// value is worked from the default key's dimensions, not read off the
    /// code: the capstan 129.5 mm behind the balance pin, and the measuring
    /// point 13 mm back from the covering's lip, which stands 2 mm proud of
    /// the key front at 258 mm — so 247 mm in front. It must land inside the
    /// published 0.50–0.60 band for a natural.
    #[test]
    fn key_ratio_is_the_arm_ratio_and_lands_in_the_published_band() {
        let d = KeyDesign::default();
        let w = KeyWeighing {
            static_moment_about_pin: 0.0,
            measuring_arm: arm(&d),
            capstan_arm: -d.capstan,
        };

        let expected = 0.1295 / (0.258 + 0.002 - 0.013);
        assert!(
            (w.key_ratio() - expected).abs() < 1e-12,
            "key ratio {} is not the arm ratio {expected}",
            w.key_ratio()
        );
        assert!(
            (0.50..=0.60).contains(&w.key_ratio()),
            "key ratio {} is outside the published band for a natural",
            w.key_ratio()
        );

        // And the jig must read what the protocol's worked example reads: a
        // negative number whose magnitude, over the 10 g load, is the ratio.
        let reading = w.tared_scale_reading_g();
        assert!(reading < 0.0, "the front lifts off the roller: {reading}");
        assert!((reading / -KR_PROTOCOL_LOAD_G - w.key_ratio()).abs() < 1e-12);
    }

    /// Level 0: a mass-free key ratio really is mass-free.
    #[test]
    fn loading_the_key_does_not_change_its_ratio() {
        let d = KeyDesign::default();
        let light = KeyWeighing {
            static_moment_about_pin: 0.0,
            measuring_arm: arm(&d),
            capstan_arm: -d.capstan,
        };
        let heavy = KeyWeighing {
            static_moment_about_pin: 0.040,
            ..light
        };
        assert!((light.key_ratio() - heavy.key_ratio()).abs() < 1e-15);
    }

    /// Level 4: with no action on the key, the equation of balance closes on
    /// zero.
    ///
    /// `BW + FW = (WW × KR) + (SW × R)`, and on a bare key the right-hand side is
    /// zero because there is no wippen and no hammer. So the balance weight must
    /// be exactly the negative of the front weight. This is the whole equation
    /// checked at the only point where its answer is known in advance, and it
    /// catches a sign error that no amount of plausible-looking arithmetic would.
    #[test]
    fn the_equation_of_balance_closes_on_a_bare_key() {
        let (design, on_frame) = default_key_on_frame(kf(6.0));
        let props = design.mass_properties().unwrap();

        let weighing = KeyWeighing {
            static_moment_about_pin: props.static_moment_x(),
            measuring_arm: arm(&design),
            capstan_arm: -design.capstan,
        };
        let branches = on_frame.branches().expect("the default key weighs");

        assert!(
            (branches.balance_weight() + weighing.front_weight_g()).abs() < 1e-9,
            "BW {} + FW {} must be zero with no action on the key",
            branches.balance_weight(),
            weighing.front_weight_g()
        );
    }

    /// Level 0, first non-confusion test: **one gram of lead moves the balance
    /// weight and leaves the friction weight alone.**
    ///
    /// A technician will check this before anything else, and a developer never
    /// would. Both friction models are exercised: a constant bound must be
    /// *exactly* untouched, and a load-proportional one must move by far less
    /// than the 1 g resolution the protocol reads friction to — the extra gram
    /// does add load to the bushings, and the honest claim is that the effect is
    /// negligible, not that it is zero.
    #[test]
    fn one_gram_of_lead_moves_balance_weight_without_touching_friction() {
        for friction in [kf(6.0), KeyPinFriction::LoadProportional { mu_r: 3.2e-4 }] {
            let (design, bare) = default_key_on_frame(friction);
            let mut leaded_design = design.clone();
            leaded_design.leads = vec![Lead::at(0.150, 0.001)];
            let props = leaded_design.mass_properties().unwrap();
            let leaded = KeyOnFrame {
                mass: props.mass,
                static_moment: props.static_moment_x(),
                ..bare
            };

            let (b0, b1) = (bare.branches().unwrap(), leaded.branches().unwrap());

            // 1 g at 150 mm, referred to the 247 mm arm, is 0.61 g of balance
            // weight.
            let moved = b0.balance_weight() - b1.balance_weight();
            assert!(
                (moved - 0.150 / arm(&design)).abs() < 1e-6,
                "1 g of lead at 150 mm moved BW by {moved} g, expected {}",
                0.150 / arm(&design)
            );

            let friction_drift = (b1.friction_weight() - b0.friction_weight()).abs();
            assert!(
                friction_drift < 0.01,
                "adding lead moved the friction weight by {friction_drift} g, \
                 which a technician would call a broken model"
            );
        }
    }

    /// Level 0, second non-confusion test: **multiplying the friction by 1.2
    /// moves the friction weight and leaves the balance weight alone.**
    #[test]
    fn scaling_friction_moves_friction_weight_without_touching_balance_weight() {
        for friction in [kf(6.0), KeyPinFriction::LoadProportional { mu_r: 3.2e-4 }] {
            let (_, base) = default_key_on_frame(friction);
            let scaled = KeyOnFrame {
                friction: friction.scaled(1.2),
                ..base
            };

            let (b0, b1) = (base.branches().unwrap(), scaled.branches().unwrap());

            assert!(
                (b1.friction_weight() / b0.friction_weight() - 1.2).abs() < 1e-3,
                "friction weight went from {} to {}, which is not a factor 1.2",
                b0.friction_weight(),
                b1.friction_weight()
            );
            assert!(
                (b1.balance_weight() - b0.balance_weight()).abs() < 0.01,
                "scaling friction moved the balance weight by {} g",
                b1.balance_weight() - b0.balance_weight()
            );
        }
    }

    /// The branches are ordered, and their gap is twice the friction.
    #[test]
    fn down_weight_exceeds_up_weight_by_twice_the_friction() {
        let (_, k) = default_key_on_frame(kf(6.0));
        let b = k.branches().unwrap();
        assert!(b.down > b.up, "down {} must exceed up {}", b.down, b.up);
        assert!(
            (b.friction_weight() - 6.0).abs() < 1e-9,
            "F = {}",
            b.friction_weight()
        );
    }

    /// Level 4: **front weight is read 13 mm back from the covering's lip, and
    /// the datum is load-bearing.**
    ///
    /// Not a numerical probe at a convenient place: the datum the 2000 glossary
    /// fixes, *"on the top of the key 13mm or 1/2" back from the front lip of
    /// the key"*, the lip taken as the covering's. Three things are checked. The
    /// arm really is the lip less 13 mm, the lip standing `keytop_overhang`
    /// proud of the key front; moving the datum changes the number by the
    /// lever ratio, which is what makes it a *protocol* rather than a
    /// convention — read a weight somewhere else and you have a different
    /// weight; and the two readings of "front lip", the covering's and the key
    /// front beneath it, are 2 mm of arm apart.
    #[test]
    fn front_weight_is_read_at_the_protocol_datum_and_the_datum_matters() {
        let d = KeyDesign::default();
        let props = d.mass_properties().unwrap();
        let lip = d.front_edge + d.keytop_overhang;

        assert!(
            (arm(&d) - (lip - 0.013)).abs() < 1e-15,
            "the measuring arm must be the covering's lip less 13 mm"
        );

        let at = |lever: f64| {
            KeyWeighing {
                static_moment_about_pin: props.static_moment_x(),
                measuring_arm: lever,
                capstan_arm: -d.capstan,
            }
            .front_weight_g()
        };

        let protocol = at(arm(&d));
        // The same key read at the very lip instead: a longer arm, so a
        // smaller weight, in exact inverse proportion.
        let at_lip = at(lip);
        assert!(
            at_lip.abs() < protocol.abs(),
            "a longer arm must read a smaller weight: {protocol} -> {at_lip}"
        );
        assert!(
            (protocol * arm(&d) - at_lip * lip).abs() < 1e-9,
            "front weight times its arm is the static moment, and must not depend \
             on where it was read"
        );

        // Measured from the key front instead, 13 mm stands 2 mm further back:
        // the same static moment over a 2 mm shorter arm, 0.8 % heavier.
        let from_key_front = at(d.front_edge - 0.013);
        assert!(
            (from_key_front / protocol - arm(&d) / (arm(&d) - d.keytop_overhang)).abs() < 1e-12,
            "the key front reads {from_key_front} g against {protocol} g at the lip"
        );

        // And the half-inch alternative the same glossary offers differs by
        // 0.3 mm of arm, which is below the 0.1 g the scale resolves.
        let half_inch = at(lip - MEASURING_POINT_HALF_INCH);
        assert!(
            (half_inch - protocol).abs() < 0.1,
            "13 mm and 1/2 inch must agree inside the scale's resolution: \
             {protocol} vs {half_inch}"
        );
    }

    /// Level 4: the key-ratio protocol reproduces the source's worked example.
    ///
    /// *"If the scale reads −5.5 the Key Weight Ratio (KR) would be 0.55."*
    /// Building a key whose arms give 0.55 and checking the jig reads −5.5 is
    /// the whole protocol, run in reverse, on a number somebody else published.
    #[test]
    fn the_key_ratio_jig_reproduces_the_published_worked_example() {
        let w = KeyWeighing {
            static_moment_about_pin: 0.031,
            measuring_arm: 0.240,
            capstan_arm: 0.240 * 0.55,
        };
        assert!((w.key_ratio() - 0.55).abs() < 1e-12);
        assert!(
            (w.tared_scale_reading_g() + 5.5).abs() < 1e-12,
            "the jig must read −5.5 g, got {}",
            w.tared_scale_reading_g()
        );
    }

    /// Level 4: **one fixture, two protocols, and they agree with each other.**
    ///
    /// Front weight and key weight ratio are read on the same jig, and the
    /// only thing that separates them is what is put on the key. So putting
    /// the key's own weight through the key-ratio reading must give the front
    /// weight, and putting the standard load at the capstan through the
    /// front-weight route must give the ratio. Two formulas could satisfy
    /// their own tests and still disagree about the lever they share; one
    /// routine cannot.
    #[test]
    fn the_two_jig_protocols_are_the_same_fixture() {
        let d = KeyDesign::default();
        let props = d.mass_properties().unwrap();
        let w = KeyWeighing {
            static_moment_about_pin: props.static_moment_x(),
            measuring_arm: arm(&d),
            capstan_arm: -d.capstan,
        };

        // A load equal to the front weight, placed at the measuring point,
        // reads exactly the front weight: that is what "front weight" means.
        assert!(
            (w.jig_reading_g(w.front_weight_g(), w.measuring_arm) - w.front_weight_g()).abs()
                < 1e-9
        );

        // And the ratio is the reading at the capstan over the load, whichever
        // load is used — it is a ratio, so the standard 10 g is a convention
        // and not a dependency.
        for load in [1.0, 10.0, 100.0] {
            let ratio = -w.jig_reading_g(load, -w.capstan_arm) / load;
            assert!(
                (ratio - w.key_ratio()).abs() < 1e-12,
                "with {load} g at the capstan the ratio read {ratio}"
            );
        }

        // Anything placed *at* the pin reads nothing, because it has no arm.
        assert!(w.jig_reading_g(50.0, 0.0).abs() < 1e-15);
        // And anything behind it lifts the front off the roller.
        assert!(w.jig_reading_g(50.0, -0.050) < 0.0);
    }

    /// A degenerate geometry is an error, not a `NaN`.
    #[test]
    fn a_non_positive_measuring_arm_is_rejected() {
        let (_, k) = default_key_on_frame(kf(6.0));
        assert!(KeyOnFrame {
            measuring_arm: 0.0,
            ..k
        }
        .branches()
        .is_none());
        assert!(
            KeyOnFrame {
                friction: KeyPinFriction::LoadProportional { mu_r: 1.0 },
                ..k
            }
            .branches()
            .is_none(),
            "bushings that grip harder than the lever arm would lock the key"
        );
    }

    /// A plausible well-regulated note, with the force convention for KR.
    fn sample() -> Touchweight {
        Touchweight {
            downweight: 50.0,
            upweight: 20.0,
            front_weight: 21.5,
            wippen_weight: 14.6,
            strike_weight: 8.86,
            key_ratio: 0.554,
        }
    }

    #[test]
    fn balance_and_friction_split_the_interval() {
        let t = sample();
        assert!((t.balance_weight() - 35.0).abs() < 1e-12);
        assert!((t.friction_weight() - 15.0).abs() < 1e-12);
    }

    #[test]
    fn the_wippen_lever_reduces_it_does_not_amplify() {
        let t = sample();
        assert!(t.wippen_balance_weight() < t.wippen_weight);
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
            strike_weight: 0.0,
            ..sample()
        };
        assert!(t.strike_ratio().is_none());
    }
}

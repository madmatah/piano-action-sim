//! Stanwood's front weight jig, run on the model.
//!
//! # The fixture, and what the scale reads
//!
//! On the jig the key sits on an extra-long balance pin through a wedge, set
//! level by a bubble level on its covering, its front resting on a roller on
//! the pan of a scale directly below the measuring point [V, Stanwood 2000,
//! p. 21; glossary, *Measuring Point*, p. 23]. On the model that is
//! [`crate::parts::mounts::jig`]: the key on a revolute joint at its balance
//! pin's axis, without friction, its angle prescribed level. Holding it level
//! takes a generalized force on the angle, the prescription's reaction, and on
//! the real jig the scale is what supplies it: pushing up by `R` under the
//! measuring point, at an arm `x_s` from the pin, it applies `R · x_s` to the
//! angle. So the scale reads the reaction over `g · x_s`, in grams.
//!
//! The reading is signed. A back-heavy key reads negative, as Stanwood's
//! protocol for a negative front weight reads it: his dummy weight is a tare
//! on a pan that can only push, and it cancels exactly.
//!
//! # Two protocols, one reading
//!
//! Front weight is the reading with nothing on the key. The key weight ratio
//! is read tared: the scale zeroed with the key on it, 10 g centered on the
//! capstan/heel contact point, and the reading over the load with its sign
//! turned, so that a scale reading −5.5 gives 0.55. Both are
//! [`scale_reading_g`], so the lever they share is taken once. On a key
//! carrying its wippen, the contact point is where its capstan touches the
//! heel at the top seat, which the sheet reads; elsewhere the capstan's top,
//! on its axis.
//!
//! # What the jig cannot see
//!
//! It turns the key about its balance pin, so it cannot see where the key
//! bears on its balance punching: only a reading on the frame can
//! (`docs/FOUNDATIONS.md` > Validation instruments, "One real key, measured by the owner"). And its pivot turns without friction, which
//! is an assumption: the real key turns on a pin through its bushings.

use super::TouchweightProtocol;
use crate::geometry::Point;
use crate::mechanism::{
    Body, BodyId, Coord, CoordId, Joint, MassProps, MaterialPoint, Mechanism, PointLoad,
    Prescribed, Problem,
};
use crate::parts::mounts::{self, Mounted};
use crate::parts::KeyDesign;
use crate::G;

/// Why a jig reading cannot be taken.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JigRefusal {
    /// Front weight is the key's own weight read at the measuring point, and
    /// this key has no mass in the model: nobody has weighed it.
    KeyNotWeighed,
    /// The scale stands at or behind the pivot the jig holds the body level
    /// about, where pushing up cannot hold it level: a measuring point not in
    /// front of the balance pin, or a roller not in front of the wippen's
    /// center pin.
    ScaleNotInFrontOfThePivot,
    /// The mechanism could not be held as the jig holds it.
    Mechanism(Problem),
}

/// What a scale under `scale` reads, grams, with `mechanism` held by `held`
/// under `loads`.
///
/// `pivot` is the coordinate the scale stands in for. The reading is that
/// coordinate's reaction over `G` times the scale's arm, the generalized force
/// a newton of upward push at `scale` applies to `pivot`. Every coordinate of
/// `mechanism` must be prescribed ([`Mechanism::hold`]).
pub fn scale_reading_g(
    mechanism: &Mechanism,
    held: &[Prescribed],
    pivot: CoordId,
    scale: MaterialPoint,
    loads: &[PointLoad],
) -> Result<f64, JigRefusal> {
    let state = mechanism.hold(held, loads).map_err(JigRefusal::Mechanism)?;
    let arm = mechanism.jacobian_column(&state.pose, scale, pivot).y;
    if !(arm.is_finite() && arm > 0.0) {
        return Err(JigRefusal::ScaleNotInFrontOfThePivot);
    }
    Ok(1000.0 * state.reaction(pivot) / (G * arm))
}

/// A key on the jig, with the two points of it the protocols read.
///
/// # Which paper, and why it matters
///
/// The metrology was published in 1996 and its **protocols were superseded in
/// February 2000** by *"Standard Protocols of the New Touchweight
/// Metrology"*, which this jig follows. Three of the changes bear on the key,
/// and the 1996 text gets two of them wrong:
///
/// 1. **Front weight** is read with the key set level **with a bubble
///    level**, not merely "in a horizontal attitude similar to when the key is
///    at rest": *"the guesswork of the horizontal angle is eliminated"*. And
///    the key sits on an **extra-long balance-rail pin** through the wedge
///    rather than balanced on the wedge's edge, so it cannot tip over.
/// 2. **Key weight ratio** is taken with a **10 g** weight at the capstan, not
///    100 g: the older method *"gave results accurate to three decimal places,
///    which is more accurate than needed"*.
/// 3. **Negative front weight is a measured value, not an impossibility.** A
///    dummy weight is tared on the pan, then placed on top of the key to hold
///    it down, and *"the Front Weight (FW) is read directly off the scale as a
///    negative value"*. An unleaded treble key really is back-heavy, and the
///    protocol exists because that case *"sooner or later occurs"*. [V,
///    Stanwood 2000]
///
/// # The pin, not the punching
///
/// Front weight is what *"the level key, **tipped on its balance-pin
/// point**, exerts at the measuring point"*. In the assembled action the key
/// bears on the balance punching instead, whose bearing point is adjustable:
/// Stanwood's technique of cutting the punching moves it on purpose. So the
/// jig's arms are not the arms the key has on its frame, and a reading on the
/// frame is the only one that sees the bearing.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyJig {
    mount: Mounted,
    /// The measuring point, in the key's frame: the scale stands below it.
    measuring_point: Point,
    /// The capstan's contact point, in the key's frame: the top of its
    /// head, on its axis, where the key weight ratio is read on a key that
    /// carries no wippen, and on one whose capstan is not read touching its
    /// heel ([`Self::key_ratio_at`]).
    capstan: Point,
    /// The load the key weight ratio is read with, grams: the protocol's.
    key_ratio_load_g: f64,
}

impl KeyJig {
    /// The angle that holds the key level, radians: the covering's top
    /// horizontal, where the bubble level sits. The covering's top runs
    /// parallel to the underside's flat, the `x` axis of the key's frame, so
    /// level is zero.
    pub const LEVEL: f64 = 0.0;

    /// `key` on the jig, read at `protocol`'s measuring point.
    ///
    /// `mass` is the key's mass and center of mass in its frame, or `None`
    /// for a key nobody has weighed: its front weight is then refused, and
    /// its key weight ratio, from which the tare takes the key's own weight,
    /// is still read.
    pub fn new(
        key: &KeyDesign,
        protocol: &TouchweightProtocol,
        mass: Option<MassProps>,
    ) -> Result<Self, Problem> {
        Ok(Self {
            mount: mounts::jig(mass)?,
            measuring_point: protocol.measuring_point_on_key(key),
            capstan: key.capstan_contact(),
            key_ratio_load_g: protocol.key_ratio_load_g,
        })
    }

    /// The capstan's contact point, in the key's frame: the top of its head,
    /// on its axis.
    pub fn capstan_contact(&self) -> Point {
        self.capstan
    }

    /// A weight of `mass_g` grams centered on `at`, a point of the key in its
    /// frame.
    pub fn weight_on_the_key(&self, at: Point, mass_g: f64) -> PointLoad {
        let at = MaterialPoint {
            body: self.mount.key,
            at,
        };
        PointLoad::weight(at, mass_g / 1000.0)
    }

    /// A weight of `mass_g` grams centered on the capstan's contact point.
    pub fn weight_on_the_capstan(&self, mass_g: f64) -> PointLoad {
        self.weight_on_the_key(self.capstan, mass_g)
    }

    /// What the scale reads with `loads` on the key, grams.
    pub fn reading_g(&self, loads: &[PointLoad]) -> Result<f64, JigRefusal> {
        let level = [Prescribed {
            coord: self.mount.angle,
            value: Self::LEVEL,
        }];
        let scale = MaterialPoint {
            body: self.mount.key,
            at: self.measuring_point,
        };
        scale_reading_g(
            &self.mount.mechanism,
            &level,
            self.mount.angle,
            scale,
            loads,
        )
    }

    /// What the scale reads with `loads` on the key, having been tared with
    /// the key alone on it, grams.
    pub fn tared_reading_g(&self, loads: &[PointLoad]) -> Result<f64, JigRefusal> {
        Ok(self.reading_g(loads)? - self.reading_g(&[])?)
    }

    /// **Front weight**, grams, signed: *"the amount of static weight, to the
    /// nearest 0.1 gram, that the level key, tipped on its balance-pin point,
    /// exerts at the measuring point"* [V, Stanwood 2000, glossary, p. 22]. The
    /// scale's reading with nothing on the key; negative for a back-heavy key.
    pub fn front_weight_g(&self) -> Result<f64, JigRefusal> {
        if self.mount.mechanism.body(self.mount.key).mass.is_none() {
            return Err(JigRefusal::KeyNotWeighed);
        }
        self.reading_g(&[])
    }

    /// What the tared scale reads with the protocol's 10 g on the capstan's
    /// contact point, grams. Negative, as the source's worked example reads
    /// it: the load is behind the pin, and the front lifts off the roller.
    pub fn key_ratio_reading_g(&self) -> Result<f64, JigRefusal> {
        self.key_ratio_reading_g_at(self.capstan)
    }

    /// What the tared scale reads with the protocol's 10 g centered on
    /// `at`, a point of the key in its frame, grams: where the capstan
    /// touches the wippen's heel, on a key that carries one.
    pub fn key_ratio_reading_g_at(&self, at: Point) -> Result<f64, JigRefusal> {
        self.tared_reading_g(&[self.weight_on_the_key(at, self.key_ratio_load_g)])
    }

    /// **Key weight ratio**, force convention, dimensionless: the tared
    /// reading with 10 g on the capstan, over the load, its sign turned.
    /// *"If the scale reads −5.5 the Key Weight Ratio (KR) would be 0.55"*
    /// [V, Stanwood 2000, p. 22].
    pub fn key_ratio(&self) -> Result<f64, JigRefusal> {
        self.key_ratio_at(self.capstan)
    }

    /// **Key weight ratio** with the 10 g centered on `at`, a point of the
    /// key in its frame: *"on a vertical axis which passes through the
    /// capstan/heel contact point"* [V, Stanwood 2000, p. 22], which on a key
    /// carrying its wippen is where its capstan touches the heel at the top
    /// seat ([`crate::parts::mounts::Mounted::capstan_on_the_heel`]).
    pub fn key_ratio_at(&self, at: Point) -> Result<f64, JigRefusal> {
        Ok(-self.key_ratio_reading_g_at(at)? / self.key_ratio_load_g)
    }
}

/// A wippen assembly on the setup that weighs it: Stanwood's jig, the
/// key's, which weighs a wippen too.
///
/// *"The amount of weight, to the nearest 0.1 gram, of the level wippen,
/// pivoted without friction at the wippen center, and measured at the
/// capstan/heel contact point"* [V, Stanwood 2000, glossary, p. 23]. The 2000
/// protocols print no setup for it; the 1996 paper's rests the heel on the
/// roller at the capstan contact point and the flange, upright, on its felt
/// wedge [V, Stanwood 1996, *Wippen Weight*, Photo 2]
/// (`data/params/touchweight-protocol.json`, `wippen_weight`). On the model it
/// is two bodies: the ground, and the wippen assembly, its jack and its
/// repetition lever at rest on it, hung on a revolute joint at its center pin
/// without friction, its angle held level, and the scale under the roller
/// reading the reaction that holds it there over its arm, as the key's jig
/// reads front weight ([`scale_reading_g`]).
#[derive(Debug, Clone, PartialEq)]
pub struct WippenJig {
    mechanism: Mechanism,
}

impl WippenJig {
    /// The angle that holds the wippen level, radians: its long body
    /// horizontal, the level the owner chose for it. The wippen's frame has
    /// its `x` axis along the body's long top edge, within the trace's two
    /// pixels on Hirschkorn's (`tools/derivations/12-hirschkorn-wippen-outline.py`),
    /// so level is zero.
    pub const LEVEL: f64 = 0.0;

    const WIPPEN: BodyId = BodyId(1);
    const ANGLE: CoordId = CoordId(0);

    /// A wippen assembly of mass `mass`, its center of mass in its own
    /// frame, on the jig.
    pub fn new(mass: MassProps) -> Result<Self, Problem> {
        let wippen = Body {
            parent: BodyId::GROUND,
            joint: Joint::Revolute {
                pivot: Point::new(0.0, 0.0),
                coord: Self::ANGLE,
            },
            mass: Some(mass),
        };
        Ok(Self {
            mechanism: Mechanism::new(vec![Body::GROUND, wippen], vec![Coord::ANGLE])?,
        })
    }

    /// A weight of `mass_g` grams centered on `at`, a point of the wippen in
    /// its frame.
    pub fn weight_on_the_wippen(&self, at: Point, mass_g: f64) -> PointLoad {
        PointLoad::weight(
            MaterialPoint {
                body: Self::WIPPEN,
                at,
            },
            mass_g / 1000.0,
        )
    }

    /// What the scale reads with the roller under `roller`, a point of the
    /// wippen in its frame, and `loads` on the wippen, grams.
    pub fn reading_g(&self, roller: Point, loads: &[PointLoad]) -> Result<f64, JigRefusal> {
        let level = [Prescribed {
            coord: Self::ANGLE,
            value: Self::LEVEL,
        }];
        let scale = MaterialPoint {
            body: Self::WIPPEN,
            at: roller,
        };
        scale_reading_g(&self.mechanism, &level, Self::ANGLE, scale, loads)
    }

    /// **Wippen weight**, grams: what the scale reads with the roller under
    /// `roller` and nothing on the wippen, the assembly's static moment about
    /// its center pin over the roller's arm.
    pub fn wippen_weight_g(&self, roller: Point) -> Result<f64, JigRefusal> {
        self.reading_g(roller, &[])
    }
}

#[cfg(test)]
mod tests {
    use super::super::oracle::{KeyWeighing, WippenWeighing};
    use super::*;
    use crate::parts::KeyMass;
    use crate::test_fixture::{KEY_RATIO_LOAD_G, MEASURING_POINT_HALF_INCH};

    /// The key's mass as the fixture weighs it, the height of its center of
    /// mass its drawn section's centroid.
    fn mass_of(key: &KeyDesign) -> MassProps {
        crate::parts::fixtures::mass_of(key).expect("a key that is not degenerate")
    }

    /// The test key weighed whole: `mass_g` grams, and a front weight of
    /// `front_weight_g` grams read 13 mm back from its lip.
    fn weighed(mass_g: f64, front_weight_g: f64) -> MassProps {
        let key = crate::test_fixture::key();
        let height = key
            .section_centroid()
            .expect("the test key encloses wood")
            .y;
        KeyMass::weighed(&key, mass_g, front_weight_g, 0.013)
            .expect("a key's weighing")
            .props(height)
            .expect("a weighed key has a mass")
    }

    fn jig(key: &KeyDesign, protocol: &TouchweightProtocol) -> KeyJig {
        KeyJig::new(key, protocol, Some(mass_of(key))).expect("a key on the jig")
    }

    /// The closed form of a key of mass `mass` on the jig: static moment
    /// about the pin over the measuring arm, capstan arm over the same.
    fn oracle(key: &KeyDesign, protocol: &TouchweightProtocol, mass: MassProps) -> KeyWeighing {
        KeyWeighing {
            static_moment_about_pin: mass.mass * mass.com.x,
            measuring_arm: protocol.measuring_point(key),
            capstan_arm: -key.capstan,
        }
    }

    /// Whether two computations of one reading agree to round-off.
    ///
    /// Each path takes at most sixteen rounded operations, each within
    /// `u = ε/2` of the largest magnitude it touches, `magnitude`; so they
    /// agree within `32 u · magnitude`. A wrong arm, datum or sign is off by
    /// a part in a thousand or more.
    fn agree(a: f64, b: f64, magnitude: f64) -> bool {
        (a - b).abs() <= 16.0 * f64::EPSILON * magnitude
    }

    /// Keys that exercise every term, each with its mass: the test key, by
    /// the fixture's weighing and weighed heavy in front and heavy behind; a
    /// capstan moved, a key front brought in; and the measuring point at the
    /// lip, at the glossary's half inch and at 30 mm.
    fn keys() -> Vec<(KeyDesign, TouchweightProtocol, MassProps)> {
        let protocol = |from_lip: f64| TouchweightProtocol {
            measuring_point_from_lip: from_lip,
            ..crate::test_fixture::protocol()
        };
        let default = crate::test_fixture::key();
        let as_weighed = |key: KeyDesign, protocol| {
            let mass = mass_of(&key);
            (key, protocol, mass)
        };
        vec![
            as_weighed(default.clone(), crate::test_fixture::protocol()),
            (
                default.clone(),
                crate::test_fixture::protocol(),
                weighed(141.1, 21.0),
            ),
            (
                default.clone(),
                crate::test_fixture::protocol(),
                weighed(151.1, -8.1),
            ),
            as_weighed(
                KeyDesign {
                    capstan: -0.110,
                    ..crate::test_fixture::key()
                },
                crate::test_fixture::protocol(),
            ),
            as_weighed(
                KeyDesign {
                    front_edge: 0.240,
                    ..crate::test_fixture::key()
                },
                crate::test_fixture::protocol(),
            ),
            as_weighed(default.clone(), protocol(0.0)),
            as_weighed(default.clone(), protocol(MEASURING_POINT_HALF_INCH)),
            as_weighed(default, protocol(0.030)),
        ]
    }

    /// Consistency: **the jig run on the model reads what the closed form
    /// reads, to round-off**, front weight and key weight ratio alike, on
    /// keys front-heavy and back-heavy, with every datum moved.
    ///
    /// The closed form, `KeyWeighing`, is the hand calculation of the same
    /// two readings, kept as the tests' oracle. Agreement to round-off says
    /// the mechanism, its loads and the reaction carry the lever exactly as
    /// the hand calculation does.
    #[test]
    fn the_jig_reads_what_the_closed_form_reads() {
        for (key, protocol, mass) in keys() {
            let jig = KeyJig::new(&key, &protocol, Some(mass)).expect("a key on the jig");
            let oracle = oracle(&key, &protocol, mass);

            let fw = jig
                .front_weight_g()
                .expect("a weighed key has a front weight");
            assert!(
                agree(fw, oracle.front_weight_g(), fw.abs()),
                "front weight: jig {fw} g, closed form {} g",
                oracle.front_weight_g()
            );

            // The ratio is a difference of two readings: its round-off is
            // that of the larger of them, not of the difference.
            let tared = jig.key_ratio_reading_g().expect("a ratio");
            let with_load = jig
                .reading_g(&[jig.weight_on_the_capstan(KEY_RATIO_LOAD_G)])
                .unwrap();
            let magnitude = fw.abs().max(with_load.abs()).max(tared.abs());
            assert!(
                agree(tared, oracle.tared_scale_reading_g(), magnitude),
                "tared reading: jig {tared} g, closed form {} g",
                oracle.tared_scale_reading_g()
            );
            let kr = jig.key_ratio().unwrap();
            assert!(
                agree(kr, oracle.key_ratio(), magnitude / KEY_RATIO_LOAD_G),
                "key ratio: jig {kr}, closed form {}",
                oracle.key_ratio()
            );
        }
    }

    /// Level 4: the key ratio is the ratio of the two arms.
    ///
    /// Worked from the test key's dimensions, not read off the code: the
    /// capstan 122 mm behind the balance pin, the measuring point 13 mm back
    /// from the covering's lip, which stands 2 mm proud of the key front at
    /// 258 mm, so 247 mm in front: 0.494, a hundredth under the 0.50 to 0.60
    /// published for a natural, the key front being assumed where the
    /// capstan is measured, and inside the 0.3 to 1.0 that tells the force
    /// convention from its reciprocal (`Touchweight::conventions_are_sane`).
    #[test]
    fn the_key_ratio_is_the_arm_ratio() {
        let jig = jig(
            &crate::test_fixture::key(),
            &crate::test_fixture::protocol(),
        );
        let expected = 0.122 / (0.258 + 0.002 - 0.013);
        let kr = jig.key_ratio().unwrap();
        let fw = jig.front_weight_g().unwrap();
        assert!(
            agree(kr, expected, fw.abs() / KEY_RATIO_LOAD_G),
            "{kr} against {expected}"
        );
        assert!(
            (0.3..=1.0).contains(&kr),
            "key ratio {kr} outside the guard"
        );
        let reading = jig.key_ratio_reading_g().unwrap();
        assert!(reading < 0.0, "the front lifts off the roller: {reading}");
    }

    /// Level 4: the protocol reproduces the source's worked example on a key
    /// built to give it.
    ///
    /// *"If the scale reads −5.5 the Key Weight Ratio (KR) would be 0.55."* A
    /// key whose measuring point stands 240 mm in front of the pin, its
    /// capstan 132 mm behind it, has that ratio; the jig must read −5.5 g.
    #[test]
    fn the_jig_reproduces_the_published_worked_example() {
        let protocol = crate::test_fixture::protocol();
        let key = KeyDesign {
            // The lip 2 mm proud of the key front, the point 13 mm back from
            // the lip: 240 mm.
            front_edge: 0.240 + 0.013 - 0.002,
            capstan: -0.240 * 0.55,
            ..crate::test_fixture::key()
        };
        let jig = jig(&key, &protocol);
        let fw = jig.front_weight_g().unwrap().abs();
        let reading = jig.key_ratio_reading_g().unwrap();
        assert!(
            agree(reading, -5.5, fw),
            "the jig must read −5.5 g, read {reading}"
        );
        assert!(agree(jig.key_ratio().unwrap(), 0.55, fw / KEY_RATIO_LOAD_G));
    }

    /// Level 4: **front weight is read at the protocol's datum, and the datum
    /// matters.**
    ///
    /// Moved along the key, the measuring point reads the same static moment
    /// on another arm, in inverse proportion. At the very lip the arm is
    /// longest and the reading smallest; from the key front instead of the
    /// covering's lip, 13 mm stands 2 mm further back and the reading is 0.8 %
    /// heavier; the glossary's half inch reads within the scale's 0.1 g of
    /// 13 mm.
    #[test]
    fn front_weight_is_read_at_the_protocol_datum_and_the_datum_matters() {
        let key = crate::test_fixture::key();
        let at = |from_lip: f64| {
            let protocol = TouchweightProtocol {
                measuring_point_from_lip: from_lip,
                ..crate::test_fixture::protocol()
            };
            let arm = protocol.measuring_point(&key);
            (jig(&key, &protocol).front_weight_g().unwrap(), arm)
        };
        let (standard, arm) = at(0.013);
        let lip = key.lip();
        assert!((arm - (lip - 0.013)).abs() < 1e-15);

        let (at_lip, lip_arm) = at(0.0);
        assert!(at_lip.abs() < standard.abs(), "a longer arm reads less");
        assert!(
            agree(standard * arm, at_lip * lip_arm, standard.abs()),
            "front weight times its arm is the static moment wherever it is read"
        );

        let (from_key_front, _) = at(0.013 + key.keytop_overhang);
        let ratio = arm / (arm - key.keytop_overhang);
        assert!(
            agree(from_key_front / standard, ratio, ratio),
            "13 mm from the key front reads {from_key_front} g against {standard} g"
        );

        let (half_inch, _) = at(MEASURING_POINT_HALF_INCH);
        assert!(
            (half_inch - standard).abs() < 0.1,
            "{half_inch} against {standard}"
        );
    }

    /// Level 4: **one fixture, two protocols.** A weight equal to the front
    /// weight on the measuring point of the tared jig reads the front weight;
    /// the ratio is the same whatever load reads it; a weight on the pin
    /// reads nothing, and one behind it lifts the front off the roller.
    #[test]
    fn front_weight_and_key_ratio_are_one_reading() {
        let protocol = crate::test_fixture::protocol();
        let key = crate::test_fixture::key();
        let jig = jig(&key, &protocol);
        let fw = jig.front_weight_g().unwrap();

        let measuring_point = protocol.measuring_point_on_key(&key);
        let same = jig
            .tared_reading_g(&[jig.weight_on_the_key(measuring_point, fw)])
            .unwrap();
        assert!(agree(same, fw, fw.abs()), "{same} against {fw}");

        let kr = jig.key_ratio().unwrap();
        for load in [1.0, 10.0, 100.0] {
            let reading = jig
                .tared_reading_g(&[jig.weight_on_the_capstan(load)])
                .unwrap();
            let magnitude = (fw.abs() + load) / load;
            assert!(
                agree(-reading / load, kr, magnitude),
                "{load} g at the capstan gives {} against {kr}",
                -reading / load
            );
        }

        let on_the_pin = jig
            .tared_reading_g(&[jig.weight_on_the_key(Point::new(0.0, 0.03), 50.0)])
            .unwrap();
        assert!(agree(on_the_pin, 0.0, fw.abs()), "{on_the_pin}");
        let behind = jig
            .tared_reading_g(&[jig.weight_on_the_key(Point::new(-0.050, 0.03), 50.0)])
            .unwrap();
        assert!(behind < 0.0, "{behind}");
    }

    /// Consistency: **the key ratio at a point is its arm over the
    /// measuring arm**, and the key ratio the bare key reads is the one at
    /// the capstan's top, to the bit: the same operations, the same point.
    /// A point 1.3 mm behind the capstan's axis and 3 mm below its top, as
    /// the heel touches a flat capstan, reads the ratio of its own arm,
    /// within the two readings' round-off.
    #[test]
    fn the_key_ratio_at_a_point_is_its_arm_over_the_measuring_arm() {
        const BEHIND_THE_AXIS: f64 = 0.0013;
        const BELOW_THE_TOP: f64 = 0.003;
        let protocol = crate::test_fixture::protocol();
        let key = crate::test_fixture::key();
        let jig = jig(&key, &protocol);
        let top = key.capstan_contact();
        assert_eq!(
            jig.key_ratio().unwrap().to_bits(),
            jig.key_ratio_at(top).unwrap().to_bits()
        );
        let at = Point::new(top.x - BEHIND_THE_AXIS, top.y - BELOW_THE_TOP);
        let expected = -at.x / protocol.measuring_point(&key);
        let with_load = jig
            .reading_g(&[jig.weight_on_the_key(at, KEY_RATIO_LOAD_G)])
            .unwrap();
        let fw = jig.front_weight_g().unwrap();
        let magnitude = fw.abs().max(with_load.abs()) / KEY_RATIO_LOAD_G;
        let kr = jig.key_ratio_at(at).unwrap();
        assert!(
            agree(kr, expected, 2.0 * magnitude),
            "{kr} against {expected}"
        );
    }

    /// The tare takes the key's own weight out of the ratio: a heavier key,
    /// and a key nobody weighed, read the same ratio.
    #[test]
    fn the_tare_takes_the_key_out_of_the_ratio() {
        let protocol = crate::test_fixture::protocol();
        let key = crate::test_fixture::key();
        let light = jig(&key, &protocol);
        let heavy = KeyJig::new(
            &key,
            &protocol,
            Some(MassProps {
                mass: 0.400,
                ..mass_of(&key)
            }),
        )
        .unwrap();
        let unweighed = KeyJig::new(&key, &protocol, None).unwrap();

        let kr = light.key_ratio().unwrap();
        let magnitude = heavy.front_weight_g().unwrap().abs() / KEY_RATIO_LOAD_G;
        assert!(agree(heavy.key_ratio().unwrap(), kr, magnitude));
        assert!(agree(unweighed.key_ratio().unwrap(), kr, magnitude));
        assert_eq!(unweighed.front_weight_g(), Err(JigRefusal::KeyNotWeighed));
    }

    /// The jig levels the covering: the drawn keytop's top runs parallel to
    /// the `x` axis of the key's frame, so the level the protocol asks for is
    /// angle zero.
    #[test]
    fn level_on_the_jig_is_the_covering_horizontal() {
        let key = crate::test_fixture::key();
        let mut parts = Vec::new();
        let ranges = key.parts(crate::mechanism::BodyId(1), &mut parts);
        // From the lip anchor back along the keytop's top: named points, the
        // only honest probes.
        let lip = ranges.lip.point(&parts);
        let top = &parts[ranges.keytop.index()].outline;
        let back = top[top.len() - 1].start();
        assert!((lip.x - key.lip()).abs() < 1e-15);
        assert!((lip.x - back.x - key.keytop_length).abs() < 1e-15);
        assert_eq!(lip.y.to_bits(), back.y.to_bits());
        assert_eq!(KeyJig::LEVEL.to_bits(), 0.0_f64.to_bits());
    }

    /// The test wippen assembly's mass, its center of mass in its frame.
    fn wippen_mass() -> MassProps {
        crate::test_fixture::wippen_assembly_mass().props(&crate::test_fixture::wippen_assembly())
    }

    /// The test wippen's heel's lowest point, in its frame.
    fn heel_lowest_point() -> Point {
        crate::test_fixture::wippen_assembly()
            .parts(BodyId(1), BodyId::GROUND, &mut Vec::new())
            .heel_lowest_point
    }

    /// Consistency: **the wippen weight is its static moment over the
    /// roller's arm**, `1000 m x_c / x_r`, the closed form the jig matches
    /// within sixteen units in the last place of the weight: under the
    /// heel's lowest point, a millimeter behind it and 5 mm in front.
    #[test]
    fn the_wippen_weight_is_its_static_moment_over_the_rollers_arm() {
        let mass = wippen_mass();
        let jig = WippenJig::new(mass).expect("the wippen on the jig");
        let lowest = heel_lowest_point();
        for along in [0.0, -0.001, 0.005] {
            let roller = Point::new(lowest.x + along, lowest.y);
            let ww = jig.wippen_weight_g(roller).expect("a weight");
            let oracle = WippenWeighing {
                static_moment: mass.mass * mass.com.x,
                roller_arm: roller.x,
            }
            .wippen_weight_g();
            assert!(agree(ww, oracle, ww.abs()), "{ww} g against {oracle} g");
        }
    }

    /// Consistency: **a gram over the roller reads a gram**: on the level
    /// wippen, a gram centered over the roller, at any height, reads a gram
    /// on the tared scale, and one over the center pin nothing.
    #[test]
    fn a_gram_over_the_roller_reads_a_gram() {
        const ABOVE: f64 = 0.040;
        let jig = WippenJig::new(wippen_mass()).expect("the wippen on the jig");
        let roller = heel_lowest_point();
        let bare = jig.wippen_weight_g(roller).expect("a weight");
        let read = |at: Point| {
            jig.reading_g(roller, &[jig.weight_on_the_wippen(at, 1.0)])
                .expect("a reading")
                - bare
        };
        let magnitude = bare.abs() + 1.0;
        let over = read(Point::new(roller.x, roller.y + ABOVE));
        assert!(agree(over, 1.0, magnitude), "{over} g");
        let on_the_pin = read(Point::new(0.0, ABOVE));
        assert!(agree(on_the_pin, 0.0, magnitude), "{on_the_pin} g");
    }

    /// A roller at or behind the center pin cannot hold the wippen level:
    /// the reading is refused, never divided by a zero arm.
    #[test]
    fn a_roller_at_or_behind_the_center_pin_is_refused() {
        let jig = WippenJig::new(wippen_mass()).expect("the wippen on the jig");
        for x in [0.0, -0.010] {
            assert_eq!(
                jig.wippen_weight_g(Point::new(x, -0.02)),
                Err(JigRefusal::ScaleNotInFrontOfThePivot)
            );
        }
    }

    /// A measuring point at or behind the pin is no place for the scale: the
    /// reading is refused, never divided by a zero arm.
    #[test]
    fn a_scale_not_in_front_of_the_pin_is_refused() {
        let key = crate::test_fixture::key();
        for from_lip in [key.lip(), key.lip() + 0.010] {
            let protocol = TouchweightProtocol {
                measuring_point_from_lip: from_lip,
                ..crate::test_fixture::protocol()
            };
            assert_eq!(
                jig(&key, &protocol).front_weight_g(),
                Err(JigRefusal::ScaleNotInFrontOfThePivot)
            );
        }
    }
}

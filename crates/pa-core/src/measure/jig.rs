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
//! is read tared: the scale zeroed with the key on it, 10 g centred on the
//! capstan's contact point, and the reading over the load with its sign
//! turned, so that a scale reading −5.5 gives 0.55. Both are
//! [`scale_reading_g`], so the lever they share is taken once.
//!
//! # What the jig cannot see
//!
//! It turns the key about its balance pin, so it cannot see where the key
//! bears on its balance punching: only a reading on the frame can
//! (`docs/FOUNDATIONS.md` §5.2 a). And its pivot turns without friction, which
//! is an assumption: the real key turns on a pin through its bushings.

use super::{TouchweightProtocol, KR_PROTOCOL_LOAD_G};
use crate::geometry::Point;
use crate::mechanism::{
    CoordId, MassProps, MaterialPoint, Mechanism, PointLoad, Prescribed, Problem,
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
    /// The scale stands at or behind the pivot, where pushing up cannot hold
    /// the key level: the measuring point is not in front of the balance pin.
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
    /// The capstan's contact point, in the key's frame.
    capstan: Point,
}

impl KeyJig {
    /// The angle that holds the key level, radians: the covering's top
    /// horizontal, where the bubble level sits. The covering's top runs
    /// parallel to the underside's flat, the `x` axis of the key's frame, so
    /// level is zero.
    pub const LEVEL: f64 = 0.0;

    /// `key` on the jig, read at `protocol`'s measuring point.
    ///
    /// `mass` is the key's mass and centre of mass in its frame, or `None`
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
        })
    }

    /// A weight of `mass_g` grams centred on `at`, a point of the key in its
    /// frame.
    pub fn weight_on_the_key(&self, at: Point, mass_g: f64) -> PointLoad {
        let at = MaterialPoint {
            body: self.mount.key,
            at,
        };
        PointLoad::weight(at, mass_g / 1000.0)
    }

    /// A weight of `mass_g` grams centred on the capstan's contact point.
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
        self.tared_reading_g(&[self.weight_on_the_capstan(KR_PROTOCOL_LOAD_G)])
    }

    /// **Key weight ratio**, force convention, dimensionless: the tared
    /// reading with 10 g on the capstan, over the load, its sign turned.
    /// *"If the scale reads −5.5 the Key Weight Ratio (KR) would be 0.55"*
    /// [V, Stanwood 2000, p. 22].
    pub fn key_ratio(&self) -> Result<f64, JigRefusal> {
        Ok(-self.key_ratio_reading_g()? / KR_PROTOCOL_LOAD_G)
    }
}

#[cfg(test)]
mod tests {
    use super::super::oracle::KeyWeighing;
    use super::super::MEASURING_POINT_HALF_INCH;
    use super::*;
    use crate::parts::Lead;

    /// The key's mass properties from the volume model, as the bench's
    /// composite key takes them.
    fn mass_of(key: &KeyDesign) -> MassProps {
        let p = key.mass_properties().expect("a key that is not degenerate");
        MassProps {
            mass: p.mass,
            com: p.com,
        }
    }

    fn jig(key: &KeyDesign, protocol: &TouchweightProtocol) -> KeyJig {
        KeyJig::new(key, protocol, Some(mass_of(key))).expect("a key on the jig")
    }

    /// The closed form of the same key on the same jig: static moment about
    /// the pin over the measuring arm, capstan arm over the same.
    fn oracle(key: &KeyDesign, protocol: &TouchweightProtocol) -> KeyWeighing {
        KeyWeighing {
            static_moment_about_pin: key.mass_properties().unwrap().static_moment_x(),
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

    /// Keys that exercise every term: the default, leaded in front and
    /// behind, a capstan moved, a key front brought in, and the measuring
    /// point at the lip, at the glossary's half inch and at 30 mm.
    fn keys() -> Vec<(KeyDesign, TouchweightProtocol)> {
        let protocol = |from_lip: f64| TouchweightProtocol {
            measuring_point_from_lip: from_lip,
        };
        let default = KeyDesign::default();
        let leaded = |leads: Vec<Lead>| KeyDesign {
            leads,
            ..KeyDesign::default()
        };
        vec![
            (default.clone(), TouchweightProtocol::default()),
            (
                leaded(vec![Lead::at(0.150, 0.010), Lead::at(0.190, 0.010)]),
                TouchweightProtocol::default(),
            ),
            (
                leaded(vec![Lead::at(-0.120, 0.030)]),
                TouchweightProtocol::default(),
            ),
            (
                KeyDesign {
                    capstan: -0.110,
                    ..KeyDesign::default()
                },
                TouchweightProtocol::default(),
            ),
            (
                KeyDesign {
                    front_edge: 0.240,
                    ..KeyDesign::default()
                },
                TouchweightProtocol::default(),
            ),
            (default.clone(), protocol(0.0)),
            (default.clone(), protocol(MEASURING_POINT_HALF_INCH)),
            (default, protocol(0.030)),
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
        for (key, protocol) in keys() {
            let (jig, oracle) = (jig(&key, &protocol), oracle(&key, &protocol));

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
                .reading_g(&[jig.weight_on_the_capstan(KR_PROTOCOL_LOAD_G)])
                .unwrap();
            let magnitude = fw.abs().max(with_load.abs()).max(tared.abs());
            assert!(
                agree(tared, oracle.tared_scale_reading_g(), magnitude),
                "tared reading: jig {tared} g, closed form {} g",
                oracle.tared_scale_reading_g()
            );
            let kr = jig.key_ratio().unwrap();
            assert!(
                agree(kr, oracle.key_ratio(), magnitude / KR_PROTOCOL_LOAD_G),
                "key ratio: jig {kr}, closed form {}",
                oracle.key_ratio()
            );
        }
    }

    /// Level 4: the key ratio is the ratio of the two arms, and lands in the
    /// published band.
    ///
    /// Worked from the default key's dimensions, not read off the code: the
    /// capstan 129.5 mm behind the balance pin, the measuring point 13 mm
    /// back from the covering's lip, which stands 2 mm proud of the key front
    /// at 258 mm, so 247 mm in front. 0.524, inside the 0.50 to 0.60 band of a
    /// natural.
    #[test]
    fn the_key_ratio_is_the_arm_ratio_and_lands_in_the_published_band() {
        let jig = jig(&KeyDesign::default(), &TouchweightProtocol::default());
        let expected = 0.1295 / (0.258 + 0.002 - 0.013);
        let kr = jig.key_ratio().unwrap();
        let fw = jig.front_weight_g().unwrap();
        assert!(
            agree(kr, expected, fw.abs() / KR_PROTOCOL_LOAD_G),
            "{kr} against {expected}"
        );
        assert!(
            (0.50..=0.60).contains(&kr),
            "key ratio {kr} outside the band"
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
        let protocol = TouchweightProtocol::default();
        let key = KeyDesign {
            // The lip 2 mm proud of the key front, the point 13 mm back from
            // the lip: 240 mm.
            front_edge: 0.240 + 0.013 - 0.002,
            capstan: -0.240 * 0.55,
            ..KeyDesign::default()
        };
        let jig = jig(&key, &protocol);
        let fw = jig.front_weight_g().unwrap().abs();
        let reading = jig.key_ratio_reading_g().unwrap();
        assert!(
            agree(reading, -5.5, fw),
            "the jig must read −5.5 g, read {reading}"
        );
        assert!(agree(
            jig.key_ratio().unwrap(),
            0.55,
            fw / KR_PROTOCOL_LOAD_G
        ));
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
        let key = KeyDesign::default();
        let at = |from_lip: f64| {
            let protocol = TouchweightProtocol {
                measuring_point_from_lip: from_lip,
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
        let protocol = TouchweightProtocol::default();
        let key = KeyDesign::default();
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

    /// The tare takes the key's own weight out of the ratio: a heavier key,
    /// and a key nobody weighed, read the same ratio.
    #[test]
    fn the_tare_takes_the_key_out_of_the_ratio() {
        let protocol = TouchweightProtocol::default();
        let key = KeyDesign::default();
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
        let magnitude = heavy.front_weight_g().unwrap().abs() / KR_PROTOCOL_LOAD_G;
        assert!(agree(heavy.key_ratio().unwrap(), kr, magnitude));
        assert!(agree(unweighed.key_ratio().unwrap(), kr, magnitude));
        assert_eq!(unweighed.front_weight_g(), Err(JigRefusal::KeyNotWeighed));
    }

    /// The jig levels the covering: its top runs parallel to the `x` axis of
    /// the key's frame, so the level the protocol asks for is angle zero.
    #[test]
    fn level_on_the_jig_is_the_covering_horizontal() {
        let key = KeyDesign::default();
        let outline = key.outline_vertices();
        // Vertices 6 and 7 are the covering's top, from the lip back: the
        // outline's order is fixed, and a named vertex is the only honest
        // probe.
        let (lip, back) = (outline[6], outline[7]);
        assert!((lip.x - key.lip()).abs() < 1e-15);
        assert!((lip.x - back.x - key.keytop_length).abs() < 1e-15);
        assert_eq!(lip.y.to_bits(), back.y.to_bits());
        assert_eq!(KeyJig::LEVEL.to_bits(), 0.0_f64.to_bits());
    }

    /// A measuring point at or behind the pin is no place for the scale: the
    /// reading is refused, never divided by a zero arm.
    #[test]
    fn a_scale_not_in_front_of_the_pin_is_refused() {
        let key = KeyDesign::default();
        for from_lip in [key.lip(), key.lip() + 0.010] {
            let protocol = TouchweightProtocol {
                measuring_point_from_lip: from_lip,
            };
            assert_eq!(
                jig(&key, &protocol).front_weight_g(),
                Err(JigRefusal::ScaleNotInFrontOfThePivot)
            );
        }
    }
}

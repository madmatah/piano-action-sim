//! What is known of a key's mass: weighed whole, as the trade weighs a key,
//! or estimated where nobody has weighed it.
//!
//! # Weighed whole
//!
//! A key arrives leaded, with leads whose masses nobody knows, so it is
//! weighed whole, its leads in it: its mass on a scale, and its front weight
//! on Stanwood's jig, the levelled key's weight read on the scale under the
//! measuring point [V, Stanwood 2000, p. 21]. Those two give all the statics
//! read of a key but one number. The front weight times the arm it was read
//! on, times gravity, is the moment of the key's weight about its balance pin,
//! its static moment; over the key's weight it stands the centre of mass that
//! far along the key. The moment is kept as what the key is, with the reading
//! that gave it as its datum ([`FrontWeightReading`]), the way a friction is
//! kept as a torque with the radius it was read at
//! ([`crate::mechanism::AxisFriction`]): moving the measuring point
//! afterwards, or editing the key, reads the same moment on another arm, and
//! the front weight the jig reads follows by the lever.
//!
//! No lead is modelled. A key's leads are in its weighing, and what one more
//! would do somewhere along the key is not a calculation the bench makes.
//!
//! # What a scale does not give
//!
//! The height of the centre of mass. On the levelled jig it moves no reading;
//! on the frame it enters through the key's tilt alone, each millimetre of it
//! moving a reading taken on the tilted key by the key's mass times the sine
//! of the tilt over the measuring arm: about 0.02 g of down weight per
//! millimetre on the default key, 2.3° nose-up at the top of its stroke. So a
//! weighed key takes it from an estimate of the key's ([`KeyMass::props`]),
//! and the readings the tilt reaches name it as estimated
//! ([`crate::measure::measurement::Input::CentreOfMassHeight`]).

use super::KeyDesign;
use crate::geometry::Point;
use crate::mechanism::MassProps;
use crate::G;

/// A front weight as it was read: what the jig's scale showed, and where its
/// roller stood.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrontWeightReading {
    /// What the scale read, grams, signed: negative on a key heavy behind,
    /// as Stanwood's protocol for a negative front weight reads it.
    pub grams: f64,
    /// How far back from the covering's lip the roller stood, metres.
    pub from_lip_m: f64,
    /// The arm that put the roller at, metres in front of the balance pin, on
    /// the key as it stood when it was weighed.
    pub arm_m: f64,
}

/// What is known of a key's mass.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyMass {
    /// Nobody has weighed the key: an estimate of its mass and centre of
    /// mass stands in for the whole, which the bench derives for the
    /// composite key from its volume at an assumed density
    /// (`docs/FOUNDATIONS.md` §2.3).
    Estimated,
    /// Weighed whole, its leads in it.
    Weighed {
        /// Its mass, kilograms.
        mass: f64,
        /// The moment of its weight about the balance pin, newton-metres,
        /// positive turning the front down: its front weight times the arm
        /// it was read on, times standard gravity.
        static_moment_n_m: f64,
        /// The reading that gave the moment.
        datum: FrontWeightReading,
    },
}

/// Why a key's weighing is refused: the numbers typed are no key's.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotAWeighing {
    /// A value that is not a number.
    NotANumber,
    /// A mass of nothing or less.
    MassNotPositive,
    /// The jig's roller would have stood off the covering's top, in front of
    /// its lip or behind its back end, or not in front of the balance pin:
    /// no front weight is read there.
    RollerOffTheCovering {
        /// Where it was said to stand, metres back from the covering's lip.
        from_lip_m: f64,
    },
    /// The two readings put the centre of mass off the key: in front of its
    /// covering's lip, or behind its tail.
    CentreOfMassOffTheKey {
        /// Where they put it, metres in front of the balance pin.
        at_m: f64,
    },
}

impl KeyMass {
    /// `key` weighed whole: `mass_g` grams on a scale, and a front weight of
    /// `front_weight_g` grams read on the levelled jig, its roller
    /// `from_lip_m` metres back from the covering's lip; or why those numbers
    /// are no key's.
    ///
    /// The static moment is the front weight times the arm the roller stood
    /// at on `key` as it is now, the datum the weighing keeps. A front weight
    /// may be negative, a key heavy behind, and zero; a mass may not. The
    /// roller stands on the covering, as the measuring point does, and the
    /// centre of mass the two readings give stands on the key, between its
    /// tail and its covering's lip: a rigid body's centre of mass lies within
    /// it.
    pub fn weighed(
        key: &KeyDesign,
        mass_g: f64,
        front_weight_g: f64,
        from_lip_m: f64,
    ) -> Result<Self, NotAWeighing> {
        if !(mass_g.is_finite() && front_weight_g.is_finite() && from_lip_m.is_finite()) {
            return Err(NotAWeighing::NotANumber);
        }
        if mass_g <= 0.0 {
            return Err(NotAWeighing::MassNotPositive);
        }
        let arm_m = key.lip() - from_lip_m;
        if !((0.0..=key.keytop_length).contains(&from_lip_m) && arm_m > 0.0) {
            return Err(NotAWeighing::RollerOffTheCovering { from_lip_m });
        }
        let mass = mass_g / 1000.0;
        let static_moment_n_m = front_weight_g / 1000.0 * G * arm_m;
        let at_m = static_moment_n_m / (mass * G);
        if !(key.tail_end..=key.lip()).contains(&at_m) {
            return Err(NotAWeighing::CentreOfMassOffTheKey { at_m });
        }
        Ok(Self::Weighed {
            mass,
            static_moment_n_m,
            datum: FrontWeightReading {
                grams: front_weight_g,
                from_lip_m,
                arm_m,
            },
        })
    }

    /// The mass and centre of mass the statics take, `estimate` standing in
    /// for what is not weighed: the whole of an estimated key's; a weighed
    /// key's own mass, its centre of mass along the key where its static
    /// moment over its weight puts it, at the height of the estimate's, which
    /// no scale gives.
    pub fn props(&self, estimate: MassProps) -> MassProps {
        match *self {
            Self::Estimated => estimate,
            Self::Weighed {
                mass,
                static_moment_n_m,
                ..
            } => MassProps {
                mass,
                com: Point::new(static_moment_n_m / (mass * G), estimate.com.y),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measure::jig::KeyJig;
    use crate::measure::{TouchweightProtocol, MEASURING_POINT_HALF_INCH};
    use crate::parts::fixtures::mass_of;

    /// The protocol whose measuring point stands `from_lip` back from the
    /// covering's lip.
    fn at(from_lip: f64) -> TouchweightProtocol {
        TouchweightProtocol {
            measuring_point_from_lip: from_lip,
        }
    }

    /// Weighings of the default key that exercise every sign: heavy in
    /// front, heavy behind, balanced on its pin, each read at another
    /// measuring point, as grams, grams and metres from the lip.
    const WEIGHINGS: [(f64, f64, f64); 4] = [
        (121.1, 6.5, 0.013),
        (151.1, -8.1, 0.013),
        (140.0, 30.0, MEASURING_POINT_HALF_INCH),
        (95.0, 0.0, 0.030),
    ];

    /// Consistency: **a weighed key reads its front weight back on the jig
    /// where it was read, and by the lever elsewhere.** The jig run on the
    /// mechanism with the weighed key's mass reads the typed grams with its
    /// roller where the weighing says it stood, and at another measuring
    /// point the same static moment on the other arm. The bound is derived:
    /// the moment, the centre of mass, and the jig's reaction and reading
    /// take at most sixteen rounded operations, each within `ε/2` of what
    /// it touches, so the two agree within `16 ε` of the front weight.
    #[test]
    fn a_weighed_key_reads_its_front_weight_back_where_it_was_read() {
        let key = KeyDesign::default();
        let estimate = mass_of(&key).expect("the default key has a volume");
        for (mass_g, grams, from_lip) in WEIGHINGS {
            let weighed = KeyMass::weighed(&key, mass_g, grams, from_lip).expect("a weighing");
            let props = weighed.props(estimate);
            let read = |from: f64| {
                KeyJig::new(&key, &at(from), Some(props))
                    .expect("a key on the jig")
                    .front_weight_g()
                    .expect("a weighed key has a front weight")
            };
            let bound = 16.0 * f64::EPSILON * grams.abs();
            let back = read(from_lip);
            assert!(
                (back - grams).abs() <= bound,
                "{mass_g} g weighed at {grams} g reads {back} g where it was read"
            );
            let other = if from_lip < 0.020 { 0.030 } else { 0.013 };
            let moment = grams * (key.lip() - from_lip);
            let lever = read(other) * (key.lip() - other);
            assert!(
                (lever - moment).abs() <= 2.0 * bound * key.lip(),
                "{grams} g at {from_lip} m reads {} g at {other} m, not the same moment",
                read(other)
            );
            let KeyMass::Weighed { datum, .. } = weighed else {
                panic!("{weighed:?}");
            };
            assert_eq!(
                datum,
                FrontWeightReading {
                    grams,
                    from_lip_m: from_lip,
                    arm_m: key.lip() - from_lip,
                },
                "the reading kept as typed"
            );
        }
    }

    /// Consistency: **what no scale gives comes from the estimate, and
    /// nothing else does.** A weighed key's centre of mass stands at the
    /// estimate's height to the bit, its mass is its own; an estimated key's
    /// mass and centre of mass are the estimate's to the bit.
    #[test]
    fn the_height_no_scale_gives_is_the_estimates() {
        let key = KeyDesign::default();
        let estimate = mass_of(&key).expect("the default key has a volume");
        let same = KeyMass::Estimated.props(estimate);
        assert_eq!(same.mass.to_bits(), estimate.mass.to_bits());
        assert_eq!(same.com.x.to_bits(), estimate.com.x.to_bits());
        assert_eq!(same.com.y.to_bits(), estimate.com.y.to_bits());
        for (mass_g, grams, from_lip) in WEIGHINGS {
            let props = KeyMass::weighed(&key, mass_g, grams, from_lip)
                .expect("a weighing")
                .props(estimate);
            assert_eq!(props.com.y.to_bits(), estimate.com.y.to_bits());
            assert_eq!(props.mass.to_bits(), (mass_g / 1000.0).to_bits());
        }
    }

    /// Consistency: **numbers no key has are refused, with why.** A value
    /// that is not a number; a mass of nothing or less; a roller in front of
    /// the covering's lip or behind its back end; a centre of mass the two
    /// readings put in front of the lip or behind the tail, as 50 g of front
    /// weight on a 40 g key does, or −100 g.
    #[test]
    fn numbers_no_key_has_are_refused() {
        let key = KeyDesign::default();
        let weigh = |mass_g, grams, from_lip| KeyMass::weighed(&key, mass_g, grams, from_lip);
        for bad in [f64::NAN, f64::INFINITY] {
            assert_eq!(weigh(bad, 6.5, 0.013), Err(NotAWeighing::NotANumber));
            assert_eq!(weigh(121.1, bad, 0.013), Err(NotAWeighing::NotANumber));
            assert_eq!(weigh(121.1, 6.5, bad), Err(NotAWeighing::NotANumber));
        }
        assert_eq!(weigh(0.0, 6.5, 0.013), Err(NotAWeighing::MassNotPositive));
        assert_eq!(weigh(-5.0, 6.5, 0.013), Err(NotAWeighing::MassNotPositive));
        for from_lip in [-0.001, key.keytop_length + 0.001] {
            assert_eq!(
                weigh(121.1, 6.5, from_lip),
                Err(NotAWeighing::RollerOffTheCovering {
                    from_lip_m: from_lip
                })
            );
        }
        for grams in [50.0, -100.0] {
            match weigh(40.0, grams, 0.013) {
                Err(NotAWeighing::CentreOfMassOffTheKey { at_m }) => assert!(
                    at_m > key.lip() || at_m < key.tail_end,
                    "{grams} g: {at_m} m"
                ),
                other => panic!("{grams} g on a 40 g key: {other:?}"),
            }
        }
        assert!(weigh(121.1, 6.5, 0.0).is_ok(), "read at the very lip");
        assert!(
            weigh(121.1, 6.5, key.keytop_length).is_ok(),
            "read at the covering's back end"
        );
    }
}

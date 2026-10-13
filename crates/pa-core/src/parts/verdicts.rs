//! What holds a key where it stands: read from determinate quantities only.
//!
//! # Why not from the reactions
//!
//! Rigid statics leave some splits of a load indeterminate. A key standing
//! on its punching under a weight its friction could hold is held by the
//! punching, by its bushings' friction, or by any mixture of the two: the
//! static solve reports one admissible split, the one of least utilisation,
//! and it is no answer to the question a technician asks
//! (`docs/FOUNDATIONS.md` §2.1). So a verdict is built from what does not
//! depend on the split: which contacts are closed, how far the key stands
//! off its balance punching, and whether the generalized force along the
//! motion the key has with every stop removed, its bearing kept, lies inside
//! its friction's box.
//!
//! That is what lets the bench say that friction holds a key: on its
//! punching or its cloth, or floating between them, wherever a weight
//! between up weight and down weight has left it. How far the weight stands
//! from moving it is measured to the edge of the band the key would move
//! past: on its cloth, to the down weight read from there; on its punching,
//! to the up weight.
//!
//! # Off its balance punching
//!
//! A key free to lift may stand off its balance punching: riding on its two
//! outer stops where they hold it higher than the punching would, or lifted
//! off it onto what it touches. Either is said with the lift, read off the
//! key's own coordinate, and the second with the stops it touches, the
//! contacts closed there, and whether they hold it alone or with its
//! friction: whether pushes along their rows, none pulling, balance the loads
//! with no friction at all, which does not depend on how a solve split them.
//! All three are determinate.
//!
//! # What a centre bearing leaves aside
//!
//! A punching read at its centre is a rigid stand-in for a felt that spreads
//! its load. A key standing out of level on it overlaps the drawn felt's edge
//! by what the felt would compress, half the washer's width times the tilt:
//! about a quarter of a millimetre under the default key at rest on its
//! balance punching. [`edge_overlaps`] says how much, so that the
//! idealization does not pass for a drawing error.

use crate::contact::{ContactCode, PairGeometry, Rows, ROW_MARGIN};
use crate::mechanism::PointLoad;
use crate::parts::mounts::Mounted;

/// How near closing a contact must stand to count as touching, metres: a
/// tenth of a micron, a thousandth of the thinnest paper, and far above the
/// rounding a static solve leaves on a closed contact.
pub const TOUCHING: f64 = 1e-7;

/// What holds the key.
#[derive(Debug, Clone, PartialEq)]
pub enum Holding {
    /// With the stops removed, the loads would turn the key into this one,
    /// past what its friction holds: the stop carries it.
    RestingOn {
        /// The stop.
        stop: ContactCode,
    },
    /// With the stops removed, the loads' generalized force lies inside the
    /// friction's box: friction alone holds the key where it is, touching a
    /// stop or not. On a key, the weight on it lies between its up weight
    /// and its down weight.
    HeldByFriction {
        /// The stop it touches, if it touches one.
        touching: Option<ContactCode>,
        /// How far the loads' generalized force stands from the edge of the
        /// box past which the key moves, newton-metres on the key's angle:
        /// away from the stop it touches, the only way it can move; either
        /// way, the nearer, where it touches none.
        margin: f64,
    },
    /// Off its balance punching, on its front stop and its back rail cloth:
    /// the key's stops fight, and hold it higher than its punching would.
    RidesOnBothStops {
        /// The front stop it rides on: the front punching, or the front rail
        /// pin in its mortise or on the wood beside it.
        front: ContactCode,
        /// How far above its balance punching it stands, metres.
        lift: f64,
    },
    /// Off its balance punching, held by what it touches, with its friction
    /// or without, but not on both its outer stops.
    LiftsOffBalancePunching {
        /// The stops it touches, each closed within [`TOUCHING`], in the
        /// mechanism's order of pairs: its cloth where a pull has turned it
        /// up on it, its front punching and its front pin against the wall
        /// of its slot where a front rail raised under it holds it up.
        touching: Vec<ContactCode>,
        /// Whether those stops need its friction to hold it there: false
        /// where pushes along their rows, none pulling, balance the loads
        /// alone, as a front punching and a front pin against the wall of
        /// its slot do; true where they cannot, as its cloth alone, pushing
        /// at one place, cannot hold a pull turning the key about it.
        needs_friction: bool,
        /// How far above its balance punching it stands, metres.
        lift: f64,
    },
}

/// What holds the key `mounted` at `q` under `loads`, gravity besides; `None`
/// when nothing does, which a state a static solve converged on never is.
pub fn holding(mounted: &Mounted, q: &[f64], loads: &[PointLoad]) -> Option<Holding> {
    let m = &mounted.mechanism;
    let angle = mounted.angle.0;
    let pose = m.pose(q);
    let mut rows = Rows::new();
    m.rows(&pose, ROW_MARGIN, &mut rows);
    let code = |pair: u16| m.pairs()[usize::from(pair)].code;
    // The balance punching is the key's bearing, kept with every stop
    // removed: none of its rows is a stop.
    let bearing = mounted.lift.as_ref().map(|lift| lift.pair);
    let stops: Vec<usize> = (0..rows.len())
        .filter(|&i| Some(rows.rows()[i].pair) != bearing)
        .collect();
    let lift = mounted.lift_at(q);
    if lift > TOUCHING {
        // The stops closed, each pair once, in the mechanism's order.
        let mut touching: Vec<ContactCode> = Vec::new();
        for &i in &stops {
            let stop = code(rows.rows()[i].pair);
            if rows.rows()[i].gap <= TOUCHING && !touching.contains(&stop) {
                touching.push(stop);
            }
        }
        let front = touching.iter().copied().find(|c| {
            matches!(
                c,
                ContactCode::FrontRailPunching
                    | ContactCode::FrontPinInTheMortise
                    | ContactCode::FrontPinOnTheWood
            )
        });
        return Some(match front {
            Some(front) if touching.contains(&ContactCode::BackRailCloth) => {
                Holding::RidesOnBothStops { front, lift }
            }
            _ => {
                let closed: Vec<usize> = stops
                    .iter()
                    .copied()
                    .filter(|&i| rows.rows()[i].gap <= TOUCHING)
                    .collect();
                let mut forces = vec![0.0; m.coords().len()];
                m.generalized_forces(&pose, loads, &mut forces);
                Holding::LiftsOffBalancePunching {
                    touching,
                    needs_friction: !balanced_by(&rows, &closed, &forces),
                    lift,
                }
            }
        });
    }
    let mut forces = vec![0.0; m.coords().len()];
    m.generalized_forces(&pose, loads, &mut forces);
    let push = forces[angle];
    let bound = m.coords()[angle].friction;
    if push.abs() <= bound {
        let touched = stops
            .iter()
            .copied()
            .find(|&i| rows.rows()[i].gap <= TOUCHING);
        // Away from a stop the key touches is the way its gap opens: the
        // generalized force must pass the box's edge on that side. Into the
        // stop, past the other edge, it moves nothing; the stop takes it.
        let margin = match touched.map(|i| rows.grad(i)[angle]) {
            Some(opens) if opens > 0.0 => bound - push,
            Some(opens) if opens < 0.0 => bound + push,
            _ => bound - push.abs(),
        };
        return Some(Holding::HeldByFriction {
            touching: touched.map(|i| code(rows.rows()[i].pair)),
            margin,
        });
    }
    // The stop the push drives the key into: a closed contact the motion it
    // pushes would close further.
    stops
        .iter()
        .find(|&&i| rows.rows()[i].gap <= TOUCHING && rows.grad(i)[angle] * push < 0.0)
        .map(|&i| Holding::RestingOn {
            stop: code(rows.rows()[i].pair),
        })
}

/// Whether contacts along the rows `closed` of `rows`, each pushing with a
/// force of its own and none pulling, balance the generalized force `forces`
/// with no friction: whether minus it is a combination of the rows'
/// gradients with no negative weight.
///
/// Read on a key free to lift, which has two coordinates, its slide and its
/// angle: in two dimensions such a combination needs at most two of the
/// rows, so each row is tried alone, the force along its gradient, and each
/// two by solving for their two forces. Two rows whose gradients are
/// parallel to the last bit hold nothing a single one does not.
fn balanced_by(rows: &Rows, closed: &[usize], forces: &[f64]) -> bool {
    let &[f0, f1] = forces else {
        return false;
    };
    for (k, &i) in closed.iter().enumerate() {
        let (a0, a1) = (rows.grad(i)[0], rows.grad(i)[1]);
        if a0 * f1 - a1 * f0 == 0.0 && a0 * f0 + a1 * f1 <= 0.0 {
            return true;
        }
        for &j in &closed[k + 1..] {
            let (b0, b1) = (rows.grad(j)[0], rows.grad(j)[1]);
            let det = a0 * b1 - a1 * b0;
            if det == 0.0 {
                continue;
            }
            let along_i = (b0 * f1 - b1 * f0) / det;
            let along_j = (a1 * f0 - a0 * f1) / det;
            if along_i >= 0.0 && along_j >= 0.0 {
                return true;
            }
        }
    }
    false
}

/// How far each centre bearing the key stands on overlaps it at `q`, metres:
/// for every pair read at its centre whose centre bears, within
/// [`TOUCHING`] of closing, how far the drawn face stands past the key's
/// underside ([`crate::mechanism::Mechanism::face_overlap`]), in the
/// mechanism's order of pairs.
///
/// What the felt would compress where the key stands out of level on it,
/// which the rigid reading at the centre leaves aside: half the face's width
/// times the tilt, so always present under a key at rest on its balance
/// punching, nose-up by its dip.
pub fn edge_overlaps(mounted: &Mounted, q: &[f64]) -> Vec<(ContactCode, f64)> {
    let m = &mounted.mechanism;
    let pose = m.pose(q);
    let mut rows = Rows::new();
    m.rows(&pose, ROW_MARGIN, &mut rows);
    (0..m.pairs().len())
        .filter_map(|p| {
            let pair = u16::try_from(p).expect("fewer than 65 536 pairs");
            let declared = &m.pairs()[p];
            let bears = rows.gap(pair).is_some_and(|g| g <= TOUCHING);
            (declared.geometry == PairGeometry::CentreBearing && bears)
                .then(|| (declared.code, m.face_overlap(&pose, pair)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;
    use crate::mechanism::MaterialPoint;
    use crate::parts::fixtures::{on_frame, regulated, OnFrame};
    use crate::parts::frame::KeyKind;
    use crate::solve::{Friction, Statics};
    use crate::G;

    /// The bushings' friction of the default bench, a 6 g band half-width
    /// read at the measuring point, as a torque on the key's angle.
    fn bench() -> (OnFrame, f64) {
        let a = regulated();
        let arm = a.protocol.measuring_point(&a.key);
        let bound = 0.006 * G * arm;
        (on_frame(&a, KeyKind::Natural, bound, true), bound)
    }

    /// `grams` laid on the measuring point, negative for a pull upwards.
    fn weight(k: &OnFrame, grams: f64) -> PointLoad {
        PointLoad {
            at: MaterialPoint {
                body: k.mounted.key,
                at: k.measuring_point,
            },
            force: Point::new(0.0, -grams * 1e-3 * G),
        }
    }

    /// The generalized force the loads put on the key's angle, gravity
    /// besides, at `q`.
    fn push(k: &OnFrame, q: &[f64], loads: &[PointLoad]) -> f64 {
        let m = &k.mounted.mechanism;
        let mut forces = vec![0.0; m.coords().len()];
        m.generalized_forces(&m.pose(q), loads, &mut forces);
        forces[k.mounted.angle.0]
    }

    /// Consistency, the audit's BWA-10: **a weight between the up weight
    /// and the down weight leaves the key where it is, and friction is what
    /// holds it there**, on its punching or on its cloth alike.
    ///
    /// The default key alone is front-heavy, its front weight about 6.5 g
    /// against a 6 g band, so its down weight is about −0.5 g and its up
    /// weight about −12.5 g: unweighted it goes down onto its front
    /// punching, a 13 g pull brings it up onto its cloth, and a 5 g pull,
    /// inside the band, holds it wherever it was. Each weight is an
    /// increment from the state the one before left, friction on, and the
    /// key that holds has not moved by a single bit. The margin is how far
    /// the loads' generalized force stands from the edge of the band past
    /// which the key leaves the stop it touches: on the punching, where the
    /// weight would lift it, `b − push`; on the cloth, where it would take it
    /// down, `b + push`, the push positive lifting the front. Both are
    /// determinate; the balance punching under the key all along is its
    /// bearing, and never the stop named.
    #[test]
    fn a_weight_inside_the_band_leaves_the_key_where_it_is_held_by_friction() {
        let (k, bound) = bench();
        let mut statics = Statics::new();
        let start = crate::parts::mounts::nominal_pose(&k.mounted);
        let rest = k
            .mounted
            .solve(&mut statics, &start, &[weight(&k, 0.0)], Friction::Off)
            .expect("the key comes to rest")
            .q;
        assert_eq!(
            holding(&k.mounted, &rest, &[weight(&k, 0.0)]),
            Some(Holding::RestingOn {
                stop: ContactCode::FrontRailPunching
            })
        );
        let mut q = rest;
        for (grams, stop, held) in [
            (-5.0, ContactCode::FrontRailPunching, true),
            (-13.0, ContactCode::BackRailCloth, false),
            (-5.0, ContactCode::BackRailCloth, true),
            (0.0, ContactCode::FrontRailPunching, false),
        ] {
            let loads = [weight(&k, grams)];
            let next = k
                .mounted
                .solve(&mut statics, &q, &loads, Friction::On)
                .expect("an increment solves")
                .q;
            assert_eq!(k.mounted.lift_at(&next).to_bits(), 0.0_f64.to_bits());
            let verdict = holding(&k.mounted, &next, &loads);
            if held {
                assert_eq!(
                    next.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                    q.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                    "{grams} g moved the key"
                );
                let pushed = push(&k, &next, &loads);
                let margin = match stop {
                    ContactCode::BackRailCloth => bound + pushed,
                    _ => bound - pushed,
                };
                assert_eq!(
                    verdict,
                    Some(Holding::HeldByFriction {
                        touching: Some(stop),
                        margin
                    }),
                    "{grams} g"
                );
                assert!(margin > 0.0);
            } else {
                assert_eq!(verdict, Some(Holding::RestingOn { stop }), "{grams} g");
            }
            q = next;
        }
    }

    /// Consistency: **a key friction does not hold stands on the stop its
    /// loads push it into**, and the verdict does not depend on how the
    /// static solve split the load: with no friction at all, every weight
    /// either rests the key on its punching or on its cloth.
    #[test]
    fn with_no_friction_every_weight_rests_the_key_on_a_stop() {
        let a = regulated();
        let k = on_frame(&a, KeyKind::Natural, 0.0, true);
        let mut statics = Statics::new();
        let start = crate::parts::mounts::nominal_pose(&k.mounted);
        for (grams, stop) in [
            (-20.0, ContactCode::BackRailCloth),
            (-7.0, ContactCode::BackRailCloth),
            (-6.0, ContactCode::FrontRailPunching),
            (20.0, ContactCode::FrontRailPunching),
        ] {
            let loads = [weight(&k, grams)];
            let q = k
                .mounted
                .solve(&mut statics, &start, &loads, Friction::Off)
                .expect("the key comes to rest")
                .q;
            assert_eq!(
                holding(&k.mounted, &q, &loads),
                Some(Holding::RestingOn { stop }),
                "{grams} g"
            );
        }
    }

    /// Consistency: **a key whose stops fight rides on them, and says so**:
    /// the cloth holding its tail 5 mm above the balance bearing, the key at
    /// rest rides on its front punching and its cloth, its lift the
    /// coordinate's height off its punching, whatever weight inside its band
    /// is on it.
    #[test]
    fn a_key_whose_stops_fight_rides_on_both() {
        let mut a = regulated();
        a.frame.back_rail_height =
            a.frame.balance_bearing_height() + 0.005 - a.frame.back_rail_cloth;
        let arm = a.protocol.measuring_point(&a.key);
        let k = on_frame(&a, KeyKind::Natural, 0.006 * G * arm, true);
        let mut statics = Statics::new();
        let start = crate::parts::mounts::nominal_pose(&k.mounted);
        let q = k
            .mounted
            .solve(&mut statics, &start, &[weight(&k, 0.0)], Friction::Off)
            .expect("the key rides")
            .q;
        let lift = k.mounted.lift_at(&q);
        for grams in [0.0, 3.0, -3.0] {
            assert_eq!(
                holding(&k.mounted, &q, &[weight(&k, grams)]),
                Some(Holding::RidesOnBothStops {
                    front: ContactCode::FrontRailPunching,
                    lift
                }),
                "{grams} g"
            );
        }
        assert!(lift > TOUCHING);
    }

    /// Consistency: **a key off its balance punching names what it
    /// touches, and whether they need its friction to hold it.** The front
    /// rail raised to 41.7 mm stands its punching so high that the key,
    /// unweighted and front-heavy, tips forward onto it and leaves its
    /// balance punching: the front punching carries it, the front pin's top
    /// corner meets the wall of the key's slot, which moves along the pin as
    /// the key turns, and holds it from turning further, and its tail stands
    /// clear of its cloth. The two hold it with no friction at all: it comes
    /// to rest there with its friction off. Pulled up 62.7 g at its
    /// measuring point, a few hundredths of a gram past what its punching
    /// carries on the frame as built, the key turns up on its cloth instead
    /// and leaves its punching, held there by its cloth and its friction:
    /// the cloth pushes at one place, and the pull turns the key about it.
    /// Each verdict names the stops closed where the key stands, in the
    /// order its contacts are declared, and the cloth only where it is
    /// closed.
    #[test]
    fn a_key_off_its_balance_punching_names_what_it_touches() {
        let gap = |k: &OnFrame, q: &[f64], code: ContactCode| {
            let m = &k.mounted.mechanism;
            let mut rows = Rows::new();
            m.rows(&m.pose(q), ROW_MARGIN, &mut rows);
            let pair = m.pairs().iter().position(|p| p.code == code).unwrap();
            rows.gap(u16::try_from(pair).unwrap())
        };

        let mut a = regulated();
        a.frame.front_rail_height = 0.0417;
        let arm = a.protocol.measuring_point(&a.key);
        let raised = on_frame(&a, KeyKind::Natural, 0.006 * G * arm, true);
        let start = crate::parts::mounts::nominal_pose(&raised.mounted);
        let unweighted = [weight(&raised, 0.0)];
        let q = raised
            .mounted
            .solve(&mut Statics::new(), &start, &unweighted, Friction::Off)
            .expect("the key comes to rest")
            .q;
        let Some(Holding::LiftsOffBalancePunching {
            touching,
            needs_friction,
            lift,
        }) = holding(&raised.mounted, &q, &unweighted)
        else {
            panic!("{:?}", holding(&raised.mounted, &q, &unweighted));
        };
        assert_eq!(
            touching,
            vec![
                ContactCode::FrontRailPunching,
                ContactCode::FrontPinOnTheWood
            ]
        );
        assert!(!needs_friction, "its front punching and pin hold it alone");
        assert!(lift > TOUCHING, "{lift} m");
        assert!(
            gap(&raised, &q, ContactCode::BackRailCloth).is_none_or(|g| g > TOUCHING),
            "the cloth is open"
        );

        let (k, _) = bench();
        let start = crate::parts::mounts::nominal_pose(&k.mounted);
        let rest = k
            .mounted
            .solve(
                &mut Statics::new(),
                &start,
                &[weight(&k, 0.0)],
                Friction::Off,
            )
            .expect("the key comes to rest")
            .q;
        let pulled = [weight(&k, -62.7)];
        let q = k
            .mounted
            .solve(&mut Statics::new(), &rest, &pulled, Friction::On)
            .expect("the key is held on its cloth")
            .q;
        assert_eq!(
            holding(&k.mounted, &q, &pulled),
            Some(Holding::LiftsOffBalancePunching {
                touching: vec![ContactCode::BackRailCloth],
                needs_friction: true,
                lift: k.mounted.lift_at(&q),
            })
        );
        assert!(k.mounted.lift_at(&q) > TOUCHING);
    }

    /// Consistency: **the balance punching's drawn edge overlaps a key at
    /// rest on it by half its width times the tilt**, what a felt would
    /// compress and the rigid centre bearing leaves aside, about a quarter of
    /// a millimetre on the default key; the front punching, which a key at
    /// rest does not bear on, is not named. Down on its front punching, the
    /// key stands level to a few thousandths of a degree, and that punching's
    /// edge overlaps it by its own half width times that tilt.
    #[test]
    fn a_tilted_key_overlaps_the_edge_of_a_centre_bearing_by_the_tilt() {
        let (k, _) = bench();
        let m = &k.mounted;
        let seats = k
            .seating()
            .seats(&mut Statics::new())
            .expect("the key seats");
        let half = |code: ContactCode| {
            let a = regulated();
            match code {
                ContactCode::BalanceRailPunching => a.frame.sections.balance.punching_radius,
                _ => a.frame.sections.front.punching_radius,
            }
        };
        for (seat, code) in [
            (&seats.top, ContactCode::BalanceRailPunching),
            (&seats.bottom, ContactCode::FrontRailPunching),
        ] {
            let overlaps = edge_overlaps(m, &seat.q);
            let (_, mm) = overlaps
                .iter()
                .copied()
                .find(|(c, _)| *c == code)
                .unwrap_or_else(|| panic!("{code:?} not named: {overlaps:?}"));
            let tilt = libm::sin(seat.q[m.angle.0]).abs();
            let expected = half(code) * tilt;
            assert!(
                (mm - expected).abs() <= 1e-3 * expected + 1e-12,
                "{code:?}: {mm} m, half its width times the tilt {expected} m"
            );
        }
        let at_rest = edge_overlaps(m, &seats.top.q);
        assert!(
            at_rest
                .iter()
                .all(|(c, _)| *c != ContactCode::FrontRailPunching),
            "{at_rest:?}"
        );
        let (_, balance) = at_rest[0];
        assert!(
            balance > 0.0002 && balance < 0.0003,
            "a quarter of a millimetre: {balance}"
        );
    }
}

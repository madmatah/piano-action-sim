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
//! depend on the split: which contacts are closed, and whether the
//! generalized force along the motion the key has with every stop removed,
//! its bearing kept, lies inside its friction's box.
//!
//! That is what lets the bench say that friction holds a key: on its
//! punching or its cloth, or floating between them, wherever a weight
//! between up weight and down weight has left it. How far the weight stands
//! from moving it is measured to the edge of the band the key would move
//! past: on its cloth, to the down weight read from there; on its punching,
//! to the up weight.

use crate::contact::{ContactCode, Rows, ROW_MARGIN};
use crate::mechanism::PointLoad;
use crate::parts::mounts::Mounted;

/// How near closing a contact must stand to count as touching, metres: a
/// tenth of a micron, a thousandth of the thinnest paper, and far above the
/// rounding a static solve leaves on a closed contact.
pub const TOUCHING: f64 = 1e-7;

/// What holds the key.
#[derive(Debug, Clone, Copy, PartialEq)]
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
}

/// What holds the key `mounted` at `q` under `loads`, gravity besides; `None`
/// when nothing does, which a state a static solve converged on never is.
pub fn holding(mounted: &Mounted, q: &[f64], loads: &[PointLoad]) -> Option<Holding> {
    let m = &mounted.mechanism;
    let angle = mounted.angle.0;
    let pose = m.pose(q);
    let mut forces = vec![0.0; m.coords().len()];
    m.generalized_forces(&pose, loads, &mut forces);
    let push = forces[angle];
    let bound = m.coords()[angle].friction;
    let mut rows = Rows::new();
    m.rows(&pose, ROW_MARGIN, &mut rows);
    let code = |pair: u16| m.pairs()[usize::from(pair)].code;
    if push.abs() <= bound {
        let touched = rows
            .rows()
            .iter()
            .enumerate()
            .find(|(_, r)| r.gap <= TOUCHING);
        // Away from a stop the key touches is the way its gap opens: the
        // generalized force must pass the box's edge on that side. Into the
        // stop, past the other edge, it moves nothing; the stop takes it.
        let margin = match touched.map(|(i, _)| rows.grad(i)[angle]) {
            Some(opens) if opens > 0.0 => bound - push,
            Some(opens) if opens < 0.0 => bound + push,
            _ => bound - push.abs(),
        };
        return Some(Holding::HeldByFriction {
            touching: touched.map(|(_, r)| code(r.pair)),
            margin,
        });
    }
    // The stop the push drives the key into: a closed contact the motion it
    // pushes would close further.
    rows.rows()
        .iter()
        .enumerate()
        .find(|(i, r)| r.gap <= TOUCHING && rows.grad(*i)[angle] * push < 0.0)
        .map(|(_, r)| Holding::RestingOn { stop: code(r.pair) })
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
    /// determinate.
    #[test]
    fn a_weight_inside_the_band_leaves_the_key_where_it_is_held_by_friction() {
        let (k, bound) = bench();
        let m = &k.mounted.mechanism;
        let mut statics = Statics::new();
        let start = crate::parts::mounts::nominal_pose(&k.mounted);
        let rest = statics
            .solve(m, &start, &[weight(&k, 0.0)], &[], Friction::Off)
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
            let next = statics
                .solve(m, &q, &loads, &[], Friction::On)
                .expect("an increment solves")
                .q;
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
        let m = &k.mounted.mechanism;
        let mut statics = Statics::new();
        let start = crate::parts::mounts::nominal_pose(&k.mounted);
        for (grams, stop) in [
            (-20.0, ContactCode::BackRailCloth),
            (-7.0, ContactCode::BackRailCloth),
            (-6.0, ContactCode::FrontRailPunching),
            (20.0, ContactCode::FrontRailPunching),
        ] {
            let loads = [weight(&k, grams)];
            let q = statics
                .solve(m, &start, &loads, &[], Friction::Off)
                .expect("the key comes to rest")
                .q;
            assert_eq!(
                holding(&k.mounted, &q, &loads),
                Some(Holding::RestingOn { stop }),
                "{grams} g"
            );
        }
    }
}

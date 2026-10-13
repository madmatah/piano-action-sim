//! The seated pose: the key put on its frame by the trade's gestures, at the
//! top of its stroke and at the bottom. Key height and key dip are read from
//! it.
//!
//! # The gestures
//!
//! Friction off: a key is seated by rule, not left wherever a weight has put
//! it, so that what the sheet reads is the same whatever the user did with
//! the touch weight. In both seats a downward load at the capstan's contact
//! point stands for the action, which in the piano holds the back of the key
//! down on its cloth; for the bottom seat a press at the measuring point is
//! added to it, as a finger presses the key down. Gravity acts when the key
//! has a mass in the model; an unweighed key is seated by the two loads
//! alone.
//!
//! The magnitudes follow one frozen rule. The load at the capstan seats the
//! key at the top with [`SEAT_MARGIN`] of torque over gravity's generalized
//! force; the press seats it at the bottom with the same margin over gravity
//! and the load at the capstan together. Both are taken with the key level:
//! over its stroke, gravity's generalized force on a key changes by its
//! weight times the height of its centre of mass times the angle, a fraction
//! of the margin. With rigid stops the pose depends on neither magnitude,
//! only on which stops the loads put the key on
//! (`doubling_the_preparation_loads_changes_no_seat`).
//!
//! The top seat starts from the key's nominal pose
//! ([`crate::parts::mounts::nominal_pose`]), level wherever its drawn parts
//! overlap its frame there by no more than the row margin, and the bottom
//! seat from the top seat.
//!
//! # Key height and dip
//!
//! Read at the key front's top, an anchor on the keytop, the way a rule
//! standing on the keybed reads them: key height is that point's height at
//! the top seat; key dip, how far it falls from the top seat to the bottom.
//! Both are the positions the static solve puts the drawn parts in, so they
//! see whichever stop binds first: the front punching, the front pin
//! bottoming in its mortise, or the pin meeting the wood beside its slot.

use crate::contact::ContactCode;
use crate::geometry::Point;
use crate::mechanism::{MaterialPoint, PointLoad, Problem};
use crate::parts::mounts::Mounted;
use crate::solve::{Friction, Holder, Refusal, SolveReport, Statics};
use crate::G;

/// The torque, newton-metres, by which each preparation load seats the key
/// past what holds it the other way.
///
/// A margin, not a magnitude anything reads: a hundredth of a newton-metre
/// is the weight of about 4 g at the measuring point, enough that the seat
/// the loads choose does not hang on rounding, and with rigid stops the pose
/// does not depend on it.
pub const SEAT_MARGIN: f64 = 0.01;

/// How deep a seat's drawn parts may stand in one another, read off their
/// outlines, metres: a micron, a hundredth of the thinnest paper. A seat the
/// static solve converges on has every contact it reads closed to a
/// picometre; drawn parts deeper in one another than this are sunk where no
/// contact reads them, past an edge's centre or across a corner's sector
/// (`docs/FOUNDATIONS.md` §6.6), and the seat is not one.
pub const SEAT_OVERLAP: f64 = 1e-6;

/// The key on its frame as the procedure needs it: the mounted key, and the
/// two points of it the gestures load.
#[derive(Debug, Clone, Copy)]
pub struct Seating<'a> {
    /// The key on its frame.
    pub mounted: &'a Mounted,
    /// The capstan's contact point, in the key's frame: where the load
    /// standing for the action acts.
    pub capstan: Point,
    /// The measuring point, in the key's frame: where the press acts.
    pub measuring_point: Point,
    /// The margin each load seats the key by, newton-metres:
    /// [`SEAT_MARGIN`].
    pub margin: f64,
}

/// One seat.
#[derive(Debug, Clone, PartialEq)]
pub struct Seat {
    /// The key's coordinates in it.
    pub q: Vec<f64>,
    /// The contacts carrying the key there, in the mechanism's order of
    /// pairs: which rows are closed and loaded, which is determinate; how the
    /// load splits between them, where several are, is not.
    pub held_by: Vec<ContactCode>,
    /// The vertical load the key's bearing carries there, newtons, positive
    /// pushing the key up: the balance punching stays loaded while it is.
    pub bearing_load: f64,
    /// How the solve went.
    pub report: SolveReport,
}

/// The two seats, and the loads that put the key on them.
#[derive(Debug, Clone, PartialEq)]
pub struct Seats {
    /// At the top of the stroke: tail down on the cloth.
    pub top: Seat,
    /// At the bottom: front down on whichever stop binds first.
    pub bottom: Seat,
    /// The load at the capstan standing for the action, newtons.
    pub capstan_load: f64,
    /// The press at the measuring point at the bottom, newtons.
    pub press: f64,
}

/// Why the key cannot be seated.
#[derive(Debug, Clone, PartialEq)]
pub enum SeatRefusal {
    /// The mounted key cannot be solved as given.
    Mechanism(Problem),
    /// The capstan stands at or in front of the key's bearing, where a load
    /// on it cannot hold the back of the key down.
    CapstanNotBehindTheBearing,
    /// The measuring point stands at or behind the key's bearing, where a
    /// press cannot take the key down.
    MeasuringPointNotInFront,
    /// The key has no seat: these stops fight, leaving it no attitude that
    /// clears them all, or the attitude the solve found leaves them sunk in
    /// the key where no contact reads them. On a real keyboard a key whose
    /// stops fight rides on them and lifts off its balance punching, which
    /// the key on a pivot cannot do.
    CannotSeat {
        /// The contacts that fight, in the mechanism's order.
        fighting: Vec<ContactCode>,
    },
    /// Nothing holds the back of the key at the top: it turns to the edge of
    /// the modelled world.
    NoStopUnderTheTail,
    /// Nothing stops the front of the key at the bottom truthfully: a front
    /// punching whose centre stands past the key's underside bears on
    /// nothing, and the key turns to the edge of the modelled world, or comes
    /// down through the part of the punching drawn under it.
    NoStopBelowTheFront,
    /// The static solve reached its cap first.
    NotConverged {
        /// Steps taken.
        iterations: u32,
        /// The stationarity residual left, newtons.
        residual: f64,
    },
}

impl Seat {
    /// The height above the keybed of `point`, a point of the key in its
    /// frame, in this seat, metres.
    pub fn height(&self, mounted: &Mounted, point: Point) -> f64 {
        mounted
            .mechanism
            .pose(&self.q)
            .world(MaterialPoint {
                body: mounted.key,
                at: point,
            })
            .y
    }
}

impl Seats {
    /// The height above the keybed of `point`, a point of the key in its
    /// frame, at each seat, metres: at the top and at the bottom.
    pub fn heights(&self, mounted: &Mounted, point: Point) -> (f64, f64) {
        (
            self.top.height(mounted, point),
            self.bottom.height(mounted, point),
        )
    }
}

/// The preparation loads of the two seats, by the frozen rule.
struct Preparation {
    /// At the top: the load at the capstan.
    top: [PointLoad; 1],
    /// At the bottom: the load at the capstan and the press.
    bottom: [PointLoad; 2],
    /// The load at the capstan, newtons.
    capstan_load: f64,
    /// The press at the measuring point, newtons.
    press: f64,
}

impl Seating<'_> {
    /// Seats the key at the top and then at the bottom, as the procedure
    /// says, with `statics`.
    pub fn seats(&self, statics: &mut Statics) -> Result<Seats, SeatRefusal> {
        let m = &self.mounted.mechanism;
        let prepared = self.preparation()?;
        let nominal = crate::parts::mounts::nominal_pose(self.mounted);
        let top = self.seat(statics, &nominal, &prepared.top, true)?;
        let bottom = self.seat(statics, &top.q, &prepared.bottom, false)?;
        // A punching read at its centre bears on nothing once that centre
        // stands past the key's underside; if its drawn face stands under the
        // key all the same, the key has come down through it as drawn.
        let pose = m.pose(&bottom.q);
        let mut rows = crate::contact::Rows::new();
        m.rows(&pose, f64::INFINITY, &mut rows);
        for (p, pair) in m.pairs().iter().enumerate() {
            let p = u16::try_from(p).expect("fewer than 65 536 pairs");
            if pair.geometry == crate::contact::PairGeometry::CentreBearing
                && rows.gap(p).is_none()
                && m.face_overlap(&pose, p) > crate::parts::verdicts::TOUCHING
            {
                return Err(SeatRefusal::NoStopBelowTheFront);
            }
        }
        Ok(Seats {
            top,
            bottom,
            capstan_load: prepared.capstan_load,
            press: prepared.press,
        })
    }

    /// Seats the key at the top only, as [`Self::seats`] seats it there, the
    /// same loads from the same start, so the same bits: what key height is
    /// read from, without the bottom seat a procedure reading height alone
    /// does not need.
    pub fn top(&self, statics: &mut Statics) -> Result<Seat, SeatRefusal> {
        let prepared = self.preparation()?;
        let nominal = crate::parts::mounts::nominal_pose(self.mounted);
        self.seat(statics, &nominal, &prepared.top, true)
    }

    /// The preparation loads, by the frozen rule of the module documentation;
    /// refused where the capstan or the measuring point stands on the wrong
    /// side of the bearing for its load to seat the key.
    fn preparation(&self) -> Result<Preparation, SeatRefusal> {
        let m = &self.mounted.mechanism;
        let n = m.coords().len();
        let level = vec![0.0; n];
        let pose = m.pose(&level);
        let angle = self.mounted.angle;
        let key = self.mounted.key;
        let mut gravity = vec![0.0; n];
        m.generalized_forces(&pose, &[], &mut gravity);
        let gravity = gravity[angle.0];
        // A downward newton at a point turns the key by minus the vertical
        // speed that point has per radian.
        let per_newton = |at: Point| {
            -m.jacobian_column(&pose, MaterialPoint { body: key, at }, angle)
                .y
        };
        let (at_capstan, at_press) = (per_newton(self.capstan), per_newton(self.measuring_point));
        if at_capstan.is_nan() || at_capstan <= 0.0 {
            return Err(SeatRefusal::CapstanNotBehindTheBearing);
        }
        if at_press.is_nan() || at_press >= 0.0 {
            return Err(SeatRefusal::MeasuringPointNotInFront);
        }
        let capstan_load = (self.margin + (-gravity).max(0.0)) / at_capstan;
        let press = (self.margin + (gravity + capstan_load * at_capstan).max(0.0)) / -at_press;
        let down = |at: Point, newtons: f64| PointLoad {
            at: MaterialPoint { body: key, at },
            force: Point::new(0.0, -newtons),
        };
        Ok(Preparation {
            top: [down(self.capstan, capstan_load)],
            bottom: [
                down(self.capstan, capstan_load),
                down(self.measuring_point, press),
            ],
            capstan_load,
            press,
        })
    }

    /// One seat, from `from`, under `loads`, friction off.
    fn seat(
        &self,
        statics: &mut Statics,
        from: &[f64],
        loads: &[PointLoad],
        top: bool,
    ) -> Result<Seat, SeatRefusal> {
        let m = &self.mounted.mechanism;
        let code = |pair: u16| m.pairs()[usize::from(pair)].code;
        let eq =
            statics
                .solve(m, from, loads, &[], Friction::Off)
                .map_err(|refusal| match refusal {
                    Refusal::Mechanism(p) => SeatRefusal::Mechanism(p),
                    Refusal::NotConverged {
                        iterations,
                        residual,
                    } => SeatRefusal::NotConverged {
                        iterations,
                        residual,
                    },
                    Refusal::NoEquilibrium { on } => {
                        let mut fighting: Vec<ContactCode> = Vec::new();
                        let mut pairs: Vec<u16> = on
                            .iter()
                            .filter_map(|h| match *h {
                                Holder::Contact { pair, .. } => Some(pair),
                                Holder::Limit { .. } => None,
                            })
                            .collect();
                        pairs.sort_unstable();
                        pairs.dedup();
                        for pair in pairs {
                            fighting.push(code(pair));
                        }
                        if !fighting.is_empty() {
                            SeatRefusal::CannotSeat { fighting }
                        } else if top {
                            SeatRefusal::NoStopUnderTheTail
                        } else {
                            SeatRefusal::NoStopBelowTheFront
                        }
                    }
                })?;
        // Sunk where no contact reads: no seat.
        let pose = m.pose(&eq.q);
        let sunk: Vec<ContactCode> = (0..m.pairs().len())
            .filter(|&p| m.pair_overlap(&pose, u16::try_from(p).expect("a pair")) > SEAT_OVERLAP)
            .map(|p| m.pairs()[p].code)
            .collect();
        if !sunk.is_empty() {
            return Err(SeatRefusal::CannotSeat { fighting: sunk });
        }
        let mut held_by: Vec<ContactCode> = Vec::new();
        let mut lift = 0.0;
        let mut pairs: Vec<u16> = Vec::new();
        for s in &eq.supports {
            if let Holder::Contact { pair, .. } = s.by {
                if s.force > 0.0 {
                    pairs.push(pair);
                    lift += s.force * s.normal.y;
                }
            }
        }
        pairs.sort_unstable();
        pairs.dedup();
        for pair in pairs {
            held_by.push(code(pair));
        }
        // What the bearing carries: whatever the weights and the contacts
        // leave of the vertical balance.
        let weight = m.bodies()[self.mounted.key.0]
            .mass
            .map_or(0.0, |mass| mass.mass * G);
        let pressed: f64 = loads.iter().map(|l| l.force.y).sum();
        let bearing_load = weight - pressed - lift;
        Ok(Seat {
            q: eq.q,
            held_by,
            bearing_load,
            report: eq.report,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parts::assembly::nose_up_angle;
    use crate::parts::fixtures::{on_frame, regulated};
    use crate::parts::frame::KeyKind;
    use crate::solve::statics::{tests::assert_virtual_work, GAP_TOLERANCE};

    /// The key front's height at each seat, metres.
    fn heights(a: &crate::parts::KeyAssembly) -> (Seats, f64, f64) {
        let k = on_frame(a, KeyKind::Natural, 0.0, true);
        let seats = k
            .seating()
            .seats(&mut Statics::new())
            .expect("the key seats");
        let (top, bottom) = seats.heights(&k.mounted, k.front_top);
        (seats, top, bottom)
    }

    /// Consistency: **on the regulated bench the seats are where the
    /// one-coordinate statics put the key**: the same rest on the cloth's
    /// back edge and the same bottom on the front punching, so the same key
    /// height and dip. The two read the same drawn cloth. At the bottom the
    /// built stop is a disc of the punching's edge radius `r` under the
    /// felt's centre, which at a tilt `φ` reaches `r (1 − cos φ)` further
    /// than the centre does: picometres, on a key regulated level at the
    /// bottom to within a paper.
    ///
    /// The bound is derived. The static solve closes each gap within
    /// [`GAP_TOLERANCE`], which the arm from the cloth's edge to the key
    /// front, against the arm to the edge, turns into height; the closed form
    /// and the pose each round their last bits on numbers of the key front's
    /// height. The disc's reach, at the front rail 225 mm out, moves the
    /// bottom attitude by itself over that arm, and the dip by it through the
    /// lever to the key front.
    #[test]
    fn the_seats_are_where_the_built_statics_put_the_key() {
        let a = regulated();
        let (seats, top, bottom) = heights(&a);
        let t = crate::measure::oracle::Travelling::travel(&a).expect("a travel");
        let ratio = 0.258 / 0.200;
        let bound = 2.0 * GAP_TOLERANCE * ratio + 64.0 * f64::EPSILON * top;
        assert!(
            (top - t.key_height).abs() <= bound,
            "key height {top} against {}",
            t.key_height
        );
        let phi = seats.bottom.q[0].abs();
        let disc = crate::parts::assembly::PUNCHING_EDGE_RADIUS * (1.0 - libm::cos(phi));
        assert!(
            ((top - bottom) - t.key_dip).abs() <= 2.0 * bound + disc * 0.258 / 0.225,
            "key dip {} against {}",
            top - bottom,
            t.key_dip
        );
        assert_eq!(seats.top.held_by, vec![ContactCode::BackRailCloth]);
        assert_eq!(seats.bottom.held_by, vec![ContactCode::FrontRailPunching]);
        let built = nose_up_angle(t.sin_bottom);
        assert!(
            (seats.bottom.q[0] - built).abs()
                <= (disc + 2.0 * GAP_TOLERANCE) / 0.225 + 8.0 * f64::EPSILON,
            "the bottom at {} rad, the built statics' at {built} rad",
            seats.bottom.q[0]
        );
    }

    /// The balance punching stays loaded in both seats: the load at the
    /// capstan keeps the bearing carrying the key, its weight and the press,
    /// less what the cloth or the front stop takes.
    #[test]
    fn the_balance_punching_stays_loaded_in_both_seats() {
        let (seats, ..) = heights(&regulated());
        for seat in [&seats.top, &seats.bottom] {
            assert!(seat.bearing_load > 0.0, "{seat:?}");
        }
        assert!(seats.capstan_load > 0.0 && seats.press > 0.0);
    }

    /// C12, consistency: **the preparation loads choose the seat, not the
    /// pose.** Doubled, they put the key on the same stops at the same
    /// attitudes, within what the solve converges to.
    #[test]
    fn doubling_the_preparation_loads_changes_no_seat() {
        let a = regulated();
        let k = on_frame(&a, KeyKind::Natural, 0.0, true);
        let once = k.seating().seats(&mut Statics::new()).unwrap();
        let twice = Seating {
            margin: 2.0 * SEAT_MARGIN,
            ..k.seating()
        }
        .seats(&mut Statics::new())
        .unwrap();
        let bound = 2.0 * GAP_TOLERANCE / 0.2;
        for (one, two) in [(&once.top, &twice.top), (&once.bottom, &twice.bottom)] {
            assert_eq!(one.held_by, two.held_by);
            assert!(
                (one.q[0] - two.q[0]).abs() <= bound,
                "{one:?} against {two:?}"
            );
        }
        assert!(twice.capstan_load > once.capstan_load);
    }

    /// C12, consistency: **the metric shapes the path, not the seat.** Its
    /// scale multiplied by ten, the seats are the same, within what the
    /// solve converges to.
    #[test]
    fn scaling_the_metric_changes_no_seat() {
        let a = regulated();
        let k = on_frame(&a, KeyKind::Natural, 0.0, true);
        let one = k.seating().seats(&mut Statics::new()).unwrap();
        let ten = k
            .seating()
            .seats(&mut Statics::with_metric_scaled(10.0))
            .unwrap();
        let bound = 2.0 * GAP_TOLERANCE / 0.2;
        for (x, y) in [(&one.top, &ten.top), (&one.bottom, &ten.bottom)] {
            assert_eq!(x.held_by, y.held_by);
            assert!((x.q[0] - y.q[0]).abs() <= bound, "{x:?} against {y:?}");
        }
    }

    /// An unweighed key is seated by the preparation loads alone, on the
    /// same stops as the weighed one: with rigid stops, its key height and
    /// dip do not depend on its mass.
    #[test]
    fn an_unweighed_key_is_seated_by_its_preparation_loads() {
        let a = regulated();
        let weighed = on_frame(&a, KeyKind::Natural, 0.0, true);
        let bare = on_frame(&a, KeyKind::Natural, 0.0, false);
        let w = weighed.seating().seats(&mut Statics::new()).unwrap();
        let b = bare.seating().seats(&mut Statics::new()).unwrap();
        assert_eq!(w.top.held_by, b.top.held_by);
        assert_eq!(w.bottom.held_by, b.bottom.held_by);
        let bound = 2.0 * GAP_TOLERANCE / 0.2;
        assert!((w.top.q[0] - b.top.q[0]).abs() <= bound);
        assert!((w.bottom.q[0] - b.bottom.q[0]).abs() <= bound);
    }

    /// C6, consistency: each seat is an equilibrium of its loads, by the
    /// unilateral virtual-work inequality.
    #[test]
    fn each_seat_is_an_equilibrium_of_its_loads() {
        let a = regulated();
        let k = on_frame(&a, KeyKind::Natural, 0.0, true);
        let seats = k.seating().seats(&mut Statics::new()).unwrap();
        let key = k.mounted.key;
        let down = |at: Point, f: f64| PointLoad {
            at: MaterialPoint { body: key, at },
            force: Point::new(0.0, -f),
        };
        let top = [down(k.capstan, seats.capstan_load)];
        let bottom = [
            down(k.capstan, seats.capstan_load),
            down(k.measuring_point, seats.press),
        ];
        for (seat, loads) in [(&seats.top, &top[..]), (&seats.bottom, &bottom[..])] {
            let eq = Statics::new()
                .solve(&k.mounted.mechanism, &seat.q, loads, &[], Friction::Off)
                .expect("a seat is an equilibrium");
            assert_eq!(
                eq.q[0].to_bits(),
                seat.q[0].to_bits(),
                "solved again, it stays"
            );
            assert_virtual_work(
                &k.mounted.mechanism,
                &eq,
                loads,
                Friction::Off,
                &eq.q.clone(),
                &[0],
                1e-6,
                0x5ea7,
            );
        }
    }

    /// The front pin bottoming in its mortise stops the key before its
    /// punching: one pin a millimetre prouder, and it is the pin that holds
    /// the bottom seat, on one of its corners, the dip shorter by the
    /// millimetre seen through the lever.
    #[test]
    fn a_pin_too_proud_holds_the_bottom_seat() {
        let base = regulated();
        let mut proud = base.clone();
        proud.frame.sections.front.pin_height += 0.001;
        let (_, top, bottom) = heights(&base);
        let (seats, p_top, p_bottom) = heights(&proud);
        assert_eq!(
            seats.bottom.held_by,
            vec![ContactCode::FrontPinInTheMortise]
        );
        assert!((p_top - top).abs() < 1e-12, "the rest does not move");
        assert!(p_top - p_bottom < top - bottom, "the dip is shorter");
    }

    /// A pin standing under the key's wood, outside its mortise, meets the
    /// wood beside the slot: on a frame whose cloth lets the key stand
    /// nose-up enough to clear it, the key rests on the cloth, and the pin,
    /// not the punching, holds the bottom of a dip it cuts short. On the
    /// frame as built, whose cloth does not, the key has no seat on a pivot:
    /// it would ride on the pin and the cloth.
    #[test]
    fn a_pin_under_the_wood_beside_its_mortise_holds_the_key() {
        let mut short = regulated();
        short.key.front_edge -= 0.018;
        let k = on_frame(&short, KeyKind::Natural, 0.0, true);
        match k.seating().seats(&mut Statics::new()) {
            Err(SeatRefusal::CannotSeat { fighting }) => {
                assert!(
                    fighting.contains(&ContactCode::FrontPinOnTheWood),
                    "{fighting:?}"
                );
            }
            other => panic!("{other:?}"),
        }
        short.frame.back_rail_height -= 0.015;
        let (seats, top, bottom) = heights(&short);
        assert_eq!(seats.top.held_by, vec![ContactCode::BackRailCloth]);
        assert_eq!(seats.bottom.held_by, vec![ContactCode::FrontPinOnTheWood]);
        assert!(
            top - bottom > 0.001 && top - bottom < 0.010,
            "a dip cut short: {}",
            top - bottom
        );
    }

    /// A frame whose stops fight, its cloth holding the tail 5 mm above the
    /// balance bearing while the front punching stands level with it, gives
    /// the key no seat on a pivot, and the refusal names the two stops: on a
    /// real keyboard the key rides on both and lifts off its balance
    /// punching.
    #[test]
    fn a_frame_whose_stops_fight_names_them() {
        let mut a = regulated();
        a.frame.back_rail_height =
            a.frame.balance_bearing_height() + 0.005 - a.frame.back_rail_cloth;
        let k = on_frame(&a, KeyKind::Natural, 0.0, true);
        match k.seating().seats(&mut Statics::new()) {
            Err(SeatRefusal::CannotSeat { fighting }) => {
                assert!(
                    fighting.contains(&ContactCode::BackRailCloth),
                    "{fighting:?}"
                );
                assert!(
                    fighting.contains(&ContactCode::FrontRailPunching),
                    "{fighting:?}"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    /// A pin straddling its slot's edge, the frame's front rail 3 mm forward
    /// of the mortise, meets the wood at the slot's mouth: the key cannot
    /// come down over it onto its cloth.
    #[test]
    fn a_pin_straddling_its_slots_edge_meets_the_wood() {
        let mut a = regulated();
        a.frame.front_rail += 0.003;
        let k = on_frame(&a, KeyKind::Natural, 0.0, true);
        match k.seating().seats(&mut Statics::new()) {
            Err(SeatRefusal::CannotSeat { fighting }) => {
                assert!(
                    fighting.contains(&ContactCode::FrontPinOnTheWood),
                    "{fighting:?}"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    /// A key on the sharp row stands on the sharp row's punching and pin,
    /// where its frame draws them: the bottom seat's contact is at the
    /// drawn punching's centre, the row offset behind the natural's. The
    /// sharp's mortise is cut over its row's pin.
    #[test]
    fn a_sharp_bottoms_on_the_punching_drawn_on_its_row() {
        for kind in [KeyKind::Natural, KeyKind::Sharp] {
            let mut a = regulated();
            if kind == KeyKind::Sharp {
                a.key.front_mortise_from_front_edge += a.frame.sections.front.sharp_row_offset;
            }
            let k = on_frame(&a, kind, 0.0, true);
            let seats = k
                .seating()
                .seats(&mut Statics::new())
                .unwrap_or_else(|r| panic!("{kind:?}: {r:?}"));
            assert_eq!(seats.bottom.held_by, vec![ContactCode::FrontRailPunching]);
            let m = &k.mounted.mechanism;
            let pose = m.pose(&seats.bottom.q);
            let mut rows = crate::contact::Rows::new();
            m.rows(&pose, crate::contact::ROW_MARGIN, &mut rows);
            let front = m
                .pairs()
                .iter()
                .position(|p| p.code == ContactCode::FrontRailPunching)
                .unwrap();
            let centre = m.parts()[k.frame.front_rail_punching.index()].sweep;
            let crate::contact::Sweep::Revolved { axis_x } = centre else {
                panic!("a punching is turned")
            };
            let expected = a.frame.sections.front.pin_x(a.frame.front_rail, kind);
            assert_eq!(
                axis_x.to_bits(),
                expected.to_bits(),
                "{kind:?}: drawn on its row"
            );
            let i = rows
                .governing(u16::try_from(front).unwrap())
                .expect("the punching bears");
            assert!(
                (rows.rows()[i].point.x - expected).abs() < 1e-9,
                "{kind:?}: the punching bears at {} m, drawn at {expected} m",
                rows.rows()[i].point.x
            );
        }
    }

    /// Determinism: the same seats twice are the same bits.
    #[test]
    fn the_same_seats_twice_are_the_same_bits() {
        let a = regulated();
        let k = on_frame(&a, KeyKind::Natural, 0.0, true);
        let mut statics = Statics::new();
        let one = k.seating().seats(&mut statics).unwrap();
        let two = k.seating().seats(&mut statics).unwrap();
        assert_eq!(one, two);
        assert_eq!(one.bottom.q[0].to_bits(), two.bottom.q[0].to_bits());
    }
}

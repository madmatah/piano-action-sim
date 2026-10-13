//! The mechanisms a key is put in to be read, and the contacts it has with
//! its frame.
//!
//! A mount is a builder, not a type: the same key, with the same mass, goes
//! into whichever mechanism a procedure needs, and the procedure reads it
//! there. Built: the jig; the key on its frame free to lift off its balance
//! punching, [`KeyMount::OnPunching`], the mount every procedure on the frame
//! reads; and the key turning about a fixed axis through its balance bearing,
//! [`KeyMount::Pivot`], kept for the comparisons with an action published
//! with its key on a pivot.
//!
//! # The key on its punching
//!
//! A key is not pinned to its frame. It rests on its balance punching, which
//! can push it up and cannot hold it down, and its balance pin only guides
//! it. So the key on its frame has two coordinates: a slide carrying the key
//! up and down the balance pin's axis, and the key's angle about the slide's
//! origin. The balance punching is a unilateral contact like the other stops,
//! read at the centre of its drawn top. A frame whose stops fight, a key lifted
//! off its punching and a key seated on it come out of the same static solve:
//! the key rides on its two outer stops off the punching where they hold it
//! higher than the punching would, on the parts as drawn.
//!
//! # Solved on its punching first
//!
//! While the punching carries the key, the key turns about the punching's
//! centre, which stands on the slide's axis: the key on its punching is then
//! the key on a pivot there. [`Mounted::solve`] tries that first, the slide
//! held on the punching and every other contact read as it is. Where the
//! punching then pushes the key up, that equilibrium meets every condition of
//! the full problem, the punching's multiplier being the slide's reaction, and
//! it is to the bit the one the key on a pivot gives. Where the punching would
//! have to pull, or the key stands off it, the punching is released and the
//! full problem solved.

use super::frame::{FrameDesign, FrameRanges, KeyKind};
use super::key::KeyRanges;
use super::KeyDesign;
use crate::contact::{
    ContactCode, ContactLaw, ContactPair, PairGeometry, Part, PartCode, PartId, Surface, ROW_MARGIN,
};
use crate::geometry::{Dir, Point};
use crate::mechanism::{
    Body, BodyId, Coord, CoordId, Joint, MassProps, Mechanism, PointLoad, Prescribed, Problem,
};
use crate::solve::statics::GAP_TOLERANCE;
use crate::solve::{Equilibrium, Friction, Holder, Refusal, SolveReport, Statics};

/// A key in a mechanism, and where it is in it.
#[derive(Debug, Clone, PartialEq)]
pub struct Mounted {
    /// The mechanism.
    pub mechanism: Mechanism,
    /// The key's body. Its frame is the key's own: origin at the balance pin
    /// on the key's underside, `x` towards the player.
    pub key: BodyId,
    /// The key's angle, positive nose-up.
    pub angle: CoordId,
    /// The key's freedom to lift off its balance punching, on a key free to
    /// ([`KeyMount::OnPunching`]); `None` on the jig and on a pivot.
    pub lift: Option<Lift>,
}

/// A key's freedom to lift off its balance punching.
#[derive(Debug, Clone, PartialEq)]
pub struct Lift {
    /// The slide's coordinate: the height of the key frame's origin above
    /// the keybed, metres.
    pub coord: CoordId,
    /// The slide's value with the key on its balance punching: the height of
    /// the punching's drawn top, metres.
    pub on_punching: f64,
    /// The balance punching's pair: the mechanism's last.
    pub pair: u16,
    /// The key held on its punching: the same bodies, coordinates and parts,
    /// every pair but the punching's, solved with the slide held at
    /// [`Self::on_punching`]. `None` where the punching's centre does not
    /// stand on the slide's axis, where the key held on it would not be the
    /// key turning about it.
    turning: Option<Mechanism>,
}

/// Stanwood's wedge jig: the key turning about its balance pin's axis, and
/// nothing else.
///
/// Two bodies: the ground, which is the jig, and the key, hung from it by a
/// revolute joint at the origin of the key's frame, on the balance pin's
/// axis. The joint turns without friction, which is an assumption: the real
/// key turns on an extra-long balance pin through its bushings. `mass` is the
/// key's, `None` for a key nobody has weighed.
pub fn jig(mass: Option<MassProps>) -> Result<Mounted, Problem> {
    let key = Body {
        parent: BodyId::GROUND,
        joint: Joint::Revolute {
            pivot: Point::new(0.0, 0.0),
            coord: CoordId(0),
        },
        mass,
    };
    Ok(Mounted {
        mechanism: Mechanism::new(vec![Body::GROUND, key], vec![Coord::ANGLE])?,
        key: BodyId(1),
        angle: CoordId(0),
        lift: None,
    })
}

/// How a key stands on its frame while it is read.
///
/// The same key parts and the same mass go into whichever mount a procedure
/// needs; the mount says what the key may do there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyMount {
    /// Turning about a fixed axis through its balance bearing, the one
    /// coordinate its angle, positive nose-up, within
    /// `±`[`KEY_ANGLE_LIMIT`], its bushings' friction acting on it.
    /// Hirschkorn's own simplification of the key, "like every joint, a
    /// simple revolute joint" (`data/params/hirschkorn-boston-gp178-note52.json`,
    /// note on Pg1), kept to compare with an action published so. The key
    /// cannot lift off its balance punching on it: a frame whose stops fight
    /// leaves it no angle at all.
    Pivot,
    /// Free to lift off its balance punching ([`on_punching`]): a massless
    /// slide up the balance pin's axis, carrying the key turning about the
    /// slide's origin, and the punching a unilateral contact read at the
    /// centre of its drawn top. The mount every procedure on the frame reads.
    OnPunching,
}

impl KeyMount {
    /// The key's body in this mount: the ground's child on a pivot, the
    /// slide's on its punching.
    pub fn key_body(self) -> BodyId {
        match self {
            Self::Pivot => BodyId(1),
            Self::OnPunching => BodyId(2),
        }
    }
}

/// How far the key may turn either way on its frame, radians: 15°, the edge
/// of the modelled world. Far past any stroke (a key's dip turns it about
/// 2.3°), it bounds what a static solve searches, and a key that leans on it
/// stands on nothing modelled.
pub const KEY_ANGLE_LIMIT: f64 = core::f64::consts::PI / 12.0;

/// The steps of the scan [`nominal_pose`] makes over the key's range: a
/// degree.
pub(crate) const NOMINAL_STEP: f64 = core::f64::consts::PI / 180.0;

/// How many golden-section steps narrow the attitude at which a key lowered
/// onto its frame from above, by [`nominal_pose`] or by
/// [`lowered_onto_its_frame`], clears it, where no attitude a whole degree
/// apart clears below where the key leaves its balance pin. Each step keeps
/// 0.618 of the bracket, two degrees wide: 22 steps leave under a
/// microradian, under a micron at the front of a key half a metre long,
/// where the lift a whole degree's step costs at the key's stops, up to half
/// a degree times their arm from the pin, is a millimetre or two.
pub(crate) const ATTITUDE_REFINEMENTS: u32 = 22;

/// The ratio of the golden section, `(√5 − 1) / 2`: the part of its bracket
/// each step of [`ATTITUDE_REFINEMENTS`] keeps.
const GOLDEN: f64 = 0.618_033_988_749_894_9;

/// In how many pieces of the static solve's cap of steps the problem of a
/// key free to lift may be solved, each going on from where the last
/// stopped ([`Mounted::solve`]): four.
///
/// A key turning on a closed stop moves its farthest point a row margin a
/// step at most, every step bounded by the stop's margin, and a key lifting
/// off its punching under a pull turns up on its cloth until it leaves its
/// balance pin ([`Mounted::leaves_its_pin`]): some 20 mm at its front on
/// the default frame. A key riding on a front pin standing in its wood,
/// solved from where it stands level on its punching, takes three quarters
/// of a cap to come to rest with every row that carries it closed
/// (`a_load_on_a_row_standing_open_is_no_rest`). Four caps hold either
/// with room; a seat of the corpus takes a few steps.
pub const FREE_PIECES: u32 = 4;

impl Mounted {
    /// The key level, and on its balance punching where it is free to lift:
    /// every coordinate zero but the slide, at the punching.
    pub fn level(&self) -> Vec<f64> {
        let mut q = vec![0.0; self.mechanism.coords().len()];
        if let Some(lift) = &self.lift {
            q[lift.coord.0] = lift.on_punching;
        }
        q
    }

    /// How far above its balance punching the key stands at `q`, metres:
    /// the slide less its value on the punching; zero on a mount with no
    /// slide.
    pub fn lift_at(&self, q: &[f64]) -> f64 {
        self.lift
            .as_ref()
            .map_or(0.0, |lift| q[lift.coord.0] - lift.on_punching)
    }

    /// How far above its balance punching the key leaves its balance pin,
    /// metres: the top of the slide's range, where the key's balance hole
    /// clears the pin's drawn top ([`on_punching`]), less the slide's value
    /// on the punching. Negative where the punching stands above the pin's
    /// top, the key on it already off its pin; `None` on a mount with no
    /// slide.
    pub fn leaves_its_pin(&self) -> Option<f64> {
        self.lift.as_ref().map(|lift| {
            let (_, top) = self.mechanism.coords()[lift.coord.0].limits;
            top - lift.on_punching
        })
    }

    /// Whether `refusal` is the key lifting off its frame: an increment that
    /// came to rest leaning on the slide's upper limit, where the key leaves
    /// its balance pin ([`Self::leaves_its_pin`]), or that has the key on a
    /// punching standing above the pin's top.
    pub fn lifts_off_its_frame(&self, refusal: &Refusal) -> bool {
        let Some(lift) = &self.lift else {
            return false;
        };
        match refusal {
            Refusal::NoEquilibrium { on } => on.contains(&Holder::Limit {
                coord: lift.coord,
                upper: true,
            }),
            _ => false,
        }
    }

    /// This key with its bushings' friction bounded by `bound` newton-metres
    /// instead, everything else kept bit for bit; or why not
    /// ([`Mechanism::with_friction`]).
    pub fn with_friction(mut self, bound: f64) -> Result<Self, Problem> {
        self.mechanism = self.mechanism.with_friction(self.angle, bound)?;
        if let Some(lift) = &mut self.lift {
            if let Some(turning) = lift.turning.take() {
                lift.turning = Some(turning.with_friction(self.angle, bound)?);
            }
        }
        Ok(self)
    }

    /// One load increment of the key from `from`, under `loads`, friction on
    /// or off: [`Statics::solve`] on the mechanism, nothing prescribed.
    ///
    /// On a key free to lift, standing on its punching to within the static
    /// solve's gap tolerance, the key is first solved on its punching: the
    /// slide held there, every contact but the punching's read. Where the
    /// slide's reaction then pushes the key up, or is nothing, that
    /// equilibrium is the full problem's: the punching's gap is closed, its
    /// multiplier that reaction over the cosine of the key's angle, and its
    /// centre on the slide's axis gives the angle nothing to carry. The
    /// equilibrium returned then holds that push as the slide's reaction, and
    /// its supports do not list the punching.
    ///
    /// The punching is released, and the full problem solved from `from`,
    /// where it would have to pull the key down, or where the key's stops
    /// fight on its punching, leaving it no motion: a key free to lift may
    /// ride on them. Any other refusal of the key on its punching is the full
    /// problem's too, and is returned: a key that turns onto the edge of its
    /// range with nothing to stop it, off its punching or on it, has no
    /// equilibrium either way. A key standing off its punching is solved free
    /// from the start.
    ///
    /// The full problem is not solved from a start whose drawn parts stand in
    /// one another deeper than the row margin, which no row reads truthfully:
    /// what the pairs read ([`Mechanism::overlap`]), and the key in any part
    /// of its frame, a punching's felt and papers whole, so that a key sunk
    /// right through a felt thicker than itself, which leaves the punching's
    /// centre clear, is not one. There the refusal of the key on its punching
    /// stands. It is solved in up to [`FREE_PIECES`] pieces of the static
    /// solve's cap of steps, each going on from where the last stopped.
    ///
    /// A key whose balance punching stands above its balance pin's top has
    /// no pose on its pin that clears the punching: it is refused as leaning
    /// on the slide's upper limit, off its frame, whatever the loads, and
    /// nothing is solved.
    pub fn solve(
        &self,
        statics: &mut Statics,
        from: &[f64],
        loads: &[PointLoad],
        friction: Friction,
    ) -> Result<Equilibrium, Refusal> {
        if let Some(lift) = &self.lift {
            if self.leaves_its_pin().is_some_and(|room| room < 0.0) {
                return Err(Refusal::NoEquilibrium {
                    on: vec![Holder::Limit {
                        coord: lift.coord,
                        upper: true,
                    }],
                });
            }
            if let Some(turning) = &lift.turning {
                let c = lift.coord.0;
                if (from[c] - lift.on_punching).abs() <= GAP_TOLERANCE {
                    let held = [Prescribed {
                        coord: lift.coord,
                        value: lift.on_punching,
                    }];
                    match statics.solve(turning, from, loads, &held, friction) {
                        Ok(eq) if eq.reactions[c] >= 0.0 => return Ok(eq),
                        Ok(_) => {}
                        Err(Refusal::NoEquilibrium { on })
                            if on.iter().any(|h| matches!(h, Holder::Contact { .. })) =>
                        {
                            if sunk_in_its_frame(self, from) > ROW_MARGIN {
                                return Err(Refusal::NoEquilibrium { on });
                            }
                        }
                        Err(refusal) => return Err(refusal),
                    }
                }
            }
        }
        self.solve_free(statics, from, loads, friction)
    }

    /// The full problem, the key free to lift, from `from`: [`Statics::solve`]
    /// on the mechanism, going on from where it stopped at its cap, up to
    /// [`FREE_PIECES`] pieces in all, as one increment along its path; its
    /// report one increment's, its steps all the pieces'.
    fn solve_free(
        &self,
        statics: &mut Statics,
        from: &[f64],
        loads: &[PointLoad],
        friction: Friction,
    ) -> Result<Equilibrium, Refusal> {
        let mut start = from.to_vec();
        let mut before = SolveReport::NOTHING;
        for piece in 1..=FREE_PIECES {
            match statics.solve(&self.mechanism, &start, loads, &[], friction) {
                Err(Refusal::NotConverged { .. }) if piece < FREE_PIECES => {
                    let (at, report) = statics.stopped();
                    start = at.to_vec();
                    before = before.continued(report);
                }
                Ok(mut eq) => {
                    eq.report = before.continued(eq.report);
                    return Ok(eq);
                }
                Err(Refusal::NotConverged {
                    iterations,
                    residual,
                }) => {
                    return Err(Refusal::NotConverged {
                        iterations: before.iterations + iterations,
                        residual,
                    });
                }
                Err(refusal) => return Err(refusal),
            }
        }
        unreachable!("the last piece returns")
    }
}

/// What each contact row carries in one equilibrium of a mounted key: the
/// rows the static solve read, with the force each pushed with, and the
/// balance punching's push where the solve held the key on it.
///
/// One admissible split of the loads, the one the solve returned: where
/// friction or redundant rows share a load, rigid statics leave the split
/// open (`docs/FOUNDATIONS.md` §2.1), and no verdict is read from it
/// ([`crate::parts::verdicts`]). It is what a view draws to show where the
/// key bears and roughly how hard.
#[derive(Debug, Clone, PartialEq)]
pub struct Carried {
    supports: Vec<crate::solve::Support>,
    /// The balance punching's pair and its push, newtons, where the solve
    /// held the key on its punching and the punching's rows are not among
    /// the supports.
    punching: Option<(u16, f64)>,
}

impl Carried {
    /// Nothing carried: a key that no equilibrium holds.
    pub const NOTHING: Self = Self {
        supports: Vec::new(),
        punching: None,
    };

    /// The force the row of `pair` meeting at `feature` pushes with,
    /// newtons: zero for a row the equilibrium left open or did not read.
    pub fn force(&self, pair: u16, feature: crate::contact::Feature) -> f64 {
        match self.punching {
            Some((p, push)) if p == pair => push,
            _ => self
                .supports
                .iter()
                .find(|s| s.by == Holder::Contact { pair, feature })
                .map_or(0.0, |s| s.force),
        }
    }

    /// The rows the equilibrium read, in the solve's order, each with its
    /// gap, its force and its normal.
    pub fn supports(&self) -> &[crate::solve::Support] {
        &self.supports
    }

    /// The balance punching's pair and its push, newtons, where the solve
    /// held the key on its punching and the punching's rows are not among
    /// the supports.
    pub fn punching(&self) -> Option<(u16, f64)> {
        self.punching
    }
}

impl Mounted {
    /// What each row carries in `eq`, an equilibrium of this key
    /// ([`Self::solve`]). Where the solve held the key on its punching, the
    /// slide's reaction is the punching's push straight up, and its
    /// multiplier along the underside's normal, tilted with the key, that
    /// reaction over the cosine of the key's angle.
    pub fn carried(&self, eq: &Equilibrium) -> Carried {
        let punching = self.lift.as_ref().and_then(|lift| {
            let listed = eq
                .supports
                .iter()
                .any(|s| matches!(s.by, Holder::Contact { pair, .. } if pair == lift.pair));
            let reaction = eq.reactions[lift.coord.0];
            (!listed && reaction != 0.0)
                .then(|| (lift.pair, reaction / libm::cos(eq.q[self.angle.0])))
        });
        Carried {
            supports: eq.supports.clone(),
            punching,
        }
    }
}

/// The key's nominal pose on its frame, where a static solve that seats it
/// by rule starts.
///
/// On its balance punching, level, wherever its drawn parts overlap its
/// frame there by no more than the row margin; otherwise the nearest
/// attitude to level, a degree apart and nose-up before nose-down, at which
/// they do; either way an overlap a solve resolves through its rows. A key
/// free to lift that has no such attitude on its punching, its stops
/// fighting or a pin standing in its wood, is lowered onto its frame from
/// above instead, at the attitude that clears its frame standing least far
/// off its punching: at each attitude the least lift that clears, found by
/// halving the slide's range to within the row margin, the attitude narrowed
/// between two whole degrees, by golden-section steps, where no whole degree
/// clears. On a pivot, or where nothing clears below where the key leaves
/// its balance pin ([`Mounted::leaves_its_pin`]) at any attitude, whole or
/// narrowed, the attitude of least overlap on the punching, the nearest
/// level among equals.
///
/// The overlap is read off the drawn outlines, not the rows: a static solve
/// resolves an overlap by its contacts' rows, which read a part sunk deep
/// into another truthfully only near the surface, or not at all. It is the
/// deepest of what the declared pairs read ([`Mechanism::overlap`]), of how
/// deep the key stands in its front and back rails, which no pair declares,
/// and of how far below the keybed it reaches: a key turned until it has
/// gone right through a stop leaves the stop's parts clear, and only the
/// rail under the stop says it is there; turned further, through the rail
/// too, only the keybed the rails stand on does.
pub fn nominal_pose(mounted: &Mounted) -> Vec<f64> {
    let level = mounted.level();
    // The attitudes are scanned on the key held on its punching where it is
    // free to lift: the punching's pair reads nothing there, and the scan is
    // then the pivot's, to the bit.
    let on_punching = mounted
        .lift
        .as_ref()
        .and_then(|lift| lift.turning.as_ref())
        .unwrap_or(&mounted.mechanism);
    let (q, least) = attitude_of_least_overlap(mounted, on_punching, &level);
    match &mounted.lift {
        Some(lift) if least > ROW_MARGIN => {
            let (key, rails) = key_and_rails(mounted);
            lowered(mounted, lift, &level, (&key, &rails), ROW_MARGIN).unwrap_or(q)
        }
        _ => q,
    }
}

/// The key lowered onto its frame from above, where it is free to lift: of
/// the attitudes [`nominal_pose`] scans, or narrowed between two of them by
/// golden-section steps where none clears, the one at which it stands clear
/// of every part of its frame, nothing overlapping at all, least far off its
/// punching, within the row margin above where it first touches. `None` on a
/// mount with no slide, and where nothing clears below where the key leaves
/// its balance pin ([`Mounted::leaves_its_pin`]): every pose clear of the
/// frame then has the key off its pin.
///
/// Clear of every part, the punchings' felts and papers whole: the pairs read
/// a punching at its centre, which a key sunk right through a felt thicker
/// than the key leaves clear. A key tilted on a punching clears its felt's
/// drawn edge only half the washer's width times the tilt above where it
/// stands on the felt's centre, so one riding that close under its pin's top
/// is not lowered onto its frame below it. The balance rail pin alone is
/// left out, which stands hidden in the key's wood, the balance mortise not
/// being drawn.
///
/// The start a placement goes back to when the nominal pose, an attitude on
/// the punching whose overlap the rows read within their margin, gives the
/// key no seat: an overlap no motion of the key on its frame resolves, such
/// as a front rail pin standing a few tenths of a millimetre into the wood
/// beside its mortise, its top well up in the slot, which overlaps the wood
/// along the key where no turn moves the wood along the key and no lift
/// short of the pin's top clears it; or a nominal pose lowered onto its
/// frame at the top of the slide's range, a whole degree from where the key
/// rides below it, from which the solve carries the key off its pin. A
/// technician lowers such a key onto its frame, and it lands on what stands
/// highest.
pub fn lowered_onto_its_frame(mounted: &Mounted) -> Option<Vec<f64>> {
    let lift = mounted.lift.as_ref()?;
    let (key, frame) = key_and_frame(mounted);
    lowered(mounted, lift, &mounted.level(), (&key, &frame), 0.0)
}

/// How deep the key stands in its frame at `q`, metres: what the pairs read,
/// the key's parts in every part of its frame but the balance rail pin
/// ([`lowered_onto_its_frame`]), and how far the key reaches below the
/// keybed.
fn sunk_in_its_frame(mounted: &Mounted, q: &[f64]) -> f64 {
    let (key, frame) = key_and_frame(mounted);
    overlap_at(&mounted.mechanism, &key, &frame, q)
}

/// How deep the key's drawn parts stand in its frame's at `q`, metres, read
/// as [`nominal_pose`] reads them: what the pairs read, the key in its front
/// and back rails, which no pair declares, and how far it reaches below the
/// keybed.
///
/// The depth an increment's start is held to: a rail raised into the key
/// where it stood, its cloth clear above the key, is read here and by no
/// row.
pub fn overlap_on_its_frame(mounted: &Mounted, q: &[f64]) -> f64 {
    let (key, rails) = key_and_rails(mounted);
    overlap_at(&mounted.mechanism, &key, &rails, q)
}

/// The parts of `mounted` that `keep` keeps, in their order.
fn parts_where(mounted: &Mounted, keep: &dyn Fn(&Part) -> bool) -> Vec<PartId> {
    let m = &mounted.mechanism;
    (0..m.parts().len())
        .filter(|&i| keep(&m.parts()[i]))
        .map(|i| PartId(u16::try_from(i).expect("fewer than 65 536 parts")))
        .collect()
}

/// The parts of `mounted`'s key, and its frame's front and back rails, which
/// no pair declares.
fn key_and_rails(mounted: &Mounted) -> (Vec<PartId>, Vec<PartId>) {
    let key = parts_where(mounted, &|p| p.body == mounted.key);
    let rails = parts_where(mounted, &|p| {
        p.body != mounted.key && matches!(p.code, PartCode::FrontRail | PartCode::BackRail)
    });
    (key, rails)
}

/// The parts of `mounted`'s key, and every part of its frame but the balance
/// rail pin, which stands hidden in the key's wood.
fn key_and_frame(mounted: &Mounted) -> (Vec<PartId>, Vec<PartId>) {
    let key = parts_where(mounted, &|p| p.body == mounted.key);
    let frame = parts_where(mounted, &|p| {
        p.body != mounted.key && p.code != PartCode::BalanceRailPin
    });
    (key, frame)
}

/// How deep the drawn parts stand in one another at `q`, read on `m`: what
/// its pairs read, the key in its rails, and how far the key's outline
/// reaches below the keybed, the ground's level zero.
fn overlap_at(m: &Mechanism, key: &[PartId], rails: &[PartId], q: &[f64]) -> f64 {
    let pose = m.pose(q);
    let below = key
        .iter()
        .flat_map(|id| {
            let part = &m.parts()[id.index()];
            let placement = pose.placement(part.body);
            part.silhouette()
                .into_iter()
                .map(move |seg| -placement.world(seg.start()).y)
        })
        .fold(0.0, f64::max);
    m.overlap(&pose)
        .max(m.parts_overlap(&pose, key, rails))
        .max(below)
}

/// The attitude nearest `level` at which the drawn parts overlap by no more
/// than the row margin, read on `m`, a degree at a time, nose-up first; or
/// the attitude of least overlap. With how deep they overlap there.
fn attitude_of_least_overlap(mounted: &Mounted, m: &Mechanism, level: &[f64]) -> (Vec<f64>, f64) {
    let (lo, hi) = m.coords()[mounted.angle.0].limits;
    let (key, rails) = key_and_rails(mounted);
    let at = |angle: f64| {
        let mut q = level.to_vec();
        q[mounted.angle.0] = angle;
        q
    };
    let overlap = |angle: f64| overlap_at(m, &key, &rails, &at(angle));
    let mut least = (0.0, overlap(0.0));
    let steps = (hi.min(-lo) / NOMINAL_STEP) as i32;
    'scan: for k in 1..=steps {
        for angle in [f64::from(k) * NOMINAL_STEP, -f64::from(k) * NOMINAL_STEP] {
            if least.1 <= ROW_MARGIN {
                break 'scan;
            }
            let o = overlap(angle);
            if o < least.1 {
                least = (angle, o);
            }
        }
    }
    (at(least.0), least.1)
}

/// The key lowered onto its frame from above: of the attitudes the scan
/// visits, level first and then a degree at a time, nose-up before
/// nose-down, the one at which the key clears its frame standing least far
/// off its punching, and that lift. At each attitude the lift that clears,
/// nothing overlapping by more than `within`, the key's parts read against
/// the frame's `parts.1` as well as through the pairs, is found by halving
/// the slide's range until what is left of it is under the row margin, so
/// the key starts within a margin of where it first touches; the first
/// attitude of the lowest lift is kept.
///
/// Where no attitude a degree apart clears below the slide's upper limit,
/// where the key leaves its balance pin, the attitude is narrowed instead:
/// within a degree either side of the one at which the key, at that limit,
/// overlaps its frame least, by [`ATTITUDE_REFINEMENTS`] golden-section
/// steps towards the attitude of least overlap there. A key riding on two
/// stops clears them only near the attitude at which it rides, and each
/// whole degree either side of it costs the angle between them times the
/// stops' arms of lift, up to a millimetre or two: a key riding that close
/// under its pin's top clears its frame below it at no whole degree. `None`
/// where the key clears its frame below that limit at no attitude, whole or
/// narrowed: it clears it only off its pin.
///
/// The key lowered least far stands nearest the stops it will rest on, so a
/// key whose stops fight starts near where it rides, and the static solve
/// has only a margin or two to bring it down onto them.
fn lowered(
    mounted: &Mounted,
    lift: &Lift,
    level: &[f64],
    parts: (&[PartId], &[PartId]),
    within: f64,
) -> Option<Vec<f64>> {
    let m = &mounted.mechanism;
    let (key, frame) = parts;
    let (lo, hi) = m.coords()[mounted.angle.0].limits;
    let (_, top) = m.coords()[lift.coord.0].limits;
    let steps = (hi.min(-lo) / NOMINAL_STEP) as i32;
    let at = |angle: f64, h: f64| {
        let mut q = level.to_vec();
        q[mounted.angle.0] = angle;
        q[lift.coord.0] = h;
        q
    };
    let overlap = |q: &[f64]| overlap_at(m, key, frame, q);
    let clear = |q: &[f64]| overlap(q) <= within;
    // The least lift that clears at `angle`, to within the row margin, the
    // key clearing its frame at the slide's upper limit.
    let lowest = |angle: f64| {
        let (mut low, mut high) = (lift.on_punching, top);
        while high - low > ROW_MARGIN {
            let middle = 0.5 * (low + high);
            if clear(&at(angle, middle)) {
                high = middle;
            } else {
                low = middle;
            }
        }
        high
    };
    let angles = core::iter::once(0.0).chain(
        (1..=steps).flat_map(|k| [f64::from(k) * NOMINAL_STEP, -f64::from(k) * NOMINAL_STEP]),
    );
    let mut best: Option<(f64, Vec<f64>)> = None;
    // The attitude at which the key overlaps its frame least at the slide's
    // upper limit, and by how much: where the narrowing starts.
    let mut least: Option<(f64, f64)> = None;
    for angle in angles {
        let deep = overlap(&at(angle, top));
        if least.is_none_or(|(_, o)| deep < o) {
            least = Some((angle, deep));
        }
        if deep > within {
            continue;
        }
        let high = lowest(angle);
        if best.as_ref().is_none_or(|(lowest, _)| high < *lowest) {
            best = Some((high, at(angle, high)));
        }
    }
    if best.is_none() {
        let (around, _) = least?;
        let (mut a, mut b) = (
            (around - NOMINAL_STEP).max(lo),
            (around + NOMINAL_STEP).min(hi),
        );
        let mut c = b - GOLDEN * (b - a);
        let mut d = a + GOLDEN * (b - a);
        let (mut at_c, mut at_d) = (overlap(&at(c, top)), overlap(&at(d, top)));
        for _ in 0..ATTITUDE_REFINEMENTS {
            if at_c <= at_d {
                b = d;
                d = c;
                at_d = at_c;
                c = b - GOLDEN * (b - a);
                at_c = overlap(&at(c, top));
            } else {
                a = c;
                c = d;
                at_c = at_d;
                d = a + GOLDEN * (b - a);
                at_d = overlap(&at(d, top));
            }
        }
        let angle = if at_c <= at_d { c } else { d };
        if clear(&at(angle, top)) {
            let high = lowest(angle);
            best = Some((high, at(angle, high)));
        }
    }
    best.map(|(_, q)| q)
}

/// The key on its frame, turning about its balance bearing
/// ([`KeyMount::Pivot`]): the ground, which carries the frame, and the key
/// hung from it by a revolute joint whose axis is `bearing`, in the keybed's
/// frame, where the key's frame has its origin; its angle bounded by
/// [`KEY_ANGLE_LIMIT`] either way, and its bushings' dry friction, a torque
/// bound `friction` in newton-metres, acting on it.
///
/// `parts` are the frame's on the ground and the key's on body 1, as
/// [`super::FrameDesign::parts`] and [`super::KeyDesign::parts`] build them;
/// `pairs` the contacts between them ([`key_on_frame`]). The key bears at its
/// balance pin, on the underside, which is where every key the bench builds
/// bears.
pub fn pivot(
    mass: Option<MassProps>,
    bearing: Point,
    friction: f64,
    parts: Vec<Part>,
    pairs: Vec<ContactPair>,
) -> Result<Mounted, Problem> {
    let key = Body {
        parent: BodyId::GROUND,
        joint: Joint::Revolute {
            pivot: bearing,
            coord: CoordId(0),
        },
        mass,
    };
    let angle = Coord::ANGLE
        .limited(-KEY_ANGLE_LIMIT, KEY_ANGLE_LIMIT)
        .with_friction(friction);
    Ok(Mounted {
        mechanism: Mechanism::new(vec![Body::GROUND, key], vec![angle])?
            .with_contacts(parts, pairs)?,
        key: BodyId(1),
        angle: CoordId(0),
        lift: None,
    })
}

/// The key on its frame, free to lift off its balance punching
/// ([`KeyMount::OnPunching`]): the ground, which carries the frame; a
/// massless slide, body 1, standing on the keybed at `axis_x` along it and
/// travelling up the balance pin's axis, its coordinate the height of the
/// key frame's origin above the keybed, from the keybed to `pin_top`, with
/// no friction; and the key, body 2, turning about the slide's origin, its
/// angle bounded by [`KEY_ANGLE_LIMIT`] either way, its bushings' dry
/// friction, a torque bound `friction` in newton-metres, acting on it.
///
/// `parts` are the frame's on the ground and the key's on body 2; `pairs`
/// the key's contacts with its frame but the punching's ([`key_on_frame`]),
/// and `punching` the balance punching's ([`balance_punching`]), declared
/// last. `bearing` is the punching's drawn top: the key stands on its
/// punching with the slide there. `pin_top` is where the key leaves its
/// balance pin: the slide's height at which the key's balance hole, on the
/// pin's axis at the key frame's origin, clears the pin's drawn top
/// ([`on_its_frame`] reads it off the drawn pin). Past it the key has left
/// its pin and its frame: an increment that leans on it has no equilibrium
/// ([`Mounted::lifts_off_its_frame`]). It may stand below `bearing`, a
/// punching taller than its pin: no pose then has the key on its pin.
///
/// The slide is frictionless, a simplification: the bushings' friction
/// against the key sliding along its pin acts only while the key leaves its
/// punching, a state the procedures report and never weigh.
#[allow(clippy::too_many_arguments)]
pub fn on_punching(
    mass: Option<MassProps>,
    axis_x: f64,
    bearing: f64,
    pin_top: f64,
    friction: f64,
    parts: Vec<Part>,
    pairs: Vec<ContactPair>,
    punching: ContactPair,
) -> Result<Mounted, Problem> {
    let slide = Body {
        parent: BodyId::GROUND,
        joint: Joint::Prismatic {
            origin: Point::new(axis_x, 0.0),
            axis: Dir::Y,
            coord: CoordId(0),
        },
        mass: None,
    };
    let key = Body {
        parent: BodyId(1),
        joint: Joint::Revolute {
            pivot: Point::new(0.0, 0.0),
            coord: CoordId(1),
        },
        mass,
    };
    let bodies = vec![Body::GROUND, slide, key];
    let coords = vec![
        Coord::LENGTH.limited(0.0, pin_top),
        Coord::ANGLE
            .limited(-KEY_ANGLE_LIMIT, KEY_ANGLE_LIMIT)
            .with_friction(friction),
    ];
    let others = pairs.clone();
    let mut all = pairs;
    all.push(punching);
    let pair = u16::try_from(all.len() - 1).expect("fewer than 65 536 pairs");
    let mechanism =
        Mechanism::new(bodies.clone(), coords.clone())?.with_contacts(parts.clone(), all)?;
    // The punching's centre, in the ground's frame, where its pair reads it.
    let centre = mechanism.features[usize::from(pair)].b.centre.1;
    let on_the_axis =
        centre.x.to_bits() == axis_x.to_bits() && centre.y.to_bits() == bearing.to_bits();
    let turning = if on_the_axis {
        Some(Mechanism::new(bodies, coords)?.with_contacts(parts, others)?)
    } else {
        None
    };
    Ok(Mounted {
        mechanism,
        key: BodyId(2),
        angle: CoordId(1),
        lift: Some(Lift {
            coord: CoordId(0),
            on_punching: bearing,
            pair,
            turning,
        }),
    })
}

/// A key on its frame as the procedures read it: the key in its mount, its
/// contacts with its frame, what the two sets of parts declare, and the
/// points of the key the procedures load and read.
#[derive(Debug, Clone, PartialEq)]
pub struct OnItsFrame {
    /// How the key stands on its frame.
    pub mount: KeyMount,
    /// The mechanism: the frame's parts on the ground, the key's on its body.
    pub mounted: Mounted,
    /// What the key's parts declare.
    pub key: KeyRanges,
    /// What the frame's parts declare.
    pub frame: FrameRanges,
    /// The capstan's contact point, in the key's frame: where a load
    /// standing for the action acts.
    pub capstan: Point,
    /// The key front's top, in the key's frame: where key height and dip are
    /// read.
    pub front_top: Point,
}

/// `key` on `frame` in `mount`, its key standing on the row of pins of
/// `kind`, the papers its punchings hold drawn on them; the key's mass
/// `mass`, `None` for a key nobody has weighed; its bushings' friction a
/// torque bound of `friction` newton-metres.
///
/// The one place a key is put on its frame for a procedure to read, so that
/// the bench's readings and the regulation solve that lays its papers read
/// the same mechanism, part for part and bit for bit. The frame's parts come
/// first, on the ground, then the key's, on its mount's body. The key turns
/// about its own balance pin, on its row: the natural's at the origin of the
/// keybed's frame, the sharp's the row's offset behind it, where the frame
/// draws that pin and its punching.
pub fn on_its_frame(
    mount: KeyMount,
    key: &KeyDesign,
    frame: &FrameDesign,
    kind: KeyKind,
    mass: Option<MassProps>,
    friction: f64,
) -> Result<OnItsFrame, Problem> {
    let mut parts = Vec::new();
    let frame_ranges = frame.parts(key, kind, &mut parts);
    let key_ranges = key.parts(mount.key_body(), &mut parts);
    let capstan = key_ranges.capstan_contact.point(&parts);
    let front_top = key_ranges.key_front_top.point(&parts);
    let pairs = key_on_frame(&key_ranges, &frame_ranges);
    let bearing = frame.balance_bearing_height();
    let pin = frame.sections.balance.pin_x(0.0, kind);
    let mounted = match mount {
        KeyMount::Pivot => pivot(mass, Point::new(pin, bearing), friction, parts, pairs)?,
        KeyMount::OnPunching => {
            let punching = balance_punching(&key_ranges, &frame_ranges);
            // Where the key leaves its balance pin: the pin's drawn top,
            // which the key's balance hole, at the key frame's origin on the
            // pin's axis, clears with the slide there. A rail with no pin
            // guides nothing: a key lifted off its punching has left its
            // frame.
            let pin_top = frame_ranges
                .balance_rail_pin
                .map_or(bearing, |id| drawn_top(&parts[id.index()]));
            on_punching(
                mass, pin, bearing, pin_top, friction, parts, pairs, punching,
            )?
        }
    };
    Ok(OnItsFrame {
        mount,
        mounted,
        key: key_ranges,
        frame: frame_ranges,
        capstan,
        front_top,
    })
}

/// The height of a part's drawn top, in its body's frame: the highest point
/// of its silhouette, its arcs read where they turn over.
fn drawn_top(part: &Part) -> f64 {
    part.silhouette()
        .iter()
        .map(|seg| {
            let ends = seg.start().y.max(seg.end().y);
            match *seg {
                crate::geometry::Seg::Arc { c, r, from, to }
                    if crate::geometry::signed::in_sector(Point::new(0.0, 1.0), from, to) =>
                {
                    ends.max(c.y + r)
                }
                _ => ends,
            }
        })
        .fold(f64::NEG_INFINITY, f64::max)
}

/// A rigid contact between two surfaces.
fn pair(code: ContactCode, a: Surface, b: Surface, geometry: PairGeometry) -> ContactPair {
    ContactPair {
        code,
        a,
        b,
        geometry,
        law: ContactLaw::Rigid,
    }
}

/// The key's contacts with its frame but its balance punching, each a pair
/// of the ranges the two sets of parts declare, the key on side A.
///
/// - **The front rail punching**: the felt's drawn top, read at its centre,
///   against the underside's flat ([`PairGeometry::CentreBearing`]): a key
///   landing level loads the whole washer, and the pair says the load is at
///   its middle.
/// - **The back rail cloth**: the cloth's drawn top, its face and its two
///   rounded edges, and the sides they lead into, against the whole
///   underside, chamfer, kink and flat ([`PairGeometry::Features`]): flat on
///   the cloth, on its edge, or on the key's own corner.
/// - **The front rail pin in its mortise**: the pin's top corners against the
///   top of the mortise, a cut presenting its inside.
/// - **The front rail pin on the wood**: the pin's top and its corners
///   against the wood round the mortise: the underside beside it, which a
///   pin standing outside its mortise but under the key reaches; the corners
///   of the wood at the slot's mouth, which one straddling the slot's edge
///   meets; and the slot's walls along the key, which one standing off the
///   middle of its slot meets as the key turns, the slot moving along the
///   pin by its height above the bearing times the turn.
///
/// The two pin pairs are declared only where the front rail has a pin. Which
/// of the front pairs stops the key is the static solve's to find
/// ([`crate::measure::seat`]).
pub fn key_on_frame(key: &KeyRanges, frame: &FrameRanges) -> Vec<ContactPair> {
    let mut pairs = vec![
        pair(
            ContactCode::FrontRailPunching,
            Surface::of(&key.underside_flat),
            Surface::one(frame.front_bearing.clone()),
            PairGeometry::CentreBearing,
        ),
        pair(
            ContactCode::BackRailCloth,
            Surface::of(&key.underside),
            Surface::one(frame.cloth_and_sides.clone()),
            PairGeometry::Features,
        ),
    ];
    if let Some(pin) = &frame.front_pin_top {
        pairs.push(pair(
            ContactCode::FrontPinInTheMortise,
            Surface::one(key.front_mortise_top.clone()),
            Surface::one(pin.clone()),
            PairGeometry::Features,
        ));
        if !key.beside_the_mortise.is_empty() {
            pairs.push(pair(
                ContactCode::FrontPinOnTheWood,
                Surface::of(&key.beside_the_mortise),
                Surface::one(pin.clone()),
                PairGeometry::Features,
            ));
        }
    }
    pairs
}

/// The key's balance punching: the felt's drawn top, read at its centre, on
/// the balance pin's axis, against the underside's flat
/// ([`PairGeometry::CentreBearing`]), the key on side A.
///
/// A felt washer under a key spreads its load, and the pair says the load is
/// at the washer's middle, where the key's frame has its origin: a rigid
/// stand-in, whose drawn edge overlaps a key tilted on it by what a felt
/// would compress, half the washer's width times the tilt
/// ([`crate::parts::verdicts::edge_overlaps`]). The key bears on the
/// punching beside its balance mortise, out of the sagittal plane, so the
/// pair reads the whole flat.
pub fn balance_punching(key: &KeyRanges, frame: &FrameRanges) -> ContactPair {
    pair(
        ContactCode::BalanceRailPunching,
        Surface::of(&key.underside_flat),
        Surface::one(frame.balance_bearing.clone()),
        PairGeometry::CentreBearing,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contact::oracle::{self, SPACING};
    use crate::contact::{Feature, HowItBears, Piece, Rows, Surface};
    use crate::mechanism::Pose;
    use crate::parts::fixtures::{on_frame, Setup};
    use crate::parts::frame::{KeyKind, CLOTH_EDGE_RADIUS};
    use crate::solve::Statics;

    /// The default key regulated on the default frame, as the bench builds
    /// it.
    fn regulated() -> Setup {
        crate::parts::fixtures::regulated()
    }

    /// The regulated bench, its cloth holding the tail 5 mm above the balance
    /// bearing while its front punching stands level with it: a frame whose
    /// stops fight, as the golden sheet's fixture builds it.
    fn wedged() -> Setup {
        let mut a = regulated();
        a.frame.back_rail_height =
            a.frame.balance_bearing_height() + 0.005 - a.frame.back_rail_cloth;
        a
    }

    /// The regulated bench, and the ways the golden sheet's fixtures and the
    /// bearing cases push it: a pin bottoming in its mortise, a key
    /// shortened under its frame, a cloth hung off the end of the key, a
    /// cloth reaching in front of the chamfer.
    fn assemblies() -> Vec<(&'static str, Setup)> {
        let base = regulated();
        let mut proud = base.clone();
        proud.frame.sections.front.pin_height += 0.001;
        let mut short = base.clone();
        short.key.front_edge -= 0.018;
        let mut overhung = base.clone();
        overhung.frame.back_rail = -0.232;
        let mut forward = base.clone();
        forward.frame.back_rail = -0.170;
        vec![
            ("the regulated bench", base),
            ("a pin bottoming in its mortise", proud),
            ("a key shortened under its frame", short),
            ("a cloth hung off the end of the key", overhung),
            ("a cloth in front of the chamfer", forward),
        ]
    }

    /// The assembly's parts as the bench draws them, the contacts between
    /// them, and the key on them, free to lift off its punching.
    fn mounted(a: &Setup) -> (Mounted, KeyRanges, FrameRanges) {
        let k = on_its_frame(
            KeyMount::OnPunching,
            &a.key,
            &a.frame,
            KeyKind::Natural,
            None,
            0.0,
        )
        .expect("the key's contacts are drawn");
        (k.mounted, k.key, k.frame)
    }

    /// The key on its punching at the angle `theta`, radians nose-up.
    fn on_it(m: &Mounted, theta: f64) -> Vec<f64> {
        let mut q = m.level();
        q[m.angle.0] = theta;
        q
    }

    /// Every row of the mounted key at `q`.
    fn rows_at(m: &Mounted, q: &[f64]) -> (Pose, Rows) {
        let pose = m.mechanism.pose(q);
        let mut rows = Rows::new();
        m.mechanism.rows(&pose, f64::INFINITY, &mut rows);
        (pose, rows)
    }

    fn pair_of(m: &Mounted, code: ContactCode) -> u16 {
        let i = m
            .mechanism
            .pairs()
            .iter()
            .position(|p| p.code == code)
            .expect("the key declares it");
        u16::try_from(i).unwrap()
    }

    /// The key's coordinates at its top seat and at its bottom one, by the
    /// seated pose; `None` where the key cannot be seated.
    fn seated(a: &Setup) -> Option<(Vec<f64>, Vec<f64>)> {
        let k = on_frame(a, KeyKind::Natural, 0.0, true);
        let seats = k.seating().seats(&mut Statics::new()).ok()?;
        Some((seats.top.q, seats.bottom.q))
    }

    /// The key's coordinates at its top seat.
    fn rest(a: &Setup) -> Vec<f64> {
        let k = on_frame(a, KeyKind::Natural, 0.0, true);
        k.seating()
            .top(&mut Statics::new())
            .expect("the key rests on its cloth")
            .q
    }

    /// The edges of the back rail cloth's face, metres along the key: the
    /// back one and the front one, where the drawn arcs are centred.
    fn cloth_edges(a: &Setup) -> (f64, f64) {
        let section = a.frame.sections.back;
        let centre = section.centre_x(a.frame.back_rail);
        (centre - section.half_length, centre + section.half_length)
    }

    /// Consistency: **every contact surface of the key on its frame is a
    /// range its parts declare**: the punchings' drawn tops against the
    /// underside's flat, the cloth's drawn top and sides against the whole
    /// underside, the pin's drawn top against the mortise's and against the
    /// wood round it, the balance punching last. Nothing else is there to be
    /// read: a pair holds ranges, and the mechanism has checked each is
    /// inside its part's silhouette.
    #[test]
    fn every_contact_reads_a_range_its_parts_declare() {
        for (name, a) in assemblies() {
            let (m, key, frame) = mounted(&a);
            let pin = frame
                .front_pin_top
                .clone()
                .expect("the front rail has a pin");
            let expected = [
                (
                    ContactCode::FrontRailPunching,
                    Surface::of(&key.underside_flat),
                    Surface::one(frame.front_bearing.clone()),
                ),
                (
                    ContactCode::BackRailCloth,
                    Surface::of(&key.underside),
                    Surface::one(frame.cloth_and_sides.clone()),
                ),
                (
                    ContactCode::FrontPinInTheMortise,
                    Surface::one(key.front_mortise_top.clone()),
                    Surface::one(pin.clone()),
                ),
                (
                    ContactCode::FrontPinOnTheWood,
                    Surface::of(&key.beside_the_mortise),
                    Surface::one(pin),
                ),
                (
                    ContactCode::BalanceRailPunching,
                    Surface::of(&key.underside_flat),
                    Surface::one(frame.balance_bearing.clone()),
                ),
            ];
            let pairs = m.mechanism.pairs();
            assert_eq!(pairs.len(), expected.len(), "{name}");
            for (pair, (code, a_side, b_side)) in pairs.iter().zip(expected) {
                assert_eq!(pair.code, code, "{name}");
                assert_eq!(pair.a, a_side, "{name}: {code:?}");
                assert_eq!(pair.b, b_side, "{name}: {code:?}");
                for side in [&pair.a, &pair.b] {
                    for range in &side.ranges {
                        let drawn = m.mechanism.parts()[range.part.index()].silhouette().len();
                        assert!(usize::from(range.segs.end) <= drawn, "{name}: {code:?}");
                    }
                }
            }
            let lift = m.lift.as_ref().expect("the key is free to lift");
            assert_eq!(usize::from(lift.pair), pairs.len() - 1);
            assert!(
                lift.turning.is_some(),
                "{name}: the punching is on the axis"
            );
        }
    }

    /// Consistency, `CLAUDE.md` rule 1's check by the brute-force oracle:
    /// **every contact lies on its drawn outline.** Wherever the key stands,
    /// seated at the top and at the bottom, riding on a frame whose stops
    /// fight, lifted off its punching or turned across its stroke, every row
    /// of every pair has its two surface points on the two drawn outlines,
    /// within half a spacing of a sample of each, the farthest a point of a
    /// curve can stand from the nearest sample; and a centre bearing's row
    /// stands at the middle of the drawn face, within the same. The oracle
    /// shares no code with the rows: it samples what is drawn.
    #[test]
    fn every_contact_lies_on_its_drawn_outline() {
        let mut cases: Vec<(String, Setup, Vec<f64>)> = Vec::new();
        let mut benches = assemblies();
        benches.push(("a frame whose stops fight", wedged()));
        for (name, a) in benches {
            let (m, ..) = mounted(&a);
            if let Some((top, bottom)) = seated(&a) {
                let middle: Vec<f64> = top
                    .iter()
                    .zip(&bottom)
                    .map(|(t, b)| 0.5 * (t + b))
                    .collect();
                cases.push((format!("{name}, at rest"), a.clone(), top.clone()));
                cases.push((format!("{name}, at the bottom"), a.clone(), bottom));
                cases.push((format!("{name}, mid-stroke"), a.clone(), middle));
                let mut lifted = top;
                lifted[0] += 0.0004;
                cases.push((format!("{name}, lifted"), a.clone(), lifted));
            }
            for theta in [-0.05, 0.0, 0.05] {
                cases.push((
                    format!("{name}, at {theta} rad"),
                    a.clone(),
                    on_it(&m, theta),
                ));
            }
        }
        let on = 0.5 * SPACING + 1e-12;
        let mut read = 0;
        for (name, a, q) in cases {
            let (m, ..) = mounted(&a);
            let (pose, rows) = rows_at(&m, &q);
            for (p, pair) in m.mechanism.pairs().iter().enumerate() {
                let o = oracle::Pair::read(&m.mechanism, &pose, pair);
                for (_, row) in rows.of_pair(u16::try_from(p).unwrap()) {
                    let (on_a, on_b) = (row.on_a(), row.on_b());
                    assert!(
                        oracle::nearest(on_a, &o.a) <= on && oracle::nearest(on_b, &o.b) <= on,
                        "{name}, {:?}: a row's points are off the drawn outlines: {row:?}",
                        pair.code
                    );
                    if let Some(centre) = o.centre {
                        let (dx, dy) = (on_b.x - centre.x, on_b.y - centre.y);
                        assert!(
                            (dx * dx + dy * dy).sqrt() <= on,
                            "{name}, {:?}: the row bears at {on_b:?}, the face's middle is \
                             {centre:?}",
                            pair.code
                        );
                    }
                    read += 1;
                }
            }
        }
        assert!(read > 100, "only {read} rows read");
    }

    /// Consistency: **the rows against the brute-force gap oracle**, the
    /// drawn outlines sampled every 0.05 mm, sharing no code with the rows.
    ///
    /// At rest, in the middle of the stroke and at the bottom of every case,
    /// where the seated pose puts the key; where a frame whose stops fight
    /// seats the key, riding on both; and with a key shortened under its
    /// frame held where the regulated key rests, its front pin standing in
    /// its wood, overlaps the static solve never leaves:
    ///
    /// - every row's two surface points stand on their drawn outlines, within
    ///   half a spacing of a sample, the farthest a point of the curve can be
    ///   from the nearest;
    /// - a pair whose rows stand apart reads the oracle's distance within
    ///   one spacing, the most the closest samples can be from the closest
    ///   points, and the oracle sees no overlap where it stands clear by more,
    ///   none deeper than the half spacing a touching surface reads at, where
    ///   another pair of the same parts touches;
    /// - a row that overlaps by more than a spacing, the least the samples
    ///   can tell, has a surface point inside the other side's drawn
    ///   material, and is at least as deep as that point stands from the
    ///   other surface, within a spacing: a row reads the distance to its own
    ///   face, which no nearer boundary of that material exceeds;
    /// - of the overlapping rows of one corner or edge, the shallowest is as
    ///   deep as the oracle reads, within a spacing, wherever it overlaps by
    ///   no more than the row margin, the depth to which a static solve ever
    ///   lets an overlap go. Deeper, inside the wood round a slot, where the
    ///   nearer face may lie outside the corner's sector, it is only at
    ///   least as deep.
    #[test]
    fn the_rows_agree_with_the_brute_force_oracle() {
        let mut cases: Vec<(String, Setup, Vec<f64>)> = Vec::new();
        let regulated_seats = seated(&regulated()).expect("the regulated key seats");
        for (name, a) in assemblies() {
            let (top, bottom) = seated(&a).unwrap_or_else(|| regulated_seats.clone());
            let middle: Vec<f64> = top
                .iter()
                .zip(&bottom)
                .map(|(t, b)| 0.5 * (t + b))
                .collect();
            for (at, q) in [
                ("at rest", top),
                ("mid-stroke", middle),
                ("at the bottom", bottom),
            ] {
                cases.push((format!("{name}, {at}"), a.clone(), q));
            }
        }
        let (riding, _) = seated(&wedged()).expect("the key rides on its stops");
        cases.push(("a frame whose stops fight".to_owned(), wedged(), riding));
        let mut short = regulated();
        short.key.front_edge -= 0.018;
        cases.push((
            "a key shortened under its frame, where the regulated key rests".to_owned(),
            short,
            regulated_seats.0.clone(),
        ));

        let on = 0.5 * SPACING + 1e-12;
        let (mut apart, mut deep) = (0, 0);
        for (name, a, q) in cases {
            let (m, ..) = mounted(&a);
            let (pose, rows) = rows_at(&m, &q);
            for (p, pair) in m.mechanism.pairs().iter().enumerate() {
                let p = u16::try_from(p).unwrap();
                let o = oracle::Pair::read(&m.mechanism, &pose, pair);
                // Each overlapping corner or edge: its shallowest row, and how
                // deep the oracle reads its point.
                let mut shallowest: Vec<(Feature, f64, f64)> = Vec::new();
                for (_, row) in rows.of_pair(p) {
                    let (on_a, on_b) = (row.on_a(), row.on_b());
                    assert!(
                        oracle::nearest(on_a, &o.a) <= on && oracle::nearest(on_b, &o.b) <= on,
                        "{name}, {:?}: a row's points are off the drawn outlines: {row:?}",
                        pair.code
                    );
                    if row.gap < -SPACING {
                        let depth = if o.in_a.holds(on_b) {
                            oracle::nearest(on_b, &o.a)
                        } else if o.in_b.holds(on_a) {
                            oracle::nearest(on_a, &o.b)
                        } else {
                            panic!(
                                "{name}, {:?}: the row overlaps by {} m and neither point is \
                                 inside the other side's drawn material",
                                pair.code, row.gap
                            );
                        };
                        assert!(
                            -row.gap >= depth - SPACING,
                            "{name}, {:?}: the row is {} m deep, its point {depth} m inside",
                            pair.code,
                            -row.gap
                        );
                        // The corner or edge is the row's disc; on a face
                        // against a face's centre, the centre.
                        let f = row.feature;
                        let key = match (f.a.is_disc(), f.b.is_disc()) {
                            (true, false) => Feature { a: f.a, b: f.a },
                            (false, _) => Feature { a: f.b, b: f.b },
                            (true, true) => f,
                        };
                        match shallowest.iter_mut().find(|(k, ..)| *k == key) {
                            Some(entry) if -row.gap < entry.1 => {
                                entry.1 = -row.gap;
                                entry.2 = depth;
                            }
                            Some(_) => {}
                            None => shallowest.push((key, -row.gap, depth)),
                        }
                        deep += 1;
                    }
                }
                for (key, row_depth, depth) in shallowest {
                    assert!(
                        (row_depth - depth).abs() <= SPACING
                            || row_depth > crate::contact::ROW_MARGIN,
                        "{name}, {:?}: the shallowest row of {key:?} is {row_depth} m deep, the \
                         oracle {depth} m",
                        pair.code
                    );
                }
                if let Some(gap) = rows.gap(p).filter(|&g| g >= 0.0) {
                    let distance = o.apart();
                    assert!(
                        (gap - distance).abs() <= SPACING,
                        "{name}, {:?}: the rows read {gap} m, the oracle {distance} m",
                        pair.code
                    );
                    let overlap = o.overlap();
                    assert!(
                        gap <= SPACING || overlap <= 0.5 * SPACING,
                        "{name}, {:?}: {gap} m apart, and the oracle sees an overlap {overlap} m \
                         deep",
                        pair.code
                    );
                    apart += 1;
                }
            }
        }
        assert!(apart > 30, "only {apart} pairs stood apart");
        assert!(deep > 0, "no case overlapped");
    }

    /// The chamfer under the tail is there so the key beds flat on its
    /// cloth: at the one rail height that stands the key at the chamfer's own
    /// angle, both edges of the cloth close on the chamfer and the rows call
    /// it flat; half a millimetre lower the tail digs in at the cloth's back
    /// edge, half a millimetre higher it rides on the front edge.
    #[test]
    fn bedding_flat_on_the_cloth_is_both_edges_closing_on_the_chamfer() {
        let mut a = regulated();
        a.frame.back_rail = -0.200;
        let chamfer = a.key.tail_rise / a.key.tail_rise_run;
        let flat_attitude = -chamfer / (1.0 + chamfer * chamfer).sqrt();
        let (flat_from, _) = a.key.underside_flat();
        let on_the_chamfer = Point::new(
            a.frame.back_rail,
            a.key.tail_rise * (flat_from - a.frame.back_rail) / a.key.tail_rise_run,
        );
        let c = (1.0 - flat_attitude * flat_attitude).sqrt();
        let bedded_height = a.frame.balance_bearing_height() - flat_attitude * on_the_chamfer.x
            + c * on_the_chamfer.y;
        a.frame.back_rail_height = bedded_height - a.frame.back_rail_cloth;

        let read = |a: &Setup| {
            let (m, ..) = mounted(a);
            let (pose, rows) = rows_at(&m, &rest(a));
            let back = pair_of(&m, ContactCode::BackRailCloth);
            (
                m.mechanism.bearing(&pose, &rows, back).expect("a bearing"),
                cloth_edges(a),
            )
        };
        let (flat, _) = read(&a);
        assert_eq!(flat.how, HowItBears::Flat, "{flat:?}");
        assert!(flat.slope.unwrap().abs() < 1e-9, "{flat:?}");

        let mut low = a.clone();
        low.frame.back_rail_height -= 0.0005;
        let (digging, (back_edge, _)) = read(&low);
        assert_eq!(digging.how, HowItBears::StopEdge);
        assert!(digging.slope.unwrap() > 0.0, "the rear end is the low one");
        assert!(
            (digging.point.x - back_edge).abs() < 1e-3,
            "it digs in at the back edge"
        );

        let mut high = a.clone();
        high.frame.back_rail_height += 0.0005;
        let (riding, (_, front_edge)) = read(&high);
        assert_eq!(riding.how, HowItBears::StopEdge);
        assert!(riding.slope.unwrap() < 0.0);
        assert!(
            (riding.point.x - front_edge).abs() < 1e-3,
            "it rides on the front edge"
        );
    }

    /// A cloth reaching past the end of the key is carried by the key's own
    /// corner, the tail's: the governing row is that vertex on the cloth,
    /// and the slope is the chamfer's, the face of the corner that runs on
    /// over the cloth.
    #[test]
    fn a_cloth_past_the_end_of_the_key_bears_on_the_key_corner() {
        let mut a = regulated();
        a.frame.back_rail = -0.232;
        let q = rest(&a);
        let (m, key, _) = mounted(&a);
        let theta = q[m.angle.0];
        let (pose, rows) = rows_at(&m, &q);
        let back = pair_of(&m, ContactCode::BackRailCloth);
        let bearing = m.mechanism.bearing(&pose, &rows, back).expect("a bearing");
        assert_eq!(bearing.how, HowItBears::KeyCorner);
        let governing = rows.rows()[rows.governing(back).unwrap()];
        assert_eq!(
            governing.feature.a,
            Piece::Vertex {
                part: key.keystick,
                seg: 0
            },
            "the tail's corner, where the chamfer starts"
        );
        let chamfer = a.key.tail_rise / a.key.tail_rise_run;
        let expected = (chamfer - libm::sin(theta).abs() * (1.0 + chamfer * chamfer).sqrt()).abs();
        assert!(bearing.slope.is_some_and(|x| x.abs() > 0.0), "{bearing:?}");
        assert!(
            (bearing.slope.unwrap().abs() - expected).abs() < 1e-3,
            "the chamfer's slope over the cloth: {bearing:?}, expected about {expected}"
        );
    }

    /// Consistency: **a punching standing past the end of the key has no
    /// row**, the span condition of a centre bearing. With the key's front a
    /// third of a millimetre behind the felt's centre, the centre projects
    /// past the underside both at rest and with the key level, and the pair
    /// reads nothing: no stop stands in front of the key. A third of a
    /// millimetre the other way, it reads one. The key's front is placed
    /// against the felt's drawn centre, its pin's axis, which no stop of the
    /// key's own reads.
    #[test]
    fn a_punching_past_the_end_of_the_key_has_no_row() {
        let at_rest = rest(&regulated());
        for (behind, reads) in [(0.0003, false), (-0.0003, true)] {
            let mut a = regulated();
            a.key.front_edge = a
                .frame
                .sections
                .front
                .pin_x(a.frame.front_rail, KeyKind::Natural)
                - behind;
            let (m, ..) = mounted(&a);
            let front = pair_of(&m, ContactCode::FrontRailPunching);
            // Nose-up as the regulated key rests: this one, its front pin
            // now under its wood, has no seat on its punching.
            for theta in [at_rest[m.angle.0], 0.0] {
                let (_, rows) = rows_at(&m, &on_it(&m, theta));
                assert_eq!(
                    rows.gap(front).is_some(),
                    reads,
                    "the key's front {behind} m behind the punching's centre, at {theta} rad"
                );
            }
        }
    }

    /// The limit `docs/FOUNDATIONS.md` §6.6 warns of: a corner sunk deeper
    /// than an edge arc's centre has a row only while a face beside the arc
    /// lies in the corner's own angle. The key's tail corner, a quarter of a
    /// millimetre in front of where the cloth's face ends, sunk past the front
    /// edge's centre with the key nose-up, faces neither the cloth's face nor
    /// its front side, and the cloth reads no row although the corner is
    /// inside the drawn cloth. The statics never put it there; a reading that
    /// sees it makes this test fail, and the warning goes with it.
    #[test]
    fn a_corner_sunk_past_an_edge_centre_can_have_no_row() {
        let r = CLOTH_EDGE_RADIUS;
        let mut a = regulated();
        let (_, face_front) = cloth_edges(&a);
        a.frame.back_rail += a.key.tail_end - 0.00025 - face_front;
        let (m, key, frame) = mounted(&a);
        let back = pair_of(&m, ContactCode::BackRailCloth);
        let parts = m.mechanism.parts();
        let face_front = frame
            .cloth_top
            .segs(parts)
            .iter()
            .find_map(|s| match s {
                crate::geometry::Seg::Line { a, .. } => Some(a.x),
                crate::geometry::Seg::Arc { .. } => None,
            })
            .expect("the cloth has a face");
        let corner = key.underside[0].segs(parts)[0].start();
        let top = a.frame.back_bearing_height();
        let mut sunk = 0;
        for k in 0..=60 {
            let theta = 0.03 + 0.0005 * f64::from(k);
            let pose = m.mechanism.pose(&on_it(&m, theta));
            let c = pose.placement(m.key).world(corner);
            let depth = top - c.y;
            let inside = c.x > face_front && c.x < face_front + r;
            if inside && depth > 1.5 * r && depth < a.frame.back_rail_cloth {
                let mut rows = Rows::new();
                m.mechanism.rows(&pose, f64::INFINITY, &mut rows);
                assert!(
                    rows.gap(back).is_none(),
                    "the tail corner {depth} m into the cloth now reads a row: update §6.6"
                );
                sunk += 1;
            }
        }
        assert!(
            sunk > 0,
            "no attitude sank the corner past the edge's centre"
        );
    }

    /// Consistency: **a front pin straddling the front edge of its slot
    /// reads the wood, at rest.** With the front rail 2 to 5 mm forward of
    /// where the bench builds it, the pin stands across the slot's front
    /// edge, and with the key at rest on its cloth, nose-up, the slot's front
    /// wall stands in the pin's top. The pin's pair on the wood has an
    /// overlapping row there, the contact the static solve and the view
    /// read: the wall's, as deep as the brute-force oracle reads the pin's
    /// top in the wood, within one spacing, the most the closest sample can
    /// be from the wall. Without the slot's walls in the wood round the
    /// mortise, nothing of the pair reads the pin at this attitude at all.
    #[test]
    fn a_pin_straddling_its_slots_edge_reads_the_wood_at_rest() {
        let at_rest = rest(&regulated());
        for forward in [0.002, 0.003, 0.004, 0.005] {
            let mut a = regulated();
            a.frame.front_rail += forward;
            let (m, _, _) = mounted(&a);
            let (pose, rows) = rows_at(&m, &at_rest);
            let p = pair_of(&m, ContactCode::FrontPinOnTheWood);
            let overlapping: Vec<f64> = rows
                .of_pair(p)
                .map(|(_, r)| r.gap)
                .filter(|&g| g < 0.0)
                .collect();
            let shallowest = overlapping
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            assert!(
                !overlapping.is_empty(),
                "{forward} m forward, the pin stands in the wood at rest with no row"
            );
            let o = oracle::Pair::read(&m.mechanism, &pose, &m.mechanism.pairs()[usize::from(p)])
                .overlap();
            assert!(
                (-shallowest - o).abs() <= SPACING,
                "{forward} m forward: the wall's row is {} m deep, the oracle reads {o} m",
                -shallowest
            );
        }
    }

    /// Consistency: **the nominal pose is level on the punching wherever the
    /// key can stand so, the nearest attitude within the row margin where it
    /// cannot, and, where the key has no attitude on its punching that
    /// clears its stops, the key lowered onto its frame from above: never a
    /// pose past a stop.**
    ///
    /// On the regulated bench nothing overlaps with the key level on its
    /// punching. A key shortened under its frame, its cloth lowered to let it
    /// stand nose-up, has its front pin in its wood with the key level and
    /// for some degrees nose-up: it starts nose-up, clear of the pin. A frame
    /// whose stops fight by 5 mm leaves no attitude on the punching within
    /// the margin, while a key turned until it has gone right through its
    /// cloth or its punching reads clear of both, standing in the rail under
    /// it: the key starts level, raised off its punching until nothing
    /// overlaps, by the margin at a time, and no lower.
    #[test]
    fn the_nominal_pose_is_level_where_it_can_be_and_never_past_a_stop() {
        let (m, _, _) = mounted(&regulated());
        assert_eq!(nominal_pose(&m), m.level());

        let mut short = regulated();
        short.key.front_edge -= 0.018;
        short.frame.back_rail_height -= 0.015;
        let (m, _, _) = mounted(&short);
        let q = nominal_pose(&m);
        assert!(q[m.angle.0] > 0.0, "nose-up, off the pin: {q:?}");
        assert_eq!(
            m.lift_at(&q).to_bits(),
            0.0_f64.to_bits(),
            "on its punching"
        );
        assert!(m.mechanism.overlap(&m.mechanism.pose(&q)) <= ROW_MARGIN);

        let (m, _, _) = mounted(&wedged());
        let q = nominal_pose(&m);
        assert_eq!(q[m.angle.0].to_bits(), 0.0_f64.to_bits(), "level: {q:?}");
        let lift = m.lift_at(&q);
        assert!(lift > ROW_MARGIN, "raised off its punching: {q:?}");
        let (key, rails) = key_and_rails(&m);
        assert!(overlap_at(&m.mechanism, &key, &rails, &q) <= ROW_MARGIN);
        let mut lower = q.clone();
        lower[0] -= ROW_MARGIN;
        assert!(
            overlap_at(&m.mechanism, &key, &rails, &lower) > ROW_MARGIN,
            "and no higher than it must"
        );
        let through = (1..=15)
            .flat_map(|k| [f64::from(k), -f64::from(k)])
            .find(|&k| {
                let pose = m.mechanism.pose(&on_it(&m, k * NOMINAL_STEP));
                m.mechanism.overlap(&pose) <= ROW_MARGIN
                    && m.mechanism.parts_overlap(&pose, &key, &rails) > ROW_MARGIN
            });
        assert!(
            through.is_some(),
            "some attitude on the punching reads clear of the stops it has gone through"
        );
    }

    /// Consistency: **the key held on its punching is the key on a pivot
    /// there, to the bit.** Its pose places every part where the pivot's
    /// does, the bits of its frame's origin and rotation included, and its
    /// rows over the angle are the pivot's rows; the slide's coordinate adds
    /// a column of its own and changes none. Across the stroke and past it.
    #[test]
    fn held_on_its_punching_the_key_is_the_key_on_a_pivot() {
        let a = regulated();
        let build = |mount| {
            on_its_frame(mount, &a.key, &a.frame, KeyKind::Natural, None, 0.0)
                .unwrap()
                .mounted
        };
        let (free, pinned) = (build(KeyMount::OnPunching), build(KeyMount::Pivot));
        for theta in [-0.1, -0.04, 0.0, 0.02, 0.0397, 0.1] {
            let (q_free, q_pinned) = (on_it(&free, theta), vec![theta]);
            let (pf, pp) = (
                free.mechanism.pose(&q_free),
                pinned.mechanism.pose(&q_pinned),
            );
            let (a, b) = (pf.placement(free.key), pp.placement(pinned.key));
            for (x, y) in [
                (a.origin.x, b.origin.x),
                (a.origin.y, b.origin.y),
                (a.cos, b.cos),
                (a.sin, b.sin),
            ] {
                assert_eq!(x.to_bits(), y.to_bits(), "{theta} rad");
            }
            let (mut rf, mut rp) = (Rows::new(), Rows::new());
            free.mechanism.rows(&pf, f64::INFINITY, &mut rf);
            pinned.mechanism.rows(&pp, f64::INFINITY, &mut rp);
            let lift = free.lift.as_ref().unwrap();
            let theirs: Vec<usize> = (0..rf.len())
                .filter(|&i| rf.rows()[i].pair != lift.pair)
                .collect();
            assert_eq!(theirs.len(), rp.len(), "{theta} rad");
            for (&i, j) in theirs.iter().zip(0..rp.len()) {
                let (x, y) = (rf.rows()[i], rp.rows()[j]);
                assert_eq!(x.gap.to_bits(), y.gap.to_bits(), "{theta} rad");
                assert_eq!(
                    rf.grad(i)[free.angle.0].to_bits(),
                    rp.grad(j)[pinned.angle.0].to_bits(),
                    "{theta} rad"
                );
            }
        }
    }
}

//! The mechanisms a key is put in to be read, and the contacts it has with
//! its frame.
//!
//! A mount is a builder, not a type: the same key, with the same mass, goes
//! into whichever mechanism a procedure needs, and the procedure reads it
//! there. Built: the jig; the key on its frame turning about its balance
//! bearing, [`KeyMount::Pivot`], with its contacts. Planned: the key free to
//! lift off its balance punching, the default once it is built.

use super::frame::FrameRanges;
use super::key::KeyRanges;
use crate::contact::{
    ContactCode, ContactLaw, ContactPair, PairGeometry, Part, PartCode, PartId, Surface, ROW_MARGIN,
};
use crate::geometry::Point;
use crate::mechanism::{Body, BodyId, Coord, CoordId, Joint, MassProps, Mechanism, Problem};

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
    /// note on Pg1). The key cannot lift off its balance punching on it: a
    /// frame whose stops fight leaves it no angle at all.
    Pivot,
}

impl KeyMount {
    /// The key on its frame in this mount: [`pivot`] for
    /// [`KeyMount::Pivot`].
    pub fn on_frame(
        self,
        mass: Option<MassProps>,
        bearing: Point,
        friction: f64,
        parts: Vec<Part>,
        pairs: Vec<ContactPair>,
    ) -> Result<Mounted, Problem> {
        match self {
            Self::Pivot => pivot(mass, bearing, friction, parts, pairs),
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
const NOMINAL_STEP: f64 = core::f64::consts::PI / 180.0;

/// The key's nominal pose on its frame, where a static solve that seats it
/// by rule starts: level, wherever its drawn parts overlap its frame there
/// by no more than the row margin; otherwise the nearest attitude to level,
/// a degree apart and nose-up before nose-down, at which they do; either
/// way an overlap a solve resolves through its rows. On a frame that leaves
/// it none, the attitude of least overlap, the nearest level among equals.
///
/// The overlap is read off the drawn outlines, not the rows: a static solve
/// resolves an overlap by its contacts' rows, which read a part sunk deep
/// into another truthfully only near the surface, or not at all. It is the
/// deepest of what the declared pairs read ([`Mechanism::overlap`]) and of
/// how deep the key stands in its front and back rails, which no pair
/// declares: a key turned until it has gone right through a stop leaves
/// the stop's parts clear, and only the rail under the stop says it is
/// there.
pub fn nominal_pose(mounted: &Mounted) -> Vec<f64> {
    let m = &mounted.mechanism;
    let n = m.coords().len();
    let (lo, hi) = m.coords()[mounted.angle.0].limits;
    let ids = |keep: &dyn Fn(&Part) -> bool| -> Vec<PartId> {
        (0..m.parts().len())
            .filter(|&i| keep(&m.parts()[i]))
            .map(|i| PartId(u16::try_from(i).expect("fewer than 65 536 parts")))
            .collect()
    };
    let key = ids(&|p| p.body == mounted.key);
    let rails = ids(&|p| {
        p.body != mounted.key && matches!(p.code, PartCode::FrontRail | PartCode::BackRail)
    });
    let at = |angle: f64| {
        let mut q = vec![0.0; n];
        q[mounted.angle.0] = angle;
        q
    };
    let overlap = |angle: f64| {
        let pose = m.pose(&at(angle));
        m.overlap(&pose).max(m.parts_overlap(&pose, &key, &rails))
    };
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
    at(least.0)
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
/// bears; a bearing moved behind the pin, as cutting the punching moves it,
/// would need the key's parts written about that point.
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
    })
}

/// The key's contacts with its frame, each a pair of the ranges the two sets
/// of parts declare, the key on side A.
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
    let pair = |code, a, b, geometry| ContactPair {
        code,
        a,
        b,
        geometry,
        law: ContactLaw::Rigid,
    };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contact::oracle::{self, SPACING};
    use crate::contact::{Feature, HowItBears, Piece, Rows, Surface};
    use crate::geometry::KeyBearing;
    use crate::measure::oracle::Travelling;
    use crate::measure::TouchweightProtocol;
    use crate::mechanism::Pose;
    use crate::parts::assembly::{nose_up_angle, KeyAssembly, Regulation, Touching};
    use crate::parts::frame::{papers_for, KeyKind};
    use crate::parts::{FrameDesign, KeyDesign};

    /// The default key regulated on the default frame, as the bench builds
    /// it: each felt kept, the rest of each stack laid in papers from the
    /// drawer, and the stack taken at what those papers make.
    fn regulated() -> KeyAssembly {
        let key = KeyDesign::default();
        let mut frame = FrameDesign::default();
        let target = Regulation {
            key_height: 0.0650,
            key_dip: 0.0102,
        };
        let (balance, front) = target.solve(&key, &frame, 0.0).unwrap();
        for (stack, total) in [
            (&mut frame.balance_punching, balance),
            (&mut frame.front_punching, front),
        ] {
            stack.paper = papers_for(total - stack.felt).iter().sum();
        }
        KeyAssembly {
            key,
            frame,
            bearing: KeyBearing {
                offset_from_pin: 0.0,
            },
            protocol: TouchweightProtocol::default(),
        }
    }

    /// The regulated bench, and the ways the golden sheet's fixtures and the
    /// bearing cases push it: a pin bottoming in its mortise, a key
    /// shortened under its frame, a cloth hung off the end of the key, a
    /// cloth reaching in front of the chamfer.
    fn assemblies() -> Vec<(&'static str, KeyAssembly)> {
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
    /// them, and the key on them, its bearing raised by `lift`.
    fn mounted(a: &KeyAssembly, lift: f64) -> (Mounted, KeyRanges, FrameRanges) {
        let mut parts = Vec::new();
        let balance = papers_for(a.frame.balance_punching.paper);
        let front = papers_for(a.frame.front_punching.paper);
        let frame = a
            .frame
            .parts(&a.key, KeyKind::Natural, &balance, &front, &mut parts);
        let key = a.key.parts(BodyId(1), &mut parts);
        let pairs = key_on_frame(&key, &frame);
        let bearing = Point::new(0.0, a.frame.balance_bearing_height() + lift);
        let m = pivot(None, bearing, 0.0, parts, pairs).expect("the key's contacts are drawn");
        (m, key, frame)
    }

    /// Every row of the mounted key at an attitude of the statics, front-down
    /// sine `s`.
    fn rows_at(m: &Mounted, s: f64) -> (Pose, Rows) {
        let pose = m.mechanism.pose(&[nose_up_angle(s)]);
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

    /// Attitudes from rest to the bottom of the dip, both included.
    fn stroke(a: &KeyAssembly) -> Vec<f64> {
        let t = a.travel().expect("a key with a travel");
        (0..=10)
            .map(|k| t.sin_rest + f64::from(k) / 10.0 * (t.sin_bottom - t.sin_rest))
            .collect()
    }

    /// Consistency: **every contact surface of the key on its frame is a
    /// range its parts declare**: the punching's drawn top against the
    /// underside's flat, the cloth's drawn top and sides against the whole
    /// underside, the pin's drawn top against the mortise's and against the
    /// wood round it. Nothing else is there to be read: a pair holds ranges,
    /// and the mechanism has checked each is inside its part's silhouette.
    #[test]
    fn every_contact_reads_a_range_its_parts_declare() {
        for (name, a) in assemblies() {
            let (m, key, frame) = mounted(&a, 0.0);
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
        }
    }

    /// Consistency: **the rows against the built `circle_vs_line` at the
    /// built stops**, over the stroke.
    ///
    /// - The back rail cloth: the stop is the drawn cloth, edge arcs and
    ///   face, so the two agree to the rounding of positions a quarter of a
    ///   metre from the pin, wherever the key stands on or above the cloth,
    ///   where the built gap keeps its sign.
    /// - The front punching: the built stop is a disc of the edge radius `r`
    ///   centred `r` under the felt's top, the new one the top's centre
    ///   itself. At a tilt `φ` the disc reaches `r (1 − cos φ)` further
    ///   towards the key than its top does, so the row reads exactly that
    ///   much more: four tenths of a micron at rest, nothing level.
    /// - The pin in its mortise: the built stop is a point at the pin's
    ///   middle, the mortise's depth taken straight up; the new one the
    ///   pin's drawn corners against the drawn mortise top, its depth taken
    ///   square to the key. Level they agree; tilted, the nearer corner is
    ///   closer by up to half the pin's length times the tilt's sine, and the
    ///   depth and the disc differ by `(depth + r)(1 − cos φ)`.
    #[test]
    fn the_rows_agree_with_the_built_stops() {
        let r = crate::parts::assembly::PUNCHING_EDGE_RADIUS;
        let rounding = 256.0 * f64::EPSILON * 0.5;
        for (name, a) in assemblies() {
            let (m, ..) = mounted(&a, 0.0);
            let (back, front, mortise) = (
                pair_of(&m, ContactCode::BackRailCloth),
                pair_of(&m, ContactCode::FrontRailPunching),
                pair_of(&m, ContactCode::FrontPinInTheMortise),
            );
            let mut attitudes = stroke(&a);
            attitudes.push(0.0);
            for s in attitudes {
                let (_, rows) = rows_at(&m, s);
                let tilt = 1.0 - (1.0 - s * s).sqrt();

                let built = a.stop_contact(a.back_stop(), s).gap;
                if built >= 0.0 {
                    let new = rows.gap(back).expect("the cloth has a row");
                    assert!(
                        (new - built).abs() <= rounding,
                        "{name}, sin {s}: the cloth's row reads {new}, the built stop {built}"
                    );
                }

                let built = a.stop_contact(a.punching_stop(), s).gap;
                let new = rows.gap(front).expect("the punching stands under the flat");
                assert!(
                    (new - built - r * tilt).abs() <= rounding,
                    "{name}, sin {s}: the punching's row reads {new}, the built stop {built}, \
                     apart by {} where the disc accounts for {}",
                    new - built,
                    r * tilt
                );

                let stop = a.mortise_stop().expect("the front rail has a pin");
                let built = a.stop_contact(stop, s).gap;
                if let Some(new) = rows.gap(mortise) {
                    let pin = a.frame.sections.front.pin.unwrap().length();
                    let bound =
                        0.5 * pin * s.abs() + (a.key.front_mortise_depth + r) * tilt + rounding;
                    assert!(
                        (new - built).abs() <= bound,
                        "{name}, sin {s}: the mortise's row reads {new}, the built stop {built}"
                    );
                    if s == 0.0 {
                        assert!((new - built).abs() <= rounding, "{name}: level, they agree");
                    }
                }
            }
        }
    }

    /// Consistency: **the rows against the brute-force gap oracle**, the
    /// drawn outlines sampled every 0.05 mm, sharing no code with the rows.
    ///
    /// At rest, in the middle of the stroke and at the bottom of every case,
    /// and where the bench draws a key whose stops fight, riding on both:
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
        let mut cases: Vec<(String, KeyAssembly, f64, f64)> = Vec::new();
        for (name, a) in assemblies() {
            let t = a.travel().expect("a travel");
            for (at, s) in [
                ("at rest", t.sin_rest),
                ("mid-stroke", 0.5 * (t.sin_rest + t.sin_bottom)),
                ("at the bottom", t.sin_bottom),
            ] {
                cases.push((format!("{name}, {at}"), a.clone(), s, 0.0));
            }
        }
        let mut wedged = regulated();
        wedged.frame.back_rail_height =
            wedged.frame.balance_bearing_height() + 0.005 - wedged.frame.back_rail_cloth;
        let (s, lift) = wedged.riding_on_both_stops().expect("the key rides");
        cases.push(("a frame whose stops fight".to_owned(), wedged, s, lift));

        let on = 0.5 * SPACING + 1e-12;
        let (mut apart, mut deep) = (0, 0);
        for (name, a, s, lift) in cases {
            let (m, ..) = mounted(&a, lift);
            let (pose, rows) = rows_at(&m, s);
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

    /// How the key bears on its cloth, by the rows and by the built
    /// `first_touch`, at the attitude the built statics rest it at: on the
    /// regulated bench, and with the back rail moved and raised and lowered
    /// through all three ways. The same way, the same slope to the rounding
    /// of the key's attitude, the same place to a picometre.
    #[test]
    fn the_rows_bear_as_the_built_stop_bears() {
        let mut cases = assemblies();
        for raise in [-0.0006, -0.0002, 0.0002, 0.0006] {
            let mut a = regulated();
            a.frame.back_rail_height += raise;
            cases.push(("a back rail raised or lowered", a));
        }
        for (name, a) in cases {
            let touch = a
                .first_touch(a.back_stop())
                .expect("the cloth is reachable");
            let (m, ..) = mounted(&a, 0.0);
            let (pose, rows) = rows_at(&m, touch.sin_theta);
            let back = pair_of(&m, ContactCode::BackRailCloth);
            let bearing = m.mechanism.bearing(&pose, &rows, back).expect("a bearing");
            let how = match touch.how {
                Touching::Flat => HowItBears::Flat,
                Touching::StopEdge => HowItBears::StopEdge,
                Touching::KeyCorner => HowItBears::KeyCorner,
            };
            assert_eq!(bearing.how, how, "{name}");
            let slope = bearing.slope.expect("the cloth has a face");
            assert!(
                (slope - touch.slope).abs() <= 64.0 * f64::EPSILON * (1.0 + touch.slope.abs()),
                "{name}: the rows read a slope of {slope}, the built stop {}",
                touch.slope
            );
            assert!(
                (bearing.point.x - touch.at).abs() <= 1e-12,
                "{name}: the rows bear at {} m, the built stop at {} m",
                bearing.point.x,
                touch.at
            );
            assert!(
                bearing.gap.abs() <= 1e-12,
                "{name}: the key stands on its cloth"
            );
        }
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

        let read = |a: &KeyAssembly| {
            let s = a.sin_angle_on_back_rail().expect("the cloth is reachable");
            let (m, ..) = mounted(a, 0.0);
            let (pose, rows) = rows_at(&m, s);
            let back = pair_of(&m, ContactCode::BackRailCloth);
            (
                m.mechanism.bearing(&pose, &rows, back).expect("a bearing"),
                a.back_stop(),
            )
        };
        let (flat, _) = read(&a);
        assert_eq!(flat.how, HowItBears::Flat, "{flat:?}");
        assert!(flat.slope.unwrap().abs() < 1e-9, "{flat:?}");

        let mut low = a.clone();
        low.frame.back_rail_height -= 0.0005;
        let (digging, stop) = read(&low);
        assert_eq!(digging.how, HowItBears::StopEdge);
        assert!(digging.slope.unwrap() > 0.0, "the rear end is the low one");
        assert!(digging.point.x < stop.at, "it digs in at the back edge");

        let mut high = a.clone();
        high.frame.back_rail_height += 0.0005;
        let (riding, stop) = read(&high);
        assert_eq!(riding.how, HowItBears::StopEdge);
        assert!(riding.slope.unwrap() < 0.0);
        assert!(riding.point.x > stop.at, "it rides on the front edge");
    }

    /// A cloth reaching past the end of the key is carried by the key's own
    /// corner, the tail's: the governing row is that vertex on the cloth,
    /// and the slope is the chamfer's, the face of the corner that runs on
    /// over the cloth.
    #[test]
    fn a_cloth_past_the_end_of_the_key_bears_on_the_key_corner() {
        let mut a = regulated();
        a.frame.back_rail = -0.232;
        let s = a.sin_angle_on_back_rail().expect("the cloth is reachable");
        let (m, key, _) = mounted(&a, 0.0);
        let (pose, rows) = rows_at(&m, s);
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
        let expected = (chamfer - s.abs() * (1.0 + chamfer * chamfer).sqrt()).abs();
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
    /// millimetre the other way, it reads one.
    #[test]
    fn a_punching_past_the_end_of_the_key_has_no_row() {
        for (behind, reads) in [(0.0003, false), (-0.0003, true)] {
            let mut a = regulated();
            a.key.front_edge = a.punching_stop().at - behind;
            let (m, ..) = mounted(&a, 0.0);
            let front = pair_of(&m, ContactCode::FrontRailPunching);
            let rest = a.sin_angle_on_back_rail().expect("the cloth is reachable");
            for s in [rest, 0.0] {
                let (_, rows) = rows_at(&m, s);
                assert_eq!(
                    rows.gap(front).is_some(),
                    reads,
                    "the key's front {behind} m behind the punching's centre, sin {s}"
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
        let r = crate::parts::assembly::PUNCHING_EDGE_RADIUS;
        let mut a = regulated();
        let section = a.frame.sections.back;
        let face_front = section.centre_x(a.frame.back_rail) + section.half_length;
        a.frame.back_rail += a.key.tail_end - 0.00025 - face_front;
        let (m, key, frame) = mounted(&a, 0.0);
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
            let pose = m.mechanism.pose(&[theta]);
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

    /// The rails of a mounted key on its frame, which no pair declares.
    fn rails(m: &Mounted) -> (Vec<PartId>, Vec<PartId>) {
        let parts = m.mechanism.parts();
        let id = |i: usize| PartId(u16::try_from(i).unwrap());
        let key = (0..parts.len())
            .filter(|&i| parts[i].body == m.key)
            .map(id)
            .collect();
        let rails = (0..parts.len())
            .filter(|&i| matches!(parts[i].code, PartCode::FrontRail | PartCode::BackRail))
            .map(id)
            .collect();
        (key, rails)
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
        let rest = regulated()
            .sin_angle_on_back_rail()
            .expect("the key rests on its cloth");
        for forward in [0.002, 0.003, 0.004, 0.005] {
            let mut a = regulated();
            a.frame.front_rail += forward;
            let (m, _, _) = mounted(&a, 0.0);
            let (pose, rows) = rows_at(&m, rest);
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

    /// Consistency: **the nominal pose is level wherever the key can stand
    /// level, the nearest attitude within the row margin where it cannot,
    /// and never one past a stop.**
    ///
    /// On the regulated bench nothing overlaps with the key level. A key
    /// shortened under its frame, its cloth lowered to let it stand nose-up,
    /// has its front pin in its wood with the key level and for some degrees
    /// nose-up: it starts nose-up, clear of the pin. A frame whose stops fight
    /// by 5 mm leaves no attitude near level within the margin, while a key
    /// turned until it has gone right through its cloth or its punching reads
    /// clear of both, standing in the rail under it: the nominal pose is the
    /// least overlap near level, not that attitude.
    #[test]
    fn the_nominal_pose_is_level_where_it_can_be_and_never_past_a_stop() {
        let (m, _, _) = mounted(&regulated(), 0.0);
        assert_eq!(nominal_pose(&m), vec![0.0]);

        let mut short = regulated();
        short.key.front_edge -= 0.018;
        short.frame.back_rail_height -= 0.015;
        let (m, _, _) = mounted(&short, 0.0);
        let q = nominal_pose(&m);
        assert!(q[0] > 0.0, "nose-up, off the pin: {q:?}");
        assert!(m.mechanism.overlap(&m.mechanism.pose(&q)) <= ROW_MARGIN);

        let mut wedged = regulated();
        wedged.frame.back_rail_height =
            wedged.frame.balance_bearing_height() + 0.005 - wedged.frame.back_rail_cloth;
        let (m, _, _) = mounted(&wedged, 0.0);
        let q = nominal_pose(&m);
        assert!(q[0].abs() <= 2.0 * NOMINAL_STEP, "{q:?}");
        let m = &m.mechanism;
        assert!(m.overlap(&m.pose(&q)) > ROW_MARGIN, "the stops fight");
        let (key, rails) = rails(&mounted(&wedged, 0.0).0);
        let through = (1..=15)
            .flat_map(|k| [f64::from(k), -f64::from(k)])
            .find(|&k| {
                let pose = m.pose(&[k * NOMINAL_STEP]);
                m.overlap(&pose) <= ROW_MARGIN && m.parts_overlap(&pose, &key, &rails) > ROW_MARGIN
            });
        assert!(
            through.is_some(),
            "some attitude reads clear of the stops it has gone through"
        );
    }
}

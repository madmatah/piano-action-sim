//! What the tests of the key on its frame share: the regulated bench, and
//! the key mounted on its frame as the bench mounts it.

use super::frame::{FrameRanges, KeyKind};
use super::mounts::{on_its_frame, KeyMount, Mounted};
use super::{FrameDesign, KeyDesign, WippenAssembly, WippenAssemblyMass};
use crate::geometry::Point;
use crate::measure::seat::{Seating, SEAT_MARGIN};
use crate::measure::TouchweightProtocol;
use crate::mechanism::MassProps;
use crate::regulation::{Regulating, TargetOutcome};
/// The fixture's targets: 65.0 mm of key height and 10.2 mm of key dip.
pub use crate::test_fixture::TARGETS;

/// A key, its frame and the protocol it is weighed by: what a test sets up
/// and then pushes one way or another.
#[derive(Debug, Clone, PartialEq)]
pub struct Setup {
    /// The key.
    pub key: KeyDesign,
    /// Its frame, the papers laid on its punchings.
    pub frame: FrameDesign,
    /// Where every weight is centered.
    pub protocol: TouchweightProtocol,
}

impl Setup {
    /// The measuring point, meters in front of the balance pin.
    pub fn measuring_point(&self) -> f64 {
        self.protocol.measuring_point(&self.key)
    }
}

/// The key's mass, as the bench weighs it: the fixture's test weighing
/// ([`crate::test_fixture::weighed`]), its static moment taken on the
/// fixture's key, the height of its center of mass `key`'s drawn section's
/// centroid; `None` for a key whose section encloses nothing.
pub fn mass_of(key: &KeyDesign) -> Option<MassProps> {
    crate::test_fixture::weighed().props(key.section_centroid()?.y)
}

/// The test key regulated on its frame, as the bench builds it:
/// each felt kept, and on it the papers the regulation solve lays from the
/// drawer for [`TARGETS`]. Solved once, the first time a test asks.
pub fn regulated() -> Setup {
    static REGULATED: std::sync::OnceLock<Setup> = std::sync::OnceLock::new();
    REGULATED.get_or_init(regulate_the_fixture).clone()
}

fn regulate_the_fixture() -> Setup {
    let key = crate::test_fixture::key();
    let mut frame = crate::test_fixture::frame();
    let protocol = crate::test_fixture::protocol();
    let outcome = Regulating {
        key: &key,
        frame: &frame,
        kind: KeyKind::Natural,
        mass: mass_of(&key),
        protocol: &protocol,
        wippen_assembly: None,
    }
    .solve(TARGETS);
    let Ok(TargetOutcome::Met { stacks, .. }) = outcome else {
        panic!("the fixture's targets are reachable on its frame: {outcome:?}");
    };
    frame.balance_punching.papers = stacks.balance.papers;
    frame.front_punching.papers = stacks.front.papers;
    Setup {
        key,
        frame,
        protocol,
    }
}

/// A key mounted on its frame, with what its parts declare and the points
/// the procedures load and read.
pub struct OnFrame {
    /// The key on its frame.
    pub mounted: Mounted,
    /// What the frame's parts declare.
    pub frame: FrameRanges,
    /// The capstan's contact point, in the key's frame.
    pub capstan: Point,
    /// The measuring point, in the key's frame.
    pub measuring_point: Point,
    /// The key front's top, in the key's frame: where height and dip are
    /// read.
    pub front_top: Point,
}

impl OnFrame {
    /// The seated-pose procedure on this key.
    pub fn seating(&self) -> Seating<'_> {
        Seating {
            mounted: &self.mounted,
            capstan: self.capstan,
            measuring_point: self.measuring_point,
            margin: SEAT_MARGIN,
            standing: &[],
        }
    }
}

/// `a`'s key on its frame, free to lift off its balance punching as the
/// bench mounts it, of `kind`, its bushings' friction bounded by `friction`
/// newton-meters, its mass the test weighing's, or none for `weighed: false`;
/// the papers its punchings hold drawn on them.
pub fn on_frame(a: &Setup, kind: KeyKind, friction: f64, weighed: bool) -> OnFrame {
    mounted_as(KeyMount::OnPunching, a, kind, friction, weighed)
}

/// The same key turning about a fixed axis through its balance bearing
/// ([`KeyMount::Pivot`]): for what the key on a pivot is tested for, the
/// static solve on one coordinate and the comparison with the key on its
/// punching.
pub fn on_pivot(a: &Setup, kind: KeyKind, friction: f64, weighed: bool) -> OnFrame {
    mounted_as(KeyMount::Pivot, a, kind, friction, weighed)
}

/// `a`'s key on its frame as [`on_frame`] mounts it, carrying `wippen` on
/// its capstan, with what is known of its mass.
pub fn on_frame_carrying(
    a: &Setup,
    friction: f64,
    weighed: bool,
    wippen: (&WippenAssembly, &WippenAssemblyMass),
) -> OnFrame {
    carrying(
        KeyMount::OnPunching,
        a,
        KeyKind::Natural,
        friction,
        weighed,
        Some(wippen),
    )
}

/// `a`'s key turning about a fixed axis through its balance bearing, as
/// [`on_pivot`] mounts it, carrying `wippen` on its capstan, with what is
/// known of its mass: a key that cannot lift, for what the wippen's
/// refusals are named on such a key.
pub fn on_pivot_carrying(
    a: &Setup,
    friction: f64,
    weighed: bool,
    wippen: (&WippenAssembly, &WippenAssemblyMass),
) -> OnFrame {
    carrying(
        KeyMount::Pivot,
        a,
        KeyKind::Natural,
        friction,
        weighed,
        Some(wippen),
    )
}

/// The test wippen assembly ([`crate::test_fixture::wippen_assembly`]) with
/// its published mass, as the bench carries it.
pub fn test_wippen() -> (WippenAssembly, WippenAssemblyMass) {
    (
        crate::test_fixture::wippen_assembly(),
        crate::test_fixture::wippen_assembly_mass(),
    )
}

/// `a`'s key on its frame, weighed, its bushings without friction, carrying
/// `wippen` with the test wippen's mass.
pub fn carrying_the(a: &Setup, wippen: &WippenAssembly) -> OnFrame {
    on_frame_carrying(
        a,
        0.0,
        true,
        (wippen, &crate::test_fixture::wippen_assembly_mass()),
    )
}

fn mounted_as(mount: KeyMount, a: &Setup, kind: KeyKind, friction: f64, weighed: bool) -> OnFrame {
    carrying(mount, a, kind, friction, weighed, None)
}

fn carrying(
    mount: KeyMount,
    a: &Setup,
    kind: KeyKind,
    friction: f64,
    weighed: bool,
    wippen: Option<(&WippenAssembly, &WippenAssemblyMass)>,
) -> OnFrame {
    let mass = if weighed { mass_of(&a.key) } else { None };
    let k = on_its_frame(mount, &a.key, &a.frame, kind, mass, friction, wippen)
        .expect("the key's contacts are drawn");
    OnFrame {
        mounted: k.mounted,
        frame: k.frame,
        capstan: k.capstan,
        measuring_point: a.protocol.measuring_point_on_key(&a.key),
        front_top: k.front_top,
    }
}

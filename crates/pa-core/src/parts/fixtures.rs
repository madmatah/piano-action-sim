//! What the tests of the key on its frame share: the regulated bench, and
//! the key mounted on its frame as the bench mounts it.

use super::frame::{papers_for, FrameRanges, KeyKind};
use super::mounts::{on_its_frame, KeyMount, Mounted};
use super::{FrameDesign, KeyDesign};
use crate::geometry::Point;
use crate::measure::seat::{Seating, SEAT_MARGIN};
use crate::measure::TouchweightProtocol;
use crate::mechanism::MassProps;
use crate::regulation::{Regulating, TargetOutcome, Targets};

/// The bench's targets: 65.0 mm of key height and 10.2 mm of key dip.
pub const TARGETS: Targets = Targets {
    key_height: 0.0650,
    key_dip: 0.0102,
};

/// A key, its frame and the protocol it is weighed by: what a test sets up
/// and then pushes one way or another.
#[derive(Debug, Clone, PartialEq)]
pub struct Setup {
    /// The key.
    pub key: KeyDesign,
    /// Its frame, the papers laid on its punchings as totals.
    pub frame: FrameDesign,
    /// Where every weight is centred.
    pub protocol: TouchweightProtocol,
}

impl Setup {
    /// The measuring point, metres in front of the balance pin.
    pub fn measuring_point(&self) -> f64 {
        self.protocol.measuring_point(&self.key)
    }
}

/// The key's mass, as the bench weighs it: the volume model's.
pub fn mass_of(key: &KeyDesign) -> Option<MassProps> {
    key.mass_properties().map(|p| MassProps {
        mass: p.mass,
        com: p.com,
    })
}

/// The default key regulated on the default frame, as the bench builds it:
/// each felt kept, and on it the papers the regulation solve lays from the
/// drawer for [`TARGETS`]. Solved once, the first time a test asks.
pub fn regulated() -> Setup {
    static REGULATED: std::sync::OnceLock<Setup> = std::sync::OnceLock::new();
    REGULATED.get_or_init(regulate_the_default).clone()
}

fn regulate_the_default() -> Setup {
    let key = KeyDesign::default();
    let mut frame = FrameDesign::default();
    let protocol = TouchweightProtocol::default();
    let outcome = Regulating {
        key: &key,
        frame: &frame,
        kind: KeyKind::Natural,
        mass: mass_of(&key),
        protocol: &protocol,
    }
    .solve(TARGETS);
    let Ok(TargetOutcome::Met { stacks, .. }) = outcome else {
        panic!("the default targets are reachable on the default frame: {outcome:?}");
    };
    frame.balance_punching.paper = stacks.balance.papers.iter().sum();
    frame.front_punching.paper = stacks.front.papers.iter().sum();
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
/// newton-metres, its mass the volume model's, or none for `weighed: false`;
/// the papers on its punchings laid from the drawer for the totals its frame
/// holds.
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

fn mounted_as(mount: KeyMount, a: &Setup, kind: KeyKind, friction: f64, weighed: bool) -> OnFrame {
    let balance = papers_for(a.frame.balance_punching.paper);
    let front = papers_for(a.frame.front_punching.paper);
    let mass = if weighed { mass_of(&a.key) } else { None };
    let k = on_its_frame(
        mount, &a.key, &a.frame, kind, &balance, &front, mass, friction,
    )
    .expect("the key's contacts are drawn");
    OnFrame {
        mounted: k.mounted,
        frame: k.frame,
        capstan: k.capstan,
        measuring_point: a.protocol.measuring_point_on_key(&a.key),
        front_top: k.front_top,
    }
}

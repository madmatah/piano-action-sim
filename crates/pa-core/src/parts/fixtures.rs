//! What the tests of the key on its frame share: the regulated bench, and
//! the key mounted on its frame as the bench mounts it.

use super::frame::{papers_for, FrameRanges, KeyKind};
use super::mounts::{on_its_frame, Mounted};
use super::{FrameDesign, KeyAssembly, KeyDesign};
use crate::geometry::{KeyBearing, Point};
use crate::measure::seat::{Seating, SEAT_MARGIN};
use crate::measure::TouchweightProtocol;
use crate::mechanism::MassProps;
use crate::regulation::{Regulating, TargetOutcome, Targets};

/// The bench's targets: 65.0 mm of key height and 10.2 mm of key dip.
pub const TARGETS: Targets = Targets {
    key_height: 0.0650,
    key_dip: 0.0102,
};

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
pub fn regulated() -> KeyAssembly {
    static REGULATED: std::sync::OnceLock<KeyAssembly> = std::sync::OnceLock::new();
    REGULATED.get_or_init(regulate_the_default).clone()
}

fn regulate_the_default() -> KeyAssembly {
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
    KeyAssembly {
        key,
        frame,
        bearing: KeyBearing {
            offset_from_pin: 0.0,
        },
        protocol,
    }
}

/// A key mounted on its frame, with what its parts declare and the points
/// the procedures load and read.
pub struct OnFrame {
    /// The key on its frame, turning about its balance bearing.
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

/// `a`'s key on its frame, of `kind`, its bushings' friction bounded by
/// `friction` newton-metres, its mass the volume model's, or none for
/// `weighed: false`; the papers on its punchings laid from the drawer for
/// the totals its frame holds.
pub fn on_frame(a: &KeyAssembly, kind: KeyKind, friction: f64, weighed: bool) -> OnFrame {
    let balance = papers_for(a.frame.balance_punching.paper);
    let front = papers_for(a.frame.front_punching.paper);
    let mass = if weighed { mass_of(&a.key) } else { None };
    let k = on_its_frame(
        &a.key, &a.frame, kind, &balance, &front, mass, friction, 0.0,
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

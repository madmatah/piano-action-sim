//! What the tests of the key on its frame share: the regulated bench, and
//! the key mounted on its frame as the bench mounts it.

use super::frame::{papers_for, FrameRanges, KeyKind};
use super::mounts::{key_on_frame, pivot, Mounted};
use super::{FrameDesign, KeyAssembly, KeyDesign, Regulation};
use crate::geometry::{KeyBearing, Point};
use crate::measure::seat::{Seating, SEAT_MARGIN};
use crate::measure::TouchweightProtocol;
use crate::mechanism::{BodyId, MassProps};

/// The default key regulated on the default frame, as the bench builds it:
/// each felt kept, the rest of each stack laid in papers from the drawer,
/// and the stack taken at what those papers make.
pub fn regulated() -> KeyAssembly {
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
        }
    }
}

/// `a`'s key on its frame, of `kind`, its bushings' friction bounded by
/// `friction` newton-metres, its mass the volume model's, or none for
/// `weighed: false`.
pub fn on_frame(a: &KeyAssembly, kind: KeyKind, friction: f64, weighed: bool) -> OnFrame {
    let mut parts = Vec::new();
    let balance = papers_for(a.frame.balance_punching.paper);
    let front = papers_for(a.frame.front_punching.paper);
    let frame = a.frame.parts(&a.key, kind, &balance, &front, &mut parts);
    let key = a.key.parts(BodyId(1), &mut parts);
    let pairs = key_on_frame(&key, &frame);
    let mass = if weighed {
        a.key.mass_properties().map(|p| MassProps {
            mass: p.mass,
            com: p.com,
        })
    } else {
        None
    };
    let (capstan, front_top) = (
        key.capstan_contact.point(&parts),
        key.key_front_top.point(&parts),
    );
    let bearing = Point::new(0.0, a.frame.balance_bearing_height());
    let mounted =
        pivot(mass, bearing, friction, parts, pairs).expect("the key's contacts are drawn");
    OnFrame {
        mounted,
        frame,
        capstan,
        measuring_point: a.protocol.measuring_point_on_key(&a.key),
        front_top,
    }
}

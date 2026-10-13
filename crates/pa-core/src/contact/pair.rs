//! Contact pairs: two surfaces of drawn parts, how they meet, and by what law.

use super::SurfaceRef;

/// What a contact is: the two parts meeting, as the trade names them.
///
/// The identifiers follow the parts' own ([`super::PartCode`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContactCode {
    /// The key's underside on the front rail punching: what the key comes
    /// down on at the bottom of its dip.
    FrontRailPunching,
    /// The key's underside on the back rail cloth: what it rests on.
    BackRailCloth,
    /// The front rail pin's top against the top of the key's front mortise:
    /// the pin bottoming in its slot.
    FrontPinInTheMortise,
    /// The front rail pin's top against the wood of the key round the front
    /// mortise: the underside beside the slot, the corners of the wood at the
    /// slot's mouth and the slot's walls along the key. A pin standing
    /// outside its mortise but under the key reaches it, as does one
    /// straddling the slot's edge, and one standing off the middle of its
    /// slot meets a wall as the key turns and the slot moves along it.
    FrontPinOnTheWood,
}

impl ContactCode {
    /// The contact's identifier, English `snake_case`.
    pub fn code(self) -> &'static str {
        match self {
            Self::FrontRailPunching => "front_rail_punching",
            Self::BackRailCloth => "back_rail_cloth",
            Self::FrontPinInTheMortise => "front_pin_in_the_mortise",
            Self::FrontPinOnTheWood => "front_pin_on_the_wood",
        }
    }
}

/// One side of a contact: ranges of drawn parts, all on one body, in order.
///
/// Usually one range. Several where the surface a contact reads runs across
/// parts or skips a stretch of one: the key's underside is the keystick's and
/// then, past the end grain, the key front's; the wood beside the front
/// mortise is the underside less the slot's mouth. Where one range ends
/// exactly where the next begins, bit for bit, the two are one chain and the
/// vertex between them is read with both its segments, so the straight joint
/// between the keystick and the key front is no corner. Anywhere else a
/// range's end is read with its own outline's next segment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Surface {
    /// The ranges, in order.
    pub ranges: Vec<SurfaceRef>,
}

impl Surface {
    /// A surface of one range.
    pub fn one(range: SurfaceRef) -> Self {
        Self {
            ranges: vec![range],
        }
    }

    /// A surface of several ranges, in order.
    pub fn of(ranges: &[SurfaceRef]) -> Self {
        Self {
            ranges: ranges.to_vec(),
        }
    }
}

/// How the two surfaces of a pair are read against each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PairGeometry {
    /// Every disc of one surface (its arcs and convex vertices) against
    /// every straight segment of the other, both ways, and every disc against
    /// every disc: the rigid contact of two drawn outlines. A concave corner,
    /// and anything of a cut but its straight segments, is never a feature.
    Features,
    /// The centre of side B's drawn face against the supporting line of side
    /// A's segments, while the centre projects into their span.
    ///
    /// The rigid idealization of a felt that spreads its load: a key landing
    /// level loads a washer over its whole face, so its resultant is at the
    /// centre, and the pair says so rather than letting the washer's edge
    /// carry it. The point is read from the drawn face, so nothing parallel
    /// to the drawing exists. Where the key stands out of level on such a
    /// pair, the drawn face's edge overlaps the key by what a felt would
    /// compress.
    CentreBearing,
}

/// The law a contact's force follows.
///
/// Every contact is rigid: a unilateral row, the gap kept from closing past
/// zero, and the linings drawn at their thickness. A compliant felt law comes
/// later through the same rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContactLaw {
    /// A unilateral row.
    Rigid,
}

/// A declared contact: two surfaces of drawn parts on two bodies.
///
/// It holds nothing but ranges of drawn outlines, so whatever a contact reads
/// is something drawn. Side A and side B are the two bodies; every row of the
/// pair has its normal from B towards A. The key's pairs put the key on A.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContactPair {
    /// What it is.
    pub code: ContactCode,
    /// Side A.
    pub a: Surface,
    /// Side B.
    pub b: Surface,
    /// How the two are read against each other.
    pub geometry: PairGeometry,
    /// The law its force follows.
    pub law: ContactLaw,
}

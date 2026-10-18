//! Parts, and the ranges into their outlines that a contact may read.
//!
//! # One outline, drawn and touched
//!
//! If the surfaces a contact reads lived in a list *parallel* to the drawn
//! outline, the two would drift apart silently and the displayed dimensions
//! would stop describing what is measured. So a [`Part`] owns exactly one
//! outline, closed and counter-clockwise in its body's frame, and the only
//! geometry a contact may reach is a [`SurfaceRef`], a *range into that
//! outline*. A named point is an [`Anchor`] on a drawn segment, never a
//! coordinate typed beside it.
//!
//! A body carries several parts, each of one material: the key is a keystick
//! with a keytop, a key front, a key button and a back check block glued to
//! it, a felt on its tail and a capstan screwed into it. A cut out of a body,
//! a mortise, is a part too, of [`Role::Cut`]: its material lies outside its
//! outline, and it is drawn in the section as the void it is.
//!
//! # Contacts read the drawn ranges and nothing else
//!
//! A [`ContactPair`] holds two [`Surface`]s, each nothing but ranges of drawn
//! parts, and how the two are read against each other ([`PairGeometry`]). A
//! mechanism holds its parts and its pairs, and reads every pair at a pose
//! into [`Rows`] ([`crate::mechanism::Mechanism::rows`]): one row per pair
//! of features that meet, a signed gap with its normal, its point, and its
//! gradient and slip over every coordinate. How the key bears on a stop is
//! read off those rows ([`crate::mechanism::Mechanism::bearing`]). The three
//! signed primitives the rows are made of are in [`crate::geometry::signed`].
//!
//! # Built and planned
//!
//! Built: the parts of the key and of its frame
//! ([`crate::parts::KeyDesign::parts`], [`crate::parts::FrameDesign::parts`]),
//! drawn by the bench in each body's own frame and read by the faults
//! ([`crate::parts::faults`]); the key's contacts with its frame as pairs of
//! their declared ranges ([`crate::parts::mounts::key_on_frame`]), its
//! balance punching among them ([`crate::parts::mounts::balance_punching`]),
//! read into rows for the static solve that places the key
//! ([`crate::solve`]), for the bench's gaps, for how the key bears on its
//! cloth, and for the contacts the view marks; and how deep drawn parts stand
//! in one another, read off their outlines
//! ([`crate::mechanism::Mechanism::overlap`]), where a solve starts and
//! whether a state it finds is one. Nothing reads anything but a drawn range,
//! a key riding on a frame whose stops fight included: it rides on the same
//! pairs, lifted off its balance punching. The brute-force oracle holds every
//! row's points on the drawn outlines
//! (`every_contact_lies_on_its_drawn_outline`). Planned: a compliant felt law
//! through the same rows ([`ContactLaw`]).

mod bearing;
mod features;
#[cfg(test)]
pub(crate) mod fixtures;
#[cfg(test)]
pub(crate) mod oracle;
mod pair;
mod rows;

pub use bearing::{Bearing, HowItBears, FLAT_WITHIN};
pub(crate) use features::PairFeatures;
pub use features::{Feature, Piece};
pub use pair::{ContactCode, ContactLaw, ContactPair, PairGeometry, Surface};
pub use rows::{Row, Rows, ROW_MARGIN};

use core::ops::Range;

use crate::geometry::{Point, Seg};
use crate::mechanism::BodyId;

/// A part's place in the list of parts it was built into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartId(pub u16);

impl PartId {
    /// The index into the list.
    pub fn index(self) -> usize {
        usize::from(self.0)
    }
}

/// What a part is, as the trade names it.
///
/// The identifiers follow the glossary of
/// `docs/research/A3-nomenclature-and-regulation.md` > Bilingual glossary FR ↔ EN (+ DE) and code identifiers where it has one; the
/// keystick, the back check block and the key tail felt have none there yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PartCode {
    /// The key's own wood: one thickness, everything else glued to it or cut
    /// out of it.
    Keystick,
    /// The covering on top of the key, its lip overhanging the key front.
    Keytop,
    /// The plate glued over the keystick's front end grain.
    KeyFront,
    /// The block glued on the keystick where the balance mortise is cut.
    KeyButton,
    /// The block glued on the keystick for the back check wire.
    BackCheckBlock,
    /// The clearance cut in the keystick's top for the wippen rail, where the
    /// key has one: a shape of the keystick's outline, named on its own where
    /// a refusal places it.
    TopClearance,
    /// The hard felt on the key's thinned tail, where the damper bears.
    KeyTailFelt,
    /// The capstan screwed into the keystick.
    Capstan,
    /// The slot the front rail pin stands in, cut up from the underside.
    FrontMortise,
    /// The balance rail, its crown included.
    BalanceRail,
    /// The round pin the key turns on.
    BalanceRailPin,
    /// The felt washer on the balance rail.
    BalanceRailPunching,
    /// The front rail.
    FrontRail,
    /// The oblong pin standing in the key's front mortise.
    FrontRailPin,
    /// The felt washer on the front rail.
    FrontRailPunching,
    /// A paper under a punching.
    PaperPunching,
    /// The back rail.
    BackRail,
    /// The cloth strip on the back rail.
    BackRailCloth,
}

impl PartCode {
    /// The part's identifier, English `snake_case`.
    pub fn code(self) -> &'static str {
        match self {
            Self::Keystick => "keystick",
            Self::Keytop => "keytop",
            Self::KeyFront => "key_front",
            Self::KeyButton => "key_button",
            Self::BackCheckBlock => "back_check_block",
            Self::TopClearance => "top_clearance",
            Self::KeyTailFelt => "key_tail_felt",
            Self::Capstan => "capstan",
            Self::FrontMortise => "front_mortise",
            Self::BalanceRail => "balance_rail",
            Self::BalanceRailPin => "balance_rail_pin",
            Self::BalanceRailPunching => "balance_rail_punching",
            Self::FrontRail => "front_rail",
            Self::FrontRailPin => "front_rail_pin",
            Self::FrontRailPunching => "front_rail_punching",
            Self::PaperPunching => "paper_punching",
            Self::BackRail => "back_rail",
            Self::BackRailCloth => "back_rail_cloth",
        }
    }
}

/// What a part is to the body that carries it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// Material: wood, metal, a covering.
    Solid,
    /// Material taken away: its outline bounds a void in the body's solids,
    /// so a range on it faces into the void (the top of a mortise faces down,
    /// onto its pin), and its corners are concave from the wood's side.
    Cut,
    /// A compliant layer: felt, cloth, paper. Rigid today, as every stop is.
    Lining,
}

/// What a part is made of.
///
/// A property of the part; what color a material is drawn in is the view's
/// choice. Two parts of one material are the same stuff: the key button and
/// the back check block are of one wood, and it is not the keystick's.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Material {
    /// The keystick's wood.
    KeyWood,
    /// The wood of the blocks glued on the keystick, the key button and the
    /// back check block: a wood other than the keystick's.
    BlockWood,
    /// The keytop's covering.
    Covering,
    /// The key front's.
    KeyFront,
    /// The key frame's rails.
    RailWood,
    /// Pins and the capstan.
    Steel,
    /// The balance and front punchings.
    PunchingFelt,
    /// The hard felt on the key's tail.
    HardFelt,
    /// The back rail cloth.
    Cloth,
    /// A punching paper. The trade identifies a paper by its color, and the
    /// color is its thickness, so the stock is named by the thickness.
    Paper {
        /// The paper's thickness, meters.
        thickness: f64,
    },
}

/// How a part's outline becomes a solid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sweep {
    /// Swept sideways across the keyboard: keys, rails, the back rail cloth,
    /// an oblong pin. The outline is in its body's `x` and `y`.
    Extruded {
        /// Extent across the keyboard, meters. Zero where the part has none
        /// the model knows: a cut, which only the section draws.
        width: f64,
    },
    /// Turned about a vertical axis: punchings, papers, a round pin, the
    /// capstan. The outline is the **profile** in radius and height, the
    /// height in the body's `y`; see [`Part::silhouette`].
    Revolved {
        /// Where the axis stands, meters along the body's `x`.
        axis_x: f64,
    },
}

/// What places a stretch of an outline: the number of the design whose value
/// draws it there, by the name [`crate::validate::numbers`] gives it, as
/// `"stick_height"` places the keystick's top and `"front_edge"` its front.
///
/// Where that number comes from, measured on a part, read from a published
/// table, traced from a figure or assumed, is not the domain's to say: the
/// domain starts from no number of its own, and builds its parts from the
/// numbers an instrument hands it. The instrument's profile keeps each
/// number's provenance, and an application marks a drawn stretch by the
/// provenance of the number that places it.
pub type PlacedBy = &'static str;

/// One part: a closed outline on one body, of one material.
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    /// The body that carries it.
    pub body: BodyId,
    /// What it is.
    pub code: PartCode,
    /// Closed and counter-clockwise, in the body's frame: each segment ends
    /// where the next begins, bit for bit, and the last where the first
    /// begins. A turned part's is its profile, whose **last segment is its
    /// inner side**, the axis or the wall of its hole, and every other segment
    /// of which stands at a radius of zero or more.
    pub outline: Vec<Seg>,
    /// How the outline becomes a solid.
    pub sweep: Sweep,
    /// Material, void or lining.
    pub role: Role,
    /// What it is made of.
    pub material: Material,
    /// Which number places each run of segments of the outline, in order,
    /// covering it ([`PlacedBy`]).
    pub placed_by: Vec<(Range<u16>, PlacedBy)>,
}

/// `p` moved `dx` along `x`: a profile's point carried out to its axis.
fn shifted(p: Point, dx: f64) -> Point {
    Point::new(dx + p.x, p.y)
}

/// A segment of a profile, carried out to the axis at `axis_x`, its radius
/// multiplied by `side` (1 for the profile, −1 for its mirror).
fn about_axis(seg: Seg, axis_x: f64, side: f64) -> Seg {
    match seg {
        Seg::Line { a, b } => Seg::Line {
            a: shifted(Point::new(side * a.x, a.y), axis_x),
            b: shifted(Point::new(side * b.x, b.y), axis_x),
        },
        Seg::Arc { c, r, from, to } => {
            let center = shifted(Point::new(side * c.x, c.y), axis_x);
            if side > 0.0 {
                Seg::Arc {
                    c: center,
                    r,
                    from,
                    to,
                }
            } else {
                // A mirror turns a counter-clockwise sweep clockwise; run back
                // the other way, it is counter-clockwise again.
                let flip = |d: crate::geometry::Dir| {
                    let v = d.vector();
                    crate::geometry::Dir::new(-v.x, v.y).unwrap_or(d)
                };
                Seg::Arc {
                    c: center,
                    r,
                    from: flip(to),
                    to: flip(from),
                }
            }
        }
    }
}

/// The same segment run the other way. An arc stays as it is: it always runs
/// counter-clockwise, and [`about_axis`] has already run a mirrored one
/// backwards.
fn reversed(seg: Seg) -> Seg {
    match seg {
        Seg::Line { a, b } => Seg::Line { a: b, b: a },
        arc @ Seg::Arc { .. } => arc,
    }
}

impl Part {
    /// The part as the sagittal plane sees it, in its body's frame: the only
    /// geometry a [`SurfaceRef`] or an [`Anchor`] reads.
    ///
    /// A prism's is its outline. A turned part's is its profile and the
    /// profile's mirror about the axis, **with its hole hidden**: the key bears
    /// on a washer's annulus on both sides of the plane, so a washer's top
    /// runs from `−r_out` to `+r_out` across the axis. Derived from the stored
    /// profile by this one function, so what a range reads cannot drift from
    /// what is drawn. Built from the profile less its inner side, the start
    /// and end of what remains carried in to the axis; then that chain, then
    /// its mirror run backwards. A profile of `n` segments has a silhouette of
    /// `2(n − 1)`; where the profile reaches its top along its last segment
    /// before its inner side, as every turned part here does, the
    /// silhouette's top is the two segments either side of the axis, `n − 2`
    /// and `n − 1`.
    pub fn silhouette(&self) -> Vec<Seg> {
        let axis_x = match self.sweep {
            Sweep::Extruded { .. } => return self.outline.clone(),
            Sweep::Revolved { axis_x } => axis_x,
        };
        let n = self.outline.len();
        if n < 2 {
            return Vec::new();
        }
        let mut chain: Vec<Seg> = self.outline[..n - 1].to_vec();
        // Hide the hole: the chain's two ends are carried in to the axis.
        if let Some(Seg::Line { a, .. }) = chain.first_mut() {
            a.x = 0.0;
        }
        if let Some(Seg::Line { b, .. }) = chain.last_mut() {
            b.x = 0.0;
        }
        let mut out: Vec<Seg> = chain.iter().map(|&s| about_axis(s, axis_x, 1.0)).collect();
        out.extend(
            chain
                .iter()
                .rev()
                .map(|&s| reversed(about_axis(s, axis_x, -1.0))),
        );
        out
    }

    /// Whether the outline is closed: each segment ends where the next
    /// begins, and the last where the first begins, bit for bit.
    pub fn is_closed(&self) -> bool {
        let same =
            |p: Point, q: Point| p.x.to_bits() == q.x.to_bits() && p.y.to_bits() == q.y.to_bits();
        let n = self.outline.len();
        n >= 2 && (0..n).all(|i| same(self.outline[i].end(), self.outline[(i + 1) % n].start()))
    }
}

/// A range of segments of one part's [`Part::silhouette`]: the only geometry
/// a contact can reach.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SurfaceRef {
    /// The part.
    pub part: PartId,
    /// The segments, as indices into the part's silhouette.
    pub segs: Range<u16>,
}

impl SurfaceRef {
    /// The segments, in the part's body's frame.
    ///
    /// # Panics
    ///
    /// When `parts` is not the list the range was declared in: a part or a
    /// segment it does not have.
    pub fn segs(&self, parts: &[Part]) -> Vec<Seg> {
        let silhouette = parts[self.part.index()].silhouette();
        silhouette[usize::from(self.segs.start)..usize::from(self.segs.end)].to_vec()
    }
}

/// What a named point is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnchorCode {
    /// The top of the key at its key front: where key height and key dip are
    /// read.
    KeyFrontTop,
    /// The covering's lip, the front edge of the keytop: where every
    /// touchweight's measuring point is taken back from.
    Lip,
    /// The top of the capstan on its axis: where the wippen's heel bears, and
    /// where the key weight ratio's 10 g is centered.
    CapstanContact,
}

/// A material point on a drawn segment.
///
/// Never an extreme vertex found by searching: which vertex is extreme
/// changes as a body turns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Anchor {
    /// The part.
    pub part: PartId,
    /// The segment, as an index into the part's silhouette.
    pub seg: u16,
    /// How far along the segment, from 0 at its start to 1 at its end: along
    /// a line, of its length; along an arc, of its sweep.
    pub at: f64,
    /// What it is for.
    pub code: AnchorCode,
}

impl Anchor {
    /// The point, in its part's body's frame. At 0 it is the segment's start
    /// bit for bit.
    ///
    /// # Panics
    ///
    /// When `parts` is not the list the anchor was declared in.
    pub fn point(&self, parts: &[Part]) -> Point {
        let seg = parts[self.part.index()].silhouette()[usize::from(self.seg)];
        match seg {
            Seg::Line { a, b } => {
                Point::new(a.x + self.at * (b.x - a.x), a.y + self.at * (b.y - a.y))
            }
            Seg::Arc { c, r, from, to } => {
                let (u, v) = (from.vector(), to.vector());
                let mut sweep = libm::atan2(u.x * v.y - u.y * v.x, u.x * v.x + u.y * v.y);
                if sweep < 0.0 {
                    sweep += 2.0 * core::f64::consts::PI;
                }
                let turn = self.at * sweep;
                let (s, k) = (libm::sin(turn), libm::cos(turn));
                Point::new(c.x + r * (k * u.x - s * u.y), c.y + r * (s * u.x + k * u.y))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Dir;

    fn lines(points: &[Point]) -> Vec<Seg> {
        (0..points.len())
            .map(|i| Seg::Line {
                a: points[i],
                b: points[(i + 1) % points.len()],
            })
            .collect()
    }

    /// A washer of 2 to 11 mm turned about an axis 225 mm along its body, as
    /// the front punching is.
    fn washer() -> Part {
        Part {
            body: BodyId::GROUND,
            code: PartCode::FrontRailPunching,
            outline: lines(&[
                Point::new(0.0025, 0.026),
                Point::new(0.011, 0.026),
                Point::new(0.011, 0.028),
                Point::new(0.0025, 0.028),
            ]),
            sweep: Sweep::Revolved { axis_x: 0.225 },
            role: Role::Lining,
            material: Material::PunchingFelt,
            placed_by: vec![(0..4, "a test")],
        }
    }

    /// A washer's silhouette is the strip it presents to a key landing on it:
    /// its hole hidden, its top running from −r_out to +r_out across the axis.
    #[test]
    fn a_washer_presents_its_whole_width_with_its_hole_hidden() {
        let part = washer();
        let silhouette = part.silhouette();
        assert_eq!(silhouette.len(), 2 * (part.outline.len() - 1));
        let silhouette_part = Part {
            outline: silhouette.clone(),
            sweep: Sweep::Extruded { width: 0.0 },
            ..part.clone()
        };
        assert!(silhouette_part.is_closed(), "{silhouette:?}");
        // The top: the two segments either side of the axis.
        let n = part.outline.len();
        let top = SurfaceRef {
            part: PartId(0),
            segs: (n as u16 - 2)..(n as u16),
        }
        .segs(&[part]);
        let (front, back) = (top[0].start(), top[1].end());
        assert_eq!(front.x.to_bits(), (0.225_f64 + 0.011).to_bits());
        assert_eq!(back.x.to_bits(), (0.225_f64 - 0.011).to_bits());
        for seg in &top {
            assert_eq!(seg.start().y.to_bits(), 0.028_f64.to_bits());
            assert_eq!(seg.end().y.to_bits(), 0.028_f64.to_bits());
        }
        // And nothing of it stands inside the hole but the hidden top and
        // bottom running across it.
        for seg in &silhouette {
            let (a, b) = (seg.start(), seg.end());
            let vertical = a.x.to_bits() == b.x.to_bits();
            assert!(
                !(vertical && (a.x - 0.225).abs() < 0.0025),
                "the hole's wall is drawn into the silhouette: {seg:?}"
            );
        }
    }

    /// A part's outline is closed bit for bit, or it is not.
    #[test]
    fn a_gap_of_one_bit_leaves_an_outline_open() {
        let mut part = washer();
        assert!(part.is_closed());
        if let Seg::Line { b, .. } = &mut part.outline[1] {
            b.y = f64::from_bits(b.y.to_bits() + 1);
        }
        assert!(!part.is_closed());
    }

    /// An anchor at 0 is its segment's start, to the bit; along an arc it
    /// turns with the arc.
    #[test]
    fn an_anchor_stands_on_its_segment() {
        let parts = [washer()];
        let start = Anchor {
            part: PartId(0),
            seg: 2,
            at: 0.0,
            code: AnchorCode::Lip,
        };
        assert_eq!(start.point(&parts), parts[0].silhouette()[2].start());

        let arc = Part {
            outline: vec![
                Seg::Arc {
                    c: Point::new(0.0, 0.0),
                    r: 1.0,
                    from: Dir::X,
                    to: Dir::NEG_X,
                },
                Seg::Line {
                    a: Point::new(-1.0, 0.0),
                    b: Point::new(1.0, 0.0),
                },
            ],
            sweep: Sweep::Extruded { width: 0.0 },
            ..washer()
        };
        let quarter = Anchor {
            part: PartId(0),
            seg: 0,
            at: 0.5,
            code: AnchorCode::Lip,
        }
        .point(&[arc]);
        assert!(
            quarter.x.abs() < 1e-15 && (quarter.y - 1.0).abs() < 1e-15,
            "{quarter:?}"
        );
    }
}

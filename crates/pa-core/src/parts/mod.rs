//! Parametric part definitions, the parts they build, and what is known of
//! a key's mass.
//!
//! # One key, drawn and touched
//!
//! [`KeyDesign`] holds the key's dimensions, and its **parts** ([`key`]),
//! one closed outline per piece of material, are built from them: what is
//! drawn, what the faults read and what the key's contacts with its frame
//! read ([`mounts::key_on_frame`]); [`FrameDesign`] builds the frame's the
//! same way ([`frame`]).
//!
//! # A key's mass is weighed, never integrated
//!
//! A key is weighed whole, its leads in it: its mass and its front weight,
//! which gives its static moment about the balance pin ([`key_mass`]); or a
//! source publishes it whole, its center of mass's height included. Until it
//! is weighed it has no mass, and what reads one is refused. An outline draws
//! a key and never weighs it: the one number of a weighed key no scale gives,
//! the height of its center of mass, is estimated from its drawn section's
//! area centroid ([`KeyDesign::section_centroid`]), and every reading it
//! reaches names it (`docs/FOUNDATIONS.md` > Named simplifications).
//!
//! # The key's local frame
//!
//! Origin at the **balance rail pin**, `x` positive toward the player, `y`
//! positive up. That is the datum the trade already uses — the Reyburn sheet
//! measures from the balance hole. Every longitudinal dimension in
//! [`KeyDesign`] is therefore signed: positive in front of the balance pin,
//! negative behind it.

pub mod faults;
#[cfg(test)]
pub(crate) mod fixtures;
pub mod frame;
pub mod key;
pub mod key_mass;
pub mod mounts;
pub mod verdicts;

pub use faults::Fault;
pub use frame::{FrameDesign, FrameRanges, PunchingStack};
pub use key::KeyRanges;
pub use key_mass::{FrontWeightReading, KeyMass, NotAWeighing};

use crate::contact::{Part, PartId};
use crate::geometry::{Dir, Point, Seg, DRAWN_HALVINGS};
use crate::registry::{Kind, Layer, ParamSpec, Quantity, Step};

/// A closed outline through `points`, in order.
fn polygon(points: &[Point]) -> Vec<Seg> {
    (0..points.len())
        .map(|i| Seg::Line {
            a: points[i],
            b: points[(i + 1) % points.len()],
        })
        .collect()
}

/// A part's index as its list will hold it.
fn next_id(out: &[Part]) -> PartId {
    PartId(u16::try_from(out.len()).expect("a scene holds fewer than 65 536 parts"))
}

/// The number of segments in an outline, as a range bound.
fn count(outline: &[Seg]) -> u16 {
    u16::try_from(outline.len()).expect("an outline holds fewer than 65 536 segments")
}

/// The area of the closed polygon through `vertices`, in order, and its
/// first moments about the two axes, `∫ x dA` and `∫ y dA`: the shoelace
/// sums, normalized so that the area is positive whichever way the polygon
/// winds, a draftsman's habit and not a modeling choice. Zero for a
/// polygon that encloses nothing.
pub fn area_and_moments(vertices: &[Point]) -> (f64, Point) {
    let (mut twice, mut along, mut across) = (0.0, 0.0, 0.0);
    for i in 0..vertices.len() {
        let p = vertices[i];
        let q = vertices[(i + 1) % vertices.len()];
        let cross = p.x * q.y - q.x * p.y;
        twice += cross;
        along += (p.x + q.x) * cross;
        across += (p.y + q.y) * cross;
    }
    let sign = if twice < 0.0 { -1.0 } else { 1.0 };
    (
        sign * 0.5 * twice,
        Point::new(sign * along / 6.0, sign * across / 6.0),
    )
}

/// The area centroid of the closed polygon through `vertices`, in order,
/// whichever way it winds; `None` for one that encloses nothing, fewer than
/// three vertices or all of them on a line, which an edited outline can
/// reach and which would otherwise surface as a `NaN` far from its cause.
pub fn area_centroid(vertices: &[Point]) -> Option<Point> {
    let (area, moments) = area_and_moments(vertices);
    if vertices.len() < 3 || area.is_nan() || area <= 0.0 {
        return None;
    }
    Some(Point::new(moments.x / area, moments.y / area))
}

/// The capstan screw standing on the back of the key.
///
/// Part of the key, not of the action above it: it is screwed into the
/// keystick and it moves with the key, its mass in the key's weighing.
/// Modeling it as an anchor point leaves it out of the silhouette and out of
/// the drawing, and a technician looking at the section notices immediately.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Capstan {
    /// How far the **top of the head** stands above the keystick, meters.
    /// This is what screwing it up and down changes, and it is the dimension
    /// a technician sets.
    pub height: f64,
    /// Threaded shaft diameter, meters.
    pub diameter: f64,
    /// Head diameter, meters. Wider than the shaft: it is the head's flat top
    /// that bears on the wippen heel.
    pub head_diameter: f64,
    /// Height of the cylindrical part of the head, meters: up to its flat
    /// top, or to where its dome begins.
    pub head_height: f64,
    /// Height of the cove that flares the shaft out into the head, meters.
    pub cove_height: f64,
    /// Length of the threaded shaft below the cove, meters, part of it
    /// buried in the key.
    pub shaft_length: f64,
    /// Length of the conical tip at the bottom of the shaft, meters.
    pub tip_height: f64,
    /// The radius of the dome its head is turned to, meters, where its top
    /// is one: the head's top is then a spherical cap of this radius, its
    /// center on the axis this far below the top, which bears on the wippen's
    /// heel as a circle does; none where the top is flat. No smaller than
    /// the head's radius, or the dome would not cap the head
    /// ([`crate::validate`]).
    pub dome_radius: Option<f64>,
}

impl Capstan {
    /// How far below its top the head's side ends: where a dome meets it,
    /// the dome's sagitta over the head's radius; nothing for a flat top.
    pub fn dome_sagitta(&self) -> f64 {
        match self.dome_radius {
            None => 0.0,
            Some(r) => {
                let half = 0.5 * self.head_diameter;
                r - (r * r - half * half).sqrt()
            }
        }
    }

    /// The half-profile to turn, as `(radius, height above the keystick)`
    /// points running from the bottom of the shaft to the center of the flat
    /// top.
    ///
    /// Shaped from a photograph of a real capstan: a pointed, threaded shaft;
    /// a cove flaring out; a short cylindrical head; a flat top, or a dome
    /// ([`Self::dome_radius`]), whose profile runs here as the chord from the
    /// head's side to the top on the axis: the drawn part turns that chord
    /// into the arc ([`KeyDesign::parts`]). The thread itself is not
    /// drawn: a helix is not a profile.
    pub fn profile(&self) -> Vec<(f64, f64)> {
        let (r_shaft, r_head) = (0.5 * self.diameter, 0.5 * self.head_diameter);
        let top = self.height;
        // Where the head's side ends: the top itself where it is flat.
        let shoulder = match self.dome_radius {
            None => top,
            Some(_) => top - self.dome_sagitta(),
        };
        let head_bottom = shoulder - self.head_height;
        let cove_bottom = head_bottom - self.cove_height;
        // The shaft runs down into the key, drawn as the full screw.
        let shaft_bottom = cove_bottom - self.shaft_length;
        vec![
            (0.0, shaft_bottom),
            (r_shaft, shaft_bottom + self.tip_height),
            (r_shaft, cove_bottom),
            (r_head, head_bottom),
            (r_head, shoulder),
            (0.0, top),
        ]
    }
}

/// A clearance cut in the keystick's top, where the key has one, so that the
/// **wippen rail**, which runs across the keyboard over the keys between
/// their capstans and their back check blocks, cannot rub the key: the top
/// lowered by its depth between its two ends, the key coming back to its
/// full height behind it, where the back check block is glued on.
///
/// Placed from the balance pin, as the back check block is
/// ([`KeyDesign::back_check_block_back`]), and for the same reason: it is
/// there to clear a part of the action, which the action places, and a key
/// lengthened or shortened at either end keeps it under the rail. Cut across
/// the key's whole width, as the rail it clears runs, it is a shape of the
/// keystick's outline, drawn and counted in its section as the thinned tail
/// is, and not a void of its own as the front mortise is. It stands on the
/// keystick's full height, between the back check block and the capstan's
/// head, and leaves wood under it ([`crate::validate`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopClearance {
    /// Its back end, meters from the balance pin (negative behind it).
    pub back: f64,
    /// Its front end, meters from the balance pin (negative behind it).
    pub front: f64,
    /// How deep it is cut below the keystick's full top, meters.
    pub depth: f64,
}

/// The balance mortise, as the owner describes it: an oblong slot through
/// the key button, its upper part bushed on its two long sides, which guides
/// the key across the keyboard, the cloth on each side turning out at its
/// foot through a side hole in the key button's wall; under it a cavity in
/// the keystick, as long as the slot; and at that cavity's floor, just above
/// the underside, the balance hole, fitted to the pin, where the key is held
/// fore and aft. The
/// slot runs along the key's front axis, the model's `x`, whatever the key's
/// crank behind its covering, so that every key rocks the same way;
/// centered on the balance pin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BalanceMortise {
    /// Length of the slot along the key, meters; the cavity under it is as
    /// long.
    pub length: f64,
    /// Width of the slot across the keyboard between its bare walls,
    /// meters: the cavity under it is as wide, and the bushing takes its two
    /// thicknesses out of it.
    pub width: f64,
    /// Thickness of the keystick's wood under the cavity, meters, where the
    /// balance hole is drilled.
    pub floor: f64,
    /// Diameter of the balance hole, meters: fitted to the pin.
    pub hole_diameter: f64,
    /// Thickness of the cloth on each long side of the slot, meters.
    pub bushing_thickness: f64,
    /// How far down the slot that cloth runs from the key button's top,
    /// meters: only its upper part is bushed. At its foot the cloth turns
    /// out through the key button's side holes, glued against their top.
    pub bushing_depth: f64,
    /// Height of the key button's side holes, meters: no less than the
    /// thickness of the cloth turned out through them, which is glued
    /// against their top; taller, it passes through them easily.
    pub side_hole_height: f64,
}

/// A glued block's front, top and back, counter-clockwise from the foot of
/// its front to the foot of its back, its two top corners rounded to
/// `radius`. A radius up to the block's height rounds each corner into the
/// block's end, which runs straight under the arc; a larger one runs each
/// arc from the top down to the foot of the end, the block's length still
/// its footprint's; a radius of zero leaves the corners square. Each arc
/// reaches in from its end by [`end_reach`], and the two meet at most.
pub(crate) fn rounded_ends(back: f64, front: f64, base: f64, height: f64, radius: f64) -> Vec<Seg> {
    let top = base + height;
    let (front_foot, back_foot) = (Point::new(front, base), Point::new(back, base));
    let square = || {
        polygon(&[
            front_foot,
            Point::new(front, top),
            Point::new(back, top),
            back_foot,
        ])[..3]
            .to_vec()
    };
    if radius <= 0.0 {
        return square();
    }
    let reach = end_reach(height, radius);
    let drop = if radius > height {
        radius - height
    } else {
        0.0
    };
    let (Some(from_front), Some(to_back)) = (Dir::new(reach, drop), Dir::new(-reach, drop)) else {
        return square();
    };
    let front_arc = Seg::Arc {
        c: Point::new(front - reach, top - radius),
        r: radius,
        from: from_front,
        to: Dir::Y,
    };
    let back_arc = Seg::Arc {
        c: Point::new(back + reach, top - radius),
        r: radius,
        from: Dir::Y,
        to: to_back,
    };
    let mut segs = Vec::with_capacity(5);
    if radius < height {
        segs.push(Seg::Line {
            a: front_foot,
            b: front_arc.start(),
        });
    }
    segs.push(front_arc);
    if front_arc.end() != back_arc.start() {
        segs.push(Seg::Line {
            a: front_arc.end(),
            b: back_arc.start(),
        });
    }
    segs.push(back_arc);
    if radius < height {
        segs.push(Seg::Line {
            a: back_arc.end(),
            b: back_foot,
        });
    }
    segs
}

/// How far a rounded end of a block `height` high reaches in from its end
/// along the block's top, at `radius`: the radius itself up to the
/// block's height, where the arc rounds a corner; beyond, the arc runs
/// from the top to the foot of the end, which lies `√(2rh − h²)` out from
/// its tangent point on the top. Both no less than nothing, as their
/// settings admit them, so the root is never of a negative number.
pub fn end_reach(height: f64, radius: f64) -> f64 {
    if radius <= height {
        radius
    } else {
        (2.0 * radius * height - height * height).sqrt()
    }
}

/// A block glued on the key's top: where it stands along the key and how
/// high it stands above the wood under it.
///
/// It stands on whatever wood is under it, the keystick's full height or its
/// thinned tail, wholly on one or the other ([`KeyDesign::wood_under`],
/// [`crate::validate`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GluedBlock {
    /// Its back end, meters from the balance pin (negative behind it).
    pub back: f64,
    /// Its length along the key, meters, forward from its back end.
    pub length: f64,
    /// Its height above the wood it stands on, meters.
    pub height: f64,
}

impl GluedBlock {
    /// Its front end, meters from the balance pin.
    pub fn front(&self) -> f64 {
        self.back + self.length
    }
}

/// Everything about one key that the choice of parts fixes.
///
/// Longitudinal dimensions are signed distances from the balance pin, positive
/// toward the player. The ones that carry the trade's own meaning are named
/// after where a technician measures: the front edge, where key height and dip
/// are read; the covering's lip ([`Self::lip`]), which the touchweight
/// measuring point is taken back from; and the capstan.
///
/// The measuring point itself is **not** here. Where a weight is centered is a
/// convention of the protocol, not a dimension of the wood, so it is set on
/// [`crate::measure::TouchweightProtocol`] and read through
/// [`crate::measure::TouchweightProtocol::measuring_point`].
#[derive(Debug, Clone, PartialEq)]
pub struct KeyDesign {
    /// The key front, meters in front of the balance pin: the outer face of
    /// the plate glued over the end grain ([`Self::front_board`]).
    ///
    /// Key height and key dip are read here. The covering's lip stands
    /// [`Self::keytop_overhang`] proud of it.
    pub front_edge: f64,
    /// Rear end of the keystick, meters (negative: behind the balance pin).
    pub tail_end: f64,
    /// Capstan axis, meters (negative).
    pub capstan: f64,
    /// Front mortise, meters back from the key's **front edge**.
    ///
    /// Back from the front edge and not forward from the balance pin, because
    /// that is the dimension a key carries: the slot is cut near the front end
    /// of the keystick, and lengthening a key moves its front end and its
    /// mortise together. Stated the other way round, a longer key would leave
    /// its own mortise behind and the front rail pin would stand in fresh air.
    ///
    /// It is a **key** dimension and the front rail's position is a *frame*
    /// one. They have to agree, and [`faults::faults`] is what says
    /// so when they do not; what places the rail by default is
    /// [`Self::front_mortise_x`]. [E]
    pub front_mortise_from_front_edge: f64,
    /// Length of the front mortise along the key, meters.
    ///
    /// The slot is longer along the key than the pin standing in it. It has to
    /// be: the key swings about the balance pin, so the mortise travels through
    /// a small arc past a pin that does not move, and a slot cut to the pin's
    /// own length would bind at the ends of the stroke. The clearance is also
    /// what lets a frame be a few tenths out of place without the key knowing.
    ///
    /// What it is **not** is the lateral fit. Across the keyboard the mortise is
    /// bushed and close on the pin's wide faces — that is what holds the key
    /// straight — and that dimension is not in the sagittal plane. [E]
    pub front_mortise_length: f64,
    /// Depth of the front mortise above the key's underside, meters.
    ///
    /// The slot the front rail pin stands in, measured from the underside, so
    /// it already includes the recess the slot begins after: 3 mm of notch and
    /// 19 mm of mortise is 22 mm of room. [V]
    ///
    /// It is a **stop**, not decoration: the key cannot descend past the point
    /// where the pin's tip reaches the top of the slot, whatever the punchings
    /// underneath would otherwise allow.
    pub front_mortise_depth: f64,
    /// Thickness of the keystick's own wood, meters.
    ///
    /// **One piece, one thickness.** Everything else on the key is either glued
    /// to this — the covering, the capsule, the block that carries the back
    /// check — or cut out of it. Reading the key as a stack of steps gets the
    /// mass about right and the reasons entirely wrong, and the reasons are
    /// what say where a step may be moved and where it may not.
    pub stick_height: f64,
    /// Height of the **capsule** above the keystick, meters.
    ///
    /// The block glued on top that the balance mortise is cut through — more
    /// wood where the pin passes, which is the whole reason it is there.
    pub capsule_height: f64,
    /// Length of the capsule along the key, meters.
    pub capsule_length: f64,
    /// Where the capsule's middle stands along the key, meters from the
    /// balance pin; none where it stands centered on the pin.
    pub capsule_center: Option<f64>,
    /// Radius of the key button's two rounded ends, meters, from its flat
    /// top down to the key, each reaching in by [`end_reach`]; zero for
    /// square ends.
    pub capsule_end_radius: f64,
    /// The balance mortise, through the key button and the keystick.
    pub balance_mortise: BalanceMortise,
    /// Height of the block that carries the **back check**, above the wood
    /// it stands on: the keystick's full height, or the thinned tail where
    /// the tail runs on under it ([`Self::wood_under`]).
    ///
    /// Glued on, and there to give the back check wire something to be driven
    /// into — which is why it sits where the hammer's tail arrives and nowhere
    /// else.
    pub back_check_block_height: f64,
    /// Back end of that block, meters from the balance pin (negative: behind
    /// it).
    ///
    /// From the balance pin, and not from the key's end, because the
    /// **action** places the block: it stands where the hammer's tail falls
    /// as the back check catches it, and that is fixed by where the hammer
    /// stands over the key, not by how long the key is. Lengthen a key at the
    /// back and the block stays under the hammer's tail, the keystick running
    /// at its full height between the block and the thinned tail; measured
    /// from the key's end, a longer key would carry its block away from the
    /// hammer it is there to check.
    ///
    /// It stands wholly on the keystick's full height or wholly on the
    /// thinned tail, and the capstan stands in front of it
    /// ([`crate::validate`]).
    pub back_check_block_back: f64,
    /// Length of that block along the key, meters, forward from its back end
    /// ([`Self::back_check_block_back`]).
    pub back_check_block_length: f64,
    /// The clearance cut in the keystick's top for the wippen rail, between
    /// the capstan and the back check block, where the key has one; none
    /// where its top runs on at its full height.
    pub top_clearance: Option<TopClearance>,
    /// Thickness of the key's thinned tail, meters, above the underside.
    ///
    /// The last stretch of the key is cut down so that the **damper** system
    /// can bear on it: the felt on top of it is the contact, and the wood is
    /// taken away to put that contact at the height the damper lever wants.
    pub tail_height: f64,
    /// Length of that thinned tail along the key, meters, forward from the
    /// key's end ([`Self::tail_end`]).
    ///
    /// Where it carries a felt, it ends at or behind the back check block's
    /// back end, and where it ends behind it, the keystick runs at its full
    /// height between the two; bare, it may run on under the back check
    /// block, which stands wholly on it or wholly in front of it. The capstan
    /// and a clearance in the top stand in front of it, on the full height.
    pub tail_length: f64,
    /// Thickness of the felt on the tail, meters, hard felt and not punching
    /// felt; none where the tail is bare.
    pub damper_felt: Option<f64>,
    /// How far the underside rises toward the very end of the key, meters.
    pub tail_rise: f64,
    /// The length that rise is spread over, meters.
    ///
    /// Long and shallow: 2.5 mm over 77 mm, which is 1.86°. **That is the
    /// angle the key is meant to stand at when released**, and the reason the
    /// relief exists: the cloth is a flat strip, so relieving the underside by
    /// the attitude the key stands at puts the two faces parallel and the load
    /// on the whole strip instead of on a corner. The owner's own account.
    ///
    /// The attitude it has to match is the dip over the arm to the front of
    /// the key, 2.26° here, so on these figures the two do not agree and the
    /// key bears on the edge of its cloth by 0.4°. Both numbers were given as
    /// approximate; the bench reports the miss rather than arbitrating.
    pub tail_rise_run: f64,
    /// Thickness of the keytop covering, meters.
    ///
    /// It sits **on** the wood rather than in a rebate, so the front of the
    /// key is the full stick plus this, and the step a reader sees partway
    /// along the key is the covering ending — not the wood thinning.
    pub keytop_thickness: f64,
    /// Length of the keytop covering along the key, meters.
    pub keytop_length: f64,
    /// Thickness of the **key front** glued to the wood's front face, meters.
    ///
    /// The small plate — the *fronton* — that closes the end grain. Its outer
    /// face is what [`KeyDesign::front_edge`] names, where key height and dip
    /// are read. Touchweights are referred to the covering's lip instead,
    /// [`Self::keytop_overhang`] further forward.
    pub front_board: f64,
    /// How far the covering's lip overhangs that face, meters.
    ///
    /// The lip is the "front lip of the key" every touchweight's measuring
    /// point is taken back from ([`Self::lip`]), so this is the distance
    /// between the touchweight datum and the key height and dip datum.
    pub keytop_overhang: f64,
    /// Width of the keystick across the sagittal plane, meters: what the
    /// solid view extrudes the key by. Nothing weighs it: a key's mass is
    /// weighed or published ([`KeyMass`]).
    pub width: f64,
    /// The capstan screwed into the back of the key.
    pub capstan_screw: Capstan,
}

/// A dimension of the key: no less than nothing, and under a meter, longer
/// than any key, so that no setting squares or cubes a length past what a
/// number holds.
const fn key_length(
    name: &'static str,
    get: fn(&KeyDesign) -> f64,
    set: fn(&mut KeyDesign, f64),
) -> ParamSpec<KeyDesign> {
    ParamSpec {
        name,
        quantity: Quantity::Length,
        hard: (0.0, 1.0),
        step: Step::Continuous,
        layer: Layer::Design,
        kind: Kind::Actuated,
        get,
        set,
    }
}

impl KeyDesign {
    /// The key's settings: what an instrument may set on it, each with what
    /// it admits ([`ParamSpec`]).
    ///
    /// The key front and the capstan are signed from the balance pin like
    /// every other length here: the front in front of it, the capstan behind
    /// it. Whether they leave the key buildable, the covering clear of the
    /// key button and the capstan clear of the glued blocks, of the thinned
    /// tail and of a clearance in the top, is [`crate::validate`]'s to say,
    /// since it reads several at once. The
    /// capstan turns continuously, a screw having no quantum, within the
    /// 2 to 30 mm above the keystick the bench has always held it to. The
    /// key front, the front mortise's depth and the key button's and the
    /// balance mortise's dimensions are no less than nothing and under a
    /// meter; whether they fit each other and the keystick is
    /// [`crate::validate`]'s to say too.
    pub const PARAMS: [ParamSpec<Self>; 13] = [
        key_length("front_edge", |k| k.front_edge, |k, v| k.front_edge = v),
        ParamSpec {
            name: "capstan.x",
            quantity: Quantity::Length,
            hard: (f64::NEG_INFINITY, 0.0),
            step: Step::Continuous,
            layer: Layer::Design,
            kind: Kind::Actuated,
            get: |k| k.capstan,
            set: |k, v| k.capstan = v,
        },
        ParamSpec {
            name: "capstan.height",
            quantity: Quantity::Length,
            hard: (0.002, 0.030),
            step: Step::Continuous,
            layer: Layer::Regulation,
            kind: Kind::Actuated,
            get: |k| k.capstan_screw.height,
            set: |k, v| k.capstan_screw.height = v,
        },
        key_length(
            "front_mortise.depth",
            |k| k.front_mortise_depth,
            |k, v| k.front_mortise_depth = v,
        ),
        key_length(
            "capsule_height",
            |k| k.capsule_height,
            |k, v| k.capsule_height = v,
        ),
        key_length(
            "capsule_length",
            |k| k.capsule_length,
            |k, v| k.capsule_length = v,
        ),
        key_length(
            "capsule_end_radius",
            |k| k.capsule_end_radius,
            |k, v| k.capsule_end_radius = v,
        ),
        key_length(
            "balance_mortise.length",
            |k| k.balance_mortise.length,
            |k, v| k.balance_mortise.length = v,
        ),
        key_length(
            "balance_mortise.width",
            |k| k.balance_mortise.width,
            |k, v| k.balance_mortise.width = v,
        ),
        key_length(
            "balance_mortise.floor",
            |k| k.balance_mortise.floor,
            |k, v| k.balance_mortise.floor = v,
        ),
        key_length(
            "balance_mortise.hole_diameter",
            |k| k.balance_mortise.hole_diameter,
            |k, v| k.balance_mortise.hole_diameter = v,
        ),
        key_length(
            "balance_mortise.bushing_thickness",
            |k| k.balance_mortise.bushing_thickness,
            |k, v| k.balance_mortise.bushing_thickness = v,
        ),
        key_length(
            "balance_mortise.bushing_depth",
            |k| k.balance_mortise.bushing_depth,
            |k, v| k.balance_mortise.bushing_depth = v,
        ),
    ];

    /// The covering's lip, meters in front of the balance pin: the front edge
    /// of the keytop, standing [`Self::keytop_overhang`] proud of the key
    /// front.
    ///
    /// The bench's reading of Stanwood's *"front lip of the key"*: the datum
    /// the touchweight measuring point is taken back from
    /// ([`crate::measure::TouchweightProtocol::measuring_point`]). One
    /// definition, which the drawn outline uses too, so the lip a reader sees
    /// is the one the touchweights are referred to.
    pub fn lip(&self) -> f64 {
        self.front_edge + self.keytop_overhang
    }

    /// The front of the thinned tail, meters from the balance pin: where the
    /// keystick steps down to it.
    pub fn tail_front(&self) -> f64 {
        self.tail_end + self.tail_length
    }

    /// The height of the wood a block glued from `back` to `front` stands
    /// on, meters above the underside: the thinned tail's top where the
    /// block stands wholly on the tail, the keystick's top otherwise.
    pub fn wood_under(&self, back: f64, front: f64) -> f64 {
        debug_assert!(back <= front);
        if front <= self.tail_front() {
            self.tail_height
        } else {
            self.stick_height
        }
    }

    /// The key button, as the two x it runs between, meters: centered on the
    /// balance pin, or on [`Self::capsule_center`].
    pub fn key_button_span(&self) -> (f64, f64) {
        match self.capsule_center {
            None => (-self.capsule_length * 0.5, self.capsule_length * 0.5),
            Some(c) => (c - self.capsule_length * 0.5, c + self.capsule_length * 0.5),
        }
    }

    /// The back check block, as a glued block.
    pub fn back_check_block(&self) -> GluedBlock {
        GluedBlock {
            back: self.back_check_block_back,
            length: self.back_check_block_length,
            height: self.back_check_block_height,
        }
    }

    /// Middle of the front mortise, meters in front of the balance pin.
    ///
    /// **Where the front rail pin belongs**, and therefore where the frame
    /// places it. The key states its mortise as a setback from its own front
    /// edge; this is that setback resolved against the balance pin, which is
    /// the datum everything else on the frame is measured from.
    pub fn front_mortise_x(&self) -> f64 {
        self.front_edge - self.front_mortise_from_front_edge
    }

    /// The front mortise, as the two x it runs between, meters.
    pub fn front_mortise_span(&self) -> (f64, f64) {
        let half = 0.5 * self.front_mortise_length;
        let x = self.front_mortise_x();
        (x - half, x + half)
    }

    /// The flat of the underside, as the two x it runs between, meters.
    ///
    /// **This is the bearing surface**, and it is not the whole key: the tail is
    /// chamfered over its last stretch, behind everything that touches. The
    /// drawn key declares the same underside as ranges of its parts
    /// ([`KeyRanges::underside`]), built from the same fields, and the
    /// contacts read those.
    pub fn underside_flat(&self) -> (f64, f64) {
        (self.tail_end + self.tail_rise_run, self.front_edge)
    }

    /// The key's whole sagittal section as one polygon, an ordered list of
    /// vertices, the covering and the key front counted with the wood and the
    /// front mortise not taken out: what the drawn parts ([`Self::parts`])
    /// tile exactly, for the tests that hold them to it.
    #[cfg(test)]
    pub(crate) fn outline_vertices(&self) -> Vec<Point> {
        let top = self.stick_height;
        let wood_front = self.front_edge - self.front_board;
        let lip = self.lip();
        let keytop_back = lip - self.keytop_length;
        let tail_front = self.tail_front();
        let rise_from = self.tail_end + self.tail_rise_run;
        let tail_top = match self.damper_felt {
            Some(felt) => self.tail_height + felt,
            None => self.tail_height,
        };

        let mut outline = vec![
            // The underside, in two straight faces. The flat carries the
            // balance punching and the front punching; the last 77 mm of the
            // tail is relieved by 2.5 mm so that the key beds flat on its back
            // rail cloth, which is what the back rail bears on. Breaking
            // either face breaks a stop contact.
            Point::new(self.tail_end, self.tail_rise),
            Point::new(rise_from, 0.0),
            Point::new(wood_front, 0.0),
            // The key front: a plate glued over the end grain. Its outer face
            // is where key height and dip are read.
            Point::new(self.front_edge, 0.0),
            Point::new(self.front_edge, top),
            // The covering's lip, which overhangs that face with nothing
            // under it. Every touchweight's measuring point is taken back
            // from it.
            Point::new(lip, top),
            Point::new(lip, top + self.keytop_thickness),
            Point::new(keytop_back, top + self.keytop_thickness),
            Point::new(keytop_back, top),
        ];
        // What stands on the top or is cut into it, front to back: the
        // capsule, the block the balance mortise is cut through, on the
        // keystick's top; the capstan screws straight into the wood, with no
        // pad under it to draw; the clearance for the wippen rail, where
        // there is one, a block of negative height on the full top; and the
        // block that carries the back check, on the wood under it.
        let (capsule_back, capsule_front) = self.key_button_span();
        let mut blocks = vec![(
            capsule_back,
            capsule_front,
            self.wood_under(capsule_back, capsule_front),
            self.capsule_height,
            self.capsule_end_radius,
        )];
        if let Some(c) = self.top_clearance {
            blocks.push((c.back, c.front, top, -c.depth, 0.0));
        }
        let check = self.back_check_block();
        blocks.push((
            check.back,
            check.front(),
            self.wood_under(check.back, check.front()),
            check.height,
            0.0,
        ));
        let mut stepped = false;
        let step = |outline: &mut Vec<Point>| {
            if outline.last() != Some(&Point::new(tail_front, top)) {
                outline.push(Point::new(tail_front, top));
            }
            outline.push(Point::new(tail_front, tail_top));
        };
        for (back, front, base, height, radius) in blocks {
            if !stepped && front <= tail_front {
                step(&mut outline);
                stepped = true;
            }
            if outline.last() != Some(&Point::new(front, base)) {
                outline.push(Point::new(front, base));
            }
            // Its front, its top and its back, its ends rounded where it has
            // a radius, drawn as its part draws them: from the foot of its
            // front, which is in the outline already, to the foot of its
            // back, which follows.
            let ends = rounded_ends(back, front, base, height, radius);
            outline.extend(
                ends.iter()
                    .flat_map(|seg| seg.drawn(DRAWN_HALVINGS))
                    .skip(1),
            );
            // Where the tail begins right behind a block on the keystick, the
            // block's back face and the tail's front are one face, with no
            // corner at the keystick's top. Where it begins further back, the
            // keystick's full height runs between them; a felted tail run on
            // under a block is refused ([`crate::validate`]).
            if !stepped && back.to_bits() == tail_front.to_bits() {
                outline.push(Point::new(tail_front, tail_top));
                stepped = true;
            } else {
                outline.push(Point::new(back, base));
            }
        }
        if !stepped {
            step(&mut outline);
        }
        // And the tail: cut down to put the damper's contact at the right
        // height, with its hard felt on top where it has one.
        outline.push(Point::new(self.tail_end, tail_top));
        outline
    }

    /// Height of the wood the capstan is screwed into, meters: the
    /// keystick's full height, the capstan driven into a hole bored straight
    /// through it. Nothing is glued under it, and the validation layer
    /// refuses it on the thinned tail or across the step down to it, and a
    /// clearance cut under its head ([`crate::validate`]).
    pub fn capstan_seat(&self) -> f64 {
        self.stick_height
    }

    /// The capstan's contact point, in the key's frame: the top of its head,
    /// on its axis.
    ///
    /// Where the wippen's heel bears, and where the key weight ratio's 10 g is
    /// centered on the jig, *"on a vertical axis which passes through the
    /// capstan/heel contact point"* [V, Stanwood 2000, p. 22].
    pub fn capstan_contact(&self) -> Point {
        Point::new(
            self.capstan,
            self.capstan_seat() + self.capstan_screw.height,
        )
    }

    /// The capstan's half-profile in the key's frame, ready to be turned.
    ///
    /// Separate from the key's own outline because a screw is **not a prism**:
    /// extruding its silhouette across the width of the key makes a slab
    /// that reads, correctly, as a block of wood. It is turned about its own
    /// axis, and that is how it is drawn.
    pub fn capstan_profile(&self) -> Vec<(f64, f64)> {
        let seat = self.capstan_seat();
        self.capstan_screw
            .profile()
            .into_iter()
            .map(|(r, y)| (r, y + seat))
            .collect()
    }

    /// The area centroid of the key's drawn section, in the key's frame:
    /// its parts as the sagittal section draws them ([`Self::parts`]), the
    /// keystick, its top lowered by the clearance for the wippen rail where
    /// it has one, the covering, the key front, the glued blocks and the
    /// tail's felt, the front mortise's and the balance mortise's voids
    /// taken out, the turned capstan left out, and so are the bushing and
    /// the key button's side holes, which stand beside the sagittal plane;
    /// `None` for a section that encloses nothing.
    ///
    /// **What stands in for the height of a weighed key's center of mass**,
    /// which no scale gives (`docs/FOUNDATIONS.md` > Named simplifications): where the center of
    /// mass of a key of one material and one width across the keyboard
    /// would stand. A real key is narrower behind its head, its covering and
    /// its leads denser than its wood, so the height is an estimate, and
    /// every reading it reaches names it; it acts on the frame through the
    /// key's tilt alone ([`KeyMass`]). Nothing weighs the outline: a key's
    /// mass and where it stands along the key are its weighing's.
    pub fn section_centroid(&self) -> Option<Point> {
        let mut parts = Vec::new();
        self.parts(crate::mechanism::BodyId(0), &mut parts);
        let (mut area, mut along, mut across) = (0.0, 0.0, 0.0);
        for part in parts.iter().filter(|p| {
            matches!(p.sweep, crate::contact::Sweep::Extruded { .. }) && p.sweep.is_in_the_section()
        }) {
            // An arc, the key button's rounded ends, is read through its
            // drawn chords, as the section draws it.
            let points: Vec<Point> = part
                .outline
                .iter()
                .flat_map(|s| s.drawn(DRAWN_HALVINGS))
                .collect();
            let (a, m) = area_and_moments(&points);
            let sign = if part.role == crate::contact::Role::Cut {
                -1.0
            } else {
                1.0
            };
            area += sign * a;
            along += sign * m.x;
            across += sign * m.y;
        }
        if area.is_nan() || area <= 0.0 {
            return None;
        }
        Some(Point::new(along / area, across / area))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Consistency: **a block's rounded ends reach in as their radius
    /// says.** On a block 6 mm high and 40 mm long: a radius of 4 mm, under
    /// the height, rounds each top corner, its end straight under the arc,
    /// and reaches in by the radius; one of 12 mm, over it, runs each arc
    /// from the top down to the foot of its end and reaches in
    /// √(2·12·6 − 6²) = √108 mm. Each arc is tangent to the top where it
    /// reaches in, its center a radius under the top, and the outline runs
    /// from the foot of the front to the foot of the back.
    #[test]
    fn a_blocks_rounded_ends_reach_in_as_their_radius_says() {
        let (back, front, base, height) = (-0.020, 0.020, 0.0245, 0.006);
        let top = base + height;
        // A few roundings of lengths under a decimeter.
        let near = |a: f64, b: f64| (a - b).abs() <= 8.0 * f64::EPSILON * 0.1;
        for (radius, reach, segments) in [(0.004, 0.004, 5), (0.012, 0.001 * 108.0_f64.sqrt(), 3)] {
            assert!(near(end_reach(height, radius), reach), "{radius} m");
            let ends = rounded_ends(back, front, base, height, radius);
            assert_eq!(ends.len(), segments, "{radius} m: {ends:?}");
            assert!(near(ends[0].start().x, front) && near(ends[0].start().y, base));
            let last = ends[ends.len() - 1].end();
            assert!(near(last.x, back) && near(last.y, base));
            let arcs: Vec<(Point, f64, Point)> = ends
                .iter()
                .filter_map(|seg| match *seg {
                    Seg::Arc { c, r, .. } => Some((c, r, seg.start())),
                    Seg::Line { .. } => None,
                })
                .collect();
            assert_eq!(arcs.len(), 2);
            for ((c, r, _), end) in arcs.iter().zip([front, back]) {
                assert_eq!(r.to_bits(), radius.to_bits());
                assert!(near(c.y, top - radius));
                assert!(
                    near((end - c.x).abs(), reach),
                    "{radius} m: reaches in {}",
                    (end - c.x).abs()
                );
            }
            let flat = ends.iter().find_map(|seg| match *seg {
                Seg::Line { a, b } if a.y.to_bits() == b.y.to_bits() && a.x > b.x => Some((a, b)),
                _ => None,
            });
            let (from, to) = flat.expect("a flat top between the ends");
            assert!(near(from.y, top) && near(from.x, front - reach) && near(to.x, back + reach));
        }
    }

    /// Level 0: the shoelace sums against a shape whose answer is known.
    ///
    /// A 100 × 20 mm rectangle with its lower-left corner at the origin:
    /// area, first moments and centroid are closed forms, so a sign error or
    /// a factor of two cannot hide. Every number is a short binary fraction
    /// of the rectangle's, so the sums are exact but for a few roundings of
    /// the products, each within half an ulp of the largest term.
    #[test]
    fn the_shoelace_sums_match_a_rectangle_computed_by_hand() {
        let (l, h) = (0.100_f64, 0.020_f64);
        let rectangle = [
            Point::new(0.0, 0.0),
            Point::new(l, 0.0),
            Point::new(l, h),
            Point::new(0.0, h),
        ];
        let (area, moments) = area_and_moments(&rectangle);
        let bound = 8.0 * f64::EPSILON;
        assert!((area - l * h).abs() <= bound * l * h, "area {area}");
        assert!((moments.x - l * h * l / 2.0).abs() <= bound * l * l * h);
        assert!((moments.y - l * h * h / 2.0).abs() <= bound * l * h * h);
        let c = area_centroid(&rectangle).expect("a rectangle encloses an area");
        assert!((c.x - l / 2.0).abs() <= bound * l, "{c:?}");
        assert!((c.y - h / 2.0).abs() <= bound * h, "{c:?}");
    }

    /// Consistency: **winding is a draftsman's habit, not a modeling
    /// choice**: the key's section read clockwise has the same area and
    /// centroid, to the rounding of its sums.
    #[test]
    fn reversing_the_winding_changes_nothing() {
        let key = crate::test_fixture::key();
        let mut v = key.outline_vertices();
        let ccw = area_centroid(&v).expect("the key's section");
        let (area, _) = area_and_moments(&v);
        v.reverse();
        let cw = area_centroid(&v).expect("the key's section, the other way");
        // The rounding of a shoelace sum of `v.len()` terms, each within
        // half an ulp of the largest, the reach squared: summed the other
        // way, the key button's arc points round otherwise.
        let reach = key.lip() - key.tail_end;
        let bound = v.len() as f64 * f64::EPSILON * reach * reach;
        assert!((area - area_and_moments(&v).0).abs() <= bound);
        assert!((cw.x - ccw.x).abs() <= 1e-15 && (cw.y - ccw.y).abs() <= 1e-15);
    }

    /// The voids the section's centroid takes out, each its area and its
    /// middle: the front mortise, the balance mortise's slot and cavity,
    /// and the balance hole, all rectangles in the sagittal plane.
    /// How many points the drawn section's sums run over: every point of
    /// every drawn part the section cuts.
    fn drawn_points(key: &KeyDesign) -> usize {
        let mut parts = Vec::new();
        key.parts(crate::mechanism::BodyId(0), &mut parts);
        parts
            .iter()
            .filter(|p| {
                matches!(p.sweep, crate::contact::Sweep::Extruded { .. })
                    && p.sweep.is_in_the_section()
            })
            .map(|p| {
                p.outline
                    .iter()
                    .map(|seg| seg.drawn(DRAWN_HALVINGS).len())
                    .sum::<usize>()
            })
            .sum()
    }

    fn voids(key: &KeyDesign) -> [(f64, Point); 3] {
        let (back, front) = key.front_mortise_span();
        let mortise = key.balance_mortise;
        let (button_back, button_front) = key.key_button_span();
        let button_top = key.wood_under(button_back, button_front) + key.capsule_height;
        [
            (
                (front - back) * key.front_mortise_depth,
                Point::new(0.5 * (back + front), 0.5 * key.front_mortise_depth),
            ),
            (
                mortise.length * (button_top - mortise.floor),
                Point::new(0.0, 0.5 * (mortise.floor + button_top)),
            ),
            (
                mortise.hole_diameter * mortise.floor,
                Point::new(0.0, 0.5 * mortise.floor),
            ),
        ]
    }

    /// Consistency: **a polygon that encloses nothing has no centroid**,
    /// rather than a `NaN`: three points on a line, a single point, nothing.
    #[test]
    fn a_degenerate_outline_has_no_centroid() {
        let line = [
            Point::new(0.0, 0.0),
            Point::new(0.1, 0.0),
            Point::new(0.2, 0.0),
        ];
        assert_eq!(area_centroid(&line), None);
        assert_eq!(area_centroid(&[Point::new(0.0, 0.0)]), None);
        assert_eq!(area_centroid(&[]), None);
    }

    /// Consistency: **the drawn section's centroid is the section's, the
    /// mortises taken out.** The parts tile the key's section
    /// (`the_drawn_key_tiles_its_section`), so the centroid read off the
    /// drawn parts is the section polygon's with the voids' rectangles
    /// removed by hand: the front mortise, the balance mortise's slot and
    /// cavity, and the balance hole; the turned capstan, and the bushing
    /// and the key button's side holes beside the sagittal plane, are in
    /// neither. On every key the fixture
    /// has, its centroid stands over the key's underside, under its top, and
    /// between its tail and its lip. The bound is derived: two shoelace sums,
    /// one over the outline's points and one over every drawn part's, and
    /// the removals' few products, each rounding within half an ulp of the
    /// largest term, the reach to the third power over the area.
    #[test]
    fn the_drawn_section_is_the_sections_with_the_mortises_taken_out() {
        for key in [
            crate::test_fixture::key(),
            crate::test_fixture::key_with_every_feature(),
        ] {
            let drawn = key.section_centroid().expect("the key encloses wood");
            let outline = key.outline_vertices();
            let (area, moments) = area_and_moments(&outline);
            let (mut left, mut along, mut across) = (area, moments.x, moments.y);
            for (void, middle) in voids(&key) {
                left -= void;
                along -= void * middle.x;
                across -= void * middle.y;
            }
            let by_hand = Point::new(along / left, across / left);
            let reach = key.lip() - key.tail_end;
            let terms = (outline.len() + drawn_points(&key)) as f64;
            let bound = (terms + 20.0) * f64::EPSILON * reach * reach * reach / left;
            assert!(
                (drawn.x - by_hand.x).abs() <= bound && (drawn.y - by_hand.y).abs() <= bound,
                "drawn {drawn:?}, by hand {by_hand:?}"
            );
            assert!(drawn.y > 0.0 && drawn.y < key.stick_height + key.keytop_thickness);
            assert!(drawn.x > key.tail_end && drawn.x < key.lip());
        }
    }

    /// Consistency: **the clearance takes its wood out of the section.**
    /// The key with every feature read with its clearance and without it:
    /// the one's drawn centroid is the other's with the clearance's
    /// rectangle, its length by its depth, taken out by hand, its middle
    /// halfway along it and half its depth under the full top. The bound is
    /// derived as the section's own: the two keys' drawn parts' points and
    /// the rectangle's few products, each rounding within half an ulp of the
    /// largest term, the reach to the third power over the area.
    #[test]
    fn the_clearance_takes_its_wood_out_of_the_section() {
        let cut = crate::test_fixture::key_with_every_feature();
        let c = cut.top_clearance.expect("a clearance");
        let whole = KeyDesign {
            top_clearance: None,
            ..cut.clone()
        };
        let (area, _) = area_and_moments(&whole.outline_vertices());
        let area = voids(&whole)
            .iter()
            .fold(area, |left, (void, _)| left - void);
        let before = whole.section_centroid().expect("the key encloses wood");
        let rectangle = (c.front - c.back) * c.depth;
        let middle = Point::new(0.5 * (c.back + c.front), whole.stick_height - 0.5 * c.depth);
        let left = area - rectangle;
        let by_hand = Point::new(
            (before.x * area - middle.x * rectangle) / left,
            (before.y * area - middle.y * rectangle) / left,
        );
        let drawn = cut.section_centroid().expect("the key encloses wood");
        let reach = cut.lip() - cut.tail_end;
        let terms = (drawn_points(&whole) + drawn_points(&cut)) as f64;
        let bound = (terms + 20.0) * f64::EPSILON * reach * reach * reach / left;
        assert!(
            (drawn.x - by_hand.x).abs() <= bound && (drawn.y - by_hand.y).abs() <= bound,
            "drawn {drawn:?}, by hand {by_hand:?}"
        );
        assert!(
            drawn.y < before.y,
            "wood taken off the top lowers the centroid"
        );
    }

    /// Consistency: **the centroid is the key's wood, not its regulation**:
    /// screwing the capstan up moves its contact point and leaves the
    /// section's centroid where it was, to the bit, the capstan being no part
    /// of the section.
    #[test]
    fn screwing_the_capstan_up_leaves_the_sections_centroid() {
        let base = crate::test_fixture::key();
        let mut raised = base.clone();
        raised.capstan_screw.height += 0.004;
        assert!(raised.capstan_contact().y > base.capstan_contact().y);
        assert_eq!(raised.section_centroid(), base.section_centroid());
    }
}

#[cfg(test)]
mod capstan_tests {
    /// The capstan is **not** in the keystick's polygon, and has its own
    /// profile instead.
    ///
    /// Keeping it out of the polygon is what lets the screw be *drawn* as a
    /// turned part rather than a slab, and keeps steel out of the section
    /// whose centroid estimates the height of the key's center of mass.
    #[test]
    fn the_capstan_is_its_own_turned_body_and_not_part_of_the_keystick() {
        let d = crate::test_fixture::key();
        let stick = d.outline_vertices();
        let cap_top = d.capstan_seat() + d.capstan_screw.height;
        let r = 0.5 * d.capstan_screw.head_diameter;

        // Nothing of the keystick stands where the screw is. Checked over the
        // screw's own footprint rather than over the whole key: the capsule is
        // legitimately taller than the capstan, so "nothing is above it" is not
        // the same claim as "it is not part of the key".
        assert!(
            !stick
                .iter()
                .any(|p| (p.x - d.capstan).abs() < r && p.y > d.capstan_seat() + 1e-9),
            "the keystick polygon reaches up into the capstan's own space"
        );

        let profile = d.capstan_profile();
        assert!(profile.len() >= 5, "the capstan needs a real profile");
        assert!(
            profile.iter().any(|&(_, y)| (y - cap_top).abs() < 1e-12),
            "and its profile must reach the height the technician set"
        );
    }

    /// The head is wider than the shaft, and the top is flat.
    ///
    /// Both matter beyond looks: the flat top is what will bear on the wippen
    /// heel, and the head's diameter decides where that contact can sit.
    #[test]
    fn the_capstan_has_a_flared_head_with_a_flat_top() {
        let c = crate::test_fixture::capstan();
        let profile = c.profile();

        assert!(
            c.head_diameter > c.diameter,
            "the head must be wider than the shaft"
        );
        let top = profile.last().unwrap();
        assert!((top.0).abs() < 1e-15, "the profile must close on the axis");
        assert!(
            (top.1 - c.height).abs() < 1e-15,
            "and the top of the head is the height a technician sets"
        );
        // Two points at the same height, at the head radius and at the axis:
        // that is a flat top, not a dome.
        let shoulder = profile[profile.len() - 2];
        assert!(
            (shoulder.1 - top.1).abs() < 1e-15 && shoulder.0 > 0.0,
            "the top must be flat across the head"
        );
        // And the shaft ends in a point.
        assert!(profile[0].0.abs() < 1e-15, "the shaft must come to a tip");
    }
}

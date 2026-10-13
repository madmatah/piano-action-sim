//! Parametric part definitions, the parts they build, the mass properties
//! of the composite key, and what is known of a key's mass.
//!
//! # Two things built from one key
//!
//! [`KeyDesign`] holds the key's dimensions, and two things are built from
//! them. Its **parts** ([`key`]), one closed outline per piece of material,
//! are what is drawn, what the faults read and what the key's contacts with
//! its frame read ([`mounts::key_on_frame`]); [`FrameDesign`] builds the
//! frame's the same way ([`frame`]). Its **volume model**, the polygon
//! [`KeyDesign::outline_vertices`] integrated as a uniform prism and the
//! capstan turned from its profile, is the composite key's mass: an
//! assumption with no record, drawn by nothing, which the parts tile exactly
//! (`the_drawn_key_tiles_the_polygon_its_mass_is_integrated_over`).
//!
//! # A key's mass is weighed
//!
//! A key is weighed whole, its leads in it: its mass and its front weight,
//! which gives its static moment about the balance pin ([`key_mass`]). Until
//! it is, the volume model's estimate stands for the whole of its mass; once
//! it is, for the height of its centre of mass alone, which no scale gives.
//! The volume model leaves with the composite key (`docs/FOUNDATIONS.md`
//! §2.3).
//!
//! # The key's local frame
//!
//! Origin at the **balance rail pin**, `x` positive towards the player, `y`
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
use crate::geometry::{Point, Seg};

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

/// Mass properties of a planar region extruded across the sagittal plane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MassProperties {
    /// Total mass, kg.
    pub mass: f64,
    /// Centre of mass in the part's local frame.
    pub com: Point,
    /// Second moment of mass about the local frame's **origin**, kg·m².
    ///
    /// About the origin, not about the centre of mass, because the origin is the
    /// pivot in every frame this project defines. A caller that wants the central
    /// moment subtracts `mass * |com|²` itself, visibly.
    pub i_origin: f64,
}

impl MassProperties {
    /// The properties of nothing: what two bodies of no mass combine into.
    pub const ZERO: Self = Self {
        mass: 0.0,
        com: Point::new(0.0, 0.0),
        i_origin: 0.0,
    };

    /// Combines two bodies rigidly joined in the same frame.
    pub fn combined(self, other: Self) -> Self {
        let mass = self.mass + other.mass;
        if mass <= 0.0 {
            return Self::ZERO;
        }
        Self {
            mass,
            com: Point::new(
                (self.mass * self.com.x + other.mass * other.com.x) / mass,
                (self.mass * self.com.y + other.mass * other.com.y) / mass,
            ),
            // Second moments about a shared origin add directly. No parallel-axis
            // term is needed precisely because both are already about the origin,
            // which is why `i_origin` is defined there.
            i_origin: self.i_origin + other.i_origin,
        }
    }

    /// Removes a region from a body: `self` without `other`.
    ///
    /// Mass, first moment and second moment all subtract exactly, so a part cut
    /// out of a larger one needs no boolean geometry — only arithmetic, as when
    /// a part is cut out of the wood and replaced by the same shape in another
    /// material. Only the tests call it.
    ///
    /// `None` when the result would have non-positive mass, which means the
    /// caller is removing more than is there.
    pub fn minus(self, other: Self) -> Option<Self> {
        let mass = self.mass - other.mass;
        if mass <= 0.0 {
            return None;
        }
        Some(Self {
            mass,
            com: Point::new(
                (self.mass * self.com.x - other.mass * other.com.x) / mass,
                (self.mass * self.com.y - other.mass * other.com.y) / mass,
            ),
            i_origin: self.i_origin - other.i_origin,
        })
    }

    /// A point mass at `at`. Only the tests call it.
    pub fn point_mass(at: Point, mass: f64) -> Self {
        Self {
            mass,
            com: at,
            i_origin: mass * (at.x * at.x + at.y * at.y),
        }
    }

    /// The **static moment** about the origin, kg·m — the quantity every
    /// touchweight observable is built from.
    ///
    /// Positive when the body overhangs towards positive `x`. Stanwood's "radius
    /// weight" is this divided by a radius.
    pub fn static_moment_x(&self) -> f64 {
        self.mass * self.com.x
    }
}

/// Mass properties of a **solid of revolution**, from its half-profile.
///
/// # Why a lathe and not an extrusion
///
/// Most of this action is prismatic: a key, a rail, a punching are all a
/// section swept sideways. A screw is not. Extruding a capstan's silhouette
/// across the width of the key makes a slab several times too heavy, which
/// reads as a block of wood. Turned parts get turned.
///
/// The profile is a list of `(radius, height)` points in order along the axis,
/// which stands vertical at `x = axis_x`; consecutive points are joined by
/// straight lines, so a cone, a cove and a cylinder are all expressible.
/// Between two points the solid is a truncated cone, whose volume and moments
/// are closed-form, so this is an exact integral and not a discretisation.
///
/// Returns `None` for a degenerate profile: fewer than two points, a negative
/// radius, or no enclosed volume.
pub fn revolved_mass_properties(
    profile: &[(f64, f64)],
    axis_x: f64,
    density: f64,
) -> Option<MassProperties> {
    if profile.len() < 2 || density <= 0.0 || profile.iter().any(|&(r, _)| r < 0.0) {
        return None;
    }

    let pi = core::f64::consts::PI;
    let mut volume = 0.0;
    let mut moment_y = 0.0;
    // The planar second moment about the frame origin, `∫ (x² + y²) dV`.
    let mut i_about_origin = 0.0;

    for w in profile.windows(2) {
        let ((r0, y0), (r1, y1)) = (w[0], w[1]);
        let h = y1 - y0;
        if h.abs() < 1e-15 {
            continue;
        }
        // A truncated cone of radii r0, r1 over height h. With s = y − y0 the
        // radius is linear in s, so the volume and its first two moments in s
        // are polynomial integrals of π r(s)², done in closed form. All three
        // carry the sign of h, as the volume does.
        let v = pi * h * (r0 * r0 + r0 * r1 + r1 * r1) / 3.0;
        let s1 = pi * h * h * (r0 * r0 + 2.0 * r0 * r1 + 3.0 * r1 * r1) / 12.0; // ∫ s dV
        let s2 = pi * h * h * h * (r0 * r0 + 3.0 * r0 * r1 + 6.0 * r1 * r1) / 30.0; // ∫ s² dV
        volume += v;
        moment_y += y0 * v + s1;

        // Across the axis a point at radius ρ and azimuth φ sits at
        // x = axis_x + ρ cos φ, so ∫ x² dV = axis_x² V + ½ ∫ ρ² dV, and
        // ½ ∫ ρ² dV = π ∫ r(s)⁴ / 4 ds. Along it, ∫ y² dV = y0² V + 2 y0 ∫ s dV
        // + ∫ s² dV. Each term is exact, so a cylinder and a cone both come out
        // right, including the spread of mass along the axis.
        let (a, b, p) = (r0 * r0, r1 * r1, r0 * r1);
        let radial = pi * h * (a * a + a * p + a * b + p * b + b * b) / 20.0;
        let along = y0 * y0 * v + 2.0 * y0 * s1 + s2;
        i_about_origin += axis_x * axis_x * v + radial + along;
    }

    if volume.abs() < 1e-18 {
        return None;
    }
    let mass = density * volume.abs();
    Some(MassProperties {
        mass,
        com: Point::new(axis_x, moment_y / volume),
        i_origin: density * i_about_origin.abs(),
    })
}

/// Mass properties of a closed polygon extruded by `width`, in a material of
/// `density`.
///
/// The polygon is given as its vertices in order; it is treated as closed, so
/// the last vertex joins the first. Winding may be either way — the sign of the
/// area is normalised — because a part outline drawn clockwise is not an error,
/// it is a draughtsman's habit.
///
/// Returns `None` for a degenerate outline: fewer than three vertices, or zero
/// enclosed area. That is a real possibility, since the user edits the geometry,
/// and a silent `NaN` here would surface much later, far from its cause.
pub fn polygon_mass_properties(
    vertices: &[Point],
    width: f64,
    density: f64,
) -> Option<MassProperties> {
    if vertices.len() < 3 || width <= 0.0 || density <= 0.0 {
        return None;
    }

    let mut a2 = 0.0; // twice the signed area
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut i_xx = 0.0; // integral of y^2 dA
    let mut i_yy = 0.0; // integral of x^2 dA

    for i in 0..vertices.len() {
        let p = vertices[i];
        let q = vertices[(i + 1) % vertices.len()];
        let cross = p.x * q.y - q.x * p.y;
        a2 += cross;
        cx += (p.x + q.x) * cross;
        cy += (p.y + q.y) * cross;
        i_xx += (p.y * p.y + p.y * q.y + q.y * q.y) * cross;
        i_yy += (p.x * p.x + p.x * q.x + q.x * q.x) * cross;
    }

    let area = 0.5 * a2;
    if area.abs() < 1e-14 {
        return None;
    }

    let mass = density * width * area.abs();
    let sign = if area >= 0.0 { 1.0 } else { -1.0 };
    Some(MassProperties {
        mass,
        com: Point::new(cx / (3.0 * a2), cy / (3.0 * a2)),
        // J = Ixx + Iyy is the polar second moment of area about the origin; for
        // a prism turning about the z-axis through that origin the mass moment is
        // density * width * J.
        i_origin: density * width * sign * (i_xx + i_yy) / 12.0,
    })
}

/// The capstan screw standing on the back of the key.
///
/// Part of the key, not of the action above it: it is screwed into the
/// keystick, it moves with the key, and its brass is heavy enough to matter.
/// Modelling it as an anchor point leaves it out of the silhouette, out of the
/// mass and out of the drawing, and a technician looking at the section notices
/// immediately.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Capstan {
    /// How far the **top of the head** stands above the keystick, metres.
    /// This is what screwing it up and down changes, and it is the dimension
    /// a technician sets.
    pub height: f64,
    /// Threaded shaft diameter, metres.
    pub diameter: f64,
    /// Head diameter, metres. Wider than the shaft: it is the head's flat top
    /// that bears on the wippen heel.
    pub head_diameter: f64,
    /// Height of the cylindrical part of the head, metres.
    pub head_height: f64,
    /// Height of the cove that flares the shaft out into the head, metres.
    pub cove_height: f64,
    /// Length of the threaded shaft below the cove, metres. Part of it is
    /// buried in the key; all of it counts towards the mass.
    pub shaft_length: f64,
    /// Length of the conical tip at the bottom of the shaft, metres.
    pub tip_height: f64,
    /// Density of the capstan's material, kg/m³. Brass unless stated.
    pub density: f64,
}

impl Capstan {
    /// The half-profile to turn, as `(radius, height above the keystick)`
    /// points running from the bottom of the shaft to the centre of the flat
    /// top.
    ///
    /// Shaped from a photograph of a real capstan: a pointed, threaded shaft;
    /// a cove flaring out; a short cylindrical head; a flat top. The thread
    /// itself is not modelled — it changes the mass by about a twentieth and
    /// nothing else, and a helix is not a profile.
    pub fn profile(&self) -> Vec<(f64, f64)> {
        let (r_shaft, r_head) = (0.5 * self.diameter, 0.5 * self.head_diameter);
        let top = self.height;
        let head_bottom = top - self.head_height;
        let cove_bottom = head_bottom - self.cove_height;
        // The shaft runs down into the key; how far does not matter to the
        // section, but it does to the mass, so it is the full screw.
        let shaft_bottom = cove_bottom - self.shaft_length;
        vec![
            (0.0, shaft_bottom),
            (r_shaft, shaft_bottom + self.tip_height),
            (r_shaft, cove_bottom),
            (r_head, head_bottom),
            (r_head, top),
            (0.0, top),
        ]
    }
}

impl Default for Capstan {
    /// Proportioned from a close photograph of a capstan on a grand key.
    ///
    /// **Steel, not brass.** The owner's photograph shows a bright chromed or
    /// nickelled screw, which is common on modern grands; brass capstans exist
    /// and are 8 % heavier, so the material is a field and not a constant. The
    /// head is small — about 5 mm against a 3 mm shaft — with a cross hole
    /// through it for the regulating tool, and a short cove flaring the one
    /// into the other.
    ///
    /// Scaled from the photograph rather than measured with callipers, so
    /// still [E] — but from a real screw rather than from nothing.
    fn default() -> Self {
        Self {
            height: 0.0090,
            diameter: 0.0030,
            head_diameter: 0.0050,
            head_height: 0.0030,
            cove_height: 0.0008,
            shaft_length: 0.0130,
            tip_height: 0.0015,
            // Steel. Brass is 8500 and is what many makers fit.
            density: 7850.0,
        }
    }
}

/// Everything about one key that the choice of parts fixes.
///
/// Longitudinal dimensions are signed distances from the balance pin, positive
/// towards the player. The ones that carry the trade's own meaning are named
/// after where a technician measures: the front edge, where key height and dip
/// are read; the covering's lip ([`Self::lip`]), which the touchweight
/// measuring point is taken back from; and the capstan.
///
/// The measuring point itself is **not** here. Where a weight is centred is a
/// convention of the protocol, not a dimension of the wood, so it is set on
/// [`crate::measure::TouchweightProtocol`] and read through
/// [`crate::measure::TouchweightProtocol::measuring_point`].
#[derive(Debug, Clone, PartialEq)]
pub struct KeyDesign {
    /// The key front, metres in front of the balance pin: the outer face of
    /// the plate glued over the end grain ([`Self::front_board`]).
    ///
    /// Key height and key dip are read here. The covering's lip stands
    /// [`Self::keytop_overhang`] proud of it.
    pub front_edge: f64,
    /// Rear end of the keystick, metres (negative: behind the balance pin).
    pub tail_end: f64,
    /// Capstan axis, metres (negative).
    pub capstan: f64,
    /// Front mortise, metres back from the key's **front edge**.
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
    /// Length of the front mortise along the key, metres.
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
    /// Depth of the front mortise above the key's underside, metres.
    ///
    /// The slot the front rail pin stands in, measured from the underside, so
    /// it already includes the recess the slot begins after: 3 mm of notch and
    /// 19 mm of mortise is 22 mm of room. [V]
    ///
    /// It is a **stop**, not decoration: the key cannot descend past the point
    /// where the pin's tip reaches the top of the slot, whatever the punchings
    /// underneath would otherwise allow.
    pub front_mortise_depth: f64,
    /// Thickness of the keystick's own wood, metres.
    ///
    /// **One piece, one thickness.** Everything else on the key is either glued
    /// to this — the covering, the capsule, the block that carries the back
    /// check — or cut out of it. Reading the key as a stack of steps gets the
    /// mass about right and the reasons entirely wrong, and the reasons are
    /// what say where a step may be moved and where it may not.
    pub stick_height: f64,
    /// Height of the **capsule** above the keystick, metres.
    ///
    /// The block glued on top that the balance mortise is cut through — more
    /// wood where the pin passes, which is the whole reason it is there.
    pub capsule_height: f64,
    /// Length of the capsule along the key, metres.
    pub capsule_length: f64,
    /// Height of the block that carries the **back check**, above the wood.
    ///
    /// Glued on, and there to give the back check wire something to be driven
    /// into — which is why it sits where the hammer's tail arrives and nowhere
    /// else.
    pub back_check_block_height: f64,
    /// Length of that block along the key, metres.
    pub back_check_block_length: f64,
    /// Thickness of the key's thinned tail, metres, above the underside.
    ///
    /// The last stretch of the key is cut down so that the **damper** system
    /// can bear on it: the felt on top of it is the contact, and the wood is
    /// taken away to put that contact at the height the damper lever wants.
    pub tail_height: f64,
    /// Length of that thinned tail along the key, metres.
    pub tail_length: f64,
    /// Thickness of the felt on the tail, metres. Hard felt, not punching felt.
    pub damper_felt: f64,
    /// How far the underside rises towards the very end of the key, metres.
    pub tail_rise: f64,
    /// The length that rise is spread over, metres.
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
    /// Thickness of the keytop covering, metres.
    ///
    /// It sits **on** the wood rather than in a rebate, so the front of the
    /// key is the full stick plus this, and the step a reader sees partway
    /// along the key is the covering ending — not the wood thinning.
    pub keytop_thickness: f64,
    /// Length of the keytop covering along the key, metres.
    pub keytop_length: f64,
    /// Thickness of the **key front** glued to the wood's front face, metres.
    ///
    /// The small plate — the *fronton* — that closes the end grain. Its outer
    /// face is what [`KeyDesign::front_edge`] names, where key height and dip
    /// are read. Touchweights are referred to the covering's lip instead,
    /// [`Self::keytop_overhang`] further forward.
    pub front_board: f64,
    /// How far the covering's lip overhangs that face, metres.
    ///
    /// The lip is the "front lip of the key" every touchweight's measuring
    /// point is taken back from ([`Self::lip`]), so this is the distance
    /// between the touchweight datum and the key height and dip datum.
    pub keytop_overhang: f64,
    /// Width of the keystick across the sagittal plane, metres.
    pub width: f64,
    /// Density of the keystick wood, kg/m³.
    pub density: f64,
    /// The capstan screwed into the back of the key.
    pub capstan_screw: Capstan,
    /// Measured total mass of the bare keystick, kg, when it has been weighed.
    ///
    /// When present it scales the derived distribution rather than replacing it:
    /// the shape still decides *where* the mass is, the scale decides how much.
    /// A key is not a uniform prism — it is cut away underneath and the covering
    /// is a different material — so the derived total is the weakest part of the
    /// derivation and the one worth overriding first.
    pub measured_stick_mass: Option<f64>,
}

impl KeyDesign {
    /// The covering's lip, metres in front of the balance pin: the front edge
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

    /// Middle of the front mortise, metres in front of the balance pin.
    ///
    /// **Where the front rail pin belongs**, and therefore where the frame
    /// places it. The key states its mortise as a setback from its own front
    /// edge; this is that setback resolved against the balance pin, which is
    /// the datum everything else on the frame is measured from.
    pub fn front_mortise_x(&self) -> f64 {
        self.front_edge - self.front_mortise_from_front_edge
    }

    /// The front mortise, as the two x it runs between, metres.
    pub fn front_mortise_span(&self) -> (f64, f64) {
        let half = 0.5 * self.front_mortise_length;
        let x = self.front_mortise_x();
        (x - half, x + half)
    }

    /// The flat of the underside, as the two x it runs between, metres.
    ///
    /// **This is the bearing surface**, and it is not the whole key: the tail is
    /// chamfered over its last stretch, behind everything that touches. The
    /// drawn key declares the same underside as ranges of its parts
    /// ([`KeyRanges::underside`]), built from the same fields, and the
    /// contacts read those.
    pub fn underside_flat(&self) -> (f64, f64) {
        (self.tail_end + self.tail_rise_run, self.front_edge)
    }

    /// The volume model's polygon: the key's whole sagittal section, as an
    /// ordered list of vertices, the covering and the key front counted as wood.
    ///
    /// The composite key's mass properties integrate it, and nothing else
    /// reads it: the key is drawn from its parts ([`Self::parts`]), which tile
    /// this polygon exactly.
    pub fn outline_vertices(&self) -> Vec<Point> {
        let top = self.stick_height;
        let wood_front = self.front_edge - self.front_board;
        let lip = self.lip();
        let keytop_back = lip - self.keytop_length;
        let capsule_front = self.capsule_length * 0.5;
        let capsule_back = -self.capsule_length * 0.5;
        let tail_front = self.tail_end + self.tail_length;
        let block_front = tail_front + self.back_check_block_length;
        let rise_from = self.tail_end + self.tail_rise_run;

        vec![
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
            // The capsule: the block the balance mortise is cut through.
            Point::new(capsule_front, top),
            Point::new(capsule_front, top + self.capsule_height),
            Point::new(capsule_back, top + self.capsule_height),
            Point::new(capsule_back, top),
            // The capstan screws straight into the wood here; there is no pad
            // under it, so nothing to draw.
            //
            // The block that carries the back check.
            Point::new(block_front, top),
            Point::new(block_front, top + self.back_check_block_height),
            Point::new(tail_front, top + self.back_check_block_height),
            // And the tail: cut down to put the damper's contact at the right
            // height, with its hard felt on top.
            Point::new(tail_front, self.tail_height + self.damper_felt),
            Point::new(self.tail_end, self.tail_height + self.damper_felt),
        ]
    }

    /// Height of the keystick's top where the capstan is screwed in, metres.
    ///
    /// The wood itself: the capstan is driven into a hole bored straight
    /// through the keystick, with nothing glued under it.
    fn capstan_seat(&self) -> f64 {
        self.stick_height
    }

    /// The capstan's contact point, in the key's frame: the top of its head,
    /// on its axis.
    ///
    /// Where the wippen's heel bears, and where the key weight ratio's 10 g is
    /// centred on the jig, *"on a vertical axis which passes through the
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
    /// extruding its silhouette across the 22 mm width of the key makes a slab
    /// that is several times too heavy and reads, correctly, as a block of
    /// wood. It is its own body, turned about its own axis, and that is also
    /// how it is drawn.
    pub fn capstan_profile(&self) -> Vec<(f64, f64)> {
        let seat = self.capstan_seat();
        self.capstan_screw
            .profile()
            .into_iter()
            .map(|(r, y)| (r, y + seat))
            .collect()
    }

    /// Mass properties of the capstan, turned from its own profile.
    pub fn capstan_mass_properties(&self) -> Option<MassProperties> {
        revolved_mass_properties(
            &self.capstan_profile(),
            self.capstan,
            self.capstan_screw.density,
        )
    }

    /// Mass properties of the bare keystick, from its outline.
    ///
    /// `None` only for a degenerate design, which the user can reach by editing.
    pub fn stick_mass_properties(&self) -> Option<MassProperties> {
        // The keystick is a prism and the capstan is a turned part, so each is
        // integrated as what it is and the two are combined. Carrying the capstan
        // inside the key's own polygon and correcting for it afterwards also
        // works, but it is more code and it forecloses drawing the capstan as a
        // screw rather than a slab.
        let derived = polygon_mass_properties(&self.outline_vertices(), self.width, self.density)?
            .combined(self.capstan_mass_properties()?);
        match self.measured_stick_mass {
            None => Some(derived),
            Some(m) if derived.mass > 0.0 => {
                let k = m / derived.mass;
                Some(MassProperties {
                    mass: m,
                    com: derived.com,
                    i_origin: derived.i_origin * k,
                })
            }
            Some(_) => None,
        }
    }

    /// Mass properties of the whole key as its volume model gives them: the
    /// keystick and its capstan, [`Self::stick_mass_properties`]. No lead is
    /// modelled: a real key's leads are in its weighing ([`KeyMass`]).
    pub fn mass_properties(&self) -> Option<MassProperties> {
        self.stick_mass_properties()
    }
}

impl Default for KeyDesign {
    /// A natural key of the middle of the keyboard.
    ///
    /// The two longitudinal dimensions that set the key weight ratio: the
    /// capstan 129.5 mm behind the balance pin, close to Thorin's 128.7 and
    /// 129 mm; and the key front 258 mm in front of it, 13 mm ahead of
    /// Thorin's 245 mm key arm [V, Thorin 2013, Table 2.1]. The covering's lip
    /// stands 2 mm proud of the key front, so the measuring point, 13 mm back
    /// from the lip, is at 247 mm, and the key ratio is 0.524 — inside the
    /// published 0.50–0.60 band for a natural. Neither dimension has a record
    /// yet (`data/params/README.md`).
    ///
    /// The section is measured too: 24.5 mm of wood, a 1.5 mm covering
    /// 155 mm long over a 2 mm key front, and the capsule, back check block
    /// and thinned tail on it. [V] — `data/params/key-section.json`.
    ///
    /// What stays [E] is where the rails bear along the key, the wood's
    /// density, and the position of the capsule, which depends on whether the
    /// key is a natural or a sharp and belongs with the balance rail.
    fn default() -> Self {
        Self {
            front_edge: 0.258,
            tail_end: -0.240,
            capstan: -0.1295,
            front_mortise_from_front_edge: 0.033,
            front_mortise_length: 0.008,
            front_mortise_depth: 0.022,
            stick_height: 0.0245,
            capsule_height: 0.0060,
            capsule_length: 0.062,
            back_check_block_height: 0.0080,
            back_check_block_length: 0.027,
            tail_height: 0.0070,
            tail_length: 0.020,
            damper_felt: 0.0050,
            tail_rise: 0.0025,
            tail_rise_run: 0.077,
            keytop_thickness: 0.0015,
            front_board: 0.0020,
            keytop_overhang: 0.0020,
            keytop_length: 0.155,
            width: 0.022,
            density: 430.0,
            capstan_screw: Capstan::default(),
            measured_stick_mass: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Level 0: the polygon integrator against a shape whose answer is known.
    ///
    /// A 100 × 20 mm rectangle with its lower-left corner at the origin, 22 mm
    /// wide, in a wood of 430 kg/m³. Mass, centroid and second moment are all
    /// closed-form, so a sign error or a factor of two cannot hide.
    #[test]
    fn polygon_integrator_matches_a_rectangle_computed_by_hand() {
        let (l, h, w, rho) = (0.100_f64, 0.020_f64, 0.022_f64, 430.0_f64);
        let p = polygon_mass_properties(
            &[
                Point::new(0.0, 0.0),
                Point::new(l, 0.0),
                Point::new(l, h),
                Point::new(0.0, h),
            ],
            w,
            rho,
        )
        .expect("a rectangle is not degenerate");

        assert!((p.mass - rho * w * l * h).abs() < 1e-15, "mass {}", p.mass);
        assert!((p.com.x - l / 2.0).abs() < 1e-15);
        assert!((p.com.y - h / 2.0).abs() < 1e-15);

        // About a corner, a rectangular lamina has J = m (l² + h²) / 3.
        let expected = p.mass * (l * l + h * h) / 3.0;
        assert!(
            (p.i_origin - expected).abs() < 1e-18,
            "i_origin {} vs {expected}",
            p.i_origin
        );
    }

    /// Winding direction is a draughtsman's habit, not a modelling choice.
    #[test]
    fn reversing_the_winding_changes_nothing_physical() {
        let mut v = KeyDesign::default().outline_vertices();
        let cw = polygon_mass_properties(&v, 0.022, 430.0).unwrap();
        v.reverse();
        let ccw = polygon_mass_properties(&v, 0.022, 430.0).unwrap();

        assert!((cw.mass - ccw.mass).abs() < 1e-15);
        assert!((cw.com.x - ccw.com.x).abs() < 1e-15);
        assert!((cw.com.y - ccw.com.y).abs() < 1e-15);
        assert!(
            cw.i_origin > 0.0 && (cw.i_origin - ccw.i_origin).abs() < 1e-18,
            "a second moment of mass cannot be negative: {} / {}",
            cw.i_origin,
            ccw.i_origin
        );
    }

    /// A degenerate outline is an error, not a `NaN`.
    #[test]
    fn a_degenerate_outline_is_rejected_rather_than_producing_nan() {
        let line = [
            Point::new(0.0, 0.0),
            Point::new(0.1, 0.0),
            Point::new(0.2, 0.0),
        ];
        assert!(polygon_mass_properties(&line, 0.022, 430.0).is_none());
        assert!(polygon_mass_properties(&[Point::new(0.0, 0.0)], 0.022, 430.0).is_none());
        assert!(
            polygon_mass_properties(&KeyDesign::default().outline_vertices(), 0.0, 430.0).is_none(),
            "zero width is degenerate"
        );
    }

    /// A measured mass rescales the distribution without detaching from the shape.
    #[test]
    fn a_measured_mass_overrides_the_total_but_not_the_geometry() {
        let mut d = KeyDesign::default();
        let derived = d.stick_mass_properties().unwrap();

        d.measured_stick_mass = Some(derived.mass * 1.3);
        let scaled = d.stick_mass_properties().unwrap();

        assert!((scaled.mass - derived.mass * 1.3).abs() < 1e-15);
        assert!(
            (scaled.com.x - derived.com.x).abs() < 1e-15,
            "weighing a key must not move its centre of gravity"
        );
        assert!((scaled.i_origin - derived.i_origin * 1.3).abs() < 1e-18);
    }
}

#[cfg(test)]
mod capstan_tests {
    use super::*;

    /// The capstan is **not** in the keystick's polygon, and has its own
    /// profile instead.
    ///
    /// Keeping it out of the polygon needs no post-hoc correction to stop the
    /// integrator weighing steel as wood, and it is what lets the screw be
    /// *drawn* as a turned part rather than a slab.
    #[test]
    fn the_capstan_is_its_own_turned_body_and_not_part_of_the_keystick() {
        let d = KeyDesign::default();
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

    /// It is brass and **turned**, not a slab of wood the width of the key.
    ///
    /// The silhouette extruded across the 22 mm of the key would be a block
    /// several times too heavy, sitting 130 mm behind the balance point —
    /// which is exactly where an error in mass moves the front weight most.
    /// Turning it from its own profile is both the right mass and the right
    /// picture.
    #[test]
    fn the_capstan_is_turned_from_its_profile_and_not_extruded() {
        let d = KeyDesign::default();
        let turned = d.capstan_mass_properties().unwrap();

        // The same silhouette extruded across the key, for comparison.
        let c = d.capstan_screw;
        let (r, r_head) = (0.5 * c.diameter, 0.5 * c.head_diameter);
        let slab_area = 2.0 * r * (c.shaft_length + c.tip_height + c.cove_height)
            + 2.0 * r_head * c.head_height;
        let slab_mass = c.density * d.width * slab_area;

        assert!(
            turned.mass < 0.5 * slab_mass,
            "a turned screw must weigh far less than its extruded silhouette: \
             {} g against {} g",
            turned.mass * 1000.0,
            slab_mass * 1000.0
        );
        // A steel capstan of these proportions is about a gram.
        assert!(
            (0.0006..0.0020).contains(&turned.mass),
            "the capstan should weigh about a gram, got {} g",
            turned.mass * 1000.0
        );
        assert!(
            (turned.com.x - d.capstan).abs() < 1e-15,
            "and must sit on its own axis"
        );
    }

    /// The head is wider than the shaft, and the top is flat.
    ///
    /// Both matter beyond looks: the flat top is what will bear on the wippen
    /// heel, and the head's diameter decides where that contact can sit.
    #[test]
    fn the_capstan_has_a_flared_head_with_a_flat_top() {
        let c = Capstan::default();
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

    /// The lathe integrator against a shape whose answer is known.
    ///
    /// A plain cylinder: two points at the same radius. Volume, centroid and
    /// the planar second moment are all closed-form, so a factor of two or a
    /// dropped pi cannot hide.
    ///
    /// The moment is about a horizontal line through the origin, across the
    /// keyboard. Standing on that origin, a cylinder of radius r and height h
    /// gives `m (r²/4 + h²/3)`: the first term from the mass spread across the
    /// axis, the second from its spread along it, which a disc taken at the
    /// centroid height would miss. Moved to `axis_x`, the parallel-axis term
    /// `m axis_x²` adds on.
    #[test]
    fn the_lathe_integrator_matches_a_cylinder_computed_by_hand() {
        let (r, h, rho) = (0.003_f64, 0.020_f64, 8500.0_f64);
        let p = revolved_mass_properties(&[(r, 0.0), (r, h)], 0.0, rho)
            .expect("a cylinder is not degenerate");

        let expected = rho * core::f64::consts::PI * r * r * h;
        assert!(
            (p.mass - expected).abs() < 1e-12,
            "mass {} against {expected}",
            p.mass
        );
        assert!((p.com.y - h / 2.0).abs() < 1e-12, "com.y {}", p.com.y);
        assert!(p.com.x.abs() < 1e-15);

        let on_the_origin = p.mass * (r * r / 4.0 + h * h / 3.0);
        assert!(
            (p.i_origin - on_the_origin).abs() < 1e-12 * on_the_origin,
            "i_origin {} against {on_the_origin}",
            p.i_origin
        );

        let axis_x = -0.1295;
        let moved = revolved_mass_properties(&[(r, 0.0), (r, h)], axis_x, rho).unwrap();
        let expected = p.mass * (axis_x * axis_x + r * r / 4.0 + h * h / 3.0);
        assert!(
            (moved.i_origin - expected).abs() < 1e-12 * expected,
            "i_origin {} against {expected} with the axis at {axis_x} m",
            moved.i_origin
        );
    }

    /// A cone is a third of its cylinder, which is the oldest check there is.
    #[test]
    fn a_cone_is_a_third_of_its_cylinder() {
        let (r, h) = (0.003_f64, 0.020_f64);
        let cone = revolved_mass_properties(&[(0.0, 0.0), (r, h)], 0.0, 8500.0).unwrap();
        let cylinder = revolved_mass_properties(&[(r, 0.0), (r, h)], 0.0, 8500.0).unwrap();

        assert!(
            (cone.mass / cylinder.mass - 1.0 / 3.0).abs() < 1e-12,
            "a cone came out at {} of its cylinder",
            cone.mass / cylinder.mass
        );
        // And its centroid is a quarter of the way up from the base.
        assert!(
            (cone.com.y - 0.75 * h).abs() < 1e-12,
            "com.y {}",
            cone.com.y
        );
        // Its second moment, apex on the origin: `3 m r² / 20` from the spread
        // across the axis, and `3 m h² / 5` from the spread along it.
        let expected = cone.mass * (3.0 * r * r / 20.0 + 3.0 * h * h / 5.0);
        assert!(
            (cone.i_origin - expected).abs() < 1e-12 * expected,
            "i_origin {} against {expected}",
            cone.i_origin
        );
    }

    /// A degenerate profile is an error, not a `NaN`.
    #[test]
    fn a_degenerate_profile_is_rejected() {
        assert!(revolved_mass_properties(&[(0.003, 0.0)], 0.0, 8500.0).is_none());
        assert!(revolved_mass_properties(&[(-0.001, 0.0), (0.003, 0.01)], 0.0, 8500.0).is_none());
        assert!(
            revolved_mass_properties(&[(0.003, 0.0), (0.003, 0.0)], 0.0, 8500.0).is_none(),
            "zero height encloses nothing"
        );
    }

    /// **Screwing the capstan up moves it; it does not make it heavier.**
    ///
    /// A model that conflates the capstan's *height above the keystick*, which is
    /// the regulation, with its *length*, which is the part, makes a raised
    /// capstan weigh more — and passes a test asserting exactly that. They are
    /// different numbers: turning the screw moves the same piece of steel up and
    /// down.
    ///
    /// What follows from that is worth stating, because it is not obvious:
    /// raising a capstan does **not** change the key's front weight at all.
    /// Front weight is a moment about a horizontal distance, the capstan's
    /// longitudinal position has not moved, and its mass has not changed. It
    /// raises the centre of gravity and nothing else.
    #[test]
    fn screwing_the_capstan_up_moves_it_without_changing_the_balance() {
        let base = KeyDesign::default();
        let mut raised = base.clone();
        raised.capstan_screw.height += 0.004;

        let m0 = base.mass_properties().unwrap();
        let m1 = raised.mass_properties().unwrap();

        assert!(
            (m1.mass - m0.mass).abs() < 1e-15,
            "the same screw cannot weigh two different amounts: {} vs {}",
            m0.mass,
            m1.mass
        );
        assert!(
            m1.com.y > m0.com.y,
            "but its centre of gravity must rise: {} -> {}",
            m0.com.y,
            m1.com.y
        );
        assert!(
            (m1.static_moment_x() - m0.static_moment_x()).abs() < 1e-15,
            "and the front weight must not move, because nothing moved \
             horizontally"
        );
    }

    /// Removing a region is exact arithmetic, not an approximation.
    #[test]
    fn minus_is_the_inverse_of_combined() {
        let a = MassProperties {
            mass: 0.080,
            com: Point::new(0.030, 0.010),
            i_origin: 1.2e-4,
        };
        let b = MassProperties::point_mass(Point::new(-0.100, 0.015), 0.012);
        let round_trip = a.combined(b).minus(b).unwrap();

        assert!((round_trip.mass - a.mass).abs() < 1e-15);
        assert!((round_trip.com.x - a.com.x).abs() < 1e-15);
        assert!((round_trip.com.y - a.com.y).abs() < 1e-15);
        assert!((round_trip.i_origin - a.i_origin).abs() < 1e-18);
    }

    /// Removing more than is there is an error, not a negative body.
    #[test]
    fn removing_more_than_is_there_is_refused() {
        let a = MassProperties::point_mass(Point::new(0.0, 0.0), 0.010);
        let b = MassProperties::point_mass(Point::new(0.0, 0.0), 0.020);
        assert!(a.minus(b).is_none());
    }
}

#[cfg(test)]
mod profile_tests {
    use super::*;

    /// Shaping the key must not break its weighing.
    ///
    /// However the profile is shaped, the key must weigh what a key weighs and
    /// balance where a key balances. This is the guard that a prettier picture
    /// does not quietly become a different instrument.
    #[test]
    fn a_shaped_key_still_weighs_like_a_key() {
        let p = KeyDesign::default().mass_properties().unwrap();
        assert!(
            (0.045..0.130).contains(&p.mass),
            "a grand key weighs 45 to 130 g, this one {} g",
            p.mass * 1000.0
        );
        assert!(
            p.com.x.abs() < 0.080,
            "and balances near its pin, not {} m away",
            p.com.x
        );
    }
}

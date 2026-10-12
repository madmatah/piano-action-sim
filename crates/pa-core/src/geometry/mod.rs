//! Parametric part definitions and exact planar contact kinematics.
//!
//! # The outline, and what reads it
//!
//! If the contact primitives live in a list *parallel* to the drawn outline,
//! the two drift apart silently and the displayed dimensions stop describing
//! what is measured. So a [`Part`] owns exactly one `outline`, and a
//! [`Surface`] is a *range into that outline*, never a copy of it.
//!
//! That is what the types say; the code that touches does not go through them.
//! No contact reads a [`Surface`]. The key's stops read
//! [`crate::parts::KeyDesign::underside_corners`] and
//! [`crate::parts::KeyDesign::underside_segments`], which are built from the
//! same fields as the outline [`crate::parts::KeyDesign::to_part`] carries. The
//! two agree because they share those fields, and a test on the default key
//! checks it; nothing structural prevents a drift.

pub mod kinematics;

pub use kinematics::{circle_vs_circle, circle_vs_line, Frame, Rot};

/// A point in a part's local frame, in metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// Horizontal coordinate, metres. Positive towards the player.
    pub x: f64,
    /// Vertical coordinate, metres. Positive upwards.
    pub y: f64,
}

impl Point {
    /// Constructs a point.
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// One segment of a part outline.
///
/// These two primitives are enough for the whole action: Hirschkorn and Thorin
/// independently converged on circle-against-line, circle-against-circle, and a
/// hybrid profile made of an arc joined to two tangent lines. The hybrid profile
/// is expressed here as consecutive `Arc` and `Line` segments; because the lines
/// are tangent to the arc, the contact normal is C1 across the junction, which
/// is exactly why the jack nose can slide on the knuckle without the
/// transmitted torque jumping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Seg {
    /// Straight segment from `a` to `b`.
    Line {
        /// Start point, local frame.
        a: Point,
        /// End point, local frame.
        b: Point,
    },
    /// Circular arc, swept counter-clockwise from `th0` to `th1`.
    Arc {
        /// Centre, local frame.
        c: Point,
        /// Radius, metres.
        r: f64,
        /// Start angle, radians.
        th0: f64,
        /// End angle, radians.
        th1: f64,
    },
}

/// A compliant lining glued onto a surface: felt, leather, buckskin, cloth, skin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lining {
    /// Index of the lining's material in a material table. No such table
    /// exists, so nothing reads this.
    pub material: u16,
    /// Nominal thickness, metres.
    pub thickness: f64,
    /// Where `gap == 0` actually is, relative to the nominal lining surface, in metres.
    ///
    /// This field is not a refinement, it is a necessity. Hirschkorn states the
    /// problem plainly:
    /// *"the surface of the felt is not clearly defined."* The datum is therefore a
    /// **calibrated, persisted** quantity, not a derived one. Without it, a let-off
    /// regulated to 1.5 mm would be drawn with the jack buried 1 mm into the
    /// knuckle, since the published load curves reach ~40 N at 1 mm of compression.
    pub datum_offset: f64,
}

/// A named, contactable stretch of a part outline.
#[derive(Debug, Clone)]
pub struct Surface {
    /// Stable identifier.
    pub id: &'static str,
    /// Range of [`Seg`] indices in the owning part's outline.
    pub segs: core::ops::Range<usize>,
    /// Compliant lining, if any. `None` means bare wood or metal.
    pub lining: Option<Lining>,
}

/// How a part's inertia is obtained.
///
/// This has to be an explicit choice. If masses and centres of gravity were
/// stored independently of the outline, then lengthening a hammer shank would
/// leave `I_h` untouched and the sensitivity figures — which the project calls
/// its real product — would be wrong in a plausible, hard-to-notice way.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Inertia {
    /// Derived from the outline by extrusion. Changing a dimension changes the inertia.
    Derived {
        /// Material density, kg/m³.
        density: f64,
        /// Extrusion width across the sagittal plane, metres.
        width: f64,
    },
    /// Measured on the real part. Does not follow the outline.
    Measured {
        /// Mass, kg.
        mass: f64,
        /// Centre of mass in the local frame.
        com: Point,
        /// Moment of inertia about the part's own pivot, kg·m².
        i_pivot: f64,
    },
}

/// A named point on a part that is not a contact: a pin hole, a lead position,
/// a measuring point, a glue witness.
#[derive(Debug, Clone)]
pub struct Anchor {
    /// Stable identifier.
    pub id: &'static str,
    /// Position in the local frame.
    pub at: Point,
}

/// One mechanical part, defined parametrically.
#[derive(Debug, Clone)]
pub struct Part {
    /// Stable identifier, English `snake_case`, matching the project glossary.
    pub id: &'static str,
    /// Pivot position in the parent frame, and rest angle in radians.
    pub pivot: Point,
    /// Angle at rest, radians.
    pub rest_angle: f64,
    /// The part's shape. Its [`Surface`]s are ranges into it, never copies.
    pub outline: Vec<Seg>,
    /// Contactable stretches of `outline`.
    pub surfaces: Vec<Surface>,
    /// How inertia is obtained.
    pub inertia: Inertia,
    /// Named non-contact points.
    pub anchors: Vec<Anchor>,
}

/// The effective bearing of a key.
///
/// A key does not pivot on its balance rail pin: it pivots on the *bearing
/// surface of the punching*, and where that bearing sits is adjustable. Stanwood
/// documents cutting the front half of the balance rail punching away to shift
/// the effective bearing point behind the pin, which drops the action ratio by
/// 0.3 to 0.4 — a deliberate, reversible regulation action. Modelling the pivot
/// as the pin would make that whole technique inexpressible.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyBearing {
    /// Offset of the effective bearing point from the balance rail pin, metres.
    /// Positive towards the back of the key.
    pub offset_from_pin: f64,
    /// Punching stack height, metres. This is what key levelling adjusts.
    pub stack_height: f64,
}

/// Result of a planar contact query between two surfaces.
///
/// `gap` is positive when separated. `d_gap_d_theta` is the instantaneous lever
/// arm on each body — and it is the quantity the whole project exists to compute
/// exactly, because the variation of the lever arm through the stroke *is* the
/// touch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContactQuery {
    /// Signed gap, metres. Positive = separated, negative = interpenetrating.
    pub gap: f64,
    /// Unit normal, pointing from body B towards body A.
    pub normal: Point,
    /// Contact point in world coordinates. It migrates along the surface as the
    /// bodies rotate; that migration is the physical origin of the varying ratio.
    pub point: Point,
    /// Partial derivative of `gap` with respect to body A's angle, metres/radian.
    pub d_gap_d_theta_a: f64,
    /// Partial derivative of `gap` with respect to body B's angle, metres/radian.
    pub d_gap_d_theta_b: f64,
    /// Relative tangential velocity at the contact point, m/s.
    pub v_tangential: f64,
}

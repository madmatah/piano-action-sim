//! The key frame: the three rails, their pins, and the punchings that sit on
//! them.
//!
//! # Why the frame is a part and not a set of constants
//!
//! Two of the settings a technician spends the most time on live here and
//! nowhere else. **Key height** is the balance rail punching stack. **Key dip**
//! is the gap left above the front rail punching. Both are *stacks of paper and
//! felt washers*, adjusted a few hundredths of a millimetre at a time, and both
//! have to be manipulable for the bench to teach anything. A frame expressed as
//! fixed numbers would make levelling and dipping inexpressible.
//!
//! # Frame
//!
//! Shared with [`super::KeyDesign`]: origin at the balance rail pin, `x` towards
//! the player, `y` up. The keybed surface is the vertical datum, `y = 0`.

/// A stack of punchings on a rail: paper shims plus the felt or leather washer.
///
/// Kept as two numbers rather than one total because they are two different
/// operations on the bench — paper is how you adjust, felt is what you fit — and
/// because the felt is compliant while the paper is not. The geometry treats
/// the whole stack as rigid and reads only the total; the split is what a
/// model of the felt's compliance will need.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PunchingStack {
    /// Total thickness of the paper shims, metres.
    pub paper: f64,
    /// Thickness of the felt or leather washer, metres.
    pub felt: f64,
}

impl PunchingStack {
    /// A stack of paper and felt.
    pub const fn new(paper: f64, felt: f64) -> Self {
        Self { paper, felt }
    }

    /// Total uncompressed height, metres.
    pub fn height(&self) -> f64 {
        self.paper + self.felt
    }
}

/// The key frame and everything mounted on it.
///
/// Rail heights are the top of the bare rail above the keybed; the punching
/// stack sits on top of that.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameDesign {
    /// Top of the balance rail above the keybed, metres.
    pub balance_rail_height: f64,
    /// Punchings under the key at the balance pin.
    pub balance_punching: PunchingStack,
    /// Top of the front rail above the keybed, metres.
    pub front_rail_height: f64,
    /// Punchings on the front rail. Their height sets the **key dip**.
    pub front_punching: PunchingStack,
    /// Top of the back rail above the keybed, metres.
    pub back_rail_height: f64,
    /// Cloth on the back rail.
    pub back_rail_cloth: f64,
    /// Where the front rail stands, metres in front of the balance pin,
    /// measured to the **natural** row of pins.
    ///
    /// A frame coordinate and not a key one, although it is placed from the
    /// key: the pin runs in the key's front mortise, so
    /// [`Self::placed_under`] puts it there. What keeps it here is that a key
    /// changed afterwards must **not** drag the frame along behind it. Lengthen
    /// a key on a real keyboard and the rail stays bolted where it was; the
    /// pin leaves the mortise and the keyboard is wrong until somebody moves
    /// the rail. [`super::KeyAssembly::faults`] says so.
    pub front_rail: f64,
    /// Where the back rail stands, metres behind the balance pin (negative).
    ///
    /// **Bounded by the key's chamfer, and bounded the other way round from
    /// how it first looks.** The chamfer under the tail is cut so the key beds
    /// *flat* on this cloth, so the cloth has to lie **within** the chamfer —
    /// behind where it begins, and far enough behind that the whole width of
    /// the rail is under it. Cloth in front of the chamfer could only ever be
    /// met by the kink, which is the thing the chamfer exists to prevent.
    pub back_rail: f64,
    /// The three rails as parts: their sections, pins and punching sizes.
    ///
    /// Here rather than beside the design because a section is not only drawn.
    /// A pin's height decides how far the key may descend before it bottoms in
    /// its mortise, which is a **stop** — so the statics have to be able to see
    /// it, and a frame that held its regulation in one place and its parts in
    /// another would have the contact set reading half of itself.
    pub sections: FrameSections,
}

impl FrameDesign {
    /// Height of the key's underside at the balance pin, at rest, metres.
    ///
    /// This is what key levelling moves, and it is the only one of the three
    /// bearings that carries the key at rest on a bench with no action fitted.
    pub fn balance_bearing_height(&self) -> f64 {
        self.balance_rail_height + self.balance_punching.height()
    }

    /// Height of the top of the front rail punching, metres.
    pub fn front_bearing_height(&self) -> f64 {
        self.front_rail_height + self.front_punching.height()
    }

    /// Height of the top of the back rail cloth, metres.
    pub fn back_bearing_height(&self) -> f64 {
        self.back_rail_height + self.back_rail_cloth
    }

    /// This frame with its rails placed under a key, by the relations a maker
    /// builds to.
    ///
    /// # Why this is a rule and not a definition
    ///
    /// Two longitudinal positions and two relations:
    ///
    /// - **The front rail stands under the key's front mortise.** The pin runs
    ///   in that slot, so there is nowhere else for it to be.
    /// - **The back rail is set in from the end of the key**, far enough that
    ///   the tail overhangs it and the whole of its cloth falls within the
    ///   chamfer cut to bed on it. [`BACK_RAIL_FROM_THE_TAIL`].
    ///
    /// Running this is how a frame is **built**, and it is the right default:
    /// nobody should have to type a coordinate to get a keyboard that goes
    /// together. What it is not is how a frame is **maintained**. Change a key
    /// afterwards and the frame stays where it is — rails are bolted to a
    /// keybed and do not follow — so the mismatch appears, and
    /// [`super::KeyAssembly::faults`] names it. That is the project's standing
    /// rule that upstream settings drift rather than cascade, applied to
    /// geometry: a frame that silently re-placed itself would hide exactly the
    /// lesson a keyboard designer is here to learn.
    ///
    /// So a measured frame keeps its own numbers and is judged against the key
    /// it is meant to carry, which is a different job from building one and a
    /// real one.
    pub fn placed_under(mut self, key: &super::KeyDesign) -> Self {
        self.front_rail = key.front_mortise_x();
        self.back_rail = key.tail_end + BACK_RAIL_FROM_THE_TAIL;
        self
    }
}

/// How far in from the end of the key the back rail is set, metres.
///
/// The tail has to overhang the rail — the key is lifted off the frame from
/// behind, and the damper lift works at the very end of the tail — and the
/// cloth has to sit wholly within the 77 mm of chamfer that is cut to bed on
/// it. 40 mm satisfies both with room either side, and puts a 24 mm rail with
/// 26 mm of chamfer in front of it and 28 mm of tail behind. [E] — waiting on
/// a measurement of a real frame.
pub const BACK_RAIL_FROM_THE_TAIL: f64 = 0.040;

impl Default for FrameDesign {
    /// A plausible frame for a grand. The heights themselves are **not
    /// measured** — [E] — but they are no longer free, because trade practice
    /// fixes the relation between them.
    ///
    /// **A correctly set key is level at the bottom of its travel**, and
    /// slightly nose-up when released. That is a statement about the *back
    /// rail*, not the front one: the key's rest attitude is set by the balance
    /// bearing and the back rail cloth together, and the front punching only
    /// catches the key where that attitude brings it. The condition reduces to
    ///
    /// ```text
    /// balance bearing − back bearing = dip × (back rail arm ÷ key front arm)
    /// ```
    ///
    /// — and the arm in it is where the key **really bears**, which is the back
    /// edge of the cloth and not the middle of the rail: 212 mm, not 200. So
    /// 10.2 × 212/258 = 8.4 mm, less the 1.6 mm the chamfer has risen by where
    /// the cloth meets it. There is also an equivalent form a test can read off
    /// the finished frame, and it is the one to trust: **the front punching
    /// comes up to exactly the height of the balance punching**.
    /// `the_default_frame_puts_the_key_level_at_the_bottom_of_its_travel` holds it.
    ///
    /// Within that, the three are **chosen for consistency, not invented
    /// freely**: each rail is placed so that the published targets are met
    /// with a positive stack of real paper on it, and
    /// [`super::Regulation::solve`] refuses a design where they cannot be
    /// rather than clamping. The targets are published: key height 65.88 mm
    /// Steinway New York, and key dip 10.16 mm Steinway NY / 9.9–10.6 Hamburg
    /// / 10.1–10.3 Kawai. [V]
    ///
    /// The **felt washers are deliberately thin**, 1.5 mm and 2.0 mm, so that
    /// regulating to those targets leaves positive paper on both rails. A felt
    /// thicker than the whole stack the regulation calls for is not a rounding
    /// problem, it is a wrong washer — and the solver refuses rather than
    /// returning a negative shim.
    fn default() -> Self {
        Self {
            balance_rail_height: 0.0266,
            balance_punching: PunchingStack::new(0.0005, 0.0015),
            front_rail_height: 0.0259,
            front_punching: PunchingStack::new(0.0005, 0.0020),
            back_rail_height: 0.0190,
            back_rail_cloth: 0.0030,
            front_rail: 0.225,
            back_rail: -0.200,
            sections: FrameSections::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stacking is addition, and the split into paper and felt does not change
    /// the geometry at rest.
    #[test]
    fn a_punching_stack_is_the_sum_of_what_is_on_it() {
        let a = PunchingStack::new(0.0005, 0.0030);
        let b = PunchingStack::new(0.0020, 0.0015);
        assert!((a.height() - 0.0035).abs() < 1e-15);
        assert!(
            (a.height() - b.height()).abs() < 1e-15,
            "same total, different split: the rest geometry must not care"
        );
    }

    /// Adding paper under the balance punching raises the key, and moves nothing
    /// else.
    ///
    /// This is the trade operation "key levelling", and it is the simplest
    /// possible non-confusion test: the lever that lifts the key must not be the
    /// lever that sets the dip.
    #[test]
    fn levelling_the_key_does_not_change_the_front_or_back_bearings() {
        let base = FrameDesign::default();
        let mut raised = base.clone();
        raised.balance_punching.paper += 0.0004;

        assert!(
            (raised.balance_bearing_height() - base.balance_bearing_height() - 0.0004).abs()
                < 1e-15,
            "a 0.4 mm shim must raise the bearing by 0.4 mm"
        );
        assert!(
            (raised.front_bearing_height() - base.front_bearing_height()).abs() < 1e-15,
            "levelling must not touch the front rail"
        );
        assert!(
            (raised.back_bearing_height() - base.back_bearing_height()).abs() < 1e-15,
            "levelling must not touch the back rail"
        );
    }
}

/// Which row of pins a key stands on.
///
/// Each rail carries **two rows**, one for the naturals and one for the sharps,
/// offset along the key because a sharp balances and bottoms at a different
/// place. A single-key bench shows one row — the one its key uses — because the
/// other row's pins stand between the naturals, at other positions across the
/// keyboard, and drawing one beside a natural would say something false.
///
/// The distinction is here rather than deferred because it is not only a
/// drawing matter: sharps have their own key ratio, their own balance weight
/// and their own dip, so a design that cannot name a sharp cannot graduate a
/// keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyKind {
    /// A white key, on the front row.
    Natural,
    /// A black key, on the row set back from it.
    Sharp,
}

/// The cross-section of a guide pin.
///
/// The two are not interchangeable, and the difference is functional. A
/// **balance pin is round**: the key turns on it, and a round pin lets the
/// mortise swing without binding. A **front rail pin is oblong** — flattened,
/// its wide faces presented to the bushing cloth on either side of the key's
/// front mortise. That is what holds the key straight: the wide face gives the
/// cloth a flat to bear on, while the key is free to rise and fall along it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PinShape {
    /// Turned round.
    Round {
        /// Diameter, metres.
        diameter: f64,
    },
    /// Flattened, with its wide faces across the keyboard. The sagittal
    /// section shows the length.
    ///
    /// **Only the part that shows is oblong.** The shank driven into the rail
    /// is turned round, like any pin, and swells to the flattened head at a
    /// shoulder sitting about at the rail's surface. This describes the head,
    /// which is the whole of what the key and the punchings ever touch — the
    /// shank is buried in wood and is not drawn.
    Oblong {
        /// Extent along the key, metres.
        length: f64,
        /// Extent across the keyboard, metres.
        thickness: f64,
    },
}

impl PinShape {
    /// Extent along the key, metres — what a sagittal section shows.
    pub fn length(&self) -> f64 {
        match *self {
            PinShape::Round { diameter } => diameter,
            PinShape::Oblong { length, .. } => length,
        }
    }

    /// Extent across the keyboard, metres.
    pub fn thickness(&self) -> f64 {
        match *self {
            PinShape::Round { diameter } => diameter,
            PinShape::Oblong { thickness, .. } => thickness,
        }
    }
}

/// The raised part of a rail, above its rectangular body.
///
/// The balance rail is **one piece**: a block with a trapezoid on top, narrower
/// than the block and with sloping sides. It matters to more than the picture —
/// the crown is what the key's balance mortise straddles, so its width is what
/// limits how far a key can be moved sideways before it fouls.
///
/// Not every maker builds it this way. Steinway rounds the bearing instead, so
/// the key rolls on an arc rather than sitting on a flat; that is a different
/// profile and a different contact, and it is not modelled here yet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Crown {
    /// Half-length of the flat on top, metres. Smaller than the rail's own.
    pub half_length: f64,
    /// Height of the trapezoid above the shoulder, metres.
    pub height: f64,
}

/// What one rail is, as a part: its section, its pin and its punchings.
///
/// Separate from [`FrameDesign`] because these are what the *parts* are, while
/// the design holds what the *regulation* sets. A technician never changes a
/// rail's width; he changes what is stacked on it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RailSection {
    /// Half-length of the rail along the key, metres.
    pub half_length: f64,
    /// The trapezoid on top, if the rail has one.
    pub crown: Option<Crown>,
    /// The pin standing in it, if any.
    pub pin: Option<PinShape>,
    /// How far the pin stands above the top of the **bare rail**, metres.
    ///
    /// Measured from the wood, not from the punchings, because the wood is
    /// what a depth gauge reaches and because the punching stack changes with
    /// every regulation — a pin described relative to it would appear to grow
    /// when the key is levelled. How far the pin reaches into the key's
    /// mortise is this, less whatever is stacked underneath.
    ///
    /// **The mortise bounds it, and the bound is read at the bottom of the
    /// key's travel, not at rest.** At rest the key's front hangs clear of its
    /// punching — that gap *is* the key dip — so the pin is at its shallowest
    /// in the mortise. Pressing the key down drives the mortise onto the pin.
    /// The constraint is therefore
    ///
    /// ```text
    /// pin_height ≤ punching stack + mortise step + mortise depth
    /// ```
    ///
    /// with every term measured from the rail's wood upward. On the measured
    /// frame that reads 4 + 3 + 19 = 26 mm against a pin standing 26 mm proud:
    /// the tip touches the top of the mortise exactly at the bottom of the
    /// key's travel, which is too fine to build. Driving the pin 2 mm past its
    /// shoulder gives 24 mm proud and 2 mm of headroom, and that is the value
    /// below.
    pub pin_height: f64,
    /// Outer radius of the punching washers, metres.
    pub punching_radius: f64,
    /// Radius of the hole through the punchings, metres.
    ///
    /// Measured rather than derived from the pin, because a punching is pierced
    /// with a **round** die: the hole is round even where the pin is not, and
    /// its diameter is a stock size and not a clearance anybody chose. On the
    /// measured frame the front rail's 5 mm hole is *exactly* its oblong pin's
    /// long axis — the circle touches the pin at its two ends, which is the
    /// design and not a coincidence.
    pub punching_hole_radius: f64,
    /// Distance along the key from the natural row to the sharp row, metres.
    /// Positive is away from the player.
    pub sharp_row_offset: f64,
}

impl RailSection {
    /// Where this rail's pin stands for a key of this kind, metres.
    pub fn pin_x(&self, natural_x: f64, kind: KeyKind) -> f64 {
        match kind {
            KeyKind::Natural => natural_x,
            KeyKind::Sharp => natural_x - self.sharp_row_offset,
        }
    }

    /// The middle of the rail along the key, metres.
    ///
    /// A rail is centred **between** its two rows, not on either of them, so
    /// the front rail reaches back under the key rather than sitting
    /// symmetrically beneath the natural's pin. Where a rail serves one row —
    /// the back rail, whose cloth every key tail meets at the same place — this
    /// is the bearing itself.
    pub fn centre_x(&self, natural_x: f64) -> f64 {
        natural_x - 0.5 * self.sharp_row_offset
    }

    /// The smallest half-length that still carries both rows, metres.
    ///
    /// One rail serves both, so its width along the key is not free: from the
    /// midpoint between the rows it has to reach half the row offset plus a
    /// punching's radius, or a punching hangs off the wood. On the measured
    /// front rail that is 24.5 + 11 = 35.5 mm, which puts the rail at 71 mm
    /// along the key. It is the constraint that matters most in this struct,
    /// because it is invisible on a single-key bench: only one row is drawn, so
    /// a rail too narrow to carry the other looks perfectly correct.
    pub fn minimum_half_length(&self) -> f64 {
        0.5 * self.sharp_row_offset + self.punching_radius
    }
}

/// The three rails, as parts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameSections {
    /// The balance rail: crowned, round pin.
    pub balance: RailSection,
    /// The front rail: a plain block, oblong pin.
    pub front: RailSection,
    /// The back rail: a plain block, no pin, cloth along its whole length.
    pub back: RailSection,
}

impl Default for FrameSections {
    /// The sections of a grand key frame.
    ///
    /// Nine dimensions come off the owner's calliper on a real key frame and
    /// they fix most of the rest: punchings 12 mm across at the balance and
    /// 22 mm at the front, with 4 mm and 5 mm holes; a round balance pin of
    /// 3.4 mm; a front rail pin oblong, 5.0 mm along the key by 3.4 mm across
    /// it; and 27 mm at the balance, 49 mm at the front, between the natural
    /// and sharp rows. [V] — `data/params/key-frame-sections.json`.
    ///
    /// **This is a generic base and not a model.** Key frames differ a great
    /// deal between makers, so one measured frame stands in for a plausible
    /// grand. A maker's own figures would be a named template, measured and
    /// labelled as that maker's; nothing here should be read as *the* figure
    /// for any particular instrument.
    ///
    /// **The row offsets are what set the rails' width**, through
    /// [`RailSection::minimum_half_length`], and they set it wide: 49 mm
    /// between the front rail's rows plus a 22 mm punching is 71 mm of wood at
    /// the least. So the rail lengths and the crown's flat below are not free
    /// choices: each is that minimum plus a margin, [E] only in the margin —
    /// 2 mm at the front rail and at the crown, 6 mm on the balance rail's body
    /// so that its crown has a shoulder to stand on.
    ///
    /// The **front rail pin stands 24 mm proud** of the rail: a 37 mm pin with
    /// a 13 mm round shank, driven 2 mm past its shoulder so that its tip
    /// still clears the top of the key's front mortise when the key is fully
    /// down — see [`RailSection::pin_height`] for that arithmetic. [V]
    ///
    /// What stays genuinely [E] is the crown's height, the back rail's length,
    /// and the **balance** pin's height, which waits on the balance mortise.
    fn default() -> Self {
        Self {
            balance: RailSection {
                half_length: 0.0255,
                crown: Some(Crown {
                    half_length: 0.0215,
                    height: 0.0060,
                }),
                pin: Some(PinShape::Round { diameter: 0.0034 }),
                pin_height: 0.0112,
                punching_radius: 0.0060,
                punching_hole_radius: 0.0020,
                sharp_row_offset: 0.0270,
            },
            front: RailSection {
                half_length: 0.0375,
                crown: None,
                pin: Some(PinShape::Oblong {
                    length: 0.0050,
                    thickness: 0.0034,
                }),
                pin_height: 0.0240,
                punching_radius: 0.0110,
                punching_hole_radius: 0.0025,
                sharp_row_offset: 0.0490,
            },
            back: RailSection {
                half_length: 0.0120,
                crown: None,
                pin: None,
                pin_height: 0.0,
                punching_radius: 0.0,
                punching_hole_radius: 0.0,
                sharp_row_offset: 0.0,
            },
        }
    }
}

/// How many thicknesses the drawer holds.
pub const PAPER_KINDS: usize = 8;

/// The thicknesses a punching drawer actually holds, metres, thinnest first.
///
/// 0.08, 0.12, 0.20, 0.40, 0.50, 0.75, 1.00 and 1.25 mm. Key height and key dip
/// are not continuous quantities on a real bench: they move in these steps, and a
/// target between two of them is not reachable. The bench shims with these and
/// reports what is left over rather than hiding it.
///
/// Thickness is also how a paper punching is **identified**: a drawer is sorted
/// by colour and the colour *is* the thickness — white, pastel green, sky blue,
/// camel, black, and card for anything above half a millimetre. That is why the
/// geometry emits one member per paper rather than one block of the right total
/// height, and why the view keys its palette on the thickness rather than on
/// the part. [V] — `data/params/punching-papers.json`.
pub const PAPER_THICKNESSES: [f64; PAPER_KINDS] = [
    0.00008, 0.00012, 0.0002, 0.0004, 0.0005, 0.00075, 0.001, 0.00125,
];

/// The step the drawer is graded in: 0.01 mm.
///
/// Every thickness in [`PAPER_THICKNESSES`] is a whole number of these, which
/// is what lets [`papers_for`] search the drawer *exactly* rather than
/// approximately. `every_paper_is_a_whole_number_of_steps` holds it true.
const PAPER_STEP: f64 = 0.000_01;

/// The tallest stack worth searching for, metres.
///
/// Beyond this the design is wrong and not the drawer, and
/// [`super::Regulation::solve`] has already refused it; this only stops a wild
/// number from sizing a table.
const PAPER_SEARCH_LIMIT: f64 = 0.100;

/// Builds the stack of real papers that comes closest to a wanted thickness
/// without overshooting — thickest first, which is how a pile is laid.
///
/// **Not greedy, and the difference is physical.** Taking the thickest paper
/// that fits and repeating is correct only for a drawer whose sizes divide one
/// another. This one does not: 0.24 mm is two pastel greens *exactly*, while
/// greed reaches for a sky blue and strands 0.04 mm that nothing in the drawer
/// can fill. A bench that reported that leftover would be blaming the drawer
/// for its own arithmetic — and key height is regulated in precisely those
/// hundredths. So the search is exhaustive: the closest total from below, and
/// among those, the fewest papers.
///
/// What is left over after that *is* a genuine miss. 0.37 mm is unreachable
/// because every paper thin enough to help is an even number of hundredths; the
/// honest answer is 0.36 and a stated 0.01 mm short, not a rounded 0.37.
pub fn papers_for(total: f64) -> Vec<f64> {
    if !total.is_finite() || total <= 0.0 || total > PAPER_SEARCH_LIMIT {
        return Vec::new();
    }
    let steps: [usize; PAPER_KINDS] =
        core::array::from_fn(|i| (PAPER_THICKNESSES[i] / PAPER_STEP).round() as usize);
    let target = (total / PAPER_STEP).round() as usize;

    // Fewest papers that reach exactly `n` steps, and which paper to peel off
    // the top of that stack to get there. `usize::MAX` means nothing reaches it.
    let mut count = vec![usize::MAX; target + 1];
    let mut peel = vec![0usize; target + 1];
    count[0] = 0;
    for n in 1..=target {
        for (i, &step) in steps.iter().enumerate() {
            if step <= n && count[n - step] != usize::MAX && count[n - step] + 1 < count[n] {
                count[n] = count[n - step] + 1;
                peel[n] = i;
            }
        }
    }

    let mut n = (0..=target)
        .rev()
        .find(|&n| count[n] != usize::MAX)
        .unwrap_or(0);
    let mut stack = Vec::with_capacity(count[n]);
    while n > 0 {
        let i = peel[n];
        stack.push(PAPER_THICKNESSES[i]);
        n -= steps[i];
    }
    stack.sort_by(|a, b| b.total_cmp(a));
    stack
}

/// An axis-aligned rectangle in the sagittal plane, as an outline.
fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<crate::geometry::Point> {
    use crate::geometry::Point;
    vec![
        Point::new(x0, y0),
        Point::new(x1, y0),
        Point::new(x1, y1),
        Point::new(x0, y1),
    ]
}

/// A washer, as a profile in radius and height to be turned about its axis.
///
/// Closed, and starting at the inner radius, so the lathe leaves the hole the
/// pin passes through. A disc with no hole would swallow its own pin.
fn washer(r_in: f64, r_out: f64, y0: f64, y1: f64) -> Vec<crate::geometry::Point> {
    use crate::geometry::Point;
    vec![
        Point::new(r_in, y0),
        Point::new(r_out, y0),
        Point::new(r_out, y1),
        Point::new(r_in, y1),
    ]
}

/// One drawable, contactable member of the frame.
///
/// The frame does not move, so its members need no inertia and no pivot — but
/// they do need outlines, because the view has to draw the thing the physics
/// is touching. The stops do not read these outlines: they read the bearing
/// heights of [`FrameDesign`], from which the members are built.
/// `what_is_drawn_is_what_is_touched` checks that the two meet.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameMember {
    /// Stable identifier, English `snake_case`, matching the glossary.
    pub id: &'static str,
    /// Closed outline. In the keybed frame — `x` towards the player from the
    /// balance pin, `y` up from the keybed — when [`super::Sweep::Extruded`];
    /// in **radius and height** about [`FrameMember::axis_x`] when
    /// [`super::Sweep::Revolved`].
    pub outline: Vec<crate::geometry::Point>,
    /// Whether this member is a compliant lining — felt, cloth, paper — rather
    /// than wood or metal.
    pub compliant: bool,
    /// How the outline becomes a solid.
    pub sweep: super::Sweep,
    /// Extent across the keyboard, metres. Meaningless when revolved.
    pub width: f64,
    /// Where the axis stands along the key, metres. Meaningless when extruded.
    pub axis_x: f64,
    /// For a paper punching, its own thickness in metres — which is how the
    /// trade identifies one, and therefore how it is coloured. `None` for
    /// everything else.
    pub paper: Option<f64>,
}

/// What the frame needs to know beyond its regulation to be drawn.
///
/// The papers arrive as the **discrete stack actually laid**, not as a total,
/// because two stacks of the same height are not the same object on a bench:
/// one is four thin shims and the other is one thick one, and the technician
/// reaches for a different drawer.
#[derive(Debug, Clone, Copy)]
pub struct FrameLayout<'a> {
    /// Which row of pins this key stands on.
    pub kind: KeyKind,
    /// Papers under the balance punching, bottom first, metres.
    pub balance_papers: &'a [f64],
    /// Papers under the front punching, bottom first, metres.
    pub front_papers: &'a [f64],
}

impl FrameDesign {
    /// Every member of the frame, as outlines in the keybed frame.
    ///
    /// Ordered back to front, which is the order a technician would name them.
    /// `key` supplies the longitudinal positions of the three bearings, because
    /// the rails stand where the key needs them and not at arbitrary places.
    pub fn members(&self, key: &super::KeyDesign, layout: &FrameLayout) -> Vec<FrameMember> {
        use super::Sweep;
        let s = &self.sections;
        let mut out = Vec::with_capacity(16);

        // The rails. A rail is continuous across the keyboard, so a single-key
        // bench shows a slice a little wider than the key — enough to read as
        // "this goes on", not so much that it hides the key.
        //
        // The slice has to beat the **longest** rail along the key as well, and
        // not only the key's own width. A front rail 75 mm long cut to a 40 mm
        // slice reads as a post standing under the key, which is the opposite
        // of what it is.
        let longest = s
            .balance
            .half_length
            .max(s.front.half_length)
            .max(s.back.half_length);
        let slice = (key.width * 1.8).max(2.4 * longest);
        let mut rail = |id: &'static str, natural_x: f64, top: f64, sec: &RailSection| {
            // Centred between its two rows. A rail wide enough for both is not
            // symmetric about either, and the front rail is the clear case: it
            // reaches 24.5 mm back from the natural's pin towards the sharps.
            let x = sec.centre_x(natural_x);
            let h = sec.half_length;
            let outline = match sec.crown {
                None => rect(x - h, 0.0, x + h, top),
                Some(crown) => {
                    let shoulder = top - crown.height;
                    let c = crown.half_length;
                    vec![
                        crate::geometry::Point::new(x - h, 0.0),
                        crate::geometry::Point::new(x + h, 0.0),
                        crate::geometry::Point::new(x + h, shoulder),
                        crate::geometry::Point::new(x + c, top),
                        crate::geometry::Point::new(x - c, top),
                        crate::geometry::Point::new(x - h, shoulder),
                    ]
                }
            };
            out.push(FrameMember {
                id,
                outline,
                compliant: false,
                sweep: Sweep::Extruded,
                width: slice,
                axis_x: x,
                paper: None,
            });
        };
        rail("back_rail", self.back_rail, self.back_rail_height, &s.back);
        rail("balance_rail", 0.0, self.balance_rail_height, &s.balance);
        rail(
            "front_rail",
            self.front_rail,
            self.front_rail_height,
            &s.front,
        );

        // The back rail cloth runs the length of the rail rather than being a
        // washer, so it stays a strip.
        out.push(FrameMember {
            id: "back_rail_cloth",
            outline: rect(
                s.back.centre_x(self.back_rail) - s.back.half_length,
                self.back_rail_height,
                s.back.centre_x(self.back_rail) + s.back.half_length,
                self.back_bearing_height(),
            ),
            compliant: true,
            sweep: Sweep::Extruded,
            width: slice,
            axis_x: self.back_rail,
            paper: None,
        });

        // The punching stacks: papers first, felt on top, each a washer turned
        // about the pin it sits on. Drawn one paper at a time, because that is
        // what the drawer holds and what the technician counts.
        let balance_x = s.balance.pin_x(0.0, layout.kind);
        let front_x = s.front.pin_x(self.front_rail, layout.kind);
        out.extend(punching_stack(
            self.balance_rail_height,
            balance_x,
            &s.balance,
            layout.balance_papers,
            self.balance_punching.felt,
            "balance_rail_paper",
            "balance_rail_punching",
        ));
        out.extend(punching_stack(
            self.front_rail_height,
            front_x,
            &s.front,
            layout.front_papers,
            self.front_punching.felt,
            "front_rail_paper",
            "front_rail_punching",
        ));

        // The pins. They guide the key laterally and carry no vertical load,
        // which is why the bearing is the punching and not the pin — but they
        // are drawn, because a reader checks that the key is on its pins.
        if let Some(pin) = s.balance.pin {
            out.push(pin_member(
                "balance_rail_pin",
                pin,
                balance_x,
                self.balance_rail_height,
                self.balance_rail_height + s.balance.pin_height,
            ));
        }
        if let Some(pin) = s.front.pin {
            out.push(pin_member(
                "front_rail_pin",
                pin,
                front_x,
                self.front_rail_height,
                self.front_rail_height + s.front.pin_height,
            ));
        }
        out
    }
}

/// One rail's papers and its felt washer, bottom to top.
fn punching_stack(
    rail_top: f64,
    axis_x: f64,
    sec: &RailSection,
    papers: &[f64],
    felt: f64,
    paper_id: &'static str,
    felt_id: &'static str,
) -> Vec<FrameMember> {
    use super::Sweep;
    let r_in = sec.punching_hole_radius;
    let r_out = sec.punching_radius;
    let mut out = Vec::with_capacity(papers.len() + 1);
    let mut y = rail_top;
    for &t in papers {
        out.push(FrameMember {
            id: paper_id,
            outline: washer(r_in, r_out, y, y + t),
            compliant: true,
            sweep: Sweep::Revolved,
            width: 0.0,
            axis_x,
            paper: Some(t),
        });
        y += t;
    }
    out.push(FrameMember {
        id: felt_id,
        outline: washer(r_in, r_out, y, y + felt),
        compliant: true,
        sweep: Sweep::Revolved,
        width: 0.0,
        axis_x,
        paper: None,
    });
    out
}

/// A pin, turned if it is round and extruded if it is flattened.
fn pin_member(id: &'static str, pin: PinShape, x: f64, bottom: f64, top: f64) -> FrameMember {
    use super::Sweep;
    match pin {
        PinShape::Round { diameter } => FrameMember {
            id,
            outline: washer(0.0, 0.5 * diameter, bottom, top),
            compliant: false,
            sweep: Sweep::Revolved,
            width: 0.0,
            axis_x: x,
            paper: None,
        },
        PinShape::Oblong { length, thickness } => FrameMember {
            id,
            outline: rect(x - 0.5 * length, bottom, x + 0.5 * length, top),
            compliant: false,
            sweep: Sweep::Extruded,
            width: thickness,
            axis_x: x,
            paper: None,
        },
    }
}

#[cfg(test)]
mod drawer_tests {
    use super::*;

    /// The search through the drawer is exact only while this holds.
    #[test]
    fn every_paper_is_a_whole_number_of_steps() {
        for &t in &PAPER_THICKNESSES {
            let steps = t / PAPER_STEP;
            assert!(
                (steps - steps.round()).abs() < 1e-6,
                "{t} m is not a whole number of 0.01 mm steps, so the search is no longer exact"
            );
        }
        assert!(
            PAPER_THICKNESSES.windows(2).all(|w| w[0] < w[1]),
            "the drawer is listed thinnest first"
        );
    }

    /// Two pastel greens, not one sky blue and a gap nothing can fill.
    ///
    /// The drawer's sizes do not divide one another, so taking the thickest
    /// paper that fits is simply wrong: 0.24 mm is exactly 2 x 0.12, while
    /// greed takes 0.20 and strands 0.04 below the thinnest paper there is.
    /// Key height is regulated in those hundredths, so this is the difference
    /// between a bench that can reach a target and one that cannot.
    #[test]
    fn the_stack_is_the_best_the_drawer_can_build_not_the_first_that_fits() {
        let stack = papers_for(0.00024);
        let total: f64 = stack.iter().sum();
        assert!(
            (total - 0.00024).abs() < 1e-12,
            "0.24 mm is two 0.12 mm papers; got {stack:?}"
        );
        assert_eq!(stack.len(), 2, "and two is the fewest that reach it");
    }

    /// A stack never overshoots, is laid thickest first, and holds only papers
    /// that exist.
    #[test]
    fn a_stack_is_laid_thickest_first_and_never_overshoots() {
        for target in [0.0, 0.00005, 0.0001, 0.00037, 0.0005, 0.0012, 0.003, 0.0075] {
            let stack = papers_for(target);
            let total: f64 = stack.iter().sum();
            assert!(total <= target + 1e-12, "{target} m overshot: {stack:?}");
            assert!(
                stack.windows(2).all(|w| w[0] >= w[1]),
                "{target} m: thick papers go under the thin ones, got {stack:?}"
            );
            for &paper in &stack {
                assert!(
                    PAPER_THICKNESSES.iter().any(|t| (t - paper).abs() < 1e-15),
                    "{paper} m is not a paper anybody stocks"
                );
            }
        }
    }

    /// An unreachable target misses by what the drawer genuinely cannot fill.
    ///
    /// 0.37 mm is unreachable: every paper thin enough to help is an even
    /// number of hundredths, and 37 is odd. The honest answer is 0.36 and a
    /// stated 0.01 mm short.
    #[test]
    fn an_unreachable_target_misses_by_what_the_drawer_cannot_fill() {
        let stack = papers_for(0.00037);
        let total: f64 = stack.iter().sum();
        assert!((total - 0.00036).abs() < 1e-12, "got {stack:?}");
    }

    /// Nonsense in, nothing out — never a table sized from a wild number.
    #[test]
    fn an_impossible_thickness_builds_no_stack() {
        for bad in [0.0, -0.001, f64::NAN, f64::INFINITY, 10.0] {
            assert!(papers_for(bad).is_empty(), "{bad} built a stack");
        }
    }
}

#[cfg(test)]
mod member_tests {
    use super::*;
    use crate::parts::{KeyDesign, Sweep};

    /// The frame as the bench lays it: real papers under both felts.
    fn laid(frame: &FrameDesign, sections: &FrameSections, kind: KeyKind) -> Vec<FrameMember> {
        let mut frame = frame.clone();
        frame.sections = *sections;
        let balance = papers_for(frame.balance_punching.paper);
        let front = papers_for(frame.front_punching.paper);
        frame.members(
            &KeyDesign::default(),
            &FrameLayout {
                kind,
                balance_papers: &balance,
                front_papers: &front,
            },
        )
    }

    fn top_of(members: &[FrameMember], id: &str) -> f64 {
        members
            .iter()
            .filter(|m| m.id == id)
            .flat_map(|m| m.outline.iter())
            .fold(f64::MIN, |a, p| a.max(p.y))
    }

    /// Every member the goal names is present, and the stops the key touches
    /// are among them.
    #[test]
    fn the_frame_carries_its_rails_pins_punchings_and_cloth() {
        let members = laid(
            &FrameDesign::default(),
            &FrameSections::default(),
            KeyKind::Natural,
        );
        for id in [
            "balance_rail",
            "balance_rail_pin",
            "balance_rail_punching",
            "balance_rail_paper",
            "front_rail",
            "front_rail_pin",
            "front_rail_punching",
            "front_rail_paper",
            "back_rail",
            "back_rail_cloth",
        ] {
            assert!(
                members.iter().any(|m| m.id == id),
                "the frame is missing its {id}"
            );
        }
    }

    /// The drawn top of each bearing is the height the physics uses as a stop.
    ///
    /// This is the single-definition invariant at its smallest scale: if the
    /// punching were drawn from one number and contacted at another, a
    /// technician would add paper, see the key rise, and read an unchanged dip.
    #[test]
    fn what_is_drawn_is_what_is_touched() {
        let frame = FrameDesign::default();
        let members = laid(&frame, &FrameSections::default(), KeyKind::Natural);

        assert!(
            (top_of(&members, "balance_rail_punching") - frame.balance_bearing_height()).abs()
                < 1e-15
        );
        assert!(
            (top_of(&members, "front_rail_punching") - frame.front_bearing_height()).abs() < 1e-15
        );
        assert!((top_of(&members, "back_rail_cloth") - frame.back_bearing_height()).abs() < 1e-15);
    }

    /// Adding paper raises the felt that sits on it, by exactly what was added.
    ///
    /// The felt does not grow — it is lifted. That is the operation, and a
    /// model where the washer got thicker instead would teach the wrong thing.
    #[test]
    fn adding_paper_lifts_the_felt_without_thickening_it() {
        let sections = FrameSections::default();
        let base = FrameDesign::default();
        let mut raised = base.clone();
        raised.balance_punching.paper += 0.0004;

        let felt_span = |f: &FrameDesign| {
            let m = laid(f, &sections, KeyKind::Natural);
            let p = m
                .iter()
                .find(|m| m.id == "balance_rail_punching")
                .expect("the felt washer");
            let (lo, hi) = p.outline.iter().fold((f64::MAX, f64::MIN), |(lo, hi), q| {
                (lo.min(q.y), hi.max(q.y))
            });
            (lo, hi - lo)
        };
        let (base_bottom, base_thickness) = felt_span(&base);
        let (raised_bottom, raised_thickness) = felt_span(&raised);

        assert!(
            (raised_bottom - base_bottom - 0.0004).abs() < 1e-15,
            "0.4 mm of paper must lift the felt by 0.4 mm"
        );
        assert!(
            (raised_thickness - base_thickness).abs() < 1e-15,
            "the felt washer is a part, not a variable thickness"
        );
    }

    /// The papers are drawn one at a time, and they stack to the whole shim.
    ///
    /// A technician counts papers, so the picture has to show papers. One block
    /// of the right total height would be the same geometry and a different
    /// instrument.
    #[test]
    fn each_paper_is_drawn_as_itself_and_they_stack_without_a_gap() {
        let frame = FrameDesign::default();
        let members = laid(&frame, &FrameSections::default(), KeyKind::Natural);
        let papers: Vec<&FrameMember> = members
            .iter()
            .filter(|m| m.id == "balance_rail_paper")
            .collect();

        assert_eq!(
            papers.len(),
            papers_for(frame.balance_punching.paper).len(),
            "one member per paper in the drawer's stack"
        );
        let mut y = frame.balance_rail_height;
        for m in &papers {
            let t = m.paper.expect("a paper carries its own thickness");
            let (lo, hi) = m.outline.iter().fold((f64::MAX, f64::MIN), |(lo, hi), q| {
                (lo.min(q.y), hi.max(q.y))
            });
            assert!((lo - y).abs() < 1e-15, "papers stack with no gap");
            assert!(
                (hi - lo - t).abs() < 1e-15,
                "a paper is drawn its own thickness"
            );
            y += t;
        }
        assert!(
            (y - frame.balance_rail_height - frame.balance_punching.paper).abs() < 1e-12,
            "the stack sums to the shim the regulation asked for"
        );
    }

    /// A punching is a washer, and its hole admits the pin that passes through.
    ///
    /// Drawn as a disc it would swallow its own pin, and the reader would lose
    /// the one feature that says the key is located rather than merely resting.
    ///
    /// The bound is `>=`, not `>`, because at the front rail the measured hole
    /// is *exactly* the pin's long axis: a 5 mm circle round a 5 mm oblong,
    /// touching at its two ends. Demanding a clearance there would reject the
    /// real part.
    #[test]
    fn a_punching_is_a_washer_whose_hole_admits_its_pin() {
        let sections = FrameSections::default();
        let members = laid(&FrameDesign::default(), &sections, KeyKind::Natural);

        for (id, sec) in [
            ("balance_rail_punching", sections.balance),
            ("front_rail_punching", sections.front),
        ] {
            let m = members.iter().find(|m| m.id == id).expect("the washer");
            assert_eq!(m.sweep, Sweep::Revolved, "{id} is turned, not extruded");
            let r_in = m.outline.iter().fold(f64::MAX, |a, p| a.min(p.x));
            let r_out = m.outline.iter().fold(f64::MIN, |a, p| a.max(p.x));
            let pin = sec.pin.expect("this rail has a pin");
            assert!(
                r_in >= 0.5 * pin.length() - 1e-12,
                "{id}: the hole must admit the pin's longest axis"
            );
            assert!(r_out > r_in, "{id}: a washer has a rim");
            assert!(
                (r_out - sec.punching_radius).abs() < 1e-15,
                "{id}: drawn at the washer's own radius"
            );
        }
    }

    /// The balance rail is a block with a trapezoid on it, narrower on top.
    ///
    /// One piece, and the crown is what the key's balance mortise straddles.
    #[test]
    fn the_balance_rail_is_crowned_and_the_others_are_not() {
        let sections = FrameSections::default();
        let frame = FrameDesign::default();
        let members = laid(&frame, &sections, KeyKind::Natural);

        let rail = members
            .iter()
            .find(|m| m.id == "balance_rail")
            .expect("the balance rail");
        let crown = sections.balance.crown.expect("it is crowned");
        let width_at = |y: f64| {
            let xs: Vec<f64> = rail
                .outline
                .iter()
                .filter(|p| (p.y - y).abs() < 1e-12)
                .map(|p| p.x)
                .collect();
            xs.iter().fold(f64::MIN, |a, &x| a.max(x)) - xs.iter().fold(f64::MAX, |a, &x| a.min(x))
        };
        assert!(
            (width_at(0.0) - 2.0 * sections.balance.half_length).abs() < 1e-15,
            "the block is the rail's full width at the keybed"
        );
        assert!(
            (width_at(frame.balance_rail_height) - 2.0 * crown.half_length).abs() < 1e-15,
            "the flat on top is the crown's width"
        );
        assert!(
            width_at(frame.balance_rail_height) < width_at(0.0),
            "a crown is narrower than what carries it"
        );

        for id in ["front_rail", "back_rail"] {
            let m = members.iter().find(|m| m.id == id).expect(id);
            assert_eq!(m.outline.len(), 4, "{id} is a plain block");
        }
    }

    /// A rail carries both of its rows, so it has to be wide enough for them.
    ///
    /// The constraint is invisible on the bench, which draws one row: a rail
    /// far too narrow to carry the other looks perfectly correct in the view.
    /// A rail of 20 mm cannot hold two rows 49 mm apart, whatever it looks like
    /// from the side.
    #[test]
    fn each_rail_carries_both_of_its_rows() {
        let sections = FrameSections::default();

        for (name, sec) in [
            ("balance rail", sections.balance),
            ("front rail", sections.front),
            ("back rail", sections.back),
        ] {
            assert!(
                sec.half_length >= sec.minimum_half_length(),
                "the {name} is {:.1} mm along the key and needs {:.1} to carry both rows",
                sec.half_length * 2000.0,
                sec.minimum_half_length() * 2000.0
            );
            if let Some(crown) = sec.crown {
                assert!(
                    crown.half_length >= sec.minimum_half_length(),
                    "the {name}'s punchings bear on the crown, so the flat carries both rows too"
                );
                assert!(
                    crown.half_length < sec.half_length,
                    "a crown is narrower than what carries it"
                );
            }
        }

        // And the rail as drawn really does span them, which is what
        // `centre_x` is for: the width alone would be satisfied by a rail
        // sitting in the wrong place.
        for (id, sec, natural_x) in [
            ("balance_rail", sections.balance, 0.0),
            (
                "front_rail",
                sections.front,
                FrameDesign::default().front_rail,
            ),
        ] {
            let members = laid(&FrameDesign::default(), &sections, KeyKind::Natural);
            let rail = members.iter().find(|m| m.id == id).expect(id);
            let (x0, x1) = rail
                .outline
                .iter()
                .fold((f64::MAX, f64::MIN), |(a, b), p| (a.min(p.x), b.max(p.x)));
            for kind in [KeyKind::Natural, KeyKind::Sharp] {
                let pin = sec.pin_x(natural_x, kind);
                assert!(
                    pin - sec.punching_radius >= x0 - 1e-12
                        && pin + sec.punching_radius <= x1 + 1e-12,
                    "{id}: the {kind:?} row's punching hangs off the wood"
                );
            }
        }
    }

    /// The frame shipped as the default is the one the rule builds for the
    /// default key.
    ///
    /// Two expressions of the same frame, and they have to agree: the
    /// coordinates are typed out in `FrameDesign::default` so a reader can see
    /// them, and derived in `placed_under` so a changed key can be given a
    /// frame. If they drifted, the bench would open on a keyboard its own rule
    /// calls wrong.
    #[test]
    fn the_default_frame_is_the_one_the_rule_builds_for_the_default_key() {
        let key = KeyDesign::default();
        let typed = FrameDesign::default();
        let placed = FrameDesign::default().placed_under(&key);
        assert!(
            (typed.front_rail - placed.front_rail).abs() < 1e-15,
            "the front rail is typed at {} m and placed at {} m",
            typed.front_rail,
            placed.front_rail
        );
        assert!(
            (typed.back_rail - placed.back_rail).abs() < 1e-15,
            "the back rail is typed at {} m and placed at {} m",
            typed.back_rail,
            placed.back_rail
        );
    }

    /// Placing a frame under a key puts the front rail pin in its mortise and
    /// the back rail cloth inside the chamfer — whatever the key's length.
    ///
    /// The rule is tested on keys the default was not built for, because a
    /// relation that only holds at the value it was derived from is not a
    /// relation. A 300 mm key is a long natural; a 180 mm one is shorter than
    /// anything in a piano, and the rule still has to produce a frame that
    /// goes together.
    #[test]
    fn placing_the_frame_follows_the_key_it_is_built_for() {
        for front_edge in [0.180_f64, 0.220, 0.258, 0.300] {
            for tail_end in [-0.200_f64, -0.240, -0.280] {
                let key = KeyDesign {
                    front_edge,
                    tail_end,
                    ..KeyDesign::default()
                };
                let frame = FrameDesign::default().placed_under(&key);
                let (mortise_back, mortise_front) = key.front_mortise_span();
                let pin = frame.sections.front.pin.expect("the front rail has a pin");
                assert!(
                    frame.front_rail - 0.5 * pin.length() >= mortise_back - 1e-12
                        && frame.front_rail + 0.5 * pin.length() <= mortise_front + 1e-12,
                    "a key with its front edge at {front_edge} m gets a pin at {} m, \
                     against a mortise running {mortise_back} to {mortise_front} m",
                    frame.front_rail
                );

                let (chamfer_begins, _) = key.underside_flat();
                let half = frame.sections.back.half_length;
                assert!(
                    frame.back_rail + half < chamfer_begins,
                    "a key ending at {tail_end} m gets its cloth reaching {} m, \
                     in front of the chamfer at {chamfer_begins} m",
                    frame.back_rail + half
                );
                assert!(
                    frame.back_rail - half > tail_end,
                    "and reaching {} m, past the end of the key at {tail_end} m",
                    frame.back_rail - half
                );
            }
        }
    }

    /// A sharp stands on the other row of pins, and nothing else moves.
    ///
    /// Each rail carries two rows. The bench shows one key, so it shows one
    /// row — but the design has to be able to name the other, because sharps
    /// are first-class and a keyboard cannot be graduated without them.
    #[test]
    fn a_sharp_stands_on_the_second_row_of_pins() {
        let sections = FrameSections::default();
        let frame = FrameDesign::default();
        let natural = laid(&frame, &sections, KeyKind::Natural);
        let sharp = laid(&frame, &sections, KeyKind::Sharp);

        let axis = |m: &[FrameMember], id: &str| m.iter().find(|x| x.id == id).expect(id).axis_x;
        assert!(
            (axis(&natural, "balance_rail_pin")
                - axis(&sharp, "balance_rail_pin")
                - sections.balance.sharp_row_offset)
                .abs()
                < 1e-15,
            "the sharp's balance pin is set back by the row offset"
        );
        assert!(
            (axis(&natural, "front_rail_pin")
                - axis(&sharp, "front_rail_pin")
                - sections.front.sharp_row_offset)
                .abs()
                < 1e-15,
            "and so is its front pin"
        );

        let rail_of =
            |m: &[FrameMember], id: &str| m.iter().find(|x| x.id == id).expect(id).outline.clone();
        for id in ["balance_rail", "front_rail", "back_rail"] {
            assert_eq!(
                rail_of(&natural, id),
                rail_of(&sharp, id),
                "{id} is one rail; only the row of pins differs"
            );
        }
    }

    /// The balance pin is turned and the front rail pin is flattened.
    #[test]
    fn the_two_pins_are_not_the_same_part() {
        let sections = FrameSections::default();
        let members = laid(&FrameDesign::default(), &sections, KeyKind::Natural);

        let balance = members
            .iter()
            .find(|m| m.id == "balance_rail_pin")
            .expect("balance pin");
        assert_eq!(balance.sweep, Sweep::Revolved, "a balance pin is round");

        let front = members
            .iter()
            .find(|m| m.id == "front_rail_pin")
            .expect("front pin");
        assert_eq!(front.sweep, Sweep::Extruded, "a front rail pin is oblong");
        let length = front.outline.iter().fold(f64::MIN, |a, p| a.max(p.x))
            - front.outline.iter().fold(f64::MAX, |a, p| a.min(p.x));
        assert!(
            length > front.width,
            "an oblong pin is longer along the key than it is thick across it"
        );
    }
}

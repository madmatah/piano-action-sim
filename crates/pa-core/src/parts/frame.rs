//! The key frame: the three rails, their pins, and the punchings that sit on
//! them.
//!
//! # Why the frame is a part and not a set of constants
//!
//! Two of the settings a technician spends the most time on live here and
//! nowhere else. **Key height** is the balance rail punching stack. **Key dip**
//! is the gap left above the front rail punching. Both are *stacks of paper and
//! felt washers*, adjusted a few hundredths of a millimeter at a time, and both
//! have to be manipulable for the bench to teach anything. A frame expressed as
//! fixed numbers would make leveling and dipping inexpressible.
//!
//! # Frame
//!
//! Shared with [`super::KeyDesign`]: origin at the balance rail pin, `x` toward
//! the player, `y` up. The keybed surface is the vertical datum, `y = 0`. The
//! frame is the ground's: [`FrameDesign::parts`] builds its parts on
//! [`BodyId::GROUND`].

use crate::contact::{Material, Part, PartCode, PartId, PlacedBy, Role, SurfaceRef, Sweep};
use crate::geometry::{Dir, Point, Seg};
use crate::mechanism::BodyId;
use crate::registry::{Kind, Layer, ParamSpec, Quantity, Step};
use crate::validate::Problem;

/// A stack of punchings on a rail: the papers laid, bottom first, and the
/// felt or leather washer on them.
///
/// The papers are a list and not a total because a stack is a pile a
/// technician counts, laid and lifted one paper at a time: the list **is**
/// the coordinate regulation sets, held here once, and every total is summed
/// from it. Kept apart from the felt because they are two different
/// operations on the bench — paper is how you adjust, felt is what you fit —
/// and because the felt is compliant while the paper is not. The geometry
/// treats the whole stack as rigid and reads only the total; the split is
/// what a model of the felt's compliance will need.
#[derive(Debug, Clone, PartialEq)]
pub struct PunchingStack {
    /// The papers laid on the rail, bottom first, meters each.
    pub papers: Vec<f64>,
    /// Thickness of the felt or leather washer, meters.
    pub felt: f64,
}

impl PunchingStack {
    /// A stack of these papers, bottom first, and a felt.
    pub fn new(papers: Vec<f64>, felt: f64) -> Self {
        Self { papers, felt }
    }

    /// The papers' total thickness, meters, summed bottom first.
    pub fn paper(&self) -> f64 {
        self.papers.iter().sum()
    }

    /// Total uncompressed height, meters.
    pub fn height(&self) -> f64 {
        self.paper() + self.felt
    }
}

/// The key frame and everything mounted on it.
///
/// Rail heights are the top of the bare rail above the keybed; the punching
/// stack sits on top of that.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameDesign {
    /// Top of the balance rail above the keybed, meters.
    pub balance_rail_height: f64,
    /// Punchings under the key at the balance pin.
    pub balance_punching: PunchingStack,
    /// Top of the front rail above the keybed, meters.
    pub front_rail_height: f64,
    /// Punchings on the front rail. Their height sets the **key dip**.
    pub front_punching: PunchingStack,
    /// Top of the back rail above the keybed, meters.
    pub back_rail_height: f64,
    /// Cloth on the back rail.
    pub back_rail_cloth: f64,
    /// The radius the back rail cloth's two top edges are rounded to,
    /// meters: a shape, not a setting. Small enough that where the key bears
    /// on an edge is essentially the end of the cloth's face, large enough
    /// to be a real curve rather than a numerical fiction; it is drawn, and
    /// the contact with the key's underside reads the arcs as drawn.
    pub back_rail_cloth_edge_radius: f64,
    /// Where the front rail stands, meters in front of the balance pin,
    /// measured to the **natural** row of pins.
    ///
    /// A frame coordinate and not a key one, although it is placed from the
    /// key: the pin runs in the key's front mortise, so
    /// [`Self::placed_under`] puts it there. What keeps it here is that a key
    /// changed afterwards must **not** drag the frame along behind it. Lengthen
    /// a key on a real keyboard and the rail stays bolted where it was; the
    /// pin leaves the mortise and the keyboard is wrong until somebody moves
    /// the rail. [`super::faults::faults`] says so.
    pub front_rail: f64,
    /// Where the back rail stands, meters behind the balance pin (negative).
    ///
    /// **Bounded by the key's chamfer, and bounded the other way round from
    /// how it first looks.** The chamfer under the tail is cut so the key beds
    /// *flat* on this cloth, so the cloth has to lie **within** the chamfer —
    /// behind where it begins, and far enough behind that the whole width of
    /// the rail is under it. Cloth in front of the chamfer could only ever be
    /// met by the kink, which is the thing the chamfer exists to prevent.
    pub back_rail: f64,
    /// How far in from the end of the key [`Self::placed_under`] sets the
    /// back rail, meters: the relation a frame is built to under a key, not
    /// a dimension of any part.
    ///
    /// The tail has to overhang the rail — the key is lifted off the frame
    /// from behind, and the damper lift works at the very end of the tail —
    /// and the cloth has to sit wholly within the chamfer cut to bed on it.
    pub back_rail_from_the_tail: f64,
    /// The three rails as parts: their sections, pins and punching sizes.
    ///
    /// Here rather than beside the design because a section is not only drawn.
    /// A pin's height decides how far the key may descend before it bottoms in
    /// its mortise, which is a **stop** — so the statics have to be able to see
    /// it, and a frame that held its regulation in one place and its parts in
    /// another would have the contact set reading half of itself.
    pub sections: FrameSections,
    /// The drawer the punchings' papers are laid from: the stock the frame
    /// is shimmed with, every paper on a punching one of its thicknesses.
    /// Held with the frame because the regulation lays from it and the
    /// validation reads every paper against it.
    pub drawer: Drawer,
}

/// One length of the frame, set directly in the design layer, from nothing
/// up: a rail's height, a cloth, a felt, a pin's height above its rail.
const fn frame_length(
    name: &'static str,
    get: fn(&FrameDesign) -> f64,
    set: fn(&mut FrameDesign, f64),
) -> ParamSpec<FrameDesign> {
    ParamSpec {
        name,
        quantity: Quantity::Length,
        hard: (0.0, f64::INFINITY),
        step: Step::Continuous,
        layer: Layer::Design,
        kind: Kind::Actuated,
        get,
        set,
    }
}

impl FrameDesign {
    /// The frame's settings: what an instrument may set on it, each with
    /// what it admits ([`ParamSpec`]).
    ///
    /// Heights, thicknesses and the pins' heights are no less than
    /// nothing, the balance pin under a meter, past which the key's slide
    /// would run; the front rail stands in front of the balance pin and the
    /// back rail behind it. Anything else a frame can be, a rail placed for
    /// another key or one raised until the stops fight, is computed and
    /// reported, not refused: a frame drifts from its key, it does not
    /// cascade. The papers are not here: they are a list, laid and lifted a
    /// paper at a time ([`PAPER`]).
    pub const PARAMS: [ParamSpec<Self>; 10] = [
        frame_length(
            "balance_rail.height",
            |f| f.balance_rail_height,
            |f, v| f.balance_rail_height = v,
        ),
        frame_length(
            "front_rail.height",
            |f| f.front_rail_height,
            |f, v| f.front_rail_height = v,
        ),
        frame_length(
            "back_rail.height",
            |f| f.back_rail_height,
            |f, v| f.back_rail_height = v,
        ),
        frame_length("front_rail.x", |f| f.front_rail, |f, v| f.front_rail = v),
        ParamSpec {
            hard: (f64::NEG_INFINITY, 0.0),
            ..frame_length("back_rail.x", |f| f.back_rail, |f, v| f.back_rail = v)
        },
        frame_length(
            "front_rail.pin_height",
            |f| f.sections.front.pin_height,
            |f, v| f.sections.front.pin_height = v,
        ),
        frame_length(
            "back_rail.cloth",
            |f| f.back_rail_cloth,
            |f, v| f.back_rail_cloth = v,
        ),
        frame_length(
            "balance_punching.felt",
            |f| f.balance_punching.felt,
            |f, v| f.balance_punching.felt = v,
        ),
        frame_length(
            "front_punching.felt",
            |f| f.front_punching.felt,
            |f, v| f.front_punching.felt = v,
        ),
        ParamSpec {
            hard: (0.0, 1.0),
            ..frame_length(
                "balance_rail.pin_height",
                |f| f.sections.balance.pin_height,
                |f, v| f.sections.balance.pin_height = v,
            )
        },
    ];

    /// Height of the key's underside at the balance pin, at rest, meters.
    ///
    /// This is what key leveling moves, and it is the only one of the three
    /// bearings that carries the key at rest on a bench with no action fitted.
    pub fn balance_bearing_height(&self) -> f64 {
        self.balance_rail_height + self.balance_punching.height()
    }

    /// Height of the top of the front rail punching, meters.
    pub fn front_bearing_height(&self) -> f64 {
        self.front_rail_height + self.front_punching.height()
    }

    /// Height of the top of the back rail cloth, meters.
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
    ///   chamfer cut to bed on it: [`Self::back_rail_from_the_tail`].
    ///
    /// Running this is how a frame is **built**, and it is the right default:
    /// nobody should have to type a coordinate to get a keyboard that goes
    /// together. What it is not is how a frame is **maintained**. Change a key
    /// afterwards and the frame stays where it is — rails are part of the
    /// key frame and do not follow — so the mismatch appears, and
    /// [`super::faults::faults`] names it. That is the project's standing
    /// rule that upstream settings drift rather than cascade, applied to
    /// geometry: a frame that silently re-placed itself would hide exactly the
    /// lesson a keyboard designer is here to learn.
    ///
    /// So a measured frame keeps its own numbers and is judged against the key
    /// it is meant to carry, which is a different job from building one and a
    /// real one.
    pub fn placed_under(mut self, key: &super::KeyDesign) -> Self {
        self.front_rail = key.front_mortise_x();
        self.back_rail = key.tail_end + self.back_rail_from_the_tail;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stacking is addition, and the split into paper and felt does not change
    /// the geometry at rest.
    #[test]
    fn a_punching_stack_is_the_sum_of_what_is_on_it() {
        let a = PunchingStack::new(vec![0.0005], 0.0030);
        let b = PunchingStack::new(vec![0.0020], 0.0015);
        assert!((a.height() - 0.0035).abs() < 1e-15);
        assert!(
            (a.height() - b.height()).abs() < 1e-15,
            "same total, different split: the rest geometry must not care"
        );
    }

    /// Adding paper under the balance punching raises the key, and moves nothing
    /// else.
    ///
    /// This is the trade operation "key leveling", and it is the simplest
    /// possible non-confusion test: the lever that lifts the key must not be the
    /// lever that sets the dip.
    #[test]
    fn leveling_the_key_does_not_change_the_front_or_back_bearings() {
        let base = crate::test_fixture::frame();
        let mut raised = base.clone();
        raised.balance_punching.papers.push(0.0004);

        assert!(
            (raised.balance_bearing_height() - base.balance_bearing_height() - 0.0004).abs()
                < 1e-15,
            "a 0.4 mm shim must raise the bearing by 0.4 mm"
        );
        assert!(
            (raised.front_bearing_height() - base.front_bearing_height()).abs() < 1e-15,
            "leveling must not touch the front rail"
        );
        assert!(
            (raised.back_bearing_height() - base.back_bearing_height()).abs() < 1e-15,
            "leveling must not touch the back rail"
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
/// mortise swing without binding. A **front rail pin is oval**, as the owner
/// describes it: longer along the key than across it, its domed sides
/// presented to the bushing cloth on either side of the key's front mortise.
/// That is what holds the key straight, while the key is free to rise and
/// fall along it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PinShape {
    /// Turned round.
    Round {
        /// Diameter, meters.
        diameter: f64,
    },
    /// Oval, its long axis along the key, its sides domed, with no flat on
    /// them. The sagittal section shows the length.
    ///
    /// **Only the part that shows is oval.** The shank driven into the rail
    /// is turned round, like any pin, and swells to the oval head at a
    /// shoulder sitting about at the rail's surface. This describes the head,
    /// which is the whole of what the key and the punchings ever touch — the
    /// shank is buried in wood and is not drawn.
    Oval {
        /// Extent along the key, meters: the oval's long axis.
        length: f64,
        /// Extent across the keyboard, meters: the oval's short axis.
        thickness: f64,
    },
}

impl PinShape {
    /// Extent along the key, meters — what a sagittal section shows.
    pub fn length(&self) -> f64 {
        match *self {
            PinShape::Round { diameter } => diameter,
            PinShape::Oval { length, .. } => length,
        }
    }

    /// Extent across the keyboard, meters.
    pub fn thickness(&self) -> f64 {
        match *self {
            PinShape::Round { diameter } => diameter,
            PinShape::Oval { thickness, .. } => thickness,
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
/// profile and a different contact, and it is not modeled here yet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Crown {
    /// Half-length of the flat on top, meters. Smaller than the rail's own.
    pub half_length: f64,
    /// Height of the trapezoid above the shoulder, meters.
    pub height: f64,
}

/// What one rail is, as a part: its section, its pin and its punchings.
///
/// Separate from [`FrameDesign`] because these are what the *parts* are, while
/// the design holds what the *regulation* sets. A technician never changes a
/// rail's width; he changes what is stacked on it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RailSection {
    /// Half-length of the rail along the key, meters.
    pub half_length: f64,
    /// The trapezoid on top, if the rail has one.
    pub crown: Option<Crown>,
    /// The pin standing in it, if any.
    pub pin: Option<PinShape>,
    /// How far the pin stands above the top of the **bare rail**, meters.
    ///
    /// Measured from the wood, not from the punchings, because the wood is
    /// what a depth gauge reaches and because the punching stack changes with
    /// every regulation — a pin described relative to it would appear to grow
    /// when the key is leveled. How far the pin reaches into the key's
    /// mortise is this, less whatever is stacked underneath.
    ///
    /// **The mortise bounds it, and the bound is read at the bottom of the
    /// key's travel, not at rest.** At rest the key's front hangs clear of its
    /// punching — that gap *is* the key dip — so the pin is at its shallowest
    /// in the mortise. Pressing the key down drives the mortise onto the pin.
    /// The constraint is therefore
    ///
    /// ```text
    /// pin_height ≤ punching stack + groove depth + mortise depth above it
    /// ```
    ///
    /// with every term measured from the rail's wood upward. On the measured
    /// frame that reads 4 + 3 + 19 = 26 mm against a pin standing 26 mm proud:
    /// the tip touches the top of the mortise exactly at the bottom of the
    /// key's travel, which is too fine to build. Driving the pin 2 mm past its
    /// shoulder gives 24 mm proud and 2 mm of headroom, and that is the value
    /// below.
    pub pin_height: f64,
    /// Outer radius of the punching washers, meters.
    pub punching_radius: f64,
    /// Radius of the hole through the punchings, meters.
    ///
    /// Measured rather than derived from the pin, because a punching is pierced
    /// with a **round** die: the hole is round even where the pin is not, and
    /// its diameter is a stock size and not a clearance anybody chose. On the
    /// measured frame the front rail's 5 mm hole is *exactly* its oval pin's
    /// long axis — the circle touches the pin at its two ends, which is the
    /// design and not a coincidence.
    pub punching_hole_radius: f64,
    /// Distance along the key from the natural row to the sharp row, meters.
    /// Positive is away from the player.
    pub sharp_row_offset: f64,
}

impl RailSection {
    /// Where this rail's pin stands for a key of this kind, meters.
    pub fn pin_x(&self, natural_x: f64, kind: KeyKind) -> f64 {
        match kind {
            KeyKind::Natural => natural_x,
            KeyKind::Sharp => natural_x - self.sharp_row_offset,
        }
    }

    /// The middle of the rail along the key, meters.
    ///
    /// A rail is centered **between** its two rows, not on either of them, so
    /// the front rail reaches back under the key rather than sitting
    /// symmetrically beneath the natural's pin. Where a rail serves one row —
    /// the back rail, whose cloth every key tail meets at the same place — this
    /// is the bearing itself.
    pub fn center_x(&self, natural_x: f64) -> f64 {
        natural_x - 0.5 * self.sharp_row_offset
    }

    /// The smallest half-length that still carries both rows, meters.
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
    /// The front rail: a plain block, oval pin.
    pub front: RailSection,
    /// The back rail: a plain block, no pin, cloth along its whole length.
    pub back: RailSection,
}

/// The punching papers a drawer holds: their thicknesses, meters, thinnest
/// first, each a whole number of [`PAPER_STEP`]s.
///
/// Key height and key dip are not continuous quantities on a real bench: they
/// move in these steps, and a target between two of them is not reachable.
/// The bench shims with these and reports what is left over rather than
/// hiding it.
///
/// Thickness is also how a paper punching is **identified**: a drawer is
/// sorted by color and the color *is* the thickness. That is why the
/// geometry builds one part per paper rather than one block of the right
/// total height. Which thicknesses a drawer holds is the stock a technician
/// buys, data and not code: an instrument's profile binds them to their
/// record.
#[derive(Debug, Clone, PartialEq)]
pub struct Drawer {
    /// Thinnest first, each a whole number of steps: held by
    /// [`Drawer::graded`], the only way to build one.
    thicknesses: Vec<f64>,
}

/// Why a list of thicknesses is no drawer the exact search can lay from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotADrawer {
    /// It holds no paper.
    Empty,
    /// A thickness that is not a number, or not more than nothing, meters.
    NotAThickness(f64),
    /// A thickness that is not a whole number of [`PAPER_STEP`]s, meters: the
    /// search through the drawer is exact only while every paper is one.
    NotGraded(f64),
    /// A thickness listed after one as thick or thicker, meters.
    NotThinnestFirst(f64),
}

impl Drawer {
    /// A drawer holding these thicknesses, meters; or why they are none the
    /// exact search can lay from: listed thinnest first, each a whole
    /// number of [`PAPER_STEP`]s and more than nothing.
    pub fn graded(thicknesses: Vec<f64>) -> Result<Self, NotADrawer> {
        if thicknesses.is_empty() {
            return Err(NotADrawer::Empty);
        }
        for &t in &thicknesses {
            if !t.is_finite() || t <= 0.0 {
                return Err(NotADrawer::NotAThickness(t));
            }
            let steps = t / PAPER_STEP;
            if (steps - steps.round()).abs() >= 1e-6 {
                return Err(NotADrawer::NotGraded(t));
            }
        }
        if let Some(w) = thicknesses.windows(2).find(|w| w[0] >= w[1]) {
            return Err(NotADrawer::NotThinnestFirst(w[1]));
        }
        Ok(Self { thicknesses })
    }

    /// The thicknesses it holds, meters, thinnest first.
    pub fn thicknesses(&self) -> &[f64] {
        &self.thicknesses
    }

    /// Its thinnest paper, meters: what key height and dip are adjusted by,
    /// and read to.
    pub fn thinnest(&self) -> f64 {
        self.thicknesses[0]
    }

    /// Whether it holds a paper of exactly this thickness, to the bit.
    pub fn holds(&self, thickness: f64) -> bool {
        self.thicknesses
            .iter()
            .any(|t| t.to_bits() == thickness.to_bits())
    }

    /// Whether `paper` is one it holds, a paper being one of them
    /// ([`PAPER`]); or why not.
    pub fn admits(&self, paper: f64) -> Result<(), Problem> {
        PAPER.admits(paper)?;
        if !self.holds(paper) {
            return Err(Problem::NotInTheDrawer {
                param: PAPER.name,
                thickness: paper,
            });
        }
        Ok(())
    }

    /// Builds the stack of its papers that comes closest to a wanted
    /// thickness without overshooting — thickest first, which is how a pile
    /// is laid.
    ///
    /// **Not greedy, and the difference is physical.** Taking the thickest
    /// paper that fits and repeating is correct only for a drawer whose
    /// sizes divide one another. The owner's does not: 0.24 mm is two pastel
    /// greens *exactly*, while greed reaches for a sky blue and strands
    /// 0.04 mm that nothing in the drawer can fill. A bench that reported
    /// that leftover would be blaming the drawer for its own arithmetic — and
    /// key height is regulated in precisely those hundredths. So the search
    /// is exhaustive: the closest total from below, and among those, the
    /// fewest papers.
    ///
    /// What is left over after that *is* a genuine miss. 0.37 mm is
    /// unreachable from that drawer because every paper thin enough to help
    /// is an even number of hundredths; the honest answer is 0.36 and a
    /// stated 0.01 mm short, not a rounded 0.37.
    ///
    /// **From below, always.** The wanted thickness is floored to the
    /// drawer's step, never rounded: 0.395 mm asks for 0.39 and gets 0.36,
    /// where rounding would have laid 0.40, five thousandths over what was
    /// asked. The regulation solve relies on it: a stack laid from below
    /// lands a key height under its target and a dip past it, each by less
    /// than one paper ([`crate::regulation`]).
    pub fn papers_for(&self, total: f64) -> Vec<f64> {
        if !total.is_finite() || total <= 0.0 || total > PAPER_SEARCH_LIMIT {
            return Vec::new();
        }
        let steps: Vec<usize> = self
            .thicknesses
            .iter()
            .map(|&t| (t / PAPER_STEP).round() as usize)
            .collect();
        let target = ((total + PAPER_EPSILON) / PAPER_STEP).floor() as usize;

        // Fewest papers that reach exactly `n` steps, and which paper to peel
        // off the top of that stack to get there. `usize::MAX` means nothing
        // reaches it.
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
            stack.push(self.thicknesses[i]);
            n -= steps[i];
        }
        stack.sort_by(|a, b| b.total_cmp(a));
        stack
    }
}

/// A paper laid on a punching: more than nothing; laid and lifted one at a
/// time, a stack being a pile and not a thickness. Which thicknesses are
/// papers is the drawer's to say ([`Drawer::admits`]).
pub const PAPER: ParamSpec<f64> = ParamSpec {
    name: "punching.paper",
    quantity: Quantity::Length,
    hard: (0.0, f64::INFINITY),
    step: Step::Drawer,
    layer: Layer::Regulation,
    kind: Kind::Actuated,
    get: |p| *p,
    set: |p, v| *p = v,
};

/// The step a drawer is graded in: 0.01 mm.
///
/// Every thickness a [`Drawer`] holds is a whole number of these, which is
/// what lets [`Drawer::papers_for`] search it *exactly* rather than
/// approximately.
pub const PAPER_STEP: f64 = 0.000_01;

/// The tallest stack worth searching for, meters.
///
/// Beyond this the design is wrong and not the drawer, and the regulation
/// solve has already refused it ([`crate::regulation::STACK_SEARCH`]); this
/// only stops a wild number from sizing a table.
pub(crate) const PAPER_SEARCH_LIMIT: f64 = 0.100;

/// How far short of a whole number of [`PAPER_STEP`]s a wanted thickness may
/// fall and still count as that number, meters: a picometer.
///
/// A sum of papers carries the rounding of its additions, a few parts in
/// 10¹⁶ of a millimeter, and a floor taken without this would read 0.70 mm
/// written as 0.6999… as 0.69 mm and lose a whole step. A picometer is ten
/// million times less than the step, so no thickness a technician can tell
/// apart from the next step is ever rounded up to it.
pub(crate) const PAPER_EPSILON: f64 = 1e-12;

/// The numbers that place one rail's parts: its height and its length, its
/// crown, its punching's felt and sizes, and its pin.
struct RailNames {
    height: PlacedBy,
    half_length: PlacedBy,
    crown_height: PlacedBy,
    felt: PlacedBy,
    punching_radius: PlacedBy,
    punching_hole_radius: PlacedBy,
    pin_diameter: PlacedBy,
    pin_length: PlacedBy,
    pin_height: PlacedBy,
}

const BALANCE: RailNames = RailNames {
    height: "balance_rail.height",
    half_length: "balance_rail.half_length",
    crown_height: "balance_rail.crown.height",
    felt: "balance_punching.felt",
    punching_radius: "balance_rail.punching_radius",
    punching_hole_radius: "balance_rail.punching_hole_radius",
    pin_diameter: "balance_rail.pin.diameter",
    pin_length: "balance_rail.pin.length",
    pin_height: "balance_rail.pin_height",
};

const FRONT: RailNames = RailNames {
    height: "front_rail.height",
    half_length: "front_rail.half_length",
    crown_height: "front_rail.crown.height",
    felt: "front_punching.felt",
    punching_radius: "front_rail.punching_radius",
    punching_hole_radius: "front_rail.punching_hole_radius",
    pin_diameter: "front_rail.pin.diameter",
    pin_length: "front_rail.pin.length",
    pin_height: "front_rail.pin_height",
};

const BACK: RailNames = RailNames {
    height: "back_rail.height",
    half_length: "back_rail.half_length",
    crown_height: "back_rail.crown.height",
    felt: "back_rail.cloth",
    punching_radius: "back_rail.punching_radius",
    punching_hole_radius: "back_rail.punching_hole_radius",
    pin_diameter: "back_rail.pin.diameter",
    pin_length: "back_rail.pin.length",
    pin_height: "back_rail.pin_height",
};

/// Each segment of an outline placed by its own number, in order.
fn each(names: &[PlacedBy]) -> Vec<(core::ops::Range<u16>, PlacedBy)> {
    (0u16..).zip(names).map(|(i, &n)| (i..i + 1, n)).collect()
}

/// What the frame's parts declare: where each is, and the ranges of them
/// that anything else may read.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameRanges {
    /// The back rail.
    pub back_rail: PartId,
    /// The balance rail, its crown included.
    pub balance_rail: PartId,
    /// The front rail.
    pub front_rail: PartId,
    /// The back rail cloth.
    pub back_rail_cloth: PartId,
    /// The papers under the balance punching, bottom first.
    pub balance_papers: Vec<PartId>,
    /// The balance punching's felt.
    pub balance_rail_punching: PartId,
    /// The papers under the front punching, bottom first.
    pub front_papers: Vec<PartId>,
    /// The front punching's felt.
    pub front_rail_punching: PartId,
    /// The balance pin, where the rail has one.
    pub balance_rail_pin: Option<PartId>,
    /// The front rail pin, where the rail has one.
    pub front_rail_pin: Option<PartId>,
    /// The balance punching's top, across its whole width: what the key bears
    /// on at its pin.
    pub balance_bearing: SurfaceRef,
    /// The front punching's top, across its whole width.
    pub front_bearing: SurfaceRef,
    /// The cloth's top, from the front: its front edge, the face, its back
    /// edge.
    pub cloth_top: SurfaceRef,
    /// The cloth's top and the two sides its rounded edges lead into, from
    /// the front side round to the back: what the key's underside meets. The
    /// sides are there so that a corner driven deeper than an edge's center
    /// still stands over a face the contact reads.
    pub cloth_and_sides: SurfaceRef,
    /// The front rail pin's top, where the rail has one.
    pub front_pin_top: Option<SurfaceRef>,
}

/// A rectangle's outline, counter-clockwise from its bottom left corner.
fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<Seg> {
    super::polygon(&[
        Point::new(x0, y0),
        Point::new(x1, y0),
        Point::new(x1, y1),
        Point::new(x0, y1),
    ])
}

/// A washer's profile in radius and height, counter-clockwise, its inner side
/// last: the wall of the hole the pin passes through. A disc with no hole
/// would swallow its own pin.
fn washer(r_in: f64, r_out: f64, y0: f64, y1: f64) -> Vec<Seg> {
    super::polygon(&[
        Point::new(r_in, y0),
        Point::new(r_out, y0),
        Point::new(r_out, y1),
        Point::new(r_in, y1),
    ])
}

/// The top of a turned part's silhouette, given the profile's number of
/// segments: the two segments either side of the axis ([`Part::silhouette`]).
fn top_of_turned(part: PartId, segments: u16) -> SurfaceRef {
    SurfaceRef {
        part,
        segs: segments - 2..segments,
    }
}

impl FrameDesign {
    /// Builds every part of the frame onto `out`, on the ground, in the keybed
    /// frame: `x` toward the player from the balance pin, `y` up from the
    /// keybed. Returns what they declare.
    ///
    /// Ordered back to front, the order a technician names them. `key`
    /// supplies the width the rails are drawn across; `kind`, which row of
    /// pins and punchings the key stands on. Each punching's papers are drawn
    /// one part each, as the stack holds them, bottom first, because two
    /// stacks of one height are not one object on a bench: a technician
    /// counts papers.
    ///
    /// A punching's felt is drawn from the top of its papers up to its
    /// bearing height ([`Self::balance_bearing_height`],
    /// [`Self::front_bearing_height`]), and the cloth's face to the back
    /// bearing's, its edges rounded to
    /// [`Self::back_rail_cloth_edge_radius`].
    pub fn parts(&self, key: &super::KeyDesign, kind: KeyKind, out: &mut Vec<Part>) -> FrameRanges {
        let s = &self.sections;
        let mut push = |part: Part| {
            let id = super::next_id(out);
            out.push(part);
            id
        };
        let whole = |outline: &[Seg], name: PlacedBy| vec![(0..super::count(outline), name)];

        // The rails. A rail is continuous across the keyboard, so a single-key
        // bench shows a slice a little wider than the key: enough to read as
        // "this goes on", not so much that it hides the key. The slice has to
        // beat the longest rail along the key as well, or a 75 mm front rail
        // cut to a 40 mm slice reads as a post standing under the key.
        let longest = s
            .balance
            .half_length
            .max(s.front.half_length)
            .max(s.back.half_length);
        let slice = (key.width * 1.8).max(2.4 * longest);
        let mut rail = |code, natural_x: f64, top: f64, sec: &RailSection, n: &RailNames| {
            // Centered between its two rows: a rail wide enough for both is not
            // symmetric about either.
            let x = sec.center_x(natural_x);
            let h = sec.half_length;
            let (outline, placed_by) = match sec.crown {
                None => (
                    rect(x - h, 0.0, x + h, top),
                    each(&[n.height, n.half_length, n.height, n.half_length]),
                ),
                Some(crown) => {
                    let shoulder = top - crown.height;
                    let c = crown.half_length;
                    (
                        super::polygon(&[
                            Point::new(x - h, 0.0),
                            Point::new(x + h, 0.0),
                            Point::new(x + h, shoulder),
                            Point::new(x + c, top),
                            Point::new(x - c, top),
                            Point::new(x - h, shoulder),
                        ]),
                        each(&[
                            n.height,
                            n.half_length,
                            n.crown_height,
                            n.height,
                            n.crown_height,
                            n.half_length,
                        ]),
                    )
                }
            };
            push(Part {
                body: BodyId::GROUND,
                code,
                placed_by,
                outline,
                sweep: Sweep::Extruded {
                    width: slice,
                    across: 0.0,
                },
                role: Role::Solid,
                material: Material::RailWood,
            })
        };
        let back_rail = rail(
            PartCode::BackRail,
            self.back_rail,
            self.back_rail_height,
            &s.back,
            &BACK,
        );
        let balance_rail = rail(
            PartCode::BalanceRail,
            0.0,
            self.balance_rail_height,
            &s.balance,
            &BALANCE,
        );
        let front_rail = rail(
            PartCode::FrontRail,
            self.front_rail,
            self.front_rail_height,
            &s.front,
            &FRONT,
        );

        // The cloth: a strip the length of the rail, its two top edges rounded,
        // the arcs centered under the face's two ends. Each line takes its ends
        // from the arcs beside it, so the outline closes bit for bit.
        let center = s.back.center_x(self.back_rail);
        let (x0, x1) = (center - s.back.half_length, center + s.back.half_length);
        let (bottom, top) = (self.back_rail_height, self.back_bearing_height());
        let r = self.back_rail_cloth_edge_radius;
        let front_edge = Seg::Arc {
            c: Point::new(x1, top - r),
            r,
            from: Dir::X,
            to: Dir::Y,
        };
        let back_edge = Seg::Arc {
            c: Point::new(x0, top - r),
            r,
            from: Dir::Y,
            to: Dir::NEG_X,
        };
        let (front_low, back_low) = (front_edge.start(), back_edge.end());
        let cloth_outline = vec![
            Seg::Line {
                a: Point::new(back_low.x, bottom),
                b: Point::new(front_low.x, bottom),
            },
            Seg::Line {
                a: Point::new(front_low.x, bottom),
                b: front_low,
            },
            front_edge,
            Seg::Line {
                a: front_edge.end(),
                b: back_edge.start(),
            },
            back_edge,
            Seg::Line {
                a: back_low,
                b: Point::new(back_low.x, bottom),
            },
        ];
        let back_rail_cloth = push(Part {
            body: BodyId::GROUND,
            code: PartCode::BackRailCloth,
            placed_by: each(&[
                BACK.height,
                BACK.half_length,
                "back_rail.cloth_edge_radius",
                BACK.felt,
                "back_rail.cloth_edge_radius",
                BACK.half_length,
            ]),
            outline: cloth_outline,
            sweep: Sweep::Extruded {
                width: slice,
                across: 0.0,
            },
            role: Role::Lining,
            material: Material::Cloth,
        });
        let cloth_top = SurfaceRef {
            part: back_rail_cloth,
            segs: 2..5,
        };
        let cloth_and_sides = SurfaceRef {
            part: back_rail_cloth,
            segs: 1..6,
        };

        // The punching stacks: papers first, each its own washer turned about
        // the pin, then the felt, drawn up to the stack's bearing height. A
        // washer's rim is measured and its thickness is not.
        let mut stack = |rail_top: f64,
                         axis_x: f64,
                         sec: &RailSection,
                         papers: &[f64],
                         bearing: f64,
                         felt_code,
                         n: &RailNames| {
            let (r_in, r_out) = (sec.punching_hole_radius, sec.punching_radius);
            let mut laid = Vec::with_capacity(papers.len());
            let mut y = rail_top;
            for &t in papers {
                let outline = washer(r_in, r_out, y, y + t);
                laid.push(push(Part {
                    body: BodyId::GROUND,
                    code: PartCode::PaperPunching,
                    placed_by: whole(&outline, PAPER.name),
                    outline,
                    sweep: Sweep::Revolved { axis_x },
                    role: Role::Lining,
                    material: Material::Paper { thickness: t },
                }));
                y += t;
            }
            let outline = washer(r_in, r_out, y, bearing);
            let segments = super::count(&outline);
            let felt = push(Part {
                body: BodyId::GROUND,
                code: felt_code,
                placed_by: each(&[n.felt, n.punching_radius, n.felt, n.punching_hole_radius]),
                outline,
                sweep: Sweep::Revolved { axis_x },
                role: Role::Lining,
                material: Material::PunchingFelt,
            });
            (laid, felt, top_of_turned(felt, segments))
        };
        let balance_x = s.balance.pin_x(0.0, kind);
        let front_x = s.front.pin_x(self.front_rail, kind);
        let (balance_papers, balance_rail_punching, balance_bearing) = stack(
            self.balance_rail_height,
            balance_x,
            &s.balance,
            &self.balance_punching.papers,
            self.balance_bearing_height(),
            PartCode::BalanceRailPunching,
            &BALANCE,
        );
        let (front_papers, front_rail_punching, front_bearing) = stack(
            self.front_rail_height,
            front_x,
            &s.front,
            &self.front_punching.papers,
            self.front_bearing_height(),
            PartCode::FrontRailPunching,
            &FRONT,
        );

        // The pins. They guide the key and carry no vertical load, which is
        // why the bearing is the punching and not the pin; they are drawn
        // because a reader checks that the key is on its pins. A round pin's
        // height is assumed, an oval one's measured.
        let balance_rail_pin = s.balance.pin.map(|pin| {
            let (bottom, top) = (
                self.balance_rail_height,
                self.balance_rail_height + s.balance.pin_height,
            );
            pin_part(
                pin,
                balance_x,
                bottom,
                top,
                PartCode::BalanceRailPin,
                &BALANCE,
                &mut push,
            )
        });
        let front_rail_pin = s.front.pin.map(|pin| {
            let (bottom, top) = (
                self.front_rail_height,
                self.front_rail_height + s.front.pin_height,
            );
            pin_part(
                pin,
                front_x,
                bottom,
                top,
                PartCode::FrontRailPin,
                &FRONT,
                &mut push,
            )
        });
        FrameRanges {
            back_rail,
            balance_rail,
            front_rail,
            back_rail_cloth,
            balance_papers,
            balance_rail_punching,
            front_papers,
            front_rail_punching,
            balance_rail_pin: balance_rail_pin.map(|(id, _)| id),
            front_rail_pin: front_rail_pin.as_ref().map(|&(id, _)| id),
            balance_bearing,
            front_bearing,
            cloth_top,
            cloth_and_sides,
            front_pin_top: front_rail_pin.map(|(part, segs)| SurfaceRef { part, segs }),
        }
    }
}

/// Where a pin's tip, rounded to `radius` on a pin `height` tall, leaves the
/// pin: how far out from the axis, and how far under the tip's center. A
/// pin taller than the radius has its tip leave its side level with the
/// center; one no taller is all tip, a cap of that sphere, whose foot
/// stands under the center by what the radius has over the height.
fn tip_start(height: f64, radius: f64) -> (f64, f64) {
    let drop = if radius > height {
        radius - height
    } else {
        0.0
    };
    (super::end_reach(height, radius), drop)
}

/// A round pin's profile, its tip rounded as the owner describes a balance
/// pin's, to the pin's own radius, which the model assumes: from its axis at
/// its foot out to its side, up
/// its side to the tip, over the tip to the axis at its top, and down the
/// axis. A pin no taller than its radius is all tip, a cap of that sphere.
fn profile_round_pin(radius: f64, bottom: f64, top: f64) -> Vec<Seg> {
    let height = top - bottom;
    let (reach, drop) = tip_start(height, radius);
    let foot = Point::new(0.0, bottom);
    let Some(from) = Dir::new(reach, drop) else {
        return washer(0.0, radius, bottom, top);
    };
    let tip = Seg::Arc {
        c: Point::new(0.0, top - radius),
        r: radius,
        from,
        to: Dir::Y,
    };
    let mut profile = Vec::with_capacity(4);
    if radius < height {
        let side = Point::new(radius, bottom);
        profile.push(Seg::Line { a: foot, b: side });
        profile.push(Seg::Line {
            a: side,
            b: tip.start(),
        });
    } else {
        profile.push(Seg::Line {
            a: foot,
            b: tip.start(),
        });
    }
    profile.push(tip);
    profile.push(Seg::Line {
        a: tip.end(),
        b: foot,
    });
    profile
}

/// An oval pin's section along the key, through its middle at `x`, its
/// tip rounded as a round pin's is, to a half sphere of half its `length`,
/// its top at `top`: from its back foot along its foot, up its front, over
/// its tip and down its back; with the range of it that is its tip. A pin
/// no taller than that radius is all tip, a cap of that sphere.
fn outline_oval_pin(
    x: f64,
    length: f64,
    bottom: f64,
    top: f64,
) -> (Vec<Seg>, core::ops::Range<u16>) {
    let radius = 0.5 * length;
    let height = top - bottom;
    let (reach, drop) = tip_start(height, radius);
    let (Some(from_front), Some(to_back)) = (Dir::new(reach, drop), Dir::new(-reach, drop)) else {
        return (rect(x - radius, bottom, x + radius, top), 2..3);
    };
    let center = Point::new(x, top - radius);
    let front = Seg::Arc {
        c: center,
        r: radius,
        from: from_front,
        to: Dir::Y,
    };
    let back = Seg::Arc {
        c: center,
        r: radius,
        from: Dir::Y,
        to: to_back,
    };
    let has_sides = radius < height;
    let (back_foot, front_foot) = if has_sides {
        (
            Point::new(x - radius, bottom),
            Point::new(x + radius, bottom),
        )
    } else {
        (back.end(), front.start())
    };
    let mut outline = vec![Seg::Line {
        a: back_foot,
        b: front_foot,
    }];
    if has_sides {
        outline.push(Seg::Line {
            a: front_foot,
            b: front.start(),
        });
    }
    let tip = super::count(&outline);
    outline.extend([front, back]);
    if has_sides {
        outline.push(Seg::Line {
            a: back.end(),
            b: back_foot,
        });
    }
    (outline, tip..tip + 2)
}

/// Which number places each segment of a pin's outline: its rounded tip the
/// pin's height, a level line its rail's height, any other line `side`, the
/// pin's diameter or its length along the key.
fn place_the_pin(
    outline: &[Seg],
    n: &RailNames,
    side: PlacedBy,
) -> Vec<(core::ops::Range<u16>, PlacedBy)> {
    (0u16..)
        .zip(outline)
        .map(|(index, seg)| {
            let name = match seg {
                Seg::Arc { .. } => n.pin_height,
                Seg::Line { a, b } if a.y.to_bits() == b.y.to_bits() => n.height,
                Seg::Line { .. } => side,
            };
            (index..index + 1, name)
        })
        .collect()
}

/// A pin, turned if it is round and swept oval if it is oval, pushed by
/// `push`; with the range of its silhouette that is its top.
fn pin_part(
    pin: PinShape,
    x: f64,
    bottom: f64,
    top: f64,
    code: PartCode,
    n: &RailNames,
    push: &mut impl FnMut(Part) -> PartId,
) -> (PartId, core::ops::Range<u16>) {
    match pin {
        PinShape::Round { diameter } => {
            let outline = profile_round_pin(0.5 * diameter, bottom, top);
            let segments = super::count(&outline);
            let placed_by = place_the_pin(&outline, n, n.pin_diameter);
            let id = push(Part {
                body: BodyId::GROUND,
                code,
                placed_by,
                outline,
                sweep: Sweep::Revolved { axis_x: x },
                role: Role::Solid,
                material: Material::Steel,
            });
            (id, top_of_turned(id, segments).segs)
        }
        PinShape::Oval { length, thickness } => {
            let (outline, tip) = outline_oval_pin(x, length, bottom, top);
            let placed_by = place_the_pin(&outline, n, n.pin_length);
            // A pin no taller than its tip's radius is a cap of the squeezed
            // sphere, its foot shorter along the key than the oval, and as
            // much narrower across.
            let radius = 0.5 * length;
            let (foot, _) = tip_start(top - bottom, radius);
            let width = if foot < radius {
                thickness * foot / radius
            } else {
                thickness
            };
            let id = push(Part {
                body: BodyId::GROUND,
                code,
                placed_by,
                outline,
                sweep: Sweep::Oval { width },
                role: Role::Solid,
                material: Material::Steel,
            });
            (id, tip)
        }
    }
}

#[cfg(test)]
mod drawer_tests {
    use super::*;
    use crate::test_fixture::PAPER_THICKNESSES;

    /// The owner's drawer's stack for `total` meters of paper.
    fn papers_for(total: f64) -> Vec<f64> {
        crate::test_fixture::drawer().papers_for(total)
    }

    /// Consistency: **the search through the drawer is exact only while
    /// every paper is a whole number of steps, listed thinnest first**, so a
    /// drawer is built only so: the owner's is, and a list that breaks
    /// either is refused, naming the paper that does.
    #[test]
    fn a_drawer_is_graded_in_hundredths_and_listed_thinnest_first() {
        assert_eq!(
            crate::test_fixture::drawer().thicknesses(),
            &PAPER_THICKNESSES[..]
        );
        assert_eq!(Drawer::graded(Vec::new()), Err(NotADrawer::Empty));
        assert_eq!(
            Drawer::graded(vec![0.00008, 0.000125]),
            Err(NotADrawer::NotGraded(0.000125))
        );
        assert_eq!(
            Drawer::graded(vec![0.0002, 0.00012]),
            Err(NotADrawer::NotThinnestFirst(0.00012))
        );
        assert_eq!(
            Drawer::graded(vec![0.0002, 0.0002]),
            Err(NotADrawer::NotThinnestFirst(0.0002))
        );
        for bad in [0.0, -0.0001, f64::NAN, f64::INFINITY] {
            assert!(
                matches!(Drawer::graded(vec![bad]), Err(NotADrawer::NotAThickness(t)) if t.to_bits() == bad.to_bits()),
                "{bad} m taken as a paper"
            );
        }
    }

    /// Consistency: **a paper the drawer does not hold is refused**, to the
    /// bit, and so is one that is no thickness at all; one it holds is
    /// admitted.
    #[test]
    fn a_paper_the_drawer_does_not_hold_is_refused() {
        let drawer = crate::test_fixture::drawer();
        assert_eq!(
            drawer.admits(0.0003),
            Err(Problem::NotInTheDrawer {
                param: PAPER.name,
                thickness: 0.0003,
            })
        );
        assert_eq!(
            drawer.admits(f64::NAN),
            Err(Problem::NotFinite { param: PAPER.name })
        );
        for &t in &PAPER_THICKNESSES {
            assert_eq!(drawer.admits(t), Ok(()));
        }
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
    /// that exist: over whole hundredths, and over thicknesses between them,
    /// which a regulation asks for and which rounding to the nearest step
    /// would overshoot by up to half a step.
    ///
    /// The bound is the floor's own epsilon, a picometer.
    #[test]
    fn a_stack_is_laid_thickest_first_and_never_overshoots() {
        for target in [
            0.0,
            0.00005,
            0.0001,
            0.00037,
            0.0005,
            0.0012,
            0.003,
            0.0075,
            0.000395,
            0.000245,
            0.000_899_9,
            0.001_234_5,
            0.000_079_9,
            0.002_005,
        ] {
            let stack = papers_for(target);
            let total: f64 = stack.iter().sum();
            assert!(
                total <= target + PAPER_EPSILON,
                "{target} m overshot: {stack:?}"
            );
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

    /// The audit's RUST-18: 0.395 mm, between two hundredths, is floored to
    /// 0.39 mm, which the drawer reaches no closer than 0.36; rounding to the
    /// nearest step laid 0.40, over what was asked.
    #[test]
    fn a_thickness_between_two_steps_is_taken_from_below() {
        let total: f64 = papers_for(0.000395).iter().sum();
        assert!((total - 0.00036).abs() < 1e-12, "laid {total} m");
        let total: f64 = papers_for(0.000245).iter().sum();
        assert!((total - 0.00024).abs() < 1e-12, "laid {total} m");
    }

    /// A sum of papers, carrying the rounding of its additions, gives the
    /// same papers back: the floor's epsilon keeps a total written a hair
    /// under its whole number of steps from losing one.
    #[test]
    fn a_sum_of_papers_is_laid_again_as_the_same_papers() {
        for stack in [
            vec![0.0005, 0.0002],
            vec![0.0005, 0.0004],
            vec![0.0004, 0.0004, 0.00008],
            vec![
                0.00125, 0.001, 0.00075, 0.0005, 0.0004, 0.0002, 0.00012, 0.00008,
            ],
        ] {
            let total: f64 = stack.iter().sum();
            let again: f64 = papers_for(total).iter().sum();
            assert!(
                (again - total).abs() < 1e-12,
                "{stack:?} summed to {total} m, laid again as {again} m"
            );
        }
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
mod part_tests {
    use super::*;
    use crate::parts::KeyDesign;

    /// The frame's parts, its papers drawn under both felts.
    fn laid(frame: &FrameDesign, kind: KeyKind) -> (Vec<Part>, FrameRanges) {
        let mut parts = Vec::new();
        let ranges = frame.parts(&crate::test_fixture::key(), kind, &mut parts);
        (parts, ranges)
    }

    /// The points an outline is drawn through, arcs cut in eight.
    fn drawn(part: &Part) -> Vec<Point> {
        part.outline.iter().flat_map(|s| s.drawn(3)).collect()
    }

    /// The extent of what is drawn: `(x_min, x_max)` and `(y_min, y_max)`, in
    /// the outline's own coordinates (radius and height for a turned part).
    fn extent(part: &Part) -> ((f64, f64), (f64, f64)) {
        drawn(part).iter().fold(
            ((f64::MAX, f64::MIN), (f64::MAX, f64::MIN)),
            |((x0, x1), (y0, y1)), p| ((x0.min(p.x), x1.max(p.x)), (y0.min(p.y), y1.max(p.y))),
        )
    }

    fn axis(part: &Part) -> f64 {
        match part.sweep {
            Sweep::Revolved { axis_x } => axis_x,
            Sweep::Extruded { .. } | Sweep::Milled { .. } | Sweep::Oval { .. } => {
                let ((x0, x1), _) = extent(part);
                0.5 * (x0 + x1)
            }
        }
    }

    /// Every part the frame carries is there, each with its code, and papers
    /// under both punchings.
    #[test]
    fn the_frame_carries_its_rails_pins_punchings_and_cloth() {
        let (parts, ranges) = laid(&crate::test_fixture::frame(), KeyKind::Natural);
        for papers in [&ranges.balance_papers, &ranges.front_papers] {
            assert!(!papers.is_empty(), "a punching stands on its papers");
            for id in papers {
                assert_eq!(parts[id.index()].code, PartCode::PaperPunching);
            }
        }
        for code in [
            PartCode::BalanceRail,
            PartCode::BalanceRailPin,
            PartCode::BalanceRailPunching,
            PartCode::FrontRail,
            PartCode::FrontRailPin,
            PartCode::FrontRailPunching,
            PartCode::PaperPunching,
            PartCode::BackRail,
            PartCode::BackRailCloth,
        ] {
            assert!(
                parts.iter().any(|p| p.code == code),
                "the frame is missing its {}",
                code.code()
            );
        }
        assert!(parts.iter().all(|p| p.body == BodyId::GROUND));
    }

    /// Consistency: every part of the frame is closed bit for bit, turns
    /// counter-clockwise, and has a source for every segment.
    #[test]
    fn every_part_of_the_frame_is_closed_and_counter_clockwise() {
        let (parts, _) = laid(&crate::test_fixture::frame(), KeyKind::Natural);
        for part in &parts {
            assert!(part.is_closed(), "{} is open", part.code.code());
            let p = drawn(part);
            let twice_area: f64 = (0..p.len())
                .map(|i| p[i].x * p[(i + 1) % p.len()].y - p[(i + 1) % p.len()].x * p[i].y)
                .sum();
            assert!(twice_area > 0.0, "{} turns clockwise", part.code.code());
            let covered: u16 = part.placed_by.iter().map(|(r, _)| r.end - r.start).sum();
            assert_eq!(usize::from(covered), part.outline.len());
        }
    }

    /// The cloth is drawn with its rounded edges: two quarter circles of
    /// the frame's cloth edge radius, centered under the ends of its face, the face
    /// between them.
    #[test]
    fn the_cloth_is_drawn_with_its_rounded_edges() {
        let frame = crate::test_fixture::frame();
        let (parts, ranges) = laid(&frame, KeyKind::Natural);
        let top = ranges.cloth_top.segs(&parts);
        let r = frame.back_rail_cloth_edge_radius;
        let center = frame.sections.back.center_x(frame.back_rail);
        let half = frame.sections.back.half_length;
        match (top[0], top[1], top[2]) {
            (
                Seg::Arc {
                    c: front, r: rf, ..
                },
                Seg::Line { a, b },
                Seg::Arc { c: back, r: rb, .. },
            ) => {
                assert_eq!(rf.to_bits(), r.to_bits());
                assert_eq!(rb.to_bits(), r.to_bits());
                assert_eq!(front.x.to_bits(), (center + half).to_bits());
                assert_eq!(back.x.to_bits(), (center - half).to_bits());
                assert_eq!(a.x.to_bits(), (center + half).to_bits());
                assert_eq!(b.x.to_bits(), (center - half).to_bits());
            }
            other => panic!("the cloth's top is an edge, a face and an edge, not {other:?}"),
        }
    }

    /// Adding paper raises the felt that sits on it, by exactly what was added.
    ///
    /// The felt does not grow: it is lifted. That is the operation, and a model
    /// where the washer got thicker instead would teach the wrong thing. The
    /// felt is drawn up to its stack's bearing height, so its drawn
    /// thickness is the felt's within the roundings of that height's sum.
    #[test]
    fn adding_paper_lifts_the_felt_without_thickening_it() {
        let base = crate::test_fixture::frame();
        let mut raised = base.clone();
        raised.balance_punching.papers.push(0.0004);

        let felt_span = |f: &FrameDesign| {
            let (parts, ranges) = laid(f, KeyKind::Natural);
            let (_, (lo, hi)) = extent(&parts[ranges.balance_rail_punching.index()]);
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
        let frame = crate::test_fixture::frame();
        let (parts, ranges) = laid(&frame, KeyKind::Natural);
        assert_eq!(
            ranges.balance_papers.len(),
            frame.balance_punching.papers.len(),
            "one part per paper in the stack"
        );
        let mut y = frame.balance_rail_height;
        for id in &ranges.balance_papers {
            let paper = &parts[id.index()];
            let Material::Paper { thickness } = paper.material else {
                panic!("a paper is made of paper, not {:?}", paper.material);
            };
            let (_, (lo, hi)) = extent(paper);
            assert!((lo - y).abs() < 1e-15, "papers stack with no gap");
            assert!(
                (hi - lo - thickness).abs() < 1e-15,
                "a paper is drawn its own thickness"
            );
            y += thickness;
        }
        assert!(
            (y - frame.balance_rail_height - frame.balance_punching.paper()).abs() < 1e-12,
            "the stack sums to the shim the regulation asked for"
        );
    }

    /// A punching is a washer, and its hole admits the pin that passes through.
    ///
    /// Drawn as a disc it would swallow its own pin, and the reader would lose
    /// the one feature that says the key is located rather than merely resting.
    ///
    /// The bound is `>=`, not `>`, because at the front rail the measured hole
    /// is *exactly* the pin's long axis: a 5 mm circle round a 5 mm oval,
    /// touching at its two ends. Demanding a clearance there would reject the
    /// real part.
    #[test]
    fn a_punching_is_a_washer_whose_hole_admits_its_pin() {
        let frame = crate::test_fixture::frame();
        let (parts, ranges) = laid(&frame, KeyKind::Natural);
        for (id, sec) in [
            (ranges.balance_rail_punching, frame.sections.balance),
            (ranges.front_rail_punching, frame.sections.front),
        ] {
            let m = &parts[id.index()];
            assert!(
                matches!(m.sweep, Sweep::Revolved { .. }),
                "turned, not extruded"
            );
            let ((r_in, r_out), _) = extent(m);
            let pin = sec.pin.expect("this rail has a pin");
            assert!(
                r_in >= 0.5 * pin.length() - 1e-12,
                "{}: the hole must admit the pin's longest axis",
                m.code.code()
            );
            assert!(r_out > r_in, "a washer has a rim");
            assert!((r_out - sec.punching_radius).abs() < 1e-15);
        }
    }

    /// The balance rail is a block with a trapezoid on it, narrower on top.
    ///
    /// One piece, and the crown is what the key's balance mortise straddles.
    #[test]
    fn the_balance_rail_is_crowned_and_the_others_are_not() {
        let frame = crate::test_fixture::frame();
        let (parts, ranges) = laid(&frame, KeyKind::Natural);
        let rail = &parts[ranges.balance_rail.index()];
        let crown = frame.sections.balance.crown.expect("it is crowned");
        let width_at = |y: f64| {
            let xs: Vec<f64> = drawn(rail)
                .iter()
                .filter(|p| (p.y - y).abs() < 1e-12)
                .map(|p| p.x)
                .collect();
            xs.iter().fold(f64::MIN, |a, &x| a.max(x)) - xs.iter().fold(f64::MAX, |a, &x| a.min(x))
        };
        assert!((width_at(0.0) - 2.0 * frame.sections.balance.half_length).abs() < 1e-15);
        assert!((width_at(frame.balance_rail_height) - 2.0 * crown.half_length).abs() < 1e-15);
        assert!(width_at(frame.balance_rail_height) < width_at(0.0));
        for id in [ranges.front_rail, ranges.back_rail] {
            assert_eq!(parts[id.index()].outline.len(), 4, "a plain block");
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
        let sections = crate::test_fixture::sections();
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
                assert!(crown.half_length >= sec.minimum_half_length());
                assert!(crown.half_length < sec.half_length);
            }
        }

        // And the rail as drawn really does span them, which is what
        // `center_x` is for: the width alone would be satisfied by a rail
        // sitting in the wrong place.
        let frame = crate::test_fixture::frame();
        let (parts, ranges) = laid(&frame, KeyKind::Natural);
        for (id, sec, natural_x) in [
            (ranges.balance_rail, sections.balance, 0.0),
            (ranges.front_rail, sections.front, frame.front_rail),
        ] {
            let ((x0, x1), _) = extent(&parts[id.index()]);
            for kind in [KeyKind::Natural, KeyKind::Sharp] {
                let pin = sec.pin_x(natural_x, kind);
                assert!(
                    pin - sec.punching_radius >= x0 - 1e-12
                        && pin + sec.punching_radius <= x1 + 1e-12,
                    "the {kind:?} row's punching hangs off the wood"
                );
            }
        }
    }

    /// The test frame is the one the rule builds for the test key.
    ///
    /// Two expressions of the same frame, and they have to agree: the
    /// coordinates are stated in the measured section's profile, as the fixture
    /// repeats them, so a reader can see them, and derived in `placed_under`
    /// so a changed key can be given a frame. If they drifted, the bench
    /// would open on a keyboard its own rule calls wrong.
    #[test]
    fn the_default_frame_is_the_one_the_rule_builds_for_the_default_key() {
        let key = crate::test_fixture::key();
        let typed = crate::test_fixture::frame();
        let placed = crate::test_fixture::frame().placed_under(&key);
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
    /// The rule is tested on keys the test frame was not built for, because a
    /// relation that only holds at the value it was derived from is not a
    /// relation. A 300 mm key is a long natural; a 180 mm one is shorter than
    /// anything in a piano, and the rule still has to produce a frame that
    /// goes together. A key 40 mm shorter at the back is built for an action
    /// whose back check block stands right in front of its tail, as the
    /// test key's does; a longer one keeps the test key's block.
    #[test]
    fn placing_the_frame_follows_the_key_it_is_built_for() {
        for front_edge in [0.180_f64, 0.220, 0.258, 0.300] {
            for tail_end in [-0.200_f64, -0.240, -0.280] {
                let standard = crate::test_fixture::key();
                let key = KeyDesign {
                    front_edge,
                    tail_end,
                    back_check_block_back: standard
                        .back_check_block_back
                        .max(tail_end + standard.tail_length),
                    ..standard
                };
                let frame = crate::test_fixture::frame().placed_under(&key);
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
        let frame = crate::test_fixture::frame();
        let (natural, n) = laid(&frame, KeyKind::Natural);
        let (sharp, s) = laid(&frame, KeyKind::Sharp);
        let at = |parts: &[Part], id: Option<PartId>| axis(&parts[id.expect("a pin").index()]);
        assert!(
            (at(&natural, n.balance_rail_pin)
                - at(&sharp, s.balance_rail_pin)
                - frame.sections.balance.sharp_row_offset)
                .abs()
                < 1e-15,
            "the sharp's balance pin is set back by the row offset"
        );
        assert!(
            (at(&natural, n.front_rail_pin)
                - at(&sharp, s.front_rail_pin)
                - frame.sections.front.sharp_row_offset)
                .abs()
                < 1e-15,
            "and so is its front pin"
        );
        for (a, b) in [
            (n.balance_rail, s.balance_rail),
            (n.front_rail, s.front_rail),
            (n.back_rail, s.back_rail),
        ] {
            assert_eq!(
                natural[a.index()].outline,
                sharp[b.index()].outline,
                "one rail; only the row of pins differs"
            );
        }
    }

    /// Consistency: **a round pin's tip is rounded to the pin's radius**, its
    /// top at the pin's height: a pin of the measured section's 3.4 mm,
    /// 11.2 mm tall, runs straight up its side to 1.7 mm under its top, then
    /// over a half sphere of 1.7 mm. A pin a millimeter tall is all tip, a
    /// cap of that sphere, whose foot reaches √(2 · 1.7 · 1 − 1²) = √2.4 mm
    /// out from its axis.
    #[test]
    fn a_round_pin_is_drawn_with_its_tip_rounded_to_its_radius() {
        let radius = 0.0017;
        let (rail_height, pin_height) = (0.0266, 0.0112);
        let (bottom, top) = (rail_height, rail_height + pin_height);
        let profile = profile_round_pin(radius, bottom, top);
        let [Seg::Line { .. }, Seg::Line {
            a: side,
            b: shoulder,
        }, Seg::Arc { c, r, .. }, Seg::Line { .. }] = profile[..]
        else {
            panic!("{profile:?}");
        };
        assert_eq!((side.x, r), (radius, radius));
        assert_eq!(
            c.x.to_bits(),
            0.0_f64.to_bits(),
            "the tip is centered on the axis"
        );
        // One rounding in the center's height, one in the shoulder's.
        assert!((shoulder.y - (top - radius)).abs() <= 2.0 * f64::EPSILON * top);
        assert!((profile[2].end().y - top).abs() <= 2.0 * f64::EPSILON * top);
        let short = profile_round_pin(radius, bottom, bottom + 0.001);
        let [Seg::Line { b: foot, .. }, Seg::Arc { .. }, Seg::Line { .. }] = short[..] else {
            panic!("{short:?}");
        };
        let expected = 0.001 * 2.4_f64.sqrt();
        assert!(
            (foot.x - expected).abs() <= 4.0 * f64::EPSILON * radius,
            "{}",
            foot.x
        );
    }

    /// Consistency: **an oval pin's tip is rounded to half its length**,
    /// its top at the pin's height: a pin of the measured frame's 5 mm, 24 mm
    /// tall, 225 mm in front of the balance pin, runs straight up its two
    /// sides to 2.5 mm under its top, then over a half sphere of 2.5 mm
    /// centered on its middle, the range of its top the two arcs. A pin a
    /// millimeter tall is all tip, a cap of that sphere, whose foot reaches
    /// √(2 · 2.5 · 1 − 1²) = 2 mm out from its middle on each side.
    #[test]
    fn an_oval_pins_tip_is_rounded_to_half_its_length() {
        let (x, length) = (0.225, 0.005);
        let radius = 0.5 * length;
        let (bottom, top) = (0.0259, 0.0259 + 0.024);
        let (outline, tip) = outline_oval_pin(x, length, bottom, top);
        let [Seg::Line { .. }, Seg::Line { a: front_foot, .. }, Seg::Arc { c, r, .. }, Seg::Arc { .. }, Seg::Line { b: back_foot, .. }] =
            outline[..]
        else {
            panic!("{outline:?}");
        };
        assert_eq!(tip, 2..4);
        assert_eq!(r.to_bits(), radius.to_bits());
        assert_eq!(c.x.to_bits(), x.to_bits(), "the tip is centered on the pin");
        assert_eq!(
            (front_foot.x.to_bits(), back_foot.x.to_bits()),
            ((x + radius).to_bits(), (x - radius).to_bits())
        );
        // One rounding in the center's height, one in the top's.
        assert!((c.y - (top - radius)).abs() <= 2.0 * f64::EPSILON * top);
        assert!((outline[2].end().y - top).abs() <= 2.0 * f64::EPSILON * top);
        let (short, short_tip) = outline_oval_pin(x, length, bottom, bottom + 0.001);
        let [Seg::Line { a: back, b: front }, Seg::Arc { .. }, Seg::Arc { .. }] = short[..] else {
            panic!("{short:?}");
        };
        assert_eq!(short_tip, 1..3);
        // A square root and a few roundings of lengths near 225 mm.
        let near = |found: f64, expected: f64| (found - expected).abs() <= 8.0 * f64::EPSILON * x;
        assert!(
            near(front.x - x, 0.002) && near(x - back.x, 0.002),
            "{back:?} to {front:?}"
        );
    }

    /// Consistency: **an oval pin's outline is the same either side of its
    /// middle**, so that the solid turns its half about that middle.
    #[test]
    fn an_oval_pins_outline_is_the_same_either_side_of_its_middle() {
        let (parts, ranges) = laid(&crate::test_fixture::frame(), KeyKind::Natural);
        let pin = &parts[ranges.front_rail_pin.expect("a front pin").index()];
        let ((back, front), _) = extent(pin);
        let middle = 0.5 * (back + front);
        let corners: Vec<(f64, f64)> = pin
            .outline
            .iter()
            .flat_map(|seg| seg.drawn(crate::geometry::DRAWN_HALVINGS))
            .map(|corner| (corner.x - middle, corner.y))
            .collect();
        // Lengths near 225 mm, a few roundings each.
        let near =
            |found: f64, expected: f64| (found - expected).abs() <= 8.0 * f64::EPSILON * 0.25;
        for &(along, up) in &corners {
            assert!(
                corners
                    .iter()
                    .any(|&(other, height)| near(other, -along) && near(height, up)),
                "no twin for {along} m, {up} m"
            );
        }
    }

    /// Consistency: **an oval pin is as wide across the keyboard at its
    /// foot as its oval is**, the measured frame's 3.4 mm, to the bit; a pin
    /// a millimeter tall, a cap of its tip's sphere squeezed to the oval,
    /// whose foot reaches 2 mm of the tip's 2.5 mm radius each side, is
    /// 2/2.5 of that wide at its foot, 2.72 mm.
    #[test]
    fn an_oval_pin_is_as_wide_at_its_foot_as_its_oval_squeezed_there() {
        let width_of = |frame: &FrameDesign| {
            let (parts, ranges) = laid(frame, KeyKind::Natural);
            match parts[ranges.front_rail_pin.expect("a front pin").index()].sweep {
                Sweep::Oval { width } => width,
                other => panic!("{other:?}"),
            }
        };
        let mut frame = crate::test_fixture::frame();
        let Some(PinShape::Oval { thickness, .. }) = frame.sections.front.pin else {
            panic!("an oval front pin");
        };
        assert_eq!(width_of(&frame).to_bits(), thickness.to_bits());
        frame.sections.front.pin_height = 0.001;
        // A square root and a product and a quotient of lengths of a few
        // millimeters.
        assert!(
            (width_of(&frame) - 0.8 * thickness).abs() <= 8.0 * f64::EPSILON * thickness,
            "{} m",
            width_of(&frame)
        );
    }

    /// Consistency: **each segment of an oval pin is placed by its own
    /// number**: its foot by its rail's height, its sides by its length
    /// along the key, its rounded tip by its height.
    #[test]
    fn each_segment_of_an_oval_pin_is_placed_by_its_own_number() {
        let (parts, ranges) = laid(&crate::test_fixture::frame(), KeyKind::Natural);
        let pin = &parts[ranges.front_rail_pin.expect("a front pin").index()];
        assert_eq!(
            pin.placed_by,
            vec![
                (0..1, "front_rail.height"),
                (1..2, "front_rail.pin.length"),
                (2..3, "front_rail.pin_height"),
                (3..4, "front_rail.pin_height"),
                (4..5, "front_rail.pin.length"),
            ]
        );
    }

    /// The balance pin is turned and the front rail pin is oval.
    #[test]
    fn the_two_pins_are_not_the_same_part() {
        let (parts, ranges) = laid(&crate::test_fixture::frame(), KeyKind::Natural);
        let balance = &parts[ranges.balance_rail_pin.expect("a balance pin").index()];
        assert!(
            matches!(balance.sweep, Sweep::Revolved { .. }),
            "a balance pin is round"
        );
        let front = &parts[ranges.front_rail_pin.expect("a front pin").index()];
        let Sweep::Oval { width } = front.sweep else {
            panic!("a front rail pin is oval");
        };
        let ((x0, x1), _) = extent(front);
        assert!(
            x1 - x0 > width,
            "an oval pin is longer along the key than it is thick across it"
        );
    }
}

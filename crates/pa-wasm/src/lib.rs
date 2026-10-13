//! The only crate that knows about JavaScript.
//!
//! # What crosses, and in which direction
//!
//! Two kinds of thing cross this boundary and they are treated differently.
//!
//! **Geometry and readouts change when the technician changes something** — a
//! punching, a lead, a target — which is at human speed, a few times a second at
//! most. Those cross as JSON. It is a few hundred numbers, parsed once per edit,
//! and the clarity is worth more than the microseconds.
//!
//! **Per-step state**, once a time stepper exists, will change many times per
//! frame and will not cross as JSON. It will cross through the linear memory as
//! a pointer and a length, with the JavaScript view rebuilt whenever
//! `memory.buffer` is replaced, because growing the memory detaches the old
//! buffer. `wasm-bindgen` does not let a function return a borrowed slice.
//!
//! # What this crate may not do
//!
//! Add physics. It adapts [`pa_app`], which adapts `pa-core`. If a number is
//! wrong, it is wrong in the domain, and fixing it here would put the engine and
//! the browser permanently out of step.

use pa_app::key_bench::{
    Bearing, Fault, FingerVerdict, KeyBench, Material, Provenance, Role, Sweep, Which, KEY,
    PAPER_THICKNESSES,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// One part, in millimetres, in its own body's frame, ready to draw.
///
/// Millimetres rather than metres because every dimension the user reads is in
/// millimetres, and converting once here beats converting in every label.
/// Where the part stands is not here: it is its body's pose, which the view
/// applies as it is given.
#[derive(Serialize)]
struct OutlineDto {
    /// The body that carries it, an index into the scene's poses.
    body: usize,
    /// What it is, `snake_case`.
    code: &'static str,
    /// `"solid"`, `"cut"` or `"lining"`. A cut is drawn in the section only,
    /// as the void it is.
    role: &'static str,
    /// What it is made of, `snake_case`. The view picks its colour.
    material: &'static str,
    /// For a paper punching, its own thickness in millimetres. The trade
    /// identifies a paper by colour and the colour is the thickness, so the
    /// view is given the number rather than inventing a palette.
    paper_mm: Option<f32>,
    /// `"extruded"` or `"revolved"`. A screw is not a prism, and a punching is
    /// not a strip of felt.
    sweep: &'static str,
    /// Extrusion across the sagittal plane, millimetres. Zero when turned.
    width: f32,
    /// Where a turned part's axis stands along its body's `x`, millimetres.
    /// Zero when extruded.
    axis_x: f32,
    /// Flat `[x0, y0, x1, y1, …]`, millimetres, in the body's frame; for a
    /// turned part, its profile in radius and height.
    points: Vec<f32>,
    /// The segment of the part's outline each point starts or lies on.
    segs: Vec<u16>,
    /// Which segments come from where.
    sources: Vec<SourceDto>,
}

/// Where a stretch of an outline comes from.
#[derive(Serialize)]
struct SourceDto {
    /// The first segment.
    from: u16,
    /// One past the last.
    to: u16,
    /// `"measured"`, `"published"`, `"traced"` or `"assumed"`.
    how: &'static str,
    /// The record, a path under `data/params/`.
    record: &'static str,
}

/// Where one body's frame stands: its origin in millimetres, and the cosine
/// and sine of its rotation.
#[derive(Serialize)]
struct PoseDto {
    origin: [f32; 2],
    cos: f32,
    sin: f32,
}

/// A lead, as a circle to draw on its body.
#[derive(Serialize)]
struct LeadDto {
    /// The body it rides on, an index into the scene's poses.
    body: usize,
    /// Centre, millimetres, in the body's frame.
    x: f32,
    y: f32,
    radius: f32,
    mass_g: f32,
    /// Position along the key from the balance pin, millimetres. The number the
    /// technician actually works with.
    from_balance_mm: f32,
}

/// A point marked on a body.
#[derive(Serialize)]
struct MarkerDto {
    /// The body, an index into the scene's poses.
    body: usize,
    /// Where, millimetres, in the body's frame.
    at: [f32; 2],
}

/// Everything the view needs to draw one frame.
#[derive(Serialize)]
struct SceneDto {
    /// Every part, each in its own body's frame.
    outlines: Vec<OutlineDto>,
    /// One per body, in body order: where each body's frame stands.
    poses: Vec<PoseDto>,
    leads: Vec<LeadDto>,
    /// The measuring point. Drawn because a reader must see where the engine
    /// thinks it is weighing.
    measuring_point: MarkerDto,
    /// Why this frame cannot be weighed, when it cannot. Travels with the
    /// scene rather than with the readout, because the readout is exactly
    /// what is missing in that case.
    frame_fault: Option<&'static str>,
    /// Everything about this key and this frame that no keyboard could be
    /// built from, already written out as sentences a technician reads.
    ///
    /// Here and not on the readout for the same reason as `frame_fault`: a
    /// frame edited past what can be weighed is precisely when these matter,
    /// and precisely when there is no readout to carry them.
    ///
    /// Sentences rather than a tagged union because the panel's only job with
    /// a fault is to say it, and a second place deciding the wording is a
    /// second place for it to drift from the geometry that found it.
    faults: Vec<String>,
}

/// Metres to millimetres, narrowed for drawing.
fn mm(metres: f64) -> f32 {
    (metres * 1000.0) as f32
}

fn role(r: Role) -> &'static str {
    match r {
        Role::Solid => "solid",
        Role::Cut => "cut",
        Role::Lining => "lining",
    }
}

fn material(m: Material) -> &'static str {
    match m {
        Material::KeyWood => "key_wood",
        Material::BlockWood => "block_wood",
        Material::Covering => "covering",
        Material::KeyFront => "key_front",
        Material::RailWood => "rail_wood",
        Material::Steel => "steel",
        Material::PunchingFelt => "punching_felt",
        Material::HardFelt => "hard_felt",
        Material::Cloth => "cloth",
        Material::Paper { .. } => "paper",
    }
}

fn source(range: &core::ops::Range<u16>, p: Provenance) -> SourceDto {
    let (how, record) = match p {
        Provenance::Measured(r) => ("measured", r),
        Provenance::Published(r) => ("published", r),
        Provenance::Traced(r) => ("traced", r),
        Provenance::Assumed(r) => ("assumed", r),
    };
    SourceDto {
        from: range.start,
        to: range.end,
        how,
        record,
    }
}

/// The trade quantities, in the trade's units.
#[derive(Serialize)]
struct ReadoutDto {
    front_weight_g: f32,
    key_ratio: f32,
    key_height_mm: f32,
    key_dip_mm: f32,
    down_weight_g: f32,
    up_weight_g: f32,
    balance_weight_g: f32,
    friction_weight_g: f32,
    key_mass_g: f32,
    lead_count: u32,
    balance_paper_mm: f32,
    front_paper_mm: f32,
    key_height_target_mm: f32,
    key_dip_target_mm: f32,
    /// What is carrying the key: "on the front punching", "on the back rail
    /// cloth" or "held by friction alone". Three and not two, because the third
    /// is the friction.
    finger: &'static str,
    out_of_level_deg: f32,
    bedding_at_the_back_deg: f32,
    bears_at_the_back_mm: f32,
    back_bearing: &'static str,
    stopped_by_the_mortise: bool,
    front_pin_mm: f32,
    front_mortise_mm: f32,
    finger_weight_g: f32,
    gap_front_mm: f32,
    gap_back_mm: f32,
    friction_band_g: f32,
    /// What the protocol measures on the model, or `null` if it cannot be run
    /// on this geometry.
    measured_kf_g: Option<f32>,
    /// What pressing the key measures, millimetres, or `null` if the key
    /// cannot be pressed on this geometry.
    measured_dip_mm: Option<f32>,
    /// The papers actually stacked under the balance punching, millimetres,
    /// in the order they were laid.
    balance_papers_mm: Vec<f32>,
    /// The papers on the front punching, millimetres.
    front_papers_mm: Vec<f32>,
    /// The thicknesses a technician can pick up, millimetres.
    paper_sizes_mm: Vec<f32>,
    capstan_behind_mm: f32,
    front_edge_mm: f32,
    front_rail_at_mm: f32,
    back_rail_at_mm: f32,
    capstan_height_mm: f32,
    balance_felt_mm: f32,
    front_felt_mm: f32,
    balance_rail_mm: f32,
    front_rail_mm: f32,
    back_rail_mm: f32,
    back_cloth_mm: f32,
    /// How far back from the front lip of the key every weight is centred,
    /// millimetres: the measuring point every touchweight on the sheet is
    /// read at. The lip is the covering's.
    measuring_point_from_lip_mm: f32,
}

/// A fault, as the sentence a technician would be told.
///
/// Each names the part at fault, what it is past, and by how much — in
/// millimetres, because that is what he would reach for a rule to check.
fn say(fault: &Fault) -> String {
    match *fault {
        Fault::PinOutsideTheMortise { by } => format!(
            "the front rail pin stands {:.1} mm outside the key's mortise",
            by * 1000.0
        ),
        Fault::ClothInFrontOfTheChamfer { by } => format!(
            "the back rail cloth reaches {:.1} mm in front of the tail's chamfer, so the key \
             can only meet it on a corner",
            by * 1000.0
        ),
        Fault::ClothBehindTheKey { by } => format!(
            "the back rail cloth reaches {:.1} mm past the end of the key",
            by * 1000.0
        ),
        Fault::PunchingOffTheUnderside { rail, by } => format!(
            "the {} stands {:.1} mm off the flat of the key's underside",
            rail.code().replace('_', " "),
            by * 1000.0
        ),
    }
}

/// The single-key weighing bench.
#[wasm_bindgen]
pub struct Bench {
    inner: KeyBench,
}

#[wasm_bindgen]
impl Bench {
    /// A bench regulated to the standard targets.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: KeyBench::standard(),
        }
    }

    /// The scene to draw, as JSON: every part in its own body's frame, and
    /// each body's pose.
    pub fn scene(&self) -> String {
        let outlines = self
            .inner
            .outlines()
            .into_iter()
            .map(|o| {
                let (sweep, width, axis_x) = match o.sweep {
                    Sweep::Extruded { width } => ("extruded", mm(width), 0.0),
                    Sweep::Revolved { axis_x } => ("revolved", 0.0, mm(axis_x)),
                };
                OutlineDto {
                    body: o.body.0,
                    code: o.code.code(),
                    role: role(o.role),
                    material: material(o.material),
                    paper_mm: match o.material {
                        Material::Paper { thickness } => Some(mm(thickness)),
                        _ => None,
                    },
                    sweep,
                    width,
                    axis_x,
                    points: o.points.iter().flat_map(|p| [mm(p.x), mm(p.y)]).collect(),
                    segs: o.segs,
                    sources: o.sources.iter().map(|(r, p)| source(r, *p)).collect(),
                }
            })
            .collect();

        let poses = self
            .inner
            .poses()
            .into_iter()
            .map(|p| PoseDto {
                origin: [mm(p.origin.x), mm(p.origin.y)],
                cos: p.cos as f32,
                sin: p.sin as f32,
            })
            .collect();

        let leads = self
            .inner
            .lead_discs()
            .into_iter()
            .zip(self.inner.leads())
            .map(|((c, r), (from_balance, mass_g))| LeadDto {
                body: KEY.0,
                x: mm(c.x),
                y: mm(c.y),
                radius: mm(r),
                mass_g: mass_g as f32,
                from_balance_mm: mm(from_balance),
            })
            .collect();

        let m = self.inner.measuring_point_on_key();
        serde_json::to_string(&SceneDto {
            outlines,
            poses,
            leads,
            measuring_point: MarkerDto {
                body: KEY.0,
                at: [mm(m.x), mm(m.y)],
            },
            frame_fault: self.inner.frame_fault(),
            faults: self.inner.faults().iter().map(say).collect(),
        })
        .unwrap_or_else(|_| "null".to_owned())
    }

    /// The weighing sheet, as JSON. `null` when the geometry is degenerate,
    /// which the user can reach by editing — the view shows the last good
    /// values and says so, rather than rendering `NaN`.
    pub fn readout(&self) -> String {
        let Some(r) = self.inner.readout() else {
            return "null".to_owned();
        };
        let (balance_paper, front_paper) = self.inner.papers();
        let dims = self.inner.key_dimensions();
        let rails = self.inner.frame_dimensions();
        let positions = self.inner.rail_positions();
        let felts = self.inner.felts();
        serde_json::to_string(&ReadoutDto {
            front_weight_g: r.front_weight_g as f32,
            key_ratio: r.key_ratio as f32,
            key_height_mm: r.key_height_mm as f32,
            key_dip_mm: r.key_dip_mm as f32,
            down_weight_g: r.down_weight_g as f32,
            up_weight_g: r.up_weight_g as f32,
            balance_weight_g: r.balance_weight_g as f32,
            friction_weight_g: r.friction_weight_g as f32,
            key_mass_g: r.key_mass_g as f32,
            lead_count: r.lead_count as u32,
            balance_paper_mm: (balance_paper * 1000.0) as f32,
            front_paper_mm: (front_paper * 1000.0) as f32,
            key_height_target_mm: r.key_height_target_mm as f32,
            key_dip_target_mm: r.key_dip_target_mm as f32,
            out_of_level_deg: r.out_of_level_deg as f32,
            bedding_at_the_back_deg: r.bedding_at_the_back_deg as f32,
            bears_at_the_back_mm: r.bears_at_the_back_mm as f32,
            back_bearing: match r.back_bearing {
                Bearing::Flat => "flat on the cloth",
                Bearing::StopEdge => "on the edge of the cloth",
                Bearing::KeyCorner => "on the key's own corner",
            },
            stopped_by_the_mortise: r.stopped_by_the_mortise,
            front_pin_mm: (self.inner.front_pin_and_mortise().0 * 1000.0) as f32,
            front_mortise_mm: (self.inner.front_pin_and_mortise().1 * 1000.0) as f32,
            finger: match r.finger {
                FingerVerdict::FrontPunching => "on the front punching",
                FingerVerdict::BackRailCloth => "on the back rail cloth",
                FingerVerdict::FrictionAlone => "held by friction alone",
            },
            finger_weight_g: self.inner.finger_weight() as f32,
            gap_front_mm: r.gap_front_mm as f32,
            gap_back_mm: r.gap_back_mm as f32,
            friction_band_g: r.friction_band_g as f32,
            measured_kf_g: r.measured_kf_g.map(|v| v as f32),
            measured_dip_mm: r.measured_dip_mm.map(|v| v as f32),
            balance_papers_mm: self
                .inner
                .papers_on(Which::Balance)
                .iter()
                .map(|p| (p * 1000.0) as f32)
                .collect(),
            front_papers_mm: self
                .inner
                .papers_on(Which::Front)
                .iter()
                .map(|p| (p * 1000.0) as f32)
                .collect(),
            paper_sizes_mm: PAPER_THICKNESSES
                .iter()
                .map(|p| (p * 1000.0) as f32)
                .collect(),
            capstan_behind_mm: (-dims.0 * 1000.0) as f32,
            front_edge_mm: (dims.1 * 1000.0) as f32,
            front_rail_at_mm: (positions.0 * 1000.0) as f32,
            back_rail_at_mm: (-positions.1 * 1000.0) as f32,
            capstan_height_mm: (dims.2 * 1000.0) as f32,
            balance_felt_mm: (felts.0 * 1000.0) as f32,
            front_felt_mm: (felts.1 * 1000.0) as f32,
            balance_rail_mm: (rails.0 * 1000.0) as f32,
            front_rail_mm: (rails.1 * 1000.0) as f32,
            back_rail_mm: (rails.2 * 1000.0) as f32,
            back_cloth_mm: (rails.3 * 1000.0) as f32,
            measuring_point_from_lip_mm: (self.inner.measuring_point_from_lip() * 1000.0) as f32,
        })
        .unwrap_or_else(|_| "null".to_owned())
    }

    /// Presses the key, 0 at rest and 1 at the bottom of the dip.
    ///
    /// A direct geometric placement, for inspecting the section. The physical
    /// control is [`Bench::set_finger_weight`].
    pub fn set_depression(&mut self, f: f64) {
        self.inner.set_depression(f);
    }

    /// Puts a weight on the measuring point and lets the contact solve decide
    /// where the key comes to rest.
    pub fn set_finger_weight(&mut self, grams: f64) {
        self.inner.set_finger_weight(grams);
    }

    /// Sets how far back from the front lip of the key every weight is
    /// centred, millimetres: the measuring point of every touchweight.
    pub fn set_measuring_point_from_lip(&mut self, mm: f64) {
        self.inner.set_measuring_point_from_lip(mm * 1e-3);
    }

    /// Fits a lead, position in millimetres from the balance pin, mass in grams.
    pub fn add_lead(&mut self, from_balance_mm: f64, mass_g: f64) {
        self.inner.add_lead(from_balance_mm * 1e-3, mass_g);
    }

    /// Moves lead `i`, position in millimetres from the balance pin.
    pub fn move_lead(&mut self, i: usize, from_balance_mm: f64) {
        self.inner.move_lead(i, from_balance_mm * 1e-3);
    }

    /// Removes lead `i`.
    pub fn remove_lead(&mut self, i: usize) {
        self.inner.remove_lead(i);
    }

    /// Paper under the balance punching, millimetres. Key levelling.
    pub fn set_balance_paper(&mut self, mm: f64) {
        self.inner.set_balance_paper(mm * 1e-3);
    }

    /// Paper on the front punching, millimetres. Key dip.
    pub fn set_front_paper(&mut self, mm: f64) {
        self.inner.set_front_paper(mm * 1e-3);
    }

    /// Key friction weight, grams at the measuring point.
    pub fn set_key_friction_weight(&mut self, g: f64) {
        self.inner.set_key_friction_weight(g);
    }

    /// Moves the capstan, millimetres behind the balance pin (pass a positive
    /// number; the capstan is always behind).
    pub fn set_capstan_behind(&mut self, mm: f64) {
        self.inner.set_capstan(-mm * 1e-3);
    }

    /// Moves the key's front edge, millimetres in front of the balance pin.
    pub fn set_front_edge(&mut self, mm: f64) {
        self.inner.set_front_edge(mm * 1e-3);
    }

    /// Moves the front rail, millimetres in front of the balance pin.
    pub fn set_front_rail_at(&mut self, mm: f64) {
        self.inner.set_front_rail(mm * 1e-3);
    }

    /// Moves the back rail, millimetres behind the balance pin (pass a
    /// positive number; the back rail is always behind).
    pub fn set_back_rail_at(&mut self, mm: f64) {
        self.inner.set_back_rail(-mm * 1e-3);
    }

    /// Unbolts the rails and stands them where this key needs them.
    ///
    /// Does not re-regulate: moving a rail changes the arm the punchings were
    /// placed on, so the sheet will show a key height and a dip off target
    /// until they are shimmed again. That is the operation, in that order.
    pub fn place_the_frame_under_the_key(&mut self) {
        self.inner.place_the_frame_under_the_key();
    }

    /// Screws the capstan up or down, millimetres above the keystick.
    pub fn set_capstan_height(&mut self, mm: f64) {
        self.inner.set_capstan_height(mm * 1e-3);
    }

    /// Height of the bare balance rail above the keybed, millimetres.
    pub fn set_balance_rail_height(&mut self, mm: f64) {
        self.inner.set_balance_rail_height(mm * 1e-3);
    }

    /// Height of the bare front rail above the keybed, millimetres.
    pub fn set_front_rail_height(&mut self, mm: f64) {
        self.inner.set_front_rail_height(mm * 1e-3);
    }

    /// Height of the bare back rail above the keybed, millimetres.
    pub fn set_back_rail_height(&mut self, mm: f64) {
        self.inner.set_back_rail_height(mm * 1e-3);
    }

    /// How far the front rail pin stands above the bare rail, millimetres.
    pub fn set_front_pin_height(&mut self, mm: f64) {
        self.inner.set_front_pin_height(mm * 1e-3);
    }

    /// Depth of the key's front mortise, millimetres.
    pub fn set_front_mortise_depth(&mut self, mm: f64) {
        self.inner.set_front_mortise_depth(mm * 1e-3);
    }

    /// Thickness of the back rail cloth, millimetres.
    pub fn set_back_rail_cloth(&mut self, mm: f64) {
        self.inner.set_back_rail_cloth(mm * 1e-3);
    }

    /// Lays one more paper on the balance stack, millimetres. Key levelling.
    pub fn add_balance_paper(&mut self, mm: f64) {
        self.inner.add_paper(Which::Balance, mm * 1e-3);
    }

    /// Takes paper `i` back out of the balance stack.
    pub fn remove_balance_paper(&mut self, i: usize) {
        self.inner.remove_paper(Which::Balance, i);
    }

    /// Lays one more paper on the front stack, millimetres. Key dip.
    pub fn add_front_paper(&mut self, mm: f64) {
        self.inner.add_paper(Which::Front, mm * 1e-3);
    }

    /// Takes paper `i` back out of the front stack.
    pub fn remove_front_paper(&mut self, i: usize) {
        self.inner.remove_paper(Which::Front, i);
    }

    /// Thickness of the felt washer under the balance punching, millimetres.
    pub fn set_balance_felt(&mut self, mm: f64) {
        self.inner.set_balance_felt(mm * 1e-3);
    }

    /// Thickness of the felt washer on the front punching, millimetres.
    pub fn set_front_felt(&mut self, mm: f64) {
        self.inner.set_front_felt(mm * 1e-3);
    }
}

impl Default for Bench {
    fn default() -> Self {
        Self::new()
    }
}

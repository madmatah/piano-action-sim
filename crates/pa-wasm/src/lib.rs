//! The only crate that knows about JavaScript.
//!
//! # What crosses, and in which direction
//!
//! Two kinds of thing cross this boundary and they are treated differently.
//!
//! **The bench's documents change when the technician changes something** —
//! a punching, a weighing, a target — which is at human speed, a few times a
//! second at most. They cross as JSON, each on its own: the settings, the
//! weighing sheet, the diagnostics, where each body stands and what is drawn
//! ([`pa_app::boundary`]). Each is in codes and numbers, never sentences: the
//! page words them from its catalogue, in the reader's language. Their
//! TypeScript types are generated from the Rust ones, and this crate's
//! package re-exports them (`types/`).
//!
//! **Per-step state**, once a time stepper exists, will change many times per
//! frame and will not cross as JSON. It will cross through the linear memory as
//! a pointer and a length, with the JavaScript view rebuilt whenever
//! `memory.buffer` is replaced, because growing the memory detaches the old
//! buffer. `wasm-bindgen` does not let a function return a borrowed slice.
//!
//! # What this crate may not do
//!
//! Add physics, or wording. It adapts [`pa_app`], which adapts `pa-core`. If a
//! number is wrong, it is wrong in the domain, and fixing it here would put
//! the engine and the browser permanently out of step.

use pa_app::boundary::{self, json, Diagnostics, Scene, Settings, Sheet, State};
use pa_app::golden;
use pa_app::key_bench::{KeyBench, Which};
use wasm_bindgen::prelude::*;

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

    /// Every setting, as JSON (`Settings`): always valid, whatever the frame
    /// does, so every control reads its value here.
    pub fn settings(&self) -> String {
        json(&Settings::of(&self.inner))
    }

    /// The weighing sheet, as JSON (`Sheet`): every reading, read or refused
    /// with its reason, whether or not the key can be seated.
    pub fn sheet(&self) -> String {
        json(&Sheet::of(&self.inner))
    }

    /// How the key stands on its frame and why, as JSON (`Diagnostics`): why
    /// it cannot be seated, how it rides, the faults, its seats, what holds
    /// it under the weight on it, the punchings' edges.
    pub fn diagnostics(&self) -> String {
        json(&Diagnostics::of(&self.inner))
    }

    /// Where each body stands, as JSON (`State`): one pose per body, the
    /// measuring point, the contacts the view marks.
    pub fn state(&self) -> String {
        json(&State::of(&self.inner))
    }

    /// What is drawn, as JSON (`Scene`): every part in its own body's frame.
    pub fn scene(&self) -> String {
        json(&Scene::of(&self.inner))
    }

    /// Regulates the key to a key height and a key dip, millimetres: the
    /// papers on both punchings laid from the drawer, key height first, then
    /// the dip on the key as it then stands. Returns, as JSON
    /// (`Regulation`), the papers laid and what they give, within one paper
    /// of the targets; or, refused, the code saying which target is out of
    /// reach, by how much and what keeps it out, the bench unchanged.
    pub fn regulate(&mut self, key_height_mm: f64, key_dip_mm: f64) -> String {
        json(&boundary::regulate(
            &mut self.inner,
            key_height_mm,
            key_dip_mm,
        ))
    }

    /// Puts a weight on the measuring point, as a static increment from
    /// where the key stands: between the up weight and the down weight the
    /// key stays where it is, held by its friction.
    pub fn set_finger_weight(&mut self, grams: f64) {
        self.inner.set_finger_weight(grams);
    }

    /// Sets how far back from the front lip of the key every weight is
    /// centred, millimetres: the measuring point of every touchweight.
    pub fn set_measuring_point_from_lip(&mut self, mm: f64) {
        self.inner.set_measuring_point_from_lip(mm * 1e-3);
    }

    /// Weighs the key whole, as it arrives, its leads in it: its mass in
    /// grams, and its front weight in grams read on the levelled jig
    /// `from_lip_mm` millimetres back from the covering's lip. Returns, as
    /// JSON (`WeighingRefusal` or `null`), why it was refused, the key's
    /// weighing left as it was, or `null` when it was taken.
    pub fn weigh_key(&mut self, mass_g: f64, front_weight_g: f64, from_lip_mm: f64) -> String {
        json(&boundary::weigh_key(
            &mut self.inner,
            mass_g,
            front_weight_g,
            from_lip_mm,
        ))
    }

    /// Paper under the balance punching, millimetres. Key levelling.
    pub fn set_balance_paper(&mut self, mm: f64) {
        self.inner.set_balance_paper(mm * 1e-3);
    }

    /// Paper on the front punching, millimetres. Key dip.
    pub fn set_front_paper(&mut self, mm: f64) {
        self.inner.set_front_paper(mm * 1e-3);
    }

    /// Key friction weight, grams at the measuring point, as Stanwood's
    /// protocol reads it: the bushings set to the torque at which the
    /// protocol reads it back. Returns, as JSON (`FrictionRefusal` or
    /// `null`), why it was refused, the bushings left as they were, or
    /// `null` when it was set.
    pub fn set_key_friction_weight(&mut self, g: f64) -> String {
        json(&boundary::set_key_friction_weight(&mut self.inner, g))
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

/// The golden sheet of the fixture bench `name`, as the native engine writes
/// it (`crates/pa-app/tests/golden/`): for the test that WebAssembly
/// computes every reading to the same bits as the native build. `None` for a
/// name no fixture has.
#[wasm_bindgen]
pub fn golden_sheet(name: &str) -> Option<String> {
    golden::FIXTURES
        .iter()
        .find(|f| f.name == name)
        .map(golden::sheet)
}

impl Default for Bench {
    fn default() -> Self {
        Self::new()
    }
}

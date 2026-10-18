//! The only crate that knows about JavaScript.
//!
//! # What crosses, and in which direction
//!
//! Two kinds of thing cross this boundary and they are treated differently.
//!
//! **The bench's documents change when the technician changes something** —
//! a punching, a weighing, a target — which is at human speed, a few times a
//! second at most. They cross as JSON, each on its own: the registry, the
//! settings, the weighing sheet, the diagnostics, where each body stands and
//! what is drawn ([`pa_app::boundary`]); and the band, asked for only when it
//! is shown. Each is in codes and numbers, never
//! sentences: the page words them from its catalog, in the reader's
//! language. The page changes the bench by two generic calls, `set` a path
//! to a value and `act` an action, so a second application reuses this
//! adapter unchanged. Their TypeScript types are generated from the Rust
//! ones, and this crate's package re-exports them (`types/`).
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

use pa_app::boundary::{self, json, Band, Sheet};
use pa_app::golden;
use pa_app::instrument::Instrument;
use wasm_bindgen::prelude::*;

/// The single-key bench: the instrument, by its generic calls.
#[wasm_bindgen]
pub struct Bench {
    inner: Instrument,
}

#[wasm_bindgen]
impl Bench {
    /// The standard instrument: Hirschkorn's key on his printed frame, as it
    /// arrives.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Instrument::standard(),
        }
    }

    /// Every path, what it admits and how it is reached, and the page that
    /// presents them, as JSON (`Registry`).
    pub fn describe(&self) -> String {
        json(&self.inner.describe())
    }

    /// Every setting, nested as the paths are, as JSON (`Settings`): always
    /// valid, whatever the frame does, so every control reads its value
    /// here.
    pub fn settings(&self) -> String {
        json(&self.inner.settings())
    }

    /// The weighing sheet, as JSON (`Sheet`): every reading, read or refused
    /// with its reason, whether or not the key can be seated.
    pub fn sheet(&self) -> String {
        json(&Sheet::of(&self.inner))
    }

    /// The band, as JSON (`Band`): the key swept through its stroke, down
    /// and back, the force on the measuring point at every step, and the
    /// down and up weights read on it; or why it cannot be drawn. Taken when
    /// it is asked for, so a page asks for it when it shows the band and not
    /// with every edit.
    pub fn band(&self) -> String {
        json(&Band::of(&self.inner))
    }

    /// How the key stands on its frame and why, and the last edit refused,
    /// as JSON (`Diagnostics`).
    pub fn diagnostics(&self) -> String {
        json(&self.inner.diagnostics())
    }

    /// Where each body stands, as JSON (`State`): one pose per body, the
    /// measuring point, the contacts the view marks with their normals and
    /// forces.
    pub fn state(&self) -> String {
        json(&self.inner.state())
    }

    /// What is drawn, as JSON (`Scene`): every part in its own body's frame.
    pub fn scene(&self) -> String {
        json(&self.inner.scene())
    }

    /// Sets the setting at `path` to `value`, in the path's unit, and
    /// solves the key one increment from where it stands. Returns, as JSON
    /// (`Acted`), that it was done, or why it was refused, nothing changed.
    pub fn set(&mut self, path: &str, value: f64) -> String {
        json(&boundary::set(&mut self.inner, path, value))
    }

    /// Runs the action `action`, JSON (`Action`), and solves the key one
    /// increment from where it stands. Returns, as JSON (`Acted`), that it
    /// was done, the papers a regulation laid and what they give, or why it
    /// was refused, nothing changed.
    pub fn act(&mut self, action: &str) -> String {
        json(&boundary::act_json(&mut self.inner, action))
    }

    /// The state's hash, sixteen hexadecimal digits: FNV-1a over every
    /// normative coordinate and the hidden state. Two benches that hash
    /// alike read the same.
    pub fn state_hash(&self) -> String {
        format!("{:016x}", self.inner.state_hash())
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

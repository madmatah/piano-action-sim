---
paths:
  - "data/**"
  - "docs/**"
  - "tools/**"
  - "crates/pa-app/src/profile.rs"
  - "crates/pa-app/src/hirschkorn.rs"
  - "crates/pa-core/src/test_fixture.rs"
---

# Provenance

Where every number comes from, and which numbers and arguments are known to be wrong. Each rule ends with its check, or says it is *not mechanically checked*.

## No number without provenance

The engine starts from no number of its own: a profile in `data/profiles/` binds every value an instrument starts from to a record in `data/params/` or to an assumption it states with its reason, and a derived one has its rule beside it or its script in `tools/derivations/`, as every `[E]` does; a source in `private/` counts, a lost one does not (`docs/FOUNDATIONS.md` > Every number has its provenance). Check: `every_default_has_a_record_or_a_stated_assumption`, `the_measured_section_profile_loads_to_the_test_fixture`.

## Do not use

`docs/FOUNDATIONS.md` and the research annexes warn inline, in a sentence beginning **"Do not"**, against numbers and arguments that are wrong and easy to derive again, where a reader would otherwise derive them. Keep those warnings, add new ones where the error would be re-derived, and keep this index current. Each entry names the heading of `docs/FOUNDATIONS.md` that holds its warning:

- 12 µs, 570:1, 2/ω as the stability limit: The time step;
- 280 Hz, 180 µs: Rigid bodies;
- the time-step case against regularized friction: Set-valued dry friction, at the axes and at sliding contacts;
- the coupling I_cm + m(r·u): Relative coordinates for the jack and the repetition lever;
- "no game engine has joint dry friction": A dedicated planar solver, not a general-purpose engine;
- the additive-damping equivalence, "no published rate-independent felt model", "a factor of 56": The felt law;
- "a convergence test breaks reproducibility": Determinism, stated correctly;
- annex A1's per-step costs: The 88-note budget;
- "+0.1 mm of capstan ⇒ −1.4 g", ΔJ/J as a ranking, gradient descent on settings: Levels;
- Thorin's demonstrator touchweights read as wear: Validation instruments;
- "only the support spring moves BW": Springs inside the wippen assembly;
- the spurious gap-gradient term: The lever arm, and the gradient of a gap;
- Pw2 as the wippen pivot, 5.84: Hirschkorn's frames;
- "~40 % of friction unexplained", F ≈ 9 g, Hirschkorn's friction law without its bracket: Friction;
- the face beside an edge arc as the row that always carries a corner sunk past the arc's center: Seventeen contacts per note;
- the wippen leveled with its center pin and the capstan's mark at one height: The protocols, as they are.

Check: a grep for a listed value finds only its warning; *not enforced*.

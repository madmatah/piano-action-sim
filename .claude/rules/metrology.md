---
paths:
  - "crates/**"
  - "apps/bench/src/**"
  - "data/profiles/**"
  - "tools/smoke/**"
---

# Metrology

One criterion governs the simulator (`docs/FOUNDATIONS.md` > The acceptance criterion): **it must measure its own regulation using the trade's own protocols, and come back with the expected values.** These rules keep it to that. Each ends with its check, or says it is *not mechanically checked*.

## Regulation is trade targets solved to geometry

The coordinate is normative, the target kept beside it with its achieved value. Check: `the_regulated_key_meets_its_targets_then_lands_within_one_paper`.

## The simulator measures itself with the trade's protocols

The protocols are run on the model, never a parameter read back. Each reading carries its value in full, what the trade reads it as, its conditions, its trust and the solver's fingerprint; where a displayed value is still an input, as a key's mass, published or weighed, and the estimated height of its center of mass, `docs/FOUNDATIONS.md` > The protocols, as they are says so. The readings' closed forms are test oracles, compiled for `pa-core`'s tests only. Check: `the_bench_measures_its_own_friction_by_the_protocol`, `down_and_up_are_the_band_at_the_seats_attitudes`; `grep -rn oracle crates/pa-app/src crates/pa-wasm/src crates/pa-cli/src` finds nothing.

## Upstream settings drift, they do not cascade

Faults are reported, not refused: an assembled keyboard, however badly regulated, is what the bench exists to show. Only what cannot be computed or drawn truthfully is refused, never clamped: by the validation layer (`pa_core::validate`, `docs/FOUNDATIONS.md` > The parameter registry and the validation layer), before any solve, naming the limiting dimension and the shortfall, the instrument left as it was to the bit of its state hash. Check: `a_fault_shrinks_to_nothing_as_the_frame_comes_right`, `each_problem_is_refused_and_leaves_the_instrument_as_it_was`, `everything_an_assembled_keyboard_can_be_is_taken`, `no_number_that_is_not_finite_reaches_the_core`.

## Ratios are outputs

Changed only through geometry. The key ratio's force convention (≈ 0.5–0.7, the one in the equation of balance) and geometric one ("nominally 2:1") are reciprocals; swapping them inflates the wippen term about 3.3 times. Check: `grep -rnE 'set_[a-z_]*ratio' crates apps` finds nothing; `Touchweight::conventions_are_sane`, `the_inverted_key_ratio_is_rejected`.

## Validation

- **Drive in position, compare forces; keep the Bad-Model guard** (`docs/FOUNDATIONS.md` > What a test proves; drive in position, compare forces), from the dynamics prototype on. The action is dominated by inertia until escapement, so a deliberately wrong model driven by a force and judged on displacements scores nearly as well as the right one; driven in position, the reaction force at the finger tells them apart. The Bad-Model guard stiffens the felts: if validation still passes, the validation is worthless. *Not mechanically checked.*
- **Keep the non-confusion tests**: 1 g of mass on the key moves BW, not F; axis friction ×1.2 moves F, not BW; through the procedures, within what the tilt between the key's two seats derives, never a tolerance widened to pass; through the band likewise, the gram moving its middle and not its width, friction ×1.2 its width and not its middle. A key carrying its wippen keeps them, and the wippen keeps its own: 1 g on the wippen moves BW, not F; its center's friction ×1.2 moves F, not BW; within what its ratio and arm change between the two seats derives. Check: `one_gram_of_mass_moves_balance_weight_*`, `scaling_friction_moves_friction_weight_*`, `one_gram_on_the_wippen_moves_balance_weight_and_not_friction_weight`, `scaling_the_wippens_friction_moves_friction_weight_and_not_balance_weight`, `one_gram_of_mass_moves_the_bands_middle_and_not_its_width`, `tighter_bushings_widen_the_band_and_leave_its_middle`, `tools/smoke/`.

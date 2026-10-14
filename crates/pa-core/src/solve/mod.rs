//! The static solver: what one load increment does to a mechanism, and the
//! box problem it is solved through.
//!
//! - [`dense`]: the least-norm solution of a small linear system, by a
//!   complete orthogonal decomposition: the split of least utilisation
//!   between redundant rows.
//! - [`boxed`]: the box problem, rows of a linear complementarity system with
//!   their multipliers bounded in boxes, by projected Gauss–Seidel finished by
//!   an exact active-set pass. Shared with the dynamic step to come.
//! - [`statics`]: one load increment, the incremental problem of a
//!   rate-independent system (potential energy plus the dissipation of dry
//!   friction at the axes, every gap kept from closing past zero), solved by
//!   proximal sequential quadratic steps whose subproblem is a box problem.
//! - [`report`]: the counts every solve reports.

pub mod boxed;
pub mod dense;
pub mod report;
pub mod statics;

pub use report::SolveReport;
pub use statics::{Equilibrium, Friction, Holder, Refusal, Statics, Support};

/// Which solver took a reading, and with which frozen numbers: what every
/// measurement carries ([`crate::measure::measurement::Measurement`]), and
/// every saved target will.
///
/// Another solver, or this one with another cap, tolerance, margin, walk
/// or sweep step, count of halvings, station of the temporary weight,
/// protocol load or reach of a search, may read a key a bit differently; a
/// saved number re-solved by it must not pass for the same reading. So the
/// frozen numbers are hashed by a test,
/// `the_fingerprint_names_the_frozen_numbers`, which fails when one changes
/// and the fingerprint does not.
pub const SOLVER_FINGERPRINT: &str = "statics/1";

#[cfg(test)]
mod tests {
    use super::*;

    /// FNV-1a, 64 bits, over a sequence of bit patterns: no dependency, and
    /// the same on every machine, which `std`'s hasher does not promise.
    fn fnv1a(words: &[u64]) -> u64 {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for word in words {
            for byte in word.to_le_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        hash
    }

    /// Every number frozen in the solver and the procedures that read with
    /// it, in a fixed order: the static solve's caps and tolerances, the box
    /// problem's, the dense algebra's rank tolerance, the row margin, the
    /// key's angle limit and the step of its nominal pose, the steps that
    /// narrow the attitude of a key lowered onto its frame from above, in how
    /// many pieces of the cap a key free to lift is solved, the seated pose's
    /// margin and overlap, the walk's step, cap and halvings, how near a
    /// contact counts as touching, the regulation's search and halvings, the
    /// drawer's search; then how near a whole gram the walk's threshold reads
    /// as that gram, the temporary weight's two stations, the search for the
    /// bushings a key friction weight reads on and its halvings, the jig's
    /// level, gravity, which turns every gram the procedures put on a key
    /// into newtons and every torque they read back into grams, the slack of
    /// a contact row's sector test, how flat a key's bearing on its cloth
    /// counts as flat, and the step the band's sweep drives the measuring
    /// point by.
    ///
    /// Not here, because each reading carries it or its record holds it: the
    /// measuring point, a setting every reading names in its conditions; the
    /// key weight ratio's load, the protocol's, which the instrument's
    /// profile binds to its record; each reading's resolution and rounding
    /// rule; the drawer's papers and the parts' dimensions, data with their
    /// records, among them how far above its balance punching the key leaves
    /// its pin, which the drawn pin gives. Nor the numbers the bench draws
    /// with and reads nothing by.
    fn frozen() -> Vec<u64> {
        use crate::measure::{jig, seat, sweep, walk, weighing};
        use crate::parts::{frame, mounts, verdicts};
        use crate::regulation;
        let mut words = vec![
            u64::from(statics::ITERATION_CAP),
            statics::STEP_TOLERANCE.to_bits(),
            statics::GAP_TOLERANCE.to_bits(),
            statics::RESIDUAL_TOLERANCE.to_bits(),
            statics::RAISE.to_bits(),
            statics::LOWER.to_bits(),
            u64::from(statics::RAISES_PER_STEP),
            statics::FIT.to_bits(),
            statics::SWEEP_TOLERANCE.to_bits(),
            statics::MERIT_ROUNDING.to_bits(),
            u64::from(boxed::SWEEP_CAP),
        ];
        words.extend(boxed::CHECKPOINTS.iter().map(|&c| u64::from(c)));
        words.extend([
            boxed::EXACT_RESIDUAL.to_bits(),
            boxed::ON_A_BOUND.to_bits(),
            boxed::DIVERGENCE.to_bits(),
            dense::RANK_TOLERANCE.to_bits(),
            crate::contact::ROW_MARGIN.to_bits(),
            mounts::KEY_ANGLE_LIMIT.to_bits(),
            mounts::NOMINAL_STEP.to_bits(),
            u64::from(mounts::ATTITUDE_REFINEMENTS),
            u64::from(mounts::FREE_PIECES),
            seat::SEAT_MARGIN.to_bits(),
            seat::SEAT_OVERLAP.to_bits(),
            walk::STEP_G.to_bits(),
            u64::from(walk::CAP),
            u64::from(walk::HALVINGS),
            verdicts::TOUCHING.to_bits(),
            regulation::STACK_SEARCH.to_bits(),
            u64::from(regulation::HALVINGS),
            frame::PAPER_STEP.to_bits(),
            frame::PAPER_SEARCH_LIMIT.to_bits(),
            frame::PAPER_EPSILON.to_bits(),
            walk::ROUNDING_TOLERANCE_G.to_bits(),
            weighing::STATION_BEHIND.to_bits(),
            weighing::STATION_IN_FRONT.to_bits(),
            weighing::FRICTION_SEARCH_G.to_bits(),
            u64::from(weighing::FRICTION_HALVINGS),
            jig::KeyJig::LEVEL.to_bits(),
            crate::G.to_bits(),
            crate::geometry::signed::SECTOR_SLACK.to_bits(),
            crate::contact::FLAT_WITHIN.to_bits(),
            sweep::STEP.to_bits(),
        ]);
        words
    }

    /// C10, consistency: **the fingerprint names the frozen numbers.** Their
    /// hash is recorded beside the fingerprint; a number changed without a
    /// new fingerprint fails here, and so does one left out of the hash. Give
    /// the solver a new fingerprint, then record the hash this test prints.
    #[test]
    fn the_fingerprint_names_the_frozen_numbers() {
        let hash = fnv1a(&frozen());
        assert_eq!(
            (SOLVER_FINGERPRINT, hash),
            ("statics/1", 0x0891_4d79_7b51_9fb7),
            "a frozen number changed: the hash is now {hash:#018x}. Give the solver a new \
             fingerprint and record the hash beside it."
        );
    }

    /// C10, consistency: **the procedures' own numbers are among them**: the
    /// temporary weight's two stations, which the key friction weights read
    /// through, and the other numbers a procedure fixes beside the solver's,
    /// each once in the hash, so that moving one fails the test above. Each
    /// is found by its bits, which no other number frozen there shares; a
    /// count of halvings shares its value with the box problem's
    /// checkpoints, and the hash alone answers for it.
    #[test]
    fn the_procedures_numbers_are_hashed() {
        use crate::measure::{jig, sweep, walk, weighing};
        let words = frozen();
        for (name, x) in [
            ("sweep::STEP", sweep::STEP),
            ("STATION_BEHIND", weighing::STATION_BEHIND),
            ("STATION_IN_FRONT", weighing::STATION_IN_FRONT),
            ("FRICTION_SEARCH_G", weighing::FRICTION_SEARCH_G),
            ("ROUNDING_TOLERANCE_G", walk::ROUNDING_TOLERANCE_G),
            ("KeyJig::LEVEL", jig::KeyJig::LEVEL),
            ("G", crate::G),
        ] {
            assert_eq!(
                words.iter().filter(|&&w| w == x.to_bits()).count(),
                1,
                "{name} is hashed once"
            );
        }
    }
}

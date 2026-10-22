//! What a solve reports, beside its answer.

/// The counts and the residual of a static solve: one load increment, or a
/// procedure's several added together.
///
/// Every loop that stops on a test has a cap and reports its count
/// (`.claude/rules/physics.md` > Determinism): a reading that converged says how, and one that did
/// not is refused rather than shown.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolveReport {
    /// Load increments solved.
    pub increments: u32,
    /// Proximal steps taken or tried, in all.
    pub iterations: u32,
    /// The most steps any one increment took.
    pub most_iterations: u32,
    /// Steps the merit test turned back.
    pub rejected: u32,
    /// Times the friction anchor moved on along the path.
    pub anchors: u32,
    /// Sweeps of the box problems, in all.
    pub sweeps: u32,
    /// Exact passes tried, in all.
    pub exact_passes: u32,
    /// Halvings the starts of the solves took to lower the wippen the key
    /// carries onto the key, in all
    /// ([`crate::parts::mounts::wippen_lowered`]): nothing where it carries
    /// none.
    pub lowering_halvings: u32,
    /// Whether every increment converged.
    pub converged: bool,
    /// Whether every box problem was solved by its exact pass.
    pub exact: bool,
    /// The largest stationarity residual left at the end of an increment, on
    /// any coordinate, over that coordinate's reach: newtons.
    pub residual: f64,
}

impl Default for SolveReport {
    fn default() -> Self {
        Self::NOTHING
    }
}

impl SolveReport {
    /// The report of no solve at all: nothing counted, nothing failed.
    pub const NOTHING: Self = Self {
        increments: 0,
        iterations: 0,
        most_iterations: 0,
        rejected: 0,
        anchors: 0,
        sweeps: 0,
        exact_passes: 0,
        lowering_halvings: 0,
        converged: true,
        exact: true,
        residual: 0.0,
    };

    /// This increment's report, the increment then going on from where it
    /// stopped as `then` reports: one increment, its steps and sweeps added
    /// up, and how it ended, `then`'s.
    pub fn continued(self, then: Self) -> Self {
        let iterations = self.iterations + then.iterations;
        Self {
            increments: self.increments.max(then.increments),
            iterations,
            most_iterations: iterations,
            rejected: self.rejected + then.rejected,
            anchors: self.anchors + then.anchors,
            sweeps: self.sweeps + then.sweeps,
            exact_passes: self.exact_passes + then.exact_passes,
            lowering_halvings: self.lowering_halvings + then.lowering_halvings,
            converged: then.converged,
            exact: self.exact && then.exact,
            residual: then.residual,
        }
    }

    /// This report and `other` together, as one procedure's.
    pub fn and(self, other: Self) -> Self {
        Self {
            increments: self.increments + other.increments,
            iterations: self.iterations + other.iterations,
            most_iterations: self.most_iterations.max(other.most_iterations),
            rejected: self.rejected + other.rejected,
            anchors: self.anchors + other.anchors,
            sweeps: self.sweeps + other.sweeps,
            exact_passes: self.exact_passes + other.exact_passes,
            lowering_halvings: self.lowering_halvings + other.lowering_halvings,
            converged: self.converged && other.converged,
            exact: self.exact && other.exact,
            residual: self.residual.max(other.residual),
        }
    }
}

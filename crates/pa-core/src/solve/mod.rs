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

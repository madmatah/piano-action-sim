//! The box problem: multipliers bounded in boxes, each row of a linear
//! complementarity system. The static increment's subproblem is one
//! ([`super::statics`]); the dynamic step of `docs/FOUNDATIONS.md` §4.2 will
//! be another, with the mass matrix in place of the static metric.
//!
//! # The problem
//!
//! ```text
//! u = W ν + u_free,   lo_i ≤ ν_i ≤ hi_i
//! ν_i strictly inside its box ⇒ u_i = 0;   ν_i = lo_i ⇒ u_i ≥ 0;   ν_i = hi_i ⇒ u_i ≤ 0
//! ```
//!
//! `W` symmetric and positive semi-definite. A contact row's multiplier is
//! the force the contact pushes with, bounded `[0, +∞)`, and its `u` the gap
//! after the step; a friction row's is the generalized force friction
//! applies, bounded `[−b, b]`, and its `u` how far the coordinate stands from
//! where the increment started.
//!
//! # How it is solved
//!
//! The dynamics prototype's formulation (`tools/dynamics-prototype/study.py`,
//! `solve_box` and `exact_pass`), generalized from its symmetric friction
//! boxes to any bounds. Projected Gauss–Seidel sweeps run in row order from
//! the warm start. After sweeps 1, 2, 4, … 128 an exact pass starts from the
//! active set the multipliers show (each row inside its box, or on one of its
//! bounds) and finishes the problem as a primal active-set method: the rows
//! inside are solved exactly for the split of least utilization,
//! `Σ (ν_i / s_i)²`, by a complete orthogonal decomposition
//! ([`super::dense`]); the pass moves toward that target as far as the
//! boxes let it, holding the first row that reaches a bound; at the target
//! it frees the held row of largest natural residual,
//! `|ν_i − clamp(ν_i − u_i / w_ii)| / s_i`, the first of equals; when every
//! residual is under [`EXACT_RESIDUAL`] the multipliers meet every condition
//! of the problem. Every move lowers the problem's objective, which is what
//! keeps the pass from cycling. Between checkpoints the sweeps stop once one
//! moves no generalized coordinate by more than the caller's tolerance and
//! leaves no residual: the motion is unique even where rows are redundant,
//! the multipliers are not.
//!
//! Each row has a finite scale `s_i`: the bound of a friction row, a load
//! scale for a contact row, which an infinite bound cannot give. It
//! normalizes the residuals and weights the split. A row that cannot act,
//! `w_ii = 0` (its gradient vanishes), is skipped and reported.
//!
//! # A linearization no motion satisfies
//!
//! Two stops that fight, or a coordinate driven into a closed stop, leave
//! the problem unbounded below: some contact multipliers grow without end
//! along a direction `y` of the right signs with `W y = 0` and
//! `u_freeᵀ y < 0`, a certificate (Farkas's) that no motion meets every row.
//! Three things catch it, and each names the rows involved: a multiplier
//! past [`DIVERGENCE`] times its row's scale; the sweeps' multipliers still
//! moving once the motion has stopped, along such a direction; and, in the
//! exact pass, free rows no multipliers make still, whose least-squares
//! residual is such a direction meeting no bound. A direction is accepted
//! once the least of the problem along it lies past the same multiple of the
//! scales, which tolerates the rounding of `W y`.

use super::dense::LeastNorm;

/// The most sweeps one solve runs.
pub const SWEEP_CAP: u32 = 200;

/// The sweeps after which an exact pass is tried: 1, 2, 4, … 128.
pub const CHECKPOINTS: [u32; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

/// The largest natural residual, over its row's scale, at which the exact
/// pass accepts its multipliers: a part in 10⁹ of a row's scale.
pub const EXACT_RESIDUAL: f64 = 1e-9;

/// How near a bound, over the row's scale, a multiplier from the sweeps must
/// be for the exact pass to start that row on the bound.
pub const ON_A_BOUND: f64 = 1e-9;

/// How many times its row's scale a multiplier may grow before the problem
/// counts as one no motion satisfies.
pub const DIVERGENCE: f64 = 1e6;

/// Where a row stands at the solution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowState {
    /// Strictly inside its box, its `u` zero: a contact held closed, a
    /// coordinate friction holds still.
    Inside,
    /// On its lower bound, `u ≥ 0`: a contact open, or a coordinate sliding
    /// up against its friction.
    AtLo,
    /// On its upper bound, `u ≤ 0`.
    AtHi,
    /// Its gradient vanishes, `w_ii = 0`: it cannot act, and is skipped.
    Idle,
}

/// One box problem, borrowed.
#[derive(Debug, Clone, Copy)]
pub struct BoxProblem<'a> {
    /// `W`, `n × n`, row by row: symmetric, positive semi-definite.
    pub w: &'a [f64],
    /// `u` with every multiplier zero.
    pub u_free: &'a [f64],
    /// Lower bounds, possibly `−∞`.
    pub lo: &'a [f64],
    /// Upper bounds, possibly `+∞`.
    pub hi: &'a [f64],
    /// Each row's scale, finite and positive.
    pub scale: &'a [f64],
    /// The generalized motion a unit multiplier of each row produces, one
    /// row of `n` per generalized coordinate, already weighted so that its
    /// entries are lengths: what the sweeps' stopping test measures.
    pub motion: &'a [f64],
    /// The sweeps stop once one moves every generalized coordinate by at
    /// most this, in the units of `motion`.
    pub motion_tolerance: f64,
}

impl BoxProblem<'_> {
    fn n(&self) -> usize {
        self.u_free.len()
    }

    fn w(&self, i: usize, j: usize) -> f64 {
        self.w[i * self.n() + j]
    }
}

/// What a solve reports.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BoxReport {
    /// Sweeps run.
    pub sweeps: u32,
    /// Exact passes tried.
    pub exact_passes: u32,
    /// Whether it stopped on its own tests rather than on the cap.
    pub converged: bool,
    /// Whether an exact pass found the solution.
    pub exact: bool,
    /// The largest natural residual at the end, over its row's scale.
    pub residual: f64,
    /// Rows skipped because they cannot act.
    pub idle: u32,
}

/// How a solve ended.
#[derive(Debug, Clone, PartialEq)]
pub enum BoxOutcome {
    /// The multipliers are in the caller's slice.
    Solved(BoxReport),
    /// No motion meets every row: these rows' multipliers grow without end.
    Infeasible {
        /// The rows, in order.
        rows: Vec<usize>,
        /// What the solve did before it found out.
        report: BoxReport,
    },
}

/// The box solver and its workspace, kept between solves so that a solve of
/// a size already seen allocates nothing.
#[derive(Debug, Clone, Default)]
pub struct BoxSolver {
    states: Vec<RowState>,
    trial: Vec<f64>,
    before: Vec<f64>,
    u: Vec<f64>,
    stuck: Vec<usize>,
    m: Vec<f64>,
    rhs: Vec<f64>,
    z: Vec<f64>,
    y: Vec<f64>,
    least: LeastNorm,
}

/// What an exact pass found.
enum Exact {
    Found,
    Infeasible(Vec<usize>),
    Failed,
}

fn clamp(x: f64, lo: f64, hi: f64) -> f64 {
    if x < lo {
        lo
    } else if x > hi {
        hi
    } else {
        x
    }
}

impl BoxSolver {
    /// An empty workspace.
    pub fn new() -> Self {
        Self::default()
    }

    /// Where each row stood at the end of the last solve.
    pub fn states(&self) -> &[RowState] {
        &self.states
    }

    /// Solves `p`, its multipliers starting from `nu` and written back there.
    ///
    /// # Panics
    ///
    /// When the slices do not agree in size, or a scale is not positive.
    pub fn solve(&mut self, p: &BoxProblem, nu: &mut [f64]) -> BoxOutcome {
        let n = p.n();
        assert_eq!(p.w.len(), n * n, "W is n × n");
        assert!(
            p.lo.len() == n && p.hi.len() == n && p.scale.len() == n && nu.len() == n,
            "one bound, scale and multiplier per row"
        );
        assert!(
            p.scale.iter().all(|&s| s > 0.0 && s.is_finite()),
            "every row's scale is finite and positive"
        );
        assert_eq!(p.motion.len() % n.max(1), 0, "a motion row per coordinate");
        let coords = p.motion.len().checked_div(n).unwrap_or(0);

        self.states.clear();
        let mut report = BoxReport::default();
        for (i, v) in nu.iter_mut().enumerate() {
            let idle = p.w(i, i).is_nan() || p.w(i, i) <= 0.0;
            self.states.push(if idle {
                RowState::Idle
            } else {
                RowState::Inside
            });
            *v = clamp(if idle { 0.0 } else { *v }, p.lo[i], p.hi[i]);
            if idle {
                report.idle += 1;
            }
        }
        if self.states.iter().all(|&s| s == RowState::Idle) {
            report.converged = true;
            report.exact = true;
            return BoxOutcome::Solved(report);
        }

        for sweep in 1..=SWEEP_CAP {
            self.before.clear();
            self.before.extend_from_slice(nu);
            for i in 0..n {
                if self.states[i] == RowState::Idle {
                    continue;
                }
                let mut r = p.u_free[i];
                for (j, &v) in nu.iter().enumerate() {
                    r += p.w(i, j) * v;
                }
                nu[i] = clamp(nu[i] - r / p.w(i, i), p.lo[i], p.hi[i]);
            }
            report.sweeps = sweep;

            let grown: Vec<usize> = (0..n)
                .filter(|&i| nu[i].abs() > DIVERGENCE * p.scale[i])
                .collect();
            if !grown.is_empty() {
                report.residual = self.residual(p, nu);
                return BoxOutcome::Infeasible {
                    rows: grown,
                    report,
                };
            }

            if CHECKPOINTS.contains(&sweep) || sweep == SWEEP_CAP {
                report.exact_passes += 1;
                match self.exact_pass(p, nu) {
                    Exact::Found => {
                        nu.copy_from_slice(&self.trial);
                        report.converged = true;
                        report.exact = true;
                        break;
                    }
                    Exact::Infeasible(rows) => {
                        report.residual = self.residual(p, nu);
                        return BoxOutcome::Infeasible { rows, report };
                    }
                    Exact::Failed => {}
                }
            }

            let mut moved = 0.0_f64;
            for c in 0..coords {
                let mut s = 0.0;
                for (i, (&v, &b)) in nu.iter().zip(&self.before).enumerate() {
                    s += p.motion[c * n + i] * (v - b);
                }
                moved = moved.max(s.abs());
            }
            if moved <= p.motion_tolerance {
                if self.residual(p, nu) <= EXACT_RESIDUAL {
                    report.converged = true;
                    break;
                }
                // The motion has stopped and the multipliers have not: they
                // move along directions that move nothing, which is how they
                // grow without end where no motion meets every row.
                self.y.clear();
                self.y
                    .extend(nu.iter().zip(&self.before).map(|(&v, &b)| v - b));
                if let Some(rows) = self.ray(p) {
                    report.residual = self.residual(p, nu);
                    return BoxOutcome::Infeasible { rows, report };
                }
            }
        }

        report.residual = self.residual(p, nu);
        if !report.exact {
            // The states the multipliers show, for the caller to read.
            for (i, &v) in nu.iter().enumerate() {
                if self.states[i] != RowState::Idle {
                    self.states[i] = Self::state_of(p, i, v);
                }
            }
        }
        BoxOutcome::Solved(report)
    }

    /// The state a multiplier shows: on a bound if within [`ON_A_BOUND`] of
    /// its scale of it, inside otherwise.
    fn state_of(p: &BoxProblem, i: usize, nu: f64) -> RowState {
        let s = p.scale[i];
        if p.lo[i].is_finite() && nu - p.lo[i] <= ON_A_BOUND * s {
            RowState::AtLo
        } else if p.hi[i].is_finite() && p.hi[i] - nu <= ON_A_BOUND * s {
            RowState::AtHi
        } else {
            RowState::Inside
        }
    }

    /// The largest natural residual over the rows that can act.
    fn residual(&mut self, p: &BoxProblem, nu: &[f64]) -> f64 {
        let n = p.n();
        let mut worst = 0.0_f64;
        for i in 0..n {
            if self.states[i] == RowState::Idle {
                continue;
            }
            let mut u = p.u_free[i];
            for (j, &v) in nu.iter().enumerate() {
                u += p.w(i, j) * v;
            }
            let r = (nu[i] - clamp(nu[i] - u / p.w(i, i), p.lo[i], p.hi[i])).abs() / p.scale[i];
            worst = worst.max(r);
        }
        worst
    }

    /// From the multipliers `guess` (within their boxes), the exact solution,
    /// into `self.trial`: a primal active-set method on the problem's
    /// objective `½ νᵀ W ν + u_freeᵀ ν`, which every move lowers.
    ///
    /// The rows `guess` shows on a bound are held there, the rest are free.
    /// On the free rows, the multipliers that make their `u` zero, of least
    /// utilization, are the target: the pass moves toward it as far as the
    /// boxes let it, and holds the first row that reaches its bound (the
    /// first of equals). Where no multipliers make the free rows' `u` zero,
    /// their least-squares residual is a direction of zero curvature along
    /// which the objective falls; the pass moves along it to the first bound
    /// it meets, or, meeting none, has found that no motion meets every row.
    /// At the target, a row held on a bound whose `u` pushes it inside is
    /// freed, the one of largest natural residual first. When none is left
    /// the multipliers meet every condition of the problem.
    ///
    /// Moving by these ratio tests is what keeps the pass from cycling,
    /// which the dynamics prototype's pivot, moving the row of largest
    /// residual straight to its other state, does on some problems of the
    /// exhaustive check.
    fn exact_pass(&mut self, p: &BoxProblem, guess: &[f64]) -> Exact {
        let n = p.n();
        self.trial.clear();
        self.trial.extend_from_slice(guess);
        for (i, &g) in guess.iter().enumerate() {
            if self.states[i] == RowState::Idle {
                continue;
            }
            self.states[i] = Self::state_of(p, i, g);
            match self.states[i] {
                RowState::AtLo => self.trial[i] = p.lo[i],
                RowState::AtHi => self.trial[i] = p.hi[i],
                _ => {}
            }
        }
        self.u.clear();
        self.u.resize(n, 0.0);
        let acting = self.states.iter().filter(|&&s| s != RowState::Idle).count();
        for _move in 0..8 * acting + 8 {
            self.stuck.clear();
            for i in 0..n {
                if self.states[i] == RowState::Inside {
                    self.stuck.push(i);
                }
            }
            let k = self.stuck.len();
            if k > 0 {
                // The target: W_ss diag(s) z = −(u_free_s + W_s,held ν_held),
                // least z.
                self.m.clear();
                self.rhs.clear();
                for &a in &self.stuck {
                    let mut r = -p.u_free[a];
                    for j in 0..n {
                        if self.states[j] != RowState::Inside {
                            r -= p.w(a, j) * self.trial[j];
                        }
                    }
                    self.rhs.push(r);
                    for &b in &self.stuck {
                        self.m.push(p.w(a, b) * p.scale[b]);
                    }
                }
                let rhs = self.rhs.clone();
                self.z.clear();
                self.z.resize(k, 0.0);
                self.least
                    .solve(&mut self.m, k, k, &mut self.rhs, &mut self.z);
                // Its residual, zero where the free rows can all be still.
                self.y.clear();
                self.y.resize(n, 0.0);
                let mut inconsistent = false;
                for (idx, &a) in self.stuck.iter().enumerate() {
                    let mut r = -rhs[idx];
                    for (jdx, &b) in self.stuck.iter().enumerate() {
                        r += p.w(a, b) * p.scale[b] * self.z[jdx];
                    }
                    self.y[a] = -r;
                    if (r / p.w(a, a)).abs() / p.scale[a] > EXACT_RESIDUAL {
                        inconsistent = true;
                    }
                }
                let target_step = !inconsistent;
                if target_step {
                    for (idx, &a) in self.stuck.iter().enumerate() {
                        self.y[a] = p.scale[a] * self.z[idx] - self.trial[a];
                    }
                }
                // As far as the boxes let it, along y: to the target, or along
                // the residual.
                let mut alpha = if target_step { 1.0 } else { f64::INFINITY };
                let mut blocking = None;
                for &a in &self.stuck {
                    let y = self.y[a];
                    let room = if y > 0.0 {
                        (p.hi[a] - self.trial[a]) / y
                    } else if y < 0.0 {
                        (p.lo[a] - self.trial[a]) / y
                    } else {
                        continue;
                    };
                    if room < alpha {
                        alpha = room.max(0.0);
                        blocking = Some((
                            a,
                            if y > 0.0 {
                                RowState::AtHi
                            } else {
                                RowState::AtLo
                            },
                        ));
                    }
                }
                if !alpha.is_finite() {
                    return match self.ray(p) {
                        Some(rows) => Exact::Infeasible(rows),
                        None => Exact::Failed,
                    };
                }
                for &a in &self.stuck {
                    self.trial[a] += alpha * self.y[a];
                }
                if let Some((a, state)) = blocking {
                    self.states[a] = state;
                    self.trial[a] = if state == RowState::AtHi {
                        p.hi[a]
                    } else {
                        p.lo[a]
                    };
                    continue;
                }
            }
            // At the target: free the held row pushed inside the hardest.
            for i in 0..n {
                let mut u = p.u_free[i];
                for j in 0..n {
                    u += p.w(i, j) * self.trial[j];
                }
                self.u[i] = u;
            }
            let mut worst = EXACT_RESIDUAL;
            let mut release = None;
            for i in 0..n {
                let wrong = match self.states[i] {
                    RowState::AtLo => self.u[i] < 0.0,
                    RowState::AtHi => self.u[i] > 0.0,
                    _ => false,
                };
                if !wrong {
                    continue;
                }
                let nu = self.trial[i];
                let r =
                    (nu - clamp(nu - self.u[i] / p.w(i, i), p.lo[i], p.hi[i])).abs() / p.scale[i];
                if r > worst {
                    worst = r;
                    release = Some(i);
                }
            }
            match release {
                Some(i) => self.states[i] = RowState::Inside,
                None => return Exact::Found,
            }
        }
        Exact::Failed
    }

    /// Whether `self.y` is a direction along which the problem falls without
    /// bound, within the rounding of `W y`: its signs those along which a
    /// multiplier may grow (up on a row bounded only below, down on one
    /// bounded only above, nothing to speak of on a row bounded both ways),
    /// `u_freeᵀ y < 0`, and the least of the problem along it past
    /// [`DIVERGENCE`] times the rows' scales. Returns the rows where `y` is
    /// not zero: the rows that fight.
    fn ray(&mut self, p: &BoxProblem) -> Option<Vec<usize>> {
        let n = p.n();
        let size = |a: usize, y: f64| y.abs() / p.scale[a];
        let largest = (0..n)
            .filter(|&a| self.states[a] != RowState::Idle)
            .map(|a| size(a, self.y[a]))
            .fold(0.0_f64, f64::max);
        if largest == 0.0 {
            return None;
        }
        let mut rows = Vec::new();
        for a in 0..n {
            let y = self.y[a];
            if self.states[a] == RowState::Idle || size(a, y) <= EXACT_RESIDUAL * largest {
                self.y[a] = 0.0;
                continue;
            }
            let grows =
                (y > 0.0 && p.hi[a] == f64::INFINITY) || (y < 0.0 && p.lo[a] == f64::NEG_INFINITY);
            if !grows {
                return None;
            }
            rows.push(a);
        }
        let (mut lin, mut quad) = (0.0, 0.0);
        for &a in &rows {
            lin += p.u_free[a] * self.y[a];
            for &b in &rows {
                quad += self.y[a] * p.w(a, b) * self.y[b];
            }
        }
        if lin >= 0.0 {
            return None;
        }
        // Along ν + t y the problem falls as t·lin + t²·quad/2: without end
        // if quad is zero, to its least at t = −lin/quad otherwise, where
        // each multiplier has grown by t·|y|.
        let far = rows
            .iter()
            .map(|&a| size(a, self.y[a]))
            .fold(0.0_f64, f64::max);
        if quad <= 0.0 || -lin / quad * far > DIVERGENCE {
            Some(rows)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanism::fixtures::XorShift;

    /// A random problem in the shape the static increment builds: `m`
    /// generalized coordinates with a positive-definite metric `A`, a friction
    /// row on each (bounds `±b`), and contact rows bounded `[0, +∞)` with
    /// random gradients and gaps, one of them sometimes repeated, so that
    /// rows are redundant.
    struct Random {
        m: usize,
        d_free: Vec<f64>,
        w: Vec<f64>,
        u_free: Vec<f64>,
        lo: Vec<f64>,
        hi: Vec<f64>,
        scale: Vec<f64>,
        y: Vec<f64>,
    }

    impl Random {
        fn n(&self) -> usize {
            self.u_free.len()
        }

        fn new(rng: &mut XorShift, case: usize) -> Self {
            let m = 1 + case % 3;
            let contacts = 1 + (case / 3) % 3;
            let repeat = case.is_multiple_of(4);
            // A = B Bᵀ + 0.1 I, and its inverse by the least-norm solve,
            // which inverts a regular matrix.
            let b: Vec<f64> = (0..m * m).map(|_| rng.uniform(-1.0, 1.0)).collect();
            let mut a = vec![0.0; m * m];
            for i in 0..m {
                for j in 0..m {
                    a[i * m + j] = (0..m).map(|k| b[i * m + k] * b[j * m + k]).sum::<f64>()
                        + if i == j { 0.1 } else { 0.0 };
                }
            }
            let mut a_inv = vec![0.0; m * m];
            let mut least = LeastNorm::new();
            for col in 0..m {
                let mut mat = a.clone();
                let mut e = vec![0.0; m];
                e[col] = 1.0;
                let mut x = vec![0.0; m];
                least.solve(&mut mat, m, m, &mut e, &mut x);
                for row in 0..m {
                    a_inv[row * m + col] = x[row];
                }
            }
            // Rows: the friction rows, then the contacts.
            let mut jac = Vec::new();
            let (mut lo, mut hi, mut scale, mut offset) = (vec![], vec![], vec![], vec![]);
            for c in 0..m {
                for k in 0..m {
                    jac.push(if k == c { 1.0 } else { 0.0 });
                }
                let bound = rng.uniform(0.05, 1.5);
                lo.push(-bound);
                hi.push(bound);
                scale.push(bound);
                offset.push(0.0);
            }
            let (mut last, mut last_gap) = (Vec::new(), 0.0);
            for k in 0..contacts {
                // A repeated contact is the same row twice, gap included: two
                // edges of one face, closing together.
                let (row, gap): (Vec<f64>, f64) = if repeat && k > 0 {
                    (last.clone(), last_gap)
                } else {
                    (
                        (0..m).map(|_| rng.uniform(-1.0, 1.0)).collect(),
                        rng.uniform(-0.5, 0.5),
                    )
                };
                jac.extend_from_slice(&row);
                last = row;
                last_gap = gap;
                lo.push(0.0);
                hi.push(f64::INFINITY);
                scale.push(1.0);
                offset.push(gap);
            }
            let n = lo.len();
            // The motion with no multiplier, A⁻¹(−c).
            let c: Vec<f64> = (0..m).map(|_| rng.uniform(-2.0, 2.0)).collect();
            let d_free: Vec<f64> = (0..m)
                .map(|i| -(0..m).map(|k| a_inv[i * m + k] * c[k]).sum::<f64>())
                .collect();
            // Y = A⁻¹ Jᵀ (m × n), W = J Y, u_free = offset + J d_free.
            let mut y = vec![0.0; m * n];
            for i in 0..m {
                for r in 0..n {
                    y[i * n + r] = (0..m).map(|k| a_inv[i * m + k] * jac[r * m + k]).sum();
                }
            }
            let mut w = vec![0.0; n * n];
            for r in 0..n {
                for s in 0..n {
                    w[r * n + s] = (0..m).map(|k| jac[r * m + k] * y[k * n + s]).sum();
                }
            }
            let u_free: Vec<f64> = (0..n)
                .map(|r| offset[r] + (0..m).map(|k| jac[r * m + k] * d_free[k]).sum::<f64>())
                .collect();
            Self {
                m,
                d_free,
                w,
                u_free,
                lo,
                hi,
                scale,
                y,
            }
        }

        fn problem(&self) -> BoxProblem<'_> {
            BoxProblem {
                w: &self.w,
                u_free: &self.u_free,
                lo: &self.lo,
                hi: &self.hi,
                scale: &self.scale,
                motion: &self.y,
                motion_tolerance: 1e-14,
            }
        }

        /// The motion multipliers `nu` produce: `d_free + Y ν`.
        fn motion(&self, nu: &[f64]) -> Vec<f64> {
            let n = self.n();
            (0..self.m)
                .map(|i| self.d_free[i] + (0..n).map(|r| self.y[i * n + r] * nu[r]).sum::<f64>())
                .collect()
        }

        /// Every split of the rows into inside, on the lower bound and on the
        /// upper, solved and checked; the motion of the first that meets
        /// every condition, or `None` when none does.
        fn enumerated(&self) -> Option<Vec<f64>> {
            let n = self.n();
            let mut least = LeastNorm::new();
            let total = 3usize.pow(n as u32);
            for code in 0..total {
                let mut states = Vec::with_capacity(n);
                let mut c = code;
                for _ in 0..n {
                    states.push(c % 3);
                    c /= 3;
                }
                // 0 inside, 1 at lo, 2 at hi; an infinite bound is no state.
                if (0..n).any(|i| {
                    (states[i] == 1 && !self.lo[i].is_finite())
                        || (states[i] == 2 && !self.hi[i].is_finite())
                }) {
                    continue;
                }
                let mut nu: Vec<f64> = (0..n)
                    .map(|i| match states[i] {
                        1 => self.lo[i],
                        2 => self.hi[i],
                        _ => 0.0,
                    })
                    .collect();
                let stuck: Vec<usize> = (0..n).filter(|&i| states[i] == 0).collect();
                if !stuck.is_empty() {
                    let k = stuck.len();
                    let mut mat = Vec::with_capacity(k * k);
                    let mut rhs = Vec::with_capacity(k);
                    for &a in &stuck {
                        rhs.push(
                            -self.u_free[a]
                                - (0..n)
                                    .filter(|j| !stuck.contains(j))
                                    .map(|j| self.w[a * n + j] * nu[j])
                                    .sum::<f64>(),
                        );
                        for &b in &stuck {
                            mat.push(self.w[a * n + b]);
                        }
                    }
                    let mut x = vec![0.0; k];
                    least.solve(&mut mat, k, k, &mut rhs, &mut x);
                    for (idx, &a) in stuck.iter().enumerate() {
                        nu[a] = x[idx];
                    }
                }
                let u: Vec<f64> = (0..n)
                    .map(|i| {
                        self.u_free[i] + (0..n).map(|j| self.w[i * n + j] * nu[j]).sum::<f64>()
                    })
                    .collect();
                let ok = (0..n).all(|i| match states[i] {
                    0 => {
                        nu[i] >= self.lo[i] - 1e-12
                            && nu[i] <= self.hi[i] + 1e-12
                            && u[i].abs() <= 1e-10
                    }
                    1 => u[i] >= -1e-12,
                    _ => u[i] <= 1e-12,
                });
                if ok {
                    return Some(self.motion(&nu));
                }
            }
            None
        }
    }

    /// C4, consistency: **the box solver against an exhaustive enumeration
    /// of active sets**, the dynamics prototype's check ported: 300 seeded
    /// problems of at most six rows, one to three coordinates each with its
    /// friction row, one to three contact rows bounded only below, redundant
    /// contacts in a quarter of them. Every split of the rows into inside, on
    /// one bound and on the other is solved and checked; where one meets
    /// every condition the problem is feasible, and the motions are
    /// compared, unique even where the multipliers are not. Where none does,
    /// the solver must say that no motion meets every row.
    ///
    /// The bound is derived. The exact pass accepts natural residuals of up
    /// to [`EXACT_RESIDUAL`] of each row's scale, so each multiplier may be
    /// off by that much, and the motion by `Σ_r |Y_ir| s_r` times it; the
    /// enumeration's own solve and the products add rounding on the same
    /// numbers, which `1e-12` of their magnitude covers.
    #[test]
    fn the_box_solver_agrees_with_an_exhaustive_enumeration_of_active_sets() {
        let mut rng = XorShift::new(0x00c4_c4c4);
        let mut solver = BoxSolver::new();
        let (mut feasible, mut infeasible, mut redundant) = (0, 0, 0);
        let mut most_sweeps = 0;
        for case in 0..300 {
            let r = Random::new(&mut rng, case);
            let n = r.n();
            assert!(n <= 6, "at most six rows");
            let mut nu = vec![0.0; n];
            let outcome = solver.solve(&r.problem(), &mut nu);
            let reference = r.enumerated();
            match (outcome, reference) {
                (BoxOutcome::Solved(report), Some(expected)) => {
                    feasible += 1;
                    most_sweeps = most_sweeps.max(report.sweeps);
                    if case % 4 == 0 && n > r.m + 1 {
                        redundant += 1;
                    }
                    let got = r.motion(&nu);
                    for i in 0..r.m {
                        let sensitivity: f64 =
                            (0..n).map(|k| r.y[i * n + k].abs() * r.scale[k]).sum();
                        let magnitude = r.d_free[i].abs()
                            + (0..n).map(|k| (r.y[i * n + k] * nu[k]).abs()).sum::<f64>();
                        let bound = EXACT_RESIDUAL * sensitivity + 1e-12 * (1.0 + magnitude);
                        assert!(
                            (got[i] - expected[i]).abs() <= bound,
                            "case {case}: motion {got:?}, enumeration {expected:?}, report \
                             {report:?}"
                        );
                    }
                }
                (BoxOutcome::Infeasible { .. }, None) => infeasible += 1,
                (outcome, reference) => {
                    panic!(
                        "case {case}: the solver says {outcome:?}, the enumeration {reference:?}"
                    )
                }
            }
        }
        assert!(feasible > 200, "only {feasible} feasible problems");
        assert!(infeasible > 0, "no problem was infeasible");
        assert!(
            redundant > 20,
            "only {redundant} problems had redundant rows"
        );
        assert!(most_sweeps < SWEEP_CAP, "a solve ran to the cap");
    }

    /// Two contacts that fight on one coordinate: the solver names both.
    #[test]
    fn two_stops_that_fight_are_named() {
        // One coordinate, metric 1; the front stop needs d ≥ 0.002, the back
        // one d ≤ −0.001.
        let jac = [1.0, -1.0];
        let gaps = [-0.002, -0.001];
        let w = [1.0, -1.0, -1.0, 1.0];
        let u_free = gaps;
        let p = BoxProblem {
            w: &w,
            u_free: &u_free,
            lo: &[0.0, 0.0],
            hi: &[f64::INFINITY, f64::INFINITY],
            scale: &[1.0, 1.0],
            motion: &jac,
            motion_tolerance: 1e-15,
        };
        let mut nu = [0.0, 0.0];
        match BoxSolver::new().solve(&p, &mut nu) {
            BoxOutcome::Infeasible { rows, .. } => assert_eq!(rows, vec![0, 1]),
            other => panic!("{other:?}"),
        }
    }

    /// A row whose gradient vanishes is skipped and reported, and the rest
    /// solve as if it were not there.
    #[test]
    fn a_row_that_cannot_act_is_skipped() {
        let w = [2.0, 0.0, 0.0, 0.0];
        let p = BoxProblem {
            w: &w,
            u_free: &[-1.0, 0.5],
            lo: &[0.0, 0.0],
            hi: &[f64::INFINITY, f64::INFINITY],
            scale: &[1.0, 1.0],
            motion: &[0.5, 0.0],
            motion_tolerance: 1e-15,
        };
        let mut nu = [0.0, 0.0];
        let BoxOutcome::Solved(report) = BoxSolver::new().solve(&p, &mut nu) else {
            panic!("solvable")
        };
        assert_eq!(report.idle, 1);
        assert!(report.exact);
        assert!((nu[0] - 0.5).abs() < 1e-15 && nu[1] == 0.0, "{nu:?}");
    }

    /// The same problem gives the same bits.
    #[test]
    fn the_same_problem_gives_the_same_bits() {
        let mut rng = XorShift::new(0x0000_b0b0);
        for case in 0..40 {
            let r = Random::new(&mut rng, case);
            let (mut a, mut b) = (vec![0.0; r.n()], vec![0.0; r.n()]);
            let (oa, ob) = (
                BoxSolver::new().solve(&r.problem(), &mut a),
                BoxSolver::new().solve(&r.problem(), &mut b),
            );
            assert_eq!(oa, ob);
            for (x, y) in a.iter().zip(&b) {
                assert_eq!(x.to_bits(), y.to_bits());
            }
        }
    }
}

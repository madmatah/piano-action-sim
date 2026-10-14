//! One load increment: where a mechanism comes to rest from where it stands,
//! under new loads, against rigid contacts, with dry friction at its axes.
//!
//! # What one increment solves
//!
//! From the state `q_k`, under the loads `L`, the next state is a local
//! minimiser, reached from `q_k` by descent, of
//!
//! ```text
//! Π(q) = V(q; L) + Σ_i b_i |q_i − q_k,i|
//! subject to   g_r(q) ≥ 0   for every contact row and every coordinate limit
//!              q_p = value  for every prescribed coordinate
//! ```
//!
//! `V` the potential of gravity and the loads, `b_i` each coordinate's dry
//! friction bound. Its stationarity is the equilibrium the trade's protocols
//! probe: `0 ∈ ∇V − Σ λ_r ∇g_r + b ⊙ ∂|q − q_k|`, each `λ_r ≥ 0` complementary
//! to its gap. So each axis carries a friction within `±b_i`, and a
//! coordinate whose friction is below its bound has not moved:
//! `q_i = q_k,i` exactly. Friction enters as a dissipation, which is
//! set-valued by nature, never as a `sign()` or a `tanh()`
//! (`docs/FOUNDATIONS.md` §3.5); contacts are constraints; inertia is
//! dropped (§4.2).
//!
//! Dissipation is summed along the path, not along the chord: once an iterate
//! has moved some point of the mechanism further than the row margin from the
//! anchor `q_k`, the anchor moves on to it and the increment goes on under the
//! same loads. While each frictional coordinate moves one way, as the key's
//! one coordinate does, this changes no stationary point; it is what will
//! keep a contact's friction honest where its slip reverses.
//!
//! # How: proximal sequential quadratic steps
//!
//! Each iteration steps `q ← q + d`, with
//!
//! ```text
//! d = argmin ½ dᵀ (H + ρ P) d + ∇V(q)ᵀ d + Σ_i b_i |q_i + d_i − q_k,i|
//!     subject to   g_r(q) + ∇g_r(q)ᵀ d ≥ 0
//! ```
//!
//! whose dual is a box problem ([`super::boxed`]): the linearized gaps and
//! limits bounded `[0, +∞)`, one row per frictional coordinate bounded
//! `[−b_i, b_i]`.
//!
//! - **`H`**, the stiffness the energy terms declare, is zero: no spring and
//!   no compliant contact law exists yet, and gravity's curvature, negative
//!   under every load the protocols put on a key, never enters. So `A = ρP`,
//!   diagonal.
//! - **`P`** is geometric, never a mass: for an angle the square of its
//!   *reach*, the farthest a point of what it turns stands from its axis; for
//!   a length, one. So `P d²` is the square of how far the step moves the
//!   farthest point, and `ρ` a stiffness in newtons per metre.
//! - **`ρ`** starts at the load scale over the row margin, a step of about
//!   one margin; it is divided by [`LOWER`] when a step is accepted,
//!   multiplied by [`RAISE`] when one is rejected, and raised, in proportion
//!   to how far a step overshoots and then at least fourfold, until no point
//!   of the mechanism moves further than every declared pair and every limit
//!   allows: a pair with a row within [`ROW_MARGIN`] by the margin, a pair
//!   further off until it stands one margin apart. No feature can be stepped
//!   over: a pair is read row by row before it comes within the margin.
//! - **An overlap deeper than a step can undo**, as a start from the nominal
//!   pose may hold, is asked to shrink by half of what a step within the
//!   margin can undo rather than to close at once; the overlap is then
//!   resolved over several steps, each within the margin, and the converged
//!   state, which has none, is unchanged by it.
//! - **A step is accepted** when it lowers the merit
//!   `φ = Π + μ Σ max(0, −g_r)`, every row's gap a length (a limit's taken at
//!   its coordinate's reach), `μ` twice the largest contact force of the last
//!   subproblem; or raises it by no more than its rounding,
//!   [`MERIT_ROUNDING`] units of `ε` in the size of its terms. A key just
//!   past its breakaway starts with steps whose energy is below that
//!   rounding, and compared bit for bit, rounding alone would decide them.
//!
//! At a fixed point `d = 0` the proximal term vanishes from the optimality
//! conditions, which are then the increment's own: whatever `ρ` and `P`, the
//! answer is a stationary point of `Π`. They shape the path only.
//!
//! # Converged, under a cap
//!
//! An increment has converged when the step moves no point by more than
//! [`STEP_TOLERANCE`], every gap is at least `−`[`GAP_TOLERANCE`], every row
//! carrying a load is closed to within [`GAP_TOLERANCE`], the stationarity
//! residual on each coordinate, over its reach, is at most
//! [`RESIDUAL_TOLERANCE`] times the load scale, and the step's box problem
//! was solved, not left at its cap of sweeps: the residual is the step's
//! own metric times the step, small whenever the step is, and only
//! multipliers that solve their box problem make it an equilibrium's. The
//! residual cannot see a load on a row standing open: under a metric many
//! accepted steps have softened, the box problem's tolerance reads rows no
//! motion closes together as closed while they stand microns open, and the
//! loads balance to the rounding. A still step from such a state is turned
//! back, which stiffens the metric until the rows close or let go. Where
//! the step and the gaps pass and the residual does not, the residual is
//! measured again with every gap already closed to within the tolerance taken
//! as closed: the step's own includes the force of closing what rounding
//! leaves of a gap, a few hundredths of a femtometre, which through a row
//! with a centimetre's arm, at the metric an increment starts with, stands
//! above the tolerance without being a force out of balance. It is
//! refused with a code otherwise: [`Refusal::NotConverged`] at
//! [`ITERATION_CAP`] steps; [`Refusal::NoEquilibrium`] when the rows admit no
//! motion (two stops that fight) or the state leans on a coordinate's limit,
//! the edge of the modelled world. A metrological path never shows an
//! unconverged number.
//!
//! # Determinism
//!
//! IEEE-754 operations only; rows in the mechanism's order of pairs and
//! features, then the limits and the friction rows in coordinate order;
//! every tie to the lowest index; caps and tolerances frozen here. The same
//! mechanism, state and loads give the same bits on every machine.

use super::boxed::{BoxOutcome, BoxProblem, BoxSolver, RowState};
use super::report::SolveReport;
use crate::contact::{Feature, Rows, ROW_MARGIN};
use crate::geometry::{Point, Seg};
use crate::mechanism::{
    BodyId, CoordId, CoordKind, MaterialPoint, Mechanism, PointLoad, Pose, Prescribed, Problem,
};
use crate::G;

/// The most proximal steps one increment takes.
pub const ITERATION_CAP: u32 = 100;

/// The step under which an increment has stopped moving, metres: as far as
/// the farthest point of the mechanism may still move.
pub const STEP_TOLERANCE: f64 = 1e-12;

/// How far past closing a gap may stand at a converged state, metres.
pub const GAP_TOLERANCE: f64 = 1e-12;

/// The stationarity residual left on a coordinate, over its reach, as a
/// fraction of the load scale.
pub const RESIDUAL_TOLERANCE: f64 = 1e-12;

/// What `ρ` is multiplied by when a step is rejected.
pub const RAISE: f64 = 4.0;

/// What `ρ` is divided by when a step is accepted.
pub const LOWER: f64 = 3.0;

/// The most times one step may raise `ρ` to keep within the row margin.
pub(crate) const RAISES_PER_STEP: u32 = 64;

/// How far past the proportion a step too long raises `ρ`, so that rounding
/// cannot leave the next one a hair past the margin.
pub(crate) const FIT: f64 = 1.000_001;

/// The box problem's sweeps stop once one moves no point by more than this,
/// metres: a thousandth of [`STEP_TOLERANCE`].
pub(crate) const SWEEP_TOLERANCE: f64 = 1e-15;

/// How far a step's merit may stand above the merit it starts from and the
/// step still count as not raising it, in units of `ε` times the size of the
/// two merits' terms: eight.
///
/// The merit is a sum of a few energies, heights times weights measured from
/// the keybed, each rounded at its last bit; two merits closer than a few
/// units in the last place of their terms are one merit as far as they can be
/// computed. A key loaded a hundred-thousandth of a gram past its breakaway
/// starts moving with a step whose energy, the excess force times a
/// picometre, is a hundredth of that rounding: compared bit for bit, rounding
/// alone decides it, each step it turns back is a step a quarter as long, and
/// the key never leaves its seat.
pub(crate) const MERIT_ROUNDING: f64 = 8.0;

/// Whether the axes' dry friction acts in an increment.
///
/// Off for a procedure that seats a part by rule, and for the
/// initialization that finds a state needing no friction to hold; on for
/// every increment that follows the state a user leaves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Friction {
    /// Each coordinate's bound acts.
    On,
    /// Every bound is zero.
    Off,
}

/// A row that holds the mechanism, besides its joints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Holder {
    /// A contact row: one pair of features of one declared pair.
    Contact {
        /// The pair, an index into the mechanism's pairs.
        pair: u16,
        /// Which features meet.
        feature: Feature,
    },
    /// A coordinate's limit, the edge of the modelled world.
    Limit {
        /// The coordinate.
        coord: CoordId,
        /// Its upper limit, or its lower.
        upper: bool,
    },
}

/// A row of the last subproblem: its gap, and the force it carries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Support {
    /// The row.
    pub by: Holder,
    /// Its gap at the state, metres; a limit's taken at its coordinate's
    /// reach.
    pub gap: f64,
    /// The force it pushes with, newtons: zero on a row standing open.
    pub force: f64,
    /// Which way a contact pushes side A, the unit normal from side B
    /// towards it, in the world; zero for a limit.
    pub normal: Point,
}

/// Where an increment came to rest, and what holds it there.
#[derive(Debug, Clone, PartialEq)]
pub struct Equilibrium {
    /// The coordinates.
    pub q: Vec<f64>,
    /// The rows of the last subproblem, contacts first in the mechanism's
    /// order, then limits. Their forces are one admissible split: where rows
    /// are redundant, the split of least utilisation.
    pub supports: Vec<Support>,
    /// The generalized force each coordinate's friction applies, one per
    /// coordinate: newton-metres on an angle, newtons on a length.
    pub friction: Vec<f64>,
    /// The generalized force each prescription applies, one per coordinate,
    /// zero on a free one. On a prescribed coordinate that carries friction
    /// the true reaction lies within its bound of this.
    pub reactions: Vec<f64>,
    /// How the increment went.
    pub report: SolveReport,
}

/// Why an increment cannot be solved, or has no answer.
#[derive(Debug, Clone, PartialEq)]
pub enum Refusal {
    /// The mechanism, its prescriptions or its loads cannot be solved as
    /// given.
    Mechanism(Problem),
    /// No equilibrium in the modelled world: the rows named admit no motion
    /// (two stops that fight), or the state comes to rest leaning on a
    /// coordinate's limit, where nothing modelled stands.
    NoEquilibrium {
        /// The rows that fight, or the limit leaned on.
        on: Vec<Holder>,
    },
    /// The cap was reached first.
    NotConverged {
        /// Steps taken.
        iterations: u32,
        /// The stationarity residual left, newtons.
        residual: f64,
    },
}

/// What one row of a subproblem is.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Kind {
    /// The `i`-th contact row.
    Contact(usize),
    /// A limit of the `k`-th free coordinate.
    Limit { k: usize, upper: bool },
    /// The friction of the `k`-th free coordinate.
    Friction { k: usize },
}

/// A point the mechanism carries, for the reach: a vertex of a drawn part, an
/// arc's centre with its radius, a centre of mass, a load's point.
#[derive(Debug, Clone, Copy)]
struct Reached {
    body: BodyId,
    at: Point,
    radius: f64,
}

/// The static solver, with its workspace.
///
/// Kept between increments, so that an increment of a size already seen
/// allocates no buffer anew.
#[derive(Debug, Clone)]
pub struct Statics {
    metric_scale: f64,
    rows: Rows,
    trial_rows: Rows,
    boxed: BoxSolver,
    kinds: Vec<Kind>,
    jac: Vec<f64>,
    offsets: Vec<f64>,
    gaps: Vec<f64>,
    lo: Vec<f64>,
    hi: Vec<f64>,
    scale: Vec<f64>,
    y: Vec<f64>,
    w: Vec<f64>,
    u_free: Vec<f64>,
    motion: Vec<f64>,
    nu: Vec<f64>,
    u_closed: Vec<f64>,
    nu_closed: Vec<f64>,
    /// Where the last solve refused as not converged left its iterate, and
    /// how it went: what a caller continuing the same increment starts from.
    stopped: (Vec<f64>, SolveReport),
}

impl Default for Statics {
    fn default() -> Self {
        Self::new()
    }
}

/// The load scale: every weight and every applied force, by magnitude,
/// added up, newtons; one newton where nothing acts.
fn load_scale(m: &Mechanism, loads: &[PointLoad]) -> f64 {
    let mut s = 0.0;
    for body in m.bodies() {
        if let Some(mass) = body.mass {
            s += mass.mass * G;
        }
    }
    for load in loads {
        s += (load.force.x * load.force.x + load.force.y * load.force.y).sqrt();
    }
    if s > 0.0 {
        s
    } else {
        1.0
    }
}

/// Every point the mechanism carries, in its body's frame.
fn reached_points(m: &Mechanism, loads: &[PointLoad]) -> Vec<Reached> {
    let mut out = Vec::new();
    for part in m.parts() {
        for seg in part.silhouette() {
            match seg {
                Seg::Line { a, .. } => out.push(Reached {
                    body: part.body,
                    at: a,
                    radius: 0.0,
                }),
                Seg::Arc { c, r, .. } => out.push(Reached {
                    body: part.body,
                    at: c,
                    radius: r,
                }),
            }
        }
    }
    for (i, body) in m.bodies().iter().enumerate() {
        if let Some(mass) = body.mass {
            out.push(Reached {
                body: BodyId(i),
                at: mass.com,
                radius: 0.0,
            });
        }
    }
    for load in loads {
        out.push(Reached {
            body: load.at.body,
            at: load.at.at,
            radius: 0.0,
        });
    }
    out
}

/// Whether coordinate `c` moves body `b`: it is the joint of `b` or of one
/// of its ancestors.
fn moves(m: &Mechanism, c: usize, b: BodyId) -> bool {
    let mut body = b;
    while body != BodyId::GROUND {
        let joint = m.body(body).joint;
        if joint.coord() == Some(CoordId(c)) {
            return true;
        }
        body = m.body(body).parent;
    }
    false
}

/// How fast the farthest point moves per unit of coordinate `c`, at `pose`:
/// for an angle, its reach, metres per radian; for a length, one.
fn speed(m: &Mechanism, pose: &Pose, c: usize, points: &[Reached]) -> f64 {
    if m.coords()[c].kind == CoordKind::Length {
        return 1.0;
    }
    let Some(b) = (1..m.bodies().len()).find(|&b| m.bodies()[b].joint.coord() == Some(CoordId(c)))
    else {
        return 0.0;
    };
    let o = pose.placement(BodyId(b)).origin;
    let mut reach = 0.0_f64;
    for p in points {
        if moves(m, c, p.body) {
            let w = pose.world(MaterialPoint {
                body: p.body,
                at: p.at,
            });
            let (dx, dy) = (w.x - o.x, w.y - o.y);
            reach = reach.max((dx * dx + dy * dy).sqrt() + p.radius);
        }
    }
    reach
}

impl Statics {
    /// A solver with the geometric metric of the module documentation.
    pub fn new() -> Self {
        Self {
            metric_scale: 1.0,
            rows: Rows::new(),
            trial_rows: Rows::new(),
            boxed: BoxSolver::new(),
            kinds: Vec::new(),
            jac: Vec::new(),
            offsets: Vec::new(),
            gaps: Vec::new(),
            lo: Vec::new(),
            hi: Vec::new(),
            scale: Vec::new(),
            y: Vec::new(),
            w: Vec::new(),
            u_free: Vec::new(),
            motion: Vec::new(),
            nu: Vec::new(),
            u_closed: Vec::new(),
            nu_closed: Vec::new(),
            stopped: (Vec::new(), SolveReport::NOTHING),
        }
    }

    /// Where the last solve refused as not converged stopped: its last
    /// iterate, one value per coordinate, and its report. The same increment
    /// continued from there, under the same loads, goes on along the path,
    /// its friction's anchor moved on to that iterate as the solve itself
    /// moves it. Empty before any such refusal.
    pub fn stopped(&self) -> (&[f64], SolveReport) {
        (&self.stopped.0, self.stopped.1)
    }

    /// A solver whose metric `P` is multiplied by `k`: the path changes, the
    /// stationary point does not. For the tests that hold that to be so.
    pub fn with_metric_scaled(k: f64) -> Self {
        Self {
            metric_scale: k,
            ..Self::new()
        }
    }

    /// Solves one load increment of `m` from `from`, under `loads`, with
    /// `prescribed` coordinates held at their values and `friction` on or
    /// off.
    ///
    /// # Panics
    ///
    /// When `from` does not hold one value per coordinate.
    pub fn solve(
        &mut self,
        m: &Mechanism,
        from: &[f64],
        loads: &[PointLoad],
        prescribed: &[Prescribed],
        friction: Friction,
    ) -> Result<Equilibrium, Refusal> {
        let n = m.coords().len();
        assert_eq!(from.len(), n, "a state holds one value per coordinate");
        let refuse = |p: Problem| Err(Refusal::Mechanism(p));

        for (i, load) in loads.iter().enumerate() {
            if load.at.body.0 >= m.bodies().len() {
                return refuse(Problem::NoSuchBody { body: load.at.body });
            }
            let numbers = [load.at.at.x, load.at.at.y, load.force.x, load.force.y];
            if !numbers.iter().all(|x| x.is_finite()) {
                return refuse(Problem::LoadNotFinite { load: i });
            }
        }
        let mut q = from.to_vec();
        if let Some(c) = q.iter().position(|x| !x.is_finite()) {
            return refuse(Problem::StartNotFinite { coord: CoordId(c) });
        }
        let mut free = vec![true; n];
        for p in prescribed {
            let c = p.coord.0;
            if c >= n {
                return refuse(Problem::NoSuchCoordinate { coord: p.coord });
            }
            if !free[c] {
                return refuse(Problem::PrescribedTwice { coord: p.coord });
            }
            if !p.value.is_finite() {
                return refuse(Problem::PrescribedNotFinite { coord: p.coord });
            }
            q[c] = p.value;
            free[c] = false;
        }
        let free_ids: Vec<usize> = (0..n).filter(|&c| free[c]).collect();
        for &c in &free_ids {
            let (lo, hi) = m.coords()[c].limits;
            if !(lo.is_finite() && hi.is_finite()) {
                return refuse(Problem::Unlimited { coord: CoordId(c) });
            }
        }
        let nf = free_ids.len();
        let bounds: Vec<f64> = free_ids
            .iter()
            .map(|&c| match friction {
                Friction::On => m.coords()[c].friction,
                Friction::Off => 0.0,
            })
            .collect();
        let load = load_scale(m, loads);
        let points = reached_points(m, loads);

        let mut report = SolveReport {
            increments: 1,
            ..SolveReport::NOTHING
        };
        let mut anchor = q.clone();
        let mut rho = load / ROW_MARGIN;
        let mut gf = vec![0.0; n];
        let mut pose = m.pose(&q);
        m.rows(&pose, f64::INFINITY, &mut self.rows);

        loop {
            m.generalized_forces(&pose, loads, &mut gf);
            let speeds: Vec<f64> = free_ids
                .iter()
                .map(|&c| speed(m, &pose, c, &points))
                .collect();
            if let Some(k) = speeds.iter().position(|&s| s.is_nan() || s <= 0.0) {
                return refuse(Problem::MovesNothing {
                    coord: CoordId(free_ids[k]),
                });
            }

            self.assemble(m, &q, &anchor, &free_ids, &speeds, &bounds, load);
            let nr = self.kinds.len();
            let reach = self.step_bound(m, &pose, &q, &free_ids, &speeds);
            let mut d = vec![0.0; nf];
            let mut step;
            let mut raises = 0;
            // Whether the last box problem was solved, its multipliers then
            // meeting every condition of it: a step its sweeps left at their
            // cap certifies no equilibrium, however short it is.
            let mut solved;
            loop {
                let a: Vec<f64> = (0..nf)
                    .map(|k| {
                        let s = speeds[k];
                        rho * self.metric_scale * s * s
                    })
                    .collect();
                // Y = A⁻¹ Jᵀ, W = J Y, u_free = offsets + J A⁻¹(−∇V).
                self.y.clear();
                self.y.resize(nf * nr, 0.0);
                for (k, &ak) in a.iter().enumerate() {
                    for r in 0..nr {
                        self.y[k * nr + r] = self.jac[r * nf + k] / ak;
                    }
                }
                self.w.clear();
                self.w.resize(nr * nr, 0.0);
                for r in 0..nr {
                    for s in 0..nr {
                        let mut v = 0.0;
                        for k in 0..nf {
                            v += self.jac[r * nf + k] * self.y[k * nr + s];
                        }
                        self.w[r * nr + s] = v;
                    }
                }
                let d_free: Vec<f64> = (0..nf).map(|k| gf[free_ids[k]] / a[k]).collect();
                self.u_free.clear();
                for r in 0..nr {
                    let mut v = self.offsets[r];
                    for (k, &dk) in d_free.iter().enumerate() {
                        v += self.jac[r * nf + k] * dk;
                    }
                    self.u_free.push(v);
                }
                self.motion.clear();
                for (k, &sk) in speeds.iter().enumerate() {
                    for r in 0..nr {
                        self.motion.push(self.y[k * nr + r] * sk);
                    }
                }
                self.nu.clear();
                self.nu.resize(nr, 0.0);
                let problem = BoxProblem {
                    w: &self.w,
                    u_free: &self.u_free,
                    lo: &self.lo,
                    hi: &self.hi,
                    scale: &self.scale,
                    motion: &self.motion,
                    motion_tolerance: SWEEP_TOLERANCE,
                };
                let outcome = self.boxed.solve(&problem, &mut self.nu);
                let box_report = match outcome {
                    BoxOutcome::Solved(r) => r,
                    BoxOutcome::Infeasible { rows, .. } => {
                        return Err(Refusal::NoEquilibrium {
                            on: rows
                                .iter()
                                .filter_map(|&r| self.holder(r, &free_ids))
                                .collect(),
                        });
                    }
                };
                report.sweeps += box_report.sweeps;
                report.exact_passes += box_report.exact_passes;
                report.exact &= box_report.exact;
                solved = box_report.converged;
                for k in 0..nf {
                    let mut v = d_free[k];
                    for r in 0..nr {
                        v += self.y[k * nr + r] * self.nu[r];
                    }
                    d[k] = v;
                }
                // A coordinate friction holds still has not moved from its
                // anchor: by the solution, not by rounding.
                if box_report.exact {
                    for (r, kind) in self.kinds.iter().enumerate() {
                        if let Kind::Friction { k } = *kind {
                            if self.boxed.states()[r] == RowState::Inside {
                                let c = free_ids[k];
                                d[k] = anchor[c] - q[c];
                            }
                        }
                    }
                }
                step = (0..nf).map(|k| speeds[k] * d[k].abs()).sum::<f64>();
                if step <= reach {
                    break;
                }
                // Raised in proportion first: a step free of its rows shrinks
                // as 1/ρ, so it lands on the margin. One a closed row holds
                // on its contact does not shrink until the row lets it go, so
                // a second raise is at least fourfold.
                let proportion = FIT * step / reach;
                rho *= if raises == 0 {
                    proportion
                } else {
                    proportion.max(RAISE)
                };
                raises += 1;
                if raises > RAISES_PER_STEP {
                    self.stopped = (q, report);
                    return Err(Refusal::NotConverged {
                        iterations: report.iterations,
                        residual: f64::INFINITY,
                    });
                }
            }

            // The stationarity residual at q, with this subproblem's
            // multipliers on the contacts and limits, and on each axis the
            // friction its set allows.
            let residual = self.residual(&gf, &free_ids, &speeds, &q, &anchor, &bounds, &self.nu);
            let least_gap = self
                .kinds
                .iter()
                .enumerate()
                .filter(|(_, k)| !matches!(k, Kind::Friction { .. }))
                .map(|(r, _)| self.gaps[r])
                .fold(f64::INFINITY, f64::min);
            report.residual = residual;
            // Every row carrying a load closed: a load on a row standing open
            // is no equilibrium, however small the residual it balances.
            let loaded_open = self.loaded_gap(&self.nu, load) > GAP_TOLERANCE;
            let still = solved && step <= STEP_TOLERANCE && least_gap >= -GAP_TOLERANCE;
            if still && !loaded_open && residual <= RESIDUAL_TOLERANCE * load {
                report.most_iterations = report.iterations;
                return self.settled(m, q, &anchor, &free_ids, &bounds, &gf, report);
            }
            // Still, but the residual above its tolerance: it may be only the
            // force of closing gaps already closed to within the tolerance.
            if still {
                if let Some((closed, exact)) = self.with_gaps_closed(
                    &gf,
                    &free_ids,
                    &speeds,
                    &q,
                    &anchor,
                    &bounds,
                    rho,
                    &mut report,
                ) {
                    if closed <= RESIDUAL_TOLERANCE * load
                        && self.loaded_gap(&self.nu_closed, load) <= GAP_TOLERANCE
                    {
                        core::mem::swap(&mut self.nu, &mut self.nu_closed);
                        report.exact &= exact;
                        report.residual = closed;
                        report.most_iterations = report.iterations;
                        return self.settled(m, q, &anchor, &free_ids, &bounds, &gf, report);
                    }
                }
            }

            // The trial point and the merit test.
            let mut trial = q.clone();
            for k in 0..nf {
                trial[free_ids[k]] += d[k];
            }
            let mu = 2.0
                * self
                    .kinds
                    .iter()
                    .enumerate()
                    .filter(|(_, k)| !matches!(k, Kind::Friction { .. }))
                    .map(|(r, _)| self.nu[r])
                    .fold(0.0_f64, f64::max);
            let trial_pose = m.pose(&trial);
            m.rows(&trial_pose, f64::INFINITY, &mut self.trial_rows);
            let (here, here_size) = merit(
                m, &pose, &self.rows, &q, &anchor, &free_ids, &speeds, &bounds, loads, mu,
            );
            let (there, there_size) = merit(
                m,
                &trial_pose,
                &self.trial_rows,
                &trial,
                &anchor,
                &free_ids,
                &speeds,
                &bounds,
                loads,
                mu,
            );
            report.iterations += 1;

            // A step that moves nothing from a state some gap still overlaps,
            // or where a row standing open carries a load, is no progress,
            // however it scores: the box problem accepts its multipliers to a
            // fraction of their scale, which a soft metric turns into
            // nanometres of overlap, or microns of gap left open under a
            // contact that pushes, unresolved step after step while the
            // metric only softens. Only a stiffer one tightens it.
            let stalled = step <= STEP_TOLERANCE && (least_gap < -GAP_TOLERANCE || loaded_open);
            let rounding = MERIT_ROUNDING * f64::EPSILON * (here_size + there_size);

            if there <= here + rounding && !stalled {
                q = trial;
                pose = trial_pose;
                core::mem::swap(&mut self.rows, &mut self.trial_rows);
                rho /= LOWER;
                let travelled: f64 = free_ids
                    .iter()
                    .enumerate()
                    .map(|(k, &c)| speeds[k] * (q[c] - anchor[c]).abs())
                    .sum();
                if travelled > ROW_MARGIN {
                    anchor.copy_from_slice(&q);
                    report.anchors += 1;
                }
            } else {
                rho *= RAISE;
                report.rejected += 1;
            }
            if report.iterations >= ITERATION_CAP {
                report.most_iterations = report.iterations;
                self.stopped = (q, report);
                return Err(Refusal::NotConverged {
                    iterations: report.iterations,
                    residual,
                });
            }
        }
    }

    /// The rows of the subproblem at `q`: every contact row within the margin,
    /// every limit within it, and a friction row for each free coordinate
    /// with a bound.
    #[allow(clippy::too_many_arguments)]
    fn assemble(
        &mut self,
        m: &Mechanism,
        q: &[f64],
        anchor: &[f64],
        free_ids: &[usize],
        speeds: &[f64],
        bounds: &[f64],
        load: f64,
    ) {
        let nf = free_ids.len();
        self.kinds.clear();
        self.jac.clear();
        self.offsets.clear();
        self.gaps.clear();
        self.lo.clear();
        self.hi.clear();
        self.scale.clear();
        let mut push = |kind: Kind, grad: &dyn Fn(usize) -> f64, offset: f64, lo, hi, scale| {
            self.kinds.push(kind);
            let start = self.jac.len();
            for k in 0..nf {
                self.jac.push(grad(k));
            }
            self.gaps.push(offset);
            // An overlap deeper than half of what a step within the margin can
            // undo is asked to shrink by that half, not to close at once: a
            // step that closed it would move some point further than the
            // margin, which no raise of ρ could shorten.
            let undo = if matches!(kind, Kind::Friction { .. }) {
                f64::INFINITY
            } else {
                (0..nf)
                    .map(|k| self.jac[start + k].abs() / speeds[k])
                    .fold(0.0_f64, f64::max)
                    * ROW_MARGIN
            };
            self.offsets.push(if offset < -0.5 * undo {
                -0.5 * undo
            } else {
                offset
            });
            self.lo.push(lo);
            self.hi.push(hi);
            self.scale.push(scale);
        };
        for (i, row) in self.rows.rows().iter().enumerate() {
            if row.gap > ROW_MARGIN {
                continue;
            }
            let g = self.rows.grad(i);
            push(
                Kind::Contact(i),
                &|k| g[free_ids[k]],
                row.gap,
                0.0,
                f64::INFINITY,
                load,
            );
        }
        for (k, &c) in free_ids.iter().enumerate() {
            let (lo, hi) = m.coords()[c].limits;
            let s = speeds[k];
            let below = s * (q[c] - lo);
            if below <= ROW_MARGIN {
                push(
                    Kind::Limit { k, upper: false },
                    &|j| if j == k { s } else { 0.0 },
                    below,
                    0.0,
                    f64::INFINITY,
                    load,
                );
            }
            let above = s * (hi - q[c]);
            if above <= ROW_MARGIN {
                push(
                    Kind::Limit { k, upper: true },
                    &|j| if j == k { -s } else { 0.0 },
                    above,
                    0.0,
                    f64::INFINITY,
                    load,
                );
            }
        }
        for (k, &c) in free_ids.iter().enumerate() {
            let b = bounds[k];
            if b > 0.0 {
                push(
                    Kind::Friction { k },
                    &|j| if j == k { 1.0 } else { 0.0 },
                    q[c] - anchor[c],
                    -b,
                    b,
                    b,
                );
            }
        }
    }

    /// The generalized force on free coordinate `k` of the loads and of the
    /// contacts and limits, their multipliers `nu`: all but its friction.
    fn loaded(&self, gf: &[f64], free_ids: &[usize], k: usize, nu: &[f64]) -> f64 {
        let nf = free_ids.len();
        let mut v = gf[free_ids[k]];
        for (r, kind) in self.kinds.iter().enumerate() {
            if !matches!(kind, Kind::Friction { .. }) {
                v += self.jac[r * nf + k] * nu[r];
            }
        }
        v
    }

    /// How far open the widest contact or limit row stands at the state
    /// among those its multipliers `nu` load, metres; zero where none is
    /// loaded. A row is loaded when its multiplier passes the residual's
    /// tolerance of the load scale `load`: a force under that, applied
    /// through a row whose gradient over the reach is at most one, moves the
    /// residual by no more than its tolerance, and the residual cannot tell
    /// it from nothing.
    ///
    /// What the convergence test reads besides the residual. The box problem
    /// accepts its multipliers to a part in 10⁹ of their scale, and under the
    /// soft metric an increment ends on, after many accepted steps have
    /// softened it, that tolerance holds a row's gap to nothing: rows that no
    /// motion can close together are read as closed, their loads balancing
    /// the key to the rounding while they stand microns open. Such a state
    /// has a residual of nothing and no equilibrium: a contact pushes only
    /// where it touches.
    fn loaded_gap(&self, nu: &[f64], load: f64) -> f64 {
        self.kinds
            .iter()
            .enumerate()
            .filter(|&(r, kind)| {
                !matches!(kind, Kind::Friction { .. }) && nu[r] > RESIDUAL_TOLERANCE * load
            })
            .map(|(r, _)| self.gaps[r])
            .fold(0.0_f64, f64::max)
    }

    /// The friction an axis applies at a state of the increment, its bound
    /// `b`, the rest of the generalized force on it `loaded`, and `moved` how
    /// far it stands from the anchor: set-valued, as the increment defines
    /// it. An axis that has moved has its friction on the bound against the
    /// way it moved; one that has not takes whatever within its bound
    /// balances the rest, and the nearer bound past that. Never a sign of a
    /// velocity: a displacement's side, and a box.
    fn axis_friction(loaded: f64, moved: f64, b: f64) -> f64 {
        if moved > 0.0 {
            -b
        } else if moved < 0.0 {
            b
        } else {
            (-loaded).clamp(-b, b)
        }
    }

    /// The stationarity residual at `q`, over each coordinate's reach: what
    /// is left of the generalized force once the contacts and limits push
    /// with `nu` and each axis's friction takes what its set allows
    /// ([`Self::axis_friction`]).
    ///
    /// Not the friction rows' own multipliers: the box problem places a
    /// multiplier within a part in 10⁹ of its bound on the bound
    /// ([`super::boxed::ON_A_BOUND`]), and an axis held still by a friction
    /// that much short of its bound would carry the difference as a force
    /// out of balance, far past the residual's tolerance, at a state that
    /// is an equilibrium.
    #[allow(clippy::too_many_arguments)]
    fn residual(
        &self,
        gf: &[f64],
        free_ids: &[usize],
        speeds: &[f64],
        q: &[f64],
        anchor: &[f64],
        bounds: &[f64],
        nu: &[f64],
    ) -> f64 {
        let mut residual = 0.0_f64;
        for (k, &c) in free_ids.iter().enumerate() {
            let loaded = self.loaded(gf, free_ids, k, nu);
            let left = loaded + Self::axis_friction(loaded, q[c] - anchor[c], bounds[k]);
            residual = residual.max(left.abs() / speeds[k]);
        }
        residual
    }

    /// The stationarity residual at the current iterate, over each
    /// coordinate's reach, with every contact and limit whose gap is closed
    /// to within [`GAP_TOLERANCE`] taken as closed; `None` where its box
    /// problem is not solved, or its step moves some point further than
    /// [`STEP_TOLERANCE`]; with it, whether its box problem was solved
    /// exactly. Its multipliers are left in `nu_closed`, for the caller to
    /// adopt if the residual passes.
    ///
    /// The subproblem's own residual is its metric times its step, and its
    /// step closes every gap, however little is left of it. A gap closed to
    /// the rounding of positions a quarter of a metre out, some hundredths of
    /// a femtometre, closed through a row whose arm is a centimetre, moves
    /// the farthest point of the mechanism fifteen or more times as far, and
    /// at the metric an increment starts with, the force of that motion
    /// stands above a part in 10¹² of the loads. It is no force out of
    /// balance: the gap is closed as far as any gap is ever closed. So the
    /// same problem is solved again with those gaps at zero, the metric and
    /// the rows as they are, and a state still under it, with a residual
    /// within its tolerance, is an equilibrium of the increment: multipliers
    /// in their boxes balance the loads, each complementary to a gap closed
    /// to within the tolerance. The test is the same, and only the gaps
    /// already within it are read as closed. It runs only where the
    /// subproblem's own residual fails: an increment that passed on its
    /// first test still does, and one that went on to take steps of a few
    /// femtometres, until its metric had fallen far enough, now ends where it
    /// stands.
    #[allow(clippy::too_many_arguments)]
    fn with_gaps_closed(
        &mut self,
        gf: &[f64],
        free_ids: &[usize],
        speeds: &[f64],
        q: &[f64],
        anchor: &[f64],
        bounds: &[f64],
        rho: f64,
        report: &mut SolveReport,
    ) -> Option<(f64, bool)> {
        let nf = free_ids.len();
        let nr = self.kinds.len();
        let a: Vec<f64> = speeds
            .iter()
            .map(|&s| rho * self.metric_scale * s * s)
            .collect();
        let d_free: Vec<f64> = (0..nf).map(|k| gf[free_ids[k]] / a[k]).collect();
        self.u_closed.clear();
        for r in 0..nr {
            let closed = !matches!(self.kinds[r], Kind::Friction { .. })
                && self.gaps[r].abs() <= GAP_TOLERANCE;
            let mut v = if closed { 0.0 } else { self.offsets[r] };
            for (k, &dk) in d_free.iter().enumerate() {
                v += self.jac[r * nf + k] * dk;
            }
            self.u_closed.push(v);
        }
        self.nu_closed.clear();
        self.nu_closed.resize(nr, 0.0);
        let problem = BoxProblem {
            w: &self.w,
            u_free: &self.u_closed,
            lo: &self.lo,
            hi: &self.hi,
            scale: &self.scale,
            motion: &self.motion,
            motion_tolerance: SWEEP_TOLERANCE,
        };
        let BoxOutcome::Solved(solved) = self.boxed.solve(&problem, &mut self.nu_closed) else {
            return None;
        };
        report.sweeps += solved.sweeps;
        report.exact_passes += solved.exact_passes;
        if !solved.converged {
            return None;
        }
        let mut step = 0.0;
        for k in 0..nf {
            let mut d = d_free[k];
            for r in 0..nr {
                d += self.y[k * nr + r] * self.nu_closed[r];
            }
            step += speeds[k] * d.abs();
        }
        if step > STEP_TOLERANCE {
            return None;
        }
        let residual = self.residual(gf, free_ids, speeds, q, anchor, bounds, &self.nu_closed);
        Some((residual, solved.exact))
    }

    /// How far a step may move the farthest point of the mechanism, metres:
    /// the least budget of every declared pair and every limit.
    ///
    /// A pair with a row within the row margin may approach by the margin and
    /// no more, as may a limit within it: what is that near is read and
    /// linearized, and a step past the margin could step over a feature the
    /// linearization does not hold. A pair further off may approach until it
    /// stands one margin apart: by its least gap where it has rows at any
    /// distance, by the boxes round its pieces where it has none. So no pair
    /// is stepped over, and a key falling freely towards a stop a centimetre
    /// away does so in steps of a centimetre, not of a millimetre.
    fn step_bound(
        &self,
        m: &Mechanism,
        pose: &Pose,
        q: &[f64],
        free_ids: &[usize],
        speeds: &[f64],
    ) -> f64 {
        let budget = |distance: f64| {
            if distance <= ROW_MARGIN {
                ROW_MARGIN
            } else {
                (distance - ROW_MARGIN).max(ROW_MARGIN)
            }
        };
        let mut bound = f64::INFINITY;
        for p in 0..m.pairs().len() {
            let p = u16::try_from(p).expect("fewer than 65 536 pairs");
            let distance = match self.rows.gap(p) {
                Some(gap) => gap,
                None => m.pair_bound(pose, p),
            };
            bound = bound.min(budget(distance));
        }
        for (k, &c) in free_ids.iter().enumerate() {
            let (lo, hi) = m.coords()[c].limits;
            bound = bound.min(budget(speeds[k] * (q[c] - lo).min(hi - q[c])));
        }
        bound
    }

    /// The holder row `r` of the last subproblem stands for, if it is a
    /// contact or a limit.
    fn holder(&self, r: usize, free_ids: &[usize]) -> Option<Holder> {
        match self.kinds[r] {
            Kind::Contact(i) => {
                let row = self.rows.rows()[i];
                Some(Holder::Contact {
                    pair: row.pair,
                    feature: row.feature,
                })
            }
            Kind::Limit { k, upper } => Some(Holder::Limit {
                coord: CoordId(free_ids[k]),
                upper,
            }),
            Kind::Friction { .. } => None,
        }
    }

    /// The converged state, read: its supports, its friction and its
    /// reactions; or the refusal of a state leaning on a limit.
    #[allow(clippy::too_many_arguments)]
    fn settled(
        &self,
        m: &Mechanism,
        q: Vec<f64>,
        anchor: &[f64],
        free_ids: &[usize],
        bounds: &[f64],
        gf: &[f64],
        report: SolveReport,
    ) -> Result<Equilibrium, Refusal> {
        let n = m.coords().len();
        let leaning: Vec<Holder> = self
            .kinds
            .iter()
            .enumerate()
            .filter(|(r, k)| matches!(k, Kind::Limit { .. }) && self.nu[*r] > 0.0)
            .filter_map(|(r, _)| self.holder(r, free_ids))
            .collect();
        if !leaning.is_empty() {
            return Err(Refusal::NoEquilibrium { on: leaning });
        }
        let mut supports = Vec::new();
        let mut friction = vec![0.0; n];
        for (r, kind) in self.kinds.iter().enumerate() {
            match *kind {
                Kind::Friction { k } => {
                    let c = free_ids[k];
                    let loaded = self.loaded(gf, free_ids, k, &self.nu);
                    friction[c] = Self::axis_friction(loaded, q[c] - anchor[c], bounds[k]);
                }
                _ => {
                    if let Some(by) = self.holder(r, free_ids) {
                        let normal = match *kind {
                            Kind::Contact(i) => self.rows.rows()[i].normal.vector(),
                            _ => Point::new(0.0, 0.0),
                        };
                        supports.push(Support {
                            by,
                            gap: self.gaps[r],
                            force: self.nu[r],
                            normal,
                        });
                    }
                }
            }
        }
        // A prescription applies whatever balances the loads and the
        // contacts on its coordinate.
        let mut reactions = vec![0.0; n];
        for c in (0..n).filter(|c| !free_ids.contains(c)) {
            let mut v = gf[c];
            for (r, kind) in self.kinds.iter().enumerate() {
                if let Kind::Contact(i) = *kind {
                    v += self.rows.grad(i)[c] * self.nu[r];
                }
            }
            reactions[c] = -v;
        }
        Ok(Equilibrium {
            q,
            supports,
            friction,
            reactions,
            report: SolveReport {
                converged: true,
                ..report
            },
        })
    }
}

/// The merit of a state: the increment's own `Π`, and `μ` times every gap's
/// closing past zero; and the size of the terms it adds up, which its
/// rounding scales with.
#[allow(clippy::too_many_arguments)]
fn merit(
    m: &Mechanism,
    pose: &Pose,
    rows: &Rows,
    q: &[f64],
    anchor: &[f64],
    free_ids: &[usize],
    speeds: &[f64],
    bounds: &[f64],
    loads: &[PointLoad],
    mu: f64,
) -> (f64, f64) {
    let mut phi = m.potential(pose, loads);
    let mut size = m.potential_magnitude(pose, loads);
    for (k, &c) in free_ids.iter().enumerate() {
        let dissipated = bounds[k] * (q[c] - anchor[c]).abs();
        let (lo, hi) = m.coords()[c].limits;
        let s = speeds[k];
        let past = mu * ((s * (lo - q[c])).max(0.0) + (s * (q[c] - hi)).max(0.0));
        phi += dissipated;
        phi += past;
        size += dissipated + past;
    }
    for row in rows.rows() {
        let overlap = mu * (-row.gap).max(0.0);
        phi += overlap;
        size += overlap;
    }
    (phi, size)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::contact::{
        ContactCode, ContactLaw, ContactPair, Material, PairGeometry, Part, PartCode, PartId, Role,
        Surface, SurfaceRef, Sweep,
    };
    use crate::geometry::Dir;
    use crate::mechanism::fixtures::XorShift;
    use crate::mechanism::{Body, Coord, MassProps};

    fn line(a: Point, b: Point) -> Seg {
        Seg::Line { a, b }
    }

    fn part(body: usize, outline: Vec<Seg>) -> Part {
        let n = u16::try_from(outline.len()).unwrap();
        Part {
            body: BodyId(body),
            code: PartCode::Keystick,
            outline,
            sweep: Sweep::Extruded { width: 0.01 },
            role: Role::Solid,
            material: Material::KeyWood,
            placed_by: vec![(0..n, "a test")],
        }
    }

    fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<Seg> {
        let (a, b, c, d) = (
            Point::new(x0, y0),
            Point::new(x1, y0),
            Point::new(x1, y1),
            Point::new(x0, y1),
        );
        vec![line(a, b), line(b, c), line(c, d), line(d, a)]
    }

    /// A rod along its body's `x` from its pin to `l`, its end a half disc of
    /// radius `r` centred on the axis at `l`.
    fn rod(body: usize, l: f64, r: f64) -> Part {
        let cap = Seg::Arc {
            c: Point::new(l, 0.0),
            r,
            from: Dir::NEG_Y,
            to: Dir::Y,
        };
        let (low, high) = (cap.start(), cap.end());
        part(
            body,
            vec![
                line(Point::new(0.0, -r), low),
                cap,
                line(high, Point::new(0.0, r)),
                line(Point::new(0.0, r), Point::new(0.0, -r)),
            ],
        )
    }

    fn pair(a: SurfaceRef, b: SurfaceRef) -> ContactPair {
        ContactPair {
            code: ContactCode::BackRailCloth,
            a: Surface::one(a),
            b: Surface::one(b),
            geometry: PairGeometry::Features,
            law: ContactLaw::Rigid,
        }
    }

    fn range(part: u16, segs: core::ops::Range<u16>) -> SurfaceRef {
        SurfaceRef {
            part: PartId(part),
            segs,
        }
    }

    /// A rod's rounded end, segment 1 of [`rod`]'s outline.
    fn end(part: u16) -> SurfaceRef {
        range(part, 1..2)
    }

    /// Whether two numbers agree within `bound`.
    fn near(a: f64, b: f64, bound: f64) -> bool {
        (a - b).abs() <= bound
    }

    /// C6, consistency: **a settled state is an equilibrium of the unilateral
    /// problem**. Its stationarity residual is within the tolerance it
    /// converged to, and no admissible virtual displacement releases more
    /// energy than friction can absorb: `−δV ≤ Σ b_i |δq_i|` for every `δq`
    /// that keeps every gap open. Displacements are drawn at random on every
    /// free coordinate, as small as `scale`; those that open no gap past
    /// zero by the second-order rounding of a rotation are kept.
    ///
    /// The bound is derived: `V` is a sum of weights and forces times
    /// heights, so its second derivative along `δq` is at most the load
    /// scale times the reach squared, and the inequality holds to first
    /// order: `½ L ℓ² |δq|²` covers the rest, and `64 ε` of the potential's
    /// magnitude its rounding.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn assert_virtual_work(
        m: &Mechanism,
        eq: &Equilibrium,
        loads: &[PointLoad],
        friction: Friction,
        anchor: &[f64],
        free: &[usize],
        scale: f64,
        seed: u64,
    ) {
        let load = load_scale(m, loads);
        assert!(
            eq.report.residual <= RESIDUAL_TOLERANCE * load,
            "the residual {} is past its tolerance",
            eq.report.residual
        );
        let points = reached_points(m, loads);
        let pose = m.pose(&eq.q);
        let v0 = m.potential(&pose, loads);
        let reach = free
            .iter()
            .map(|&c| speed(m, &pose, c, &points))
            .fold(0.0_f64, f64::max);
        let mut rng = XorShift::new(seed);
        let mut rows = Rows::new();
        let mut tried = 0;
        for _ in 0..200 {
            let mut dq = vec![0.0; eq.q.len()];
            for &c in free {
                dq[c] = rng.uniform(-scale, scale);
            }
            let moved: Vec<f64> = eq.q.iter().zip(&dq).map(|(a, b)| a + b).collect();
            let shifted = m.pose(&moved);
            m.rows(&shifted, f64::INFINITY, &mut rows);
            let size = dq.iter().map(|x| x.abs()).fold(0.0_f64, f64::max);
            let second = 0.5 * load * reach * reach * size * size;
            let within_limits = free.iter().all(|&c| {
                let (lo, hi) = m.coords()[c].limits;
                moved[c] >= lo && moved[c] <= hi
            });
            // A gap that was closed may open to second order only; one
            // that closes is no admissible displacement.
            let admissible = within_limits
                && rows
                    .rows()
                    .iter()
                    .all(|r| r.gap >= -reach * reach * size * size);
            if !admissible {
                continue;
            }
            tried += 1;
            let released = v0 - m.potential(&shifted, loads);
            let absorbed: f64 = match friction {
                Friction::On => free
                    .iter()
                    .map(|&c| {
                        let b = m.coords()[c].friction;
                        b * ((moved[c] - anchor[c]).abs() - (eq.q[c] - anchor[c]).abs())
                    })
                    .sum(),
                Friction::Off => 0.0,
            };
            let rounding = 64.0 * f64::EPSILON * (v0.abs() + load * reach);
            assert!(
                released <= absorbed + second + rounding,
                "a virtual move {dq:?} releases {released} J, more than friction absorbs, \
                 {absorbed} J: not an equilibrium"
            );
        }
        assert!(
            tried > 10,
            "only {tried} admissible displacements were drawn"
        );
    }

    /// A pendulum: a rod of length `l` on a pin at the origin, its centre of
    /// mass `d` out along it, its end a half disc of radius `r`; under it a
    /// floor whose top stands at `floor`.
    fn pendulum(mass: f64, d: f64, l: f64, r: f64, floor: f64, friction: f64) -> Mechanism {
        let bodies = vec![
            Body::GROUND,
            Body {
                parent: BodyId::GROUND,
                joint: Joint::Revolute {
                    pivot: Point::new(0.0, 0.0),
                    coord: CoordId(0),
                },
                mass: Some(MassProps {
                    mass,
                    com: Point::new(d, 0.0),
                }),
            },
        ];
        let limit = 0.35;
        let coords = vec![Coord::ANGLE.limited(-limit, limit).with_friction(friction)];
        let parts = vec![rod(1, l, r), part(0, rect(-0.5, floor - 0.02, 0.5, floor))];
        // The rod's end against the floor's top, the faces that meet.
        let pairs = vec![pair(end(0), range(1, 2..3))];
        Mechanism::new(bodies, coords)
            .unwrap()
            .with_contacts(parts, pairs)
            .unwrap()
    }

    use crate::mechanism::Joint;

    /// C5, consistency: **a pendulum comes to rest on its stop.** From
    /// level, gravity turns it down until its rounded end meets the floor,
    /// where the centre of its end stands one radius above the floor:
    /// `l sin θ = floor + r`. The floor then carries `m g d / l`, the weight
    /// times its arm over the end's: virtual work, since the contact's lever
    /// arm is `l cos θ` and gravity's `d cos θ`.
    ///
    /// The bounds are derived. The gap converges to within
    /// [`GAP_TOLERANCE`], and the end's height moves `l cos θ` per radian;
    /// the residual is within [`RESIDUAL_TOLERANCE`] of the load scale at the
    /// rod's reach, which the floor's arm `l cos θ` turns into a force.
    #[test]
    #[allow(clippy::disallowed_methods)]
    fn a_pendulum_comes_to_rest_on_its_stop() {
        let (mass, d, l, r, floor) = (0.120, 0.08, 0.20, 0.005, -0.06);
        let m = pendulum(mass, d, l, r, floor, 0.0);
        let eq = Statics::new()
            .solve(&m, &[0.0], &[], &[], Friction::Off)
            .expect("the pendulum rests");
        let expected = ((floor + r) / l).asin();
        let theta = eq.q[0];
        let arm = l * theta.cos();
        assert!(
            near(
                theta,
                expected,
                2.0 * GAP_TOLERANCE / arm + 8.0 * f64::EPSILON
            ),
            "rests at {theta}, expected {expected}"
        );
        let force: f64 = eq.supports.iter().map(|s| s.force).sum();
        let carried = mass * G * d / l;
        let reach = l + r;
        assert!(
            near(
                force,
                carried,
                RESIDUAL_TOLERANCE * mass * G * reach / arm + 1e-12 * carried
            ),
            "the floor carries {force} N, the lever {carried} N"
        );
        assert!(eq.report.converged && eq.report.exact);
        assert_virtual_work(
            &m,
            &eq,
            &[],
            Friction::Off,
            &eq.q.clone(),
            &[0],
            1e-6,
            0x0c6,
        );
    }

    /// C5, consistency: **a lever with axis friction breaks away at
    /// `(b ± T) / arm`.** A level lever whose own weight presses it down with
    /// `T`, a friction bound `b` at its pin, and a weight `w` hung `a` along
    /// it: friction holds it while `|w g a + T| ≤ b`, exactly where it was;
    /// a hair past either edge it turns to the stop that way. The friction
    /// it applies while held balances the rest exactly. Its centre of mass
    /// and the weight stand above the pin, as a key's do, so that gravity's
    /// curvature is negative and a lever that starts to turn does not stall
    /// before its stop.
    #[test]
    fn a_lever_with_axis_friction_breaks_away_at_its_band() {
        let (mass, d, h, l, r) = (0.05, 0.03, 0.016, 0.2, 0.004);
        let b = 0.02;
        // Floors above and below the end, 10 mm off, so that it has
        // somewhere to go either way.
        let bodies = vec![
            Body::GROUND,
            Body {
                parent: BodyId::GROUND,
                joint: Joint::Revolute {
                    pivot: Point::new(0.0, 0.0),
                    coord: CoordId(0),
                },
                mass: Some(MassProps {
                    mass,
                    com: Point::new(d, h),
                }),
            },
        ];
        let limit = core::f64::consts::FRAC_PI_4;
        let coords = vec![Coord::ANGLE.limited(-limit, limit).with_friction(b)];
        let parts = vec![
            rod(1, l, r),
            part(0, rect(0.1, -0.034, 0.3, -0.014)),
            // A ceiling: drawn as a block above, its underside facing down.
            part(0, rect(0.1, 0.014, 0.3, 0.034)),
        ];
        let pairs = vec![pair(end(0), range(1, 2..3)), pair(end(0), range(2, 0..1))];
        let m = Mechanism::new(bodies, coords)
            .unwrap()
            .with_contacts(parts, pairs)
            .unwrap();
        let t = -mass * G * d;
        let a = 0.15;
        let at = MaterialPoint {
            body: BodyId(1),
            at: Point::new(a, 0.026),
        };
        // The weight, grams, at which the lever leaves its band downwards
        // and upwards: w g a = −b − T and w g a = b − T.
        let down = 1000.0 * (b - t.abs()) / (G * a);
        let up = -1000.0 * (b + t.abs()) / (G * a);
        let mut solver = Statics::new();
        for (w, moves) in [
            (down * (1.0 - 1e-6), false),
            (down * (1.0 + 1e-6), true),
            (up * (1.0 - 1e-6), false),
            (up * (1.0 + 1e-6), true),
            (0.5 * (down + up), false),
        ] {
            let loads = [PointLoad::weight(at, w / 1000.0)];
            let eq = solver
                .solve(&m, &[0.0], &loads, &[], Friction::On)
                .expect("the lever rests");
            assert_eq!(
                eq.q[0] != 0.0,
                moves,
                "{w} g: the lever stands at {}",
                eq.q[0]
            );
            if !moves {
                assert_eq!(
                    eq.q[0].to_bits(),
                    0.0_f64.to_bits(),
                    "held exactly where it was"
                );
                let pushed = t - w / 1000.0 * G * a;
                assert!(
                    near(eq.friction[0], -pushed, 1e-12 * b),
                    "friction applies {}, the loads {pushed}",
                    eq.friction[0]
                );
            } else {
                assert!(eq.supports.iter().any(|s| s.force > 0.0), "on a stop");
            }
            assert_virtual_work(&m, &eq, &loads, Friction::On, &[0.0], &[0], 1e-6, 0x0c6 + 1);
        }
    }

    /// C5, consistency: **two levers through an arc against an arc**, the
    /// shape of the capstan on the wippen's heel. The first is held at an
    /// angle (prescribed); the second, weighted to press its rounded end up
    /// into the first's, rests on it. By virtual work the generalized forces
    /// the contact passes are in the ratio of its gradient's entries:
    /// `(R₁ + Q₁) / Q₂ = g₁ / g₂`, the reaction holding the first included,
    /// with the gradient taken by central differences of the drawn gap, not
    /// from the rows.
    ///
    /// The bound is derived: the difference's truncation is `h² / 6` of the
    /// gap's third derivative, at most the reach cubed over the arcs'
    /// distance squared times the reach, here under 10⁻⁷ of the gradient;
    /// the residual adds its tolerance over the gradient.
    #[test]
    fn two_levers_through_arcs_pass_forces_in_the_ratio_of_their_lever_arms() {
        let bodies = vec![
            Body::GROUND,
            Body {
                parent: BodyId::GROUND,
                joint: Joint::Revolute {
                    pivot: Point::new(0.0, 0.0),
                    coord: CoordId(0),
                },
                mass: Some(MassProps {
                    mass: 0.10,
                    com: Point::new(0.05, 0.0),
                }),
            },
            Body {
                parent: BodyId::GROUND,
                joint: Joint::Revolute {
                    pivot: Point::new(0.18, -0.02),
                    coord: CoordId(1),
                },
                // Heavy behind its pin, so its end, in front, rises.
                mass: Some(MassProps {
                    mass: 0.05,
                    com: Point::new(0.04, 0.0),
                }),
            },
        ];
        let limit = core::f64::consts::FRAC_PI_4;
        let coords = vec![
            Coord::ANGLE.limited(-limit, limit),
            Coord::ANGLE.limited(-limit, limit),
        ];
        // The first rod points along +x with its end at 0.1 m; the second
        // stands at (0.18, −0.02) and points back along −x: its body's frame
        // turned half round is written out by mirroring the rod.
        let first = rod(1, 0.10, 0.008);
        let mut second = rod(2, 0.08, 0.012);
        for seg in &mut second.outline {
            *seg = match *seg {
                Seg::Line { a, b } => Seg::Line {
                    a: Point::new(-a.x, -a.y),
                    b: Point::new(-b.x, -b.y),
                },
                Seg::Arc { c, r, from, to } => Seg::Arc {
                    c: Point::new(-c.x, -c.y),
                    r,
                    from: Dir::new(-from.vector().x, -from.vector().y).unwrap(),
                    to: Dir::new(-to.vector().x, -to.vector().y).unwrap(),
                },
            };
        }
        let parts = vec![first, second];
        // The faces that meet: the second's top and end against the first's
        // bottom and end. A rod's far side is no surface of the pair: a
        // signed distance to its line would read the other rod as deep
        // inside it.
        let pairs = vec![pair(range(1, 0..2), range(0, 0..2))];
        let m = Mechanism::new(bodies, coords)
            .unwrap()
            .with_contacts(parts, pairs)
            .unwrap();
        let held = Prescribed {
            coord: CoordId(0),
            value: -0.12,
        };
        // The second starts turned down, its end 11 mm below the first's.
        let eq = Statics::new()
            .solve(&m, &[0.0, 0.3], &[], &[held], Friction::Off)
            .expect("the second rests on the first");
        let force: f64 = eq.supports.iter().map(|s| s.force).sum();
        assert!(force > 0.0, "the two ends bear: {:?}", eq.supports);

        // The gap's gradient by central differences.
        let gap = |q: &[f64]| {
            let mut rows = Rows::new();
            m.rows(&m.pose(q), f64::INFINITY, &mut rows);
            rows.gap(0).expect("the arcs face each other")
        };
        let h = 1e-6;
        let grad: Vec<f64> = (0..2)
            .map(|c| {
                let (mut up, mut down) = (eq.q.clone(), eq.q.clone());
                up[c] += h;
                down[c] -= h;
                (gap(&up) - gap(&down)) / (up[c] - down[c])
            })
            .collect();
        let mut q_forces = vec![0.0; 2];
        m.generalized_forces(&m.pose(&eq.q), &[], &mut q_forces);
        let through_first = eq.reactions[0] + q_forces[0];
        let ratio = through_first / q_forces[1];
        let expected = grad[0] / grad[1];
        assert!(
            near(ratio, expected, 1e-6 * expected.abs()),
            "the contact passes {through_first} N·m and {} N·m, in the ratio {ratio}; its lever \
             arms are in the ratio {expected}",
            q_forces[1]
        );
    }

    /// Determinism: the same increment from the same state gives the same
    /// bits, whatever was solved before it with the same solver.
    #[test]
    fn the_same_increment_gives_the_same_bits() {
        let m = pendulum(0.12, 0.08, 0.2, 0.005, -0.06, 0.004);
        let mut solver = Statics::new();
        let first = solver.solve(&m, &[0.1], &[], &[], Friction::On).unwrap();
        let _ = solver.solve(&m, &[-0.3], &[], &[], Friction::Off);
        let again = solver.solve(&m, &[0.1], &[], &[], Friction::On).unwrap();
        assert_eq!(first, again);
        assert_eq!(first.q[0].to_bits(), again.q[0].to_bits());
    }

    /// C12, consistency: **the metric shapes the path, not the answer.**
    /// Scaled by ten, the pendulum comes to rest at the same angle, within
    /// the tolerance either converges to.
    #[test]
    fn scaling_the_metric_changes_no_rest() {
        let (l, r) = (0.2, 0.005);
        let m = pendulum(0.12, 0.08, l, r, -0.06, 0.0);
        let one = Statics::new()
            .solve(&m, &[0.0], &[], &[], Friction::Off)
            .unwrap();
        let ten = Statics::with_metric_scaled(10.0)
            .solve(&m, &[0.0], &[], &[], Friction::Off)
            .unwrap();
        let arm = l * libm::cos(one.q[0]);
        assert!(
            near(one.q[0], ten.q[0], 4.0 * GAP_TOLERANCE / arm),
            "{} against {}",
            one.q[0],
            ten.q[0]
        );
        assert_ne!(
            one.report.iterations, 0,
            "the solve took steps, which the scaled metric shapes differently"
        );
    }

    /// Consistency: **a still state out of balance is not taken for a rest.**
    /// Under a metric 10¹² times stiffer than the solver's own, the
    /// pendulum's first step moves its end by under a femtometre while
    /// gravity is nowhere near balanced: the step and the gaps pass, the
    /// residual does not, so the residual is read again with every gap
    /// already closed to within [`GAP_TOLERANCE`] taken as closed. That must
    /// fail too, from level, where no row is near, and from half a millimetre
    /// above the floor, where the floor's row is within the margin but open:
    /// only a gap closed to the tolerance may be read as closed, and only a
    /// residual within its tolerance makes a rest. The solve goes on, each
    /// accepted step softening the metric threefold, so it takes more than
    /// the 25 steps that soften it by 10¹², and the pendulum comes to rest on
    /// its floor where the solver's own metric puts it, within the bound
    /// `scaling_the_metric_changes_no_rest` holds a scaled metric to.
    #[test]
    fn a_still_state_out_of_balance_is_not_taken_for_a_rest() {
        let (l, r, floor) = (0.20, 0.005, -0.06);
        let m = pendulum(0.120, 0.08, l, r, floor, 0.0);
        let rest = Statics::new()
            .solve(&m, &[0.0], &[], &[], Friction::Off)
            .expect("the pendulum rests");
        let arm = l * libm::cos(rest.q[0]);
        let above = libm::asin((floor + r + 0.0005) / l);
        for start in [0.0, above] {
            let eq = Statics::with_metric_scaled(1e12)
                .solve(&m, &[start], &[], &[], Friction::Off)
                .unwrap_or_else(|e| panic!("from {start} rad: {e:?}"));
            assert!(
                near(eq.q[0], rest.q[0], 4.0 * GAP_TOLERANCE / arm),
                "from {start} rad it stopped at {} rad, the rest is at {} rad",
                eq.q[0],
                rest.q[0]
            );
            assert!(eq.report.iterations > 25, "{:?}", eq.report);
        }
    }

    /// A free coordinate with no finite limit is refused: the search would
    /// not be bounded.
    #[test]
    fn a_free_coordinate_without_limits_is_refused() {
        let bodies = vec![
            Body::GROUND,
            Body {
                parent: BodyId::GROUND,
                joint: Joint::Revolute {
                    pivot: Point::new(0.0, 0.0),
                    coord: CoordId(0),
                },
                mass: None,
            },
        ];
        let m = Mechanism::new(bodies, vec![Coord::ANGLE]).unwrap();
        assert_eq!(
            Statics::new().solve(&m, &[0.0], &[], &[], Friction::Off),
            Err(Refusal::Mechanism(Problem::Unlimited { coord: CoordId(0) }))
        );
    }

    /// Consistency: **a load on a row standing open is no rest.** The key
    /// front 2.3725 mm forward of where the bench builds it stands the front
    /// rail pin 0.87 mm into the wood behind the key's front mortise, and the
    /// key rides on the pin and its cloth, off its balance punching. Its top
    /// seat starts level on its punching, the pin in its wood within the
    /// row margin, and the stops fight there, so the key is solved free to
    /// lift: some forty steps in, each accepted one softening the metric
    /// threefold, the box problem reads the cloth's row and the pin's as
    /// closed while they stand 2.2 and 0.18 µm open, each carrying most of a
    /// newton, and the loads balance to the rounding. That is no
    /// equilibrium: a contact pushes only where it touches. Converged, every
    /// row that carries more than the residual's tolerance of the load
    /// scale is closed to within [`GAP_TOLERANCE`], and the bottom seat,
    /// solved from the top, is the same pose to the bit: the riding key's dip
    /// is nothing, where the rows read closed while they stand open would
    /// leave 23 nm between its two seats.
    #[test]
    fn a_load_on_a_row_standing_open_is_no_rest() {
        use crate::parts::fixtures::{on_frame, regulated};
        use crate::parts::frame::KeyKind;
        use crate::parts::mounts::nominal_pose;

        let mut a = regulated();
        a.key.front_edge += 0.0023725;
        let k = on_frame(&a, KeyKind::Natural, 0.0, true);
        let seats = k
            .seating()
            .seats(&mut Statics::new())
            .expect("the key rides on the pin and its cloth");
        assert!(seats.top.rides(), "{:?}", seats.top);
        // The top seat's increment again, from where the seat starts it, to
        // read the rows that carry it.
        let m = &k.mounted.mechanism;
        let capstan = PointLoad {
            at: MaterialPoint {
                body: k.mounted.key,
                at: k.capstan,
            },
            force: Point::new(0.0, -seats.capstan_load),
        };
        let start = nominal_pose(&k.mounted);
        assert_eq!(k.mounted.lift_at(&start).to_bits(), 0.0_f64.to_bits());
        let eq = k
            .mounted
            .solve(&mut Statics::new(), &start, &[capstan], Friction::Off)
            .expect("the key rides");
        let load = load_scale(m, &[capstan]);
        let loaded: Vec<&Support> = eq
            .supports
            .iter()
            .filter(|s| s.force > RESIDUAL_TOLERANCE * load)
            .collect();
        assert_eq!(loaded.len(), 2, "{:?}", eq.supports);
        for s in loaded {
            assert!(
                s.gap.abs() <= GAP_TOLERANCE,
                "a row carrying {} N stands {} m open",
                s.force,
                s.gap
            );
        }
        for (x, y) in eq.q.iter().zip(&seats.top.q) {
            assert_eq!(x.to_bits(), y.to_bits(), "the seat is that increment");
        }
        for (x, y) in seats.top.q.iter().zip(&seats.bottom.q) {
            assert_eq!(x.to_bits(), y.to_bits(), "one pose for both seats");
        }
        let (top, bottom) = seats.heights(&k.mounted, k.front_top);
        assert_eq!((top - bottom).to_bits(), 0.0_f64.to_bits(), "no dip");
    }

    /// With nothing to stop it, the pendulum falls to its limit, the edge of
    /// the modelled world, and the increment is refused, naming the limit.
    #[test]
    fn a_state_leaning_on_a_limit_is_no_equilibrium() {
        // The floor far below the end's reach.
        let m = pendulum(0.12, 0.08, 0.2, 0.005, -0.5, 0.0);
        match Statics::new().solve(&m, &[0.0], &[], &[], Friction::Off) {
            Err(Refusal::NoEquilibrium { on }) => assert_eq!(
                on,
                vec![Holder::Limit {
                    coord: CoordId(0),
                    upper: false
                }]
            ),
            other => panic!("{other:?}"),
        }
    }

    /// Consistency, found by a seeded search over lifts: **a key already
    /// resting against the front wall of its slot, lifted again, comes to
    /// rest there in one increment.** With the front rail 0.9 to 1.5 mm in
    /// front of where the bench builds it, a lift raises the key until the
    /// slot's front wall meets the pin's front top corner, on a row whose arm
    /// about the bearing is under two centimetres. The next lift starts on
    /// that wall, its gap closed to the rounding of positions a quarter of a
    /// metre out, some hundredths of a femtometre either way. Closing such a
    /// gap through that arm turns the key fifteen times as far at its
    /// farthest point, and the proximal force of that motion, the first
    /// step's metric times it, can stand above the residual's tolerance of a
    /// part in 10¹² of the loads, while the merit test, which compares
    /// energies at their rounding, turns the step back: the key never moves,
    /// and each raise of the metric only grows that force. The state is an
    /// equilibrium all along, the wall and the friction holding the key, so
    /// every lift here must converge and pass the virtual-work inequality.
    /// Past 1.5 mm forward the pin stands across the slot's front edge, its
    /// face beyond the wall, and the key has no rest on a pivot: the
    /// initialization refuses, naming the pin's pair. On a pivot, the one
    /// coordinate this solve is tested on here: a key free to lift rides on
    /// the pin and its cloth instead.
    #[test]
    fn a_key_lifted_against_its_slots_front_wall_from_there_is_one_increment() {
        use crate::contact::ContactCode;
        use crate::parts::fixtures::{on_pivot, regulated};
        use crate::parts::frame::KeyKind;
        use crate::parts::mounts::nominal_pose;

        let mut rng = XorShift::new(25);
        let mut statics = Statics::new();
        let (mut lifts, mut on_the_wall, mut straddling) = (0, 0, 0);
        for case in 0..40 {
            let mut a = regulated();
            let forward = 1e-3 * rng.uniform(0.9, 1.6);
            a.frame.front_rail += forward;
            let kf_g = rng.uniform(0.0, 15.0);
            let b = kf_g * 1e-3 * G * a.measuring_point();
            let k = on_pivot(&a, KeyKind::Natural, b, true);
            let m = &k.mounted.mechanism;
            let wall = m
                .pairs()
                .iter()
                .position(|p| p.code == ContactCode::FrontPinOnTheWood)
                .expect("the pin has a pair on the wood");
            let at = MaterialPoint {
                body: k.mounted.key,
                at: k.measuring_point,
            };
            let start = nominal_pose(&k.mounted);
            let lifted = [PointLoad::weight(at, -0.08)];
            let mut q = match statics.solve(m, &start, &lifted, &[], Friction::Off) {
                Ok(eq) => eq.q,
                Err(Refusal::NoEquilibrium { on }) if forward > 1.5e-3 => {
                    assert!(
                        on.iter().all(|h| matches!(h, Holder::Contact { pair, .. }
                            if usize::from(*pair) == wall)),
                        "case {case}: {on:?}"
                    );
                    straddling += 1;
                    continue;
                }
                Err(refusal) => panic!("case {case}, {forward} m forward: {refusal:?}"),
            };
            for _ in 0..10 {
                let w = rng.uniform(-80.0, -20.0);
                let loads = [PointLoad::weight(at, w * 1e-3)];
                let eq = statics
                    .solve(m, &q, &loads, &[], Friction::On)
                    .unwrap_or_else(|r| panic!("case {case}, {w} g: {r:?}"));
                lifts += 1;
                if eq.supports.iter().any(|s| {
                    s.force > 0.0
                        && matches!(s.by, Holder::Contact { pair, .. } if usize::from(pair) == wall)
                }) {
                    on_the_wall += 1;
                }
                assert_virtual_work(m, &eq, &loads, Friction::On, &q, &[0], 1e-6, 0x5107);
                q = eq.q;
            }
        }
        assert!(
            straddling > 0 && lifts > 200,
            "{straddling} straddling, {lifts} lifts"
        );
        assert!(
            on_the_wall > lifts * 9 / 10,
            "only {on_the_wall} of {lifts} on the wall"
        );
    }
}

//! Small dense linear algebra: the least-norm solution of a linear system,
//! by a complete orthogonal decomposition.
//!
//! # Why least norm, and why this decomposition
//!
//! The box problem's exact pass solves for the multipliers of the rows that
//! hold still ([`super::boxed`]). Where those rows are redundant (a key
//! bedded flat on both edges of its cloth, a joint and a contact holding the
//! same body) the system is singular and every split of the load between
//! them balances it: rigid statics leave the split indeterminate. The engine
//! takes the split of least utilisation, the solution of least norm in
//! scaled multipliers, so that it is one definite answer that does not drift
//! from one solve to the next, and reports it as one admissible split among
//! many (`docs/FOUNDATIONS.md` §2.1).
//!
//! Neither a pivoted LDLᵀ nor a column-pivoted QR alone returns the
//! least-norm solution of a rank-deficient system: each returns *a* solution,
//! with the free components set to zero in its own pivot order. A complete
//! orthogonal decomposition does: a column-pivoted Householder QR, `A P = Q R`,
//! decides the rank; an orthogonal reduction of the trapezoid `[R₁₁ R₁₂]` from
//! the right to `[T 0]` then leaves every solution as `P Z [T⁻¹ c; free]`, and
//! setting the free part to zero is the least norm, `Z` being orthogonal.
//!
//! # Determinism
//!
//! Every loop runs in a fixed order; a pivot tie goes to the lowest index;
//! the rank is decided by a frozen relative tolerance, [`RANK_TOLERANCE`].
//! The same matrix gives the same bits on every machine.

/// How small a diagonal entry of `R` may be, relative to the largest, before
/// its column counts as dependent on the ones before it.
///
/// About 10⁵ units in the last place. The systems solved here are a few rows
/// of lever arms and unit rows, conditioned like the levers they describe: a
/// genuinely independent row stands many orders of magnitude above this,
/// while a redundant one, two edges of one face or a joint and a contact on
/// one body, comes out at the rounding of the elimination, near ε.
pub const RANK_TOLERANCE: f64 = 1e-11;

/// The least-norm solution of `A x = b`, with the workspace it needs, kept
/// between solves so that a solve of a size already seen allocates nothing.
#[derive(Debug, Clone, Default)]
pub struct LeastNorm {
    perm: Vec<usize>,
    /// The right reflectors of the trapezoid's reduction, one row of `n`
    /// entries each, the entries off its support zero.
    right: Vec<f64>,
    y: Vec<f64>,
}

impl LeastNorm {
    /// An empty workspace.
    pub fn new() -> Self {
        Self::default()
    }

    /// Writes into `x` the `x` of least norm among those that minimize
    /// `‖A x − b‖`, and returns the rank of `A`.
    ///
    /// `a` holds `A`, `m` rows of `n` entries each, row by row; `b` holds
    /// `m` entries; `x` receives `n`. `a` and `b` are overwritten.
    ///
    /// # Panics
    ///
    /// When the slices do not have the lengths `m` and `n` say.
    pub fn solve(
        &mut self,
        a: &mut [f64],
        m: usize,
        n: usize,
        b: &mut [f64],
        x: &mut [f64],
    ) -> usize {
        assert_eq!(a.len(), m * n, "A holds m rows of n entries");
        assert_eq!(b.len(), m, "b holds m entries");
        assert_eq!(x.len(), n, "x holds n entries");
        x.fill(0.0);
        self.perm.clear();
        self.perm.extend(0..n);
        let k = m.min(n);

        // A P = Q R, Q applied to b as it is built.
        let mut rank = k;
        for j in 0..k {
            // The column of greatest remaining norm, the first of equals.
            let mut best = j;
            let mut best_norm = column_norm2(a, m, n, j, j);
            for c in j + 1..n {
                let norm = column_norm2(a, m, n, c, j);
                if norm > best_norm {
                    best = c;
                    best_norm = norm;
                }
            }
            if best != j {
                for row in 0..m {
                    a.swap(row * n + j, row * n + best);
                }
                self.perm.swap(j, best);
            }
            if best_norm == 0.0 {
                // Every column left is zero: nothing more is independent.
                rank = j;
                break;
            }
            let alpha = a[j * n + j];
            let sigma: f64 = (j + 1..m).map(|row| a[row * n + j] * a[row * n + j]).sum();
            let norm = (alpha * alpha + sigma).sqrt();
            let beta = if alpha >= 0.0 { -norm } else { norm };
            // v = column − β e₁, with v₀ = α − β free of cancellation, and
            // H = I − 2 v vᵀ / vᵀv.
            let v0 = alpha - beta;
            let vtv = v0 * v0 + sigma;
            for c in j + 1..n {
                let mut s = v0 * a[j * n + c];
                for row in j + 1..m {
                    s += a[row * n + j] * a[row * n + c];
                }
                let f = 2.0 * s / vtv;
                a[j * n + c] -= f * v0;
                for row in j + 1..m {
                    a[row * n + c] -= f * a[row * n + j];
                }
            }
            let mut s = v0 * b[j];
            for row in j + 1..m {
                s += a[row * n + j] * b[row];
            }
            let f = 2.0 * s / vtv;
            b[j] -= f * v0;
            for row in j + 1..m {
                b[row] -= f * a[row * n + j];
            }
            a[j * n + j] = beta;
            for row in j + 1..m {
                a[row * n + j] = 0.0;
            }
        }

        // The rank: the diagonal entries above the tolerance, which column
        // pivoting leaves in non-increasing magnitude.
        if rank > 0 {
            let largest = a[0].abs();
            let mut r = 0;
            while r < rank && a[r * n + r].abs() > RANK_TOLERANCE * largest {
                r += 1;
            }
            rank = r;
        }
        if rank == 0 {
            return 0;
        }

        // [R₁₁ R₁₂] Z = [T 0], from the last row of the trapezoid up: each
        // reflector acts on column i and the columns past the rank, zeroing
        // row i's entries past the rank, and is applied to the rows above it
        // too, which leaves T upper triangular.
        self.right.clear();
        self.right.resize(rank * n, 0.0);
        if rank < n {
            for i in (0..rank).rev() {
                let alpha = a[i * n + i];
                let mut sigma = 0.0;
                for c in rank..n {
                    sigma += a[i * n + c] * a[i * n + c];
                }
                let u = &mut self.right[i * n..(i + 1) * n];
                if sigma == 0.0 {
                    continue;
                }
                let norm = (alpha * alpha + sigma).sqrt();
                let gamma = if alpha >= 0.0 { -norm } else { norm };
                u[i] = alpha - gamma;
                for c in rank..n {
                    u[c] = a[i * n + c];
                }
                let utu = u[i] * u[i] + sigma;
                for row in 0..=i {
                    let mut s = a[row * n + i] * u[i];
                    for c in rank..n {
                        s += a[row * n + c] * u[c];
                    }
                    let f = 2.0 * s / utu;
                    a[row * n + i] -= f * u[i];
                    for c in rank..n {
                        a[row * n + c] -= f * u[c];
                    }
                }
            }
        }

        // T w₁ = (Qᵀ b)₁ by back substitution, w₂ = 0.
        self.y.clear();
        self.y.resize(n, 0.0);
        for i in (0..rank).rev() {
            let mut s = b[i];
            for j in i + 1..rank {
                s -= a[i * n + j] * self.y[j];
            }
            self.y[i] = s / a[i * n + i];
        }

        // x = P Z w, Z = Z_{r−1} ⋯ Z₀: Z₀ applied first.
        if rank < n {
            for i in 0..rank {
                let u = &self.right[i * n..(i + 1) * n];
                let mut utu = u[i] * u[i];
                let mut s = u[i] * self.y[i];
                for (&uc, &yc) in u[rank..n].iter().zip(&self.y[rank..n]) {
                    utu += uc * uc;
                    s += uc * yc;
                }
                if utu == 0.0 {
                    continue;
                }
                let f = 2.0 * s / utu;
                self.y[i] -= f * u[i];
                for (yc, &uc) in self.y[rank..n].iter_mut().zip(&u[rank..n]) {
                    *yc -= f * uc;
                }
            }
        }
        for (j, &p) in self.perm.iter().enumerate() {
            x[p] = self.y[j];
        }
        rank
    }
}

/// The squared norm of column `c` of the `m × n` matrix `a`, from row `from`
/// down.
fn column_norm2(a: &[f64], m: usize, n: usize, c: usize, from: usize) -> f64 {
    let mut s = 0.0;
    for row in from..m {
        s += a[row * n + c] * a[row * n + c];
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanism::fixtures::XorShift;

    fn solve(a: &[f64], m: usize, n: usize, b: &[f64]) -> (Vec<f64>, usize) {
        let (mut a, mut b) = (a.to_vec(), b.to_vec());
        let mut x = vec![0.0; n];
        let rank = LeastNorm::new().solve(&mut a, m, n, &mut b, &mut x);
        (x, rank)
    }

    /// A random orthogonal matrix, `n × n`, row by row: a product of `n`
    /// Householder reflectors of random vectors.
    fn orthogonal(n: usize, rng: &mut XorShift) -> Vec<f64> {
        let mut q = vec![0.0; n * n];
        for i in 0..n {
            q[i * n + i] = 1.0;
        }
        for _ in 0..n {
            let v: Vec<f64> = (0..n).map(|_| rng.uniform(-1.0, 1.0)).collect();
            let vtv: f64 = v.iter().map(|x| x * x).sum();
            // q ← q (I − 2 v vᵀ / vᵀv)
            for row in 0..n {
                let s: f64 = (0..n).map(|c| q[row * n + c] * v[c]).sum();
                for c in 0..n {
                    q[row * n + c] -= 2.0 * s * v[c] / vtv;
                }
            }
        }
        q
    }

    /// Hand-checked: a regular system, a singular one whose two rows are one
    /// row twice, and an overdetermined one.
    #[test]
    fn hand_checked_systems_have_their_least_norm_solutions() {
        // 2x + y = 5, x + 3y = 10: x = 1, y = 3.
        let (x, rank) = solve(&[2.0, 1.0, 1.0, 3.0], 2, 2, &[5.0, 10.0]);
        assert_eq!(rank, 2);
        assert!(
            (x[0] - 1.0).abs() < 1e-15 && (x[1] - 3.0).abs() < 1e-15,
            "{x:?}"
        );

        // x + y = 2 twice: every (t, 2 − t) solves it, (1, 1) is the least.
        let (x, rank) = solve(&[1.0, 1.0, 1.0, 1.0], 2, 2, &[2.0, 2.0]);
        assert_eq!(rank, 1);
        assert!(
            (x[0] - 1.0).abs() < 1e-15 && (x[1] - 1.0).abs() < 1e-15,
            "{x:?}"
        );

        // Inconsistent and redundant: x + y = 1 and x + y = 3 is best met at
        // x + y = 2, least at (1, 1).
        let (x, rank) = solve(&[1.0, 1.0, 1.0, 1.0], 2, 2, &[1.0, 3.0]);
        assert_eq!(rank, 1);
        assert!(
            (x[0] - 1.0).abs() < 1e-15 && (x[1] - 1.0).abs() < 1e-15,
            "{x:?}"
        );

        // Three equations, one unknown: the least-squares mean.
        let (x, rank) = solve(&[1.0, 1.0, 1.0], 3, 1, &[1.0, 2.0, 6.0]);
        assert_eq!(rank, 1);
        assert!((x[0] - 3.0).abs() < 1e-15, "{x:?}");

        // Nothing at all.
        let (x, rank) = solve(&[0.0; 4], 2, 2, &[1.0, 1.0]);
        assert_eq!(rank, 0);
        assert_eq!(x, vec![0.0, 0.0]);
    }

    /// Consistency, against a reference built by hand: `A = U Σ Vᵀ` with
    /// random orthogonal `U` and `V` and singular values chosen in `[0.5, 2]`
    /// or zero, so its least-norm solution is `V Σ⁺ Uᵀ b` exactly, at every
    /// rank from zero to full, square and not.
    ///
    /// The bound is derived: the condition number of the nonzero part is at
    /// most 4, and every entry passes through fewer than `4 n²` roundings on
    /// numbers no larger than `2 n ‖b‖`, so `64 n³ ε ‖b‖` covers the
    /// decomposition's own error and the reference's.
    #[test]
    fn rank_deficient_systems_have_their_least_norm_solutions() {
        let mut rng = XorShift::new(0x00de_5e00);
        for case in 0..200 {
            let n = 1 + case % 6;
            let m = n + (case / 6) % 2;
            let rank = (case / 12) % (n + 1);
            let (u, v) = (orthogonal(m, &mut rng), orthogonal(n, &mut rng));
            let sigma: Vec<f64> = (0..n)
                .map(|i| if i < rank { rng.uniform(0.5, 2.0) } else { 0.0 })
                .collect();
            // A = U[:, :n] Σ Vᵀ.
            let mut a = vec![0.0; m * n];
            for row in 0..m {
                for col in 0..n {
                    a[row * n + col] = (0..n)
                        .map(|k| u[row * m + k] * sigma[k] * v[col * n + k])
                        .sum();
                }
            }
            let b: Vec<f64> = (0..m).map(|_| rng.uniform(-1.0, 1.0)).collect();
            // x* = V Σ⁺ Uᵀ b.
            let utb: Vec<f64> = (0..n)
                .map(|k| (0..m).map(|row| u[row * m + k] * b[row]).sum())
                .collect();
            let expected: Vec<f64> = (0..n)
                .map(|col| {
                    (0..n)
                        .filter(|&k| sigma[k] > 0.0)
                        .map(|k| v[col * n + k] * utb[k] / sigma[k])
                        .sum()
                })
                .collect();
            let (x, found) = solve(&a, m, n, &b);
            assert_eq!(found, rank, "case {case}: rank {found}, built at {rank}");
            let norm_b = b.iter().map(|x| x * x).sum::<f64>().sqrt();
            let bound = 64.0 * (n * n * n) as f64 * f64::EPSILON * norm_b;
            for (got, want) in x.iter().zip(&expected) {
                assert!(
                    (got - want).abs() <= bound,
                    "case {case} ({m}×{n}, rank {rank}): {x:?} against {expected:?}"
                );
            }
        }
    }

    /// The same matrix gives the same bits, and a tie between two columns of
    /// equal norm goes to the first.
    #[test]
    fn the_same_system_gives_the_same_bits() {
        let a = [1.0, 1.0, 0.5, 1.0, 1.0, 0.5, 0.5, 0.5, 3.0];
        let b = [1.0, 2.0, 3.0];
        let (x1, r1) = solve(&a, 3, 3, &b);
        let (x2, r2) = solve(&a, 3, 3, &b);
        assert_eq!(r1, r2);
        for (p, q) in x1.iter().zip(&x2) {
            assert_eq!(p.to_bits(), q.to_bits());
        }
    }
}

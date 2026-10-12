"""The thirteen contact laws of Hirschkorn 2004, Table A.8, read as springs.

1. Every elastic branch f(d) = a d^3 + b d^2 + c d is strictly increasing for
   d > 0, because its tangent stiffness k(d) = 3a d^2 + 2b d + c has no root
   at d > 0. Where b < 0 the stiffness first dips below c, to a minimum at
   d* = -b / (3a), and then rises; where b > 0 it rises from the start.
2. Hirschkorn's jack/knuckle law (contact 7) against Thorin's knuckle felt
   (k = 7e9, exponent 3, Thorin 2013 Eq. 4.9), compared at equal force. Do not
   read the ratio of the cubic coefficients, 3.92e11 / 7e9 = 56, as a ratio of
   stiffnesses: at equal force the tangent stiffnesses differ by 3.1 to 3.6.
3. Stability of a semi-implicit step with the contact damping implicit in
   velocity: 2/omega is the undamped limit, not the limit.

Effective masses and time steps are not computed here: they need the assembled
mass matrix and the moment arms of the contact normals (script 10).

Pure arithmetic: import math and nothing else. Run with python3 -I.
"""
import math

LAW = {1: (1.26e10, 3.26e7, 7.92e3, "capstan / wippen heel"),
       2: (4.41e10, -1.02e7, 1.57e4, "key front / front rail punching"),
       3: (2.42e10, -1.11e7, 6.54e3, "key back / back rail cloth"),
       4: (5.86e11, -3.70e7, 3.11e4, "back check / hammer tail"),
       5: (5.28e9, -7.54e5, 4.42e3, "wippen / jack rest"),
       6: (2.53e10, -9.89e6, 7.07e3, "wippen / lever stop"),
       7: (3.92e11, 1.10e7, 3.23e4, "jack / knuckle"),
       8: (7.46e10, -2.29e7, 1.32e4, "jack toe / let-off button"),
       9: (6.02e10, -1.68e7, 6.96e3, "jack / lever window"),
       10: (1.09e12, -7.51e7, 2.23e4, "lever / knuckle"),
       11: (8.82e11, -7.59e7, 1.67e4, "lever / drop screw"),
       12: (1.81e10, 8.19e7, 1.16e4, "hammer / string"),
       13: (8.58e8, -1.11e6, 2.39e3, "shank / rest rail")}


def depth_at(force, a, b, c):
    lo, hi = 0.0, 0.05
    for _ in range(300):
        mid = 0.5 * (lo + hi)
        if a * mid**3 + b * mid**2 + c * mid < force:
            lo = mid
        else:
            hi = mid
    return 0.5 * (lo + hi)


print("=== 1. Is each elastic branch increasing for d > 0 ? ===")
print("     (k = 3a d^2 + 2b d + c; a root of k at d > 0 would make the force fall)")
increasing = 0
for i, (a, b, c, name) in LAW.items():
    disc = 4 * b * b - 12 * a * c
    roots = []
    if disc >= 0:
        roots = [(-2 * b - math.sqrt(disc)) / (6 * a), (-2 * b + math.sqrt(disc)) / (6 * a)]
    falls = any(r > 0 for r in roots)
    increasing += not falls
    shape = "rises from d = 0"
    if b < 0:
        d_star = -b / (3 * a)
        k_min = c - b * b / (3 * a)
        shape = f"dips to {k_min:9.3e} N/m at d* = {d_star * 1e3:.3f} mm, then rises"
    verdict = "FORCE FALLS somewhere" if falls else "increasing"
    print(f"  c{i:2d} {name:32s} {verdict:10s}  k(0) = c = {c:9.3e} N/m, stiffness {shape}")
print(f"  => {increasing} of 13 elastic branches are strictly increasing for d > 0")

print("\n=== 2. Thorin's knuckle felt (k = 7e9, r = 3) against Hirschkorn's contact 7, at equal force ===")
kT, rT = 7e9, 3.0
a, b, c, _ = LAW[7]
for F in (10, 25, 100):
    dT = (F / kT) ** (1 / rT)
    dH = depth_at(F, a, b, c)
    ktT = rT * kT * dT ** (rT - 1)
    ktH = 3 * a * dH * dH + 2 * b * dH + c
    print(f"  F = {F:4d} N : Thorin d = {dT * 1e3:6.3f} mm, k_t = {ktT:9.3e} N/m | "
          f"Hirschkorn d = {dH * 1e3:6.3f} mm, k_t = {ktH:9.3e} N/m | ratio of k_t = {ktH / ktT:4.2f}")
print(f"  ratio of the cubic coefficients a_Hirschkorn / k_Thorin = {a / kT:.1f}  (not a stiffness ratio)")

print("\n=== 3. Semi-implicit step, contact force implicit in velocity ===")
print("  m x'' = -k x - c x' ;  v+ = (v - dt*k*x/m) / (1 + dt*c/m) ;  x+ = x + dt*v+")
print("  det A = 1/(1+alpha), trace A = 1 + (1-beta)/(1+alpha), alpha = dt*c/m, beta = (w*dt)^2")
print("  |trace| < 1 + det  =>  beta < 4 + 2*alpha  =>  w*dt < sqrt(4 + 2*dt*c/m)")
print("  c = 0 gives w*dt < 2: 2/w is the UNDAMPED limit, and implicit damping only widens it")
for z in (0.0, 0.1, 0.3, 1.0):
    lim = 2 * z + math.sqrt(4 * z * z + 4)
    print(f"    zeta = {z:4.2f} -> stable for w*dt < {lim:5.3f}  (dt_max = {lim / (2 * math.pi):.3f} T)")
print("  With the damping EXPLICIT there is a further condition, dt < 2m/c = 1/(zeta*w):")
print("  at zeta = 1 that is dt < 1/w, half of 2/w. Either way stability is not accuracy;")
print("  the step is chosen by a convergence study.")

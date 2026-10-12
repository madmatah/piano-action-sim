"""Can a published elastic branch push the wrong way?

Hirschkorn 2004, Table A.8: each contact's elastic force is the cubic
f(d) = a d^3 + b d^2 + c d = d (a d^2 + b d + c). Several b are negative, so
whether f stays positive for d > 0 is a real question: it does exactly when the
quadratic factor has no positive root. The second part gives the separation
speed below which Hirschkorn's damped law, f(d)(1 + D d'), turns attractive.

Pure arithmetic: import math and nothing else. Run with python3 -I.
"""
import math
# Hirschkorn 2004, Table A.8: a, b, c, damping factor D
C = {
1:(1.26e10,3.26e7,7.92e3,1.877,'capstan/wippen heel'),
2:(4.41e10,-1.02e7,1.57e4,1.219,'key front/punching'),
3:(2.42e10,-1.11e7,6.54e3,2.502,'key heel/rest rail cloth'),
4:(5.86e11,-3.70e7,3.11e4,2.446,'backcheck/hammer tail'),
5:(5.28e9,-7.54e5,4.42e3,5.781,'wippen/jack rest'),
6:(2.53e10,-9.89e6,7.07e3,7.732,'wippen/rep lever stop'),
7:(3.92e11,1.10e7,3.23e4,4.694,'JACK/KNUCKLE'),
8:(7.46e10,-2.29e7,1.32e4,1.527,'jack foot/letoff button'),
9:(6.02e10,-1.68e7,6.96e3,0.779,'jack/rep lever window'),
10:(1.09e12,-7.51e7,2.23e4,2.935,'REP LEVER/KNUCKLE'),
11:(8.82e11,-7.59e7,1.67e4,4.985,'rep lever/drop screw'),
12:(1.81e10,8.19e7,1.16e4,0.175,'hammer felt/string'),
13:(8.58e8,-1.11e6,2.39e3,8.964,'shank/rest rail'),
}
print("=== Discriminant of a*d^2+b*d+c : is elastic force ever NEGATIVE for d>0 ? ===")
bad = 0
for i,(a,b,c,D,n) in C.items():
    disc=b*b-4*a*c
    neg=""
    if disc>0:
        r1=(-b-math.sqrt(disc))/(2*a); r2=(-b+math.sqrt(disc))/(2*a)
        if r2>0: neg=f"  *** NEGATIVE ELASTIC FORCE for d in ({r1*1e3:.4f},{r2*1e3:.4f}) mm ***"
    print(f"  c{i:2d} {n:28s} b={b:+.3e} disc={disc:+.4e}{neg}")
    bad = bad + 1 if neg else bad
print(f"  => {13 - bad} of 13 elastic branches are positive for every d > 0")

print()
print("=== Hunt-Crossley f(d)(1 + D d') turns attractive below d' = -1/D ===")
th = sorted((-1 / D, i, n) for i, (a, b, c, D, n) in C.items())
for t, i, n in th:
    print(f"  c{i:2d} {n:28s} D={C[i][3]:6.3f} s/m  -> attractive below {t:+.3f} m/s")
print(f"  => from {max(t for t, _, _ in th):+.3f} to {min(t for t, _, _ in th):+.3f} m/s;"
      f" {sum(1 for t, _, _ in th if t > -1.0)} of 13 contacts turn attractive above -1 m/s")

"""Three figures in the research annexes, recomputed.

1. Hammer travel from a constant small-angle ratio (Thorin's alpha31 = 10.1):
   annex A2 compares the full dip with the regulated blow distance and finds
   57.6 against 45-47 mm. Aftertouch has to come off the dip first; done that
   way, a constant ratio overshoots by about 14 %, not 25 %.
2. The share of friction the published axis torques explain: annex A2's sum
   leaves out the jack and the repetition lever, the second largest torque.
   The sum mixes Hirschkorn's torques with Thorin's small-angle ratios and
   arm, so it bounds the annex's figure and predicts nothing.
3. Annex A1's 108 ns per step against annex A5's 125 Mflop/s per note, which
   A5 gives at a 20 us step: both cannot hold on one scalar core.

Sources: Thorin 2013, Table 2.1, for L = 0.245 m, alpha21 = 2.15 and
alpha31 = 10.1; Hirschkorn 2004, Table A.9, for the five axis torques. The
strike radius rs = 0.1397 m is the one that reproduces annex A2's 57.6 mm; no
source in the corpus gives it. Blow distance 46 mm, let-off 1.5 mm, aftertouch
1.2 mm and the 15 g friction weight are assumed mid-range values.

Pure arithmetic. Run with python3 -I.
"""
print("=== 1. Hammer travel from a constant ratio ===")
L=0.245; rs=0.1397; a31=10.1; dip=10.0
hammer=dip/1000/L*a31*rs*1000
print(f"  full dip {dip} mm * ratio -> {hammer:.1f} mm (annex A2: 57.6)  implied ratio = {hammer/dip:.2f}")
blow,letoff,at=46.0,1.5,1.2
print(f"  useful-travel ratio (blow-letoff)/(dip-aftertouch) = ({blow}-{letoff})/({dip}-{at}) = {(blow-letoff)/(dip-at):.2f}")
driven=(dip-at)/1000/L*a31*rs*1000
print(f"  constant ratio over the DRIVEN stroke ({dip-at} mm) -> {driven:.1f} mm against {blow-letoff:.1f} mm traveled to let-off")
print(f"     => error = {(driven/(blow-letoff)-1)*100:.1f} %   (the full-dip comparison makes it about 25 %)")

print()
print("=== 2. Annex A2's friction sum (59 % coverage?) ===")
A={'key':0.0122,'wippen':3.97e-4,'jack':3.97e-4,'rep':2.47e-3,'hammer':1.01e-3}
a21,a31=2.15,10.1
s3=A['key']+a21*A['wippen']+a31*A['hammer']
print(f"  A2 sum (key + 2.15*wippen + 10.1*hammer) = {s3:.5f} N.m   <- jack and rep lever OMITTED")
for Lk in (0.245,0.2175):
    print(f"    /{Lk} m -> {s3/Lk:.4f} N = {s3/Lk/9.81*1000:.2f} g")
for arr in (2.0,2.5,3.0):
    s5=s3+a21*A['jack']+arr*A['rep']
    print(f"  with jack(x{a21}) and rep lever(x{arr}) included: {s5:.5f} N.m -> {s5/0.245/9.81*1000:.2f} g  ({s5/s3*100:.0f} % of the A2 sum)")
print("  a friction weight of 15 g ->", 0.015*9.81*0.245, "N.m at L=0.245")
print(f"  coverage with A2 sum only: {s3/(0.015*9.81*0.245)*100:.0f} % ; with jack+rep (x2.5): {(s3+a21*A['jack']+2.5*A['rep'])/(0.015*9.81*0.245)*100:.0f} %")

print()
print("=== 3. Can annex A1's 108 ns/step and annex A5's 125 Mflop/s per note both hold? ===")
for dt in (5e-6,20e-6,40e-6):
    steps=1/dt
    print(f"  dt={dt*1e6:4.0f}us -> {steps:8.0f} steps/s of sim")
    print(f"     at 108 ns/step  : {steps*108e-9*1000:7.3f} ms CPU per second of sim -> per 16.7ms frame, 1 note = {steps*108e-9*0.0167*1000:.3f} ms")
    print(f"     125 Mflop/s/note implies {125e6/steps:7.0f} flops/step ; at 108 ns/step that is {125e6/steps/108e-9/1e9:5.1f} Gflop/s on ONE core")
print("  scalar f64 peak on a 3.5 GHz core, 2 FMA ports = ~14 Gflop/s ; realistic branchy scalar kernel = 1-3 Gflop/s")
print("  flops needed per frame for 10 notes at 125 Mflop/s/note:", 10*125e6*0.0167/1e6, "Mflop")
for g in (1,2,3,4):
    print(f"     at {g} Gflop/s x 4 workers = {4*g} Gflop/s -> {10*125e6*0.0167/(4*g*1e9)*1000:.2f} ms/frame")

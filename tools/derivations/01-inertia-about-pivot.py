"""Moments of inertia about each body's own pivot, Hirschkorn 2004 (Boston GP-178).

Table A.1 gives each body's mass, its inertia about its centre of mass, and the
centre of mass in the body's own frame. Every one of those frames has its origin
at the body's pivot (Figure A.1), so the parallel-axis theorem gives the
inertia about the pivot directly: I_pivot = I_cm + m |cm|^2. That holds for the
wippen too: its pivot is the frame origin, and Pw2, Pw3 in Table A.4 are the
pins of the jack and the repetition lever.

The jack and the lever turn about pins that ride on the wippen, so their
I_pivot is about their own pin, not about anything fixed. Effective masses at
the contacts are not computed here: they need the moment arm of each contact
normal and the assembled mass matrix (script 10), not I_pivot / r^2 with r a
radial distance.

Pure arithmetic: import math and nothing else. Run with python3 -I.
"""
import math

# Table A.1: mass (kg), inertia about the centre of mass (kg m^2), centre of mass (m)
BODIES = {
    "key": (0.12008, 2.75e-3, (0.0514, 0.0158)),
    "wippen": (0.01196, 1.69e-5, (0.0502, -0.0018)),
    "jack": (0.00265, 7.78e-7, (0.0008, 0.0195)),
    "repetition lever": (0.00371, 4.10e-6, (0.0021, -0.0012)),
    "hammer": (0.01174, 2.90e-5, (-0.1022, 0.0145)),
}

print("=== Inertia about each body's own pivot (frame origin = pivot) ===")
for name, (m, i_cm, cm) in BODIES.items():
    r = math.hypot(cm[0], cm[1])
    print(f"  {name:16s} m = {m * 1000:7.2f} g   I_pivot = {i_cm + m * r * r:.4e} kg m^2"
          f"   centre of mass {r * 1000:5.1f} mm from the pivot")

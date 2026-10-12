"""What force at the jack/knuckle does a given hammer acceleration need?

Hirschkorn's hammer (2004, Table A.1; inertia about its pivot from script 01),
driven by the jack under the knuckle. The arm is the moment arm of the
jack/knuckle contact normal about the hammer pivot at rest, |d(gap_7)/d(theta_h)|
= 18.17 mm from script 10. Do not use the radial distance |Ph2| = 19.3 mm: the
arm that counts is ((P - O) x n).z, not |P - O|.

The question matters because a contact's stiffness, and so any time step
derived from it, depends on the force the contact is assumed to carry.

Pure arithmetic: import math and nothing else. Run with python3 -I.
"""
I_h = 1.5409e-4   # hammer inertia about its pivot, kg m^2 (script 01)
m_h = 0.01174     # hammer mass, kg (Table A.1)
d_cm = 0.1022     # centre of mass along the hammer frame's x axis, m (Table A.1); the
                  # weight term is under 1 N, so the hammer's rest tilt does not matter here
r_k = 0.01817     # moment arm of the jack/knuckle normal about the hammer pivot, m (script 10)
r_s = 0.1368      # pivot to the centre of the hammer head's contact circle, m (Table A.7, |Ph4|)
g = 9.81

print("=== Force at the knuckle for a given hammer-head acceleration ===")
print(f"  weight torque {m_h * g * d_cm:.5f} N m -> static force at the knuckle {m_h * g * d_cm / r_k:.2f} N")
print("  head speed v reached in t, with an acceleration growing linearly from zero (peak 2v/t):")
for v, t in ((5.0, 0.020), (5.0, 0.012), (7.8, 0.012), (7.8, 0.008)):
    acc = 2 * v / t
    alpha = acc / r_s
    force = (I_h * alpha + m_h * g * d_cm) / r_k
    print(f"   {v:.1f} m/s in {t * 1e3:4.1f} ms: peak {acc:5.0f} m/s^2 -> {force:6.1f} N at the knuckle")
alpha = (100 * r_k - m_h * g * d_cm) / I_h
print(f"  100 N at the knuckle means a head acceleration of {alpha * r_s:.0f} m/s^2; held constant,"
      f" that reaches 5 m/s in {5 / (alpha * r_s) * 1e3:.2f} ms and 7.8 m/s in {7.8 / (alpha * r_s) * 1e3:.2f} ms")

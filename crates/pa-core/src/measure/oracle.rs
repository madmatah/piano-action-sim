//! The closed forms the procedures are checked against. Tests only.
//!
//! Where a reading the bench takes by running a protocol on the model also
//! comes out of a calculation by hand, that calculation is kept here, as the
//! oracle the procedure is tested against: agreement to round-off says the
//! mechanism carries the lever exactly as the hand does. Nothing the bench
//! shows is computed here. The module is compiled for `pa-core`'s own tests
//! and nowhere else, so no application can reach it.

use crate::geometry::Point;
use crate::parts::{FrameDesign, KeyDesign};
use crate::regulation::Targets;
use crate::test_fixture::KEY_RATIO_LOAD_G;
use crate::G;

/// The key on the level jig, by the closed forms of Stanwood's current
/// protocols: the key's static moment about its balance pin over the
/// measuring arm for front weight, the capstan's arm over the same for the key
/// weight ratio.
///
/// The jig turns the key about its balance pin, so both arms are taken from
/// the pin, never from the bearing on the punching.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyWeighing {
    /// Static moment of the key about the **balance pin**, kg·m, positive
    /// forwards. This is the jig's pivot.
    pub static_moment_about_pin: f64,
    /// Balance pin to measuring point, meters.
    pub measuring_arm: f64,
    /// Balance pin to capstan, meters, as the positive distance behind the pin.
    pub capstan_arm: f64,
}

impl KeyWeighing {
    /// What the jig's scale reads when `load_g` grams sit at `at` meters
    /// along the leveled key, with the scale tared, in grams.
    ///
    /// Moments about the pin, with a downward load at `at` and the roller
    /// pushing up at the measuring point: `load · at − R · arm = 0`. So the
    /// scale reads `load · at / arm`, which is negative for anything placed
    /// behind the pin.
    pub fn jig_reading_g(&self, load_g: f64, at: f64) -> f64 {
        load_g * at / self.measuring_arm
    }

    /// Front weight in grams, signed: the key's whole weight at its center of
    /// gravity, read through the same reading as any other load.
    pub fn front_weight_g(&self) -> f64 {
        self.jig_reading_g(
            1000.0 * self.static_moment_about_pin / self.measuring_arm,
            self.measuring_arm,
        )
    }

    /// Key weight ratio, dimensionless, force convention: the tared reading
    /// with the protocol's load at the capstan, over the load, sign turned.
    pub fn key_ratio(&self) -> f64 {
        -self.jig_reading_g(KEY_RATIO_LOAD_G, -self.capstan_arm) / KEY_RATIO_LOAD_G
    }

    /// What the tared scale reads with the protocol's 10 g at the capstan,
    /// grams. Negative: the load is behind the pin.
    pub fn tared_scale_reading_g(&self) -> f64 {
        self.jig_reading_g(KEY_RATIO_LOAD_G, -self.capstan_arm)
    }
}

/// The key on a pivot at its balance bearing, solved exactly: the bearing's
/// height that stands the key front's top at `key_height` while the key's
/// point `bears`, in its own frame, stands at `stop_y` above the keybed, on
/// its stop. `front_top` is the key front's top in the key's frame.
///
/// Two conditions of the rest, each the height of a point of the key, the
/// bearing's height `h`, the attitude's sine `s` (front-down) and cosine `c`:
/// `h − x s + y c` for the point `(x, y)`. Their difference has no `h` and is
/// linear in `s` and `c`, so the attitude is a root of
/// [`sines_satisfying`], the one nearest `near`, and
/// `h` follows from either condition. Exact for a point fixed on the key on
/// a stop's flat; a key bearing on its cloth's rounded edge slides along it,
/// so `bears` and `stop_y` are the contact the seated pose finds at the
/// answer.
pub fn pivot_bearing_for_height(
    front_top: Point,
    bears: Point,
    stop_y: f64,
    key_height: f64,
    near: f64,
) -> Option<f64> {
    let s = nearest(
        sines_satisfying(
            front_top.y - bears.y,
            -(front_top.x - bears.x),
            key_height - stop_y,
        ),
        near,
    )?;
    let c = (1.0 - s * s).sqrt();
    Some(key_height + front_top.x * s - front_top.y * c)
}

/// The key on a pivot whose bearing stands at `bearing`, at rest with its
/// point `bears` at `stop_y`: the attitude's sine, front-down, the root of
/// one condition nearest `near`, and the height of the key front's top there.
pub fn pivot_rest(
    front_top: Point,
    bears: Point,
    stop_y: f64,
    bearing: f64,
    near: f64,
) -> Option<(f64, f64)> {
    let s = nearest(sines_satisfying(bears.y, -bears.x, stop_y - bearing), near)?;
    let c = (1.0 - s * s).sqrt();
    Some((s, bearing - front_top.x * s + front_top.y * c))
}

/// The key on a pivot whose bearing stands at `bearing`: the height of the
/// front punching's top, its center `punching_x` along the keybed, that
/// stops the key front's top `dip` below `top_height`. The bottom attitude
/// from the key front's height, the root nearest `near`; then the
/// underside's flat, the line through the bearing at that attitude, read at
/// the punching's center, where a felt read at its center bears.
pub fn pivot_punching_for_dip(
    front_top: Point,
    bearing: f64,
    top_height: f64,
    dip: f64,
    punching_x: f64,
    near: f64,
) -> Option<f64> {
    let s = nearest(
        sines_satisfying(front_top.y, -front_top.x, top_height - dip - bearing),
        near,
    )?;
    let c = (1.0 - s * s).sqrt();
    Some(bearing - punching_x * s / c)
}

/// Solves `p·cos θ + q·sin θ = k` for `sin θ`.
///
/// Every condition of the exact pivot inverse reduces to this: the height of
/// a point of the key, written out in the key's own rotation. Clearing the
/// cosine by squaring gives a quadratic in `sin θ`, so the answer is closed
/// form, with no search.
///
/// Squaring admits the branch where the cosine comes out **negative**: the
/// key turned past a right angle, standing on its stop upside down. The
/// quadratic cannot tell the two apart, so each root is put back into the
/// **unsquared** condition and kept only if it satisfies it.
pub fn sines_satisfying(p: f64, q: f64, k: f64) -> [Option<f64>; 2] {
    let a = p * p + q * q;
    if a < 1e-18 {
        return [None, None];
    }
    let b = -2.0 * q * k;
    let c = k * k - p * p;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return [None, None];
    }
    let root = disc.sqrt();
    let genuine = |s: f64| {
        let cos = (1.0 - s * s).max(0.0).sqrt();
        s.is_finite() && s.abs() <= 1.0 && (p * cos + q * s - k).abs() < 1e-9
    };
    let pair = [(-b + root) / (2.0 * a), (-b - root) / (2.0 * a)];
    [
        genuine(pair[0]).then_some(pair[0]),
        genuine(pair[1]).then_some(pair[1]),
    ]
}

/// Of two roots, the one nearest `near`.
fn nearest(roots: [Option<f64>; 2], near: f64) -> Option<f64> {
    roots
        .into_iter()
        .flatten()
        .min_by(|a, b| (a - near).abs().total_cmp(&(b - near).abs()))
}

/// The placement the regulation solve replaced, kept to test it by: the two
/// contact conditions in their small-angle form, solved for the two stack
/// totals, meters, balance then front; `None` where they have no answer.
///
/// It takes the key top as level at both seats, dropping its height times
/// one less the cosine of the attitude, and so misses the targets by that
/// term, about 0.02 mm on the test bench; it reads the front punching
/// alone, not the pin in its mortise. `back_bears` is where the key bears on
/// its cloth, in its own frame, as the seated pose finds it, for the arm
/// the rest turns about.
pub fn small_angle_placement(
    key: &KeyDesign,
    frame: &FrameDesign,
    targets: Targets,
    back_bears: Point,
) -> Option<(f64, f64)> {
    let back_lever = back_bears.x;
    let front_lever = key.front_edge;
    let frp_lever = frame.front_rail;
    if front_lever.abs() < 1e-9 || back_lever.abs() < 1e-9 {
        return None;
    }
    // Rest: the key top at the front edge reaches the target height, and the
    // point of the underside that bears sits on the cloth.
    //   h_bearing - front_lever * sin_rest + stick + keytop = key_height
    //   h_bearing - back_lever * sin_rest + y_bears = h_back
    let h_back = frame.back_bearing_height();
    let top = key.stick_height + key.keytop_thickness;
    let sin_rest = (h_back + top - targets.key_height - back_bears.y) / (front_lever - back_lever);
    let h_bearing = h_back + back_lever * sin_rest - back_bears.y;
    // Bottom: the front edge has fallen by the dip.
    let sin_bottom = sin_rest + targets.key_dip / front_lever;
    let h_front_bearing = h_bearing - frp_lever * sin_bottom;
    let balance_stack = h_bearing - frame.balance_rail_height;
    let front_stack = h_front_bearing - frame.front_rail_height;
    if !(balance_stack.is_finite() && front_stack.is_finite())
        || balance_stack < 0.0
        || front_stack < 0.0
    {
        return None;
    }
    Some((balance_stack, front_stack))
}

/// The two branches of a weighing, grams at the measuring point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Branches {
    /// Down weight, grams.
    pub down_g: f64,
    /// Up weight, grams.
    pub up_g: f64,
}

impl Branches {
    /// Balance weight, `(D + U) / 2`, grams.
    pub fn balance_weight_g(&self) -> f64 {
        0.5 * (self.down_g + self.up_g)
    }
}

/// The key on its frame weighed by the closed form taken with the key level,
/// turning about its balance pin, one friction torque at its bushings: D and
/// U as if the key rested level, which the walks read within the tilt of the
/// seats they start from.
///
/// With the key level, a weight `W` on the measuring point at arm `d` turns
/// it front-down by `W g d`, its own weight by its moment `M`; the bushings
/// hold any torque within `±b`. So the key goes down past
/// `D = (b − M) / (g d)` and rises below `U = (−b − M) / (g d)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyOnFrame {
    /// The moment of the key's weight about its pin, newton-meters, positive
    /// turning the front down.
    pub weight_moment_n_m: f64,
    /// The pin to the measuring point, meters.
    pub measuring_arm_m: f64,
    /// The bushings' friction bound, newton-meters.
    pub friction_n_m: f64,
}

impl KeyOnFrame {
    /// The two branches, grams at the measuring point. Negative where the
    /// key goes down by itself and would have to be held up.
    pub fn branches(&self) -> Branches {
        let per_gram = 1e-3 * G * self.measuring_arm_m;
        Branches {
            down_g: (self.friction_n_m - self.weight_moment_n_m) / per_gram,
            up_g: (-self.friction_n_m - self.weight_moment_n_m) / per_gram,
        }
    }
}

/// The band of a key on a pin at attitude `theta`, radians nose-up, by hand:
/// the weight on `measuring_point` past which the key goes down, and below
/// which it rises, its masses `masses` (kilograms, each at a point of the
/// key's frame, whose origin is the pin) and its bushings holding any torque
/// within `±friction_n_m`.
///
/// A point `(x, y)` of the key at `theta` stands `x cos θ − y sin θ` in front
/// of the pin, so a weight on it turns the key front-down by that arm times
/// its weight. At the attitude a seat stands at, the walk from that seat
/// starts the key moving exactly here.
pub fn band_at(
    masses: &[(f64, Point)],
    measuring_point: Point,
    friction_n_m: f64,
    theta: f64,
) -> Branches {
    let (s, c) = (libm::sin(theta), libm::cos(theta));
    let arm = |p: Point| p.x * c - p.y * s;
    let moment: f64 = masses.iter().map(|&(m, p)| m * G * arm(p)).sum();
    KeyOnFrame {
        weight_moment_n_m: moment,
        measuring_arm_m: arm(measuring_point),
        friction_n_m,
    }
    .branches()
}

/// A key carrying a wippen, at one attitude, by hand: the wippen's angle
/// following the key's through the link that rests it on the key, by
/// `follows` radians of the wippen per radian of the key, its weight folded
/// onto the key's angle through it and its center's friction carried onto
/// the key's.
///
/// A weight `W` on the measuring point at arm `a` turns the key front-down
/// by `W g a`; the key's weight and the wippen's put `Q_k + r·Q_w` on its
/// angle, positive nose-up; the bushings hold `±b_k` and the wippen's center
/// `±b_w`, carried through the link as `±|r|·b_w`. So the key goes down past
/// `D = (b_k + |r|·b_w + Q_k + r·Q_w) / (g a)` and rises below
/// `U = (−b_k − |r|·b_w + Q_k + r·Q_w) / (g a)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Carrying {
    /// The generalized force of the key's weight on its angle, newton-meters,
    /// positive nose-up.
    pub key_force_n_m: f64,
    /// The generalized force of the wippen's weight on its angle,
    /// newton-meters, positive lifting its heel.
    pub wippen_force_n_m: f64,
    /// How the wippen's angle follows the key's, radians per radian.
    pub follows: f64,
    /// The bushings' friction bound, newton-meters.
    pub bushings_n_m: f64,
    /// The wippen center's friction bound, newton-meters.
    pub wippen_friction_n_m: f64,
    /// The measuring point's arm in front of the key's bearing, meters.
    pub measuring_arm_m: f64,
}

impl Carrying {
    /// The two branches, grams at the measuring point.
    pub fn branches(&self) -> Branches {
        KeyOnFrame {
            weight_moment_n_m: -(self.key_force_n_m + self.follows * self.wippen_force_n_m),
            measuring_arm_m: self.measuring_arm_m,
            friction_n_m: self.bushings_n_m + self.follows.abs() * self.wippen_friction_n_m,
        }
        .branches()
    }
}

/// The generalized force of a body's weight on the angle it turns by,
/// newton-meters, positive counter-clockwise: its mass `mass`, kilograms,
/// standing at `center` in the world, turning about `pivot`; `−m g (x_c − x_p)`.
pub fn weight_on_its_angle(mass: f64, center: Point, pivot: Point) -> f64 {
    -mass * G * (center.x - pivot.x)
}

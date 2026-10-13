//! The closed forms the procedures are checked against. Tests only.
//!
//! Where a reading the bench takes by running a protocol on the model also
//! comes out of a calculation by hand, that calculation is kept here, as the
//! oracle the procedure is tested against: agreement to round-off says the
//! mechanism carries the lever exactly as the hand does. Nothing the bench
//! shows is computed here. The module is compiled for `pa-core`'s own tests
//! and nowhere else, so no application can reach it.

use super::KR_PROTOCOL_LOAD_G;
use crate::geometry::Point;
use crate::parts::assembly::{sines_satisfying, KeyAssembly, ReadAt};
use crate::parts::{FrameDesign, KeyDesign};
use crate::regulation::Targets;

/// The key's rest and bottom attitudes by the one-coordinate statics, and
/// the trade quantities they give.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Travel {
    /// Sine of the key angle at rest. Positive means front-down.
    pub sin_rest: f64,
    /// Sine of the key angle at the bottom of the dip.
    pub sin_bottom: f64,
    /// Height of the key's top surface at the front edge, at rest, metres.
    pub key_height: f64,
    /// Vertical travel of the front edge between rest and bottom, metres.
    pub key_dip: f64,
}

/// The key's rest and bottom by the one-coordinate statics, and the key
/// height and dip they give: the closed-form angles of
/// [`KeyAssembly::first_touch`], against which the seated pose
/// ([`super::seat`]) is checked wherever the brute-force oracle confirms
/// them. Leaves with the angle machinery it reads.
pub trait Travelling {
    /// Rest on the back rail cloth, bottom on the front stop, the key height
    /// at rest and the dip, read at the key front.
    fn travel(&self) -> Option<Travel>;
}

impl Travelling for KeyAssembly {
    fn travel(&self) -> Option<Travel> {
        let sin_rest = self.sin_angle_on_back_rail()?;
        let sin_bottom = self.sin_angle_on_front_rail()?;
        Some(Travel {
            sin_rest,
            sin_bottom,
            key_height: self.key_height_at(ReadAt::FrontEdge, sin_rest),
            key_dip: self.key_dip_at(ReadAt::FrontEdge)?,
        })
    }
}

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
    /// Balance pin to measuring point, metres.
    pub measuring_arm: f64,
    /// Balance pin to capstan, metres, as the positive distance behind the pin.
    pub capstan_arm: f64,
}

impl KeyWeighing {
    /// What the jig's scale reads when `load_g` grams sit at `at` metres
    /// along the levelled key, with the scale tared, in grams.
    ///
    /// Moments about the pin, with a downward load at `at` and the roller
    /// pushing up at the measuring point: `load · at − R · arm = 0`. So the
    /// scale reads `load · at / arm`, which is negative for anything placed
    /// behind the pin.
    pub fn jig_reading_g(&self, load_g: f64, at: f64) -> f64 {
        load_g * at / self.measuring_arm
    }

    /// Front weight in grams, signed: the key's whole weight at its centre of
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
        -self.jig_reading_g(KR_PROTOCOL_LOAD_G, -self.capstan_arm) / KR_PROTOCOL_LOAD_G
    }

    /// What the tared scale reads with the protocol's 10 g at the capstan,
    /// grams. Negative: the load is behind the pin.
    pub fn tared_scale_reading_g(&self) -> f64 {
        self.jig_reading_g(KR_PROTOCOL_LOAD_G, -self.capstan_arm)
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
/// [`crate::parts::assembly::sines_satisfying`], the one nearest `near`, and
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
/// front punching's top, its centre `punching_x` along the keybed, that
/// stops the key front's top `dip` below `top_height`. The bottom attitude
/// from the key front's height, the root nearest `near`; then the
/// underside's flat, the line through the bearing at that attitude, read at
/// the punching's centre, where a felt read at its centre bears.
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

/// Of two roots, the one nearest `near`.
fn nearest(roots: [Option<f64>; 2], near: f64) -> Option<f64> {
    roots
        .into_iter()
        .flatten()
        .min_by(|a, b| (a - near).abs().total_cmp(&(b - near).abs()))
}

/// The placement the regulation solve replaced, kept to test it by: the two
/// contact conditions in their small-angle form, solved for the two stack
/// totals, metres, balance then front; `None` where they have no answer.
///
/// It takes the key top as level at both seats, dropping its height times
/// one less the cosine of the attitude, and so misses the targets by that
/// term, about 0.02 mm on the default bench; it reads the front punching
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

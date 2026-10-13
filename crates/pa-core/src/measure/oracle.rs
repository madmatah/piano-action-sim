//! The closed forms the procedures are checked against. Tests only.
//!
//! Where a reading the bench takes by running a protocol on the model also
//! comes out of a calculation by hand, that calculation is kept here, as the
//! oracle the procedure is tested against: agreement to round-off says the
//! mechanism carries the lever exactly as the hand does. Nothing the bench
//! shows is computed here. The module is compiled for `pa-core`'s own tests
//! and nowhere else, so no application can reach it.

use super::KR_PROTOCOL_LOAD_G;
use crate::parts::assembly::{KeyAssembly, ReadAt};

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

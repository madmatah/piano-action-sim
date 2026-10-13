//! The mechanisms a key is put in to be read.
//!
//! A mount is a builder, not a type: the same key, with the same mass, goes
//! into whichever mechanism a procedure needs, and the procedure reads it
//! there. Built: the jig. Planned: the key on its frame, turning about its
//! balance bearing, then free to lift off its balance punching.

use crate::geometry::Point;
use crate::mechanism::{Body, BodyId, Coord, CoordId, Joint, MassProps, Mechanism, Problem};

/// A key in a mechanism, and where it is in it.
#[derive(Debug, Clone, PartialEq)]
pub struct Mounted {
    /// The mechanism.
    pub mechanism: Mechanism,
    /// The key's body. Its frame is the key's own: origin at the balance pin
    /// on the key's underside, `x` towards the player.
    pub key: BodyId,
    /// The key's angle, positive nose-up.
    pub angle: CoordId,
}

/// Stanwood's wedge jig: the key turning about its balance pin's axis, and
/// nothing else.
///
/// Two bodies: the ground, which is the jig, and the key, hung from it by a
/// revolute joint at the origin of the key's frame, on the balance pin's
/// axis. The joint turns without friction, which is an assumption: the real
/// key turns on an extra-long balance pin through its bushings. `mass` is the
/// key's, `None` for a key nobody has weighed.
pub fn jig(mass: Option<MassProps>) -> Result<Mounted, Problem> {
    let key = Body {
        parent: BodyId::GROUND,
        joint: Joint::Revolute {
            pivot: Point::new(0.0, 0.0),
            coord: CoordId(0),
        },
        mass,
    };
    Ok(Mounted {
        mechanism: Mechanism::new(vec![Body::GROUND, key], vec![Coord::ANGLE])?,
        key: BodyId(1),
        angle: CoordId(0),
    })
}

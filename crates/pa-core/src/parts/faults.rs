//! What about a key and its frame no keyboard could be built from, read off
//! the drawn parts.
//!
//! # Why this exists at all
//!
//! The frame is **placed from the key**: the front rail stands under the key's
//! front mortise, the back rail under its chamfer. That keeps a keyboard
//! consistent without anybody typing a coordinate, and it is the right default
//! — but it is a default, not a law. A measured frame carries its own numbers,
//! a design under test is meant to be pushed until it breaks, and either can
//! arrive here holding a combination no keyboard could be built from.
//!
//! So the rule places and this reports. A fault is **not an error**: the
//! assembly still answers every question put to it, and has to, because
//! watching a bad design misbehave is how the bench teaches. What a fault
//! changes is that the reader is told, in the trade's terms, which part of the
//! picture is not a piano.
//!
//! # Read from what is drawn
//!
//! Every fault compares the drawn ranges of two parts: the front rail pin's
//! top against the front mortise, a punching's top against the key's
//! underside, the cloth's face against the chamfer and the end of the key. It
//! reads them at the nominal pose, the key level with its frame's origin over
//! its row's balance pin, where the balance punching is drawn: a length along
//! the key, carried along by where that pin stands, is a length along the
//! keybed. None of them depends on where the key has come to rest.

use super::frame::FrameRanges;
use super::key::KeyRanges;
use crate::contact::{Part, PartCode, SurfaceRef, Sweep};
use crate::geometry::Seg;

/// A way the frame and the key do not go together.
///
/// Distances are metres, and each is *how far past the limit*, so a fault's
/// number is always positive and shrinking it is always the repair.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fault {
    /// The front rail pin is not inside the key's front mortise.
    ///
    /// The key would be guided by a pin bearing against the wood beside its
    /// slot, or by nothing at all. On the bench this is what a key reads like
    /// when its length has been changed without the frame following.
    PinOutsideTheMortise {
        /// How far past the end of the slot the pin stands, metres.
        by: f64,
    },
    /// The back rail cloth reaches in front of where the tail's chamfer begins.
    ///
    /// The chamfer is cut so the key beds flat on this cloth; cloth in front of
    /// it can only ever be met by the kink, which is the one thing the chamfer
    /// exists to prevent.
    ClothInFrontOfTheChamfer {
        /// How far in front of the kink the cloth's face reaches, metres.
        by: f64,
    },
    /// The back rail cloth reaches past the end of the key.
    ///
    /// Then the key's own corner carries it, on a lever arm that is the key's
    /// length and not the rail's position.
    ClothBehindTheKey {
        /// How far past the tail the cloth's face reaches, metres.
        by: f64,
    },
    /// A punching stands partly off the flat of the key's underside.
    PunchingOffTheUnderside {
        /// Which punching.
        rail: PartCode,
        /// How far past the end of the flat it reaches, metres.
        by: f64,
    },
}

/// The least and the greatest `x` a range's segments reach, over their ends,
/// carried `along` the keybed.
fn span(range: &SurfaceRef, parts: &[Part], along: f64) -> (f64, f64) {
    range
        .segs(parts)
        .iter()
        .flat_map(|s| [s.start().x + along, s.end().x + along])
        .fold((f64::MAX, f64::MIN), |(lo, hi), x| (lo.min(x), hi.max(x)))
}

/// Everything about this key on this frame that no keyboard could be built
/// from, read off their drawn parts: empty for a frame that goes with its key.
///
/// Each entry carries how far past the limit it is, so the bench can say *"the
/// pin misses the mortise by 63 mm"* rather than merely that something is
/// wrong. Ordered the way a technician would walk the key: front first,
/// because that is the end he is holding.
///
/// # Panics
///
/// When `parts` is not the list `key` and `frame` were built into.
pub fn faults(parts: &[Part], key: &KeyRanges, frame: &FrameRanges) -> Vec<Fault> {
    let mut out = Vec::new();
    // Where the key's frame stands along the keybed: over its row's balance
    // pin, the axis its balance punching is turned about.
    let pin = match parts[frame.balance_rail_punching.index()].sweep {
        Sweep::Revolved { axis_x } => axis_x,
        Sweep::Extruded { .. } => 0.0,
    };
    let chamfer = key.underside[0].segs(parts)[0];
    let (tail_end, flat_from) = (chamfer.start().x + pin, chamfer.end().x + pin);
    let (_, flat_to) = span(&key.underside[1], parts, pin);

    // The pin has to stand within the slot cut for it, with its own length
    // inside: a pin whose end is past the end of the mortise is bearing on
    // wood, not running in a slot.
    if let Some(pin_top) = &frame.front_pin_top {
        let (pin_back, pin_front) = span(pin_top, parts, 0.0);
        let (mortise_back, mortise_front) = parts[key.front_mortise.index()]
            .outline
            .iter()
            .flat_map(|s| [s.start().x + pin, s.end().x + pin])
            .fold((f64::MAX, f64::MIN), |(lo, hi), x| (lo.min(x), hi.max(x)));
        let past = (mortise_back - pin_back).max(pin_front - mortise_front);
        if past > 0.0 {
            out.push(Fault::PinOutsideTheMortise { by: past });
        }
    }

    // Both punchings bear on the flat, and a washer is wide.
    for (rail, top) in [
        (PartCode::BalanceRailPunching, &frame.balance_bearing),
        (PartCode::FrontRailPunching, &frame.front_bearing),
    ] {
        let (back, front) = span(top, parts, 0.0);
        let past = (flat_from - back).max(front - flat_to);
        if past > 0.0 {
            out.push(Fault::PunchingOffTheUnderside { rail, by: past });
        }
    }

    // And the cloth has to fall inside the chamfer, whole: its face, between
    // its rounded edges, from where it ends at the back to where it ends at
    // the front.
    let face = frame
        .cloth_top
        .segs(parts)
        .into_iter()
        .find_map(|s| match s {
            Seg::Line { a, b } => Some((b.x.min(a.x), b.x.max(a.x))),
            Seg::Arc { .. } => None,
        });
    if let Some((cloth_back, cloth_front)) = face {
        if cloth_front > flat_from {
            out.push(Fault::ClothInFrontOfTheChamfer {
                by: cloth_front - flat_from,
            });
        }
        if cloth_back < tail_end {
            out.push(Fault::ClothBehindTheKey {
                by: tail_end - cloth_back,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::frame::{FrameDesign, KeyKind};
    use super::super::KeyDesign;
    use super::*;
    use crate::mechanism::BodyId;

    /// The faults of `key` on `frame`, the natural's row, read off the parts
    /// the bench would draw.
    fn faults_of(key: &KeyDesign, frame: &FrameDesign) -> Vec<Fault> {
        let mut parts = Vec::new();
        let frame_ranges = frame.parts(key, KeyKind::Natural, &mut parts);
        let key_ranges = key.parts(BodyId(1), &mut parts);
        faults(&parts, &key_ranges, &frame_ranges)
    }

    /// A frame built for its key reports nothing.
    ///
    /// The baseline every other test here is read against: if the default
    /// assembly carried a fault, the badge would be permanent and therefore
    /// invisible, which is worse than having no badge at all.
    #[test]
    fn a_frame_built_for_its_key_has_nothing_to_report() {
        let key = KeyDesign::default();
        let frame = FrameDesign::default().placed_under(&key);
        assert_eq!(
            faults_of(&key, &frame),
            Vec::new(),
            "the default keyboard must be one that could be built"
        );
    }

    /// Lengthening a key leaves the front rail where it was bolted, and the
    /// bench says by how much.
    ///
    /// **This is the behaviour, not a defect.** Rails are screwed to a keybed;
    /// nothing about cutting a longer key moves them. A model that quietly
    /// slid the rail along would draw a keyboard that cannot exist and call it
    /// correct — and it would hide the one thing a keyboard designer is here
    /// to find out, which is what else has to change when a key's length does.
    #[test]
    fn lengthening_a_key_leaves_the_front_rail_behind_and_names_the_miss() {
        let frame = FrameDesign::default();
        let short = KeyDesign {
            front_edge: 0.158,
            ..KeyDesign::default()
        };
        let moved = 0.258 - 0.158;

        let faults = faults_of(&short, &frame);
        let miss = faults
            .iter()
            .find_map(|f| match f {
                Fault::PinOutsideTheMortise { by } => Some(*by),
                _ => None,
            })
            .expect("the pin is 100 mm from a mortise 8 mm long");
        // The mortise moved with the front edge; the pin did not. What is left
        // over is the travel, less half the slot and half the pin.
        let slack = 0.5 * (0.008 - 0.005);
        assert!(
            (miss - (moved - slack)).abs() < 1e-9,
            "the miss should be {} m, and reads {miss} m",
            moved - slack
        );

        // And the punching has walked off the end of the key with it.
        assert!(
            faults
                .iter()
                .any(|f| matches!(f, Fault::PunchingOffTheUnderside { .. })),
            "a 22 mm washer 100 mm past the end of the key is off the underside"
        );
    }

    /// Placing the frame under the longer key clears both.
    ///
    /// The repair, and the other half of the rule: drift is the default, and
    /// re-placing the frame is the deliberate act that ends it. A fault that
    /// nothing could clear would be a complaint, not a diagnosis.
    #[test]
    fn placing_the_frame_under_the_new_key_clears_the_miss() {
        let key = KeyDesign {
            front_edge: 0.158,
            ..KeyDesign::default()
        };
        let frame = FrameDesign::default().placed_under(&key);
        assert_eq!(faults_of(&key, &frame), Vec::new());
    }

    /// Cloth in front of the chamfer, and cloth past the end of the key, are
    /// each named.
    ///
    /// The two bounds run opposite ways and it is easy to write one of them
    /// backwards. In front of the chamfer the key can only ever meet the cloth
    /// on the kink; behind the end of the key it meets it on its own corner.
    /// Both are keyboards somebody could build by accident and neither is one
    /// that works.
    #[test]
    fn the_cloth_has_to_fall_inside_the_chamfer_at_both_ends() {
        let key = KeyDesign::default();
        let (chamfer_begins, _) = key.underside_flat();
        let half = FrameDesign::default().sections.back.half_length;

        let forward = FrameDesign {
            back_rail: chamfer_begins,
            ..FrameDesign::default()
        };
        let faults = faults_of(&key, &forward);
        assert!(
            faults.iter().any(
                |f| matches!(f, Fault::ClothInFrontOfTheChamfer { by } if (by - half).abs() < 1e-9)
            ),
            "cloth centred on the kink reaches half a rail past it; got {faults:?}"
        );

        let behind = FrameDesign {
            back_rail: key.tail_end,
            ..FrameDesign::default()
        };
        let faults = faults_of(&key, &behind);
        assert!(
            faults
                .iter()
                .any(|f| matches!(f, Fault::ClothBehindTheKey { by } if (by - half).abs() < 1e-9)),
            "cloth centred on the tail reaches half a rail past it; got {faults:?}"
        );
    }

    /// Every fault counts from the limit, so repairing one shrinks it to zero
    /// and never to a jump.
    ///
    /// A badge that read "wrong" would say the same thing 1 mm out and 100 mm
    /// out. The number is what tells a reader whether he has mis-typed a
    /// dimension or mis-designed a keyboard.
    #[test]
    fn a_fault_shrinks_to_nothing_as_the_frame_comes_right() {
        let key = KeyDesign::default();
        let mut previous = f64::MAX;
        for front_rail in [0.100_f64, 0.150, 0.200, 0.220, 0.223] {
            let frame = FrameDesign {
                front_rail,
                ..FrameDesign::default()
            };
            let by = faults_of(&key, &frame)
                .iter()
                .find_map(|f| match f {
                    Fault::PinOutsideTheMortise { by } => Some(*by),
                    _ => None,
                })
                .unwrap_or(0.0);
            assert!(
                by < previous,
                "moving the rail towards the mortise must shrink the miss, \
                 and {front_rail} m read {by} m after {previous} m"
            );
            previous = by;
        }
        assert!(
            previous > 0.0,
            "the sweep should stop just short of correct"
        );
    }
}

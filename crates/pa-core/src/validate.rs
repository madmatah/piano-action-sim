//! The validation layer: what no part could be built as, refused before any
//! solve.
//!
//! # Refuse, compute, report
//!
//! An edit to the instrument is sorted three ways. What no part could be
//! built as is **refused**, the state left as it was, with a [`Problem`]
//! naming the limiting dimension and how far past it the edit goes: a value
//! that is not a number, one outside its setting's hard range
//! ([`crate::registry::ParamSpec`]), a key weighed at no mass, and a key
//! whose drawn parts could not be cut from wood: its covering run into its
//! key button, its thinned tail run on under its back check block, its
//! capstan in a glued block, behind the back check block or on the thinned
//! tail, a clearance cut in its top under its capstan or into a block, its
//! front mortise cut through the top of the key, its measuring point off its
//! covering. Everything an assembled keyboard can be, however badly
//! regulated, is **computed and reported**: a frame placed for another key,
//! stops that fight, a pin standing proud of its mortise, a key regulated
//! into jamming are states, which the faults, the verdicts and the sheet's
//! refusals describe, and are never refused here. What is computed is
//! **drawn as it is**.
//!
//! [`validate`] runs on every edit and every load, before any solve, so the
//! state an instrument holds always passes it: no number that is not finite
//! reaches the statics, and no outline the key is drawn by crosses itself.

use crate::contact::PartCode;
use crate::measure::TouchweightProtocol;
use crate::mechanism::AxisFriction;
use crate::parts::frame::{Crown, FrameSections, PinShape, RailSection, PAPER};
use crate::parts::{FrameDesign, KeyDesign, KeyMass};

/// Why an edit is refused: what it would make that no part could be built
/// as, naming the limiting dimension and the shortfall, SI.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Problem {
    /// A value that is not a number, or an infinity.
    NotFinite {
        /// The setting or the dimension, by its name.
        param: &'static str,
    },
    /// A value outside its setting's hard range.
    OutOfRange {
        /// The setting, by its name ([`crate::registry::ParamSpec::name`]).
        param: &'static str,
        /// The value asked for.
        value: f64,
        /// The least the setting admits.
        min: f64,
        /// The most it admits.
        max: f64,
    },
    /// A paper no drawer holds.
    NotInTheDrawer {
        /// The setting.
        param: &'static str,
        /// The thickness asked for, meters.
        thickness: f64,
    },
    /// A key weighed at no mass, or less.
    MassNotPositive {
        /// The mass typed, kilograms.
        mass: f64,
    },
    /// The key's outline crosses itself: two parts glued on the keystick
    /// overlapping along its top, the clearance cut in its top run into one
    /// of them or under the capstan's head, the thinned tail's felt run on
    /// under a glued block, a block or the covering standing across the step
    /// down to the thinned tail, or the underside's chamfer running past the
    /// key front.
    OutlineCrossesItself {
        /// The part behind, whose front end stands too far forward.
        back: PartCode,
        /// The part in front, whose back end stands too far back.
        front: PartCode,
        /// How far the front end of the part behind stands in front of the
        /// back end of the part in front, meters along the key: how far the
        /// two overlap, where neither runs past the other's far end.
        by: f64,
    },
    /// A feature that stands off the keystick's wood where it must be cut:
    /// the front mortise off the flat of the underside, or the clearance in
    /// its top off its full height, run back onto the thinned tail.
    FeatureOutsideKeystick {
        /// The feature.
        part: PartCode,
        /// How far off, meters along the key.
        by: f64,
    },
    /// The capstan stands wholly behind the back check block: on the
    /// keystick's full height between the block and the thinned tail, on the
    /// thinned tail, or past the key's end. It stands between the key button
    /// and the back check block.
    CapstanBehindBackCheckBlock {
        /// How far forward it must come for its head to clear the block,
        /// meters along the key.
        by: f64,
    },
    /// The capstan stands on the thinned tail, or across the step down to
    /// it: the back of its head behind the step. It screws into the
    /// keystick's full height.
    CapstanOffTheFullHeight {
        /// How far forward it must come for its head to stand wholly over
        /// the full height, meters along the key.
        by: f64,
    },
    /// The capstan's dome is narrower than its head: a sphere of that radius
    /// cannot cap the head.
    CapstanDomeNarrowerThanItsHead {
        /// The dome's radius, meters.
        dome: f64,
        /// The head's radius, meters.
        head: f64,
    },
    /// The capstan stands in a block glued on the keystick: the key button
    /// or the back check block.
    CapstanInBlock {
        /// The block.
        block: PartCode,
        /// How far the capstan's head stands into it, meters along the key.
        by: f64,
    },
    /// The clearance in the keystick's top is no cut: its front end not in
    /// front of its back end, or its depth nothing or less.
    ClearanceNotACut {
        /// Its front end less its back end, meters along the key.
        length: f64,
        /// Its depth below the keystick's full top, meters.
        depth: f64,
    },
    /// The clearance in the keystick's top is cut through it: as deep as the
    /// wood under it, or deeper, the keystick's full height less the rise of
    /// the chamfer under the tail where that runs under its back end.
    ClearanceThroughKeystick {
        /// Its depth below the keystick's full top, meters.
        depth: f64,
        /// The wood under its back end, where the keystick is thinnest
        /// under it, meters.
        wood: f64,
    },
    /// The key button's two rounded ends reach past each other along its
    /// top: each reaches in from its end ([`crate::parts::end_reach`]), and
    /// the two together run longer than the button.
    CapsuleEndsOverlap {
        /// How far the two reach past each other, meters along the key.
        by: f64,
    },
    /// A part of the balance mortise does not fit where it is cut or set:
    /// the slot along the key button's flat top, the cavity's floor at or
    /// under the keystick's top and the slot within its width, the balance
    /// hole within the slot, the bushing's two thicknesses within the
    /// slot's width and its depth within the key button's height, and the
    /// side holes, no lower than the cloth turned out through them is
    /// thick, within the key button's height.
    DoesNotFit {
        /// The part that does not fit.
        part: PartCode,
        /// What it does not fit in.
        within: PartCode,
        /// By how much, meters.
        by: f64,
    },
    /// The front mortise is cut through the top of the keystick.
    MortiseThroughKeyTop {
        /// Its depth above the underside, meters.
        depth: f64,
        /// The keystick's wood it is cut into, meters.
        wood: f64,
    },
    /// The measuring point stands off the covering: in front of its lip, or
    /// behind its back end.
    MeasuringPointOffCovering {
        /// Where it was asked for, meters back from the lip.
        from_lip: f64,
        /// How far the covering runs back from its lip, meters.
        covering: f64,
    },
}

/// Everything about an instrument that no part could be built as, if any,
/// read before any solve.
#[derive(Debug, Clone, Copy)]
pub struct Design<'a> {
    /// The key.
    pub key: &'a KeyDesign,
    /// What is known of its mass.
    pub mass: &'a KeyMass,
    /// The frame, with the papers its punchings hold.
    pub frame: &'a FrameDesign,
    /// The protocol the key is weighed by.
    pub protocol: &'a TouchweightProtocol,
    /// The bushings' friction.
    pub friction: &'a AxisFriction,
}

/// Whether `d` is a design parts could be built as; the first problem found
/// where it is not, in this order: every number finite, every setting within
/// its hard range, every paper one the drawer holds, a weighed mass, the
/// key's outline, the capstan between its blocks on the keystick's full
/// height, the front mortise on the flat of the underside and under the
/// key's top, the balance mortise and the key button's ends where they
/// fit, and the measuring point on the covering.
pub fn validate(d: &Design<'_>) -> Result<(), Problem> {
    for (param, value) in numbers(d) {
        if !value.is_finite() {
            return Err(Problem::NotFinite { param });
        }
    }
    for spec in &KeyDesign::PARAMS {
        spec.holds(d.key)?;
    }
    for spec in &FrameDesign::PARAMS {
        spec.holds(d.frame)?;
    }
    for spec in &TouchweightProtocol::PARAMS {
        spec.holds(d.protocol)?;
    }
    for spec in &AxisFriction::PARAMS {
        spec.holds(d.friction)?;
    }
    for paper in d
        .frame
        .balance_punching
        .papers
        .iter()
        .chain(&d.frame.front_punching.papers)
    {
        d.frame.drawer.admits(*paper)?;
    }
    if let KeyMass::Weighed { mass, .. } | KeyMass::Published { mass, .. } = *d.mass {
        if mass <= 0.0 {
            return Err(Problem::MassNotPositive { mass });
        }
    }
    outline(d.key)?;
    capstan(d.key)?;
    mortise(d.key)?;
    key_button(d.key)?;
    balance_mortise(d.key)?;
    let covering = d.key.keytop_length;
    let from_lip = d.protocol.measuring_point_from_lip;
    if !(0.0..=covering).contains(&from_lip) {
        return Err(Problem::MeasuringPointOffCovering { from_lip, covering });
    }
    Ok(())
}

/// The key's outline along its top and its underside, back to front. The
/// clearance cut in the top, where the key has one, is a cut, its front end
/// in front of its back end and its floor below the full top, and leaves
/// wood under it, where the chamfer under the tail may run on under it.
/// Each block glued on the key, the back check block and the
/// key button, stands wholly on the thinned tail or wholly in front of it,
/// and the covering and the clearance in front of it: a felted tail runs on
/// under none of them, and none stands across the step down to a bare one.
/// Each glued part, and the clearance between the back check block and the
/// key button, ends before the next begins; and the chamfer under the tail
/// runs out before the key front. Two that overlap are the outline crossing
/// itself, by how far they overlap.
fn outline(key: &KeyDesign) -> Result<(), Problem> {
    let tail_front = key.tail_front();
    let check = key.back_check_block();
    let (button_back, button_front) = key.key_button_span();
    let keytop_back = key.lip() - key.keytop_length;
    let wood_front = key.front_edge - key.front_board;
    let chamfer_end = key.tail_end + key.tail_rise_run;
    if let Some(c) = key.top_clearance {
        if !(c.front > c.back && c.depth > 0.0) {
            return Err(Problem::ClearanceNotACut {
                length: c.front - c.back,
                depth: c.depth,
            });
        }
    }
    let mut glued = vec![(PartCode::BackCheckBlock, check.back, check.front())];
    if let Some(c) = key.top_clearance {
        glued.push((PartCode::TopClearance, c.back, c.front));
    }
    glued.extend([
        (PartCode::KeyButton, button_back, button_front),
        (PartCode::Keytop, keytop_back, key.lip()),
    ]);
    for &(part, back, front) in &glued {
        if part == PartCode::TopClearance {
            // Cut in the full height, felted tail or bare: wholly in front
            // of the tail, or run back onto it.
            if tail_front > back {
                return Err(Problem::FeatureOutsideKeystick {
                    part,
                    by: tail_front - back,
                });
            }
            // And not through it. The chamfer under the tail may run on
            // under the clearance, rising toward the key's end, so the
            // wood is thinnest under its back end.
            if let Some(c) = key.top_clearance {
                let wood = key.stick_height - underside_at(key, back);
                if c.depth >= wood {
                    return Err(Problem::ClearanceThroughKeystick {
                        depth: c.depth,
                        wood,
                    });
                }
            }
            continue;
        }
        let felted = key.damper_felt.is_some();
        let covering = part == PartCode::Keytop;
        if (felted || covering) && tail_front > back {
            return Err(Problem::OutlineCrossesItself {
                back: if felted {
                    PartCode::KeyTailFelt
                } else {
                    PartCode::Keystick
                },
                front: part,
                by: tail_front - back,
            });
        }
        if back < tail_front && tail_front < front {
            return Err(Problem::OutlineCrossesItself {
                back: part,
                front: PartCode::Keystick,
                by: front - tail_front,
            });
        }
    }
    for pair in glued.windows(2) {
        let (back, _, ends) = pair[0];
        let (front, begins, _) = pair[1];
        if ends > begins {
            return Err(Problem::OutlineCrossesItself {
                back,
                front,
                by: ends - begins,
            });
        }
    }
    if chamfer_end > wood_front {
        return Err(Problem::OutlineCrossesItself {
            back: PartCode::Keystick,
            front: PartCode::KeyFront,
            by: chamfer_end - wood_front,
        });
    }
    Ok(())
}

/// How high the key's underside stands at `x`, meters, `x` at or in front of
/// the key's end: on the chamfer under the tail, the straight line the key
/// is drawn with from its end, `tail_rise` high, to the flat; nothing on the
/// flat.
fn underside_at(key: &KeyDesign, x: f64) -> f64 {
    let flat_from = key.tail_end + key.tail_rise_run;
    if x < flat_from {
        key.tail_rise * (flat_from - x) / key.tail_rise_run
    } else {
        0.0
    }
}

/// The capstan's widest radius, meters: its head or its shaft.
fn capstan_radius(key: &KeyDesign) -> f64 {
    0.5 * key
        .capstan_screw
        .head_diameter
        .max(key.capstan_screw.diameter)
}

/// The capstan stands between the back check block and the key button,
/// where nothing is glued on top, screwed into the keystick's full height:
/// its head in either block is the capstan in that block, by how far it
/// stands in; the whole of it behind the back check block, wherever the
/// key's wood runs there, is behind the block, by how far it must come
/// forward to clear it, the limit the block's, which the action places, and
/// not the key's end; in front of the block, its head over the thinned tail
/// or across the step down to it is off the full height, by how far it must
/// come forward to stand over the full height. The clearance cut in the top,
/// where the key has one, stands behind its head: run under it, it is the
/// outline crossing itself, by how far. Where its head is a dome, the dome
/// caps the head.
fn capstan(key: &KeyDesign) -> Result<(), Problem> {
    let r = capstan_radius(key);
    let (back, front) = (key.capstan - r, key.capstan + r);
    let block_back = key.back_check_block_back;
    let block_front = block_back + key.back_check_block_length;
    let (button_back, button_front) = key.key_button_span();
    if let Some(dome) = key.capstan_screw.dome_radius {
        let head = 0.5 * key.capstan_screw.head_diameter;
        if dome < head {
            return Err(Problem::CapstanDomeNarrowerThanItsHead { dome, head });
        }
    }
    if front > button_back && back < button_front {
        return Err(Problem::CapstanInBlock {
            block: PartCode::KeyButton,
            by: front - button_back,
        });
    }
    if front <= block_back {
        return Err(Problem::CapstanBehindBackCheckBlock {
            by: block_front - back,
        });
    }
    if back < block_front {
        return Err(Problem::CapstanInBlock {
            block: PartCode::BackCheckBlock,
            by: block_front - back,
        });
    }
    let tail_front = key.tail_front();
    if back < tail_front {
        return Err(Problem::CapstanOffTheFullHeight {
            by: tail_front - back,
        });
    }
    if let Some(c) = key.top_clearance {
        if c.front > back {
            return Err(Problem::OutlineCrossesItself {
                back: PartCode::TopClearance,
                front: PartCode::Capstan,
                by: c.front - back,
            });
        }
    }
    Ok(())
}

/// The key button's rounded ends meet at most: each reaches in from its end
/// ([`crate::parts::end_reach`]), and the two together run no longer than
/// the button.
fn key_button(key: &KeyDesign) -> Result<(), Problem> {
    let reach = crate::parts::end_reach(key.capsule_height, key.capsule_end_radius);
    if 2.0 * reach > key.capsule_length {
        return Err(Problem::CapsuleEndsOverlap {
            by: 2.0 * reach - key.capsule_length,
        });
    }
    Ok(())
}

/// The balance mortise fits where it is cut: its slot along the key
/// button's flat top, where it opens through it, and not into a rounded
/// end; its cavity's floor no higher than the keystick's top and its width
/// within the keystick's; its hole within the slot; its bushing's two
/// thicknesses within the slot's width, the cloth no thicker than the side
/// holes it turns out through, and its depth within the key button's
/// height; and the side holes within the key button's wall, their bottom
/// no lower than the key button's and their top, where the cloth is glued,
/// no higher than its top. A side hole whose top is the key button's top,
/// a groove open at the top, is taken: it can be drawn.
fn balance_mortise(key: &KeyDesign) -> Result<(), Problem> {
    let reach = crate::parts::end_reach(key.capsule_height, key.capsule_end_radius);
    let mortise = key.balance_mortise;
    let does_not_fit = |part, within, by| Err(Problem::DoesNotFit { part, within, by });
    let (button_back, button_front) = key.key_button_span();
    let (flat_back, flat_front) = (button_back + reach, button_front - reach);
    let past_the_flat = (0.5 * mortise.length - flat_front).max(flat_back + 0.5 * mortise.length);
    if past_the_flat > 0.0 {
        return does_not_fit(PartCode::BalanceMortise, PartCode::KeyButton, past_the_flat);
    }
    if mortise.floor > key.stick_height {
        return does_not_fit(
            PartCode::BalanceMortise,
            PartCode::Keystick,
            mortise.floor - key.stick_height,
        );
    }
    if mortise.width > key.width {
        return does_not_fit(
            PartCode::BalanceMortise,
            PartCode::Keystick,
            mortise.width - key.width,
        );
    }
    let narrowest = mortise.length.min(mortise.width);
    if mortise.hole_diameter > narrowest {
        return does_not_fit(
            PartCode::BalanceHole,
            PartCode::BalanceMortise,
            mortise.hole_diameter - narrowest,
        );
    }
    if 2.0 * mortise.bushing_thickness > mortise.width {
        return does_not_fit(
            PartCode::BalanceMortiseBushing,
            PartCode::BalanceMortise,
            2.0 * mortise.bushing_thickness - mortise.width,
        );
    }
    if mortise.side_hole_height < mortise.bushing_thickness {
        return does_not_fit(
            PartCode::BalanceMortiseBushing,
            PartCode::KeyButtonSideHole,
            mortise.bushing_thickness - mortise.side_hole_height,
        );
    }
    if mortise.bushing_depth > key.capsule_height {
        return does_not_fit(
            PartCode::BalanceMortiseBushing,
            PartCode::KeyButton,
            mortise.bushing_depth - key.capsule_height,
        );
    }
    let below_the_key_button = mortise.bushing_depth - mortise.bushing_thickness
        + mortise.side_hole_height
        - key.capsule_height;
    if below_the_key_button > 0.0 {
        return does_not_fit(
            PartCode::KeyButtonSideHole,
            PartCode::KeyButton,
            below_the_key_button,
        );
    }
    if mortise.bushing_depth < mortise.bushing_thickness {
        return does_not_fit(
            PartCode::KeyButtonSideHole,
            PartCode::KeyButton,
            mortise.bushing_thickness - mortise.bushing_depth,
        );
    }
    Ok(())
}

/// The front mortise is cut up from the flat of the underside, between the
/// chamfer and the key front, and leaves wood above it.
fn mortise(key: &KeyDesign) -> Result<(), Problem> {
    let (back, front) = key.front_mortise_span();
    let chamfer_end = key.tail_end + key.tail_rise_run;
    let wood_front = key.front_edge - key.front_board;
    if front > wood_front {
        return Err(Problem::FeatureOutsideKeystick {
            part: PartCode::FrontMortise,
            by: front - wood_front,
        });
    }
    if back < chamfer_end {
        return Err(Problem::FeatureOutsideKeystick {
            part: PartCode::FrontMortise,
            by: chamfer_end - back,
        });
    }
    if key.front_mortise_depth >= key.stick_height {
        return Err(Problem::MortiseThroughKeyTop {
            depth: key.front_mortise_depth,
            wood: key.stick_height,
        });
    }
    Ok(())
}

/// Every number of the design, by name, in a fixed order, every field
/// read: a field added to a part and not listed here is a compile error, so
/// no number reaches the statics unchecked. What [`validate`] reads for
/// finiteness, what an instrument's state hash reads of its design, and
/// what an instrument's profile must bind, each to a record or a stated
/// assumption. Each name is the number's own, and named once, but for the
/// papers on the punchings ([`PAPER`]'s name) and in the drawer
/// ([`DRAWER`]), which are lists; a number a setting sets has the setting's
/// name.
pub fn numbers(d: &Design<'_>) -> Vec<(&'static str, f64)> {
    let KeyDesign {
        front_edge,
        tail_end,
        capstan,
        front_mortise_from_front_edge,
        front_mortise_length,
        front_mortise_depth,
        stick_height,
        capsule_height,
        capsule_length,
        capsule_center,
        capsule_end_radius,
        balance_mortise,
        back_check_block_height,
        back_check_block_back,
        back_check_block_length,
        top_clearance,
        tail_height,
        tail_length,
        damper_felt,
        tail_rise,
        tail_rise_run,
        keytop_thickness,
        keytop_length,
        front_board,
        keytop_overhang,
        width,
        capstan_screw,
    } = d.key;
    let crate::parts::Capstan {
        height,
        diameter,
        head_diameter,
        head_height,
        cove_height,
        shaft_length,
        tip_height,
        dome_radius,
    } = capstan_screw;
    let mut out = vec![
        ("front_edge", *front_edge),
        ("tail_end", *tail_end),
        ("capstan.x", *capstan),
        (
            "front_mortise.from_front_edge",
            *front_mortise_from_front_edge,
        ),
        ("front_mortise.length", *front_mortise_length),
        ("front_mortise.depth", *front_mortise_depth),
        ("stick_height", *stick_height),
        ("capsule_height", *capsule_height),
        ("capsule_length", *capsule_length),
    ];
    if let Some(c) = capsule_center {
        out.push(("capsule_center", *c));
    }
    let crate::parts::BalanceMortise {
        length,
        width: mortise_width,
        floor,
        hole_diameter,
        bushing_thickness,
        bushing_depth,
        side_hole_height,
    } = balance_mortise;
    out.extend([
        ("capsule_end_radius", *capsule_end_radius),
        ("balance_mortise.length", *length),
        ("balance_mortise.width", *mortise_width),
        ("balance_mortise.floor", *floor),
        ("balance_mortise.hole_diameter", *hole_diameter),
        ("balance_mortise.bushing_thickness", *bushing_thickness),
        ("balance_mortise.bushing_depth", *bushing_depth),
        ("balance_mortise.side_hole_height", *side_hole_height),
    ]);
    out.extend([
        ("back_check_block_height", *back_check_block_height),
        ("back_check_block_back", *back_check_block_back),
        ("back_check_block_length", *back_check_block_length),
    ]);
    if let Some(crate::parts::TopClearance { back, front, depth }) = top_clearance {
        out.extend([
            ("top_clearance.back", *back),
            ("top_clearance.front", *front),
            ("top_clearance.depth", *depth),
        ]);
    }
    out.extend([("tail_height", *tail_height), ("tail_length", *tail_length)]);
    if let Some(felt) = damper_felt {
        out.push(("damper_felt", *felt));
    }
    out.extend([
        ("tail_rise", *tail_rise),
        ("tail_rise_run", *tail_rise_run),
        ("keytop_thickness", *keytop_thickness),
        ("keytop_length", *keytop_length),
        ("front_board", *front_board),
        ("keytop_overhang", *keytop_overhang),
        ("width", *width),
        ("capstan.height", *height),
        ("capstan.diameter", *diameter),
        ("capstan.head_diameter", *head_diameter),
        ("capstan.head_height", *head_height),
        ("capstan.cove_height", *cove_height),
        ("capstan.shaft_length", *shaft_length),
        ("capstan.tip_height", *tip_height),
    ]);
    if let Some(r) = dome_radius {
        out.push(("capstan.dome_radius", *r));
    }
    match *d.mass {
        KeyMass::Unweighed => {}
        KeyMass::Published {
            mass,
            center,
            inertia,
        } => out.extend([
            ("published.mass", mass),
            ("published.center.x", center.x),
            ("published.center.y", center.y),
            ("published.inertia", inertia),
        ]),
        KeyMass::Weighed {
            mass,
            static_moment_n_m,
            datum,
        } => out.extend([
            ("weighing.mass", mass),
            ("weighing.static_moment", static_moment_n_m),
            ("weighing.front_weight_g", datum.grams),
            ("weighing.from_lip", datum.from_lip_m),
            ("weighing.arm", datum.arm_m),
        ]),
    }
    let FrameDesign {
        balance_rail_height,
        balance_punching,
        front_rail_height,
        front_punching,
        back_rail_height,
        back_rail_cloth,
        back_rail_cloth_edge_radius,
        front_rail,
        back_rail,
        back_rail_from_the_tail,
        sections,
        drawer,
    } = d.frame;
    out.extend([
        ("balance_rail.height", *balance_rail_height),
        ("balance_punching.felt", balance_punching.felt),
        ("front_rail.height", *front_rail_height),
        ("front_punching.felt", front_punching.felt),
        ("back_rail.height", *back_rail_height),
        ("back_rail.cloth", *back_rail_cloth),
        ("back_rail.cloth_edge_radius", *back_rail_cloth_edge_radius),
        ("front_rail.x", *front_rail),
        ("back_rail.x", *back_rail),
        ("back_rail.from_the_tail", *back_rail_from_the_tail),
    ]);
    out.extend(
        balance_punching
            .papers
            .iter()
            .chain(&front_punching.papers)
            .map(|&p| (PAPER.name, p)),
    );
    let FrameSections {
        balance,
        front,
        back,
    } = sections;
    section_numbers(Rail::Balance, balance, &mut out);
    section_numbers(Rail::Front, front, &mut out);
    section_numbers(Rail::Back, back, &mut out);
    out.extend(drawer.thicknesses().iter().map(|&t| (DRAWER, t)));
    let TouchweightProtocol {
        measuring_point_from_lip,
        key_ratio_load_g,
    } = d.protocol;
    out.extend([
        ("measuring_point_from_lip", *measuring_point_from_lip),
        ("key_ratio_load_g", *key_ratio_load_g),
    ]);
    let AxisFriction {
        bound_n_m,
        radius_m,
    } = d.friction;
    out.push(("friction.bound", *bound_n_m));
    if let Some(r) = radius_m {
        out.push(("friction.radius", *r));
    }
    out
}

/// The name of each paper a frame's drawer holds, among the design's
/// numbers.
pub const DRAWER: &str = "drawer";

/// One of the three rails, as its section's numbers are named.
#[derive(Debug, Clone, Copy)]
enum Rail {
    Balance,
    Front,
    Back,
}

/// A rail section's numbers, each named by its rail and its field, every
/// field read.
fn section_numbers(rail: Rail, s: &RailSection, out: &mut Vec<(&'static str, f64)>) {
    let RailSection {
        half_length,
        crown,
        pin,
        pin_height,
        punching_radius,
        punching_hole_radius,
        sharp_row_offset,
    } = s;
    let name = |field: Section| section_name(rail, field);
    out.extend([
        (name(Section::HalfLength), *half_length),
        (name(Section::PinHeight), *pin_height),
        (name(Section::PunchingRadius), *punching_radius),
        (name(Section::PunchingHoleRadius), *punching_hole_radius),
        (name(Section::SharpRowOffset), *sharp_row_offset),
    ]);
    if let Some(Crown {
        half_length,
        height,
    }) = crown
    {
        out.extend([
            (name(Section::CrownHalfLength), *half_length),
            (name(Section::CrownHeight), *height),
        ]);
    }
    match pin {
        Some(PinShape::Round { diameter }) => out.push((name(Section::PinDiameter), *diameter)),
        Some(PinShape::Oblong { length, thickness }) => out.extend([
            (name(Section::PinLength), *length),
            (name(Section::PinThickness), *thickness),
        ]),
        None => {}
    }
}

/// A number of a rail's section.
#[derive(Debug, Clone, Copy)]
enum Section {
    HalfLength,
    PinHeight,
    PunchingRadius,
    PunchingHoleRadius,
    SharpRowOffset,
    CrownHalfLength,
    CrownHeight,
    PinDiameter,
    PinLength,
    PinThickness,
}

/// The name of `field` of `rail`'s section: the rail, then the field, as a
/// profile binds it; a pin's height is the name its setting has.
fn section_name(rail: Rail, field: Section) -> &'static str {
    use Rail::{Back, Balance, Front};
    use Section::*;
    match (rail, field) {
        (Balance, HalfLength) => "balance_rail.half_length",
        (Balance, PinHeight) => "balance_rail.pin_height",
        (Balance, PunchingRadius) => "balance_rail.punching_radius",
        (Balance, PunchingHoleRadius) => "balance_rail.punching_hole_radius",
        (Balance, SharpRowOffset) => "balance_rail.sharp_row_offset",
        (Balance, CrownHalfLength) => "balance_rail.crown.half_length",
        (Balance, CrownHeight) => "balance_rail.crown.height",
        (Balance, PinDiameter) => "balance_rail.pin.diameter",
        (Balance, PinLength) => "balance_rail.pin.length",
        (Balance, PinThickness) => "balance_rail.pin.thickness",
        (Front, HalfLength) => "front_rail.half_length",
        (Front, PinHeight) => "front_rail.pin_height",
        (Front, PunchingRadius) => "front_rail.punching_radius",
        (Front, PunchingHoleRadius) => "front_rail.punching_hole_radius",
        (Front, SharpRowOffset) => "front_rail.sharp_row_offset",
        (Front, CrownHalfLength) => "front_rail.crown.half_length",
        (Front, CrownHeight) => "front_rail.crown.height",
        (Front, PinDiameter) => "front_rail.pin.diameter",
        (Front, PinLength) => "front_rail.pin.length",
        (Front, PinThickness) => "front_rail.pin.thickness",
        (Back, HalfLength) => "back_rail.half_length",
        (Back, PinHeight) => "back_rail.pin_height",
        (Back, PunchingRadius) => "back_rail.punching_radius",
        (Back, PunchingHoleRadius) => "back_rail.punching_hole_radius",
        (Back, SharpRowOffset) => "back_rail.sharp_row_offset",
        (Back, CrownHalfLength) => "back_rail.crown.half_length",
        (Back, CrownHeight) => "back_rail.crown.height",
        (Back, PinDiameter) => "back_rail.pin.diameter",
        (Back, PinLength) => "back_rail.pin.length",
        (Back, PinThickness) => "back_rail.pin.thickness",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::ParamSpec;
    use crate::test_fixture::PAPER_THICKNESSES;

    /// The test key and frame, unweighed, on Stanwood's protocol and
    /// bushings of 6 g at the measuring point.
    #[derive(Clone)]
    struct Standard {
        key: KeyDesign,
        mass: KeyMass,
        frame: FrameDesign,
        protocol: TouchweightProtocol,
        friction: AxisFriction,
    }

    impl Standard {
        fn new() -> Self {
            let key = crate::test_fixture::key();
            let protocol = crate::test_fixture::protocol();
            let friction = AxisFriction::of_weight_at(6.0, protocol.measuring_point(&key));
            Self {
                key,
                mass: KeyMass::Unweighed,
                frame: crate::test_fixture::frame(),
                protocol,
                friction,
            }
        }

        fn design(&self) -> Design<'_> {
            Design {
                key: &self.key,
                mass: &self.mass,
                frame: &self.frame,
                protocol: &self.protocol,
                friction: &self.friction,
            }
        }

        fn validate(&self) -> Result<(), Problem> {
            validate(&self.design())
        }
    }

    /// `spec` set to a value no number of the test design holds, on the
    /// part `on` picks: the design's numbers show that value once, under
    /// the setting's own name, every other number as it was, and the
    /// setting reads it back. The bushings' setting drops the radius a
    /// weight was read at, a torque set as such having none.
    fn writes_the_field_it_names<T>(spec: &ParamSpec<T>, on: impl Fn(&mut Standard) -> &mut T) {
        let mut s = Standard::new();
        let before = numbers(&s.design());
        let mark = if spec.hard.1 <= 0.0 {
            -0.012_345_678_9
        } else {
            0.012_345_678_9
        };
        spec.apply(on(&mut s), mark)
            .unwrap_or_else(|p| panic!("{}: {p:?}", spec.name));
        let after = numbers(&s.design());
        let marked: Vec<_> = after
            .iter()
            .filter(|(_, x)| x.to_bits() == mark.to_bits())
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(marked, [spec.name], "{} writes {marked:?}", spec.name);
        let others = |v: &[(&'static str, f64)]| -> Vec<(&'static str, u64)> {
            v.iter()
                .filter(|(name, _)| *name != spec.name && *name != "friction.radius")
                .map(|(name, x)| (*name, x.to_bits()))
                .collect()
        };
        assert_eq!(others(&after), others(&before), "{} writes more", spec.name);
        assert_eq!(
            (spec.get)(on(&mut s)).to_bits(),
            mark.to_bits(),
            "{} reads another field",
            spec.name
        );
    }

    /// Consistency: **each setting reads and writes the field its name
    /// names, and no other**, the name a problem reports and the path an
    /// application composes from it: a setting wired to another field is
    /// caught however consistently its reader and its writer agree, since
    /// the design's numbers name every field by itself.
    #[test]
    fn each_setting_reads_and_writes_the_field_its_name_names() {
        for spec in &KeyDesign::PARAMS {
            writes_the_field_it_names(spec, |s| &mut s.key);
        }
        for spec in &FrameDesign::PARAMS {
            writes_the_field_it_names(spec, |s| &mut s.frame);
        }
        for spec in &TouchweightProtocol::PARAMS {
            writes_the_field_it_names(spec, |s| &mut s.protocol);
        }
        for spec in &AxisFriction::PARAMS {
            writes_the_field_it_names(spec, |s| &mut s.friction);
        }
    }

    /// Consistency: **a setting outside its range is refused however the
    /// design came to hold it**, a design loaded as well as an edit: a felt
    /// thinner than nothing, a capstan screwed down past its range, a key
    /// button a meter and a half high, the back rail in front of the balance
    /// pin, bushings that push.
    #[test]
    fn a_setting_outside_its_range_is_refused_on_load() {
        let refused = |s: &Standard, param: &str| {
            assert!(
                matches!(s.validate(), Err(Problem::OutOfRange { param: p, .. }) if p == param),
                "{param}: {:?}",
                s.validate()
            );
        };
        let mut s = Standard::new();
        s.frame.front_punching.felt = -0.000_1;
        refused(&s, "front_punching.felt");
        let mut s = Standard::new();
        s.frame.back_rail = 0.001;
        refused(&s, "back_rail.x");
        let mut s = Standard::new();
        s.key.capstan_screw.height = 0.031;
        refused(&s, "capstan.height");
        let mut s = Standard::new();
        s.key.capsule_height = 1.5;
        refused(&s, "capsule_height");
        let mut s = Standard::new();
        s.friction.bound_n_m = -0.001;
        refused(&s, "friction.bound");
    }

    /// Consistency: the test design is one parts can be built as.
    #[test]
    fn the_default_design_passes() {
        assert_eq!(Standard::new().validate(), Ok(()));
    }

    /// Consistency, the audit's RUST-6: **a key front brought back until its
    /// covering runs into the key button crosses its outline**, by how far
    /// the two overlap: at 170 mm the covering's back end stands
    /// 172 − 155 = 17 mm in front of the pin, inside a key button reaching
    /// 31 mm, so by 14 mm. A millimeter clear of the button, at 185 mm, it
    /// passes.
    #[test]
    fn a_key_front_pushed_into_the_key_button_crosses_the_outline() {
        let mut s = Standard::new();
        s.key.front_edge = 0.170;
        let Err(Problem::OutlineCrossesItself { back, front, by }) = s.validate() else {
            panic!("{:?}", s.validate())
        };
        assert_eq!((back, front), (PartCode::KeyButton, PartCode::Keytop));
        assert!((by - 0.014).abs() < 1e-12, "{by}");
        s.key.front_edge = 0.185;
        assert_eq!(s.validate(), Ok(()));
    }

    /// Consistency: **the capstan stands on the keystick between its two
    /// blocks**: 200 mm behind the pin its head stands 9.5 mm into the back
    /// check block, 20 mm behind it into the key button by 13.5 mm, and
    /// 230 mm behind it, on the thinned tail, it is behind the back check
    /// block, 39.5 mm short of clearing it.
    #[test]
    fn a_capstan_in_a_block_or_behind_the_back_check_block_is_refused() {
        let mut s = Standard::new();
        for (x, problem) in [
            (
                -0.200,
                Problem::CapstanInBlock {
                    block: PartCode::BackCheckBlock,
                    by: 0.0095,
                },
            ),
            (
                -0.020,
                Problem::CapstanInBlock {
                    block: PartCode::KeyButton,
                    by: 0.0135,
                },
            ),
            (-0.230, Problem::CapstanBehindBackCheckBlock { by: 0.0395 }),
        ] {
            s.key.capstan = x;
            match (s.validate(), problem) {
                (
                    Err(Problem::CapstanInBlock { block, by }),
                    Problem::CapstanInBlock {
                        block: want,
                        by: want_by,
                    },
                ) => {
                    assert_eq!(block, want, "{x}");
                    assert!((by - want_by).abs() < 1e-12, "{x}: {by}");
                }
                (
                    Err(Problem::CapstanBehindBackCheckBlock { by }),
                    Problem::CapstanBehindBackCheckBlock { by: want_by },
                ) => {
                    assert!((by - want_by).abs() < 1e-12, "{x}: {by}");
                }
                (got, _) => panic!("{x}: {got:?}"),
            }
        }
    }

    /// What a few sums and differences of lengths under 300 mm round to:
    /// each operation within half an ulp of 300 mm, and none of these
    /// probes takes more than eight.
    const ROUNDING: f64 = 8.0 * f64::EPSILON * 0.300;

    /// Consistency: **the capstan's limit behind the pin is the back check
    /// block's, not the key's end.** On the test key and on keys
    /// lengthened at the back by 30 and 60 mm, the block stays from 220 to
    /// 193 mm behind the pin, and so does the limit: a micrometer either
    /// side of the capstan's head clearing the block's front, it stands
    /// clear or a micrometer into the block; a micrometer either side of its
    /// head touching the block's back, a micrometer into it or behind it.
    /// Behind the block it is refused as behind it wherever it stands, each
    /// by how far it must come forward to clear the block: on the
    /// keystick's full height between the block and the lengthened tail
    /// (235 mm behind the pin), on the thinned tail (260 mm), past the key's
    /// end (285 mm).
    #[test]
    fn the_capstans_limit_behind_the_pin_follows_the_block_not_the_keys_end() {
        let micrometer = 1e-6;
        for longer in [0.0, 0.030, 0.060] {
            let mut s = Standard::new();
            s.key.tail_end -= longer;
            assert_eq!(s.validate(), Ok(()), "{longer}");
            let r = capstan_radius(&s.key);
            let block_back = s.key.back_check_block_back;
            let block_front = block_back + s.key.back_check_block_length;
            assert!((block_front + 0.193).abs() <= ROUNDING, "{block_front}");
            let into_block = |s: &Standard, want: f64| match s.validate() {
                Err(Problem::CapstanInBlock {
                    block: PartCode::BackCheckBlock,
                    by,
                }) => assert!((by - want).abs() <= ROUNDING, "{longer}: {by}"),
                got => panic!("{longer}, {}: {got:?}", s.key.capstan),
            };

            s.key.capstan = block_front + r + micrometer;
            assert_eq!(s.validate(), Ok(()), "{longer}");
            s.key.capstan = block_front + r - micrometer;
            into_block(&s, micrometer);

            s.key.capstan = block_back - r + micrometer;
            into_block(&s, block_front - block_back + 2.0 * r - micrometer);
            s.key.capstan = block_back - r - micrometer;
            match s.validate() {
                Err(Problem::CapstanBehindBackCheckBlock { by }) => {
                    let want = block_front - block_back + 2.0 * r + micrometer;
                    assert!((by - want).abs() <= ROUNDING, "{longer}: {by}");
                }
                got => panic!("{longer}: {got:?}"),
            }
        }

        let mut s = Standard::new();
        s.key.tail_end = -0.270;
        let tail_front = s.key.tail_end + s.key.tail_length;
        let r = capstan_radius(&s.key);
        for (x, on) in [
            (-0.235, "the full height behind the block"),
            (-0.260, "the thinned tail"),
            (-0.285, "past the key's end"),
        ] {
            s.key.capstan = x;
            let where_it_stands = if x - r > tail_front {
                "the full height behind the block"
            } else if x + r < tail_front && x - r > s.key.tail_end {
                "the thinned tail"
            } else if x + r < s.key.tail_end {
                "past the key's end"
            } else {
                "astride a step"
            };
            assert_eq!(where_it_stands, on, "{x}");
            match s.validate() {
                Err(Problem::CapstanBehindBackCheckBlock { by }) => {
                    assert!((by - (-0.193 - (x - r))).abs() <= ROUNDING, "{x}: {by}");
                }
                got => panic!("{x}, on {on}: {got:?}"),
            }
        }
    }

    /// Consistency: **the thinned tail ends at or behind the back check
    /// block**, which is glued on the keystick's full height: on the test
    /// key it begins right behind the block, and passes; a micrometer
    /// longer, it runs that far under the block and the outline crosses
    /// itself. On a key lengthened at the back by 30 mm, the tail may grow
    /// by the 30 mm of full-height wood and no more; and a key shortened at
    /// the back by 10 mm, its block left where the action puts it, runs its
    /// tail 10 mm under the block. Shortened by 40 mm, more than its tail's
    /// 20 mm, the block hangs 20 mm off the key's end and the two overlap by
    /// 7 mm, but the tail's front stands 40 mm in front of the block's back
    /// end, and that is what is said: how far one must move for the key to
    /// be cut.
    #[test]
    fn a_tail_run_under_the_back_check_block_crosses_the_outline() {
        let micrometer = 1e-6;
        let under = |s: &Standard, want: f64| match s.validate() {
            Err(Problem::OutlineCrossesItself {
                back: PartCode::KeyTailFelt,
                front: PartCode::BackCheckBlock,
                by,
            }) => assert!((by - want).abs() <= ROUNDING, "{by}"),
            got => panic!("{got:?}"),
        };
        let mut s = Standard::new();
        assert_eq!(
            (s.key.tail_end + s.key.tail_length).to_bits(),
            s.key.back_check_block_back.to_bits(),
            "the test key's tail begins right behind the block"
        );
        assert_eq!(s.validate(), Ok(()));
        s.key.tail_length += micrometer;
        under(&s, micrometer);

        for (tail_length, refused) in [(0.050 - micrometer, false), (0.050 + micrometer, true)] {
            let mut s = Standard::new();
            s.key.tail_end = -0.270;
            s.key.tail_length = tail_length;
            if refused {
                under(&s, micrometer);
            } else {
                assert_eq!(s.validate(), Ok(()), "{tail_length}");
            }
        }

        for (tail_end, by) in [(-0.230, 0.010), (-0.200, 0.040)] {
            let mut s = Standard::new();
            s.key.tail_end = tail_end;
            under(&s, by);
        }
    }

    /// Consistency: **a back check block pushed forward runs into the key
    /// button**, by how far it overlaps it: a micrometer into it, the outline
    /// crosses itself by that micrometer; a micrometer short of it, the
    /// outline is whole, and the capstan, 122 mm behind the pin, is behind
    /// the block.
    #[test]
    fn a_back_check_block_pushed_into_the_key_button_crosses_the_outline() {
        let micrometer = 1e-6;
        let mut s = Standard::new();
        let button_back = -s.key.capsule_length * 0.5;
        s.key.back_check_block_back = button_back - s.key.back_check_block_length + micrometer;
        match s.validate() {
            Err(Problem::OutlineCrossesItself {
                back: PartCode::BackCheckBlock,
                front: PartCode::KeyButton,
                by,
            }) => assert!((by - micrometer).abs() <= ROUNDING, "{by}"),
            got => panic!("{got:?}"),
        }
        s.key.back_check_block_back -= 2.0 * micrometer;
        assert!(
            matches!(
                s.validate(),
                Err(Problem::CapstanBehindBackCheckBlock { .. })
            ),
            "{:?}",
            s.validate()
        );
    }

    /// Consistency: **a front mortise cut through the key top, or off the
    /// flat of the underside, is refused.**
    #[test]
    fn a_mortise_through_the_key_top_or_off_the_underside_is_refused() {
        let mut s = Standard::new();
        s.key.front_mortise_depth = s.key.stick_height;
        assert_eq!(
            s.validate(),
            Err(Problem::MortiseThroughKeyTop {
                depth: s.key.stick_height,
                wood: s.key.stick_height,
            })
        );
        let mut s = Standard::new();
        s.key.front_mortise_from_front_edge = 0.0;
        assert!(matches!(
            s.validate(),
            Err(Problem::FeatureOutsideKeystick {
                part: PartCode::FrontMortise,
                ..
            })
        ));
    }

    /// Consistency: **the key button's rounded ends meet at most**: on the
    /// test key, 6 mm high, ends of 12 mm reach in √(2·12·6 − 6²) = √108 mm
    /// each from their ends, 20.78 mm together. A key button a micrometer
    /// longer has its ends meet a micrometer short of each other, its ends
    /// taken; one a micrometer shorter is refused for its ends, by that
    /// micrometer.
    #[test]
    fn a_key_button_whose_rounded_ends_overlap_is_refused() {
        const MICROMETER: f64 = 1e-6;
        let both_ends = 2.0 * 0.001 * 108.0_f64.sqrt();
        let mut s = Standard::new();
        s.key.capsule_length = both_ends + MICROMETER;
        assert!(
            !matches!(s.validate(), Err(Problem::CapsuleEndsOverlap { .. })),
            "{:?}",
            s.validate()
        );
        s.key.capsule_length = both_ends - MICROMETER;
        match s.validate() {
            Err(Problem::CapsuleEndsOverlap { by }) => {
                // Two square roots and a subtraction of lengths under a
                // decimeter, each rounded once.
                assert!((by - MICROMETER).abs() <= 4.0 * f64::EPSILON * 0.1, "{by}");
            }
            other => panic!("{other:?}"),
        }
    }

    /// Consistency: **each part of the balance mortise is refused where it
    /// does not fit**, naming the part, what it does not fit in and by how
    /// much, and taken a micrometer short of its limit: the slot past the
    /// key button's flat top either side, the flat the 62 mm of the key
    /// button less its two ends of 12 mm reach on its 6 mm of height,
    /// √108 mm each; the cavity's floor above the keystick's top, the slot
    /// wider than the key, the hole wider than the slot, the bushing's two
    /// thicknesses wider than the slot, the cloth thicker than the side hole
    /// it turns out through, the bushing deeper than the key button's 6 mm,
    /// the side hole's bottom below the key button's, 4.7 mm down for a
    /// 2.6 mm hole under 1.3 mm of cloth, and the bushing less deep than its
    /// 1.3 mm thickness, which would raise the side hole's top past the key
    /// button's.
    #[test]
    fn each_part_of_the_balance_mortise_is_refused_where_it_does_not_fit() {
        const MICROMETER: f64 = 1e-6;
        // Each shortfall is a difference of lengths under a decimeter, the
        // flat's through two square roots, each rounded a few times.
        let close = |by: f64| (by - MICROMETER).abs() <= 8.0 * f64::EPSILON * 0.1;
        // Puts a part of the mortise a length past its limit, or short of it.
        type Push = fn(&mut Standard, f64);
        let checks: [(Push, PartCode, PartCode); 9] = [
            (
                |s, over| {
                    let flat = 0.062 - 2.0 * 0.001 * 108.0_f64.sqrt();
                    s.key.balance_mortise.length = flat + 2.0 * over;
                },
                PartCode::BalanceMortise,
                PartCode::KeyButton,
            ),
            (
                |s, over| s.key.balance_mortise.floor = s.key.stick_height + over,
                PartCode::BalanceMortise,
                PartCode::Keystick,
            ),
            (
                |s, over| s.key.balance_mortise.width = s.key.width + over,
                PartCode::BalanceMortise,
                PartCode::Keystick,
            ),
            (
                |s, over| {
                    s.key.balance_mortise.hole_diameter = s.key.balance_mortise.width + over;
                },
                PartCode::BalanceHole,
                PartCode::BalanceMortise,
            ),
            (
                // The side hole as tall as the thicker cloth, so that the
                // cloth a micrometer short of filling the slot still passes
                // through it.
                |s, over| {
                    let thickness = 0.5 * (s.key.balance_mortise.width + over);
                    s.key.balance_mortise.bushing_thickness = thickness;
                    s.key.balance_mortise.side_hole_height = thickness;
                },
                PartCode::BalanceMortiseBushing,
                PartCode::BalanceMortise,
            ),
            (
                |s, over| {
                    s.key.balance_mortise.side_hole_height =
                        s.key.balance_mortise.bushing_thickness - over;
                },
                PartCode::BalanceMortiseBushing,
                PartCode::KeyButtonSideHole,
            ),
            (
                // A side hole as tall as the cloth, so that it reaches no
                // lower than the bushing.
                |s, over| {
                    s.key.balance_mortise.side_hole_height =
                        s.key.balance_mortise.bushing_thickness;
                    s.key.balance_mortise.bushing_depth = s.key.capsule_height + over;
                },
                PartCode::BalanceMortiseBushing,
                PartCode::KeyButton,
            ),
            (
                |s, over| {
                    let mortise = s.key.balance_mortise;
                    s.key.balance_mortise.bushing_depth =
                        s.key.capsule_height + mortise.bushing_thickness - mortise.side_hole_height
                            + over;
                },
                PartCode::KeyButtonSideHole,
                PartCode::KeyButton,
            ),
            (
                |s, over| {
                    s.key.balance_mortise.bushing_depth =
                        s.key.balance_mortise.bushing_thickness - over;
                },
                PartCode::KeyButtonSideHole,
                PartCode::KeyButton,
            ),
        ];
        for (push, part, within) in checks {
            let mut short = Standard::new();
            push(&mut short, -MICROMETER);
            assert_eq!(short.validate(), Ok(()), "{part:?} a micrometer short");
            let mut past = Standard::new();
            push(&mut past, MICROMETER);
            match past.validate() {
                Err(Problem::DoesNotFit {
                    part: refused,
                    within: limit,
                    by,
                }) if (refused, limit) == (part, within) && close(by) => {}
                other => panic!("{part:?} in {within:?}: {other:?}"),
            }
        }
    }

    /// Consistency: **a side hole flush with the key button's top is
    /// taken**: the bushing as deep as its 1.3 mm cloth is thick stands the
    /// side hole's top on the key button's, a groove open at the top, and
    /// every part the key is built of closes.
    #[test]
    fn a_side_hole_flush_with_the_key_buttons_top_is_taken() {
        let mut s = Standard::new();
        s.key.balance_mortise.bushing_depth = s.key.balance_mortise.bushing_thickness;
        assert_eq!(s.validate(), Ok(()));
        let mut parts = Vec::new();
        s.key.parts(crate::mechanism::BodyId(1), &mut parts);
        assert!(parts.iter().all(crate::contact::Part::is_closed));
    }

    /// Consistency: **the measuring point stands on the covering**, from its
    /// lip to its back end.
    #[test]
    fn a_measuring_point_off_the_covering_is_refused() {
        let mut s = Standard::new();
        for from_lip in [-0.001, s.key.keytop_length + 0.001] {
            s.protocol.measuring_point_from_lip = from_lip;
            assert_eq!(
                s.validate(),
                Err(Problem::MeasuringPointOffCovering {
                    from_lip,
                    covering: s.key.keytop_length,
                })
            );
        }
        s.protocol.measuring_point_from_lip = s.key.keytop_length;
        assert_eq!(s.validate(), Ok(()));
    }

    /// Consistency, the audit's RUST-17: **no number that is not finite
    /// passes**, in a setting or in any other dimension of the key or the
    /// frame; and **a mass of nothing is refused**, the audit's BWA-3.
    #[test]
    fn no_number_that_is_not_finite_passes_and_no_mass_of_nothing() {
        let mut s = Standard::new();
        s.key.stick_height = f64::NAN;
        assert_eq!(
            s.validate(),
            Err(Problem::NotFinite {
                param: "stick_height"
            })
        );
        let mut s = Standard::new();
        s.frame.front_punching.papers.push(f64::INFINITY);
        assert_eq!(s.validate(), Err(Problem::NotFinite { param: PAPER.name }));
        let mut s = Standard::new();
        s.frame.front_punching.papers.push(0.0003);
        assert_eq!(
            s.validate(),
            Err(Problem::NotInTheDrawer {
                param: PAPER.name,
                thickness: 0.0003,
            })
        );
        s.frame.front_punching.papers.pop();
        s.frame.front_punching.papers.push(PAPER_THICKNESSES[2]);
        assert_eq!(s.validate(), Ok(()));
        let mut s = Standard::new();
        s.mass = KeyMass::Weighed {
            mass: 0.0,
            static_moment_n_m: 0.0,
            datum: crate::parts::FrontWeightReading {
                grams: 0.0,
                from_lip_m: 0.013,
                arm_m: 0.247,
            },
        };
        assert_eq!(s.validate(), Err(Problem::MassNotPositive { mass: 0.0 }));
        let mut s = Standard::new();
        s.mass = KeyMass::Published {
            mass: -0.1,
            center: crate::geometry::Point::new(0.05, 0.015),
            inertia: 0.002,
        };
        assert_eq!(s.validate(), Err(Problem::MassNotPositive { mass: -0.1 }));
    }

    /// The test design with the key that carries every feature the
    /// test key lacks ([`crate::test_fixture::key_with_every_feature`]).
    fn featured() -> Standard {
        Standard {
            key: crate::test_fixture::key_with_every_feature(),
            ..Standard::new()
        }
    }

    /// Consistency: **a key with every feature the test key lacks passes**,
    /// and its design's numbers name each feature, and name no felt on its
    /// bare tail.
    #[test]
    fn a_key_with_every_feature_passes_and_names_them() {
        let s = featured();
        assert_eq!(s.validate(), Ok(()));
        let names: Vec<&str> = numbers(&s.design()).iter().map(|(n, _)| *n).collect();
        for name in [
            "capsule_center",
            "top_clearance.back",
            "top_clearance.front",
            "top_clearance.depth",
            "capstan.dome_radius",
        ] {
            assert!(names.contains(&name), "{name}");
        }
        assert!(!names.contains(&"damper_felt"));
        let plain: Vec<&str> = numbers(&Standard::new().design())
            .iter()
            .map(|(n, _)| *n)
            .collect();
        for name in [
            "capsule_center",
            "top_clearance.back",
            "capstan.dome_radius",
        ] {
            assert!(!plain.contains(&name), "{name} on the test key");
        }
    }

    /// Consistency: **a capstan domed narrower than its head is refused**: a
    /// dome a micrometer narrower than the head's radius caps nothing; one
    /// of the head's radius, a hemisphere, caps it.
    #[test]
    fn a_capstan_domed_narrower_than_its_head_is_refused() {
        let micrometer = 1e-6;
        let mut s = featured();
        let head = 0.5 * s.key.capstan_screw.head_diameter;
        s.key.capstan_screw.dome_radius = Some(head - micrometer);
        assert_eq!(
            s.validate(),
            Err(Problem::CapstanDomeNarrowerThanItsHead {
                dome: head - micrometer,
                head
            })
        );
        s.key.capstan_screw.dome_radius = Some(head);
        assert_eq!(s.validate(), Ok(()), "a hemisphere caps the head");
    }

    /// Consistency: **a glued block stands wholly on a bare thinned tail or
    /// wholly in front of it**: the tail stopped a millimeter short of the
    /// back check block's front end puts the block across the step, by that
    /// millimeter; the same tail felted runs its felt under the back check
    /// block, by how far the tail's front stands in front of the block's
    /// back end.
    #[test]
    fn a_block_across_the_step_to_a_bare_tail_crosses_the_outline() {
        let mut s = featured();
        let block = s.key.back_check_block();
        s.key.tail_length = block.front() - 0.001 - s.key.tail_end;
        match s.validate() {
            Err(Problem::OutlineCrossesItself {
                back: PartCode::BackCheckBlock,
                front: PartCode::Keystick,
                by,
            }) => assert!((by - 0.001).abs() <= ROUNDING, "{by}"),
            got => panic!("{got:?}"),
        }
        let mut s = featured();
        s.key.damper_felt = Some(0.005);
        match s.validate() {
            Err(Problem::OutlineCrossesItself {
                back: PartCode::KeyTailFelt,
                front: PartCode::BackCheckBlock,
                by,
            }) => {
                let want = s.key.tail_front() - s.key.back_check_block_back;
                assert!((by - want).abs() <= ROUNDING, "{by}");
            }
            got => panic!("{got:?}"),
        }
    }

    /// Consistency: **the capstan screws into the keystick's full height.**
    /// On the key with every feature, its bare thinned tail run on under its
    /// back check block to 190 mm behind the pin, its clearance taken away:
    /// a micrometer either side of the back of the capstan's head reaching
    /// the step down to the tail, it stands clear over the full height, or
    /// off it by that micrometer; over the tail, between the step and the
    /// back check block, it is off the full height by how far its head's
    /// back stands behind the step; and its head reaching the block's
    /// front, it stands in the block, the block's limit coming first.
    #[test]
    fn a_capstan_on_the_thinned_tail_is_refused() {
        let micrometer = 1e-6;
        let mut s = featured();
        s.key.top_clearance = None;
        let r = capstan_radius(&s.key);
        let step = s.key.tail_front();
        let block_front = s.key.back_check_block().front();
        assert!(block_front < step, "the tail runs on in front of the block");
        let off = |s: &Standard, want: f64| match s.validate() {
            Err(Problem::CapstanOffTheFullHeight { by }) => {
                assert!((by - want).abs() <= ROUNDING, "{}: {by}", s.key.capstan);
            }
            got => panic!("{}: {got:?}", s.key.capstan),
        };
        s.key.capstan = step + r + micrometer;
        assert_eq!(s.validate(), Ok(()));
        s.key.capstan = step + r - micrometer;
        off(&s, micrometer);
        s.key.capstan = 0.5 * (step + block_front) + r;
        off(&s, 0.5 * (step - block_front));
        s.key.capstan = block_front + r + micrometer;
        off(&s, step - block_front - micrometer);
        s.key.capstan = block_front + r - micrometer;
        match s.validate() {
            Err(Problem::CapstanInBlock {
                block: PartCode::BackCheckBlock,
                by,
            }) => assert!((by - micrometer).abs() <= ROUNDING, "{by}"),
            got => panic!("{got:?}"),
        }
    }

    /// The test design with a clearance for the wippen rail cut 3 mm deep in
    /// its key's top, from 180 to 140 mm behind the balance pin, between
    /// its back check block, which stands on the keystick's full height and
    /// ends 193 mm behind it, and its capstan, whose head's back stands
    /// 124.5 mm behind it.
    fn cleared() -> Standard {
        let mut s = Standard::new();
        s.key.top_clearance = Some(crate::parts::TopClearance {
            back: -0.180,
            front: -0.140,
            depth: 0.003,
        });
        s
    }

    /// Consistency: **a clearance runs into no part, stands behind the
    /// capstan's head on the keystick's full height, and leaves wood under
    /// it**, each refused by how far past its limit it goes, a micrometer
    /// either side of each limit: its back end on the back check block's
    /// front; its front end on the back of the capstan's head; its front
    /// end on the key button's back, where, the capstan standing between
    /// them, short of the button it runs under the capstan instead; its
    /// back end on the step down to a bare thinned tail run on under the
    /// back check block, or exactly on it; its depth on the wood under it,
    /// the keystick's full height over the flat of the underside, less the
    /// chamfer's rise under its back end over the chamfer, where a cut short
    /// of the full height alone goes through. And a clearance of no length or
    /// no depth, or turned back to front, is no cut.
    #[test]
    fn a_clearance_is_refused_past_each_of_its_limits() {
        let micrometer = 1e-6;
        let crosses = |s: &Standard, back: PartCode, front: PartCode, want: f64| match s.validate()
        {
            Err(Problem::OutlineCrossesItself {
                back: b,
                front: f,
                by,
            }) if (b, f) == (back, front) => {
                assert!((by - want).abs() <= ROUNDING, "{back:?}, {front:?}: {by}");
            }
            got => panic!("{back:?}, {front:?}: {got:?}"),
        };
        let with = |s: &Standard, back: f64, front: f64, depth: f64| {
            let mut s = s.clone();
            s.key.top_clearance = Some(crate::parts::TopClearance { back, front, depth });
            s
        };
        let s = cleared();
        assert_eq!(s.validate(), Ok(()));
        let c = s.key.top_clearance.expect("a clearance");

        // Back onto the back check block, standing on the full height.
        let check_front = s.key.back_check_block().front();
        assert!(check_front > s.key.tail_front());
        assert_eq!(
            with(&s, check_front + micrometer, c.front, c.depth).validate(),
            Ok(())
        );
        crosses(
            &with(&s, check_front - micrometer, c.front, c.depth),
            PartCode::BackCheckBlock,
            PartCode::TopClearance,
            micrometer,
        );

        // Forward under the capstan's head.
        let head_back = s.key.capstan - capstan_radius(&s.key);
        assert_eq!(
            with(&s, c.back, head_back - micrometer, c.depth).validate(),
            Ok(())
        );
        crosses(
            &with(&s, c.back, head_back + micrometer, c.depth),
            PartCode::TopClearance,
            PartCode::Capstan,
            micrometer,
        );

        // Forward into the key button, past the capstan.
        let (button_back, _) = s.key.key_button_span();
        crosses(
            &with(&s, c.back, button_back + micrometer, c.depth),
            PartCode::TopClearance,
            PartCode::KeyButton,
            micrometer,
        );
        crosses(
            &with(&s, c.back, button_back - micrometer, c.depth),
            PartCode::TopClearance,
            PartCode::Capstan,
            button_back - micrometer - head_back,
        );

        // Back past the full height, onto a bare tail.
        let bare = featured();
        let c = bare.key.top_clearance.expect("a clearance");
        let step = bare.key.tail_front();
        for back in [step + micrometer, step] {
            assert_eq!(
                with(&bare, back, c.front, c.depth).validate(),
                Ok(()),
                "{back}"
            );
        }
        assert_eq!(
            with(&bare, step - micrometer, c.front, c.depth).validate(),
            Err(Problem::FeatureOutsideKeystick {
                part: PartCode::TopClearance,
                by: step - (step - micrometer),
            })
        );

        // Down through the wood: over the flat of the underside, the
        // keystick's full height; over the chamfer, which the test key's
        // runs on 30 mm in front of its back check block, to 163 mm behind
        // the pin, the full height less the chamfer's rise under the
        // clearance's back end, 2.5 mm over 77 mm, by hand.
        let s = cleared();
        let c = s.key.top_clearance.expect("a clearance");
        let flat_from = s.key.tail_end + s.key.tail_rise_run;
        assert!((flat_from + 0.163).abs() <= ROUNDING, "{flat_from}");
        let wood = s.key.stick_height;
        assert_eq!(
            with(&s, flat_from + 0.003, c.front, wood - micrometer).validate(),
            Ok(())
        );
        assert_eq!(
            with(&s, flat_from + 0.003, c.front, wood).validate(),
            Err(Problem::ClearanceThroughKeystick { depth: wood, wood })
        );
        assert!(c.back < flat_from, "the clearance stands over the chamfer");
        let rise = 0.0025 * (-0.163 - c.back) / 0.077;
        let wood = s.key.stick_height - rise;
        assert_eq!(
            with(&s, c.back, c.front, wood - micrometer).validate(),
            Ok(())
        );
        for depth in [wood + micrometer, s.key.stick_height - micrometer] {
            match with(&s, c.back, c.front, depth).validate() {
                Err(Problem::ClearanceThroughKeystick { depth: d, wood: w }) => {
                    assert_eq!(d.to_bits(), depth.to_bits());
                    assert!((w - wood).abs() <= ROUNDING, "{w}, by hand {wood}");
                }
                got => panic!("{depth}: {got:?}"),
            }
        }

        // No cut.
        for (back, front, depth) in [
            (c.back, c.back, c.depth),
            (c.front, c.back, c.depth),
            (c.back, c.front, 0.0),
            (c.back, c.front, -micrometer),
        ] {
            assert_eq!(
                with(&s, back, front, depth).validate(),
                Err(Problem::ClearanceNotACut {
                    length: front - back,
                    depth
                }),
                "{back}, {front}, {depth}"
            );
        }
    }
}

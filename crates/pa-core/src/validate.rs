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
//! capstan in a glued block or behind the back check block, its front
//! mortise cut through the top of the key, its measuring point off its
//! covering. Everything an assembled keyboard can be, however badly
//! regulated, is **computed and reported**: a frame placed for another key,
//! stops that fight, a pin standing proud of its mortise, a key regulated
//! into jamming are states, which the faults, the verdicts and the sheet's
//! refusals describe, and are never refused here. What is computed is
//! **drawn as it is**.
//!
//! [`validate`] runs on every edit and every load, before any solve, so the
//! state an instrument holds always passes it: no number that is not finite
//! reaches the statics, and no outline the volume model integrates crosses
//! itself.

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
        /// The thickness asked for, metres.
        thickness: f64,
    },
    /// A key weighed at no mass, or less.
    MassNotPositive {
        /// The mass typed, kilograms.
        mass: f64,
    },
    /// The key's outline crosses itself: two parts glued on the keystick
    /// overlapping along its top, the thinned tail's felt run on under a
    /// glued block, a block or the covering standing across the step down
    /// to the thinned tail, or the underside's chamfer running past the key
    /// front.
    OutlineCrossesItself {
        /// The part behind, whose front end stands too far forward.
        back: PartCode,
        /// The part in front, whose back end stands too far back.
        front: PartCode,
        /// How far the front end of the part behind stands in front of the
        /// back end of the part in front, metres along the key: how far the
        /// two overlap, where neither runs past the other's far end.
        by: f64,
    },
    /// A feature that stands off the keystick's wood where it must be cut:
    /// the front mortise off the flat of the underside.
    FeatureOutsideKeystick {
        /// The feature.
        part: PartCode,
        /// How far off, metres along the key.
        by: f64,
    },
    /// The capstan stands wholly behind the back check block: on the
    /// keystick's full height between the block and the thinned tail, on the
    /// thinned tail, or past the key's end. It stands between the key button
    /// and the back check block.
    CapstanBehindBackCheckBlock {
        /// How far forward it must come for its head to clear the block,
        /// metres along the key.
        by: f64,
    },
    /// The capstan stands off the block it is screwed into: its head runs
    /// past one end of the block.
    CapstanOffItsBlock {
        /// How far past the block's end its head reaches, metres along the
        /// key.
        by: f64,
    },
    /// The capstan's dome is narrower than its head: a sphere of that radius
    /// cannot cap the head.
    CapstanDomeNarrowerThanItsHead {
        /// The dome's radius, metres.
        dome: f64,
        /// The head's radius, metres.
        head: f64,
    },
    /// The capstan stands in a block glued on the keystick: the key button
    /// or the back check block.
    CapstanInBlock {
        /// The block.
        block: PartCode,
        /// How far the capstan's head stands into it, metres along the key.
        by: f64,
    },
    /// The front mortise is cut through the top of the keystick.
    MortiseThroughKeyTop {
        /// Its depth above the underside, metres.
        depth: f64,
        /// The keystick's wood it is cut into, metres.
        wood: f64,
    },
    /// The measuring point stands off the covering: in front of its lip, or
    /// behind its back end.
    MeasuringPointOffCovering {
        /// Where it was asked for, metres back from the lip.
        from_lip: f64,
        /// How far the covering runs back from its lip, metres.
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
/// key's outline, the capstan between its blocks, the front mortise on the
/// flat of the underside and under the key's top, and the measuring point
/// on the covering.
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
    let covering = d.key.keytop_length;
    let from_lip = d.protocol.measuring_point_from_lip;
    if !(0.0..=covering).contains(&from_lip) {
        return Err(Problem::MeasuringPointOffCovering { from_lip, covering });
    }
    Ok(())
}

/// The key's outline along its top and its underside, back to front. Each
/// block glued on it, the back check block, the capstan's block where it has
/// one and the key button, stands wholly on the thinned tail or wholly in
/// front of it, and the covering in front of it: a felted tail runs on under
/// none of them, and none stands across the step down to a bare one. Each
/// glued part ends before the next begins; and the chamfer under the tail
/// runs out before the key front. Two that overlap are the outline crossing
/// itself, by how far they overlap.
fn outline(key: &KeyDesign) -> Result<(), Problem> {
    let tail_front = key.tail_front();
    let check = key.back_check_block();
    let (button_back, button_front) = key.key_button_span();
    let keytop_back = key.lip() - key.keytop_length;
    let wood_front = key.front_edge - key.front_board;
    let chamfer_end = key.tail_end + key.tail_rise_run;
    let mut glued = vec![(PartCode::BackCheckBlock, check.back, check.front())];
    if let Some(b) = key.capstan_block {
        glued.push((PartCode::CapstanBlock, b.back, b.front()));
    }
    glued.extend([
        (PartCode::KeyButton, button_back, button_front),
        (PartCode::Keytop, keytop_back, key.lip()),
    ]);
    for &(part, back, front) in &glued {
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

/// The capstan's widest radius, metres: its head or its shaft.
fn capstan_radius(key: &KeyDesign) -> f64 {
    0.5 * key
        .capstan_screw
        .head_diameter
        .max(key.capstan_screw.diameter)
}

/// The capstan stands between the back check block and the key button,
/// where nothing is glued on top but its own block: its head in either block
/// is the capstan in that block, by how far it stands in; the whole of it
/// behind the back check block, wherever the key's wood runs there, is
/// behind the block, by how far it must come forward to clear it. The limit
/// is the block's, which the action places, and not the key's end. Where it
/// stands on a block of its own, it stands wholly on it; where its head is
/// a dome, the dome caps the head.
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
    if let Some(b) = key.capstan_block {
        let past = (b.back - back).max(front - b.front());
        if past > 0.0 {
            return Err(Problem::CapstanOffItsBlock { by: past });
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
        capsule_centre,
        back_check_block_height,
        back_check_block_back,
        back_check_block_length,
        capstan_block,
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
        density,
        capstan_screw,
        measured_stick_mass,
    } = d.key;
    let crate::parts::Capstan {
        height,
        diameter,
        head_diameter,
        head_height,
        cove_height,
        shaft_length,
        tip_height,
        density: capstan_density,
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
    if let Some(c) = capsule_centre {
        out.push(("capsule_centre", *c));
    }
    out.extend([
        ("back_check_block_height", *back_check_block_height),
        ("back_check_block_back", *back_check_block_back),
        ("back_check_block_length", *back_check_block_length),
    ]);
    if let Some(crate::parts::GluedBlock {
        back,
        length,
        height,
    }) = capstan_block
    {
        out.extend([
            ("capstan_block.back", *back),
            ("capstan_block.length", *length),
            ("capstan_block.height", *height),
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
        ("density", *density),
        ("capstan.height", *height),
        ("capstan.diameter", *diameter),
        ("capstan.head_diameter", *head_diameter),
        ("capstan.head_height", *head_height),
        ("capstan.cove_height", *cove_height),
        ("capstan.shaft_length", *shaft_length),
        ("capstan.tip_height", *tip_height),
        ("capstan.density", *capstan_density),
    ]);
    if let Some(r) = dome_radius {
        out.push(("capstan.dome_radius", *r));
    }
    if let Some(m) = measured_stick_mass {
        out.push(("measured_stick_mass", *m));
    }
    match *d.mass {
        KeyMass::Estimated => {}
        KeyMass::Published {
            mass,
            centre,
            inertia,
        } => out.extend([
            ("published.mass", mass),
            ("published.centre.x", centre.x),
            ("published.centre.y", centre.y),
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

    /// The composite key and frame, unweighed, on Stanwood's protocol and
    /// bushings of 6 g at the measuring point.
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
                mass: KeyMass::Estimated,
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

    /// `spec` set to a value no number of the composite design holds, on the
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
    /// thinner than nothing, a capstan screwed down past its range, the back
    /// rail in front of the balance pin, bushings that push.
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
        s.friction.bound_n_m = -0.001;
        refused(&s, "friction.bound");
    }

    /// Consistency: the composite design is one parts can be built as.
    #[test]
    fn the_default_design_passes() {
        assert_eq!(Standard::new().validate(), Ok(()));
    }

    /// Consistency, the audit's RUST-6: **a key front brought back until its
    /// covering runs into the key button crosses its outline**, by how far
    /// the two overlap: at 170 mm the covering's back end stands
    /// 172 − 155 = 17 mm in front of the pin, inside a key button reaching
    /// 31 mm, so by 14 mm. A millimetre clear of the button, at 185 mm, it
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
    /// block's, not the key's end.** On the composite key and on keys
    /// lengthened at the back by 30 and 60 mm, the block stays from 220 to
    /// 193 mm behind the pin, and so does the limit: a micrometre either
    /// side of the capstan's head clearing the block's front, it stands
    /// clear or a micrometre into the block; a micrometre either side of its
    /// head touching the block's back, a micrometre into it or behind it.
    /// Behind the block it is refused as behind it wherever it stands, each
    /// by how far it must come forward to clear the block: on the
    /// keystick's full height between the block and the lengthened tail
    /// (235 mm behind the pin), on the thinned tail (260 mm), past the key's
    /// end (285 mm).
    #[test]
    fn the_capstans_limit_behind_the_pin_follows_the_block_not_the_keys_end() {
        let micrometre = 1e-6;
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

            s.key.capstan = block_front + r + micrometre;
            assert_eq!(s.validate(), Ok(()), "{longer}");
            s.key.capstan = block_front + r - micrometre;
            into_block(&s, micrometre);

            s.key.capstan = block_back - r + micrometre;
            into_block(&s, block_front - block_back + 2.0 * r - micrometre);
            s.key.capstan = block_back - r - micrometre;
            match s.validate() {
                Err(Problem::CapstanBehindBackCheckBlock { by }) => {
                    let want = block_front - block_back + 2.0 * r + micrometre;
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
    /// block**, which is glued on the keystick's full height: on the default
    /// key it begins right behind the block, and passes; a micrometre
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
        let micrometre = 1e-6;
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
            "the default tail begins right behind the block"
        );
        assert_eq!(s.validate(), Ok(()));
        s.key.tail_length += micrometre;
        under(&s, micrometre);

        for (tail_length, refused) in [(0.050 - micrometre, false), (0.050 + micrometre, true)] {
            let mut s = Standard::new();
            s.key.tail_end = -0.270;
            s.key.tail_length = tail_length;
            if refused {
                under(&s, micrometre);
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
    /// button**, by how far it overlaps it: a micrometre into it, the outline
    /// crosses itself by that micrometre; a micrometre short of it, the
    /// outline is whole, and the capstan, 129.5 mm behind the pin, is behind
    /// the block.
    #[test]
    fn a_back_check_block_pushed_into_the_key_button_crosses_the_outline() {
        let micrometre = 1e-6;
        let mut s = Standard::new();
        let button_back = -s.key.capsule_length * 0.5;
        s.key.back_check_block_back = button_back - s.key.back_check_block_length + micrometre;
        match s.validate() {
            Err(Problem::OutlineCrossesItself {
                back: PartCode::BackCheckBlock,
                front: PartCode::KeyButton,
                by,
            }) => assert!((by - micrometre).abs() <= ROUNDING, "{by}"),
            got => panic!("{got:?}"),
        }
        s.key.back_check_block_back -= 2.0 * micrometre;
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
            centre: crate::geometry::Point::new(0.05, 0.015),
            inertia: 0.002,
        };
        assert_eq!(s.validate(), Err(Problem::MassNotPositive { mass: -0.1 }));
    }

    /// The composite design with the key that carries every feature the
    /// composite key lacks ([`crate::test_fixture::key_with_every_feature`]).
    fn featured() -> Standard {
        Standard {
            key: crate::test_fixture::key_with_every_feature(),
            ..Standard::new()
        }
    }

    /// Consistency: **a key with every feature the composite lacks passes**,
    /// and its design's numbers name each feature, and name no felt on its
    /// bare tail.
    #[test]
    fn a_key_with_every_feature_passes_and_names_them() {
        let s = featured();
        assert_eq!(s.validate(), Ok(()));
        let names: Vec<&str> = numbers(&s.design()).iter().map(|(n, _)| *n).collect();
        for name in [
            "capsule_centre",
            "capstan_block.back",
            "capstan_block.length",
            "capstan_block.height",
            "capstan.dome_radius",
        ] {
            assert!(names.contains(&name), "{name}");
        }
        assert!(!names.contains(&"damper_felt"));
        let composite: Vec<&str> = numbers(&Standard::new().design())
            .iter()
            .map(|(n, _)| *n)
            .collect();
        for name in [
            "capsule_centre",
            "capstan_block.back",
            "capstan.dome_radius",
        ] {
            assert!(!composite.contains(&name), "{name} on the composite key");
        }
    }

    /// Consistency: **a capstan off its block, or domed narrower than its
    /// head, is refused**: moved a millimetre past its block's back end, its
    /// head stands that far off the block; a dome a micrometre narrower than
    /// the head's radius caps nothing.
    #[test]
    fn a_capstan_off_its_block_or_domed_narrower_than_its_head_is_refused() {
        let micrometre = 1e-6;
        let mut s = featured();
        let block = s.key.capstan_block.expect("a block");
        let head = 0.5 * s.key.capstan_screw.head_diameter;
        s.key.capstan = block.back + head - 0.001;
        match s.validate() {
            Err(Problem::CapstanOffItsBlock { by }) => {
                assert!((by - 0.001).abs() <= ROUNDING, "{by}");
            }
            got => panic!("{got:?}"),
        }
        let mut s = featured();
        s.key.capstan_screw.dome_radius = Some(head - micrometre);
        assert_eq!(
            s.validate(),
            Err(Problem::CapstanDomeNarrowerThanItsHead {
                dome: head - micrometre,
                head
            })
        );
        s.key.capstan_screw.dome_radius = Some(head);
        assert_eq!(s.validate(), Ok(()), "a hemisphere caps the head");
    }

    /// Consistency: **a glued block stands wholly on a bare thinned tail or
    /// wholly in front of it**: the tail stopped a millimetre short of the
    /// capstan's block's front end puts the block across the step, by that
    /// millimetre; the same tail felted runs its felt under the back check
    /// block, by how far the tail's front stands in front of the block's
    /// back end.
    #[test]
    fn a_block_across_the_step_to_a_bare_tail_crosses_the_outline() {
        let mut s = featured();
        let block = s.key.capstan_block.expect("a block");
        s.key.tail_length = block.front() - 0.001 - s.key.tail_end;
        match s.validate() {
            Err(Problem::OutlineCrossesItself {
                back: PartCode::CapstanBlock,
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
}

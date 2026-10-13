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
//! key button, its capstan in a glued block or off the full-height keystick,
//! its front mortise cut through the top of the key, its measuring point off
//! its covering. Everything an assembled keyboard can be, however badly
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
use crate::parts::frame::{PinShape, RailSection, PAPER};
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
    /// overlapping along its top, or the underside's chamfer running past
    /// the key front.
    OutlineCrossesItself {
        /// The part behind, whose front end stands too far forward.
        back: PartCode,
        /// The part in front, whose back end stands too far back.
        front: PartCode,
        /// How far the two overlap, metres along the key.
        by: f64,
    },
    /// A feature that stands off the keystick's wood where it must be cut
    /// or screwed: the capstan behind the back check block, on the thinned
    /// tail or past the key's end; the front mortise off the flat of the
    /// underside.
    FeatureOutsideKeystick {
        /// The feature.
        part: PartCode,
        /// How far off, metres along the key.
        by: f64,
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
/// key's outline, the capstan and the front mortise on the keystick, the
/// mortise under the key's top, and the measuring point on the covering.
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
        PAPER.admits(*paper)?;
    }
    if let KeyMass::Weighed { mass, .. } = *d.mass {
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

/// The key's outline along its top and its underside, back to front: each
/// part glued on the keystick ends before the next begins, and the chamfer
/// under the tail runs out before the key front. Two that overlap are the
/// outline crossing itself, by how far they overlap.
fn outline(key: &KeyDesign) -> Result<(), Problem> {
    let tail_front = key.tail_end + key.tail_length;
    let block_front = tail_front + key.back_check_block_length;
    let button_back = -key.capsule_length * 0.5;
    let button_front = key.capsule_length * 0.5;
    let keytop_back = key.lip() - key.keytop_length;
    let wood_front = key.front_edge - key.front_board;
    let chamfer_end = key.tail_end + key.tail_rise_run;
    for (back, ends, front, begins) in [
        (
            PartCode::BackCheckBlock,
            block_front,
            PartCode::KeyButton,
            button_back,
        ),
        (
            PartCode::KeyButton,
            button_front,
            PartCode::Keytop,
            keytop_back,
        ),
        (
            PartCode::Keystick,
            chamfer_end,
            PartCode::KeyFront,
            wood_front,
        ),
    ] {
        if ends > begins {
            return Err(Problem::OutlineCrossesItself {
                back,
                front,
                by: ends - begins,
            });
        }
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

/// The capstan stands on the keystick's full height between the back check
/// block and the key button, where nothing is glued on top: its head in
/// either block is the capstan in that block, by how far it stands in; the
/// whole of it behind the block, on the thinned tail or past the key's end,
/// is off the keystick, by how far it must come forward to clear the block.
fn capstan(key: &KeyDesign) -> Result<(), Problem> {
    let r = capstan_radius(key);
    let (back, front) = (key.capstan - r, key.capstan + r);
    let tail_front = key.tail_end + key.tail_length;
    let block_front = tail_front + key.back_check_block_length;
    let button_back = -key.capsule_length * 0.5;
    let button_front = key.capsule_length * 0.5;
    if front > button_back && back < button_front {
        return Err(Problem::CapstanInBlock {
            block: PartCode::KeyButton,
            by: front - button_back,
        });
    }
    if front <= tail_front {
        return Err(Problem::FeatureOutsideKeystick {
            part: PartCode::Capstan,
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
/// finiteness, and what an instrument's state hash reads of its design.
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
        back_check_block_height,
        back_check_block_length,
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
        ("back_check_block_height", *back_check_block_height),
        ("back_check_block_length", *back_check_block_length),
        ("tail_height", *tail_height),
        ("tail_length", *tail_length),
        ("damper_felt", *damper_felt),
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
    ];
    if let Some(m) = measured_stick_mass {
        out.push(("measured_stick_mass", *m));
    }
    match *d.mass {
        KeyMass::Estimated => {}
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
        front_rail,
        back_rail,
        sections,
    } = d.frame;
    out.extend([
        ("balance_rail.height", *balance_rail_height),
        ("balance_punching.felt", balance_punching.felt),
        ("front_rail.height", *front_rail_height),
        ("front_punching.felt", front_punching.felt),
        ("back_rail.height", *back_rail_height),
        ("back_rail.cloth", *back_rail_cloth),
        ("front_rail.x", *front_rail),
        ("back_rail.x", *back_rail),
    ]);
    out.extend(
        balance_punching
            .papers
            .iter()
            .chain(&front_punching.papers)
            .map(|&p| (PAPER.name, p)),
    );
    for (rail, section) in [
        ("balance_rail", &sections.balance),
        ("front_rail", &sections.front),
        ("back_rail", &sections.back),
    ] {
        section_numbers(rail, section, &mut out);
    }
    let TouchweightProtocol {
        measuring_point_from_lip,
    } = d.protocol;
    out.push(("measuring_point_from_lip", *measuring_point_from_lip));
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

/// A rail section's numbers, each named by its rail.
fn section_numbers(rail: &'static str, s: &RailSection, out: &mut Vec<(&'static str, f64)>) {
    let RailSection {
        half_length,
        crown,
        pin,
        pin_height,
        punching_radius,
        punching_hole_radius,
        sharp_row_offset,
    } = s;
    let name = |field: &'static str| -> &'static str {
        match (rail, field) {
            ("balance_rail", "pin_height") => "balance_rail.pin_height",
            ("front_rail", "pin_height") => "front_rail.pin_height",
            ("back_rail", "pin_height") => "back_rail.pin_height",
            ("balance_rail", _) => "sections.balance_rail",
            ("front_rail", _) => "sections.front_rail",
            _ => "sections.back_rail",
        }
    };
    out.extend([
        (name("half_length"), *half_length),
        (name("pin_height"), *pin_height),
        (name("punching_radius"), *punching_radius),
        (name("punching_hole_radius"), *punching_hole_radius),
        (name("sharp_row_offset"), *sharp_row_offset),
    ]);
    if let Some(c) = crown {
        out.extend([(name("crown"), c.half_length), (name("crown"), c.height)]);
    }
    match pin {
        Some(PinShape::Round { diameter }) => out.push((name("pin"), *diameter)),
        Some(PinShape::Oblong { length, thickness }) => {
            out.extend([(name("pin"), *length), (name("pin"), *thickness)]);
        }
        None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parts::frame::PAPER_THICKNESSES;
    use crate::registry::ParamSpec;

    /// The default key and frame, unweighed, on Stanwood's protocol and
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
            let key = KeyDesign::default();
            let protocol = TouchweightProtocol::default();
            let friction = AxisFriction::of_weight_at(6.0, protocol.measuring_point(&key));
            Self {
                key,
                mass: KeyMass::Estimated,
                frame: FrameDesign::default(),
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

    /// `spec` set to a value no number of the default design holds, on the
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

    /// Consistency: the default design is one parts can be built as.
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
    /// 230 mm behind it, on the thinned tail, it is off the keystick, 39.5 mm
    /// short of clearing the block.
    #[test]
    fn a_capstan_in_a_block_or_off_the_keystick_is_refused() {
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
            (
                -0.230,
                Problem::FeatureOutsideKeystick {
                    part: PartCode::Capstan,
                    by: 0.0395,
                },
            ),
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
                    Err(Problem::FeatureOutsideKeystick { part, by }),
                    Problem::FeatureOutsideKeystick {
                        part: want,
                        by: want_by,
                    },
                ) => {
                    assert_eq!(part, want, "{x}");
                    assert!((by - want_by).abs() < 1e-12, "{x}: {by}");
                }
                (got, _) => panic!("{x}: {got:?}"),
            }
        }
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
    }
}

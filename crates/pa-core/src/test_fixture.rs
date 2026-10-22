//! The owner's measured key section, its frame, the drawer and Stanwood's
//! protocol, and a test weighing of the key, as a test fixture.
//!
//! The engine starts from no number of its own: an instrument is built from
//! a profile (`data/profiles/`), each value bound to a record in
//! `data/params/` or to an assumption stated with its reason. The domain's
//! tests still need a key and a frame to push one way or another, so this
//! module holds the measured section's profile's values as Rust literals,
//! compiled for tests only: `pa-core`'s own, and those of the crates above
//! it that enable the `fixtures` feature as a dev-dependency. `pa-app`'s
//! `the_measured_section_profile_loads_to_the_test_fixture` holds the
//! profile to these bits, so the fixture the domain is tested on and the
//! instrument the bench builds cannot drift apart. Where a value comes from,
//! and why it is what it is, is the profile's to say, not this module's.
//!
//! The profile's key arrives unweighed, as nobody has weighed one of its
//! keys, and a key with no mass has no touchweight. So the fixture adds what
//! the profile leaves to the bench's user: a weighing ([`weighed`]), a test
//! weighing and not a measurement, which the same test holds to the bits
//! the bench takes it in.
//!
//! Neither profile carries a wippen assembly yet. The tests of the wippen
//! carry a test wippen ([`wippen_assembly`]) over the measured section's
//! capstan: Hirschkorn's numbers where they place a pivot, a contact or a
//! mass, its outlines simplified from the trace of his, each number a named
//! constant.

use crate::geometry::Point;
use crate::measure::TouchweightProtocol;
use crate::mechanism::AxisFriction;
use crate::parts::frame::{Crown, Drawer, FrameSections, PinShape, PunchingStack, RailSection};
use crate::parts::wippen_assembly::{HeelCloth, PartMass, PinnedPart, Stretch, WippenHeel};
use crate::parts::{
    BalanceMortise, Capstan, FrameDesign, FrontMortise, KeyDesign, KeyMass, TopClearance,
    WippenAssembly, WippenAssemblyMass,
};
use crate::regulation::Targets;

/// The thicknesses the owner's drawer holds, meters, thinnest first: 0.08,
/// 0.12, 0.20, 0.40, 0.50, 0.75, 1.00 and 1.25 mm.
pub const PAPER_THICKNESSES: [f64; 8] = [
    0.00008, 0.00012, 0.0002, 0.0004, 0.0005, 0.00075, 0.001, 0.00125,
];

/// Stanwood's measuring point, 13 mm back from the covering's lip, meters.
pub const MEASURING_POINT_FROM_LIP: f64 = 0.013;

/// The same glossary entry's imperial alternative, 1/2 inch, meters.
pub const MEASURING_POINT_HALF_INCH: f64 = 0.0127;

/// The load Stanwood's protocol reads the key weight ratio with, grams.
pub const KEY_RATIO_LOAD_G: f64 = 10.0;

/// The measured section's targets: 65.0 mm of key height and 10.2 mm of key
/// dip.
pub const TARGETS: Targets = Targets {
    key_height: 0.0650,
    key_dip: 0.0102,
};

/// The bushings the measured section starts on: the torque this many grams
/// overcome at the measuring point with the key level.
pub const KEY_FRICTION_WEIGHT_LEVEL_G: f64 = 6.0;

/// The test weighing's mass on the scale, grams.
pub const WEIGHED_MASS_G: f64 = 121.1;

/// The test weighing's front weight, grams, read on the leveled jig
/// [`WEIGHED_FROM_LIP`] back from the covering's lip.
pub const WEIGHED_FRONT_WEIGHT_G: f64 = 6.5;

/// Where the test weighing's front weight was read, meters back from the
/// covering's lip: Stanwood's measuring point.
pub const WEIGHED_FROM_LIP: f64 = MEASURING_POINT_FROM_LIP;

/// The fixture's key weighed whole, by a test weighing and not a
/// measurement: [`WEIGHED_MASS_G`] on the scale, and a front weight of
/// [`WEIGHED_FRONT_WEIGHT_G`] read [`WEIGHED_FROM_LIP`] back from its lip. A
/// key as heavy as a leaded natural and as light in front as a bare one, its
/// front weight a fraction of a gram past what its bushings hold: under no
/// weight it goes down onto its front punching, a pull of a few grams leaves
/// it there held by its friction, and one of 13 g brings it up onto its
/// cloth, so the touch weight moves it either way, and each of its two stops
/// holds it in turn.
pub fn weighed() -> KeyMass {
    KeyMass::weighed(
        &key(),
        WEIGHED_MASS_G,
        WEIGHED_FRONT_WEIGHT_G,
        WEIGHED_FROM_LIP,
    )
    .expect("the test weighing is a key's")
}

/// A balance pin standing 11.2 mm above its rail, 9 mm above the measured
/// section's regulated balance punching, its top inside the key's wood: short
/// enough that a pull, or stops raised a few millimeters, take the key off
/// it. The tests of a key leaving its pin stand it on this one; the measured
/// section's own stands above its key button, 33 mm above the punching.
pub const SHORT_BALANCE_PIN: f64 = 0.0112;

/// The measured section's steel capstan.
pub fn capstan() -> Capstan {
    Capstan {
        height: 0.0090,
        diameter: 0.0030,
        head_diameter: 0.0050,
        head_height: 0.0030,
        cove_height: 0.0008,
        shaft_length: 0.0130,
        tip_height: 0.0015,
        dome_radius: None,
    }
}

/// The measured section's key: a natural of the middle of the keyboard, its
/// section and its capstan, 122 mm behind the balance pin, measured, its key
/// front assumed 258 mm in front of it.
pub fn key() -> KeyDesign {
    KeyDesign {
        front_edge: 0.258,
        tail_end: -0.240,
        capstan: -0.122,
        front_mortise: FrontMortise {
            from_front_edge: 0.033,
            cutter_diameter: 0.006,
            travel: 0.002,
            depth: 0.022,
            groove_depth: 0.003,
            bushing_thickness: 0.0013,
            bushing_height: 0.003,
            bushing_flat_width: 0.005,
        },
        stick_height: 0.0245,
        capsule_height: 0.0060,
        capsule_length: 0.062,
        capsule_center: None,
        capsule_end_radius: 0.012,
        balance_mortise: BalanceMortise {
            cutter_diameter: 0.0060,
            travel: 0.0042,
            floor: 0.0025,
            hole_diameter: 0.0034,
            bushing_thickness: 0.0013,
            bushing_depth: 0.0030,
            side_hole_height: 0.0026,
        },
        back_check_block_height: 0.0080,
        back_check_block_back: -0.220,
        back_check_block_length: 0.027,
        top_clearance: None,
        tail_height: 0.0070,
        tail_length: 0.020,
        damper_felt: Some(0.0050),
        tail_rise: 0.0025,
        tail_rise_run: 0.077,
        keytop_thickness: 0.0015,
        front_board: 0.0020,
        keytop_overhang: 0.0020,
        keytop_length: 0.155,
        width: 0.022,
        capstan_screw: capstan(),
    }
}

/// The measured section's key with every feature a key may have that it
/// lacks, for the tests of those features: its thinned tail run on bare,
/// with no felt, 30 mm further forward, under its back check block, which
/// stands on it, to 190 mm behind the balance pin; a clearance for the
/// wippen rail cut 3 mm deep in its top, from 175 to 140 mm behind the
/// balance pin, between the back check block and the capstan, the keystick
/// at its full height either side of it; the capstan's head turned to a
/// dome of 9.7 mm; its key button 8 mm behind the balance pin.
pub fn key_with_every_feature() -> KeyDesign {
    KeyDesign {
        tail_length: 0.050,
        tail_height: 0.0170,
        damper_felt: None,
        top_clearance: Some(TopClearance {
            back: -0.175,
            front: -0.140,
            depth: 0.003,
        }),
        capsule_center: Some(-0.008),
        capstan_screw: Capstan {
            dome_radius: Some(0.0097),
            ..capstan()
        },
        ..key()
    }
}

/// The owner's drawer of punching papers.
pub fn drawer() -> Drawer {
    Drawer::graded(PAPER_THICKNESSES.to_vec()).expect("the owner's drawer is graded in hundredths")
}

/// The owner's measured key frame's three rails.
pub fn sections() -> FrameSections {
    FrameSections {
        balance: RailSection {
            half_length: 0.0255,
            crown: Some(Crown {
                half_length: 0.0215,
                height: 0.0060,
            }),
            pin: Some(PinShape::Round { diameter: 0.0034 }),
            pin_height: 0.0352,
            punching_radius: 0.0060,
            punching_hole_radius: 0.0020,
            sharp_row_offset: 0.0270,
        },
        front: RailSection {
            half_length: 0.0375,
            crown: None,
            pin: Some(PinShape::Oval {
                length: 0.0050,
                thickness: 0.0034,
            }),
            pin_height: 0.0240,
            punching_radius: 0.0110,
            punching_hole_radius: 0.0025,
            sharp_row_offset: 0.0490,
        },
        back: RailSection {
            half_length: 0.0120,
            crown: None,
            pin: None,
            pin_height: 0.0,
            punching_radius: 0.0,
            punching_hole_radius: 0.0,
            sharp_row_offset: 0.0,
        },
    }
}

/// The measured section's frame, its rails placed under its key, half a
/// millimeter of paper on each punching.
pub fn frame() -> FrameDesign {
    FrameDesign {
        balance_rail_height: 0.0266,
        balance_punching: PunchingStack::new(vec![0.0005], 0.0015),
        front_rail_height: 0.0259,
        front_punching: PunchingStack::new(vec![0.0005], 0.0020),
        back_rail_height: 0.0190,
        back_rail_cloth: 0.0030,
        back_rail_cloth_edge_radius: 0.0005,
        front_rail: 0.225,
        back_rail: -0.200,
        back_rail_from_the_tail: 0.040,
        sections: sections(),
        drawer: drawer(),
    }
}

/// Stanwood's protocol: the measuring point 13 mm back from the covering's
/// lip, the key weight ratio read with 10 g.
pub fn protocol() -> TouchweightProtocol {
    TouchweightProtocol {
        measuring_point_from_lip: MEASURING_POINT_FROM_LIP,
        key_ratio_load_g: KEY_RATIO_LOAD_G,
    }
}

/// Where the test wippen's center pin stands in the keybed's frame, meters:
/// behind the measured section's capstan, 122 mm behind its balance pin, by
/// about the reach of the heel's bearing point from the pin, and above the
/// keybed so that its heel bears on the capstan with the key at rest.
pub const WIPPEN_CENTER: Point = Point::new(-0.1850, 0.0785);

/// The jack's pin on the test wippen, meters in its frame: Hirschkorn's Pw2.
pub const JACK_PIN: Point = Point::new(0.0989, -0.0035);

/// The repetition lever's pin on the test wippen, meters in its frame:
/// Hirschkorn's Pw3.
pub const REPETITION_LEVER_PIN: Point = Point::new(0.0483, 0.0275);

/// The jack's rest angle on the test wippen, radians: on its stop, as the
/// trace of Hirschkorn's closes it.
pub const JACK_REST_ANGLE: f64 = 0.3189;

/// The repetition lever's rest angle on the test wippen, radians: on its
/// stop.
pub const REPETITION_LEVER_REST_ANGLE: f64 = 0.3463;

/// The jack's stop on the test wippen, a line through a point with its
/// normal, in the wippen's frame, meters: Hirschkorn's Pw4.
pub const JACK_STOP: (Point, Point) = (Point::new(0.0759, 0.0189), Point::new(0.969, 0.247));

/// What rests on the jack's stop, a circle in the jack's frame, its center
/// and its radius, meters: Hirschkorn's Pj4.
pub const JACK_BUTTON: (Point, f64) = (Point::new(-0.0097, 0.0300), 0.0050);

/// The repetition lever's stop on the test wippen, a line through a point
/// with its normal, in the wippen's frame, meters: Hirschkorn's Pw6.
pub const REPETITION_LEVER_STOP: (Point, Point) =
    (Point::new(0.0170, 0.0015), Point::new(0.0, 1.0));

/// What rests on the lever's stop, a circle in the lever's frame, its center
/// and its radius, meters: Hirschkorn's Pr3.
pub const REPETITION_LEVER_BUTTON: (Point, f64) = (Point::new(-0.0361, -0.0093), 0.0050);

/// The test wippen's heel: Hirschkorn's circle (Pw5), its cloth and its
/// corners as the trace of his places them, meters. Its position is Pw5's
/// `x`; its height is Pw2's `y` less Pw5's `y` plus Pw5's radius,
/// −3.5 − 3.0 + 25.3 = 18.8 mm.
pub const HEEL: WippenHeel = WippenHeel {
    position: 0.0595,
    height: 0.0188,
    radius: 0.0253,
    top: -0.00795,
    back_top: 0.01643,
    front_top: 0.01591,
    back: 0.00958,
    front: 0.01488,
    cloth: HeelCloth {
        back: 0.00555,
        front: 0.01084,
        thickness: 0.00211,
    },
};

/// The test wippen's center pin, meters across.
pub const CENTER_PIN_DIAMETER: f64 = 0.00127;

/// The test wippen's width across the keyboard, meters.
pub const WIPPEN_WIDTH: f64 = 0.010;

/// The test jack's width across the keyboard, meters.
pub const JACK_WIDTH: f64 = 0.006;

/// The test repetition lever's width across the keyboard, meters.
pub const REPETITION_LEVER_WIDTH: f64 = 0.010;

/// The friction of the test wippen's center, newton-meters: Hirschkorn's
/// Table A.9.
pub const WIPPEN_FRICTION_N_M: f64 = 3.97e-4;

/// A point in millimeters, as a trace is read, in meters.
const fn point_from_mm(x: f64, y: f64) -> Point {
    Point::new(x * 1e-3, y * 1e-3)
}

/// The test wippen's body, in its frame: the trace of Hirschkorn's, its
/// coves each taken as one chord, its post's round top as one arc.
pub fn wippen_body() -> Vec<Stretch> {
    use Stretch::{Arc, Line};
    vec![
        Line(point_from_mm(-12.53, -4.01)),
        Line(point_from_mm(3.28, -4.04)),
        Line(point_from_mm(9.47, -7.95)),
        Arc {
            from: point_from_mm(103.04, -7.98),
            center: point_from_mm(100.39, -3.57),
            radius: 0.00515,
        },
        Line(point_from_mm(102.39, 1.17)),
        Line(point_from_mm(64.65, 1.24)),
        Line(point_from_mm(58.62, 6.07)),
        Line(point_from_mm(52.44, 6.17)),
        Arc {
            from: point_from_mm(52.45, 31.59),
            center: point_from_mm(48.73, 27.48),
            radius: 0.00554,
        },
        Line(point_from_mm(44.5, 31.07)),
        Line(point_from_mm(44.34, 6.15)),
        Line(point_from_mm(38.42, 6.04)),
        Line(point_from_mm(31.46, 1.08)),
        Line(point_from_mm(23.46, 1.09)),
        Line(point_from_mm(23.43, -0.1)),
        Line(point_from_mm(11.6, 0.05)),
        Line(point_from_mm(11.56, 1.19)),
        Line(point_from_mm(7.71, 5.98)),
        Line(point_from_mm(-8.56, 6.05)),
        Line(point_from_mm(-12.5, 1.04)),
    ]
}

/// The test jack, in its own frame: an L, its toe's end on Hirschkorn's Pj3
/// and the top of its back face rounded on his Pj2, its front face on his
/// Pj5's line.
pub fn jack() -> Vec<Stretch> {
    use Stretch::{Arc, Line};
    vec![
        Arc {
            from: point_from_mm(24.7, -7.1),
            center: point_from_mm(24.7, -5.0),
            radius: 0.0021,
        },
        Line(point_from_mm(24.7, -2.9)),
        Line(point_from_mm(3.1, -2.0)),
        Line(point_from_mm(3.1, 50.5)),
        Arc {
            from: point_from_mm(-1.0, 50.5),
            center: point_from_mm(-1.0, 49.5),
            radius: 0.0010,
        },
        Line(point_from_mm(-2.0, 49.5)),
        Line(point_from_mm(-2.0, -7.1)),
    ]
}

/// The test repetition lever, in its own frame: a bar, its top curving
/// down to its front end on Hirschkorn's Pr2.
pub fn repetition_lever() -> Vec<Stretch> {
    use Stretch::{Arc, Line};
    vec![
        Line(point_from_mm(-41.0, 3.1)),
        Line(point_from_mm(-41.0, 0.0)),
        Line(point_from_mm(-32.3, -3.3)),
        Line(point_from_mm(59.9, -3.3)),
        Arc {
            from: point_from_mm(59.9, -2.2),
            center: point_from_mm(36.3, -52.1),
            radius: 0.0552,
        },
        Line(point_from_mm(36.3, 3.1)),
    ]
}

/// The test wippen assembly over the measured section's capstan.
pub fn wippen_assembly() -> WippenAssembly {
    WippenAssembly {
        center: WIPPEN_CENTER,
        body: wippen_body(),
        heel: HEEL,
        jack: PinnedPart {
            pin: JACK_PIN,
            rest_angle: JACK_REST_ANGLE,
            outline: jack(),
            width: JACK_WIDTH,
        },
        repetition_lever: PinnedPart {
            pin: REPETITION_LEVER_PIN,
            rest_angle: REPETITION_LEVER_REST_ANGLE,
            outline: repetition_lever(),
            width: REPETITION_LEVER_WIDTH,
        },
        center_pin_diameter: CENTER_PIN_DIAMETER,
        width: WIPPEN_WIDTH,
        friction: AxisFriction {
            bound_n_m: WIPPEN_FRICTION_N_M,
            radius_m: None,
        },
    }
}

/// The test wippen assembly's mass: Hirschkorn's Table A.1, its three parts
/// each with its mass, its center in its own frame and its inertia.
pub fn wippen_assembly_mass() -> WippenAssemblyMass {
    WippenAssemblyMass::Published {
        wippen: PartMass {
            mass: 0.01196,
            center: Point::new(0.0502, -0.0018),
            inertia: 1.69e-5,
        },
        jack: PartMass {
            mass: 0.00265,
            center: Point::new(0.0008, 0.0195),
            inertia: 7.78e-7,
        },
        repetition_lever: PartMass {
            mass: 0.00371,
            center: Point::new(0.0021, -0.0012),
            inertia: 4.1e-6,
        },
    }
}

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

use crate::measure::TouchweightProtocol;
use crate::parts::frame::{Crown, Drawer, FrameSections, PinShape, PunchingStack, RailSection};
use crate::parts::{BalanceMortise, Capstan, FrameDesign, KeyDesign, KeyMass, TopClearance};
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
        front_mortise_from_front_edge: 0.033,
        front_mortise_cutter_diameter: 0.006,
        front_mortise_travel: 0.002,
        front_mortise_depth: 0.022,
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

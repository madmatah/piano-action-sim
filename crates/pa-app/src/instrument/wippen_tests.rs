//! The instrument with the test wippen of [`pa_core::test_fixture`] on its
//! capstan: the owner's measured section weighed by the tests' weighing,
//! carrying it as a profile binds one, built through the same constructor a
//! profile's load and a switch to it go through ([`Instrument::new`]).
//! Tests only.

use super::*;
use crate::registry::{path, Control, PAGE};

/// The wippen a key carries in `b`, as the bench names what carries it:
/// the capstan on the heel cloth, on the heel's wood, or the heel on the
/// key's top; anything else as it is.
fn carried(b: &Instrument) -> String {
    match &b.wippen_carried_by()[..] {
        [one] => match (one.link, &one.parts[..]) {
            (ContactCode::CapstanOnTheHeel, [PartCode::WippenHeelCloth]) => "the cloth".to_owned(),
            (ContactCode::CapstanOnTheHeel, [PartCode::WippenHeel]) => "the wood".to_owned(),
            (ContactCode::HeelOnTheKey, _) => "the key's top".to_owned(),
            _ => format!("{one:?}"),
        },
        other => format!("{other:?}"),
    }
}

/// The slider `path` of the page, its soft range and step.
fn slider(path: &str) -> registry::Soft {
    PAGE.iter()
        .flat_map(|(_, controls)| controls.iter())
        .find_map(|c| match *c {
            Control::Slider { path: p, soft, .. } if p == path => Some(soft),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{path} is a slider of the page"))
}

/// What one step of a slider left on the bench: the value set, whether on
/// the way up, whether the bench took it, whether it put the key back by
/// the loading rule, where the key is drawn, what holds it, whether the
/// frame shows a fault, and what carries the wippen.
#[derive(Debug)]
struct Step {
    value: f64,
    rising: bool,
    taken: Result<(), EditRefusal>,
    restarted: bool,
    place: KeyPlace,
    verdict: Option<TouchVerdict>,
    has_fault: bool,
    carried: String,
}

/// The test bench stepped through the whole soft range of the slider
/// `path`, as the page steps it: from where it stands down to its least by
/// the slider's step, then up to its most, each step as it left the bench.
fn stepped(path: &str) -> Vec<Step> {
    let soft = slider(path);
    let (least, most) = (soft.min.expect("bounded"), soft.max.expect("bounded"));
    let mut b = Instrument::carrying_the_test_wippen();
    let now = b.value(path).expect("the bench carries a wippen");
    let below = ((now - least) / soft.step).floor() as u32;
    let down = (0..=below).rev().map(|k| least + f64::from(k) * soft.step);
    let up = (0..)
        .map(|k| least + f64::from(k) * soft.step)
        .take_while(|&value| value <= most);
    down.map(|v| (v, false))
        .chain(up.map(|v| (v, true)))
        .map(|(value, rising)| {
            let taken = b.set(path, value);
            Step {
                value,
                rising,
                taken,
                restarted: b.reinitialized(),
                place: b.key_place(),
                verdict: b.touch().map(|t| t.verdict),
                has_fault: b.frame_fault().is_some(),
                carried: carried(&b),
            }
        })
        .collect()
}

/// The steps that did not keep the key where it stood, each as it left the
/// bench: refused, putting the key back by the loading rule, the key not
/// placed, not held by its friction on its cloth, or the frame showing a
/// fault.
fn unkept(steps: &[Step]) -> Vec<String> {
    steps
        .iter()
        .filter(|step| {
            step.taken.is_err()
                || step.restarted
                || step.place != KeyPlace::Placed
                || !matches!(
                    step.verdict,
                    Some(TouchVerdict::HeldByFriction {
                        touching: Some(ContactCode::BackRailCloth),
                        ..
                    })
                )
                || step.has_fault
        })
        .map(|step| format!("{step:?}"))
        .collect()
}

/// What carries the wippen at each step on the way up.
fn carried_rising(steps: &[Step]) -> Vec<(f64, String)> {
    steps
        .iter()
        .filter(|step| step.rising)
        .map(|step| (step.value, step.carried.clone()))
        .collect()
}

/// `runs` as the steps they cover: each run from its first value to its
/// last, in millimeters, with what carries the wippen there.
fn expected(runs: &[(f64, f64, &str)], step: f64) -> Vec<(f64, String)> {
    runs.iter()
        .flat_map(|&(from, to, what)| {
            let n = ((to - from) / step).round() as u32;
            (0..=n).map(move |k| (from + f64::from(k) * step, what.to_owned()))
        })
        .collect()
}

/// Consistency: **stepping the capstan's height keeps the key where it
/// stands and its heel on the cloth**, over the page's 4 to 24 mm by half
/// millimeters: each step an increment from where the key stood, never the
/// key restarted, held by its friction on its cloth, its heel on the cloth.
#[test]
fn stepping_the_capstan_height_keeps_the_key_where_it_stands_and_its_heel_on_the_cloth() {
    let steps = stepped(path::CAPSTAN_HEIGHT);
    assert_eq!(unkept(&steps), Vec::<String>::new());
    assert_eq!(
        carried_rising(&steps),
        expected(&[(4.0, 24.0, "the cloth")], 0.5)
    );
}

/// Consistency: **stepping the wippen's center along the key moves the
/// capstan along the heel without restarting the key**, over the page's 171
/// to 191 mm behind the balance pin by half millimeters: the capstan bears
/// on the heel's wood behind the cloth up to 176.5 mm, and on the cloth from
/// 177 mm, as the seats find it at the top of the stroke.
#[test]
fn stepping_the_wippens_center_along_the_key_moves_the_capstan_along_the_heel_without_restarting_the_key(
) {
    let steps = stepped(path::WIPPEN_CENTER_BEHIND);
    assert_eq!(unkept(&steps), Vec::<String>::new());
    assert_eq!(
        carried_rising(&steps),
        expected(
            &[(171.0, 176.5, "the wood"), (177.0, 191.0, "the cloth")],
            0.5
        )
    );
}

/// Consistency: **stepping the wippen's center up and down keeps the key
/// where it stands and its heel on the cloth**, over the page's 71 to 92 mm
/// above the keybed by half millimeters.
#[test]
fn stepping_the_wippens_center_height_keeps_the_key_where_it_stands_and_its_heel_on_the_cloth() {
    let steps = stepped(path::WIPPEN_CENTER_ABOVE);
    assert_eq!(unkept(&steps), Vec::<String>::new());
    assert_eq!(
        carried_rising(&steps),
        expected(&[(71.0, 92.0, "the cloth")], 0.5)
    );
}

/// Consistency: **stepping the heel's position moves the capstan along the
/// heel without restarting the key**, over the page's 50.0 to 69.5 mm from
/// the center pin by half millimeters: on the cloth up to 68 mm, on the
/// heel's wood behind it from 68.5 mm, the heel standing that far forward
/// over a capstan that stays where it is.
#[test]
fn stepping_the_heels_position_moves_the_capstan_along_the_heel_without_restarting_the_key() {
    let steps = stepped(path::HEEL_POSITION);
    assert_eq!(unkept(&steps), Vec::<String>::new());
    assert_eq!(
        carried_rising(&steps),
        expected(&[(50.0, 68.0, "the cloth"), (68.5, 69.5, "the wood")], 0.5)
    );
}

/// Consistency: **stepping the heel's height keeps the key where it stands
/// and its heel on the cloth**, over the page's 16 to 24 mm below the jack's
/// pin by half millimeters.
#[test]
fn stepping_the_heels_height_keeps_the_key_where_it_stands_and_its_heel_on_the_cloth() {
    let steps = stepped(path::HEEL_HEIGHT);
    assert_eq!(unkept(&steps), Vec::<String>::new());
    assert_eq!(
        carried_rising(&steps),
        expected(&[(16.0, 24.0, "the cloth")], 0.5)
    );
}

/// Consistency: **stepping the wippen center's friction keeps the key where
/// it stands and its heel on the cloth**, over the page's 0 to 200 g·mm by
/// the gram-millimeter.
#[test]
fn stepping_the_wippen_centers_friction_keeps_the_key_where_it_stands_and_its_heel_on_the_cloth() {
    let steps = stepped(path::WIPPEN_FRICTION_TORQUE);
    assert_eq!(unkept(&steps), Vec::<String>::new());
    assert_eq!(
        carried_rising(&steps),
        expected(&[(0.0, 200.0, "the cloth")], 1.0)
    );
}

/// The test bench as it arrives, its papers the ones the regulation lays
/// with the wippen on, built again as recorded with its capstan `height`
/// meters tall and its wippen's center pin `center_height` meters above the
/// keybed: what an instrument loads, with no regulation in between.
fn as_recorded(height: f64, center_height: f64) -> Result<Instrument, EditRefusal> {
    let (start, profile) = recorded(height, center_height);
    Instrument::new(start, profile)
}

/// What [`as_recorded`] loads: the start and its profile.
fn recorded(height: f64, center_height: f64) -> (crate::profile::Start, &'static Profile) {
    let regulated = Instrument::carrying_the_test_wippen();
    let profile = Profile::weighed_section();
    let mut start = profile.start.clone();
    start.frame = regulated.frame().clone();
    start.key.capstan_screw.height = height;
    start.arrival = Arrival::AsRecorded;
    let mut assembly = pa_core::test_fixture::wippen_assembly();
    assembly.center.y = center_height;
    start.wippen_assembly = Some(crate::profile::WippenAssemblyStart {
        assembly,
        mass: pa_core::test_fixture::wippen_assembly_mass(),
        bases: WippenAssemblyBases {
            mass: Basis::Published,
            friction: Basis::Published,
        },
    });
    (start, profile)
}

/// Consistency: **a bench whose wippen fits only at the key's own seat
/// loads with its wippen in range.** On a capstan 30 mm tall, the most its
/// hard range takes, under a center pin 72 mm above the keybed, both within
/// what the page offers or at its edge, the wippen carried on the key level
/// would stand past its 30°: the loading finds it nowhere there, goes by
/// the key alone's top seat, lowers the wippen onto the key there, and the
/// key comes to rest at its top seat with its wippen a degree or two under
/// its range's end, placed, nothing refused. A switch to a profile builds
/// its instrument through the same constructor.
#[test]
fn a_bench_whose_wippen_fits_only_at_the_keys_own_seat_loads_with_its_wippen_in_range() {
    let b = as_recorded(TALL_CAPSTAN_MM * 1e-3, FITS_AT_THE_TOP_SEAT_ONLY_MM * 1e-3)
        .expect("the bench loads");
    let k = b.on_frame().expect("a mount");
    assert_eq!(
        mounts::wippen_lowered(&k.mounted, &mounts::nominal_pose(&k.mounted)),
        None,
        "with the key level the wippen fits nowhere"
    );
    assert_eq!(b.key_place(), KeyPlace::Placed);
    assert_eq!(b.wippen_refused(), None);
    let angle = b.wippen_angle().expect("a wippen");
    let limit = mounts::WIPPEN_ANGLE_LIMIT;
    assert!(
        angle < limit && angle > limit - 0.1,
        "the wippen stands at {angle} rad"
    );
}

/// Consistency: **an edit that leaves the heel in the key at every turn
/// puts the key back by the loading rule and says so.** On a capstan 30 mm
/// tall the key is pressed to the bottom of its stroke by 60 g and the
/// weight taken off: its friction holds it there. The center pin then
/// lowered to 72 mm in one step stands the heel in the capstan at every
/// turn of the wippen with the key where it is, so the increment cannot
/// start there. The key is put back by the loading rule, which finds it at
/// its top seat with its wippen in range, and the bench says the key was
/// restarted.
#[test]
fn an_edit_that_leaves_the_heel_in_the_key_at_every_turn_puts_the_key_back_by_the_loading_rule_and_says_so(
) {
    let mut b = Instrument::carrying_the_test_wippen();
    b.set(path::CAPSTAN_HEIGHT, TALL_CAPSTAN_MM)
        .expect("a tall capstan");
    assert!(!b.reinitialized());
    b.set(path::WEIGHT, 60.0).expect("a weight");
    b.set(path::WEIGHT, 0.0).expect("no weight");
    assert!(
        matches!(
            b.touch().map(|t| t.verdict),
            Some(TouchVerdict::HeldByFriction {
                touching: Some(ContactCode::FrontRailPunching),
                ..
            })
        ),
        "the key stays down, held by its friction: {:?}",
        b.touch()
    );
    b.set(path::WIPPEN_CENTER_ABOVE, FITS_AT_THE_TOP_SEAT_ONLY_MM)
        .expect("a center pin lowered");
    assert!(
        b.reinitialized(),
        "the key was put back by the loading rule"
    );
    assert_eq!(b.key_place(), KeyPlace::Placed);
    assert_eq!(b.wippen_refused(), None);
    assert!(
        matches!(
            b.touch().map(|t| t.verdict),
            Some(TouchVerdict::HeldByFriction {
                touching: Some(ContactCode::BackRailCloth),
                ..
            })
        ),
        "back at its top seat: {:?}",
        b.touch()
    );
}

/// Consistency: **an edit that pushes the wippen past its range even at the
/// key's own seat is drawn faded.** On a capstan 30 mm tall, the center pin
/// lowered to 69 mm stands the heel in the capstan at every turn of the
/// wippen even with the key at its own top seat: the increment is sent to
/// the loading rule, which refuses the wippen pushed past its range, its
/// heel in the key at the top of its range by no more than the 3 mm the
/// center pin was lowered from 72 mm, where the wippen fits at the key's
/// own top seat. No state is taken: the key and its wippen are drawn where
/// the last one left them, faded, and the refusal is said.
#[test]
fn an_edit_that_pushes_the_wippen_past_its_range_even_at_the_keys_own_seat_is_drawn_faded() {
    const LOWERED_TO_MM: f64 = 69.0;
    let mut b = Instrument::carrying_the_test_wippen();
    b.set(path::CAPSTAN_HEIGHT, TALL_CAPSTAN_MM)
        .expect("a tall capstan");
    let before = b.poses();
    b.set(path::WIPPEN_CENTER_ABOVE, LOWERED_TO_MM)
        .expect("a center pin within its hard range");
    assert!(
        b.reinitialized(),
        "the increment was sent to the loading rule"
    );
    assert_eq!(b.key_place(), KeyPlace::WippenRefused);
    match b.wippen_refused() {
        Some(WippenRefusal::PushedPastItsRange {
            at: WhichSeat::Top,
            heel_depth,
        }) => assert!(
            heel_depth > 0.0 && heel_depth <= (FITS_AT_THE_TOP_SEAT_ONLY_MM - LOWERED_TO_MM) * 1e-3,
            "{heel_depth} m deep"
        ),
        other => panic!("{other:?}"),
    }
    assert_eq!(b.poses(), before, "drawn where the last state left it");
    assert!(b.wippen_carried_by().is_empty(), "no equilibrium places it");
}

/// How the sheet refuses the key's seats, where it refuses them: the key
/// dip's refusal, which seats the key at the top and then at the bottom.
fn the_sheets_refusal(b: &Instrument) -> Option<SeatRefusal> {
    match b.sheet().get(ProcedureId::KeyDip) {
        Some(Err(Refused::Seat(refusal))) => Some(refusal.clone()),
        _ => None,
    }
}

/// The capstan's height the hard range takes at most, millimeters.
const TALL_CAPSTAN_MM: f64 = 30.0;

/// The center pin at which the test wippen on a capstan [`TALL_CAPSTAN_MM`]
/// tall fits at the key's top seat and is pushed past its range at the
/// bottom seat, millimeters.
const FITS_AT_THE_TOP_SEAT_ONLY_MM: f64 = 72.0;

/// A weight that takes the key down, grams: more than its down weight with
/// the wippen.
const PRESSED_G: f64 = 60.0;

/// Consistency: **a weight that pushes the wippen past its range is drawn
/// faded, naming the bottom seat and the sheet's depth.** On a capstan
/// 30 mm tall under a center pin 72 mm high, loaded as recorded, the key
/// stands at its top seat with its wippen in range, and the sheet refuses
/// the bottom seat, the wippen pushed past its range there. 60 g on the
/// measuring point, an increment from where the key stands, leans the
/// wippen on the top of its range as the key goes down, its heel clear of
/// the key where the increment started: no state is taken, the key is not
/// restarted, it is drawn where the last state left it, and the refusal
/// names the bottom seat with the depth the sheet refuses it with, to the
/// bit, more than nothing.
#[test]
fn a_weight_that_pushes_the_wippen_past_its_range_is_drawn_faded_naming_the_bottom_seat_and_the_sheets_depth(
) {
    let mut b = as_recorded(TALL_CAPSTAN_MM * 1e-3, FITS_AT_THE_TOP_SEAT_ONLY_MM * 1e-3)
        .expect("the bench loads");
    assert_eq!(b.key_place(), KeyPlace::Placed);
    let before = b.poses();
    b.set(path::WEIGHT, PRESSED_G).expect("a weight");
    assert_eq!(b.key_place(), KeyPlace::WippenRefused);
    assert!(
        !b.reinitialized(),
        "the increment started where the key stood"
    );
    assert_eq!(b.poses(), before, "drawn where the last state left it");
    let Some(WippenRefusal::PushedPastItsRange {
        at: WhichSeat::Bottom,
        heel_depth,
    }) = b.wippen_refused()
    else {
        panic!("{:?}", b.wippen_refused())
    };
    assert_eq!(
        b.wippen_refused().map(SeatRefusal::from),
        the_sheets_refusal(&b)
    );
    assert!(heel_depth > 0.0, "{heel_depth} m deep");
}

/// How far below the lowest fit [`find_the_center_pins_in_the_band`]
/// searches, micrometers of center pin: as far as the domain's sweep of
/// the band below the lowest fit, which goes past the band
/// (`SWEPT_BELOW_THE_LOWEST_FIT_UM` in `pa-core`'s tests of the seats of a
/// key carrying its wippen).
const SEARCHED_BELOW_THE_LOWEST_FIT_UM: u32 = 40;

/// How deep the heel stands in the key at the top of the wippen's range,
/// the key alone at its bottom seat, on `b`'s mount, meters: by the drawn
/// outlines, and by the links' rows
/// ([`pa_core::parts::mounts::Mounted::measure_the_links_overlap`]).
fn measure_the_heel_at_the_top_of_its_range_at_the_bottom_seat(b: &Instrument) -> (f64, f64) {
    let on_frame = b.on_frame().expect("the key on its frame");
    let wippen = on_frame.mounted.wippen.expect("a wippen");
    let alone = Seating {
        mounted: on_frame
            .mounted
            .key_alone
            .as_deref()
            .expect("the key alone"),
        ..b.seating_on(&on_frame)
    }
    .seats(&mut Statics::new())
    .expect("the key alone seats")
    .bottom;
    let mut at_the_top = alone.q;
    at_the_top.insert(
        wippen.angle.0,
        on_frame.mounted.mechanism.coords()[wippen.angle.0].limits.1,
    );
    (
        pa_core::parts::mounts::heel_in_the_key(&on_frame.mounted, &at_the_top),
        on_frame.mounted.measure_the_links_overlap(&at_the_top),
    )
}

/// The center pins, a micron at a time down from the lowest at which a
/// bench loaded as recorded on a capstan [`TALL_CAPSTAN_MM`] tall seats at
/// both seats ([`Instrument::seated`], halved as [`the_lifting_bench`]
/// halves it), for [`SEARCHED_BELOW_THE_LOWEST_FIT_UM`], at which the drawn
/// outlines clear the key at the top of the wippen's range, the key alone
/// at its bottom seat, while the links stand the heel's cloth in it, the
/// drawn outlines' eight chords to an arc standing inside it: the band,
/// from its top down.
fn find_the_center_pins_in_the_band() -> Vec<f64> {
    let fits = |center: f64| {
        as_recorded(TALL_CAPSTAN_MM * 1e-3, center)
            .expect("the bench loads")
            .seated()
            .is_ok()
    };
    let (_, lowest_fit) = read::halve_between(
        FITS_AT_THE_TOP_SEAT_ONLY_MM * 1e-3,
        pa_core::test_fixture::WIPPEN_CENTER.y,
        fits,
    );
    (1..=SEARCHED_BELOW_THE_LOWEST_FIT_UM)
        .map(|micrometers| lowest_fit - f64::from(micrometers) * 1e-6)
        .filter(|&center| {
            let b = as_recorded(TALL_CAPSTAN_MM * 1e-3, center).expect("the bench loads");
            let (drawn, links) = measure_the_heel_at_the_top_of_its_range_at_the_bottom_seat(&b);
            drawn == 0.0 && links > 0.0
        })
        .collect()
}

/// Consistency: **a weight on a bench set inside the band names the sheet's
/// bottom seat and depth.** On a capstan 30 mm tall, loaded as recorded at
/// the first center pin of the band ([`find_the_center_pins_in_the_band`])
/// at which the key loads placed, so that a weight is an increment from
/// there: 60 g on the measuring point leans the wippen on the top of its
/// range, and the bench draws it faded, naming the bottom seat with the
/// depth the sheet's key dip refuses it with, to the bit, more than
/// nothing, and not the key lifting off its frame.
#[test]
fn a_weight_on_a_bench_set_inside_the_band_names_the_sheets_bottom_seat_and_depth() {
    let mut b = find_the_center_pins_in_the_band()
        .into_iter()
        .map(|center| as_recorded(TALL_CAPSTAN_MM * 1e-3, center).expect("the bench loads"))
        .find(|b| b.key_place() == KeyPlace::Placed)
        .expect("a center pin in the band at which the bench loads its key placed");
    b.set(path::WEIGHT, PRESSED_G).expect("a weight");
    assert_eq!(b.key_place(), KeyPlace::WippenRefused);
    let Some(WippenRefusal::PushedPastItsRange {
        at: WhichSeat::Bottom,
        heel_depth,
    }) = b.wippen_refused()
    else {
        panic!("{:?}", b.wippen_refused())
    };
    assert_eq!(
        b.wippen_refused().map(SeatRefusal::from),
        the_sheets_refusal(&b)
    );
    assert!(heel_depth > 0.0, "{heel_depth} m deep");
}

/// Consistency: **a loading at the top of the band is refused at the
/// sheet's bottom seat and depth.** Friction off and no weight on it, the
/// test bench's key set just below the lowest fit has two rests: started at
/// its top seat it stays there; started level it sinks until its capstan
/// carries the wippen onto the top of its range, short of its bottom seat,
/// where the heel's cloth would stand in the key, and the solve leans the
/// wippen on that limit. The loading starts it from its nominal pose,
/// level, and tries it from above only where that lifts it off its frame
/// or finds no state, not where the wippen's limits refuse it. On a capstan
/// 30 mm tall, loaded as recorded at each center pin of the band
/// ([`find_the_center_pins_in_the_band`]) above the first at which the key
/// loads placed, of which there is at least one: no state is taken, the
/// key unplaced, and the refusal names the bottom seat with the depth the
/// sheet's key dip refuses it with, to the bit, more than nothing.
#[test]
fn a_loading_at_the_top_of_the_band_is_refused_at_the_sheets_bottom_seat_and_depth() {
    let refused: Vec<Instrument> = find_the_center_pins_in_the_band()
        .into_iter()
        .map(|center| as_recorded(TALL_CAPSTAN_MM * 1e-3, center).expect("the bench loads"))
        .take_while(|b| b.key_place() != KeyPlace::Placed)
        .collect();
    assert!(!refused.is_empty(), "the band's top loads refused");
    for b in &refused {
        assert_eq!(b.key_place(), KeyPlace::Unplaced);
        let Some(WippenRefusal::PushedPastItsRange {
            at: WhichSeat::Bottom,
            heel_depth,
        }) = b.wippen_refused()
        else {
            panic!("{:?}", b.wippen_refused())
        };
        assert_eq!(
            b.wippen_refused().map(SeatRefusal::from),
            the_sheets_refusal(b)
        );
        assert!(heel_depth > 0.0, "{heel_depth} m deep");
    }
}

/// Consistency: **an edit that leaves the wippen out of reach names the top
/// seat and the sheet's clearance.** On the test bench 60 g takes the key
/// down and, taken off, leaves it there, held by its friction on its front
/// rail punching, the key carrying its wippen weighing up less than nothing.
/// The center pin then raised from 78.5 mm to 120 mm leans the wippen on the
/// bottom of its range with the key at its bottom, where its back stands
/// highest, and the sheet refuses the top seat, the wippen out of reach
/// there. The bench names the top seat with the clearance the sheet refuses
/// it with, to the bit: more than nothing, and no more than the 41.5 mm the
/// center pin was raised.
#[test]
fn an_edit_that_leaves_the_wippen_out_of_reach_names_the_top_seat_and_the_sheets_clearance() {
    const RAISED_MM: f64 = 120.0;
    let mut b = Instrument::carrying_the_test_wippen();
    let center_mm = b
        .value(path::WIPPEN_CENTER_ABOVE)
        .expect("a wippen's center");
    b.set(path::WEIGHT, PRESSED_G).expect("a weight");
    b.set(path::WEIGHT, 0.0).expect("no weight");
    assert!(
        matches!(
            b.touch().map(|t| t.verdict),
            Some(TouchVerdict::HeldByFriction {
                touching: Some(ContactCode::FrontRailPunching),
                ..
            })
        ),
        "the key stays down: {:?}",
        b.touch()
    );
    b.set(path::WIPPEN_CENTER_ABOVE, RAISED_MM)
        .expect("a center pin within its hard range");
    let sheet = the_sheets_refusal(&b);
    let Some(SeatRefusal::WippenOutOfReach {
        at: WhichSeat::Top,
        heel_clearance,
    }) = sheet
    else {
        panic!("the sheet refuses the top seat: {sheet:?}")
    };
    assert_eq!(b.wippen_refused().map(SeatRefusal::from), sheet);
    assert!(
        heel_clearance > 0.0 && heel_clearance <= (RAISED_MM - center_mm) * 1e-3,
        "{heel_clearance} m clear"
    );
}

/// Consistency: **a loading whose weight pushes the wippen past its range
/// names the bottom seat.** The bench of the weight that pushes the wippen
/// past its range, loaded as recorded with its 60 g already on the measuring
/// point: the loading, friction off, takes the key down and leans the
/// wippen on the top of its range, and the refusal names the bottom seat
/// with the depth the sheet refuses it with, to the bit, more than nothing.
/// No state was ever taken: the key is drawn level, unplaced.
#[test]
fn a_loading_whose_weight_pushes_the_wippen_past_its_range_names_the_bottom_seat() {
    let (mut start, profile) =
        recorded(TALL_CAPSTAN_MM * 1e-3, FITS_AT_THE_TOP_SEAT_ONLY_MM * 1e-3);
    start.weight_g = PRESSED_G;
    let b = Instrument::new(start, profile).expect("the bench loads");
    assert_eq!(b.key_place(), KeyPlace::Unplaced);
    let Some(WippenRefusal::PushedPastItsRange {
        at: WhichSeat::Bottom,
        heel_depth,
    }) = b.wippen_refused()
    else {
        panic!("{:?}", b.wippen_refused())
    };
    assert_eq!(
        b.wippen_refused().map(SeatRefusal::from),
        the_sheets_refusal(&b)
    );
    assert!(heel_depth > 0.0, "{heel_depth} m deep");
}

/// Consistency: **a center pin dropped under the key's parts in one step
/// names what the sheet names.** On the test bench, the center pin is set
/// in one step from its own 78.5 mm to each half millimeter from 0 mm, the
/// bottom of its hard range, up to 36 mm, the highest height at which a
/// probe found the bench starting the wippen again. Left at the angle where
/// the last state held it, the wippen would hang free below the key's
/// parts, its heel clear of the key all the way down. The sheet instead
/// lowers it from the top of its range onto the key at the top seat the key
/// takes alone, and finds its heel in the key there. The bench starts the
/// wippen again the same way, so at every height it names the wippen pushed
/// past its range at the top seat, with the sheet's depth to the bit and
/// more than nothing, and never names it out of reach.
#[test]
fn a_center_pin_dropped_under_the_keys_parts_in_one_step_names_what_the_sheet_names() {
    const LEAST_MM: f64 = 0.0;
    const MOST_MM: f64 = 36.0;
    const STEP_MM: f64 = 0.5;
    let steps = ((MOST_MM - LEAST_MM) / STEP_MM).round() as u32;
    for step in 0..=steps {
        let height = LEAST_MM + f64::from(step) * STEP_MM;
        let mut b = Instrument::carrying_the_test_wippen();
        b.set(path::WIPPEN_CENTER_ABOVE, height)
            .expect("a center pin within its hard range");
        let named = b.wippen_refused();
        let sheet = the_sheets_refusal(&b);
        assert!(
            matches!(
                named,
                Some(WippenRefusal::PushedPastItsRange {
                    at: WhichSeat::Top,
                    heel_depth
                }) if heel_depth > 0.0
            ) && named.map(SeatRefusal::from) == sheet,
            "{height} mm: the bench names {named:?}, the sheet {sheet:?}"
        );
    }
}

/// The lifting bench as the bench loads it: on a capstan
/// [`TALL_CAPSTAN_MM`] tall, the center pin a millimeter above the lowest at
/// which the key seats at both seats with its wippen in range
/// ([`Instrument::seated`], what the sheet seats it by), found by halving to
/// a hundredth of a micron ([`read::halve_between`]) between
/// [`FITS_AT_THE_TOP_SEAT_ONLY_MM`] and the test wippen's own 78.5 mm, as
/// the domain's test of a pull finds it: the wippen about a degree under
/// the top of its range at the bottom seat.
fn the_lifting_bench() -> Instrument {
    const ABOVE_THE_LOWEST_FIT: f64 = 0.001;
    let fits = |center: f64| {
        as_recorded(TALL_CAPSTAN_MM * 1e-3, center)
            .expect("the bench loads")
            .seated()
            .is_ok()
    };
    let (_, seated) = read::halve_between(
        FITS_AT_THE_TOP_SEAT_ONLY_MM * 1e-3,
        pa_core::test_fixture::WIPPEN_CENTER.y,
        fits,
    );
    as_recorded(TALL_CAPSTAN_MM * 1e-3, seated + ABOVE_THE_LOWEST_FIT).expect("the bench loads")
}

/// Consistency: **a pull that carries the wippen to its limit is said to
/// lift the key off its frame.** On the lifting bench ([`the_lifting_bench`]) both seats stand
/// the wippen in range. A pull of 100 g on the measuring point lifts the
/// key about its back rail cloth off its balance punching and carries the
/// wippen to the top of its range: the bench draws the key lifted off its
/// frame and says so, and names no refusal of the wippen's.
#[test]
fn a_pull_that_carries_the_wippen_to_its_limit_is_said_to_lift_the_key_off_its_frame() {
    const PULL_G: f64 = -100.0;
    let mut b = the_lifting_bench();
    assert_eq!(the_sheets_refusal(&b), None);
    b.set(path::WEIGHT, PULL_G).expect("a pull");
    assert_eq!(b.key_place(), KeyPlace::LiftedOff);
    assert_eq!(
        b.touch().map(|t| t.verdict),
        Some(TouchVerdict::LiftsOffItsFrame)
    );
    assert_eq!(b.wippen_refused(), None);
}

/// One step of a setting refused at a limit of the wippen's turn: the
/// value set, the refusal, the sheet's, how far past its limit the heel
/// stands, meters, how far the setting stood from the last value at which
/// the sheet decided that limit at that seat, meters, not a number where
/// none did, and whether the bench put the key back by the loading rule.
type AtALimit = (f64, WippenRefusal, Option<SeatRefusal>, f64, f64, bool);

/// The center pin of `b` set to each of `heights`, millimeters, in turn:
/// every step the wippen refuses at a limit of its turn, with how far the
/// center pin stood from the last height at which the sheet decided that
/// limit at that seat, the height `b` stands at first included. The sheet
/// decides a limit at the top seat where its top seat seats: where it
/// refuses nothing, or refuses a limit of the wippen's at the bottom seat;
/// at the bottom seat where it refuses nothing. A height whose sheet stops
/// at the top seat for another reason, a wippen part in a key part, which
/// names no seat, or a refusal of the key's own, decides neither.
fn stepped_to_a_limit(b: &mut Instrument, heights: &[f64]) -> Vec<AtALimit> {
    let mut decided: [Option<f64>; 2] = [None, None];
    let decide =
        |b: &Instrument, height: f64, decided: &mut [Option<f64>; 2]| match the_sheets_refusal(b) {
            None => *decided = [Some(height), Some(height)],
            Some(
                SeatRefusal::WippenPushedPastItsRange {
                    at: WhichSeat::Bottom,
                    ..
                }
                | SeatRefusal::WippenOutOfReach {
                    at: WhichSeat::Bottom,
                    ..
                },
            ) => decided[0] = Some(height),
            Some(_) => {}
        };
    let first = b
        .value(path::WIPPEN_CENTER_ABOVE)
        .expect("a wippen's center");
    decide(b, first, &mut decided);
    let mut refused = Vec::new();
    for &height in heights {
        b.set(path::WIPPEN_CENTER_ABOVE, height)
            .expect("a center pin within its hard range");
        let at_its_limit = match b.wippen_refused() {
            Some(why @ WippenRefusal::PushedPastItsRange { at, heel_depth }) => {
                Some((why, at, heel_depth))
            }
            Some(why @ WippenRefusal::OutOfReach { at, heel_clearance }) => {
                Some((why, at, heel_clearance))
            }
            _ => None,
        };
        if let Some((why, at, distance)) = at_its_limit {
            let reference = match at {
                WhichSeat::Top => decided[0],
                WhichSeat::Bottom => decided[1],
            };
            refused.push((
                height,
                why,
                the_sheets_refusal(b),
                distance,
                reference.map_or(f64::NAN, |reference_height| {
                    (height - reference_height).abs() * 1e-3
                }),
                b.reinitialized(),
            ));
        }
        decide(b, height, &mut decided);
    }
    refused
}

/// The heights from `from` to `to`, millimeters, by the page's step for the
/// center pin, `to` included.
fn heights(from: f64, to: f64) -> Vec<f64> {
    let step = slider(path::WIPPEN_CENTER_ABOVE).step;
    let steps = ((to - from).abs() / step).round() as u32;
    (1..=steps)
        .map(|k| from + (to - from).signum() * f64::from(k) * step)
        .collect()
}

/// Consistency: **every wippen limit a setting meets names the sheet's seat
/// and distance, within how far the setting moved.** The center pin stepped
/// as the page steps it: on the test bench from its own 78.5 mm up to
/// 120 mm, through the top seat's out-of-reach boundary near 107 mm; on a
/// capstan 30 mm tall from 72 mm down to 65 mm, through the top seat's
/// pushed-past boundary near 70 mm; a wippen part in a key part names no
/// seat and is not collected. At every step refused at a limit, the seat and
/// the distance are the sheet's, to the bit, and the distance is more than
/// nothing. Moving the center pin moves every point of the wippen by as
/// much and leaves the key alone's seats where they are, so the distance is
/// no more than how far the center pin stands from the last height at which
/// the sheet decided that limit at that seat ([`stepped_to_a_limit`]):
/// there the heel turned to the top of its range cleared the key, or some
/// whole degree of its turn met it. Both limits are met, and both starts:
/// the wippen where the last state left it, and the key put back by the
/// loading rule.
#[test]
fn every_wippen_limit_a_setting_meets_names_the_sheets_seat_and_distance_within_how_far_the_setting_moved(
) {
    const RAISED_TO_MM: f64 = 120.0;
    const LOWERED_TO_MM: f64 = 65.0;
    let mut test_bench = Instrument::carrying_the_test_wippen();
    let own = test_bench
        .value(path::WIPPEN_CENTER_ABOVE)
        .expect("a wippen's center");
    let mut refused = stepped_to_a_limit(&mut test_bench, &heights(own, RAISED_TO_MM));
    let mut tall = Instrument::carrying_the_test_wippen();
    tall.set(path::CAPSTAN_HEIGHT, TALL_CAPSTAN_MM)
        .expect("a tall capstan");
    tall.set(path::WIPPEN_CENTER_ABOVE, FITS_AT_THE_TOP_SEAT_ONLY_MM)
        .expect("a center pin within its hard range");
    refused.extend(stepped_to_a_limit(
        &mut tall,
        &heights(FITS_AT_THE_TOP_SEAT_ONLY_MM, LOWERED_TO_MM),
    ));
    for (height, why, sheet, distance, within, _) in &refused {
        assert_eq!(Some(SeatRefusal::from(*why)), *sheet, "{height} mm");
        assert!(
            *distance > 0.0 && distance <= within,
            "{height} mm: {distance} m past its limit, within {within} m"
        );
    }
    let met = |limit: fn(&WippenRefusal) -> bool| refused.iter().any(|step| limit(&step.1));
    assert!(
        met(|why| matches!(why, WippenRefusal::PushedPastItsRange { .. }))
            && met(|why| matches!(why, WippenRefusal::OutOfReach { .. })),
        "{refused:?}"
    );
    let restarted: Vec<bool> = refused.iter().map(|step| step.5).collect();
    assert!(
        restarted.contains(&true) && restarted.contains(&false),
        "{restarted:?}"
    );
}

/// Consistency: **every weight that pushes the wippen past its range names
/// the sheet's bottom seat and depth.** On a capstan 30 mm tall under a
/// center pin 72 mm high the weight stepped by the gram from nothing to the
/// most the page offers, each an increment from where the last left the
/// key: at every step the wippen refuses at a limit of its turn, the
/// refusal is the sheet's, at the bottom seat, to the bit, and its depth
/// more than nothing. Some step does.
#[test]
fn every_weight_that_pushes_the_wippen_past_its_range_names_the_sheets_bottom_seat_and_depth() {
    let most = slider(path::WEIGHT).max.expect("bounded");
    let mut b = as_recorded(TALL_CAPSTAN_MM * 1e-3, FITS_AT_THE_TOP_SEAT_ONLY_MM * 1e-3)
        .expect("the bench loads");
    let sheet = the_sheets_refusal(&b);
    let mut met = 0;
    let mut grams = 0.0;
    while grams <= most {
        b.set(path::WEIGHT, grams).expect("a weight");
        if let Some(why @ WippenRefusal::PushedPastItsRange { heel_depth, .. }) = b.wippen_refused()
        {
            met += 1;
            assert_eq!(Some(SeatRefusal::from(why)), sheet, "{grams} g");
            assert!(heel_depth > 0.0, "{grams} g: {heel_depth} m deep");
        }
        grams += 1.0;
    }
    assert!(
        matches!(
            sheet,
            Some(SeatRefusal::WippenPushedPastItsRange {
                at: WhichSeat::Bottom,
                ..
            })
        ),
        "{sheet:?}"
    );
    assert!(met > 0, "no weight pushed the wippen past its range");
}

/// Consistency: **a wippen part standing in a key part after an increment
/// is drawn faded and named.** The center pin lowered to 55 mm above the
/// keybed, within its hard range, the increment comes to rest with the
/// wippen's body standing in the back check block: no state is taken, the
/// key and its wippen are drawn where the last one left them, faded, and
/// the refusal names the body, the block and how deep, two to three
/// millimeters.
#[test]
fn a_wippen_part_standing_in_a_key_part_after_an_increment_is_drawn_faded_and_named() {
    let mut b = Instrument::carrying_the_test_wippen();
    let before = b.poses();
    b.set(path::WIPPEN_CENTER_ABOVE, 55.0)
        .expect("a center pin within its hard range");
    assert!(
        !b.reinitialized(),
        "the increment started where the key stood"
    );
    assert_eq!(b.key_place(), KeyPlace::WippenRefused);
    match b.wippen_refused() {
        Some(WippenRefusal::InTheKey {
            part: PartCode::Wippen,
            key_part: PartCode::BackCheckBlock,
            depth,
        }) => assert!(depth > 0.002 && depth < 0.003, "{depth} m deep"),
        other => panic!("{other:?}"),
    }
    assert_eq!(b.poses(), before, "drawn where the last state left it");
}

/// Consistency: **a wippen path on a key that carries none is refused and
/// leaves the instrument as it was**: the measured section carries no
/// wippen, and a heel's position set on it is refused naming the path, its
/// state hash, its settings and its sheet unmoved, the refusal kept.
#[test]
fn a_wippen_path_on_a_key_carrying_none_is_refused_and_leaves_the_instrument_as_it_was() {
    let mut b = Instrument::weighed_section();
    let (hash, settings, sheet) = (
        b.state_hash(),
        crate::boundary::json(&b.settings()),
        b.sheet(),
    );
    assert_eq!(
        b.set(path::HEEL_POSITION, 60.0),
        Err(EditRefusal::NoWippenAssembly {
            path: path::HEEL_POSITION
        })
    );
    assert_eq!(b.state_hash(), hash);
    assert_eq!(crate::boundary::json(&b.settings()), settings);
    assert_eq!(b.sheet(), sheet);
    assert_eq!(
        b.refused_edit().map(|r| &r.why),
        Some(&EditRefusal::NoWippenAssembly {
            path: path::HEEL_POSITION
        })
    );
}

/// Consistency: **a wippen no part could be built as is refused and leaves
/// the instrument as it was**: a heel cloth of no thickness, refused by the
/// validation layer, naming it.
#[test]
fn a_wippen_no_part_could_be_built_as_is_refused_and_leaves_the_instrument_as_it_was() {
    let mut b = Instrument::carrying_the_test_wippen();
    let hash = b.state_hash();
    assert_eq!(
        b.set(path::HEEL_CLOTH_THICKNESS, 0.0),
        Err(EditRefusal::Problem(Problem::NoThickness {
            part: PartCode::WippenHeelCloth
        }))
    );
    assert_eq!(b.state_hash(), hash);
}

/// Consistency: **the log replayed with the wippen's edits reproduces the
/// state**, its hash to the bit: the wippen's center moved along the key,
/// the heel's position and the center's friction set, a weight laid on.
#[test]
fn the_log_replayed_with_the_wippens_edits_reproduces_the_state() {
    let mut b = Instrument::carrying_the_test_wippen();
    b.set(path::WIPPEN_CENTER_BEHIND, 183.0).expect("a center");
    b.set(path::HEEL_POSITION, 61.0).expect("a heel");
    b.set(path::WIPPEN_FRICTION_TORQUE, 55.0)
        .expect("a friction");
    b.set(path::WEIGHT, 20.0).expect("a weight");
    let again = Instrument::carrying_the_test_wippen()
        .replay(b.log())
        .expect("the log replays");
    assert_eq!(again.state_hash(), b.state_hash());
    assert_ne!(
        Instrument::carrying_the_test_wippen().state_hash(),
        b.state_hash()
    );
}

/// Consistency: **the wippen center's friction set by hand rests on the
/// hand**, as the bushings' does: its basis, the profile's until then, is
/// set by hand once a friction is typed.
#[test]
fn the_wippen_centers_friction_set_by_hand_rests_on_the_hand() {
    let mut b = Instrument::carrying_the_test_wippen();
    let friction = |b: &Instrument| b.wippen_bases.map(|bases| bases.friction);
    let before = friction(&b);
    b.set(path::WIPPEN_FRICTION_TORQUE, 55.0)
        .expect("a friction");
    assert_eq!(
        (before, friction(&b)),
        (Some(Basis::Published), Some(Basis::SetByHand))
    );
}

//! The key as parts: one outline for each piece of material, in the key's
//! frame.
//!
//! The keystick is one thickness of wood. Glued to it: the keytop, whose lip
//! overhangs the key front; the key front over the end grain; the key button
//! where the balance mortise is cut; the back check block; the hard felt on
//! the thinned tail. Screwed into it: the capstan, a turned part. Cut out of
//! it: the front mortise, drawn as the void it is.
//!
//! Each piece is its own part with its own material, so the keytop and the key
//! front can be drawn white and the two glued blocks in their own wood: what
//! a part is made of is the model's, the colour the view's. The key's mass is
//! not integrated over these outlines; it comes from the volume model
//! ([`KeyDesign::mass_properties`]), whose polygon these parts tile exactly.
//!
//! # The underside, and the mortise in it
//!
//! The keystick's underside is one straight chain from the end of its chamfer
//! to its front, split at the front mortise's edges into collinear segments.
//! The key bears on the front punching beside the slot, out of the sagittal
//! plane, so the chain runs on across the slot; the split is what lets a pin
//! read the wood beside the slot and not the slot itself. A straight vertex
//! has no exterior angle, so it is no feature for a contact to catch on.
//!
//! The balance mortise is not drawn: none of its dimensions is measured, and
//! the balance pin's height waits on it (`data/params/key-frame-sections.json`).

use super::{count, next_id, polygon, KeyDesign};
use crate::contact::{
    Anchor, AnchorCode, Material, Part, PartCode, PartId, Provenance, Role, SurfaceRef, Sweep,
};
use crate::geometry::{Point, Seg};
use crate::mechanism::BodyId;

/// The section the key's shape is measured in.
const SECTION: &str = "data/params/key-section.json";
/// The frame's measured sections, whose notes hold the front mortise's depth.
const FRAME_SECTIONS: &str = "data/params/key-frame-sections.json";
/// Where the values that have no record are listed.
const UNRECORDED: &str = "data/params/README.md";

/// What the key's parts declare: where each is, and the ranges and points of
/// them that anything else may read.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyRanges {
    /// The keystick.
    pub keystick: PartId,
    /// The keytop.
    pub keytop: PartId,
    /// The key front.
    pub key_front: PartId,
    /// The key button.
    pub key_button: PartId,
    /// The back check block.
    pub back_check_block: PartId,
    /// The felt on the key's tail.
    pub key_tail_felt: PartId,
    /// The capstan.
    pub capstan: PartId,
    /// The front mortise, a cut.
    pub front_mortise: PartId,
    /// The underside, from the tail forward: the keystick's chamfer and flat,
    /// the flat split at the front mortise's edges, then the key front's
    /// bottom. The first segment of the first range is the chamfer. What the
    /// back rail cloth reads.
    pub underside: [SurfaceRef; 2],
    /// The underside's flat, from where the chamfer runs out to the front of
    /// the key, the slot's mouth included: the key bears on a punching beside
    /// the slot, out of the sagittal plane. What the punchings read.
    pub underside_flat: [SurfaceRef; 2],
    /// The wood of the flat beside the front mortise: the flat less the
    /// slot's mouth, behind the slot and in front of it. What a front rail
    /// pin standing outside its mortise but under the key reads.
    pub beside_the_mortise: Vec<SurfaceRef>,
    /// The front mortise's top, facing down onto its pin.
    pub front_mortise_top: SurfaceRef,
    /// The top of the key at its key front.
    pub key_front_top: Anchor,
    /// The covering's lip.
    pub lip: Anchor,
    /// The top of the capstan on its axis.
    pub capstan_contact: Anchor,
}

impl KeyDesign {
    /// Builds every part of the key onto `out`, carried by `body`, in the
    /// key's frame: origin at the balance pin on the underside, `x` towards
    /// the player. Returns what they declare.
    ///
    /// Built from the same fields as [`Self::underside_corners`], which the
    /// stops read, and as the volume model's [`Self::outline_vertices`], with
    /// the same arithmetic, so the points the three share are the same bits.
    pub fn parts(&self, body: BodyId, out: &mut Vec<Part>) -> KeyRanges {
        let top = self.stick_height;
        let wood_front = self.front_edge - self.front_board;
        let lip = self.lip();
        let keytop_back = lip - self.keytop_length;
        let covered = top + self.keytop_thickness;
        let button_front = self.capsule_length * 0.5;
        let button_back = -self.capsule_length * 0.5;
        let tail_front = self.tail_end + self.tail_length;
        let block_front = tail_front + self.back_check_block_length;
        let kink = self.tail_end + self.tail_rise_run;
        let (mortise_back, mortise_front) = self.front_mortise_span();
        let mut push = |part: Part| {
            let id = next_id(out);
            out.push(part);
            id
        };
        let solid = |code, outline: Vec<Seg>, material, provenance| Part {
            body,
            code,
            sources: vec![(0..count(&outline), provenance)],
            outline,
            sweep: Sweep::Extruded { width: self.width },
            role: Role::Solid,
            material,
        };

        // The keystick: the underside from the tail forward, split at the
        // mortise's edges where they fall on the flat, then the end grain the
        // key front covers, the top, and the tail cut down for the damper.
        let mut underside = vec![
            Point::new(self.tail_end, self.tail_rise),
            Point::new(kink, 0.0),
        ];
        for edge in [mortise_back, mortise_front] {
            if edge > kink && edge < wood_front && underside.iter().all(|p| p.x < edge) {
                underside.push(Point::new(edge, 0.0));
            }
        }
        underside.push(Point::new(wood_front, 0.0));
        let underside_segments = u16::try_from(underside.len() - 1).expect("a short chain");
        // The flat's segments that are wood and not the slot's mouth, as runs
        // of consecutive segments: whatever of the flat lies over the slot is
        // not under the key's wood.
        let wood = |x0: f64, x1: f64| {
            let middle = 0.5 * (x0 + x1);
            middle < mortise_back || middle > mortise_front
        };
        let mut beside_runs: Vec<core::ops::Range<u16>> = Vec::new();
        for k in 1..underside_segments {
            let i = usize::from(k);
            if !wood(underside[i].x, underside[i + 1].x) {
                continue;
            }
            match beside_runs.last_mut() {
                Some(run) if run.end == k => run.end = k + 1,
                _ => beside_runs.push(k..k + 1),
            }
        }
        let mut stick = underside;
        stick.extend([
            Point::new(wood_front, top),
            Point::new(tail_front, top),
            Point::new(tail_front, self.tail_height),
            Point::new(self.tail_end, self.tail_height),
        ]);
        let keystick = push(solid(
            PartCode::Keystick,
            polygon(&stick),
            Material::KeyWood,
            Provenance::Measured(SECTION),
        ));

        let key_front = push(solid(
            PartCode::KeyFront,
            polygon(&[
                Point::new(wood_front, 0.0),
                Point::new(self.front_edge, 0.0),
                Point::new(self.front_edge, top),
                Point::new(wood_front, top),
            ]),
            Material::KeyFront,
            Provenance::Measured(SECTION),
        ));

        // The keytop, its top split where the key front stands under it: the
        // top of the key at its key front is a vertex, not a point found along
        // a segment.
        let mut keytop_points = vec![
            Point::new(keytop_back, top),
            Point::new(lip, top),
            Point::new(lip, covered),
        ];
        let key_front_top_seg = if self.front_edge < lip && self.front_edge > keytop_back {
            keytop_points.push(Point::new(self.front_edge, covered));
            3
        } else {
            2
        };
        keytop_points.push(Point::new(keytop_back, covered));
        let keytop = push(solid(
            PartCode::Keytop,
            polygon(&keytop_points),
            Material::Covering,
            Provenance::Measured(SECTION),
        ));

        let key_button = push(solid(
            PartCode::KeyButton,
            polygon(&[
                Point::new(button_back, top),
                Point::new(button_front, top),
                Point::new(button_front, top + self.capsule_height),
                Point::new(button_back, top + self.capsule_height),
            ]),
            Material::BlockWood,
            Provenance::Measured(SECTION),
        ));

        let back_check_block = push(solid(
            PartCode::BackCheckBlock,
            polygon(&[
                Point::new(tail_front, top),
                Point::new(block_front, top),
                Point::new(block_front, top + self.back_check_block_height),
                Point::new(tail_front, top + self.back_check_block_height),
            ]),
            Material::BlockWood,
            Provenance::Measured(SECTION),
        ));

        let felt_outline = polygon(&[
            Point::new(self.tail_end, self.tail_height),
            Point::new(tail_front, self.tail_height),
            Point::new(tail_front, self.tail_height + self.damper_felt),
            Point::new(self.tail_end, self.tail_height + self.damper_felt),
        ]);
        let key_tail_felt = push(Part {
            role: Role::Lining,
            ..solid(
                PartCode::KeyTailFelt,
                felt_outline,
                Material::HardFelt,
                Provenance::Measured(SECTION),
            )
        });

        // The capstan, turned from the profile the volume model turns: from
        // the point of the shaft up to the middle of the head's flat top, then
        // down the axis.
        let profile: Vec<Point> = self
            .capstan_profile()
            .into_iter()
            .map(|(r, y)| Point::new(r, y))
            .collect();
        let capstan_outline = polygon(&profile);
        let capstan_segments = count(&capstan_outline);
        let capstan = push(Part {
            body,
            code: PartCode::Capstan,
            sources: vec![(0..capstan_segments, Provenance::Traced(UNRECORDED))],
            outline: capstan_outline,
            sweep: Sweep::Revolved {
                axis_x: self.capstan,
            },
            role: Role::Solid,
            material: Material::Steel,
        });

        // The front mortise, cut up from the underside. How wide it is across
        // the keyboard is not a dimension the model has: the slot is bushed
        // and close on the pin's flats, out of the sagittal plane. Its length
        // along the key is assumed; its depth, its two walls, is measured.
        let mortise_outline = polygon(&[
            Point::new(mortise_back, 0.0),
            Point::new(mortise_front, 0.0),
            Point::new(mortise_front, self.front_mortise_depth),
            Point::new(mortise_back, self.front_mortise_depth),
        ]);
        let front_mortise = push(Part {
            body,
            code: PartCode::FrontMortise,
            sources: vec![
                (0..1, Provenance::Assumed(SECTION)),
                (1..2, Provenance::Measured(FRAME_SECTIONS)),
                (2..3, Provenance::Assumed(SECTION)),
                (3..4, Provenance::Measured(FRAME_SECTIONS)),
            ],
            outline: mortise_outline,
            sweep: Sweep::Extruded { width: 0.0 },
            role: Role::Cut,
            material: Material::KeyWood,
        });

        let front_bottom = SurfaceRef {
            part: key_front,
            segs: 0..1,
        };
        let mut beside_the_mortise: Vec<SurfaceRef> = beside_runs
            .into_iter()
            .map(|segs| SurfaceRef {
                part: keystick,
                segs,
            })
            .collect();
        if wood(wood_front, self.front_edge) {
            beside_the_mortise.push(front_bottom.clone());
        }
        KeyRanges {
            keystick,
            keytop,
            key_front,
            key_button,
            back_check_block,
            key_tail_felt,
            capstan,
            front_mortise,
            underside: [
                SurfaceRef {
                    part: keystick,
                    segs: 0..underside_segments,
                },
                front_bottom.clone(),
            ],
            underside_flat: [
                SurfaceRef {
                    part: keystick,
                    segs: 1..underside_segments,
                },
                front_bottom,
            ],
            beside_the_mortise,
            front_mortise_top: SurfaceRef {
                part: front_mortise,
                segs: 2..3,
            },
            key_front_top: Anchor {
                part: keytop,
                seg: key_front_top_seg,
                at: 0.0,
                code: AnchorCode::KeyFrontTop,
            },
            lip: Anchor {
                part: keytop,
                seg: 2,
                at: 0.0,
                code: AnchorCode::Lip,
            },
            // The silhouette's top runs across the axis: its two segments
            // either side are `n − 2` and `n − 1`, and the second starts on it.
            capstan_contact: Anchor {
                part: capstan,
                seg: capstan_segments - 1,
                at: 0.0,
                code: AnchorCode::CapstanContact,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parts::frame::FrameDesign;
    use crate::parts::polygon_mass_properties;

    const KEY: BodyId = BodyId(1);

    fn built(key: &KeyDesign) -> (Vec<Part>, KeyRanges) {
        let mut parts = Vec::new();
        let ranges = key.parts(KEY, &mut parts);
        (parts, ranges)
    }

    /// The polygon a closed outline of lines runs through.
    fn vertices(part: &Part) -> Vec<Point> {
        part.outline.iter().map(Seg::start).collect()
    }

    /// Twice the signed area of a polygon, positive when counter-clockwise.
    fn twice_area(points: &[Point]) -> f64 {
        (0..points.len())
            .map(|i| {
                let (p, q) = (points[i], points[(i + 1) % points.len()]);
                p.x * q.y - q.x * p.y
            })
            .sum()
    }

    fn same(p: Point, q: Point) -> bool {
        p.x.to_bits() == q.x.to_bits() && p.y.to_bits() == q.y.to_bits()
    }

    /// Consistency: every part of the key is closed bit for bit and turns
    /// counter-clockwise, the cut included, and every range and point it
    /// declares lies inside its outline.
    #[test]
    fn every_part_of_the_key_is_closed_and_counter_clockwise() {
        for key in [
            KeyDesign::default(),
            KeyDesign {
                front_edge: 0.180,
                ..KeyDesign::default()
            },
        ] {
            let (parts, ranges) = built(&key);
            assert_eq!(parts.len(), 8);
            for part in &parts {
                assert!(part.is_closed(), "{:?} is open", part.code);
                assert_eq!(part.body, KEY);
                // A turned part's profile turns counter-clockwise in radius
                // and height, a prism's in the body's x and y.
                assert!(
                    twice_area(&vertices(part)) > 0.0,
                    "{:?} turns clockwise",
                    part.code
                );
                let covered: u16 = part.sources.iter().map(|(r, _)| r.end - r.start).sum();
                assert_eq!(usize::from(covered), part.outline.len(), "{:?}", part.code);
            }
            for range in ranges.underside.iter().chain([&ranges.front_mortise_top]) {
                let silhouette = parts[range.part.index()].silhouette();
                assert!(usize::from(range.segs.end) <= silhouette.len());
            }
        }
    }

    /// Consistency: the drawn key tiles the volume model's polygon, so the key
    /// a reader sees is the key the composite key's mass is integrated over.
    /// The capstan is turned, and in neither; the mortise is a cut, which the
    /// volume model does not take out.
    ///
    /// The bound is derived: each shoelace sum rounds once per vertex, on
    /// terms no larger than the reach squared, and there are under 40 of them.
    #[test]
    fn the_drawn_key_tiles_the_polygon_its_mass_is_integrated_over() {
        let key = KeyDesign::default();
        let (parts, _) = built(&key);
        let drawn: f64 = parts
            .iter()
            .filter(|p| p.role != Role::Cut && matches!(p.sweep, Sweep::Extruded { .. }))
            .map(|p| twice_area(&vertices(p)))
            .sum();
        let volume = twice_area(&key.outline_vertices());
        let reach = key.front_edge - key.tail_end;
        assert!(
            (drawn - volume).abs() <= 40.0 * f64::EPSILON * reach * reach,
            "the parts enclose {drawn} m², the volume model {volume} m²"
        );
        let mass = polygon_mass_properties(&key.outline_vertices(), key.width, key.density);
        assert!(mass.is_some(), "and that polygon weighs something");
    }

    /// The underside is the chamfer, then one straight flat to the front of
    /// the key, split at the mortise's edges: the corners the stops read are
    /// its vertices, bit for bit, and the mortise's bottom is a segment of it.
    #[test]
    fn the_underside_is_split_at_the_mortise_edges_and_runs_on_across_the_slot() {
        let key = KeyDesign::default();
        let (parts, ranges) = built(&key);
        let stick = ranges.underside[0].segs(&parts);
        let front = ranges.underside[1].segs(&parts);
        assert_eq!(stick.len(), 4, "chamfer, flat, the slot's mouth, flat");
        let [tail, kink, nose] = key.underside_corners();
        assert!(same(stick[0].start(), tail));
        assert!(same(stick[0].end(), kink));
        assert!(same(front[0].end(), nose));
        let chain: Vec<Seg> = stick.iter().chain(&front).copied().collect();
        for pair in chain.windows(2) {
            assert!(same(pair[0].end(), pair[1].start()), "the chain breaks");
        }
        for seg in &chain[1..] {
            assert_eq!(
                seg.start().y.to_bits(),
                0.0_f64.to_bits(),
                "{seg:?} is off the flat"
            );
        }
        let (back, forward) = key.front_mortise_span();
        let mortise = &parts[ranges.front_mortise.index()];
        assert_eq!(mortise.role, Role::Cut);
        assert!(same(stick[2].start(), mortise.outline[0].start()));
        assert!(same(stick[2].end(), mortise.outline[0].end()));
        assert_eq!(stick[2].start().x.to_bits(), back.to_bits());
        assert_eq!(stick[2].end().x.to_bits(), forward.to_bits());
        let top = ranges.front_mortise_top.segs(&parts)[0];
        assert_eq!(top.start().y.to_bits(), key.front_mortise_depth.to_bits());
        assert!(
            top.start().x > top.end().x,
            "a cut's top runs back over its void"
        );
        // And both bearings fall on the flat: the balance punching at the pin,
        // the front punching under the mortise.
        for bearing in [0.0, key.front_mortise_x()] {
            assert!(
                bearing > kink.x && bearing < nose.x,
                "the bearing at {bearing} m is off the flat"
            );
        }
    }

    /// The flat the punchings read runs from where the chamfer ends to the
    /// front of the key, across the slot's mouth; the wood beside the mortise
    /// is that flat less the mouth, behind the slot and in front of it, the
    /// key front's bottom included. Moved off the flat, the mortise leaves
    /// the whole flat beside it.
    #[test]
    fn the_flat_and_the_wood_beside_the_mortise_are_declared() {
        let length = |segs: &[Seg]| -> f64 { segs.iter().map(|s| s.end().x - s.start().x).sum() };
        let key = KeyDesign::default();
        let (parts, ranges) = built(&key);
        let [_, kink, nose] = key.underside_corners();
        let flat: Vec<Seg> = ranges
            .underside_flat
            .iter()
            .flat_map(|r| r.segs(&parts))
            .collect();
        assert!(same(flat[0].start(), kink));
        assert!(same(flat[flat.len() - 1].end(), nose));
        let (back, front) = key.front_mortise_span();
        let beside: Vec<Seg> = ranges
            .beside_the_mortise
            .iter()
            .flat_map(|r| r.segs(&parts))
            .collect();
        for seg in &beside {
            assert!(
                seg.end().x <= back || seg.start().x >= front,
                "{seg:?} runs over the slot"
            );
        }
        assert_eq!(
            ranges.beside_the_mortise.len(),
            3,
            "behind, in front, the key front"
        );
        let mouth = front - back;
        assert!((length(&flat) - length(&beside) - mouth).abs() < 1e-15);

        // A mortise cut forward of the key's flat, out of reach: nothing of
        // the flat is over it.
        let odd = KeyDesign {
            front_mortise_from_front_edge: -0.010,
            ..KeyDesign::default()
        };
        let (parts, ranges) = built(&odd);
        let flat: Vec<Seg> = ranges
            .underside_flat
            .iter()
            .flat_map(|r| r.segs(&parts))
            .collect();
        let beside: Vec<Seg> = ranges
            .beside_the_mortise
            .iter()
            .flat_map(|r| r.segs(&parts))
            .collect();
        assert_eq!(flat, beside);
    }

    /// The named points stand where the protocols read them, to the bit: the
    /// lip every touchweight is taken back from, the key front's top where key
    /// height and dip are read, the capstan's top where the jig centres 10 g.
    #[test]
    fn the_named_points_are_where_the_protocols_read() {
        let key = KeyDesign::default();
        let (parts, ranges) = built(&key);
        let covered = key.stick_height + key.keytop_thickness;
        assert!(same(
            ranges.lip.point(&parts),
            Point::new(key.lip(), covered)
        ));
        assert!(same(
            ranges.key_front_top.point(&parts),
            Point::new(key.front_edge, covered)
        ));
        assert!(same(
            ranges.capstan_contact.point(&parts),
            key.capstan_contact()
        ));
    }

    /// The owner's materials: the keytop and the key front each their own
    /// material, both drawn white; the key button and the back check block of
    /// one wood, which is not the keystick's.
    #[test]
    fn the_glued_blocks_are_another_wood_and_the_front_its_own_material() {
        let (parts, r) = built(&KeyDesign::default());
        let material = |id: PartId| parts[id.index()].material;
        assert_eq!(material(r.keytop), Material::Covering);
        assert_eq!(material(r.key_front), Material::KeyFront);
        assert_eq!(material(r.key_button), material(r.back_check_block));
        assert_ne!(material(r.key_button), material(r.keystick));
        assert_eq!(material(r.keystick), Material::KeyWood);
    }

    /// The keystick is one thickness of wood with its parts on it: every glued
    /// part stands on its top, the block for the back check is the highest
    /// wood on the key, and the felt the damper bears on sits on the thinned
    /// tail.
    #[test]
    fn the_keystick_is_one_thickness_with_its_parts_on_it() {
        let key = KeyDesign::default();
        let (parts, r) = built(&key);
        let lowest = |id: PartId| {
            vertices(&parts[id.index()])
                .iter()
                .fold(f64::MAX, |a, p| a.min(p.y))
        };
        let highest = |id: PartId| {
            vertices(&parts[id.index()])
                .iter()
                .fold(f64::MIN, |a, p| a.max(p.y))
        };
        assert_eq!(highest(r.keystick).to_bits(), key.stick_height.to_bits());
        for glued in [r.keytop, r.key_button, r.back_check_block] {
            assert_eq!(lowest(glued).to_bits(), key.stick_height.to_bits());
        }
        let wood = [r.keystick, r.key_button, r.back_check_block];
        let top_of_the_wood = wood.iter().map(|&id| highest(id)).fold(f64::MIN, f64::max);
        assert_eq!(
            top_of_the_wood.to_bits(),
            highest(r.back_check_block).to_bits()
        );
        assert_eq!(lowest(r.key_tail_felt).to_bits(), key.tail_height.to_bits());
        assert!(key.tail_height < key.stick_height);
    }

    /// The capstan screws straight into the keystick: nothing glued under it,
    /// its seat the stick's own top, and its drawn head at the height the
    /// technician sets above that seat.
    #[test]
    fn the_capstan_screws_into_the_stick_itself() {
        let key = KeyDesign::default();
        let (parts, r) = built(&key);
        assert_eq!(key.capstan_seat().to_bits(), key.stick_height.to_bits());
        let head = vertices(&parts[r.capstan.index()])
            .iter()
            .fold(f64::MIN, |a, p| a.max(p.y));
        assert_eq!(
            head.to_bits(),
            (key.capstan_seat() + key.capstan_screw.height).to_bits()
        );
        let half = 0.5 * key.capstan_screw.head_diameter;
        for id in [r.key_button, r.back_check_block, r.keytop] {
            let part = &parts[id.index()];
            let (lo, hi) = vertices(part)
                .iter()
                .fold((f64::MAX, f64::MIN), |(lo, hi), p| {
                    (lo.min(p.x), hi.max(p.x))
                });
            assert!(
                hi < key.capstan - half || lo > key.capstan + half,
                "{:?} stands under the capstan",
                part.code
            );
        }
        let capstan = &parts[r.capstan.index()];
        assert_eq!(
            capstan.sweep,
            Sweep::Revolved {
                axis_x: key.capstan
            }
        );
        assert_eq!(capstan.material, Material::Steel);
    }

    /// The back rail the placement rule builds bears on the chamfer, rail
    /// width included: the drawn chamfer runs from the tail to where the flat
    /// begins, and the rule is free to put the cloth anywhere but there.
    #[test]
    fn the_back_rail_placed_for_the_key_falls_on_the_drawn_chamfer() {
        let key = KeyDesign::default();
        let (parts, r) = built(&key);
        let chamfer = r.underside[0].segs(&parts)[0];
        let placed = FrameDesign::default().placed_under(&key);
        let half = placed.sections.back.half_length;
        assert!(placed.back_rail - half > chamfer.start().x);
        assert!(placed.back_rail + half < chamfer.end().x);
        assert!(
            chamfer.start().y > chamfer.end().y,
            "the chamfer rises towards the tail"
        );
    }
}

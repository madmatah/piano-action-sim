//! The key as parts: one outline for each piece of material, in the key's
//! frame.
//!
//! The keystick is one thickness of wood, cut down to its thinned tail at the
//! back, and its top lowered between the capstan and the back check block
//! where it has a clearance for the wippen rail. Glued to it: the keytop,
//! whose lip overhangs the key front; the key front over the end grain; the
//! key button, its ends rounded, where the balance mortise is cut; the back
//! check block; the hard felt on the thinned tail, where it has one. Each
//! glued block stands on the wood under it, the keystick's full height or the
//! bare tail. Screwed into its full height: the capstan, a turned part, its
//! head flat or domed. Cut out of it: the front mortise, and the balance
//! mortise through the key button with its cavity and the balance hole under
//! it, each drawn as the void it is; and beside the sagittal plane, which
//! does not cut them, the key button's side holes, through its wall on each
//! side of the slot. Set into the balance mortise's two long sides, also
//! beside the plane, its bushing, an L on each side: down the slot's wall,
//! then out through the side hole to the key button's side.
//!
//! Each piece is its own part with its own material, so the keytop and the key
//! front can be drawn white and the glued blocks in their own wood: what a
//! part is made of is the model's, the color the view's. Each stretch of each
//! outline names the number of the key that places it ([`PlacedBy`]), so an
//! application can say how that number is known. The key's mass is not
//! integrated over these outlines; it is weighed or published. Their area
//! centroid in the section, the mortises and the clearance taken out, and
//! what stands beside the plane left out, stands in for the height of a
//! weighed key's center of mass, which no scale gives
//! ([`KeyDesign::section_centroid`]).
//!
//! # The underside, and the mouths in it
//!
//! The keystick's underside is one straight chain from the end of its chamfer
//! to its front, split at the balance hole's edges and the front mortise's
//! into collinear segments. The key bears on each punching beside its hole or
//! its slot, out of the sagittal plane, so the chain runs on across them; the
//! split is what lets a pin read the wood beside the slot and not the slot
//! itself, and what gives each mouth a segment of its own, which the view
//! leaves open. A straight vertex has no exterior angle, so it is no feature
//! for a contact to catch on. The key button's flat top is split the same
//! way at the slot's ends.

use super::{count, next_id, polygon, rounded_ends, GluedBlock, KeyDesign};
use crate::contact::{
    Anchor, AnchorCode, Material, Part, PartCode, PartId, PlacedBy, Role, SurfaceRef, Sweep,
};
use crate::geometry::{Dir, Point, Seg};
use crate::mechanism::BodyId;

/// The numbers that place a glued block's four sides, from its bottom round
/// counter-clockwise: the wood it stands on, its front, its top, its back.
type BlockNames = [PlacedBy; 4];

/// The number that places each run of an outline's segments.
type Placements = Vec<(core::ops::Range<u16>, PlacedBy)>;

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
    /// The felt on the key's tail, where it has one.
    pub key_tail_felt: Option<PartId>,
    /// The capstan.
    pub capstan: PartId,
    /// The front mortise, a cut.
    pub front_mortise: PartId,
    /// The balance mortise, a cut: the slot through the key button and the
    /// cavity under it.
    pub balance_mortise: PartId,
    /// The balance hole, a cut through the floor of that cavity.
    pub balance_hole: PartId,
    /// The side hole through the key button's wall on each side of the
    /// slot, on its treble side and on its bass side.
    pub key_button_side_holes: [PartId; 2],
    /// The bushing on each long side of the slot, each an L: its leg down
    /// the slot's wall, on the treble side and on the bass side
    /// ([`Sweep::Extruded`]'s `across`).
    pub balance_mortise_bushings_down_the_walls: [PartId; 2],
    /// The bushing's other leg, out through each side hole to the key
    /// button's side, on the treble side and on the bass side.
    pub balance_mortise_bushings_out_through_the_sides: [PartId; 2],
    /// The underside, from the tail forward: the keystick's chamfer and flat,
    /// the flat split at the balance hole's edges and the front mortise's,
    /// then the key front's
    /// bottom. The first segment of the first range is the chamfer. What the
    /// back rail cloth reads.
    pub underside: [SurfaceRef; 2],
    /// The underside's flat, from where the chamfer runs out to the front of
    /// the key, the mouths of the slot and of the balance hole included: the
    /// key bears on a punching beside them, out of the sagittal plane. What
    /// the punchings read.
    pub underside_flat: [SurfaceRef; 2],
    /// The wood round the front mortise: the flat less the slot's mouth and
    /// the balance hole's, behind the slot and in front of it, each run
    /// chained at the slot's
    /// mouth to the slot's wall there, which the mortise's cut draws. What a
    /// front rail pin reads that stands outside its mortise but under the
    /// key, that straddles the slot's edge, or that a turning key brings
    /// against the end of its slot: the corner of the wood at the mouth is
    /// convex, and a feature.
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
    /// key's frame: origin at the balance pin on the underside, `x` toward
    /// the player. Returns what they declare.
    ///
    /// Built from the same fields as the key's section as one polygon, with
    /// the same arithmetic, so the points the two share are the same bits
    /// and the parts tile it (`the_drawn_key_tiles_its_section`).
    pub fn parts(&self, body: BodyId, out: &mut Vec<Part>) -> KeyRanges {
        let top = self.stick_height;
        let wood_front = self.front_edge - self.front_board;
        let lip = self.lip();
        let keytop_back = lip - self.keytop_length;
        let covered = top + self.keytop_thickness;
        let tail_front = self.tail_front();
        let kink = self.tail_end + self.tail_rise_run;
        let (mortise_back, mortise_front) = self.front_mortise_span();
        let (hole_back, hole_front) = self.balance_hole_span();
        let mut push = |part: Part| {
            let id = next_id(out);
            out.push(part);
            id
        };
        let solid = |code,
                     outline: Vec<Seg>,
                     material,
                     placed_by: Vec<(core::ops::Range<u16>, PlacedBy)>| Part {
            body,
            code,
            placed_by,
            outline,
            sweep: Sweep::Extruded {
                width: self.width,
                across: 0.0,
            },
            role: Role::Solid,
            material,
        };
        // A rectangle's four sides, from its bottom round counter-clockwise,
        // each placed by its own number.
        let sides = |names: BlockNames| -> Vec<(core::ops::Range<u16>, PlacedBy)> {
            (0u16..).zip(names).map(|(i, n)| (i..i + 1, n)).collect()
        };
        // A glued block: on the wood under it, its sides placed by its own
        // numbers and its bottom by the wood's.
        let block = |code, b: GluedBlock, names: [PlacedBy; 3]| {
            let base = self.wood_under(b.back, b.front());
            let on = if base.to_bits() == top.to_bits() {
                "stick_height"
            } else {
                "tail_height"
            };
            solid(
                code,
                polygon(&[
                    Point::new(b.back, base),
                    Point::new(b.front(), base),
                    Point::new(b.front(), base + b.height),
                    Point::new(b.back, base + b.height),
                ]),
                Material::BlockWood,
                sides([on, names[0], names[1], names[2]]),
            )
        };

        // The keystick: the underside from the tail forward, split at the
        // balance hole's edges and the front mortise's where they fall on
        // the flat, then the end grain the key front covers, the top at its
        // full height back to the tail's front, lowered by the clearance for
        // the wippen rail where it has one, and the tail cut down behind it.
        let mut underside = vec![
            Point::new(self.tail_end, self.tail_rise),
            Point::new(kink, 0.0),
        ];
        let mut mouths = [hole_back, hole_front, mortise_back, mortise_front];
        mouths.sort_by(f64::total_cmp);
        for edge in mouths {
            if edge > kink && edge < wood_front && underside.iter().all(|p| p.x < edge) {
                underside.push(Point::new(edge, 0.0));
            }
        }
        underside.push(Point::new(wood_front, 0.0));
        let underside_segments = u16::try_from(underside.len() - 1).expect("a short chain");
        // Where the flat stops at the slot's two edges, for the wood round it:
        // the x the underside's points stand at.
        let edges: Vec<f64> = underside.iter().map(|p| p.x).collect();
        // The flat's segments that are wood and not the slot's mouth or the
        // balance hole's, as runs of consecutive segments: whatever of the
        // flat lies over either is not under the key's wood.
        let wood = |x0: f64, x1: f64| {
            let middle = 0.5 * (x0 + x1);
            (middle < mortise_back || middle > mortise_front)
                && (middle < hole_back || middle > hole_front)
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
        // The chamfer, the flat in its pieces, the end grain, then the top,
        // each stretch with the number that places it: the full top, the
        // clearance's front wall, its floor and its back wall, the full top
        // again, the step down to the tail, the tail's top, and the key's end
        // closing the outline. Where the clearance runs back to the tail's
        // front, its floor runs straight onto the step, with no full top
        // between them; a stretch of no length is not drawn.
        let u = underside_segments;
        let mut stick = underside;
        stick.push(Point::new(wood_front, top));
        let mut top_stretches: Vec<PlacedBy> = Vec::new();
        let mut to = |p: Point, name: PlacedBy| {
            if stick.last() != Some(&p) {
                stick.push(p);
                top_stretches.push(name);
            }
        };
        let mut to_the_tail = false;
        if let Some(c) = self.top_clearance {
            let floor = top - c.depth;
            to(Point::new(c.front, top), "stick_height");
            to(Point::new(c.front, floor), "top_clearance.front");
            to(Point::new(c.back, floor), "top_clearance.depth");
            to_the_tail = c.back.to_bits() == tail_front.to_bits();
            if !to_the_tail {
                to(Point::new(c.back, top), "top_clearance.back");
            }
        }
        if !to_the_tail {
            to(Point::new(tail_front, top), "stick_height");
        }
        to(Point::new(tail_front, self.tail_height), "tail_length");
        to(Point::new(self.tail_end, self.tail_height), "tail_height");
        top_stretches.push("tail_end");
        let mut placed_by = vec![
            (0..1, "tail_rise"),
            (1..u, "tail_rise_run"),
            (u..u + 1, "front_edge"),
        ];
        for (k, name) in (u + 1..).zip(top_stretches) {
            placed_by.push((k..k + 1, name));
        }
        let keystick = push(solid(
            PartCode::Keystick,
            polygon(&stick),
            Material::KeyWood,
            placed_by,
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
            sides(["front_board", "front_edge", "stick_height", "front_board"]),
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
        let keytop_segments = u16::try_from(keytop_points.len()).expect("a short outline");
        let keytop = push(solid(
            PartCode::Keytop,
            polygon(&keytop_points),
            Material::Covering,
            vec![
                (0..1, "stick_height"),
                (1..2, "keytop_overhang"),
                (2..keytop_segments - 1, "keytop_thickness"),
                (keytop_segments - 1..keytop_segments, "keytop_length"),
            ],
        ));

        let (button_outline, button_placed, mouth_y) = self.key_button_outline();
        let key_button = push(solid(
            PartCode::KeyButton,
            button_outline,
            Material::BlockWood,
            button_placed,
        ));

        // The back check block, where the action puts it: placed from the
        // balance pin, it stays under the hammer's tail whatever the key's
        // length.
        let back_check_block = push(block(
            PartCode::BackCheckBlock,
            self.back_check_block(),
            [
                "back_check_block_length",
                "back_check_block_height",
                "back_check_block_back",
            ],
        ));

        let key_tail_felt = self.damper_felt.map(|felt| {
            push(Part {
                role: Role::Lining,
                ..solid(
                    PartCode::KeyTailFelt,
                    polygon(&[
                        Point::new(self.tail_end, self.tail_height),
                        Point::new(tail_front, self.tail_height),
                        Point::new(tail_front, self.tail_height + felt),
                        Point::new(self.tail_end, self.tail_height + felt),
                    ]),
                    Material::HardFelt,
                    sides(["tail_height", "tail_length", "damper_felt", "tail_end"]),
                )
            })
        });

        // The capstan, turned from its profile: from
        // the point of the shaft up to the middle of the head's top, then
        // down the axis. A domed head's top is the dome's arc, from the
        // head's side to the axis, where the profile runs its chord.
        let profile: Vec<Point> = self
            .capstan_profile()
            .into_iter()
            .map(|(r, y)| Point::new(r, y))
            .collect();
        let mut capstan_outline = polygon(&profile);
        if let Some(r) = self.capstan_screw.dome_radius {
            let n = capstan_outline.len();
            let shoulder = profile[n - 2];
            let center = Point::new(0.0, profile[n - 1].y - r);
            if let Some(from) = Dir::new(shoulder.x, shoulder.y - center.y) {
                let dome = Seg::Arc {
                    c: center,
                    r,
                    from,
                    to: Dir::Y,
                };
                if let Seg::Line { b, .. } = &mut capstan_outline[n - 3] {
                    *b = dome.start();
                }
                if let Seg::Line { a, .. } = &mut capstan_outline[n - 1] {
                    *a = dome.end();
                }
                capstan_outline[n - 2] = dome;
            }
        }
        let capstan_segments = count(&capstan_outline);
        let capstan = push(Part {
            body,
            code: PartCode::Capstan,
            placed_by: vec![
                (0..1, "capstan.tip_height"),
                (1..2, "capstan.diameter"),
                (2..3, "capstan.cove_height"),
                (3..4, "capstan.head_diameter"),
                (4..5, "capstan.height"),
                (5..capstan_segments, "capstan.x"),
            ],
            outline: capstan_outline,
            sweep: Sweep::Revolved {
                axis_x: self.capstan,
            },
            role: Role::Solid,
            material: Material::Steel,
        });

        // The front mortise, cut up from the underside. How wide it is across
        // the keyboard is not a dimension the model has: the slot is bushed
        // and close on the pin's flats, out of the sagittal plane.
        let mortise_outline = polygon(&[
            Point::new(mortise_back, 0.0),
            Point::new(mortise_front, 0.0),
            Point::new(mortise_front, self.front_mortise_depth),
            Point::new(mortise_back, self.front_mortise_depth),
        ]);
        let front_mortise = push(Part {
            body,
            code: PartCode::FrontMortise,
            placed_by: vec![
                (0..1, "front_mortise.length"),
                (1..2, "front_mortise.from_front_edge"),
                (2..3, "front_mortise.depth"),
                (3..4, "front_mortise.from_front_edge"),
            ],
            outline: mortise_outline,
            sweep: Sweep::Extruded {
                width: 0.0,
                across: 0.0,
            },
            role: Role::Cut,
            material: Material::KeyWood,
        });

        let BalanceMortiseParts {
            slot_and_cavity,
            hole,
            side_holes: [treble_hole, bass_hole],
            down_the_walls: [treble_bushing, bass_bushing],
            out_through_the_sides: [treble_out, bass_out],
        } = self.balance_mortise_parts(body, mouth_y);
        let balance_mortise = push(slot_and_cavity);
        let balance_hole = push(hole);
        let key_button_side_holes = [push(treble_hole), push(bass_hole)];
        let balance_mortise_bushings_down_the_walls = [push(treble_bushing), push(bass_bushing)];
        let balance_mortise_bushings_out_through_the_sides = [push(treble_out), push(bass_out)];

        let front_bottom = SurfaceRef {
            part: key_front,
            segs: 0..1,
        };
        // The wood round the slot: the underside beside it, and where the flat
        // stops at one of the slot's edges, the slot's wall there, the cut
        // read as the wood it is cut from runs (`crate::contact`). Chained at
        // the slot's mouth, the wall makes the corner of the wood there a
        // convex corner, which a pin straddling the slot's edge meets, and is
        // itself what a pin standing off the middle of its slot meets as the
        // key turns.
        let back_wall = SurfaceRef {
            part: front_mortise,
            segs: 3..4,
        };
        let front_wall = SurfaceRef {
            part: front_mortise,
            segs: 1..2,
        };
        let at = |k: u16, x: f64| edges[usize::from(k)].to_bits() == x.to_bits();
        let mut beside_the_mortise: Vec<SurfaceRef> = Vec::new();
        for segs in beside_runs {
            let (start, end) = (segs.start, segs.end);
            if at(start, mortise_front) {
                beside_the_mortise.push(front_wall.clone());
            }
            beside_the_mortise.push(SurfaceRef {
                part: keystick,
                segs,
            });
            if at(end, mortise_back) {
                beside_the_mortise.push(back_wall.clone());
            }
        }
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
            balance_mortise,
            balance_hole,
            key_button_side_holes,
            balance_mortise_bushings_down_the_walls,
            balance_mortise_bushings_out_through_the_sides,
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

    /// The balance mortise's slot, meters along the key from the balance
    /// pin: centered on it.
    fn balance_slot_span(&self) -> (f64, f64) {
        let half = 0.5 * self.balance_mortise.length;
        (-half, half)
    }

    /// The balance hole, meters along the key from the balance pin: centered
    /// on the pin it is fitted to.
    fn balance_hole_span(&self) -> (f64, f64) {
        let half = 0.5 * self.balance_mortise.hole_diameter;
        (-half, half)
    }

    /// The key button's outline on the wood under it, what places each of
    /// its segments, and the height its flat top is drawn at. The top is
    /// split at the balance mortise's slot's ends, so that the slot's mouth
    /// is a segment of its own, which the slot's cut shares; the arcs' ends
    /// set its height, which may round a bit off the key button's top.
    fn key_button_outline(&self) -> (Vec<Seg>, Placements, f64) {
        let (button_back, button_front) = self.key_button_span();
        let button_base = self.wood_under(button_back, button_front);
        let button_on = if button_base.to_bits() == self.stick_height.to_bits() {
            "stick_height"
        } else {
            "tail_height"
        };
        let ends = rounded_ends(
            button_back,
            button_front,
            button_base,
            self.capsule_height,
            self.capsule_end_radius,
        );
        let mut outline = vec![Seg::Line {
            a: ends[ends.len() - 1].end(),
            b: ends[0].start(),
        }];
        let (slot_back, slot_front) = self.balance_slot_span();
        let mut mouth_y = button_base + self.capsule_height;
        let mut split: Vec<Seg> = Vec::with_capacity(ends.len() + 2);
        for seg in ends {
            match seg {
                Seg::Line { a, b } if a.y.to_bits() == b.y.to_bits() && a.x > b.x => {
                    mouth_y = a.y;
                    let mut from = a;
                    for x in [slot_front, slot_back] {
                        if x < from.x && x > b.x {
                            let to = Point::new(x, a.y);
                            split.push(Seg::Line { a: from, b: to });
                            from = to;
                        }
                    }
                    split.push(Seg::Line { a: from, b });
                }
                other => split.push(other),
            }
        }
        let mut placed_by: Placements = vec![(0..1, button_on)];
        for (index, seg) in (1u16..).zip(&split) {
            let name = match seg {
                Seg::Arc { .. } => "capsule_end_radius",
                Seg::Line { a, b } if a.y.to_bits() == b.y.to_bits() => "capsule_height",
                Seg::Line { .. } => "capsule_length",
            };
            placed_by.push((index..index + 1, name));
        }
        outline.extend(split);
        (outline, placed_by, mouth_y)
    }

    /// The balance mortise's parts, carried by `body`, the slot's top at
    /// `mouth_y`, the key button's. The slot's floor is split at the hole's
    /// edges, so that the hole's mouth is a segment of its own, which the
    /// hole shares. At the foot of the bushing the cloth turns out through
    /// the side hole to the key button's side, glued against the hole's
    /// top, the hole open under it.
    fn balance_mortise_parts(&self, body: BodyId, mouth_y: f64) -> BalanceMortiseParts {
        let mortise = self.balance_mortise;
        let (slot_back, slot_front) = self.balance_slot_span();
        let (hole_back, hole_front) = self.balance_hole_span();
        let mut slot = vec![Point::new(slot_back, mortise.floor)];
        for x in [hole_back, hole_front] {
            if x > slot_back && x < slot_front {
                slot.push(Point::new(x, mortise.floor));
            }
        }
        slot.push(Point::new(slot_front, mortise.floor));
        let floor_segments = u16::try_from(slot.len() - 1).expect("a short floor");
        slot.extend([
            Point::new(slot_front, mouth_y),
            Point::new(slot_back, mouth_y),
        ]);
        let cut = |code, placed_by, outline, width| Part {
            body,
            code,
            placed_by,
            outline,
            sweep: Sweep::Extruded { width, across: 0.0 },
            role: Role::Cut,
            material: Material::KeyWood,
        };
        let slot_and_cavity = cut(
            PartCode::BalanceMortise,
            vec![
                (0..floor_segments, "balance_mortise.floor"),
                (floor_segments..floor_segments + 1, "balance_mortise.length"),
                (floor_segments + 1..floor_segments + 2, "capsule_height"),
                (
                    floor_segments + 2..floor_segments + 3,
                    "balance_mortise.length",
                ),
            ],
            polygon(&slot),
            mortise.width,
        );
        let hole = cut(
            PartCode::BalanceHole,
            vec![
                (0..1, "balance_mortise.hole_diameter"),
                (1..2, "balance_mortise.hole_diameter"),
                (2..3, "balance_mortise.floor"),
                (3..4, "balance_mortise.hole_diameter"),
            ],
            polygon(&[
                Point::new(hole_back, 0.0),
                Point::new(hole_front, 0.0),
                Point::new(hole_front, mortise.floor),
                Point::new(hole_back, mortise.floor),
            ]),
            mortise.hole_diameter,
        );
        let bushing_bottom = mouth_y - mortise.bushing_depth;
        let bushing_across = 0.5 * (mortise.width - mortise.bushing_thickness);
        let side = 0.5 * (self.width - mortise.width);
        let side_across = 0.25 * (self.width + mortise.width);
        let side_hole_top = bushing_bottom + mortise.bushing_thickness;
        let along_the_slot = |bottom: f64, top: f64, placed: [PlacedBy; 2]| {
            let outline = polygon(&[
                Point::new(slot_back, bottom),
                Point::new(slot_front, bottom),
                Point::new(slot_front, top),
                Point::new(slot_back, top),
            ]);
            let placed_by: Placements = vec![
                (0..1, placed[0]),
                (1..2, "balance_mortise.length"),
                (2..3, placed[1]),
                (3..4, "balance_mortise.length"),
            ];
            (outline, placed_by)
        };
        let side_hole = |across: f64| {
            let (outline, placed_by) = along_the_slot(
                side_hole_top - mortise.side_hole_height,
                side_hole_top,
                [
                    "balance_mortise.side_hole_height",
                    "balance_mortise.bushing_thickness",
                ],
            );
            Part {
                body,
                code: PartCode::KeyButtonSideHole,
                placed_by,
                outline,
                sweep: Sweep::Extruded {
                    width: side,
                    across,
                },
                role: Role::Cut,
                material: Material::BlockWood,
            }
        };
        let bushing = |top: f64, top_placed: PlacedBy, width: f64, across: f64| {
            let (outline, placed_by) = along_the_slot(
                bushing_bottom,
                top,
                ["balance_mortise.bushing_depth", top_placed],
            );
            Part {
                body,
                code: PartCode::BalanceMortiseBushing,
                placed_by,
                outline,
                sweep: Sweep::Extruded { width, across },
                role: Role::Lining,
                material: Material::BushingCloth,
            }
        };
        let down_the_wall =
            |across: f64| bushing(mouth_y, "capsule_height", mortise.bushing_thickness, across);
        let out_through_the_side = |across: f64| {
            bushing(
                side_hole_top,
                "balance_mortise.bushing_thickness",
                side,
                across,
            )
        };
        BalanceMortiseParts {
            slot_and_cavity,
            hole,
            side_holes: [side_hole(-side_across), side_hole(side_across)],
            down_the_walls: [
                down_the_wall(-bushing_across),
                down_the_wall(bushing_across),
            ],
            out_through_the_sides: [
                out_through_the_side(-side_across),
                out_through_the_side(side_across),
            ],
        }
    }
}

/// The balance mortise's parts, each pair on the treble side then on the
/// bass side.
struct BalanceMortiseParts {
    /// The slot through the key button and the cavity under it.
    slot_and_cavity: Part,
    /// The balance hole through the cavity's floor.
    hole: Part,
    /// The key button's side holes.
    side_holes: [Part; 2],
    /// The bushing's legs down the slot's walls.
    down_the_walls: [Part; 2],
    /// The bushing's legs out through the side holes to the key button's
    /// sides.
    out_through_the_sides: [Part; 2],
}

impl KeyRanges {
    /// The top of the key as drawn at `x` along it, in the key's frame: the
    /// highest point of its material on the vertical through `x`, where a
    /// weight laid on the key there rests. `None` where none of the key
    /// stands at `x`. `parts` is the list the key's parts were built into.
    pub fn top_at(&self, parts: &[Part], x: f64) -> Option<Point> {
        let material = [
            Some(self.keystick),
            Some(self.keytop),
            Some(self.key_front),
            Some(self.key_button),
            Some(self.back_check_block),
            self.key_tail_felt,
            Some(self.capstan),
        ];
        let mut top: Option<f64> = None;
        let mut offer = |y: f64| {
            if y.is_finite() && top.is_none_or(|t| y > t) {
                top = Some(y);
            }
        };
        for part in material.into_iter().flatten() {
            for seg in parts[part.index()].silhouette() {
                match seg {
                    Seg::Line { a, b } => {
                        let (lo, hi, run) = (a.x.min(b.x), a.x.max(b.x), b.x - a.x);
                        if run != 0.0 && lo <= x && x <= hi {
                            offer(a.y + (x - a.x) * (b.y - a.y) / run);
                        }
                    }
                    Seg::Arc { c, r, from, to } => {
                        let dx = x - c.x;
                        if dx.abs() > r {
                            continue;
                        }
                        let dy = (r * r - dx * dx).sqrt();
                        for y in [c.y + dy, c.y - dy] {
                            let u = Point::new(dx / r, (y - c.y) / r);
                            if crate::geometry::signed::in_sector(u, from, to) {
                                offer(y);
                            }
                        }
                    }
                }
            }
        }
        top.map(|y| Point::new(x, y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: BodyId = BodyId(1);

    fn built(key: &KeyDesign) -> (Vec<Part>, KeyRanges) {
        let mut parts = Vec::new();
        let ranges = key.parts(KEY, &mut parts);
        (parts, ranges)
    }

    /// The polygon a closed outline runs through, its arcs drawn as the
    /// section draws them.
    fn vertices(part: &Part) -> Vec<Point> {
        part.outline
            .iter()
            .flat_map(|seg| seg.drawn(crate::geometry::DRAWN_HALVINGS))
            .collect()
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
    /// counter-clockwise, the cut included, every stretch of it placed by
    /// one number, and every range and point it declares lies inside its
    /// outline.
    #[test]
    fn every_part_of_the_key_is_closed_and_counter_clockwise() {
        for key in [
            crate::test_fixture::key(),
            KeyDesign {
                front_edge: 0.180,
                ..crate::test_fixture::key()
            },
            crate::test_fixture::key_with_every_feature(),
            cleared_to_the_tail(),
        ] {
            let (parts, ranges) = built(&key);
            assert_eq!(
                parts.len(),
                15 + usize::from(key.damper_felt.is_some()),
                "a felt where the tail has one"
            );
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
                let covered: u16 = part.placed_by.iter().map(|(r, _)| r.end - r.start).sum();
                assert_eq!(usize::from(covered), part.outline.len(), "{:?}", part.code);
            }
            for range in ranges.underside.iter().chain([&ranges.front_mortise_top]) {
                let silhouette = parts[range.part.index()].silhouette();
                assert!(usize::from(range.segs.end) <= silhouette.len());
            }
        }
    }

    /// Consistency: the drawn key tiles its section, one polygon through the
    /// key's fields, with no gap and no part drawn over another. The capstan
    /// is turned, and the balance mortise's bushing stands beside the
    /// sagittal plane: neither is in either; the mortises and the balance
    /// hole are cuts, which the polygon does not take out. The two enclose the same area with the same first
    /// moments along and across the key, so a part drawn in one place and
    /// tiled in another, as a block slid along the key's top, is caught where
    /// the area alone would not see it.
    ///
    /// Tiled on the test key, whose tail begins right behind its back check
    /// block; on a key lengthened at the back by 30 mm, with the keystick's
    /// full height between the two; on the key with every feature, its top
    /// lowered by its clearance; and on that key with its clearance run
    /// back to the tail's front.
    ///
    /// The bounds are derived: each shoelace sum rounds once per vertex, on
    /// terms no larger than the reach squared, as many as the parts and the
    /// section have points, the key button's arcs drawn in theirs; each first
    /// moment's term is rounded at most four times, on terms no larger than
    /// twice the reach cubed.
    #[test]
    fn the_drawn_key_tiles_its_section() {
        for key in [
            crate::test_fixture::key(),
            lengthened_at_the_back(0.030),
            crate::test_fixture::key_with_every_feature(),
            cleared_to_the_tail(),
        ] {
            let (parts, _) = built(&key);
            let tiles: Vec<Vec<Point>> = parts
                .iter()
                .filter(|p| {
                    p.role != Role::Cut
                        && matches!(p.sweep, Sweep::Extruded { .. })
                        && p.sweep.is_in_the_section()
                })
                .map(vertices)
                .collect();
            let drawn = tiles.iter().map(|t| moments(t)).fold([0.0; 3], |sum, m| {
                [sum[0] + m[0], sum[1] + m[1], sum[2] + m[2]]
            });
            let outline = key.outline_vertices();
            let section = moments(&outline);
            let terms = (tiles.iter().map(Vec::len).sum::<usize>() + outline.len()) as f64;
            let reach = key.front_edge - key.tail_end;
            assert!(
                (drawn[0] - section[0]).abs() <= terms * f64::EPSILON * reach * reach,
                "the parts enclose {} m², the section {} m²",
                drawn[0],
                section[0]
            );
            for axis in [1, 2] {
                assert!(
                    (drawn[axis] - section[axis]).abs()
                        <= terms * 4.0 * f64::EPSILON * 2.0 * reach * reach * reach,
                    "first moment {axis}: the parts {} m³, the section {} m³",
                    drawn[axis],
                    section[axis]
                );
            }
            assert!(section[0] > 0.0, "and that polygon encloses an area");
        }
    }

    /// Twice a polygon's signed area, and six times its first moments about
    /// the `y` axis and the `x` axis: the shoelace sums an extruded part's
    /// area and centroid are summed from.
    fn moments(points: &[Point]) -> [f64; 3] {
        (0..points.len()).fold([0.0; 3], |[area, along, across], i| {
            let (p, q) = (points[i], points[(i + 1) % points.len()]);
            let cross = p.x * q.y - q.x * p.y;
            [
                area + cross,
                along + (p.x + q.x) * cross,
                across + (p.y + q.y) * cross,
            ]
        })
    }

    /// The key with every feature, its clearance run back to the front of
    /// its thinned tail, to the bit.
    fn cleared_to_the_tail() -> KeyDesign {
        let mut key = crate::test_fixture::key_with_every_feature();
        let tail_front = key.tail_front();
        key.top_clearance.as_mut().expect("a clearance").back = tail_front;
        key
    }

    /// `key` with its key button's ends square.
    fn square_ended(key: KeyDesign) -> KeyDesign {
        KeyDesign {
            capsule_end_radius: 0.0,
            ..key
        }
    }

    /// The test key with its tail end taken `by` meters further back, its
    /// back check block left where the action puts it.
    fn lengthened_at_the_back(by: f64) -> KeyDesign {
        let mut key = crate::test_fixture::key();
        key.tail_end -= by;
        key
    }

    /// The back check block's two ends, from its outline, as drawn.
    fn block_ends(parts: &[Part], r: &KeyRanges) -> (f64, f64) {
        vertices(&parts[r.back_check_block.index()])
            .iter()
            .fold((f64::MAX, f64::MIN), |(lo, hi), p| {
                (lo.min(p.x), hi.max(p.x))
            })
    }

    /// Consistency: **on the test key the tail begins right behind the
    /// back check block**, where the measured tail ends, and the block's
    /// back face runs straight down to the tail's felt: the section's
    /// outline closes on the block's back top corner, the tail's front at
    /// the felt and the tail's end, the same points as the tail's front
    /// computed from the key's end, to the bit, and no corner at the
    /// keystick's top between them; the drawn block runs from the tail's
    /// front.
    ///
    /// Its key button is drawn square-ended, so that the outline's points
    /// are its corners and can be counted.
    #[test]
    fn on_the_default_key_the_tail_begins_right_behind_the_block() {
        let key = square_ended(crate::test_fixture::key());
        let tail_front = key.tail_end + key.tail_length;
        assert_eq!(key.back_check_block_back.to_bits(), tail_front.to_bits());
        let top = key.stick_height;
        let block_front = tail_front + key.back_check_block_length;
        let felt = key.tail_height + key.damper_felt.expect("the test key's tail is felted");
        let outline = key.outline_vertices();
        assert_eq!(outline.len(), 18);
        let tail = [
            Point::new(block_front, top),
            Point::new(block_front, top + key.back_check_block_height),
            Point::new(tail_front, top + key.back_check_block_height),
            Point::new(tail_front, felt),
            Point::new(key.tail_end, felt),
        ];
        for (got, want) in outline[outline.len() - tail.len()..].iter().zip(tail) {
            assert!(same(*got, want), "{got:?} is not {want:?}");
        }
        let (parts, r) = built(&key);
        let block = vertices(&parts[r.back_check_block.index()]);
        assert!(same(block[0], Point::new(tail_front, top)));
        assert!(same(block[1], Point::new(block_front, top)));
    }

    /// Consistency: **a key lengthened at the back keeps its back check block
    /// where the action puts it.** 30 mm longer, the block's two ends stand
    /// where the test key's do, to the bit, drawn and in the section; the thinned tail keeps its 20 mm, now 30 mm behind the block;
    /// and the key's top over those 30 mm is the keystick's full height, the
    /// felt's height behind them. The section steps down at the block's
    /// back end, runs along the keystick's top to the tail's front, and
    /// drops to the felt there.
    #[test]
    fn a_key_lengthened_at_the_back_keeps_its_block_where_it_was() {
        let standard = crate::test_fixture::key();
        let (parts, r) = built(&standard);
        let (back, front) = block_ends(&parts, &r);
        let key = lengthened_at_the_back(0.030);
        let (parts, r) = built(&key);
        let (lengthened_back, lengthened_front) = block_ends(&parts, &r);
        assert_eq!(lengthened_back.to_bits(), back.to_bits());
        assert_eq!(lengthened_front.to_bits(), front.to_bits());

        let tail_front = key.tail_end + key.tail_length;
        // Two roundings, each within half an ulp of the 270 mm they are
        // taken at.
        let rounding = 2.0 * f64::EPSILON * 0.270;
        assert!(
            (back - tail_front - 0.030).abs() <= rounding,
            "{} m of wood between the block and the tail",
            back - tail_front
        );
        let top = key.stick_height;
        let felt = key.tail_height + key.damper_felt.expect("the test key's tail is felted");
        let micrometer = 1e-6;
        for (x, height) in [
            (back - micrometer, top),
            (0.5 * (back + tail_front), top),
            (tail_front + micrometer, top),
            (tail_front - micrometer, felt),
        ] {
            let at = r.top_at(&parts, x).expect("the key stands there");
            assert_eq!(at.y.to_bits(), height.to_bits(), "at {x} m");
        }

        // Its key button square-ended, so that the outline's points are its
        // corners and can be counted.
        let outline = square_ended(key.clone()).outline_vertices();
        let tail = [
            Point::new(front, top),
            Point::new(front, top + key.back_check_block_height),
            Point::new(back, top + key.back_check_block_height),
            Point::new(back, top),
            Point::new(tail_front, top),
            Point::new(tail_front, felt),
            Point::new(key.tail_end, felt),
        ];
        assert_eq!(outline.len(), 20);
        for (got, want) in outline[outline.len() - tail.len()..].iter().zip(tail) {
            assert!(same(*got, want), "{got:?} is not {want:?}");
        }
    }

    /// The corners of the underside the key's fields put it on, from the
    /// tail forward: the end of the tail, the kink where the chamfer runs out
    /// into the flat, and the front of the key.
    fn corners(key: &KeyDesign) -> [Point; 3] {
        let (flat_from, flat_to) = key.underside_flat();
        [
            Point::new(key.tail_end, key.tail_rise),
            Point::new(flat_from, 0.0),
            Point::new(flat_to, 0.0),
        ]
    }

    /// The underside is the chamfer, then one straight flat to the front of
    /// the key, split at the balance hole's edges and the front mortise's:
    /// the corners the key's fields put it on are its vertices, bit for bit,
    /// and the bottoms of the hole and of the mortise are segments of it.
    #[test]
    fn the_underside_is_split_at_the_mouths_of_its_cuts_and_runs_on_across_them() {
        let key = crate::test_fixture::key();
        let (parts, ranges) = built(&key);
        let stick = ranges.underside[0].segs(&parts);
        let front = ranges.underside[1].segs(&parts);
        assert_eq!(
            stick.len(),
            6,
            "chamfer, flat, the hole's mouth, flat, the slot's mouth, flat"
        );
        let [tail, kink, nose] = corners(&key);
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
        let hole = &parts[ranges.balance_hole.index()];
        assert_eq!(hole.role, Role::Cut);
        assert!(same(stick[2].start(), hole.outline[0].start()));
        assert!(same(stick[2].end(), hole.outline[0].end()));
        let (back, forward) = key.front_mortise_span();
        let mortise = &parts[ranges.front_mortise.index()];
        assert_eq!(mortise.role, Role::Cut);
        assert!(same(stick[4].start(), mortise.outline[0].start()));
        assert!(same(stick[4].end(), mortise.outline[0].end()));
        assert_eq!(stick[4].start().x.to_bits(), back.to_bits());
        assert_eq!(stick[4].end().x.to_bits(), forward.to_bits());
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

    /// Consistency: **the balance mortise's mouths are segments of what they
    /// open through, to the bit**: the slot's top is a segment of the key
    /// button's flat top, split at the slot's ends, and the hole's top a
    /// segment of the slot's floor, split at the hole's edges, so that the
    /// view leaves both open. The bushing stands on each long side of the
    /// slot, its cloth's thickness wide and its outer face on the slot's
    /// wall, from the slot's top down its depth: on the test key's 6.0 mm
    /// slot and 1.3 mm cloth, 2.35 mm from the sagittal plane, on the
    /// treble side and on the bass side.
    #[test]
    fn the_balance_mortises_mouths_are_segments_of_what_they_open_through() {
        let key = crate::test_fixture::key();
        let (parts, ranges) = built(&key);
        let segments = |id: PartId| -> Vec<(Point, Point)> {
            parts[id.index()]
                .outline
                .iter()
                .map(|seg| (seg.start(), seg.end()))
                .collect()
        };
        let shared = |a: &[(Point, Point)], b: &[(Point, Point)]| -> Vec<(Point, Point)> {
            a.iter()
                .copied()
                .filter(|&(p, q)| {
                    b.iter()
                        .any(|&(u, v)| (same(p, u) && same(q, v)) || (same(p, v) && same(q, u)))
                })
                .collect()
        };
        let slot = segments(ranges.balance_mortise);
        let half_slot = 0.5 * key.balance_mortise.length;
        let mouth = shared(&slot, &segments(ranges.key_button));
        assert_eq!(mouth.len(), 1, "{mouth:?}");
        let (front, back) = mouth[0];
        assert_eq!(
            (front.x.to_bits(), back.x.to_bits()),
            (half_slot.to_bits(), (-half_slot).to_bits())
        );
        let half_hole = 0.5 * key.balance_mortise.hole_diameter;
        let hole_top = shared(&segments(ranges.balance_hole), &slot);
        assert_eq!(hole_top.len(), 1, "{hole_top:?}");
        let (p, q) = hole_top[0];
        assert_eq!(
            (p.y.to_bits(), q.y.to_bits()),
            (
                key.balance_mortise.floor.to_bits(),
                key.balance_mortise.floor.to_bits()
            )
        );
        assert_eq!(p.x.abs().to_bits(), half_hole.to_bits());

        let top = front.y;
        for (id, toward_the_bass) in ranges
            .balance_mortise_bushings_down_the_walls
            .into_iter()
            .zip([-1.0, 1.0])
        {
            let bushing = &parts[id.index()];
            let Sweep::Extruded { width, across } = bushing.sweep else {
                panic!("a bushing is extruded");
            };
            assert_eq!(
                width.to_bits(),
                key.balance_mortise.bushing_thickness.to_bits()
            );
            // One subtraction and one halving of millimeters.
            assert!((across - toward_the_bass * 0.00235).abs() <= f64::EPSILON * 0.01);
            let ys: Vec<f64> = vertices(bushing).iter().map(|v| v.y).collect();
            let highest = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let lowest = ys.iter().copied().fold(f64::INFINITY, f64::min);
            assert_eq!(highest.to_bits(), top.to_bits(), "from the slot's top");
            assert!(
                (highest - lowest - key.balance_mortise.bushing_depth).abs() <= f64::EPSILON * 0.1,
                "down its depth"
            );
        }
    }

    /// Consistency: **the bushing turns out through the key button's side
    /// holes to its sides, glued against their top.** On the test key, each
    /// side hole runs from the slot's wall, 3 mm from the sagittal plane, to
    /// the key button's side, 11 mm from it, along the slot's 10.2 mm, 2.6 mm
    /// high, its top 1.3 mm of cloth above the foot of the bushing's 3 mm
    /// depth: from 26.2 to 28.8 mm above the underside. The cloth's leg out
    /// through it runs as far across and along, from 27.5 mm up to the
    /// hole's top, on the treble side and on the bass side.
    #[test]
    fn the_bushing_turns_out_through_the_key_buttons_side_holes_against_their_top() {
        let key = crate::test_fixture::key();
        let (parts, ranges) = built(&key);
        // A few roundings of lengths under a decimeter.
        let near = |found: f64, expected: f64| (found - expected).abs() <= 8.0 * f64::EPSILON * 0.1;
        let span = |lengths: &[f64]| {
            (
                lengths.iter().copied().fold(f64::INFINITY, f64::min),
                lengths.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            )
        };
        let across_along_and_up = |part: &Part, toward_the_bass: f64| {
            let Sweep::Extruded { width, across } = part.sweep else {
                panic!("{:?} is extruded", part.code);
            };
            let out = toward_the_bass * across;
            let corners = vertices(part);
            let xs: Vec<f64> = corners.iter().map(|corner| corner.x).collect();
            let ys: Vec<f64> = corners.iter().map(|corner| corner.y).collect();
            ((out - 0.5 * width, out + 0.5 * width), span(&xs), span(&ys))
        };
        for (side, toward_the_bass) in [-1.0, 1.0].into_iter().enumerate() {
            let hole = &parts[ranges.key_button_side_holes[side].index()];
            let leg = &parts[ranges.balance_mortise_bushings_out_through_the_sides[side].index()];
            assert_eq!((hole.role, leg.role), (Role::Cut, Role::Lining));
            let (hole_across, hole_along, (hole_bottom, hole_top)) =
                across_along_and_up(hole, toward_the_bass);
            let (leg_across, leg_along, (leg_bottom, leg_top)) =
                across_along_and_up(leg, toward_the_bass);
            assert!(
                near(hole_across.0, 0.003) && near(hole_across.1, 0.011),
                "{hole_across:?} m"
            );
            assert!(
                near(hole_along.0, -0.0051) && near(hole_along.1, 0.0051),
                "{hole_along:?} m"
            );
            assert!(
                near(hole_bottom, 0.0262) && near(hole_top, 0.0288),
                "{hole_bottom} to {hole_top} m"
            );
            assert_eq!((leg_across, leg_along), (hole_across, hole_along));
            assert!(near(leg_bottom, 0.0275), "{leg_bottom} m");
            assert_eq!(
                leg_top.to_bits(),
                hole_top.to_bits(),
                "against the hole's top"
            );
        }
    }

    /// The flat the punchings read runs from where the chamfer ends to the
    /// front of the key, across the mouths of the balance hole and of the
    /// slot; the wood beside the mortise is that flat less the two mouths,
    /// behind the slot and in front of it, the key front's bottom included.
    /// Moved off the flat, the mortise leaves the whole flat but the hole's
    /// mouth beside it.
    #[test]
    fn the_flat_and_the_wood_beside_the_mortise_are_declared() {
        let length = |segs: &[Seg]| -> f64 { segs.iter().map(|s| s.end().x - s.start().x).sum() };
        let key = crate::test_fixture::key();
        let (parts, ranges) = built(&key);
        let [_, kink, nose] = corners(&key);
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
            ranges
                .beside_the_mortise
                .iter()
                .map(|r| (r.part, r.segs.clone()))
                .collect::<Vec<_>>(),
            vec![
                (ranges.keystick, 1..2),
                (ranges.keystick, 3..4),
                (ranges.front_mortise, 3..4),
                (ranges.front_mortise, 1..2),
                (ranges.keystick, 5..6),
                (ranges.key_front, 0..1),
            ],
            "behind the hole, between it and the slot, the slot's back wall, \
             its front wall, in front, the key front"
        );
        // The walls stand upright, so the flat's length less the wood's is
        // the two mouths.
        let mouths = front - back + key.balance_mortise.hole_diameter;
        assert!((length(&flat) - length(&beside) - mouths).abs() < 1e-15);

        // A mortise cut forward of the key's flat, out of reach: nothing of
        // the flat is over it.
        let odd = KeyDesign {
            front_mortise_from_front_edge: -0.010,
            ..crate::test_fixture::key()
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
        let hole_mouth = ranges.underside[0].segs(&parts)[2];
        let flat_but_the_hole: Vec<Seg> = flat.into_iter().filter(|s| *s != hole_mouth).collect();
        assert_eq!(flat_but_the_hole, beside);
    }

    /// The named points stand where the protocols read them, to the bit: the
    /// lip every touchweight is taken back from, the key front's top where key
    /// height and dip are read, the capstan's top where the jig centers 10 g.
    #[test]
    fn the_named_points_are_where_the_protocols_read() {
        let key = crate::test_fixture::key();
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
        let (parts, r) = built(&crate::test_fixture::key());
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
        let key = crate::test_fixture::key();
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
        assert_eq!(
            lowest(r.key_tail_felt.expect("a felt")).to_bits(),
            key.tail_height.to_bits()
        );
        assert!(key.tail_height < key.stick_height);
    }

    /// The capstan screws straight into the keystick: nothing glued under it,
    /// its seat the stick's own top, and its drawn head at the height the
    /// technician sets above that seat.
    #[test]
    fn the_capstan_screws_into_the_stick_itself() {
        let key = crate::test_fixture::key();
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

    /// Consistency: **a glued block stands on the wood under it, and the
    /// capstan on the keystick's full height, its dome's top where the
    /// protocols read it.** On a key whose bare thinned tail runs on under
    /// its back check block, the block stands on the tail's top, and its key
    /// button, behind the balance pin, on the keystick's; the capstan is
    /// seated on the keystick's full height, and its drawn top, the dome's
    /// apex on its axis, is the capstan's contact point within an ulp of the
    /// height it is screwed to, the dome an arc of its radius about a center
    /// that far below. No felt is drawn on the bare tail.
    #[test]
    fn a_block_stands_on_the_wood_under_it_and_the_capstan_on_the_full_height() {
        let key = crate::test_fixture::key_with_every_feature();
        let (parts, r) = built(&key);
        let lowest = |id: PartId| {
            vertices(&parts[id.index()])
                .iter()
                .fold(f64::MAX, |a, p| a.min(p.y))
        };
        assert_eq!(
            lowest(r.back_check_block).to_bits(),
            key.tail_height.to_bits()
        );
        assert_eq!(lowest(r.key_button).to_bits(), key.stick_height.to_bits());
        assert_eq!(r.key_tail_felt, None);
        assert_eq!(key.capstan_seat().to_bits(), key.stick_height.to_bits());
        assert!(key.capstan - 0.5 * key.capstan_screw.head_diameter > key.tail_front());
        let (button_back, _) = key.key_button_span();
        assert_eq!(
            vertices(&parts[r.key_button.index()])[0].x.to_bits(),
            button_back.to_bits()
        );

        let anchored = r.capstan_contact.point(&parts);
        let screwed = key.capstan_contact();
        assert_eq!(anchored.x.to_bits(), screwed.x.to_bits());
        assert!((anchored.y - screwed.y).abs() <= f64::EPSILON * screwed.y);
        let capstan = &parts[r.capstan.index()];
        assert!(capstan.is_closed());
        let dome = key.capstan_screw.dome_radius.expect("a dome");
        let arcs: Vec<Seg> = capstan
            .outline
            .iter()
            .filter(|s| matches!(s, Seg::Arc { .. }))
            .copied()
            .collect();
        let [Seg::Arc { c, r: radius, .. }] = arcs[..] else {
            panic!("one arc: {arcs:?}")
        };
        assert_eq!(radius.to_bits(), dome.to_bits());
        assert!((c.y + radius - key.capstan_screw.height - key.capstan_seat()).abs() <= 1e-15);
    }

    /// Consistency: **the clearance lowers the keystick's top between its
    /// two ends, and the key comes back to its full height behind it.** On
    /// the key with every feature, the keystick's outline runs along its
    /// full top to the clearance's front end, down its front wall by its
    /// depth, along its floor to its back end, up its back wall to the full
    /// top again and on to the step down to the tail, each corner where the
    /// clearance's numbers put it, to the bit, and each stretch placed by
    /// its own number; a weight laid on the key stands on the full top a
    /// micrometer in front of the clearance and behind it, and on its floor
    /// within it. Run back to the tail's front, the floor runs straight onto
    /// the step, the back wall and the full top behind it gone.
    #[test]
    fn the_clearance_lowers_the_top_between_its_ends() {
        let micrometer = 1e-6;
        let key = crate::test_fixture::key_with_every_feature();
        let c = key.top_clearance.expect("a clearance");
        let (top, floor) = (key.stick_height, key.stick_height - c.depth);
        let tail_front = key.tail_front();
        let (parts, r) = built(&key);
        let stick = &parts[r.keystick.index()];
        let corners = vertices(stick);
        let cut = [
            Point::new(c.front, top),
            Point::new(c.front, floor),
            Point::new(c.back, floor),
            Point::new(c.back, top),
            Point::new(tail_front, top),
            Point::new(tail_front, key.tail_height),
        ];
        let at = corners
            .iter()
            .position(|&p| same(p, cut[0]))
            .expect("the clearance's front top corner");
        for (k, want) in cut.iter().enumerate() {
            assert!(
                same(corners[at + k], *want),
                "{:?} is not {want:?}",
                corners[at + k]
            );
        }
        let placed = |seg: usize| -> PlacedBy {
            let seg = u16::try_from(seg).expect("a short outline");
            stick
                .placed_by
                .iter()
                .find(|(range, _)| range.contains(&seg))
                .map(|(_, name)| *name)
                .expect("every stretch placed")
        };
        for (k, name) in [
            "top_clearance.front",
            "top_clearance.depth",
            "top_clearance.back",
            "stick_height",
            "tail_length",
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(placed(at + k), name, "stretch {k} of the clearance");
        }
        assert_eq!(placed(at - 1), "stick_height", "the full top in front");
        for (x, height) in [
            (c.front + micrometer, top),
            (c.front - micrometer, floor),
            (0.5 * (c.back + c.front), floor),
            (c.back + micrometer, floor),
            (c.back - micrometer, top),
            (tail_front + micrometer, top),
        ] {
            let on = r.top_at(&parts, x).expect("the key stands there");
            assert_eq!(on.y.to_bits(), height.to_bits(), "at {x} m");
        }

        let key = cleared_to_the_tail();
        let (parts, r) = built(&key);
        let corners = vertices(&parts[r.keystick.index()]);
        let cut = [
            Point::new(c.front, top),
            Point::new(c.front, floor),
            Point::new(tail_front, floor),
            Point::new(tail_front, key.tail_height),
            Point::new(key.tail_end, key.tail_height),
        ];
        assert!(same(*corners.last().expect("corners"), cut[4]));
        for (got, want) in corners[corners.len() - cut.len()..].iter().zip(cut) {
            assert!(same(*got, want), "{got:?} is not {want:?}");
        }
        let on = r
            .top_at(&parts, tail_front + micrometer)
            .expect("the key stands there");
        assert_eq!(on.y.to_bits(), floor.to_bits());
    }

    /// Consistency: **a key lengthened at either end keeps its clearance
    /// where the action puts it.** The clearance clears the wippen rail,
    /// which the action places, so it is placed from the balance pin, as the
    /// back check block is: the key with every feature lengthened 30 mm at
    /// the back, its thinned tail with it, or 10 mm at the front, draws the
    /// clearance's four corners where the key as it stands draws them, to
    /// the bit.
    #[test]
    fn a_key_lengthened_at_either_end_keeps_its_clearance_where_it_was() {
        let key = crate::test_fixture::key_with_every_feature();
        let c = key.top_clearance.expect("a clearance");
        let (top, floor) = (key.stick_height, key.stick_height - c.depth);
        let corners = [
            Point::new(c.front, top),
            Point::new(c.front, floor),
            Point::new(c.back, floor),
            Point::new(c.back, top),
        ];
        let mut at_the_back = key.clone();
        at_the_back.tail_end -= 0.030;
        at_the_back.tail_length += 0.030;
        let mut at_the_front = key.clone();
        at_the_front.front_edge += 0.010;
        for (what, k) in [
            ("as it stands", key),
            ("lengthened at the back", at_the_back),
            ("lengthened at the front", at_the_front),
        ] {
            let (parts, r) = built(&k);
            let drawn = vertices(&parts[r.keystick.index()]);
            for corner in corners {
                assert!(
                    drawn.iter().any(|&p| same(p, corner)),
                    "{what}: no corner at {corner:?}"
                );
            }
        }
    }

    /// The back rail the placement rule builds bears on the chamfer, rail
    /// width included: the drawn chamfer runs from the tail to where the flat
    /// begins, and the rule is free to put the cloth anywhere but there.
    #[test]
    fn the_back_rail_placed_for_the_key_falls_on_the_drawn_chamfer() {
        let key = crate::test_fixture::key();
        let (parts, r) = built(&key);
        let chamfer = r.underside[0].segs(&parts)[0];
        let placed = crate::test_fixture::frame().placed_under(&key);
        let half = placed.sections.back.half_length;
        assert!(placed.back_rail - half > chamfer.start().x);
        assert!(placed.back_rail + half < chamfer.end().x);
        assert!(
            chamfer.start().y > chamfer.end().y,
            "the chamfer rises toward the tail"
        );
    }
}

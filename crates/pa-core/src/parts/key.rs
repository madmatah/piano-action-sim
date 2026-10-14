//! The key as parts: one outline for each piece of material, in the key's
//! frame.
//!
//! The keystick is one thickness of wood, cut down to its thinned tail at the
//! back. Glued to it: the keytop, whose lip overhangs the key front; the key
//! front over the end grain; the key button where the balance mortise is
//! cut; the back check block; the block the capstan stands on, where the key
//! has one; the hard felt on the thinned tail, where it has one. Each glued
//! block stands on the wood under it, the keystick's full height or the bare
//! tail. Screwed into it, or into its block: the capstan, a turned part, its
//! head flat or domed. Cut out of it: the front mortise, drawn as the void it
//! is.
//!
//! Each piece is its own part with its own material, so the keytop and the key
//! front can be drawn white and the glued blocks in their own wood: what a
//! part is made of is the model's, the colour the view's. Each stretch of each
//! outline names the number of the key that places it ([`PlacedBy`]), so an
//! application can say how that number is known. The key's mass is not
//! integrated over these outlines; it is weighed or published, or it comes
//! from the volume model ([`KeyDesign::mass_properties`]), whose polygon these
//! parts tile exactly.
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

use super::{count, next_id, polygon, GluedBlock, KeyDesign};
use crate::contact::{
    Anchor, AnchorCode, Material, Part, PartCode, PartId, PlacedBy, Role, SurfaceRef, Sweep,
};
use crate::geometry::{Dir, Point, Seg};
use crate::mechanism::BodyId;

/// The numbers that place a glued block's four sides, from its bottom round
/// counter-clockwise: the wood it stands on, its front, its top, its back.
type BlockNames = [PlacedBy; 4];

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
    /// The block the capstan stands on, where the key has one.
    pub capstan_block: Option<PartId>,
    /// The felt on the key's tail, where it has one.
    pub key_tail_felt: Option<PartId>,
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
    /// The wood round the front mortise: the flat less the slot's mouth,
    /// behind the slot and in front of it, each run chained at the slot's
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
    /// key's frame: origin at the balance pin on the underside, `x` towards
    /// the player. Returns what they declare.
    ///
    /// Built from the same fields as the volume model's
    /// [`Self::outline_vertices`], with the same arithmetic, so the points
    /// the two share are the same bits.
    pub fn parts(&self, body: BodyId, out: &mut Vec<Part>) -> KeyRanges {
        let top = self.stick_height;
        let wood_front = self.front_edge - self.front_board;
        let lip = self.lip();
        let keytop_back = lip - self.keytop_length;
        let covered = top + self.keytop_thickness;
        let tail_front = self.tail_front();
        let kink = self.tail_end + self.tail_rise_run;
        let (mortise_back, mortise_front) = self.front_mortise_span();
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
            sweep: Sweep::Extruded { width: self.width },
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
        // mortise's edges where they fall on the flat, then the end grain the
        // key front covers, the top at its full height back to the tail's
        // front, and the tail cut down behind it.
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
        // Where the flat stops at the slot's two edges, for the wood round it:
        // the x the underside's points stand at.
        let edges: Vec<f64> = underside.iter().map(|p| p.x).collect();
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
        // The chamfer, the flat in its pieces, the end grain, the top, the
        // step down to the tail, the tail's top and the key's end.
        let u = underside_segments;
        let keystick = push(solid(
            PartCode::Keystick,
            polygon(&stick),
            Material::KeyWood,
            vec![
                (0..1, "tail_rise"),
                (1..u, "tail_rise_run"),
                (u..u + 1, "front_edge"),
                (u + 1..u + 2, "stick_height"),
                (u + 2..u + 3, "tail_length"),
                (u + 3..u + 4, "tail_height"),
                (u + 4..u + 5, "tail_end"),
            ],
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

        // The key button, where the balance mortise is cut through.
        let (button_back, button_front) = self.key_button_span();
        let key_button = push(block(
            PartCode::KeyButton,
            GluedBlock {
                back: button_back,
                length: button_front - button_back,
                height: self.capsule_height,
            },
            ["capsule_length", "capsule_height", "capsule_length"],
        ));

        // The block the capstan stands on, where the key has one.
        let capstan_block = self.capstan_block.map(|b| {
            push(block(
                PartCode::CapstanBlock,
                b,
                [
                    "capstan_block.length",
                    "capstan_block.height",
                    "capstan_block.back",
                ],
            ))
        });

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

        // The capstan, turned from the profile the volume model turns: from
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
            let centre = Point::new(0.0, profile[n - 1].y - r);
            if let Some(from) = Dir::new(shoulder.x, shoulder.y - centre.y) {
                let dome = Seg::Arc {
                    c: centre,
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
            sweep: Sweep::Extruded { width: 0.0 },
            role: Role::Cut,
            material: Material::KeyWood,
        });

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
            capstan_block,
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
            self.capstan_block,
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
            crate::test_fixture::key(),
            KeyDesign {
                front_edge: 0.180,
                ..crate::test_fixture::key()
            },
            crate::test_fixture::key_with_every_feature(),
        ] {
            let (parts, ranges) = built(&key);
            assert_eq!(parts.len(), 8, "a felt or a capstan block, not both");
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

    /// Consistency: the drawn key tiles the volume model's polygon, so the key
    /// a reader sees is the key the composite key's mass is integrated over.
    /// The capstan is turned, and in neither; the mortise is a cut, which the
    /// volume model does not take out. The two enclose the same area with the
    /// same first moments along and across the key, so a part drawn in one
    /// place and integrated in another, as a block slid along the key's top,
    /// is caught where the area alone would not see it.
    ///
    /// Tiled on the composite key, whose tail begins right behind its back
    /// check block, and on a key lengthened at the back by 30 mm, with the
    /// keystick's full height between the two.
    ///
    /// The bounds are derived: each shoelace sum rounds once per vertex, on
    /// terms no larger than the reach squared, and there are under 40 of them;
    /// each first moment's term is rounded at most four times, on terms no
    /// larger than twice the reach cubed.
    #[test]
    fn the_drawn_key_tiles_the_polygon_its_mass_is_integrated_over() {
        for key in [
            crate::test_fixture::key(),
            lengthened_at_the_back(0.030),
            crate::test_fixture::key_with_every_feature(),
        ] {
            let (parts, _) = built(&key);
            let drawn = parts
                .iter()
                .filter(|p| p.role != Role::Cut && matches!(p.sweep, Sweep::Extruded { .. }))
                .map(|p| moments(&vertices(p)))
                .fold([0.0; 3], |sum, m| {
                    [sum[0] + m[0], sum[1] + m[1], sum[2] + m[2]]
                });
            let volume = moments(&key.outline_vertices());
            let reach = key.front_edge - key.tail_end;
            assert!(
                (drawn[0] - volume[0]).abs() <= 40.0 * f64::EPSILON * reach * reach,
                "the parts enclose {} m², the volume model {} m²",
                drawn[0],
                volume[0]
            );
            for axis in [1, 2] {
                assert!(
                    (drawn[axis] - volume[axis]).abs()
                        <= 40.0 * 4.0 * f64::EPSILON * 2.0 * reach * reach * reach,
                    "first moment {axis}: the parts {} m³, the volume model {} m³",
                    drawn[axis],
                    volume[axis]
                );
            }
            let mass = polygon_mass_properties(&key.outline_vertices(), key.width, key.density);
            assert!(mass.is_some(), "and that polygon weighs something");
        }
    }

    /// Twice a polygon's signed area, and six times its first moments about
    /// the `y` axis and the `x` axis: the shoelace sums an extruded part's
    /// mass and centre of mass are integrated from.
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

    /// The composite key with its tail end taken `by` metres further back, its
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

    /// Consistency: **on the composite key the tail begins right behind the
    /// back check block**, where the measured tail ends, and the block's
    /// back face runs straight down to the tail's felt: the volume model's
    /// outline closes on the block's back top corner, the tail's front at
    /// the felt and the tail's end, the same points as the tail's front
    /// computed from the key's end, to the bit, and no corner at the
    /// keystick's top between them; the drawn block runs from the tail's
    /// front.
    #[test]
    fn on_the_default_key_the_tail_begins_right_behind_the_block() {
        let key = crate::test_fixture::key();
        let tail_front = key.tail_end + key.tail_length;
        assert_eq!(key.back_check_block_back.to_bits(), tail_front.to_bits());
        let top = key.stick_height;
        let block_front = tail_front + key.back_check_block_length;
        let felt = key.tail_height + key.damper_felt.expect("the composite key's tail is felted");
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
    /// where the composite key's do, to the bit, drawn and in the volume
    /// model; the thinned tail keeps its 20 mm, now 30 mm behind the block;
    /// and the key's top over those 30 mm is the keystick's full height, the
    /// felt's height behind them. The volume model steps down at the block's
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
        let felt = key.tail_height + key.damper_felt.expect("the composite key's tail is felted");
        let micrometre = 1e-6;
        for (x, height) in [
            (back - micrometre, top),
            (0.5 * (back + tail_front), top),
            (tail_front + micrometre, top),
            (tail_front - micrometre, felt),
        ] {
            let at = r.top_at(&parts, x).expect("the key stands there");
            assert_eq!(at.y.to_bits(), height.to_bits(), "at {x} m");
        }

        let outline = key.outline_vertices();
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
    /// the key, split at the mortise's edges: the corners the key's fields
    /// put it on are its vertices, bit for bit, and the mortise's bottom is a
    /// segment of it.
    #[test]
    fn the_underside_is_split_at_the_mortise_edges_and_runs_on_across_the_slot() {
        let key = crate::test_fixture::key();
        let (parts, ranges) = built(&key);
        let stick = ranges.underside[0].segs(&parts);
        let front = ranges.underside[1].segs(&parts);
        assert_eq!(stick.len(), 4, "chamfer, flat, the slot's mouth, flat");
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
                (ranges.front_mortise, 3..4),
                (ranges.front_mortise, 1..2),
                (ranges.keystick, 3..4),
                (ranges.key_front, 0..1),
            ],
            "behind, the slot's back wall, its front wall, in front, the key front"
        );
        // The walls stand upright, so the flat's length less the wood's is
        // the mouth.
        let mouth = front - back;
        assert!((length(&flat) - length(&beside) - mouth).abs() < 1e-15);

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
        assert_eq!(flat, beside);
    }

    /// The named points stand where the protocols read them, to the bit: the
    /// lip every touchweight is taken back from, the key front's top where key
    /// height and dip are read, the capstan's top where the jig centres 10 g.
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

    /// Consistency: **a glued block stands on the wood under it, and a
    /// capstan on its own block, its dome's top where the protocols read
    /// it.** On a key whose bare thinned tail runs on under its back check
    /// block and its capstan's block, both stand on the tail's top, and its
    /// key button, behind the balance pin, on the keystick's; the capstan
    /// is seated on its block's top, and its drawn top, the dome's apex on
    /// its axis, is the capstan's contact point within an ulp of the height
    /// it is screwed to, the dome an arc of its radius about a centre that
    /// far below. No felt is drawn on the bare tail.
    #[test]
    fn a_block_stands_on_the_wood_under_it_and_the_capstan_on_its_block() {
        let key = crate::test_fixture::key_with_every_feature();
        let (parts, r) = built(&key);
        let lowest = |id: PartId| {
            vertices(&parts[id.index()])
                .iter()
                .fold(f64::MAX, |a, p| a.min(p.y))
        };
        let block = r.capstan_block.expect("a capstan block");
        assert_eq!(lowest(block).to_bits(), key.tail_height.to_bits());
        assert_eq!(
            lowest(r.back_check_block).to_bits(),
            key.tail_height.to_bits()
        );
        assert_eq!(lowest(r.key_button).to_bits(), key.stick_height.to_bits());
        assert_eq!(r.key_tail_felt, None);
        let b = key.capstan_block.expect("a block");
        assert_eq!(
            key.capstan_seat().to_bits(),
            (key.tail_height + b.height).to_bits()
        );
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
            "the chamfer rises towards the tail"
        );
    }
}

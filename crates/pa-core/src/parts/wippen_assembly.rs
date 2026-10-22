//! The wippen assembly as parts: the wippen's body, its heel and the heel
//! cloth, the jack and the repetition lever pinned on it, and the center pin
//! it turns on, each one closed outline of one material.
//!
//! # Its frame
//!
//! The wippen's frame has its origin at its center pin, `x` toward the player
//! and `y` up when the wippen stands at angle zero. The jack and the
//! repetition lever each have a frame of their own, origin at their own pin
//! on the wippen, and stand on the wippen at a rest angle: rigid on it, each
//! resting on its stop, so that their outlines are drawn on the
//! wippen's body, turned to that angle about their pins. The center pin is a
//! part of the ground, where the wippen's center stands in the keybed's frame.
//!
//! # Parametric and traced
//!
//! What places a pivot or a contact is a number of the design: the wippen's
//! center, the jack's and the lever's pins and rest angles, and the heel,
//! which the capstan bears on, placed by the circle of its cloth's outer face
//! and the reaches of its ends and of its corners along the wippen from that
//! circle's center, so that moving the heel's center moves the whole heel.
//! The rest of each outline, the body's, the jack's and the lever's, is
//! traced ([`Stretch`]): a source's drawing, carried into the part's frame,
//! and placed as a whole by the number that names the trace.
//!
//! # The heel
//!
//! The cloth's outer face is an arc of the heel's circle between its two
//! ends, its ends run in along the circle's radius by the cloth's thickness,
//! and its top runs straight between them. The heel stands under the
//! wippen's body from its top, the body's bottom, down to its two lower
//! corners, which stand on the line through the cloth's two ends on its outer
//! face, beyond them; between them the cloth sits in a pocket of the heel's
//! underside, flush with the heel at its ends and bulging below it between
//! them. The heel's sides run straight from its corners up to the body.

use super::{count, next_id};
use crate::contact::{Material, Part, PartCode, PartId, PlacedBy, Role, SurfaceRef, Sweep};
use crate::geometry::{Dir, Point, Seg};
use crate::mechanism::{AxisFriction, BodyId, MassProps};
use crate::registry::{Kind, Layer, ParamSpec, Quantity, Step};

/// One stretch of a traced outline: from where it starts to where the next
/// stretch starts, the last to where the first does. A traced outline runs
/// counter-clockwise, its material on its left.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stretch {
    /// Straight, from this point.
    Line(Point),
    /// Counter-clockwise round a circle, from this point: convex from the
    /// material, as every arc of a material outline is.
    Arc {
        /// Where it starts.
        from: Point,
        /// The circle's center.
        center: Point,
        /// The circle's radius, meters.
        radius: f64,
    },
}

impl Stretch {
    /// Where it starts.
    pub fn start(&self) -> Point {
        match *self {
            Self::Line(p) | Self::Arc { from: p, .. } => p,
        }
    }

    /// Its numbers, in a fixed order: the point it starts at, then an arc's
    /// center and radius.
    pub fn numbers(&self) -> Vec<f64> {
        match *self {
            Self::Line(p) => vec![p.x, p.y],
            Self::Arc {
                from,
                center,
                radius,
            } => vec![from.x, from.y, center.x, center.y, radius],
        }
    }
}

/// A part pinned on the wippen: the jack or the repetition lever, rigid on
/// it at its rest angle.
#[derive(Debug, Clone, PartialEq)]
pub struct PinnedPart {
    /// Its pin, in the wippen's frame, meters: the origin of its own frame.
    pub pin: Point,
    /// Its angle on the wippen, radians, counter-clockwise from the wippen's
    /// frame to its own: where it rests on its stop.
    pub rest_angle: f64,
    /// Its outline, traced, in its own frame.
    pub outline: Vec<Stretch>,
    /// Its width across the keyboard, meters: what the solid view extrudes
    /// it by.
    pub width: f64,
}

impl PinnedPart {
    /// A point of its own frame, in the wippen's.
    pub fn on_the_wippen(&self, p: Point) -> Point {
        let (s, c) = (libm::sin(self.rest_angle), libm::cos(self.rest_angle));
        Point::new(
            self.pin.x + (c * p.x - s * p.y),
            self.pin.y + (s * p.x + c * p.y),
        )
    }
}

/// The cloth on the heel's underside, placed on the heel's circle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeelCloth {
    /// How far behind the heel circle's center its back end stands, on its
    /// outer face, meters along the wippen.
    pub back: f64,
    /// How far in front of that center its front end stands, on its outer
    /// face, meters along the wippen.
    pub front: f64,
    /// Its thickness at its ends, meters, in along the circle's radius from
    /// its outer face.
    pub thickness: f64,
}

/// The heel under the wippen's body, placed by the trade's two quantities,
/// its position and its height, both read at the lowest point of its
/// circle, the circle of its cloth's outer face, the face the capstan bears
/// on. The circle's center follows from them ([`WippenHeel::center`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WippenHeel {
    /// Its position: how far along the wippen, at angle zero, its circle's
    /// lowest point stands in front of the center pin, meters.
    pub position: f64,
    /// Its height: how far its circle's lowest point stands below the
    /// jack's pin, meters.
    pub height: f64,
    /// The circle's radius, meters.
    pub radius: f64,
    /// The heel's top, the wippen body's bottom where it hangs, its height
    /// in the wippen's frame, meters.
    pub top: f64,
    /// How far behind the circle's center the heel's back side meets the
    /// body, meters along the wippen.
    pub back_top: f64,
    /// How far in front of it the heel's front side meets the body, meters
    /// along the wippen.
    pub front_top: f64,
    /// How far behind the circle's center the heel's back lower corner
    /// stands, meters along the wippen.
    pub back: f64,
    /// How far in front of it the heel's front lower corner stands, meters
    /// along the wippen.
    pub front: f64,
    /// Its cloth.
    pub cloth: HeelCloth,
}

/// The points that place the heel and its cloth, in the wippen's frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeelPoints {
    /// The cloth's outer face, an arc of the heel's circle from its back
    /// end to its front end.
    pub face: Seg,
    /// The cloth's back end on its outer face: where the face starts.
    pub cloth_back: Point,
    /// The cloth's front end on its outer face: where the face ends.
    pub cloth_front: Point,
    /// The cloth's back end at its top.
    pub cloth_back_top: Point,
    /// The cloth's front end at its top.
    pub cloth_front_top: Point,
    /// The heel's back lower corner.
    pub back: Point,
    /// The heel's front lower corner.
    pub front: Point,
    /// Where the heel's back side meets the body.
    pub back_top: Point,
    /// Where its front side meets the body.
    pub front_top: Point,
}

impl WippenHeel {
    /// Its circle's center in the wippen's frame, the jack's pin standing at
    /// `jack_pin`: at its position along the wippen, and its radius above
    /// its lowest point, which stands its height below the jack's pin. The
    /// jack's pin is the height's datum, so moving the pin moves the heel.
    pub fn center(&self, jack_pin: Point) -> Point {
        Point::new(self.position, jack_pin.y - self.height + self.radius)
    }

    /// The point of the heel's circle about `c`, `reach` along the wippen
    /// from its center, under it; `None` where the circle does not reach
    /// there.
    fn under(&self, c: Point, reach: f64) -> Option<Point> {
        let r = self.radius;
        let squared = r * r - reach * reach;
        (squared >= 0.0).then(|| Point::new(c.x + reach, c.y - squared.sqrt()))
    }

    /// The points that place it, the jack's pin standing at `jack_pin`;
    /// `None` where the cloth's ends stand off the heel's circle, which
    /// [`crate::validate`] refuses.
    pub fn points(&self, jack_pin: Point) -> Option<HeelPoints> {
        let c = self.center(jack_pin);
        let (back, front) = (
            self.under(c, -self.cloth.back)?,
            self.under(c, self.cloth.front)?,
        );
        let face = Seg::Arc {
            c,
            r: self.radius,
            from: Dir::new(back.x - c.x, back.y - c.y)?,
            to: Dir::new(front.x - c.x, front.y - c.y)?,
        };
        let (cloth_back, cloth_front) = (face.start(), face.end());
        let inner = self.radius - self.cloth.thickness;
        let top_of = |p: Point| {
            let d = Dir::new(p.x - c.x, p.y - c.y)?.vector();
            Some(Point::new(c.x + inner * d.x, c.y + inner * d.y))
        };
        let slope = (cloth_front.y - cloth_back.y) / (cloth_front.x - cloth_back.x);
        let on_the_line = |x: f64| Point::new(x, cloth_back.y + slope * (x - cloth_back.x));
        Some(HeelPoints {
            face,
            cloth_back,
            cloth_front,
            cloth_back_top: top_of(cloth_back)?,
            cloth_front_top: top_of(cloth_front)?,
            back: on_the_line(c.x - self.back),
            front: on_the_line(c.x + self.front),
            back_top: Point::new(c.x - self.back_top, self.top),
            front_top: Point::new(c.x + self.front_top, self.top),
        })
    }
}

/// Everything about a wippen assembly that the choice of its parts fixes,
/// and where it stands over the key.
#[derive(Debug, Clone, PartialEq)]
pub struct WippenAssembly {
    /// Where the wippen's center pin stands, in the keybed's frame, meters:
    /// along the key from the balance pin, negative behind it, and above the
    /// keybed. Placed once, as the frame's rails are.
    pub center: Point,
    /// The wippen's body, traced, in its frame.
    pub body: Vec<Stretch>,
    /// The heel and its cloth.
    pub heel: WippenHeel,
    /// The jack.
    pub jack: PinnedPart,
    /// The repetition lever.
    pub repetition_lever: PinnedPart,
    /// The center pin's diameter, meters.
    pub center_pin_diameter: f64,
    /// The wippen's width across the keyboard, its heel's and its cloth's,
    /// and the center pin's, meters: what the solid view extrudes them by.
    pub width: f64,
    /// The friction of the wippen's center: its flange's bushings on the
    /// center pin.
    pub friction: AxisFriction,
}

/// What the wippen assembly's parts declare: where each is, and the ranges
/// of them anything else may read.
#[derive(Debug, Clone, PartialEq)]
pub struct WippenAssemblyRanges {
    /// The wippen's body.
    pub wippen: PartId,
    /// The heel.
    pub wippen_heel: PartId,
    /// The heel cloth.
    pub wippen_heel_cloth: PartId,
    /// The jack.
    pub jack: PartId,
    /// The repetition lever.
    pub repetition_lever: PartId,
    /// The center pin, on the ground.
    pub center_pin: PartId,
    /// The heel's outline below the wippen's body, back to front, one
    /// chain whose ranges join bit for bit: the heel's back side, from where
    /// it meets the body, and its bottom behind the cloth; the cloth's outer
    /// face; the heel's bottom in front of the cloth and its front side, up
    /// to where it meets the body.
    pub heel_below_the_body: [SurfaceRef; 3],
}

/// A setting of the wippen assembly: a length under a meter either way.
const fn assembly_length(
    name: &'static str,
    hard: (f64, f64),
    get: fn(&WippenAssembly) -> f64,
    set: fn(&mut WippenAssembly, f64),
) -> ParamSpec<WippenAssembly> {
    ParamSpec {
        name,
        quantity: Quantity::Length,
        hard,
        step: Step::Continuous,
        layer: Layer::Design,
        kind: Kind::Actuated,
        get,
        set,
    }
}

impl WippenAssembly {
    /// The assembly's settings: what an instrument may set on it, each with
    /// what it admits ([`ParamSpec`]).
    ///
    /// The wippen's center stands behind the balance pin and above the
    /// keybed, within a meter of it; the heel's contact surface stands in
    /// front of the center pin and no higher than the jack's pin, each
    /// within a meter of it, its radius and its cloth's thickness no less
    /// than nothing and under a meter. Setting the radius keeps the heel's
    /// contact where it stands, its position and its height, and moves its
    /// circle's center by the change;
    /// whether the cloth fits in its heel, and the heel's circle has a
    /// radius, is [`crate::validate`]'s to say. The friction of the
    /// wippen's center is a torque, no less than nothing.
    pub const PARAMS: [ParamSpec<Self>; 7] = [
        assembly_length(
            "wippen.center.x",
            (-1.0, 0.0),
            |a| a.center.x,
            |a, v| a.center.x = v,
        ),
        assembly_length(
            "wippen.center.y",
            (0.0, 1.0),
            |a| a.center.y,
            |a, v| a.center.y = v,
        ),
        assembly_length(
            "wippen_heel.position",
            (0.0, 1.0),
            |a| a.heel.position,
            |a, v| a.heel.position = v,
        ),
        assembly_length(
            "wippen_heel.height",
            (0.0, 1.0),
            |a| a.heel.height,
            |a, v| a.heel.height = v,
        ),
        assembly_length(
            "wippen_heel.radius",
            (0.0, 1.0),
            |a| a.heel.radius,
            |a, v| a.heel.radius = v,
        ),
        assembly_length(
            "wippen_heel_cloth.thickness",
            (0.0, 1.0),
            |a| a.heel.cloth.thickness,
            |a, v| a.heel.cloth.thickness = v,
        ),
        ParamSpec {
            name: "wippen.friction.bound",
            quantity: Quantity::Torque,
            hard: (0.0, f64::INFINITY),
            step: Step::Continuous,
            layer: Layer::Design,
            kind: Kind::Actuated,
            get: |a| a.friction.bound_n_m,
            set: |a, v| {
                a.friction.bound_n_m = v;
                a.friction.radius_m = None;
            },
        },
    ];

    /// The body's outline, segments in the wippen's frame.
    pub fn body_outline(&self) -> Vec<Seg> {
        traced(&self.body, &|p| p)
    }

    /// The jack's outline, segments in the wippen's frame, at its rest angle.
    pub fn jack_outline(&self) -> Vec<Seg> {
        traced(&self.jack.outline, &|p| self.jack.on_the_wippen(p))
    }

    /// The repetition lever's outline, segments in the wippen's frame, at
    /// its rest angle.
    pub fn repetition_lever_outline(&self) -> Vec<Seg> {
        traced(&self.repetition_lever.outline, &|p| {
            self.repetition_lever.on_the_wippen(p)
        })
    }

    /// The heel's outline, segments in the wippen's frame, counter-clockwise
    /// from where its back side meets the body: down its back side to its
    /// back lower corner, along its bottom to the cloth, up the cloth's back
    /// end, along the cloth's top, down its front end, along the bottom to
    /// the front lower corner, up the front side, and back along the body's
    /// bottom. `None` where the cloth's ends stand off the heel's circle.
    pub fn heel_outline(&self) -> Option<Vec<Seg>> {
        let h = self.heel.points(self.jack.pin)?;
        Some(chain(&[
            h.back_top,
            h.back,
            h.cloth_back,
            h.cloth_back_top,
            h.cloth_front_top,
            h.cloth_front,
            h.front,
            h.front_top,
        ]))
    }

    /// The heel cloth's outline, segments in the wippen's frame,
    /// counter-clockwise from its back end on its outer face: its outer
    /// face, an arc, then up its front end, back along its top and down its
    /// back end. `None` where its ends stand off the heel's circle.
    pub fn heel_cloth_outline(&self) -> Option<Vec<Seg>> {
        let h = self.heel.points(self.jack.pin)?;
        let mut outline = vec![h.face];
        outline.extend(chain(&[
            h.cloth_front,
            h.cloth_front_top,
            h.cloth_back_top,
            h.cloth_back,
        ]));
        outline.pop();
        Some(outline)
    }

    /// Builds every part of the assembly onto `out`: the wippen's body, its
    /// heel and the heel cloth, the jack and the repetition lever carried by
    /// `wippen`, in its frame; the center pin carried by `ground`, in the
    /// keybed's frame. Returns what they declare.
    ///
    /// # Panics
    ///
    /// On a heel whose cloth stands off its circle, which
    /// [`crate::validate`] refuses before anything is built.
    pub fn parts(
        &self,
        wippen: BodyId,
        ground: BodyId,
        out: &mut Vec<Part>,
    ) -> WippenAssemblyRanges {
        let wood = |code: PartCode,
                    outline: Vec<Seg>,
                    placed_by: Vec<(core::ops::Range<u16>, PlacedBy)>,
                    width: f64| Part {
            body: wippen,
            code,
            outline,
            sweep: Sweep::Extruded { width, across: 0.0 },
            role: Role::Solid,
            material: Material::ActionWood,
            placed_by,
        };
        let id = next_id(out);
        let body = self.body_outline();
        let n = count(&body);
        out.push(wood(
            PartCode::Wippen,
            body,
            vec![(0..n, "wippen_outline")],
            self.width,
        ));

        let heel_id = next_id(out);
        let heel = self
            .heel_outline()
            .expect("a heel whose cloth stands on its circle");
        out.push(wood(
            PartCode::WippenHeel,
            heel,
            vec![
                (0..2, "wippen_heel.back"),
                (2..5, "wippen_heel_cloth.thickness"),
                (5..7, "wippen_heel.front"),
                (7..8, "wippen_heel.top"),
            ],
            self.width,
        ));

        let cloth_id = next_id(out);
        out.push(Part {
            body: wippen,
            code: PartCode::WippenHeelCloth,
            outline: self
                .heel_cloth_outline()
                .expect("a heel whose cloth stands on its circle"),
            sweep: Sweep::Extruded {
                width: self.width,
                across: 0.0,
            },
            role: Role::Lining,
            material: Material::HeelCloth,
            placed_by: vec![
                (0..1, "wippen_heel.radius"),
                (1..4, "wippen_heel_cloth.thickness"),
            ],
        });

        let jack_id = next_id(out);
        let jack = self.jack_outline();
        let n = count(&jack);
        out.push(wood(
            PartCode::Jack,
            jack,
            vec![(0..n, "jack_outline")],
            self.jack.width,
        ));

        let lever_id = next_id(out);
        let lever = self.repetition_lever_outline();
        let n = count(&lever);
        out.push(wood(
            PartCode::RepetitionLever,
            lever,
            vec![(0..n, "repetition_lever_outline")],
            self.repetition_lever.width,
        ));

        let pin_id = next_id(out);
        let r = 0.5 * self.center_pin_diameter;
        out.push(Part {
            body: ground,
            code: PartCode::CenterPin,
            outline: vec![
                Seg::Arc {
                    c: self.center,
                    r,
                    from: Dir::X,
                    to: Dir::NEG_X,
                },
                Seg::Arc {
                    c: self.center,
                    r,
                    from: Dir::NEG_X,
                    to: Dir::X,
                },
            ],
            sweep: Sweep::Extruded {
                width: self.width,
                across: 0.0,
            },
            role: Role::Solid,
            material: Material::Steel,
            placed_by: vec![(0..2, "center_pin.diameter")],
        });

        WippenAssemblyRanges {
            wippen: id,
            wippen_heel: heel_id,
            wippen_heel_cloth: cloth_id,
            jack: jack_id,
            repetition_lever: lever_id,
            center_pin: pin_id,
            heel_below_the_body: [
                SurfaceRef {
                    part: heel_id,
                    segs: 0..2,
                },
                SurfaceRef {
                    part: cloth_id,
                    segs: 0..1,
                },
                SurfaceRef {
                    part: heel_id,
                    segs: 5..7,
                },
            ],
        }
    }
}

/// The closed polygon through `points`, in order, as lines.
fn chain(points: &[Point]) -> Vec<Seg> {
    (0..points.len())
        .map(|i| Seg::Line {
            a: points[i],
            b: points[(i + 1) % points.len()],
        })
        .collect()
}

/// A traced outline as segments, each point carried by `place`: each
/// stretch from where it starts to where the next starts, an arc's ends on
/// its circle in the directions of its two points. The segments chain bit
/// for bit: a line runs from the end of the segment before it to the start
/// of the one after it, and between two arcs whose ends do not meet to the
/// bit, a line joins them, as long as the rounding that parts them.
pub(crate) fn traced(stretches: &[Stretch], place: &dyn Fn(Point) -> Point) -> Vec<Seg> {
    let n = stretches.len();
    let starts: Vec<Point> = stretches.iter().map(|s| place(s.start())).collect();
    let arcs: Vec<Option<Seg>> = (0..n)
        .map(|i| match stretches[i] {
            Stretch::Line(_) => None,
            Stretch::Arc { center, radius, .. } => {
                let c = place(center);
                let (a, b) = (starts[i], starts[(i + 1) % n]);
                Some(Seg::Arc {
                    c,
                    r: radius,
                    from: Dir::new(a.x - c.x, a.y - c.y).unwrap_or(Dir::X),
                    to: Dir::new(b.x - c.x, b.y - c.y).unwrap_or(Dir::X),
                })
            }
        })
        .collect();
    let joint = |i: usize| match (arcs[i], arcs[(i + n - 1) % n]) {
        (Some(arc), _) => arc.start(),
        (None, Some(before)) => before.end(),
        (None, None) => starts[i],
    };
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let next = (i + 1) % n;
        match arcs[i] {
            Some(arc) => {
                out.push(arc);
                if let Some(after) = arcs[next] {
                    if arc.end() != after.start() {
                        out.push(Seg::Line {
                            a: arc.end(),
                            b: after.start(),
                        });
                    }
                }
            }
            None => out.push(Seg::Line {
                a: joint(i),
                b: joint(next),
            }),
        }
    }
    out
}

/// One part's mass, as a source publishes it: its mass, its center of mass
/// in its own frame, and its inertia about that center, which the statics
/// do not read.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartMass {
    /// Its mass, kilograms.
    pub mass: f64,
    /// Its center of mass, meters in its own frame.
    pub center: Point,
    /// Its inertia about its center of mass, kg·m²: kept for the dynamics.
    pub inertia: f64,
}

/// What is known of a wippen assembly's mass.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WippenAssemblyMass {
    /// Each of its three parts weighed and located by a published source:
    /// the wippen's body with its heel, the jack and the repetition lever,
    /// each in its own frame.
    Published {
        /// The wippen's body, its heel and its cloth with it.
        wippen: PartMass,
        /// The jack.
        jack: PartMass,
        /// The repetition lever.
        repetition_lever: PartMass,
    },
}

impl WippenAssemblyMass {
    /// The parts it is known by, each with what it is.
    pub fn parts(&self) -> [(PartCode, PartMass); 3] {
        match *self {
            Self::Published {
                wippen,
                jack,
                repetition_lever,
            } => [
                (PartCode::Wippen, wippen),
                (PartCode::Jack, jack),
                (PartCode::RepetitionLever, repetition_lever),
            ],
        }
    }

    /// The assembly's mass and its center of mass in the wippen's frame,
    /// the jack and the repetition lever at their rest angles on it: what
    /// the statics read of the wippen.
    pub fn props(&self, assembly: &WippenAssembly) -> MassProps {
        let [(_, wippen), (_, jack), (_, lever)] = self.parts();
        let placed = [
            (wippen.mass, wippen.center),
            (jack.mass, assembly.jack.on_the_wippen(jack.center)),
            (
                lever.mass,
                assembly.repetition_lever.on_the_wippen(lever.center),
            ),
        ];
        let mass = placed.iter().map(|(m, _)| m).sum::<f64>();
        let along = placed.iter().map(|(m, c)| m * c.x).sum::<f64>();
        let across = placed.iter().map(|(m, c)| m * c.y).sum::<f64>();
        MassProps {
            mass,
            com: Point::new(along / mass, across / mass),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixture::{
        wippen_assembly, wippen_assembly_mass, HEEL, JACK_BUTTON, JACK_PIN, JACK_REST_ANGLE,
        JACK_STOP, REPETITION_LEVER_BUTTON, REPETITION_LEVER_REST_ANGLE, REPETITION_LEVER_STOP,
    };

    const WIPPEN: BodyId = BodyId(3);
    const GROUND: BodyId = BodyId::GROUND;

    fn built() -> (Vec<Part>, WippenAssemblyRanges) {
        let mut parts = Vec::new();
        let ranges = wippen_assembly().parts(WIPPEN, GROUND, &mut parts);
        (parts, ranges)
    }

    /// Twice the signed area of a part's drawn polygon, positive when
    /// counter-clockwise.
    fn twice_area(part: &Part) -> f64 {
        let points: Vec<Point> = part
            .outline
            .iter()
            .flat_map(|seg| seg.drawn(crate::geometry::DRAWN_HALVINGS))
            .collect();
        (0..points.len())
            .map(|i| {
                let (p, q) = (points[i], points[(i + 1) % points.len()]);
                p.x * q.y - q.x * p.y
            })
            .sum()
    }

    /// Consistency: **every part of the wippen assembly is closed bit for
    /// bit and turns counter-clockwise, every stretch of it placed by one
    /// number**: the body, the heel and its cloth, the jack and the
    /// repetition lever on the wippen, the center pin on the ground; the
    /// traced outlines' arcs joined to their neighbors bit for bit.
    #[test]
    fn every_part_of_the_wippen_assembly_is_closed_and_counter_clockwise() {
        let (parts, ranges) = built();
        let codes: Vec<PartCode> = parts.iter().map(|p| p.code).collect();
        assert_eq!(
            codes,
            [
                PartCode::Wippen,
                PartCode::WippenHeel,
                PartCode::WippenHeelCloth,
                PartCode::Jack,
                PartCode::RepetitionLever,
                PartCode::CenterPin,
            ]
        );
        for part in &parts {
            assert!(part.is_closed(), "{:?} is open", part.code);
            assert!(twice_area(part) > 0.0, "{:?} turns clockwise", part.code);
            let covered: u16 = part.placed_by.iter().map(|(r, _)| r.end - r.start).sum();
            assert_eq!(usize::from(covered), part.outline.len(), "{:?}", part.code);
            let body = if part.code == PartCode::CenterPin {
                GROUND
            } else {
                WIPPEN
            };
            assert_eq!(part.body, body, "{:?}", part.code);
        }
        assert_eq!(ranges.center_pin, PartId(5));
    }

    /// Consistency: **the heel cloth's outer face lies on the heel's
    /// circle**, from its back end to its front end, each the reach the
    /// heel gives from the circle's center along the wippen, within the
    /// roundings of a point placed on a circle; and the heel's outline below
    /// the body runs as one chain from its back side to its front side, the
    /// heel's back side and bottom on to the cloth's face and on to the
    /// heel's bottom and front side again, bit for bit.
    #[test]
    fn the_heel_cloths_outer_face_lies_on_the_heels_circle() {
        let (parts, ranges) = built();
        let assembly = wippen_assembly();
        let heel = assembly.heel;
        let center = heel.center(assembly.jack.pin);
        let [behind, face, ahead] = ranges.heel_below_the_body.clone().map(|r| r.segs(&parts));
        let Seg::Arc { c, r, .. } = face[0] else {
            panic!("the cloth's face is an arc: {face:?}");
        };
        assert_eq!(c, center);
        assert_eq!(r.to_bits(), heel.radius.to_bits());
        // A point placed on the circle: a few roundings of the reach.
        let within = 8.0 * f64::EPSILON * (center.x.abs() + heel.radius);
        assert!((face[0].start().x - (center.x - heel.cloth.back)).abs() <= within);
        assert!((face[0].end().x - (center.x + heel.cloth.front)).abs() <= within);
        for pair in behind.windows(2).chain(ahead.windows(2)) {
            assert_eq!(pair[0].end(), pair[1].start());
        }
        let last_behind = behind.last().expect("the heel's back side and bottom");
        assert_eq!(last_behind.end(), face[0].start());
        assert_eq!(face[0].end(), ahead[0].start());
    }

    /// Consistency: **the heel circle's lowest point stands at the heel's
    /// position along the wippen and its height below the jack's pin, and
    /// a change of the heel's radius leaves it there**, the circle's center
    /// moved by the change, by hand: the cloth's face drawn about a center
    /// at the position, and the radius above the height below the pin,
    /// within the roundings of that sum and difference of lengths of a
    /// decimeter.
    #[test]
    fn the_heels_lowest_point_stands_at_its_position_and_height_whatever_its_radius() {
        let radius_setting = WippenAssembly::PARAMS
            .iter()
            .find(|spec| spec.name == "wippen_heel.radius")
            .expect("the heel's radius setting");
        let mut assembly = wippen_assembly();
        let lowest = |a: &WippenAssembly| {
            let mut parts = Vec::new();
            let ranges = a.parts(WIPPEN, GROUND, &mut parts);
            let face = ranges.heel_below_the_body[1].segs(&parts)[0];
            let Seg::Arc { c, r, .. } = face else {
                panic!("the cloth's face is an arc: {face:?}");
            };
            (c, Point::new(c.x, c.y - r))
        };
        let within = 4.0 * f64::EPSILON * 0.1;
        let (center_before, before) = lowest(&assembly);
        let more = 0.002;
        let larger = assembly.heel.radius + more;
        radius_setting
            .apply(&mut assembly, larger)
            .expect("a larger heel circle");
        let (center_after, after) = lowest(&assembly);
        for point in [before, after] {
            assert_eq!(point.x.to_bits(), HEEL.position.to_bits());
            let height = JACK_PIN.y - point.y;
            assert!((height - HEEL.height).abs() <= within, "{height}");
        }
        assert!((center_after.y - center_before.y - more).abs() <= within);
    }

    /// The gap between a stop, a line on the wippen through a point with
    /// its normal, and the circle of a part pinned on the wippen that rests
    /// on it, at the part's angle `angle`, meters.
    fn gap(part: &PinnedPart, angle: f64, stop: (Point, Point), button: (Point, f64)) -> f64 {
        let at = PinnedPart {
            rest_angle: angle,
            ..part.clone()
        }
        .on_the_wippen(button.0);
        let n = Dir::new(stop.1.x, stop.1.y).expect("a normal").vector();
        (at.x - stop.0.x) * n.x + (at.y - stop.0.y) * n.y - button.1
    }

    /// Consistency: **the jack and the repetition lever rest on their
    /// stops**: at the rest angles the trace of Hirschkorn's action closes
    /// contacts 5 and 6 at, the jack's button touches the jack's stop and
    /// the lever's button the lever's, within what the angle's last digit
    /// leaves: half a unit of its fourth decimal times the button's center's
    /// distance from its pin.
    #[test]
    fn the_jack_and_the_repetition_lever_rest_on_their_stops() {
        let a = wippen_assembly();
        for (part, angle, stop, button) in [
            (&a.jack, JACK_REST_ANGLE, JACK_STOP, JACK_BUTTON),
            (
                &a.repetition_lever,
                REPETITION_LEVER_REST_ANGLE,
                REPETITION_LEVER_STOP,
                REPETITION_LEVER_BUTTON,
            ),
        ] {
            let arm = (button.0.x * button.0.x + button.0.y * button.0.y).sqrt();
            let digit = 0.5e-4 * arm;
            assert!(
                gap(part, angle, stop, button).abs() <= digit,
                "{} um",
                gap(part, angle, stop, button) * 1e6
            );
            assert!(gap(part, angle - 2e-4, stop, button).abs() > digit);
        }
    }

    /// Consistency: **the assembly weighs its three parts at rest**: Table
    /// A.1's 11.96 g of wippen, 2.65 g of jack and 3.71 g of repetition
    /// lever, 18.32 g, its center where `tools/derivations/12` puts it with
    /// the jack and the lever at their rest angles, (56.57, 6.52) mm in the
    /// wippen's frame, to the script's 0.005 mm.
    #[test]
    fn the_assembly_weighs_its_three_parts_at_rest() {
        let props = wippen_assembly_mass().props(&wippen_assembly());
        assert!((props.mass - 0.01832).abs() <= 4.0 * f64::EPSILON * 0.01832);
        assert!((props.com.x - 0.05657).abs() <= 0.005e-3, "{:?}", props.com);
        assert!((props.com.y - 0.00652).abs() <= 0.005e-3, "{:?}", props.com);
    }

    /// Consistency: **a traced outline runs through its points**: the
    /// test wippen's body, each line from where it starts to where the next
    /// starts, each arc's ends on its circle, joined bit for bit, and two
    /// arcs whose ends part by a rounding joined by a line as short.
    #[test]
    fn a_traced_outline_runs_through_its_points() {
        let stretches = wippen_assembly().body;
        let segs = traced(&stretches, &|p| p);
        let starts: Vec<Point> = segs.iter().map(Seg::start).collect();
        for s in &stretches {
            let p = s.start();
            let nearest = starts
                .iter()
                .map(|q| ((q.x - p.x) * (q.x - p.x) + (q.y - p.y) * (q.y - p.y)).sqrt())
                .fold(f64::INFINITY, f64::min);
            // An arc's start is its circle's point in the direction of the
            // traced point: off it by the trace's rounding to 0.01 mm.
            assert!(nearest <= 0.01e-3, "{p:?} is {nearest} m off the outline");
        }
        let two_arcs = [
            Stretch::Arc {
                from: Point::new(0.0, -0.001),
                center: Point::new(0.0, 0.0),
                radius: 0.001,
            },
            Stretch::Arc {
                from: Point::new(0.0, 0.001),
                center: Point::new(0.0, 0.000_000_1),
                radius: 0.000_9,
            },
        ];
        let joined = traced(&two_arcs, &|p| p);
        assert_eq!(joined.len(), 4, "{joined:?}");
        for i in 0..joined.len() {
            assert_eq!(joined[i].end(), joined[(i + 1) % joined.len()].start());
        }
    }
}

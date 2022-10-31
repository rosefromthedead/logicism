use std::{collections::BTreeMap, rc::Rc, sync::atomic::AtomicUsize};

use druid::{
    im, Affine, BoxConstraints, Color, Data, MouseButton, Point, Rect, RenderContext, Selector,
    Size, Vec2, Widget, WidgetId, WidgetPod,
};

use crate::{
    component::{Component, ComponentInstance, ComponentState, ComponentType, Orientation},
    wire::{Wire, WireSegment, WireState},
};

#[derive(Clone, Data)]
pub enum WireDraw {
    FromComponent { id: usize, pin: usize, loc: Coords },
    FromWire { id: usize, loc: Coords },
}

impl WireDraw {
    fn start_point(&self) -> Coords {
        match self {
            WireDraw::FromComponent { loc, .. } | WireDraw::FromWire { loc, .. } => *loc,
        }
    }
}

pub const DESELECT_ALL: Selector<WidgetId> = Selector::new("logicism/deselect-all");
pub const BEGIN_WIRE_DRAW: Selector<WireDraw> = Selector::new("logicism/begin-wire-draw");

static NEXT_ITEM_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Data, Debug, PartialEq, Eq)]
pub struct Coords {
    pub x: isize,
    pub y: isize,
}

impl Coords {
    pub fn new(x: isize, y: isize) -> Self {
        Coords { x, y }
    }

    pub fn from_widget_space(pos: Point) -> Self {
        Coords {
            x: (pos.x / 16.).round() as isize,
            y: (pos.y / 16.).round() as isize,
        }
    }

    pub fn to_widget_space(&self) -> Point {
        Point::new((self.x * 16) as f64, (self.y * 16) as f64)
    }

    pub fn from_canvas_space(pos: Point) -> Self {
        Coords {
            x: ((pos.x - 8.) / 16.).round() as isize,
            y: ((pos.y - 8.) / 16.).round() as isize,
        }
    }

    pub fn to_canvas_space(&self) -> Point {
        self.to_widget_space() + Vec2::new(8.0, 8.0)
    }
}

impl std::ops::Add<Coords> for Coords {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Coords {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::AddAssign<Coords> for Coords {
    fn add_assign(&mut self, rhs: Coords) {
        *self = *self + rhs
    }
}

#[derive(Clone, Data)]
pub enum Tool {
    Hand,
    Place(Rc<ComponentType>, Orientation),
}

#[derive(Clone, Data)]
pub struct CanvasState {
    wires: im::OrdMap<usize, WireState>,
    components: im::OrdMap<usize, ComponentState>,
    tool: Tool,
    mouse_pos: Option<Coords>,
    last_orientation: Orientation,
    drawing: Option<WireDraw>,
}

impl CanvasState {
    pub fn new() -> Self {
        CanvasState {
            wires: im::OrdMap::new(),
            components: im::OrdMap::new(),
            tool: Tool::Hand,
            mouse_pos: None,
            last_orientation: Orientation::North,
            drawing: None,
        }
    }
}

pub struct Canvas {
    component_types: Rc<Vec<Rc<ComponentType>>>,
    wires: BTreeMap<usize, WidgetPod<WireState, Wire>>,
    components: BTreeMap<usize, WidgetPod<ComponentState, Component>>,
}

impl Canvas {
    pub fn new(component_types: Rc<Vec<Rc<ComponentType>>>) -> Self {
        Canvas {
            component_types,
            wires: BTreeMap::new(),
            components: BTreeMap::new(),
        }
    }
}

impl Widget<CanvasState> for Canvas {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut CanvasState,
        env: &druid::Env,
    ) {
        for (id, widget) in self.wires.iter_mut() {
            let state = data.wires.get_mut(id).unwrap();
            widget.event(ctx, event, state, env);
        }

        for (id, widget) in self.components.iter_mut() {
            let state = data.components.get_mut(id).unwrap();
            widget.event(ctx, event, state, env);
        }

        if ctx.is_handled() {
            return;
        }

        use druid::keyboard_types::Key;
        use druid::Event::*;
        match (event, &mut data.tool) {
            (WindowConnected, _) => ctx.request_focus(),
            (KeyDown(key_event), tool) => {
                let mut new_tool = tool.clone();
                match (&key_event.key, &*tool) {
                    (Key::Character(ref s), _) if s == " " => new_tool = Tool::Hand,
                    // once again foiled by other languages existing
                    (Key::Character(ref s), _)
                        if s.len() == 1 && s.chars().next().unwrap().is_digit(10) =>
                    {
                        let n = u16::from_str_radix(&s, 10).unwrap().wrapping_sub(1) as usize;
                        if n < self.component_types.len() {
                            new_tool = Tool::Place(
                                Rc::clone(&self.component_types[n]),
                                data.last_orientation,
                            );
                        }
                    },
                    (Key::Character(ref s), &Tool::Place(ref ty, _)) if s == "w" => {
                        new_tool = Tool::Place(Rc::clone(&ty), Orientation::North)
                    },
                    (Key::Character(ref s), &Tool::Place(ref ty, _)) if s == "a" => {
                        new_tool = Tool::Place(Rc::clone(&ty), Orientation::West)
                    },
                    (Key::Character(ref s), &Tool::Place(ref ty, _)) if s == "s" => {
                        new_tool = Tool::Place(Rc::clone(&ty), Orientation::South)
                    },
                    (Key::Character(ref s), &Tool::Place(ref ty, _)) if s == "d" => {
                        new_tool = Tool::Place(Rc::clone(&ty), Orientation::East)
                    },
                    _ => {},
                }
                if !Data::same(tool, &new_tool) {
                    *tool = new_tool;
                    ctx.request_paint();
                }
            },
            (MouseMove(m), Tool::Hand) => {
                let new_coords = Coords::from_canvas_space(m.pos);
                if data.mouse_pos != Some(new_coords) {
                    data.mouse_pos = Some(new_coords);
                    if data.drawing.is_some() {
                        ctx.request_paint();
                    }
                }
            },
            (MouseMove(m), Tool::Place(_, _)) => {
                let new_coords = Coords::from_canvas_space(m.pos);
                if data.mouse_pos != Some(new_coords) {
                    data.mouse_pos = Some(new_coords);
                    ctx.request_paint();
                }
            },
            (MouseDown(ev), Tool::Hand) if ev.button == MouseButton::Left => {
                ctx.submit_command(DESELECT_ALL.with(ctx.widget_id()));
            },
            (MouseUp(ev), Tool::Hand) if ev.button == MouseButton::Left => {
                if let Some(wire_draw) = &data.drawing {
                    if let Some(segment) =
                        WireSegment::new(wire_draw.start_point(), data.mouse_pos.unwrap())
                    {
                        // TODO: merge connected segments
                        let id = NEXT_ITEM_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let widget = WidgetPod::new(Wire(id));
                        self.wires.insert(id, widget);
                        let state = WireState {
                            segments: im::Vector::from(&[segment][..]),
                        };
                        data.wires.insert(id, state);
                        ctx.children_changed();
                    }
                }
                data.drawing = None;
            },
            (MouseDown(ev), Tool::Place(ty, orientation)) if ev.button == MouseButton::Left => {
                let coords = Coords::from_canvas_space(ev.pos);
                let id = NEXT_ITEM_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                self.components.insert(id, WidgetPod::new(Component(id)));
                data.components.insert(
                    id,
                    ComponentState::new(coords, Rc::clone(&ty), *orientation),
                );
                ctx.children_changed();
                ctx.request_paint();
            },
            (Command(c), _) if c.is(BEGIN_WIRE_DRAW) => {
                let wire_draw = c.get(BEGIN_WIRE_DRAW).unwrap().clone();
                data.drawing = Some(wire_draw);
            },
            _ => {},
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &CanvasState,
        env: &druid::Env,
    ) {
        use druid::LifeCycle::*;
        match event {
            // TODO: doesn't seem to do anything, why??? instead we use WindowConnected event
            // handler
            WidgetAdded => ctx.register_for_focus(),
            _ => {},
        }

        for (widget, data) in self.wires.values_mut().zip(data.wires.values()) {
            widget.lifecycle(ctx, event, data, env);
        }

        for (widget, data) in self.components.values_mut().zip(data.components.values()) {
            widget.lifecycle(ctx, event, data, env);
        }
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        old_data: &CanvasState,
        data: &CanvasState,
        env: &druid::Env,
    ) {
        for (widget, new, old) in self
            .wires
            .iter_mut()
            .map(|(id, widget)| (widget, &data.wires[id], old_data.wires.get(id)))
        {
            widget.update(ctx, new, env);
            if let Some(old) = old {
                if !Data::same(&new.segments, &old.segments) {
                    ctx.request_layout();
                }
            }
        }

        for (widget, new, old) in self
            .components
            .iter_mut()
            .map(|(id, widget)| (widget, &data.components[id], old_data.components.get(id)))
        {
            widget.update(ctx, new, env);
            if let Some(old) = old {
                if !Data::same(&new.instance, &old.instance) {
                    ctx.request_layout();
                }
            }
        }
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &BoxConstraints,
        data: &CanvasState,
        env: &druid::Env,
    ) -> Size {
        for (widget, data) in self.wires.values_mut().zip(data.wires.values()) {
            widget.set_origin(ctx, data, env, data.bounding_rect().origin());
            widget.layout(ctx, &BoxConstraints::UNBOUNDED, data, env);
        }

        for (widget, data) in self.components.values_mut().zip(data.components.values()) {
            widget.set_origin(ctx, data, env, data.instance.bounding_rect().origin());
            widget.layout(ctx, &BoxConstraints::UNBOUNDED, data, env);
        }

        bc.max()
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &CanvasState, env: &druid::Env) {
        let size = ctx.size();

        // dots
        for x in 0..size.width as usize / 16 {
            for y in 0..size.height as usize / 16 {
                ctx.fill(
                    Rect::from_center_size(
                        Point::new(x as f64 * 16.0 + 8.0, y as f64 * 16.0 + 8.0),
                        Size::new(2.0, 2.0),
                    ),
                    &Color::GRAY,
                );
            }
        }

        // cursor ghost
        if let Tool::Place(ref ty, orientation) = data.tool {
            if let Some(c) = data.mouse_pos {
                let component = ComponentInstance::new(c, Rc::clone(&ty), orientation);
                ctx.with_save(|ctx| {
                    ctx.transform(Affine::translate(
                        component.bounding_rect().origin() - Point::ORIGIN,
                    ));
                    component.paint(ctx);
                });
            }
        }

        // drawing wire
        if let Some(wire_draw) = &data.drawing {
            let start_point = wire_draw.start_point();
            // snap start->mouse_pos line to compass directions
            let mut mouse_pos = data.mouse_pos.unwrap();
            let is_horizontal_draw =
                mouse_pos.x.abs_diff(start_point.x) > mouse_pos.y.abs_diff(start_point.y);
            if is_horizontal_draw {
                mouse_pos.y = start_point.y;
            } else {
                mouse_pos.x = start_point.x;
            }

            // unwrap: since we just snapped mouse_pos, it can't be None
            let segment = WireSegment::new(start_point, mouse_pos).unwrap();
            ctx.with_save(|ctx| {
                ctx.transform(Affine::translate(
                    segment.bounding_rect().origin() - Point::ORIGIN,
                ));
                segment.paint(ctx);
            });
        }

        for (widget, data) in self.wires.values_mut().zip(data.wires.values()) {
            widget.paint(ctx, data, env);
        }

        for (widget, data) in self.components.values_mut().zip(data.components.values()) {
            widget.paint(ctx, data, env);
        }
    }
}

use std::rc::Rc;

use druid::{
    im::Vector, Affine, BoxConstraints, Color, Data, MouseButton, Point, Rect, RenderContext,
    Selector, Size, Vec2, Widget, WidgetId, WidgetPod,
};

use crate::component::{Component, ComponentInstance, ComponentState, ComponentType, Orientation};

pub const BEGIN_DRAG: Selector<Point> = Selector::new("logicism/begin-drag");
pub const DESELECT_ALL: Selector<WidgetId> = Selector::new("logicism/deselect-all");

#[derive(Clone, Copy, Data, Debug, PartialEq, Eq)]
pub struct Coords {
    x: isize,
    y: isize,
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

#[derive(Clone, Data)]
pub enum Tool {
    Hand,
    Place(Rc<ComponentType>, Orientation),
}

#[derive(Clone, Data)]
struct Dragging {
    pub component: ComponentState,
    pub mouse_offset: Vec2,
}

#[derive(Clone, Data)]
pub struct CanvasState {
    components: Vector<ComponentState>,
    tool: Tool,
    mouse_pos: Option<Coords>,
    last_orientation: Orientation,
}

impl CanvasState {
    pub fn new() -> Self {
        CanvasState {
            components: Vector::new(),
            tool: Tool::Hand,
            mouse_pos: None,
            last_orientation: Orientation::North,
        }
    }
}

pub struct Canvas {
    component_types: Rc<Vec<Rc<ComponentType>>>,
    component_widgets: Vec<WidgetPod<ComponentState, Component>>,
}

impl Canvas {
    pub fn new(component_types: Rc<Vec<Rc<ComponentType>>>) -> Self {
        Canvas {
            component_types,
            component_widgets: Vec::new(),
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
        for (widget, state) in self
            .component_widgets
            .iter_mut()
            .zip(data.components.iter_mut())
        {
            widget.event(ctx, event, state, env);
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
                }
            },
            (MouseMove(m), Tool::Place(_, _)) => {
                let new_coords = Coords::from_canvas_space(m.pos);
                if data.mouse_pos != Some(new_coords) {
                    data.mouse_pos = Some(new_coords);
                    ctx.request_paint();
                }
            },
            (MouseDown(ev), Tool::Hand) if ev.button == MouseButton::Left && !ctx.is_handled() => {
                ctx.submit_command(DESELECT_ALL.with(ctx.widget_id()));
            },
            (MouseDown(ev), Tool::Place(ty, orientation)) if ev.button == MouseButton::Left => {
                let coords = Coords::from_canvas_space(ev.pos);
                data.components.push_back(ComponentState::new(
                    coords,
                    Rc::clone(&ty),
                    *orientation,
                ));
                self.component_widgets.push(WidgetPod::new(Component));
                ctx.children_changed();
                // if we don't return, then we end up passing an event to an uninit widget
                return;
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

        for (widget, state) in self
            .component_widgets
            .iter_mut()
            .zip(data.components.iter())
        {
            widget.lifecycle(ctx, event, state, env);
        }
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        old_data: &CanvasState,
        data: &CanvasState,
        env: &druid::Env,
    ) {
        for (widget, (new, old)) in self
            .component_widgets
            .iter_mut()
            .zip(data.components.iter().zip(old_data.components.iter()))
        {
            widget.update(ctx, new, env);
            if !Data::same(&new.instance, &old.instance) {
                ctx.request_layout();
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
        for (widget, data) in self
            .component_widgets
            .iter_mut()
            .zip(data.components.iter())
        {
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

        // components
        for (widget, state) in self
            .component_widgets
            .iter_mut()
            .zip(data.components.iter())
        {
            widget.paint(ctx, state, env);
        }
    }
}

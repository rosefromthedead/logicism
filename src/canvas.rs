use std::rc::Rc;

use druid::{
    im::Vector, Affine, Color, Data, MouseButton, Point, Rect, RenderContext, Size, Vec2, Widget,
};

use crate::component::{ComponentType, Orientation};

#[derive(Clone, Data)]
pub enum Tool {
    Hand,
    Place(Rc<ComponentType>, Orientation),
}

impl PartialEq for Tool {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Place(l_ty, l_o), Self::Place(r_ty, r_o)) => {
                Rc::ptr_eq(l_ty, r_ty) && l_o == r_o
            },
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}

impl Eq for Tool {}

#[derive(Clone, Data)]
struct Component {
    x: usize,
    y: usize,
    ty: Rc<ComponentType>,
    orientation: Orientation,
}

impl Component {
    fn new(x: usize, y: usize, ty: Rc<ComponentType>, orientation: Orientation) -> Self {
        Component {
            x,
            y,
            ty,
            orientation,
        }
    }

    pub fn bounding_rect(&self) -> Rect {
        self.ty.bounding_rect(self.x, self.y, self.orientation)
    }

    pub fn anchor_position(&self) -> Point {
        Canvas::coords_to_widget_space(self.x, self.y)
    }

    fn widget_transform(&self) -> Affine {
        let recenter = match self.orientation {
            Orientation::North => Affine::translate(Vec2::ZERO),
            Orientation::East => Affine::translate(Vec2::new(self.ty.size.width, 0.0)),
            Orientation::South => {
                Affine::translate(Vec2::new(self.ty.size.width, self.ty.size.height))
            },
            Orientation::West => Affine::translate(Vec2::new(0.0, self.ty.size.height)),
        };
        Affine::translate(self.bounding_rect().origin() - Point::ORIGIN)
            * recenter
            * Affine::rotate(self.orientation.angle())
    }
}

#[derive(Clone, Data)]
struct Dragging {
    pub component: Component,
    pub mouse_offset: Vec2,
}

#[derive(Clone, Data)]
pub struct CanvasState {
    components: Vector<Component>,
    tool: Tool,
    dragging: Option<Dragging>,
    mouse_pos: Option<(usize, usize)>,
    last_orientation: Orientation,
}

impl CanvasState {
    pub fn new() -> Self {
        CanvasState {
            components: Vector::new(),
            tool: Tool::Hand,
            dragging: None,
            mouse_pos: None,
            last_orientation: Orientation::North,
        }
    }
}

pub struct Canvas {
    component_types: Rc<Vec<Rc<ComponentType>>>,
}

impl Canvas {
    pub fn new(component_types: Rc<Vec<Rc<ComponentType>>>) -> Self {
        Canvas { component_types }
    }

    pub fn widget_space_to_coords(pos: Point) -> (usize, usize) {
        (
            ((pos.x - 8.) / 16.).round() as usize,
            ((pos.y - 8.) / 16.).round() as usize,
        )
    }

    pub fn coords_to_widget_space(x: usize, y: usize) -> Point {
        Point::new((x * 16 + 8) as f64, (y * 16 + 8) as f64)
    }
}

impl Widget<CanvasState> for Canvas {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut CanvasState,
        _env: &druid::Env,
    ) {
        use druid::keyboard_types::Key;
        use druid::Event::*;
        match (event, &mut data.tool) {
            (WindowConnected, _) => ctx.request_focus(),
            (MouseMove(m), Tool::Hand) => {
                let new_coords = Self::widget_space_to_coords(m.pos);
                if data.mouse_pos != Some(new_coords) {
                    data.mouse_pos = Some(new_coords);
                }

                if let Some(ref mut dragging) = data.dragging {
                    let c = &mut dragging.component;
                    let new_coords = Self::widget_space_to_coords(m.pos - dragging.mouse_offset);
                    if c.x != new_coords.0 || c.y != new_coords.1 {
                        c.x = new_coords.0;
                        c.y = new_coords.1;
                        ctx.request_paint();
                    }
                }
            },
            (MouseMove(m), Tool::Place(_, _)) => {
                let new_coords = Self::widget_space_to_coords(m.pos);
                if data.mouse_pos != Some(new_coords) {
                    data.mouse_pos = Some(new_coords);
                    ctx.request_paint();
                }
            },
            (MouseDown(ev), Tool::Hand) if ev.button == MouseButton::Left => {
                // if we iterate backwards then we can find the most recently placed one
                if let Some((i, _)) = data
                    .components
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, c)| c.bounding_rect().contains(ev.pos))
                {
                    ctx.set_active(true);
                    let component = data.components.remove(i);
                    let difference = ev.pos - component.anchor_position();
                    let dragging = Dragging {
                        component,
                        mouse_offset: difference,
                    };
                    data.dragging = Some(dragging);
                }
            },
            (MouseUp(ev), Tool::Hand) if ev.button == MouseButton::Left => {
                if let Some(dragging) = data.dragging.take() {
                    data.components.push_back(dragging.component);
                }
            },
            (MouseDown(ev), Tool::Place(ty, orientation)) if ev.button == MouseButton::Left => {
                match ev.button {
                    druid::MouseButton::Left => {
                        if let Some((x, y)) = data.mouse_pos {
                            data.components.push_back(Component::new(
                                x,
                                y,
                                Rc::clone(&ty),
                                *orientation,
                            ));
                        }
                    },
                    _ => {},
                }
            },
            (KeyDown(key_event), tool) => {
                let mut new_tool = tool.clone();
                match (&key_event.key, &tool) {
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
                    (Key::Character(ref s), &&mut Tool::Place(ref ty, _)) if s == "w" => {
                        new_tool = Tool::Place(Rc::clone(&ty), Orientation::North)
                    },
                    (Key::Character(ref s), &&mut Tool::Place(ref ty, _)) if s == "a" => {
                        new_tool = Tool::Place(Rc::clone(&ty), Orientation::West)
                    },
                    (Key::Character(ref s), &&mut Tool::Place(ref ty, _)) if s == "s" => {
                        new_tool = Tool::Place(Rc::clone(&ty), Orientation::South)
                    },
                    (Key::Character(ref s), &&mut Tool::Place(ref ty, _)) if s == "d" => {
                        new_tool = Tool::Place(Rc::clone(&ty), Orientation::East)
                    },
                    _ => {},
                }
                if *tool != new_tool {
                    *tool = new_tool;
                    ctx.request_paint();
                }
            },
            _ => {},
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        _data: &CanvasState,
        _env: &druid::Env,
    ) {
        use druid::LifeCycle::*;
        match event {
            // TODO: doesn't seem to do anything, why??? instead we use WindowConnected event
            // handler
            WidgetAdded => ctx.register_for_focus(),
            _ => {},
        }
    }

    fn update(
        &mut self,
        _ctx: &mut druid::UpdateCtx,
        _old_data: &CanvasState,
        _data: &CanvasState,
        _env: &druid::Env,
    ) {
    }

    fn layout(
        &mut self,
        _ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        _data: &CanvasState,
        _env: &druid::Env,
    ) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &CanvasState, _env: &druid::Env) {
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
            if let Some((x, y)) = data.mouse_pos {
                let component = Component::new(x, y, Rc::clone(&ty), orientation);
                component.ty.icon.to_piet(component.widget_transform(), ctx);
            }
        }

        // components
        for c in data.components.iter() {
            c.ty.icon.to_piet(c.widget_transform(), ctx);
        }

        // dragging
        if let Some(ref dragging) = data.dragging {
            dragging
                .component
                .ty
                .icon
                .to_piet(dragging.component.widget_transform(), ctx);
        }
    }
}

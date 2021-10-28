use std::rc::Rc;

use druid::{
    im::Vector, widget::SvgData, Affine, Color, Data, MouseButton, Point, Rect, RenderContext,
    Size, Vec2, Widget,
};

#[derive(Clone, Copy, Data, Debug, PartialEq, Eq)]
pub enum Tool {
    Hand,
    Place(u16),
}

#[derive(Clone, Data)]
struct Component {
    x: usize,
    y: usize,
    ty: u16,
}

impl Component {
    fn new(x: usize, y: usize, ty: u16) -> Self {
        Component { x, y, ty }
    }

    pub fn bounding_rect(&self) -> Rect {
        let center = Point::new(self.x as f64 * 16.0, self.y as f64 * 16.0);
        Rect::from_center_size(center, Size::new(48., 48.))
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
}

impl CanvasState {
    pub fn new() -> Self {
        CanvasState {
            components: Vector::new(),
            tool: Tool::Hand,
            dragging: None,
            mouse_pos: None,
        }
    }
}

pub struct Canvas {
    component_icons: Rc<Vec<SvgData>>,
}

impl Canvas {
    pub fn new(component_icons: Rc<Vec<SvgData>>) -> Self {
        Canvas { component_icons }
    }

    pub fn mouse_to_coords(mouse_pos: Point) -> (usize, usize) {
        (
            ((mouse_pos.x - 8.) / 16.).round() as usize,
            ((mouse_pos.y - 8.) / 16.).round() as usize,
        )
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
        match (event, data.tool) {
            (WindowConnected, _) => ctx.request_focus(),
            (MouseMove(m), Tool::Hand) => {
                if let Some(ref mut dragging) = data.dragging {
                    let new_coords = Self::mouse_to_coords(m.pos + dragging.mouse_offset);
                    if dragging.component.x != new_coords.0 || dragging.component.y != new_coords.1
                    {
                        dragging.component.x = new_coords.0;
                        dragging.component.y = new_coords.1;
                        ctx.request_paint();
                    }
                }
            },
            (MouseMove(m), Tool::Place(_)) => {
                let new_coords = Self::mouse_to_coords(m.pos);
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
                    // fun magic number
                    let difference =
                        component.bounding_rect().center() - ev.pos + Vec2::new(0., 8.);
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
            (MouseDown(ev), Tool::Place(n)) if ev.button == MouseButton::Left => match ev.button {
                druid::MouseButton::Left => {
                    if let Some((x, y)) = data.mouse_pos {
                        data.components.push_back(Component::new(x, y, n));
                    }
                },
                _ => {},
            },
            (KeyDown(key_event), _) => {
                let mut new_tool = data.tool;
                match key_event.key {
                    Key::Character(ref s) if s == " " => new_tool = Tool::Hand,
                    // once again foiled by other languages existing
                    Key::Character(ref s)
                        if s.len() == 1 && s.chars().next().unwrap().is_digit(10) =>
                    {
                        let n = u16::from_str_radix(&s, 10).unwrap().wrapping_sub(1);
                        if (n as usize) < self.component_icons.len() {
                            new_tool = Tool::Place(n);
                        }
                    },
                    _ => {},
                }
                if data.tool != new_tool {
                    data.tool = new_tool;
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
        if let Tool::Place(n) = data.tool {
            if let Some(pos) = data.mouse_pos {
                self.component_icons[n as usize].to_piet(
                    Affine::translate(Vec2::new(
                        pos.0 as f64 * 16.0 - 16.0,
                        pos.1 as f64 * 16.0 - 24.0,
                    )),
                    ctx,
                );
            }
        }

        // components
        for c in data.components.iter() {
            let icon = &self.component_icons[c.ty as usize];
            icon.to_piet(
                Affine::translate(Vec2::new(
                    c.x as f64 * 16.0 - 16.0,
                    c.y as f64 * 16.0 - 24.0,
                )),
                ctx,
            );
        }

        // dragging
        if let Some(ref dragging) = data.dragging {
            self.component_icons[dragging.component.ty as usize].to_piet(
                Affine::translate(Vec2::new(
                    dragging.component.x as f64 * 16.0 - 16.0,
                    dragging.component.y as f64 * 16.0 - 24.0,
                )),
                ctx,
            );
        }
    }
}

use std::{hash::Hasher, rc::Rc};

use iced::{
    mouse::Interaction, svg::Handle, Background, Color, Length, Point, Rectangle, Size, Vector,
};
use iced_graphics::{Backend, Primitive, Renderer};
use iced_native::{event::Status, layout::Node, Widget};

use super::Message;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tool {
    Hand,
    Place(usize),
}

#[derive(Clone)]
pub struct Component {
    pub x: usize,
    pub y: usize,
    pub ty: usize,
}

impl Component {
    pub fn new(x: usize, y: usize, ty: usize) -> Self {
        Component { x, y, ty }
    }

    pub fn bounding_rect(&self) -> Rectangle {
        let top_left = Point::new(self.x as f32 * 16. - 16., self.y as f32 * 16. - 24.);
        Rectangle::new(top_left, Size::new(48., 48.))
    }
}

#[derive(Clone)]
pub struct Dragging {
    pub component: Component,
    pub mouse_offset: Vector,
}

#[derive(Clone)]
pub struct CanvasState {
    pub components: Vec<Component>,
    pub tool: Tool,
    pub dragging: Option<Dragging>,
}

impl CanvasState {
    pub fn new() -> Self {
        CanvasState {
            components: Vec::new(),
            tool: Tool::Hand,
            dragging: None,
        }
    }
}

pub struct Canvas {
    state: CanvasState,
    component_icons: Rc<Vec<Handle>>,
}

impl Canvas {
    pub fn new(state: CanvasState, component_icons: Rc<Vec<Handle>>) -> Self {
        Canvas {
            state,
            component_icons,
        }
    }

    pub fn mouse_to_coords(mouse_pos: Point) -> (usize, usize) {
        (
            ((mouse_pos.x - 8.) / 16.).round() as usize,
            ((mouse_pos.y - 8.) / 16.).round() as usize,
        )
    }
}

impl<B> Widget<Message, Renderer<B>> for Canvas
where
    B: Backend,
{
    fn width(&self) -> iced::Length {
        Length::Fill
    }

    fn height(&self) -> iced::Length {
        Length::Fill
    }

    fn layout(
        &self,
        _renderer: &Renderer<B>,
        limits: &iced_native::layout::Limits,
    ) -> iced_native::layout::Node {
        Node::new(limits.max())
    }

    fn draw(
        &self,
        _renderer: &mut Renderer<B>,
        _defaults: &<iced_graphics::Renderer<B> as iced_native::Renderer>::Defaults,
        layout: iced_native::Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> <iced_graphics::Renderer<B> as iced_native::Renderer>::Output {
        let size = layout.bounds().size();
        let mut primitives = Vec::new();

        // dots
        for x in 0..size.width as usize / 16 {
            for y in 0..size.height as usize / 16 {
                let top_left = Point {
                    x: (16 * x + 7) as f32,
                    y: (16 * y + 7) as f32,
                };
                primitives.push(Primitive::Quad {
                    bounds: Rectangle::new(top_left, Size::new(2., 2.)),
                    background: Background::Color(Color::new(0.8, 0.8, 0.8, 1.0)),
                    border_radius: 0.,
                    border_width: 0.,
                    border_color: Color::WHITE,
                });
            }
        }

        // cursor ghost
        if let Tool::Place(component_id) = self.state.tool {
            primitives.push(Primitive::Svg {
                handle: self.component_icons[component_id].clone(),
                bounds: Rectangle {
                    x: ((cursor_position.x - 8.) / 16.).round() * 16. - 16.,
                    y: ((cursor_position.y - 8.) / 16.).round() * 16. - 24.,
                    width: 48.,
                    height: 48.,
                },
            });
        }

        // components
        for c in self.state.components.iter() {
            primitives.push(Primitive::Svg {
                handle: self.component_icons[c.ty].clone(),
                bounds: c.bounding_rect(),
            });
        }

        if let Some(ref dragging) = self.state.dragging {
            primitives.push(Primitive::Svg {
                handle: self.component_icons[dragging.component.ty].clone(),
                bounds: dragging.component.bounding_rect(),
            });
        }

        let primitive = Primitive::Group { primitives };
        (primitive, Interaction::Idle)
    }

    fn hash_layout(&self, state: &mut iced_native::Hasher) {
        state.write_isize(0)
    }

    fn on_event(
        &mut self,
        event: iced_native::Event,
        _layout: iced_native::Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer<B>,
        _clipboard: &mut dyn iced_native::Clipboard,
        messages: &mut Vec<Message>,
    ) -> Status {
        use iced_native::event::Event::*;
        use iced_native::keyboard::Event::*;
        use iced_native::mouse::Button;
        use iced_native::mouse::Event::*;
        match (event, &self.state.tool) {
            (Mouse(ButtonPressed(Button::Left)), Tool::Hand) => {
                messages.push(Message::HandToolMouseDown(cursor_position));
            },
            (Mouse(ButtonReleased(Button::Left)), Tool::Hand) => {
                messages.push(Message::HandToolMouseUp(cursor_position));
            },
            (Mouse(CursorMoved { position }), Tool::Hand) => {
                if let Some(ref dragging) = self.state.dragging {
                    let new_coords = Self::mouse_to_coords(position + dragging.mouse_offset);
                    if new_coords != (dragging.component.x, dragging.component.y) {
                        messages.push(Message::Drag(new_coords));
                    }
                }
            },
            (Mouse(ButtonPressed(Button::Left)), Tool::Place(ty)) => {
                let (x, y) = Self::mouse_to_coords(cursor_position);
                messages.push(Message::AddComponent((x, y), *ty));
            },
            (Keyboard(CharacterReceived(' ')), _) => messages.push(Message::SwitchTool(Tool::Hand)),
            (Keyboard(CharacterReceived(c)), _) if c.is_digit(10) => {
                let n = (c.to_digit(10).unwrap() as usize).wrapping_sub(1);
                if n < self.component_icons.len() {
                    messages.push(Message::SwitchTool(Tool::Place(n)));
                }
            },
            _ => {},
        }

        Status::Captured
    }
}

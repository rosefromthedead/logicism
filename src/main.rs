use std::rc::Rc;

use iced::{executor, svg::Handle, Application, Command, Element, Point, Settings, Vector};

mod canvas;

use canvas::{Canvas, CanvasState, Component, Dragging};

fn main() -> Result<(), iced::Error> {
    App::run(Settings::default())
}

struct App {
    canvas_state: CanvasState,
    component_icons: Rc<Vec<Handle>>,
}

#[derive(Debug)]
enum Message {
    AddComponent((usize, usize), usize),
    HandToolMouseDown(Point),
    HandToolMouseUp(Point),
    Drag((usize, usize)),
    SwitchTool(canvas::Tool),
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            App {
                canvas_state: CanvasState::new(),
                component_icons: Rc::new(vec![
                    Handle::from_path("res/not_gate.svg"),
                    Handle::from_path("res/and_gate.svg"),
                    Handle::from_path("res/or_gate.svg"),
                    Handle::from_path("res/nand_gate.svg"),
                ]),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Logicism")
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> Command<Self::Message> {
        match message {
            Message::AddComponent((x, y), ty) => {
                self.canvas_state.components.push(Component::new(x, y, ty))
            },
            Message::HandToolMouseDown(pos) => {
                let mut found = false;
                let mut i = 0;
                while i < self.canvas_state.components.len() {
                    if self.canvas_state.components[i]
                        .bounding_rect()
                        .contains(pos)
                    {
                        found = true;
                        break;
                    }
                    i += 1;
                }

                if found {
                    let component = self.canvas_state.components.remove(i);
                    // fun magic number
                    let difference = component.bounding_rect().center() - pos + Vector::new(0., 8.);
                    let dragging = Dragging {
                        component,
                        mouse_offset: difference,
                    };
                    self.canvas_state.dragging = Some(dragging);
                }
            },
            Message::HandToolMouseUp(pos) => {
                if let Some(mut dragging) = self.canvas_state.dragging.take() {
                    let (x, y) = Canvas::mouse_to_coords(pos + dragging.mouse_offset);
                    dragging.component.x = x;
                    dragging.component.y = y;
                    self.canvas_state.components.push(dragging.component);
                }
            },
            Message::SwitchTool(tool) => {
                dbg!(&tool);
                self.canvas_state.tool = tool;
            },
            Message::Drag((x, y)) => {
                let c = &mut self.canvas_state.dragging.as_mut().unwrap().component;
                c.x = x;
                c.y = y;
            },
        }

        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        Element::new(Canvas::new(
            self.canvas_state.clone(),
            Rc::clone(&self.component_icons),
        ))
    }
}

use std::rc::Rc;

use component::ComponentType;
use druid::{Affine, AppLauncher, Widget, WindowDesc};

mod canvas;
mod component;

use canvas::{Canvas, CanvasState};

const IDENTITY: Affine = Affine::scale(1.0);

fn main() {
    let component_types = Rc::new(ComponentType::enumerate());

    let window = WindowDesc::new(move || root_widget(Rc::clone(&component_types)))
        .title("Logicism")
        .window_size((800.0, 600.0));

    AppLauncher::with_window(window)
        .launch(CanvasState::new())
        .expect("Failed to launch application");
}

fn root_widget(component_icons: Rc<Vec<Rc<ComponentType>>>) -> impl Widget<CanvasState> {
    Canvas::new(component_icons)
}

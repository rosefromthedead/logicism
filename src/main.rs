use std::{rc::Rc, str::FromStr};

use druid::{widget::SvgData, AppLauncher, Widget, WindowDesc};

mod canvas;

use canvas::{Canvas, CanvasState};

fn main() {
    let not_gate = SvgData::from_str(include_str!("../res/not_gate.svg")).unwrap();
    let and_gate = SvgData::from_str(include_str!("../res/and_gate.svg")).unwrap();
    let or_gate = SvgData::from_str(include_str!("../res/or_gate.svg")).unwrap();
    let nand_gate = SvgData::from_str(include_str!("../res/nand_gate.svg")).unwrap();
    let component_icons = Rc::new(vec![not_gate, and_gate, or_gate, nand_gate]);

    let window = WindowDesc::new(move || root_widget(Rc::clone(&component_icons)))
        .title("Logicism")
        .window_size((800.0, 600.0));

    AppLauncher::with_window(window)
        .launch(CanvasState::new())
        .expect("Failed to launch application");
}

fn root_widget(component_icons: Rc<Vec<SvgData>>) -> impl Widget<CanvasState> {
    Canvas::new(component_icons)
}

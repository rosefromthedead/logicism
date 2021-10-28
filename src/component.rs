use std::{rc::Rc, str::FromStr};

use druid::{widget::SvgData, Data, Rect, Size, Vec2};

use crate::canvas::Canvas;

#[derive(Clone, Copy, Data, PartialEq, Eq)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

impl Orientation {
    pub fn angle(&self) -> f64 {
        match self {
            Orientation::North => 0.0,
            Orientation::East => std::f64::consts::FRAC_PI_2,
            Orientation::South => std::f64::consts::PI,
            Orientation::West => std::f64::consts::FRAC_PI_2 * 3.0,
        }
    }
}

pub struct ComponentType {
    pub size: Size,
    /// The point that is represented by the coordinates of a component when it is oriented north
    anchor_offset: Vec2,
    pub icon: SvgData,
}

impl ComponentType {
    pub fn enumerate() -> Vec<Rc<Self>> {
        let not_gate = ComponentType {
            size: Size::new(24.0, 48.0),
            anchor_offset: Vec2::new(12.0, 32.0),
            icon: SvgData::from_str(include_str!("../res/not_gate.svg")).unwrap(),
        };
        let and_gate = ComponentType {
            size: Size::new(48.0, 48.0),
            anchor_offset: Vec2::new(24.0, 32.0),
            icon: SvgData::from_str(include_str!("../res/and_gate.svg")).unwrap(),
        };
        let or_gate = ComponentType {
            size: Size::new(48.0, 48.0),
            anchor_offset: Vec2::new(24.0, 32.0),
            icon: SvgData::from_str(include_str!("../res/or_gate.svg")).unwrap(),
        };
        let nand_gate = ComponentType {
            size: Size::new(48.0, 48.0),
            anchor_offset: Vec2::new(24.0, 32.0),
            icon: SvgData::from_str(include_str!("../res/nand_gate.svg")).unwrap(),
        };
        vec![
            Rc::new(not_gate),
            Rc::new(and_gate),
            Rc::new(or_gate),
            Rc::new(nand_gate),
        ]
    }

    pub fn anchor_offset(&self, orientation: Orientation) -> Vec2 {
        let a = self.anchor_offset;
        match orientation {
            Orientation::North => a,
            Orientation::East => Vec2::new(self.size.width - a.y, a.x),
            Orientation::South => Vec2::new(self.size.width - a.x, self.size.height - a.y),
            Orientation::West => Vec2::new(a.y, self.size.height - a.x),
        }
    }

    pub fn bounding_rect(&self, x: usize, y: usize, orientation: Orientation) -> Rect {
        let top_left = Canvas::coords_to_widget_space(x, y) - self.anchor_offset(orientation);
        let size = match orientation {
            Orientation::North | Orientation::South => self.size,
            Orientation::East | Orientation::West => Size::new(self.size.height, self.size.width),
        };
        Rect::from_origin_size(top_left, size)
    }
}

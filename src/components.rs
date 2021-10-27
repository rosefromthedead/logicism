use iced::{Color, Point};
use iced_graphics::canvas::{LineCap, LineJoin, Path, Stroke};

macro_rules! p {
    ($x:literal, $y:literal) => {
        Point::new($x as f32, $y as f32)
    };
}

pub fn default_stroke() -> Stroke {
    Stroke {
        color: Color::BLACK,
        width: 2.0,
        line_cap: LineCap::Square,
        line_join: LineJoin::Miter,
    }
}

pub fn not_gate() -> Path {
    Path::new(|b| {
        b.move_to(Point::new(34., 47.));
        b.line_to(Point::new(24., 12.));
        b.line_to(Point::new(14., 47.));
        b.line_to(Point::new(34., 47.));
        
        b.circle(Point::new(24., 6.), 5.);
    })
}

pub fn and_gate() -> Path {
    Path::new(|b| {
        b.move_to(p!(1, 47));
        b.line_to(p!(47, 47));
        b.line_to(p!(47, 24));
        b.arc_to(p!(47, 24), p!(1, 24), 23.);
        b.line_to(p!(1, 47));
    })
}

pub fn or_gate() -> Path {
    Path::new(|b| {
        b.move_to(p!(1, 24));
        b.arc_to(p!(1, 24), p!(36, 24), )
    })
}

pub fn and_gate() -> Path {
    Path::new(|b| {
        b.move_to(p!(1, 47));
        b.line_to(p!(47, 47));
        b.line_to(p!(47, 24));
        b.arc_to(p!(47, 24), p!(1, 24), 23.);
        b.line_to(p!(1, 47));
    })
}

pub fn and_gate() -> Path {
    Path::new(|b| {
        b.move_to(p!(1, 47));
        b.line_to(p!(47, 47));
        b.line_to(p!(47, 24));
        b.arc_to(p!(47, 24), p!(1, 24), 23.);
        b.line_to(p!(1, 47));
    })
}

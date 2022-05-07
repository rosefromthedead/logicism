use druid::{im, Color, Data, PaintCtx, Point, Rect, RenderContext, Size, Widget};

use crate::canvas::Coords;

#[derive(Clone, Data)]
pub struct WireSegment {
    start: Coords,
    end: Coords,
}

impl WireSegment {
    pub fn new(start: Coords, end: Coords) -> Option<Self> {
        if start.x != end.x && start.y != end.y {
            None
        } else {
            Some(WireSegment { start, end })
        }
    }

    pub fn bounding_rect(&self) -> Rect {
        let start = self.start.to_widget_space();
        let end = self.end.to_widget_space();
        Rect::from_points(start, end).inflate(2.0, 2.0)
    }

    pub fn paint(&self, ctx: &mut PaintCtx) {
        let start = self.start.to_widget_space();
        let end = self.end.to_widget_space();
        let rect = Rect::from_points(start, end)
            .with_origin(Point::new(10.0, 10.0))
            .inflate(1.0, 1.0);
        ctx.fill(rect, &Color::GREEN);
        ctx.fill(
            Rect::from_origin_size(Point::ORIGIN, Size::new(2.0, 2.0)),
            &Color::RED,
        );
    }
}

#[derive(Clone, Data)]
pub struct WireState {
    pub segments: im::Vector<WireSegment>,
}

impl WireState {
    pub fn bounding_rect(&self) -> Rect {
        self.segments
            .iter()
            .map(WireSegment::bounding_rect)
            .reduce(|a, b| a.union(b))
            .unwrap()
    }
}

pub struct Wire(pub usize);

impl Widget<WireState> for Wire {
    fn event(
        &mut self,
        _ctx: &mut druid::EventCtx,
        _event: &druid::Event,
        _data: &mut WireState,
        _env: &druid::Env,
    ) {
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut druid::LifeCycleCtx,
        _event: &druid::LifeCycle,
        _data: &WireState,
        _env: &druid::Env,
    ) {
    }

    fn update(
        &mut self,
        _ctx: &mut druid::UpdateCtx,
        _old_data: &WireState,
        _data: &WireState,
        _env: &druid::Env,
    ) {
    }

    fn layout(
        &mut self,
        _ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &WireState,
        _env: &druid::Env,
    ) -> druid::Size {
        bc.constrain(data.bounding_rect().size())
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &WireState, _env: &druid::Env) {
        ctx.with_save(|ctx| {
            for segment in data.segments.iter() {
                segment.paint(ctx);
            }
        })
    }
}

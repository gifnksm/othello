use super::OthelloDisk;
use crate::model::{Board, Point};
use conrod_core::{
    builder_methods,
    color::{self, Color, Colorable},
    widget::{self, Circle, Common, CommonBuilder, Matrix, UpdateArgs},
    widget_ids, Borderable, Positionable, Scalar, Sizeable, Widget, WidgetStyle,
};

#[derive(Debug)]
pub struct OthelloBoard<'a> {
    common: CommonBuilder,
    style: Style,
    board: &'a Board,
    show_candidates: bool,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {
    #[conrod(default = "color::WHITE")]
    pub white_color: Option<Color>,
    #[conrod(default = "color::BLACK")]
    pub black_color: Option<Color>,
    #[conrod(default = "theme.background_color")]
    pub background_color: Option<Color>,
    #[conrod(default = "theme.border_width")]
    pub border: Option<Scalar>,
    #[conrod(default = "theme.border_color")]
    pub border_color: Option<Color>,
    #[conrod(default = "0.5")]
    pub radius_ratio: Option<Scalar>,
    #[conrod(default = "6.0")]
    pub dot_radius: Option<Scalar>,
}

widget_ids! {
    #[derive(Clone, Debug, PartialEq)]
    struct Ids {
        matrix,
        dot_ul,
        dot_ur,
        dot_dl,
        dot_dr,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    ids: Ids,
}

impl<'a> OthelloBoard<'a> {
    pub fn new(board: &'a Board, show_candidates: bool) -> Self {
        OthelloBoard {
            common: CommonBuilder::default(),
            style: Style::default(),
            board,
            show_candidates,
        }
    }

    builder_methods! {
        pub white_color { style.white_color = Some(Color) }
        pub black_color { style.black_color = Some(Color) }
        pub background_color { style.background_color = Some(Color) }
        pub radius_ratio { style.radius_ratio = Some(Scalar) }
        pub dot_radius { style.dot_radius = Some(Scalar) }
    }
}

impl<'a> Common for OthelloBoard<'a> {
    fn common(&self) -> &CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonBuilder {
        &mut self.common
    }
}

impl<'a> Widget for OthelloBoard<'a> {
    type State = State;
    type Style = Style;
    type Event = Option<Point>;

    fn init_state(&self, id_gen: widget::id::Generator<'_>) -> State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Style {
        self.style
    }

    fn update(self, args: UpdateArgs<'_, '_, '_, '_, Self>) -> Self::Event {
        let UpdateArgs {
            id,
            state,
            rect,
            ui,
            style,
            ..
        } = args;
        let (w, h) = rect.w_h();

        let size = self.board.size();
        let mut elements = Matrix::new(size.0 as usize, size.1 as usize)
            .middle_of(id)
            .w_h(w, h)
            .set(state.ids.matrix, ui);

        let mut result = None;
        let cands = self.board.move_candidates();
        while let Some(element) = elements.next(ui) {
            let pt = Point(element.col as u32, element.row as u32);
            let mut disk = OthelloDisk::new();
            if let Some(turn) = self.board.turn() {
                if self.show_candidates && cands.contains(pt, size) {
                    disk = disk.flow_disk(turn);
                }
            }
            if let Some(side) = self.board.get(pt) {
                disk = disk.disk(side);
            }
            disk = disk
                .background_color(style.background_color(ui.theme()))
                .border(style.border(ui.theme()))
                .border_color(style.border_color(ui.theme()))
                .white_color(style.white_color(ui.theme()))
                .black_color(style.black_color(ui.theme()))
                .radius_ratio(style.radius_ratio(ui.theme()));

            let clicked = element.set(disk, ui);
            if clicked {
                result = Some(pt);
            }
        }

        if size.0 >= 4 && size.1 >= 4 {
            let cell_width = w / f64::from(size.0);
            let cell_height = h / f64::from(size.1);
            let (sx, sy) = (f64::from(size.0), f64::from(size.1));
            let pairs = &[
                (state.ids.dot_ul, (2.0, 2.0)),
                (state.ids.dot_ur, (2.0, sy - 2.0)),
                (state.ids.dot_dl, (sx - 2.0, 2.0)),
                (state.ids.dot_dr, (sx - 2.0, sy - 2.0)),
            ];
            for &(id, (dx, dy)) in pairs {
                Circle::fill(style.dot_radius(ui.theme()))
                    .x_y_relative_to(
                        state.ids.matrix,
                        -w / 2.0 + cell_width * dx,
                        -h / 2.0 + cell_height * dy,
                    )
                    .color(style.border_color(ui.theme()))
                    .set(id, ui);
            }
        }

        result
    }
}

impl<'a> Borderable for OthelloBoard<'a> {
    fn border(mut self, width: f64) -> Self {
        self.style.border = Some(width);
        self
    }

    fn border_color(mut self, color: Color) -> Self {
        self.style.border_color = Some(color);
        self
    }
}

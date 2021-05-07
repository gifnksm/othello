use super::OthelloDisk;
use crate::model::{PlayerKind, Side};
use conrod_core::color::{self, Color, Colorable};
use conrod_core::position::Dimension;
use conrod_core::widget::{self, BorderedRectangle, Common, CommonBuilder, Text, UpdateArgs};
use conrod_core::{builder_methods, widget_ids, WidgetStyle};
use conrod_core::{Borderable, FontSize, Positionable, Scalar, Sizeable, Ui, Widget};

#[derive(Debug)]
pub struct Indicator {
    common: CommonBuilder,
    style: Style,
    side: Side,
    kind: PlayerKind,
    num_disk: u32,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {
    #[conrod(default = "30")]
    pub player_name_font_size: Option<FontSize>,
    #[conrod(default = "60")]
    pub count_font_size: Option<FontSize>,
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
    #[conrod(default = "80.0")]
    pub cell_size: Option<Scalar>,
    #[conrod(default = "0.5")]
    pub radius_ratio: Option<Scalar>,
}

widget_ids! {
    #[derive(Clone, Debug, PartialEq)]
    struct Ids {
        rectangle,
        player_name,
        icon,
        count,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    ids: Ids,
}

impl<'a> Indicator {
    pub fn new(side: Side, kind: PlayerKind, num_disk: u32) -> Self {
        Indicator {
            common: CommonBuilder::default(),
            style: Style::default(),
            side: side,
            kind: kind,
            num_disk: num_disk,
        }
    }

    builder_methods! {
        pub player_name_font_size { style.player_name_font_size = Some(FontSize) }
        pub count_font_size { style.count_font_size = Some(FontSize) }
        pub white_color { style.white_color = Some(Color) }
        pub black_color { style.black_color = Some(Color) }
        pub background_color { style.background_color = Some(Color) }
        pub cell_size { style.cell_size = Some(Scalar) }
        pub radius_ratio { style.radius_ratio = Some(Scalar) }
    }
}

impl Common for Indicator {
    fn common(&self) -> &CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonBuilder {
        &mut self.common
    }
}

impl Widget for Indicator {
    type State = State;
    type Style = Style;
    type Event = ();

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
            mut ui,
            style,
            ..
        } = args;
        let dim = rect.dim();

        BorderedRectangle::new(dim)
            .middle_of(id)
            .graphics_for(id)
            .color(style.background_color(ui.theme()))
            .border(style.border(ui.theme()))
            .border_color(style.border_color(ui.theme()))
            .set(state.ids.rectangle, &mut ui);

        Text::new(self.kind.as_ref())
            .top_left_with_margin_on(id, 5.0)
            .w(dim[0])
            .font_size(style.player_name_font_size(ui.theme()))
            .set(state.ids.player_name, ui);

        let _ = OthelloDisk::new()
            .down_from(state.ids.player_name, 5.0)
            .w_h(style.cell_size(ui.theme()), style.cell_size(ui.theme()))
            .background_color(style.background_color(ui.theme()))
            .border(0.0)
            .white_color(style.white_color(ui.theme()))
            .black_color(style.black_color(ui.theme()))
            .radius_ratio(style.radius_ratio(ui.theme()))
            .disk(self.side)
            .set(state.ids.icon, ui);

        Text::new(&self.num_disk.to_string())
            .w(dim[0] - 10.0 - style.cell_size(ui.theme()))
            .right_from(state.ids.icon, 0.0)
            .font_size(style.count_font_size(ui.theme()))
            .right_justify()
            .set(state.ids.count, ui);
    }

    fn default_y_dimension(&self, ui: &Ui) -> Dimension {
        let name_height = Text::new(self.kind.as_ref())
            .w(self.get_w(ui).unwrap_or(0.0))
            .font_size(self.style.player_name_font_size(&ui.theme))
            .get_h(ui)
            .unwrap_or(0.0);
        Dimension::Absolute(name_height + self.style.cell_size(&ui.theme) + 15.0)
    }
}

impl Borderable for Indicator {
    fn border(mut self, width: f64) -> Self {
        self.style.border = Some(width);
        self
    }

    fn border_color(mut self, color: Color) -> Self {
        self.style.border_color = Some(color);
        self
    }
}

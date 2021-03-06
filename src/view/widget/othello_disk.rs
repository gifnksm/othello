use crate::model::Side;
use conrod_core::{
    builder_methods,
    color::{self, Color, Colorable},
    widget::{self, BorderedRectangle, Circle, Common, CommonBuilder, UpdateArgs},
    widget_ids, Borderable, Point, Positionable, Scalar, Widget, WidgetStyle,
};

#[derive(Debug)]
pub struct OthelloDisk {
    common: CommonBuilder,
    style: Style,
    disk: Option<Side>,
    flow_disk: Option<Side>,
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
}

widget_ids! {
    #[derive(Clone, Debug, PartialEq)]
    struct Ids {
        circle,
        rectangle
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    ids: Ids,
}

fn is_over_circ(circ_center: Point, mouse_point: Point, radius: Scalar) -> bool {
    let offset = vecmath::vec2_sub(mouse_point, circ_center);
    vecmath::vec2_len(offset) <= radius
}

impl OthelloDisk {
    pub fn new() -> Self {
        OthelloDisk {
            common: CommonBuilder::default(),
            style: Style::default(),
            disk: None,
            flow_disk: None,
        }
    }

    builder_methods! {
        pub white_color { style.white_color = Some(Color) }
        pub black_color { style.black_color = Some(Color) }
        pub background_color { style.background_color = Some(Color) }
        pub radius_ratio { style.radius_ratio = Some(Scalar) }
        pub disk { disk = Some(Side) }
        pub flow_disk { flow_disk = Some(Side) }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Interaction {
    Normal,
    Highlighted,
    Clicked,
}

impl Interaction {
    fn color(self, color: Color) -> Color {
        match self {
            Interaction::Normal => color,
            Interaction::Highlighted => color.highlighted(),
            Interaction::Clicked => color.clicked(),
        }
    }
}

impl Common for OthelloDisk {
    fn common(&self) -> &CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonBuilder {
        &mut self.common
    }
}

impl Widget for OthelloDisk {
    type State = State;
    type Style = Style;
    type Event = bool;

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
        let radius_ratio = style.radius_ratio(ui.theme());
        let radius = rect.w() * radius_ratio;
        let (interaction, clicked) = {
            let input = ui.widget_input(id);
            let clicked = input.clicks().left().next().is_some();

            let interaction = input
                .mouse()
                .and_then(|mouse| {
                    if is_over_circ([0.0, 0.0], mouse.rel_xy(), radius) {
                        if mouse.buttons.left().is_down() {
                            Some(Interaction::Clicked)
                        } else {
                            Some(Interaction::Highlighted)
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or(Interaction::Normal);

            (interaction, clicked)
        };

        BorderedRectangle::new(dim)
            .middle_of(id)
            .graphics_for(id)
            .color(style.background_color(ui.theme()))
            .border(style.border(ui.theme()))
            .border_color(style.border_color(ui.theme()))
            .set(state.ids.rectangle, &mut ui);

        let circle_param = if let Some(side) = self.disk {
            Some((false, side, None))
        } else {
            self.flow_disk.map(|side| (true, side, Some(0.3)))
        };

        if let Some((interactive, side, alpha)) = circle_param {
            let mut color = match side {
                Side::Black => style.black_color(&ui.theme),
                Side::White => style.white_color(&ui.theme),
            };

            if interactive {
                color = interaction.color(color);
            }

            if let Some(alpha) = alpha {
                color = color.alpha(alpha);
            }

            Circle::fill(radius)
                .middle_of(id)
                .graphics_for(id)
                .color(color)
                .set(state.ids.circle, &mut ui);
        }

        clicked
    }
}

impl Borderable for OthelloDisk {
    fn border(mut self, width: f64) -> Self {
        self.style.border = Some(width);
        self
    }

    fn border_color(mut self, color: Color) -> Self {
        self.style.border_color = Some(color);
        self
    }
}

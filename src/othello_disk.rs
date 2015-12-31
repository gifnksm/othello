use conrod::{CharacterCache, Circle, CommonBuilder, Dimensions, Frameable, FramedRectangle, IndexSlot, Point, Positionable, Scalar,
             Theme, UpdateArgs, Mouse, Widget, WidgetKind};
use conrod::color::{self, Color, Colorable};
use vecmath;

use super::Side;

#[derive(Debug)]
pub struct OthelloDisk<F> {
    common: CommonBuilder,
    maybe_react: Option<F>,
    style: Style,
    disk: Option<Side>,
    flow_disk: Option<Side>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    pub maybe_white_color: Option<Color>,
    pub maybe_black_color: Option<Color>,
    pub maybe_background_color: Option<Color>,
    pub maybe_frame: Option<f64>,
    pub maybe_frame_color: Option<Color>,
    pub maybe_radius: Option<Scalar>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    interaction: Interaction,
    circle_idx: IndexSlot,
    rectangle_idx: IndexSlot,
}

pub const KIND: WidgetKind = "OthelloDisk";

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Interaction {
    Normal,
    Highlighted,
    Clicked,
}

impl Interaction {
    fn color(&self, color: Color) -> Color {
        match *self {
            Interaction::Normal => color,
            Interaction::Highlighted => color.highlighted(),
            Interaction::Clicked => color.clicked(),
        }
    }
}

fn get_new_interaction(is_over: bool, prev: Interaction, mouse: Mouse) -> Interaction {
    use conrod::MouseButtonPosition::{Down, Up};
    use self::Interaction::{Normal, Highlighted, Clicked};
    match (is_over, prev, mouse.left.position) {
        // LMB is down over the button. But the button wasn't Highlighted last
        // update. This means the user clicked somewhere outside the button and
        // moved over the button holding LMB down. We do nothing in this case.
        (true, Normal, Down) => Normal,

        // LMB is down over the button. The button was either Highlighted or Clicked
        // last update. If it was highlighted before, that means the user clicked
        // just now, and we transition to the Clicked state. If it was clicked
        // before, that means the user is still holding LMB down from a previous
        // click, in which case the state remains Clicked.
        (true, _, Down) => Clicked,

        // LMB is up. The mouse is hovering over the button. Regardless of what the
        // state was last update, the state should definitely be Highlighted now.
        (true, _, Up) => Highlighted,

        // LMB is down, the mouse is not over the button, but the previous state was
        // Clicked. That means the user clicked the button and then moved the mouse
        // outside the button while holding LMB down. The button stays Clicked.
        (false, Clicked, Down) => Clicked,

        // If none of the above applies, then nothing interesting is happening with
        // this button.
        _ => Normal,
    }
}

fn is_over_circ(circ_center: Point, mouse_point: Point, dim: Dimensions) -> bool {
    let offset = vecmath::vec2_sub(mouse_point, circ_center);
    vecmath::vec2_len(offset) <= dim[0] / 2.0
}

impl<F> OthelloDisk<F> {
    pub fn new() -> Self {
        OthelloDisk {
            common: CommonBuilder::new(),
            maybe_react: None,
            style: Style::new(),
            disk: None,
            flow_disk: None,
        }
    }

    pub fn react(mut self, reaction: F) -> Self {
        self.maybe_react = Some(reaction);
        self
    }

    pub fn white_color(mut self, color: Color) -> Self {
        self.style.maybe_white_color = Some(color);
        self
    }

    pub fn black_color(mut self, color: Color) -> Self {
        self.style.maybe_black_color = Some(color);
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.style.maybe_background_color = Some(color);
        self
    }

    pub fn radius(mut self, radius: Scalar) -> Self {
        self.style.maybe_radius = Some(radius);
        self
    }

    pub fn disk(mut self, disk: Option<Side>) -> Self {
        self.disk = disk;
        self
    }

    pub fn flow_disk(mut self, flow_disk: Option<Side>) -> Self {
        self.flow_disk = flow_disk;
        self
    }
}

impl<F> Widget for OthelloDisk<F> where F: FnMut() {
    type State = State;
    type Style = Style;

    fn common(&self) -> &CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonBuilder {
        &mut self.common
    }

    fn unique_kind(&self) -> &'static str {
        KIND
    }

    fn init_state(&self) -> State {
        State {
            interaction: Interaction::Normal,
            circle_idx: IndexSlot::new(),
            rectangle_idx: IndexSlot::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn update<C: CharacterCache>(mut self, args: UpdateArgs<Self, C>) {
        let UpdateArgs { idx, state, rect, mut ui, style, .. } = args;
        let (xy, dim) = rect.xy_dim();
        let maybe_mouse = ui.input().maybe_mouse.map(|mouse| mouse.relative_to(xy));

        let new_interaction = match (self.disk.is_none(), maybe_mouse) {
            (false, _) | (true, None) => Interaction::Normal,
            (true, Some(mouse)) => {
                let is_over = is_over_circ([0.0, 0.0], mouse.xy, dim);

                get_new_interaction(is_over, state.view().interaction, mouse)
            }
        };

        if let (Interaction::Clicked, Interaction::Highlighted) = (state.view().interaction,
                                                                   new_interaction) {
            if let Some(ref mut react) = self.maybe_react {
                react();
            }
        }

        match (state.view().interaction, new_interaction) {
            (Interaction::Highlighted, Interaction::Clicked) => {
                ui.capture_mouse();
            }
            (Interaction::Clicked, Interaction::Highlighted) |
            (Interaction::Clicked, Interaction::Normal) => {
                ui.uncapture_mouse();
            }
            _ => {}
        }

        if state.view().interaction != new_interaction {
            state.update(|state| state.interaction = new_interaction);
        }

        let rectangle_idx = state.view().rectangle_idx.get(&mut ui);
        let dim = rect.dim();
        let background_color = style.background_color(ui.theme());
        let frame = style.frame(ui.theme());
        let frame_color = style.frame_color(ui.theme());
        FramedRectangle::new(dim)
            .middle_of(idx)
            .graphics_for(idx)
            .color(background_color)
            .frame(frame)
            .frame_color(frame_color)
            .set(rectangle_idx, &mut ui);

        if let Some(disk) = self.disk {
            let radius = style.radius(rect.w());
            let circle_color = match disk {
                Side::Black => new_interaction.color(style.black_color(ui.theme())),
                Side::White => new_interaction.color(style.white_color(ui.theme())),
            };
            let circle_idx = state.view().circle_idx.get(&mut ui);
            Circle::fill(radius)
                .middle_of(idx)
                .graphics_for(idx)
                .color(circle_color)
                .set(circle_idx, &mut ui);
        }

        if let Some(flow_disk) = self.flow_disk {
            let radius = style.radius(rect.w());
            let circle_color = match flow_disk {
                Side::Black => new_interaction.color(style.black_color(ui.theme())),
                Side::White => new_interaction.color(style.white_color(ui.theme())),
            }.alpha(0.5);
            let circle_idx = state.view().circle_idx.get(&mut ui);
            Circle::fill(radius)
                .middle_of(idx)
                .graphics_for(idx)
                .color(circle_color)
                .set(circle_idx, &mut ui);
        }
    }
}

impl Style {
    pub fn new() -> Style {
        Style {
            maybe_white_color: None,
            maybe_black_color: None,
            maybe_background_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_radius: None,
        }
    }

    pub fn white_color(&self, theme: &Theme) -> Color {
        self.maybe_white_color.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_white_color.unwrap_or(color::white())
        })).unwrap_or(color::white())
    }

    pub fn black_color(&self, theme: &Theme) -> Color {
        self.maybe_black_color.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_black_color.unwrap_or(color::black())
        })).unwrap_or(color::black())
    }

    pub fn background_color(&self, theme: &Theme) -> Color {
        self.maybe_background_color.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_background_color.unwrap_or(theme.background_color)
        })).unwrap_or(theme.background_color)
    }

    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    pub fn radius(&self, w: Scalar) -> Scalar {
        self.maybe_radius.unwrap_or(w / 2.0)
    }
}

impl<F> Frameable for OthelloDisk<F> {
    fn frame(mut self, width: f64) -> Self {
        self.style.maybe_frame = Some(width);
        self
    }

    fn frame_color(mut self, color: Color) -> Self {
        self.style.maybe_frame_color = Some(color);
        self
    }
}

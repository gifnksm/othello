#![warn(bad_style)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
// #![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]

#![cfg_attr(feature="dev", feature(plugin))]
#![cfg_attr(feature="dev", plugin(clippy))]
#![cfg_attr(feature="dev", warn(mut_mut))]
#![cfg_attr(feature="dev", warn(string_add))]
#![cfg_attr(feature="dev", warn(string_add_assign))]

extern crate board_game_geom as geom;
#[macro_use]
extern crate conrod;
extern crate find_folder;
extern crate piston_window;
extern crate rand;
extern crate vecmath;

use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};

use geom::{Point, Size};

use conrod::{Button, Canvas, Circle, DropDownList, Frameable, Labelable, LineStyle, Rectangle,
             Sizeable, Text, Theme, Widget, WidgetMatrix};
use conrod::color::{self, Color, Colorable};
use conrod::Positionable;

use piston_window::{Glyphs, PistonWindow, UpdateEvent, WindowSettings};

use board::Board;
use othello_disk::OthelloDisk;

type Ui = conrod::Ui<Glyphs>;

mod board;
mod othello_disk;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Side {
    Black,
    White,
}

impl Side {
    fn flip(self) -> Side {
        match self {
            Side::Black => Side::White,
            Side::White => Side::Black,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Board(Size, Vec<(Side, Point)>),
    Locate(Side, Point),
}

struct App {
    state: State,
    game_config: GameConfig,
    view_config: ViewConfig,
}

impl Default for App {
    fn default() -> App {
        App {
            state: State::Start(StartState::default()),
            game_config: GameConfig::default(),
            view_config: ViewConfig::default(),
        }
    }
}

enum State {
    Start(StartState),
    Play(PlayState),
}

macro_rules! impl_state {
    ($state_ty:ty, $state_name:ident) => {
        impl AsRef<$state_ty> for State {
            fn as_ref(&self) -> &$state_ty {
                match *self {
                    State::$state_name(ref s) => s,
                    _ => panic!(),
                }
            }
        }
        impl AsMut<$state_ty> for State {
            fn as_mut(&mut self) -> &mut $state_ty {
                match *self {
                    State::$state_name(ref mut s) => s,
                    _ => panic!(),
                }
            }
        }
    }
}

impl_state!(StartState, Start);
impl_state!(PlayState, Play);

#[derive(Clone, Debug)]
struct StartState {
    ddl_rows: DdlBuilder<BoardSize>,
    ddl_cols: DdlBuilder<BoardSize>,
    ddl_black_player: DdlBuilder<PlayerKind>,
    ddl_white_player: DdlBuilder<PlayerKind>,
}

impl Default for StartState {
    fn default() -> StartState {
        StartState {
            ddl_rows: DdlBuilder::new(),
            ddl_cols: DdlBuilder::new(),
            ddl_black_player: DdlBuilder::new(),
            ddl_white_player: DdlBuilder::new(),
        }
    }
}

#[derive(Clone, Debug)]
struct DdlBuilder<T> {
    strings: Vec<String>,
    selected_idx: Option<usize>,
    phantom: PhantomData<T>,
}

impl<T> DdlBuilder<T> {
    fn new() -> DdlBuilder<T>
        where T: DdlString + Default
    {
        let strings = T::create_strings();
        let default_str = T::default().to_ddl_string();
        let selected = strings.iter().position(|x| x == &default_str);
        DdlBuilder {
            strings: strings,
            selected_idx: selected,
            phantom: PhantomData,
        }
    }

    fn build_drop_down_list<F>(&mut self) -> DropDownList<F>
        where T: DdlString
    {
        DropDownList::new(&mut self.strings, &mut self.selected_idx)
    }
}

trait DdlString: Sized {
    fn from_ddl_str(s: &str) -> Option<Self>;
    fn to_ddl_string(&self) -> String;
    fn create_strings() -> Vec<String>;
}

struct PlayState {
    black_player: Option<Player>,
    white_player: Option<Player>,
    board: Board,
}

impl PlayState {
    fn new(size: Size, black_kind: PlayerKind, white_kind: PlayerKind) -> PlayState {
        let board = Board::new(size);
        let black_player = Player::new(black_kind, &board, Side::Black);
        let white_player = Player::new(white_kind, &board, Side::White);
        PlayState {
            board: board,
            black_player: black_player,
            white_player: white_player,
        }
    }

    fn has_player(&self, side: Side) -> bool {
        self.get_player(side).is_some()
    }

    fn get_player(&self, side: Side) -> &Option<Player> {
        match side {
            Side::Black => &self.black_player,
            Side::White => &self.white_player,
        }
    }

    fn listen_player(&mut self) {
        let turn = match self.board.turn() {
            Some(turn) => turn,
            None => {
                self.finish();
                return;
            }
        };

        let loc = if let Some(ref player) = *self.get_player(turn) {
            match player.receiver.try_recv() {
                Ok(loc) => loc,
                Err(TryRecvError::Empty) => return,
                Err(e) => panic!("error: {}", e),
            }
        } else {
            return;
        };

        if !self.locate(loc) {
            panic!("cannot locate: {:?}", loc);
        }
    }

    fn finish(&mut self) {
        if let Some(p) = mem::replace(&mut self.black_player, None) {
            p.finish();
        }
        if let Some(p) = mem::replace(&mut self.white_player, None) {
            p.finish();
        }
    }

    fn locate(&mut self, pt: Point) -> bool {
        let turn = match self.board.turn() {
            Some(turn) => turn,
            None => return false,
        };

        if !self.board.locate(pt) {
            return false;
        }

        if let Some(ref player) = *self.get_player(turn.flip()) {
            player.sender.send(Message::Locate(turn, pt)).unwrap();
        }

        true
    }
}

struct Player {
    handle: JoinHandle<()>,
    receiver: Receiver<Point>,
    sender: Sender<Message>,
}

impl Player {
    fn new(kind: PlayerKind, board: &Board, side: Side) -> Option<Player> {
        let ai_routine = match kind {
            PlayerKind::Human => return None,
            PlayerKind::AiRandom => random_player,
        };

        let (host_tx, player_rx) = mpsc::channel();
        let (player_tx, host_rx) = mpsc::channel();
        let handle = thread::spawn(move || ai_routine(side, player_tx, player_rx));

        let disks = board.create_disks();
        let message = Message::Board(board.size(), disks);

        host_tx.send(message).unwrap();

        Some(Player {
            handle: handle,
            receiver: host_rx,
            sender: host_tx,
        })
    }

    fn finish(self) {
        let _ = self.handle.join();
    }
}

fn random_player(side: Side, tx: Sender<Point>, rx: Receiver<Message>) {
    let mut rng = rand::thread_rng();

    let mut board = match rx.recv() {
        Ok(Message::Board(size, disks)) => Board::new_with_disks(size, disks),
        Ok(msg) => panic!("{:?}", msg),
        Err(e) => panic!("error: {}", e),
    };

    loop {
        match board.turn() {
            None => break,
            Some(turn) => {
                if turn != side {
                    match rx.recv() {
                        Ok(Message::Locate(_, pt)) => {
                            board.locate(pt);
                            continue;
                        }
                        Ok(msg) => panic!("{:?}", msg),
                        Err(e) => panic!("error: {}", e),
                    }
                } else {
                    let pt = {
                        let pts = board.points().filter(|&pt| board.can_locate(pt));
                        rand::sample(&mut rng, pts, 1)[0]
                    };
                    if !board.locate(pt) {
                        panic!("cannot locate: {:?}", pt);
                    }
                    tx.send(pt).unwrap();
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct GameConfig {
    rows: i32,
    cols: i32,
    black_player: PlayerKind,
    white_player: PlayerKind,
}

impl Default for GameConfig {
    fn default() -> GameConfig {
        GameConfig {
            rows: 8,
            cols: 8,
            black_player: PlayerKind::Human,
            white_player: PlayerKind::Human,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct BoardSize(i32);

impl Default for BoardSize {
    fn default() -> BoardSize {
        BoardSize(8)
    }
}

impl DdlString for BoardSize {
    fn from_ddl_str(s: &str) -> Option<BoardSize> {
        i32::from_str(s).ok().and_then(|size| {
            if size < 2 || size > 10 {
                None
            } else {
                Some(BoardSize(size))
            }
        })
    }

    fn to_ddl_string(&self) -> String {
        self.0.to_string()
    }

    fn create_strings() -> Vec<String> {
        (2..11).map(|n| n.to_string()).collect::<Vec<_>>()
    }
}

#[derive(Copy, Clone, Debug)]
enum PlayerKind {
    Human,
    AiRandom,
}

impl Default for PlayerKind {
    fn default() -> PlayerKind {
        PlayerKind::Human
    }
}

impl DdlString for PlayerKind {
    fn from_ddl_str(s: &str) -> Option<PlayerKind> {
        match s {
            "Human" => Some(PlayerKind::Human),
            "AI Random" => Some(PlayerKind::AiRandom),
            _ => None,
        }
    }

    fn to_ddl_string(&self) -> String {
        match *self {
            PlayerKind::Human => "Human".to_owned(),
            PlayerKind::AiRandom => "AI Random".to_owned(),
        }
    }

    fn create_strings() -> Vec<String> {
        vec![PlayerKind::Human.to_ddl_string(), PlayerKind::AiRandom.to_ddl_string()]
    }
}

#[derive(Copy, Clone, Debug)]
struct ViewConfig {
    frame_width: f64,
    cell_size: f64,
    disk_radius: f64,
    dot_radius: f64,
    board_margin: f64,
    indicator_text_width: f64,

    frame_color: Color,
    board_color: Color,
    white_color: Color,
    black_color: Color,
}

impl Default for ViewConfig {
    fn default() -> ViewConfig {
        ViewConfig {
            frame_width: 1.0,
            cell_size: 80.0,
            disk_radius: 32.0,
            dot_radius: 6.0,
            board_margin: 40.0,
            indicator_text_width: 90.0,

            frame_color: color::black(),
            board_color: color::rgba(0.0, 0.5, 0.0, 1.0),
            white_color: color::white(),
            black_color: color::black(),
        }
    }
}

fn main() {
    let window: PistonWindow = {
        WindowSettings::new("Othello", (1024, 768))
            .exit_on_esc(true)
            .vsync(true)
            .build()
            .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e))
    };

    let mut ui = {
        let assets = find_folder::Search::KidsThenParents(3, 5)
                         .for_folder("assets")
                         .unwrap();
        let font_path = &assets.join("FiraSans-Regular.ttf");
        let factory = window.factory.borrow().clone();
        let glyph_cache = Glyphs::new(font_path, factory).unwrap();
        let theme = Theme::default();
        Ui::new(glyph_cache, theme)
    };

    let app_ref = Rc::new(RefCell::new(App::default()));
    for event in window {
        ui.handle_event(&event);

        let _ = event.update(|_| ui.set_widgets(|ui| set_widgets(ui, app_ref.clone())));
        event.draw_2d(|c, g| ui.draw_if_changed(c, g));
    }
}

widget_ids! {
    CANVAS,

    START_BUTTON,
    TIMES_LABEL,
    ROWS_DDL,
    COLS_DDL,
    BLACK_PLAYER_DDL,
    WHITE_PLAYER_DDL,

    PLAY_CANVAS,
    BOARD,
    DOT with 4,
    INDICATOR_LABEL_ICON with 2,
    INDICATOR_LABEL_TEXT with 2,
}

fn set_widgets(ui: &mut Ui, app_ref: Rc<RefCell<App>>) {
    let func: fn(ui: &mut Ui, Rc<RefCell<App>>) = {
        let app = app_ref.deref().borrow();
        match app.state {
            State::Start(_) => set_widgets_start,
            State::Play(_) => set_widgets_play,
        }
    };
    func(ui, app_ref.clone())
}

fn set_widgets_start(ui: &mut Ui, app_ref: Rc<RefCell<App>>) {
    let (gc, vc) = {
        let app = app_ref.deref().borrow();
        (app.game_config, app.view_config)
    };

    Canvas::new().color(vc.board_color).scrolling(true).set(CANVAS, ui);
    Text::new(&"x")
        .w_h(30.0, 50.0)
        .font_size(40)
        .align_text_middle()
        .mid_top_with_margin_on(CANVAS, 40.0)
        .set(TIMES_LABEL, ui);

    {
        let mut app = app_ref.deref().borrow_mut();
        let mut rows = app.game_config.rows;
        let mut cols = app.game_config.cols;
        {
            let start: &mut StartState = app.state.as_mut();
            start.ddl_rows
                 .build_drop_down_list()
                 .w_h(50.0, 50.0)
                 .left_from(TIMES_LABEL, 30.0)
                 .label("Rows")
                 .react(|selected_idx: &mut Option<usize>, new_idx, string: &str| {
                     *selected_idx = Some(new_idx);
                     rows = i32::from_str(string).unwrap();
                 })
                 .set(ROWS_DDL, ui);
            start.ddl_cols
                 .build_drop_down_list()
                 .w_h(50.0, 50.0)
                 .right_from(TIMES_LABEL, 30.0)
                 .label("Cols")
                 .react(|selected_idx: &mut Option<usize>, new_idx, string: &str| {
                     *selected_idx = Some(new_idx);
                     cols = i32::from_str(string).unwrap();
                 })
                 .set(COLS_DDL, ui);
        }
        app.game_config.rows = rows;
        app.game_config.cols = cols;
    }

    {
        let mut app = app_ref.deref().borrow_mut();
        let mut black_player = app.game_config.black_player;
        let mut white_player = app.game_config.white_player;
        {
            let start: &mut StartState = app.state.as_mut();
            start.ddl_black_player
                 .build_drop_down_list()
                 .w_h(150.0, 50.0)
                 .down_from(TIMES_LABEL, 40.0)
                 .left_from(TIMES_LABEL, 30.0)
                 .label("Black Player")
                 .react(|selected_idx: &mut Option<usize>, new_idx, string: &str| {
                     *selected_idx = Some(new_idx);
                     black_player = PlayerKind::from_ddl_str(string).unwrap();
                 })
                 .set(BLACK_PLAYER_DDL, ui);
            start.ddl_white_player
                 .build_drop_down_list()
                 .w_h(150.0, 50.0)
                 .down_from(TIMES_LABEL, 40.0)
                 .right_from(TIMES_LABEL, 30.0)
                 .label("White Player")
                 .react(|selected_idx: &mut Option<usize>, new_idx, string: &str| {
                     *selected_idx = Some(new_idx);
                     white_player = PlayerKind::from_ddl_str(string).unwrap();
                 })
                 .set(WHITE_PLAYER_DDL, ui);
        }
        app.game_config.black_player = black_player;
        app.game_config.white_player = white_player;
    }

    Button::new()
        .w_h(200.0, 50.0)
        .down_from(TIMES_LABEL, 130.0)
        .align_middle_x_of(TIMES_LABEL)
        .label("start")
        .react(|| {
            let mut app = app_ref.deref().borrow_mut();
            app.state = State::Play(PlayState::new(Size(gc.rows, gc.cols),
                                                   gc.black_player,
                                                   gc.white_player));
        })
        .set(START_BUTTON, ui);
}

fn set_widgets_play(ui: &mut Ui, app_ref: Rc<RefCell<App>>) {
    let (gc, vc) = {
        let app = app_ref.deref().borrow();
        (app.game_config, app.view_config)
    };

    {
        let mut app = app_ref.deref().borrow_mut();
        let play: &mut PlayState = app.state.as_mut();
        play.listen_player();
    }

    Canvas::new().color(vc.board_color).scrolling(true).set(CANVAS, ui);

    let board_width = vc.cell_size * (gc.cols as f64);
    let indicator_width = vc.cell_size + vc.indicator_text_width;
    let width = board_width + vc.board_margin * 2.0 + indicator_width + vc.board_margin;

    let board_height = vc.cell_size * (gc.rows as f64);
    let indicator_height = vc.cell_size * 2.0;
    let height = vc.board_margin * 2.0 + f64::max(board_height, indicator_height);

    let style = LineStyle::new().thickness(0.0);
    let rect = Rectangle::outline_styled([width, height], style);

    match (ui.win_w < board_width, ui.win_h < board_height) {
        (true, true) => rect.top_left_of(CANVAS),
        (false, true) => rect.mid_top_of(CANVAS),
        (true, false) => rect.mid_left_of(CANVAS),
        (false, false) => rect.middle_of(CANVAS),
    }
    .set(PLAY_CANVAS, ui);

    WidgetMatrix::new(gc.cols as usize, gc.rows as usize)
        .top_left_with_margins_on(PLAY_CANVAS, vc.board_margin, vc.board_margin)
        .w_h(vc.cell_size * (gc.cols as f64),
             vc.cell_size * (gc.rows as f64))
        .each_widget(|_n, col, row| {
            let pt = Point(row as i32, col as i32);

            let app_ref = app_ref.clone();
            {
                let app = app_ref.deref().borrow();
                let play: &PlayState = app.state.as_ref();

                match play.board.turn() {
                    Some(turn) if play.board.can_locate(pt) && !play.has_player(turn) => {
                        OthelloDisk::new().flow_disk(Some(turn))
                    }
                    _ => OthelloDisk::new(),
                }
                .disk(play.board[pt])
                .background_color(vc.board_color)
                .frame(vc.frame_width)
                .frame_color(vc.frame_color)
                .white_color(vc.white_color)
                .black_color(vc.black_color)
                .radius(vc.disk_radius)
            }
            .react(move || {
                let mut app = app_ref.deref().borrow_mut();
                let play: &mut PlayState = app.state.as_mut();
                if let Some(turn) = play.board.turn() {
                    if !play.has_player(turn) {
                        play.locate(pt);
                    }
                }
            })
        })
        .set(BOARD, ui);

    let x = vc.cell_size * ((gc.cols / 4) as f64);
    let y = vc.cell_size * ((gc.rows / 4) as f64);
    let signs = &[(-1.0, 1.0), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)];
    for (i, &(sx, sy)) in signs.iter().enumerate() {
        Circle::fill(vc.dot_radius)
            .x_y_relative_to(BOARD, sx * x, sy * y)
            .color(vc.frame_color)
            .set(DOT + i, ui);
    }

    for (i, &side) in [Side::Black, Side::White].iter().enumerate() {
        let app = app_ref.deref().borrow();
        let play: &PlayState = app.state.as_ref();

        if i == 0 {
            OthelloDisk::new().right_from(BOARD, vc.board_margin)
        } else {
            OthelloDisk::new().down_from(INDICATOR_LABEL_ICON + (i - 1), 0.0)
        }
        .w_h(vc.cell_size, vc.cell_size)
        .background_color(vc.board_color)
        .frame(0.0)
        .white_color(vc.white_color)
        .black_color(vc.black_color)
        .radius(vc.disk_radius)
        .disk(Some(side))
        .react(|| {})
        .set(INDICATOR_LABEL_ICON + i, ui);

        Text::new(&play.board.num_disk(side).to_string())
            .w(vc.indicator_text_width)
            .right_from(INDICATOR_LABEL_ICON + i, 0.0)
            .font_size(60)
            .align_text_right()
            .set(INDICATOR_LABEL_TEXT + i, ui);
    }
}

#![recursion_limit = "256"]

use rand::seq::IteratorRandom;
use rand::thread_rng;
use rand::{rngs::ThreadRng, Rng};
use std::ops::{Add, AddAssign, Sub};
use wasm_bindgen::prelude::*;
use web_sys::console;
use yew::prelude::*;

#[derive(Debug, Copy, Clone)]
struct Vec2 {
    x: i32,
    y: i32,
}

impl Vec2 {
    pub fn new(x: i32, y: i32) -> Vec2 {
        Vec2 { x, y }
    }
}

impl Sub<Vec2> for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub fn as_pair(self: Self) -> (i32, i32) {
        match self {
            Direction::Left => (0, -1),
            Direction::Right => (0, 1),
            Direction::Up => (-1, 0),
            Direction::Down => (1, 0),
        }
    }

    fn build_traversal(self) -> Vec<Position> {
        let i_traversal: Vec<usize> = match self {
            Direction::Down => (0..4).rev().collect(),
            _ => (0..4).collect(),
        };

        let j_traversal: Vec<usize> = match self {
            Direction::Right => (0..4).rev().collect(),
            _ => (0..4).collect(),
        };

        i_traversal
            .iter()
            .flat_map(|i| j_traversal.iter().map(move |j| Position::new(*i, *j)))
            .collect()
    }
}

impl From<Vec2> for Direction {
    fn from(vec: Vec2) -> Self {
        if vec.x.abs() > vec.y.abs() {
            if vec.x > 0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else {
            if vec.y > 0 {
                Direction::Down
            } else {
                Direction::Up
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Position {
    i: usize,
    j: usize,
}

impl Position {
    pub fn new(i: usize, j: usize) -> Position {
        Position { i, j }
    }

    pub fn from_index(index: usize) -> Position {
        Position {
            i: index / 4,
            j: index % 4,
        }
    }

    pub fn index(self) -> usize {
        self.i * 4 + self.j
    }

    pub fn is_out_of_bounds(self) -> bool {
        self.i >= 4 || self.j >= 4
    }
}

impl Add<Direction> for Position {
    type Output = Position;

    fn add(self, direction: Direction) -> Self::Output {
        let (i, j) = direction.as_pair();

        Position {
            i: (self.i as i32 + i) as usize,
            j: (self.j as i32 + j) as usize,
        }
    }
}

impl AddAssign<Direction> for Position {
    fn add_assign(&mut self, direction: Direction) {
        *self = *self + direction
    }
}

#[derive(Debug, Copy, Clone, Eq)]
struct Tile {
    number: i32,
    state: TileState,
    previous_position: Option<Position>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum TileState {
    New,
    Static,
    Merged,
}

impl TileState {
    fn to_string(&self) -> &str {
        match self {
            TileState::New => "new",
            TileState::Static => "static",
            TileState::Merged => "merged",
        }
    }
}

impl Tile {
    fn new(number: i32) -> Tile {
        Tile {
            number,
            state: TileState::New,
            previous_position: None,
        }
    }
}

impl PartialEq for Tile {
    fn eq(&self, other: &Tile) -> bool {
        self.number == other.number
    }
}

type Cell = Option<Tile>;

#[derive(Debug, Copy, Clone)]
struct Grid {
    cells: [Cell; 16],
    rng: ThreadRng,
    enable_new_tiles: bool,
}

impl Default for Grid {
    fn default() -> Self {
        let mut grid = Grid::new([None; 16]);

        for _ in 0..2 {
            grid.add_random_tile();
        }

        grid
    }
}

impl PartialEq for Grid {
    fn eq(&self, other: &Grid) -> bool {
        self.cells == other.cells
    }
}

impl Grid {
    pub fn new(cells: [Cell; 16]) -> Grid {
        Grid {
            cells,
            rng: thread_rng(),
            enable_new_tiles: true,
        }
    }

    pub fn disable_new_tiles(&mut self) {
        self.enable_new_tiles = false;
    }

    fn get(&self, position: Position) -> Option<Tile> {
        self.cells.get(position.index()).and_then(|tile| *tile)
    }

    fn prepare_for_move(&mut self) {
        for i in 0..16 {
            self.cells
                .get_mut(i)
                .and_then(|cell| cell.as_mut())
                .map(|tile| {
                    tile.state = TileState::Static;
                    tile.previous_position = Some(Position::from_index(i));
                });
        }
    }

    pub fn move_in(&mut self, direction: Direction) {
        self.prepare_for_move();

        let traversal = direction.build_traversal();

        let mut moved = false;

        for start_position in traversal {
            moved |= self.traverse_from(start_position, direction);
        }

        if moved {
            self.add_random_tile()
        }
    }

    fn traverse_from(&mut self, start_position: Position, in_direction: Direction) -> bool {
        let mut start_tile = match self.get(start_position) {
            Some(tile) => tile,
            None => return false,
        };

        let mut new_position = start_position;

        loop {
            let next_position = new_position + in_direction;

            if next_position.is_out_of_bounds() {
                break;
            }

            if let Some(tile) = self.get(next_position) {
                if tile == start_tile && tile.state != TileState::Merged {
                    start_tile.number *= 2;
                    start_tile.state = TileState::Merged;
                    new_position = next_position;
                }

                break;
            }

            new_position = next_position;
        }

        if start_position == new_position {
            return false;
        }

        self.cells[start_position.index()] = None;
        self.cells[new_position.index()] = Some(start_tile);

        return true;
    }

    fn add_random_tile(&mut self) {
        if !self.enable_new_tiles {
            return;
        }

        let rng = &mut self.rng;

        let empty_cells = self.cells.iter_mut().filter(|x| x.is_none());

        if let Some(empty) = empty_cells.choose(rng) {
            let number = match self.rng.gen::<f64>() {
                x if x > 0.9 => 4,
                _ => 2,
            };

            *empty = Some(Tile::new(number));
        }
    }

    fn tiles(&self) -> impl Iterator<Item = (Position, Tile)> + '_ {
        self.cells
            .iter()
            .enumerate()
            .filter_map(|(i, cell)| match cell {
                None => None,
                Some(tile) => Some((Position::from_index(i), *tile)),
            })
            .flat_map(|(position, tile)| match tile.state {
                TileState::Merged => vec![
                    (position, tile),
                    (
                        position,
                        Tile {
                            state: TileState::Static,
                            previous_position: tile.previous_position,
                            number: tile.number / 2,
                        },
                    ),
                ],
                _ => vec![(position, tile)],
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::{Direction, Grid, Tile};
    use std::convert::TryInto;

    #[test]
    fn it_works() {
        struct TestCase {
            state: [i32; 16],
            expected: [i32; 16],
            moves: Vec<Direction>,
        }

        let test_cases = [
            TestCase {
                #[rustfmt::skip]
                state: [
                    0, 0, 0, 0,
                    0, 2, 2, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                ],
                #[rustfmt::skip]
                expected: [
                    0, 2, 2, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                ],
                moves: vec![Direction::Up],
            },
            TestCase {
                #[rustfmt::skip]
                state: [
                    0, 0, 0, 0,
                    0, 2, 2, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                ],
                #[rustfmt::skip]
                expected: [
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 2, 2, 0,
                ],
                moves: vec![Direction::Down],
            },
            TestCase {
                #[rustfmt::skip]
                state: [
                    0, 0, 0, 0,
                    0, 2, 0, 0,
                    0, 2, 0, 0,
                    0, 0, 0, 0,
                ],
                #[rustfmt::skip]
                expected: [
                    0, 0, 0, 0,
                    2, 0, 0, 0,
                    2, 0, 0, 0,
                    0, 0, 0, 0,
                ],
                moves: vec![Direction::Left],
            },
            TestCase {
                #[rustfmt::skip]
                state: [
                    0, 0, 0, 0,
                    0, 2, 0, 0,
                    0, 2, 0, 0,
                    0, 0, 0, 0,
                ],
                #[rustfmt::skip]
                expected: [
                    0, 0, 0, 0,
                    0, 0, 0, 2,
                    0, 0, 0, 2,
                    0, 0, 0, 0,
                ],
                moves: vec![Direction::Right],
            },
            TestCase {
                #[rustfmt::skip]
                state: [
                    0, 0, 2, 0,
                    0, 2, 4, 0,
                    4, 0, 0, 0,
                    4, 0, 2, 0,
                ],
                #[rustfmt::skip]
                expected: [
                    0, 0, 0, 2,
                    0, 0, 2, 4,
                    0, 0, 0, 4,
                    0, 0, 4, 2,
                ],
                moves: vec![Direction::Right],
            },
            TestCase {
                #[rustfmt::skip]
                state: [
                    0, 0, 2, 2,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                ],
                #[rustfmt::skip]
                expected: [
                    0, 0, 0, 4,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                ],
                moves: vec![Direction::Right],
            },
            TestCase {
                #[rustfmt::skip]
                state: [
                    2, 2, 2, 2,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                ],
                #[rustfmt::skip]
                expected: [
                    0, 0, 4, 4,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                ],
                moves: vec![Direction::Right],
            },
            TestCase {
                #[rustfmt::skip]
                state: [
                    0, 0, 4, 4,
                    2, 0, 0, 2,
                    0, 2, 0, 2,
                    4, 0, 2, 2,
                ],
                #[rustfmt::skip]
                expected: [
                    0, 0, 0, 8,
                    0, 0, 0, 4,
                    0, 0, 0, 4,
                    0, 0, 0, 8,
                ],
                moves: vec![Direction::Right, Direction::Right],
            },
        ];

        for (i, case) in test_cases.iter().enumerate() {
            let mut state = make_grid(case.state);
            let expected = make_grid(case.expected);

            state.disable_new_tiles();

            for direction in &case.moves {
                state.move_in(*direction);
            }

            assert_eq!(state, expected, "TestCase #{}", i);
        }
    }

    fn make_grid(from_numbers: [i32; 16]) -> Grid {
        Grid::new(
            from_numbers
                .iter()
                .map(|number| {
                    if *number > 0 {
                        Some(Tile::new(*number))
                    } else {
                        None
                    }
                })
                .collect::<Vec<Option<Tile>>>()
                .try_into()
                .unwrap(),
        )
    }

    #[test]
    fn it_adds_random_tile_after_move() {
        #[rustfmt::skip]
        let mut grid = make_grid([
            0, 0, 0, 0,
            0, 2, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);

        grid.move_in(Direction::Right);

        let count = grid.cells.iter().filter(|cell| cell.is_some()).count();

        assert_eq!(2, count);
    }

    #[test]
    fn it_doesnt_add_random_tile_after_invalid_move() {
        #[rustfmt::skip]
        let mut grid = make_grid([
            0, 0, 0, 0,
            0, 0, 0, 2,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);

        grid.move_in(Direction::Right);

        let count = grid.cells.iter().filter(|cell| cell.is_some()).count();

        assert_eq!(1, count);
    }
}

pub struct Model {
    grid: Grid,
    #[allow(dead_code)]
    keyboard_event_listener: Callback<KeyboardEvent>,
    current_render: i32,
    touch_start: Option<TouchEvent>,
}

impl Model {
    fn move_in(&mut self, direction: Direction) {
        self.grid.move_in(direction);
    }
}

pub enum Msg {
    KeyboardEvent(KeyboardEvent),
    TouchStart(TouchEvent),
    TouchEnd(TouchEvent),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        let keyboard_event_listener = Callback::from(move |e: KeyboardEvent| {
            e.prevent_default();
            link.send_message(Msg::KeyboardEvent(e))
        });

        Self {
            grid: Grid::default(),
            touch_start: None,
            current_render: 0,
            keyboard_event_listener,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::KeyboardEvent(e) => match e.key_code() {
                37 => self.move_in(Direction::Left),
                38 => self.move_in(Direction::Up),
                39 => self.move_in(Direction::Right),
                40 => self.move_in(Direction::Down),
                _ => return false,
            },
            Msg::TouchStart(e) => {
                e.prevent_default();

                self.touch_start = Some(e);

                return false;
            }
            Msg::TouchEnd(touches_end) => {
                console::log_1(&"touchend".into());
                let touch_start = self
                    .touch_start
                    .as_ref()
                    .and_then(|e| e.changed_touches().item(0))
                    .map(|touch| Vec2::new(touch.client_x(), touch.client_y()));

                let touch_end = touches_end
                    .changed_touches()
                    .item(0)
                    .map(|touch| Vec2::new(touch.client_x(), touch.client_y()));

                match (touch_start, touch_end) {
                    (Some(start), Some(end)) => self.move_in((end - start).into()),
                    _ => return false,
                };
            }
        };

        self.current_render += 1;

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ontouchstart = ctx.link().callback(Msg::TouchStart);
        let ontouchend = ctx.link().callback(Msg::TouchEnd);

        let onkeydown = self.keyboard_event_listener.clone();

        html! {
            <div class="grid-wrapper" ontouchstart={ontouchstart} ontouchend={ontouchend} onkeydown={onkeydown} tabindex="0">
                <div class="grid" key={self.current_render}>
                { for (0..16).map(|_| { html! { <div class="cell"></div> }}) }
                { for self.grid.tiles().map(|(position, tile)| html! { <TileComponent {position} {tile} />} ) }
                </div>
            </div>
        }
    }
}

struct TileComponent {
    tile: Tile,
    position: Position,
}

impl TileComponent {
    fn class_name(&self) -> String {
        format!(
            "tile tile-{} tile-{}-{} tile-{}",
            if self.tile.number <= 2048 {
                self.tile.number.to_string()
            } else {
                "super".to_string()
            },
            self.position.index() % 4,
            self.position.index() / 4,
            self.tile.state.to_string(),
        )
    }
}

#[derive(Properties, Clone, PartialEq)]
struct TileComponentProps {
    tile: Tile,
    position: Position,
}

enum TileMsg {
    ActualPosition(Position),
}

impl Component for TileComponent {
    type Message = TileMsg;
    type Properties = TileComponentProps;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let mut position = props.position;

        match (props.tile.state, props.tile.previous_position) {
            (TileState::Merged, _) => {}
            (_, Some(previous_position)) => {
                position = previous_position;

                let actual_position = props.position;

                ctx.link()
                    .send_future(async move { TileMsg::ActualPosition(actual_position) });
            }
            _ => {}
        }

        Self {
            tile: props.tile,
            position,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.tile = ctx.props().tile;
        self.position = ctx.props().position;

        match msg {
            TileMsg::ActualPosition(position) => {
                self.position = position;
            }
        }

        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class={self.class_name()}>
                <div class="tile-inner">
                    { self.tile.number.to_string() }
                </div>
            </div>
        }
    }
}

#[wasm_bindgen]
pub fn run_app() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let root = document.get_element_by_id("root").unwrap();

    yew::Renderer::<Model>::with_root(root).render();
}

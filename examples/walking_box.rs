use kondi::{ContextConfiguration, Context, Game, GameStateSetup, GgezResult, util::Point2, ggez::event::KeyCode};
use kondi::object::{
    tex_box::{TexBox, TexBoxData},
};

fn main() {
    ContextConfiguration::new()
        .run::<WalkingBoxGame>()
        .unwrap()
}

struct WalkingBoxGame;

const UP: &'static str = "up";
const DOWN: &'static str = "down";
const LEFT: &'static str = "left";
const RIGHT: &'static str = "right";

const SPEED: f32 = 100.;

impl Game for WalkingBoxGame {
    fn setup(_: &mut Context, s: &mut GameStateSetup<Self>) -> GgezResult<Self> {
        let (w, h) = s.dims();

        s.bind_keys(UP, vec![KeyCode::Up, KeyCode::W]);
        s.bind_keys(DOWN, vec![KeyCode::Down, KeyCode::S]);
        s.bind_keys(LEFT, vec![KeyCode::Left, KeyCode::A]);
        s.bind_keys(RIGHT, vec![KeyCode::Right, KeyCode::D]);

        let _walking_box = s.object_set.add(TexBox::new(
            TexBoxData {
                texture: "box",
                pos: Point2::new(w / 2., h / 2.),
                rot: 0.,
            }, |data, ctx, state, delta| {
                if state.is_down(ctx, UP) {
                    data.pos.y -= SPEED * delta;
                }
                if state.is_down(ctx, DOWN) {
                    data.pos.y += SPEED * delta;
                }
                if state.is_down(ctx, LEFT) {
                    data.pos.x -= SPEED * delta;
                }
                if state.is_down(ctx, RIGHT) {
                    data.pos.x += SPEED * delta;
                }
            }
        ));

        Ok(WalkingBoxGame)
    }
}
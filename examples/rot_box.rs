use kondi::{ContextConfiguration, Context, Game, GameStateSetup, GgezResult, util::Point2, ggez::event::KeyCode};
use kondi::object::{
    tex_box::{TexBox, TexBoxData},
    ObjectId,
};

fn main() {
    ContextConfiguration::new()
        .run::<RotBoxGame>()
        .unwrap()
}

struct RotBoxGame {
    rot_box: ObjectId<TexBox<'static>>,
}

const CHANGE_KEY: &'static str = "change";

impl Game for RotBoxGame {
    fn setup(_: &mut Context, s: &mut GameStateSetup<Self>) -> GgezResult<Self> {
        let (w, h) = s.dims();
        
        // Bind space as the change key
        s.bind_key(KeyCode::Space, CHANGE_KEY);
        // You can bind several keys
        s.bind_key(KeyCode::Return, CHANGE_KEY);

        let rot_box = s.object_set.add(TexBox::new(
            TexBoxData {
                texture: "box",
                pos: Point2::new(w / 2., h / 2.),
                rot: 0.,
            }, |data, delta| {
                data.rot += 0.4 * delta;
            }
        ));

        // set a handler for the change key
        s.add_key_press_handler(CHANGE_KEY, Box::new(|_ctx, game, state| {
            let rot_box = state.object_set.get_mut(game.rot_box).unwrap();

            rot_box.data.rot -= 2.;

            Ok(())
        }));

        Ok(RotBoxGame {
            rot_box,
        })
    }
}
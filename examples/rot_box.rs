use kondi::{ContextConfiguration, Context, Game, State, GgezResult, util::Point2};
use kondi::object::tex_box::{TexBox, TexBoxData};

fn main() {
    ContextConfiguration::new()
        .run::<Empty>()
        .unwrap()
}

struct Empty;

impl Game for Empty {
    fn setup(_: &mut Context, s: &mut State) -> GgezResult<Self> {
        s.object_set.add(TexBox::new(
            TexBoxData {
                texture: "box".into(),
                pos: Point2::new(400., 400.),
                rot: 0.,
            }, |data, delta| {
                data.rot += 0.4 * delta;
            }
        ));
        Ok(Empty)
    }
    fn tick(&mut self, _: &mut State) -> GgezResult {
        Ok(())
    }
}
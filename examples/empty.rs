use kondi::{ContextConfiguration, Context, Game, GameStateSetup, GgezResult};

fn main() {
    ContextConfiguration::new()
        .run::<Empty>()
        .unwrap()
}

struct Empty;

impl Game for Empty {
    fn setup(_: &mut Context, _: &mut GameStateSetup<Self>) -> GgezResult<Self> {
        Ok(Empty)
    }
}

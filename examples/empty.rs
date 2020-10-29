use kondi::{ContextConfiguration, Context, Game, State, GgezResult};

fn main() {
    ContextConfiguration::new()
        .run::<Empty>()
        .unwrap()
}

struct Empty;

impl Game for Empty {
    fn setup(_: &mut Context, _: &mut State) -> GgezResult<Self> {
        Ok(Empty)
    }
    fn tick(&mut self, _: &mut State) -> GgezResult {
        Ok(())
    }
}
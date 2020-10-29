// #![windows_subsystem = "windows"]
//! Kondi engine
#![warn(clippy::all)]

#[macro_use]
extern crate log;

pub use ggez::{self, Context, GameError as GgezError, GameResult as GgezResult};
pub use ggez::conf::{WindowSetup, WindowMode};

use nalgebra::Matrix4;

use ggez::{
    ContextBuilder,
    event::{run, EventHandler},
    graphics::{self, Rect, Color, BLACK},
    timer,
};

use self::util::{Vector2, Point2};

pub mod util {
    use ggez::graphics::Color;
    use nalgebra::base::coordinates::XY;
    pub type Vector2 = nalgebra::Vector2<f32>;
    pub type Point2 = nalgebra::Point2<f32>;
    pub type Rotation2 = nalgebra::Rotation2<f32>;

    pub const TRANS: Color = Color{r:1.,g:1.,b:1.,a:0.5};
    pub const GREEN: Color = Color{r:0.1,g:0.7,b:0.1,a:1.};
    pub const RED: Color = Color{r:1.,g:0.,b:0.,a:1.};
    pub const BLUE: Color = Color{r:0.,g:0.,b:1.,a:1.};

    /// Makes a unit vector from a given direction angle
    #[inline]
    pub fn angle_to_vec(angle: f32) -> Vector2 {
        let (sin, cos) = angle.sin_cos();
        Vector2::new(cos, sin)
    }
    /// Gets the direction angle on the screen (0 is along the x-axis) of a vector
    #[inline]
    pub fn angle_from_vec(v: Vector2) -> f32 {
        let XY{x, y} = *v;
        y.atan2(x)
    }
}

pub mod textures;
pub mod object;

use textures::Textures;

#[derive(Debug, Clone)]
pub struct ContextConfiguration {
    game_id: &'static str,
    author: &'static str,
    window_setup: WindowSetup,
    window_mode: WindowMode,
    // TODO ggez modules (audio, gamepad)
}

impl Default for ContextConfiguration {
    fn default() -> Self {
        ContextConfiguration {
            game_id: "kondi",
            author: "Falch",
            window_setup: WindowSetup::default().title("kondi"),
            window_mode: WindowMode::default().dimensions(800., 600.),
        }
    }
}

impl ContextConfiguration {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn id(self, game_id: &'static str) -> Self {
        ContextConfiguration {
            game_id,
            .. self
        }
    }
    pub fn author(self, game_id: &'static str) -> Self {
        ContextConfiguration {
            game_id,
            .. self
        }
    }
    pub fn title(self, title: &str) -> Self {
        ContextConfiguration {
            window_setup: self.window_setup.title(title),
            .. self
        }
    }
    pub fn size(self, width: f32, height: f32) -> Self {
        ContextConfiguration {
            window_mode: self.window_mode.dimensions(width, height),
            .. self
        }
    }

    pub fn run<G: Game>(self) -> Result<(), Error> {
        // TODO maybe, add args

        let ContextConfiguration {
            game_id,
            author,
            window_mode,
            window_setup,
        } = self;

        // Create a context (the part that runs the game loop)
        let (mut ctx, mut events) = ContextBuilder::new(game_id, author)
            .window_setup(window_setup)
            .window_mode(window_mode)
            .build()?;

        #[cfg(debug_assertions)]
        {
            // Add the workspace directory to the filesystem when running with cargo
            use ggez::filesystem;
            if let Ok(manifest_dir) = ::std::env::var("CARGO_MANIFEST_DIR") {
                let mut path = ::std::path::PathBuf::from(manifest_dir);
                path.push("resources");
                filesystem::mount(&mut ctx, &path, true);
            }
        }

        let mut state = State::new(&mut ctx)?;

        let mut handler = GameState::<G> {
            game: Game::setup(&mut ctx, &mut state)?,
            state,
        };

        run(&mut ctx, &mut events, &mut handler)?;

        Ok(())
    }
}

use object::ObjectSet;

pub struct State {
    pub textures: Textures,
    pub object_set: ObjectSet,
    pub offset: Vector2,
    width: f32,
    height: f32,
    pub background: Color,
}

impl State {
    fn new(ctx: &mut Context) -> GgezResult<Self> {
        let Rect {w: width, h: height, ..} = graphics::screen_coordinates(ctx);
        Ok(State {
            textures: Textures::new(ctx)?,
            object_set: ObjectSet::new(),
            offset: Vector2::new(0., 0.),
            width,
            height,
            background: BLACK,
        })
    }
    /// Sets the offset so that the given point will be centered on the screen
    #[inline]
    pub fn focus_on(&mut self, p: Point2) {
        self.offset = -p.coords + 0.5 * Vector2::new(self.width, self.height);
    }
    #[inline(always)]
    pub fn dims(&self) -> (f32, f32) {
        (self.width, self.height)
    }
}

struct GameState<G: Game> {
    state: State,
    game: G,
}

const DESIRED_FPS: u32 = 60;

pub(crate) const DELTA: f32 = 1. / DESIRED_FPS as f32;

impl<G: Game> EventHandler for GameState<G> {
    fn update(&mut self, ctx: &mut Context) -> GgezResult {
        self.game.logic(&mut self.state)?;

        while timer::check_update_time(ctx, DESIRED_FPS) {
            for obj in self.state.object_set.iter_mut() {
                obj.update(DELTA);
            }
            self.game.tick(&mut self.state)?;
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GgezResult {
        graphics::clear(ctx, self.state.background);

        graphics::push_transform(ctx, Some(Matrix4::new_translation(&self.state.offset.fixed_resize(0.))));
        graphics::apply_transformations(ctx)?;
        
        for obj in self.state.object_set.iter() {
            obj.draw(ctx, &self.state.textures)?;
        }
        self.game.draw(ctx, &self.state)?;

        // Pop the offset tranformation to draw the UI on the screen
        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx)?;

        // Flip the buffers to see what we just drew
        graphics::present(ctx)?;

        // Give the computer some time to do other things
        timer::yield_now();
        Ok(())
    }
    fn quit_event(&mut self, _ctx: &mut Context) -> bool { false }
    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) {
        self.state.width = width;
        self.state.height = height;
    }
}

#[derive(Debug)]
pub enum Error {
    GgezError(GgezError),
}

impl From<GgezError> for Error {
    #[inline]
    fn from(e: GgezError) -> Self {
        Error::GgezError(e)
    }
}

pub trait Game: Sized {
    /// Run to create the game
    fn setup(ctx: &mut Context, state: &mut State) -> GgezResult<Self>;
    /// This is run every once in a while
    fn logic(&mut self, _state: &mut State) -> GgezResult { Ok(()) }
    /// This is run every tick
    fn tick(&mut self, _state: &mut State) -> GgezResult { Ok(()) }
    /// This function should draw other things on the screen
    /// that follow the offset
    fn draw(&self, _ctx: &mut Context, _state: &State) -> GgezResult { Ok(()) }
    /// This should draw things on top of the what's drawn in `draw`
    /// and that do not follow the offset
    fn draw_hud(&self, _ctx: &mut Context, _state: &State) -> GgezResult { Ok(()) }
}

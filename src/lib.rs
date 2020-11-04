// #![windows_subsystem = "windows"]
//! Kondi engine
#![warn(clippy::all)]

#[macro_use]
extern crate log;

use std::ops::{Deref, DerefMut};
use std::collections::{HashMap, HashSet};

pub use ggez::{self, Context, GameError as GgezError, GameResult as GgezResult};
pub use ggez::conf::{WindowSetup, WindowMode};

use nalgebra::Matrix4;

use ggez::{
    ContextBuilder,
    event::{run, EventHandler, KeyCode, KeyMods},
    graphics::{self, Rect, Color, BLACK},
    input::keyboard,
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

        
        let mut setup = GameStateSetup::<G> {
            state: State::new(&mut ctx)?,
            object_set: ObjectSet::new(),
            handlers: Handlers::new(),
        };
        let game = Game::setup(&mut ctx, &mut setup)?;
        let GameStateSetup {state, object_set, handlers} = setup;

        let mut handler = GameState::<G> {
            game,
            handlers,
            object_set,
            state,
        };

        run(&mut ctx, &mut events, &mut handler)?;

        Ok(())
    }
}

use object::ObjectSet;

pub type KeyHandler<G> = Box<dyn FnMut(&mut Context, &mut G, &mut State, &mut ObjectSet) -> GgezResult>;

#[derive(Debug)]
pub struct State<'a> {
    pub textures: Textures,
    pub offset: Vector2,
    width: f32,
    height: f32,
    pub background: Color,

    error: Option<GgezError>,
    key_to_name: HashMap<KeyCode, &'a str>,
    name_to_keys: HashMap<&'a str, HashSet<KeyCode>>
}

impl<'a> State<'a> {
    fn new(ctx: &mut Context) -> GgezResult<Self> {
        let Rect {w: width, h: height, ..} = graphics::screen_coordinates(ctx);
        Ok(State {
            textures: Textures::new(ctx)?,
            offset: Vector2::new(0., 0.),
            width,
            height,
            background: BLACK,
            error: None,
            key_to_name: HashMap::new(),
            name_to_keys: HashMap::new(),
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
    #[inline]
    pub fn bind_key(&mut self, key: KeyCode, name: &'a str) {
        self.key_to_name.insert(key, name);
        self.name_to_keys.entry(name).or_insert_with(HashSet::new).insert(key);
    }
    #[inline]
    pub fn bind_keys(&mut self, name: &'a str, keys: Vec<KeyCode>) {
        for &key in &keys {
            self.key_to_name.insert(key, name);
        }
        self.name_to_keys.entry(name).or_insert_with(HashSet::new).extend(keys);
    }
    #[inline]
    pub fn is_down(&self, ctx: &Context, name: &str) -> bool {
        if let Some(keys_for_name) = self.name_to_keys.get(name) {
            keyboard::pressed_keys(&ctx)
                .intersection(keys_for_name)
                .all(|_| false)
        } else {
            false
        }
    }
}

struct Handlers<'a, G: Game> {
    key_up_handlers: HashMap<&'a str, KeyHandler<G>>,
    key_down_handlers: HashMap<&'a str, KeyHandler<G>>,
    key_press_handlers: HashMap<&'a str, KeyHandler<G>>,
}

impl<'a, G: Game> Handlers<'a, G> {
    #[inline]
    fn new() -> Self {
        Handlers {
            key_up_handlers: HashMap::new(),
            key_down_handlers: HashMap::new(),
            key_press_handlers: HashMap::new(),
        }
    }
}

pub struct GameStateSetup<'a, G: Game> {
    pub state: State<'a>,
    pub object_set: ObjectSet,
    handlers: Handlers<'a, G>,
}

impl<'a, G: Game> GameStateSetup<'a, G> {
    #[inline]
    pub fn add_key_up_handler(&mut self, name: &'a str, handler: KeyHandler<G>) {
        self.handlers.key_up_handlers.insert(name, handler);
    }
    #[inline]
    pub fn add_key_down_handler(&mut self, name: &'a str, handler: KeyHandler<G>) {
        self.handlers.key_down_handlers.insert(name, handler);
    }
    #[inline]
    pub fn add_key_press_handler(&mut self, name: &'a str, handler: KeyHandler<G>) {
        self.handlers.key_press_handlers.insert(name, handler);
    }
}

impl<'a, G: Game> Deref for GameStateSetup<'a, G> {
    type Target = State<'a>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
impl<'a, G: Game> DerefMut for GameStateSetup<'a, G> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

struct GameState<'a, G: Game> {
    state: State<'a>,
    pub object_set: ObjectSet,
    handlers: Handlers<'a, G>,
    game: G,
}

const DESIRED_FPS: u32 = 60;

pub(crate) const DELTA: f32 = 1. / DESIRED_FPS as f32;

impl<G: Game> EventHandler for GameState<'_, G> {
    fn update(&mut self, ctx: &mut Context) -> GgezResult {
        if let Some(error) = self.state.error.take() {
            return Err(error);
        }
        self.game.logic(&mut self.state)?;

        while timer::check_update_time(ctx, DESIRED_FPS) {
            for obj in self.object_set.iter_mut() {
                obj.update(ctx, &mut self.state, DELTA);
            }
            self.game.tick(&mut self.state)?;
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GgezResult {
        graphics::clear(ctx, self.state.background);

        graphics::push_transform(ctx, Some(Matrix4::new_translation(&self.state.offset.fixed_resize(0.))));
        graphics::apply_transformations(ctx)?;

        for obj in self.object_set.iter() {
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
    fn key_up_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        if let Some(&name) = self.state.key_to_name.get(&keycode) {
            if let Some(handler) = self.handlers.key_up_handlers.get_mut(name) {
                if let Err(e) = handler(ctx, &mut self.game, &mut self.state, &mut self.object_set) {
                    self.state.error = Some(e);
                }
            }
        }
    }
    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, repeat: bool) {
        if let Some(&name) = self.state.key_to_name.get(&keycode) {
            if !repeat {
                if let Some(handler) = self.handlers.key_down_handlers.get_mut(name) {
                    if let Err(e) = handler(ctx, &mut self.game, &mut self.state, &mut self.object_set) {
                        self.state.error = Some(e);
                    }
                }
            }
            if let Some(handler) = self.handlers.key_press_handlers.get_mut(name) {
                if let Err(e) = handler(ctx, &mut self.game, &mut self.state, &mut self.object_set) {
                    self.state.error = Some(e);
                }
            }
        }
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
    fn setup(ctx: &mut Context, state: &mut GameStateSetup<Self>) -> GgezResult<Self>;
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

use std::fmt::Debug;
use ggez::{graphics::{self, DrawParam}, Context, GameResult};

use crate::{util::Point2, Textures};

use super::Object;

#[derive(Debug, Clone)]
pub struct TexBoxData<'a> {
    pub texture: &'a str,
    pub pos: Point2,
    pub rot: f32,
}

pub struct TexBox<'a> {
    pub data: TexBoxData<'a>,
    update_fn: Box<dyn FnMut(&mut TexBoxData, f32)>
}

impl<'a> TexBox<'a> {
    pub fn new<F: 'static + FnMut(&mut TexBoxData, f32)>(data: TexBoxData<'a>, update: F) -> Self {
        TexBox {
            data,
            update_fn: Box::new(update),
        }
    }
}

impl Object for TexBox<'_> {
    fn update(&mut self, delta: f32) {
        (self.update_fn)(&mut self.data, delta)
    }
    #[inline]
    fn draw(&self, ctx: &mut Context, t: &Textures) -> GameResult<()> {
        let img = t.get_img(ctx, &self.data.texture);

        let drawparams = DrawParam {
            dest: self.data.pos.into(),
            rotation: self.data.rot,
            offset: Point2::new(0.5, 0.5).into(),
            .. Default::default()
        };

        graphics::draw(ctx, &*img, drawparams)
    }
}

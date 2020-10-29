use ggez::{graphics::{self, DrawParam}, Context, GameResult};

use crate::{util::Point2, Textures};

use super::Object;

#[derive(Debug, Clone)]
pub struct TexBoxData {
    pub texture: Box<str>,
    pub pos: Point2,
    pub rot: f32,
}

pub struct TexBox {
    pub data: TexBoxData,
    update_fn: Box<dyn FnMut(&mut TexBoxData, f32)>
}

impl TexBox {
    pub fn new<F: 'static + FnMut(&mut TexBoxData, f32)>(data: TexBoxData, update: F) -> Self {
        TexBox {
            data,
            update_fn: Box::new(update),
        }
    }
}

impl Object for TexBox {
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

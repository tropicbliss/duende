use super::errors::GlError;
use bumpalo::Bump;

pub trait Drawable {
    fn draw(&self, ctx: &mut RendererContext) -> Result<(), GlError>;
}

pub struct RendererContext<'a> {
    pub(crate) bump: &'a Bump,
    pub(crate) command_queue: Vec<Box<dyn FnOnce(), &'a Bump>>,
}

impl<'a> RendererContext<'a> {
    pub(crate) fn new(bump: &'a Bump) -> Self {
        Self {
            bump,
            command_queue: Vec::new(),
        }
    }

    pub fn add_commands<F>(&mut self, queue: F)
    where
        F: FnOnce() + 'static,
    {
        let object = Box::new_in(queue, self.bump);
        self.command_queue.push(object);
    }
}

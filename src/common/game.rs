use crate::three_d::three_d_application_context::ThreeDApplicationContext;

pub trait Game {
    fn game_loop(&mut self, context: &mut ThreeDApplicationContext);
    fn setup(&mut self, _context: &mut ThreeDApplicationContext) {}
    fn teardown(&mut self, _context: &mut ThreeDApplicationContext) {}
}

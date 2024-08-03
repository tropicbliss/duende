use crate::three_d::three_d_application_context::ThreeDApplicationContext;

pub trait Game {
    fn game_loop(&self, context: &mut ThreeDApplicationContext);
    fn setup(&self, _context: &mut ThreeDApplicationContext) {}
    fn teardown(&self, _context: &mut ThreeDApplicationContext) {}
}

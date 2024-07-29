use duende::{
    three_d::{
        application_context::ThreeDApplicationContext,
        game_objects::test_game_object::TestGameObject,
    },
    ApplicationBuilder, Game,
};
use winit::keyboard::NamedKey;

fn main() {
    let app = ApplicationBuilder::new().build();
    app.render(TestGame::new()).unwrap();
}

struct TestGame {
    object: TestGameObject,
}

impl TestGame {
    pub fn new() -> Self {
        Self {
            object: TestGameObject::new(),
        }
    }
}

impl Game for TestGame {
    fn game_loop(&self, context: &mut ThreeDApplicationContext) {
        if context.is_key_pressed(NamedKey::Escape) {
            context.exit();
        }
        context.draw_game_object(&self.object);
    }

    fn teardown(&self, _context: &mut ThreeDApplicationContext) {
        println!("Bye bye!");
    }
}

use duende::{
    common::{application_builder::ApplicationBuilder, game::Game},
    three_d::{
        game_objects::test_game_object::TestGameObject,
        three_d_application_context::ThreeDApplicationContext,
    },
    Matrix3xX, Vector3,
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
            object: TestGameObject::new(Matrix3xX::from_columns(&[
                Vector3::new(0.0, -0.9, 0.0),
                Vector3::new(-0.6, 0.8, 0.0),
                Vector3::new(0.9, -0.2, 0.0),
                Vector3::new(-0.9, -0.2, 0.0),
                Vector3::new(0.6, 0.8, 0.0),
            ])),
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

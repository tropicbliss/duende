use duende::{
    common::{application_builder::ApplicationBuilder, game::Game},
    three_d::{
        game_objects::test_game_object::TestGameObject,
        three_d_application_context::ThreeDApplicationContext,
    },
    Matrix3xX,
};
use rand::{rngs::ThreadRng, Rng};
use winit::keyboard::NamedKey;

fn main() {
    let app = ApplicationBuilder::new().title("Test").build();
    app.render(TestGame::new()).unwrap();
}

struct TestGame {
    object: TestGameObject,
    rng: ThreadRng,
}

impl TestGame {
    pub fn new() -> Self {
        Self {
            object: TestGameObject::new(
                Matrix3xX::from_column_slice(&[
                    0.0, -0.9, 0.0, -0.6, 0.8, 0.0, 0.9, -0.2, 0.0, -0.9, -0.2, 0.0, 0.6, 0.8, 0.0,
                ]),
                Matrix3xX::from_column_slice(&[
                    1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0,
                ]),
            ),
            rng: rand::thread_rng(),
        }
    }
}

impl Game for TestGame {
    fn game_loop(&mut self, context: &mut ThreeDApplicationContext) {
        if context.is_key_pressed(NamedKey::Escape) {
            context.exit();
        }
        context.draw_game_object(&self.object);
    }

    fn teardown(&mut self, _context: &mut ThreeDApplicationContext) {
        println!("Bye bye!");
    }
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

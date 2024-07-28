use crate::{errors::GlError, gl, internal_game_loop::RendererContext};

use super::{Drawable, Fragment, ProgramWrapper, Shader, Vertex};

static FRAGMENT: Shader<Fragment> =
    Shader::create_fragment_shader(include_str!("shaders/fragment_shader.glsl"));

static VERTEX: Shader<Vertex> =
    Shader::create_vertex_shader(include_str!("shaders/vertex_shader.glsl"));

pub struct TestGameObject {
    program: ProgramWrapper,
}

impl TestGameObject {
    pub fn new() -> Self {
        Self {
            program: ProgramWrapper::new(&VERTEX, &FRAGMENT),
        }
    }
}

impl Drawable for TestGameObject {
    fn draw<'a>(&self, ctx: &mut RendererContext<'a>) -> Result<(), GlError> {
        unsafe {
            let program_id = self.program.get_program_id()?;
            ctx.add_commands(move || {
                gl::UseProgram(program_id);
                gl::DrawArrays(gl::POINTS, 0, 1);
                gl::PointSize(10.0);
            });
            Ok(())
        }
    }
}

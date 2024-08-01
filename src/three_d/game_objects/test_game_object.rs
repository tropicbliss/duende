use crate::common::{
    drawables::{create_program, Drawable, Fragment, RendererContext, Shader, Vertex},
    errors::GlError,
    gl,
};
use std::cell::OnceCell;

static FRAGMENT: Shader<Fragment> =
    Shader::create_fragment_shader(include_str!("shaders/fragment_shader.glsl"));

static VERTEX: Shader<Vertex> =
    Shader::create_vertex_shader(include_str!("shaders/vertex_shader.glsl"));

pub struct TestGameObject {
    program_id: OnceCell<Result<u32, GlError>>,
}

impl TestGameObject {
    pub fn new() -> Self {
        Self {
            program_id: OnceCell::new(),
        }
    }
}

impl Default for TestGameObject {
    fn default() -> Self {
        Self::new()
    }
}

impl Drawable for TestGameObject {
    fn draw(&self, ctx: &mut RendererContext<'_>) -> Result<(), GlError> {
        unsafe {
            let program_id = self.program_id.get_or_init(|| {
                let vertex_shader = VERTEX.get_shader_handle()?;
                let fragment_shader = FRAGMENT.get_shader_handle()?;
                let program_id = create_program(&vertex_shader, &fragment_shader)?;
                Ok(program_id)
            });
            let program_id = match program_id {
                Ok(program_id) => *program_id,
                Err(e) => return Err(e.clone()),
            };
            ctx.add_commands(move || {
                gl::UseProgram(program_id);
                gl::PointSize(10.0);
                gl::DrawArrays(gl::POINTS, 0, 1);
            });
            Ok(())
        }
    }
}

impl Drop for TestGameObject {
    fn drop(&mut self) {
        unsafe {
            if let Some(Ok(program_id)) = self.program_id.get() {
                gl::DeleteProgram(*program_id);
            }
        }
    }
}

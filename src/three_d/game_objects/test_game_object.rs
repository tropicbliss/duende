use crate::common::{
    drawables::{Drawable, Fragment, RendererContext, Shader, Vertex},
    errors::GlError,
    gl,
    wrappers::{ProgramWrapper, VaoWrapper},
};

static FRAGMENT: Shader<Fragment> =
    Shader::create_fragment_shader(include_str!("shaders/fragment_shader.glsl"));

static VERTEX: Shader<Vertex> =
    Shader::create_vertex_shader(include_str!("shaders/vertex_shader.glsl"));

pub struct TestGameObject {
    program: ProgramWrapper,
    vao: VaoWrapper,
}

impl TestGameObject {
    pub fn new() -> Self {
        Self {
            program: ProgramWrapper::new(&VERTEX, &FRAGMENT),
            vao: VaoWrapper::new(),
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
            let program_id = self.program.get_program_id()?;
            let vao_ref = self.vao.get_vao_ref().get_vao_ref();
            ctx.add_commands(move || {
                gl::BindVertexArray(vao_ref);
                gl::UseProgram(program_id);
                gl::DrawArrays(gl::POINTS, 0, 1);
                gl::PointSize(10.0);
            });
            Ok(())
        }
    }
}

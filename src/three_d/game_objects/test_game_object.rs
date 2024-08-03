use crate::common::{
    drawables::{Drawable, RendererContext},
    errors::GlError,
    gl,
    helpers::{create_variable, Fragment, Shader, Vertex},
    wrappers::program_wrapper::ProgramWrapper,
};

static FRAGMENT: Shader<Fragment> =
    Shader::create_fragment_shader(include_str!("shaders/fragment_shader.glsl"));

static VERTEX: Shader<Vertex> =
    Shader::create_vertex_shader(include_str!("shaders/vertex_shader.glsl"));

pub struct TestGameObject<const N: usize> {
    program_wrapper: ProgramWrapper,
    created_variable: bool,
    vertices: [f32; N],
}

impl<const N: usize> TestGameObject<N> {
    pub fn new(vertices: [f32; N]) -> Self {
        Self {
            program_wrapper: ProgramWrapper::new(&VERTEX, &FRAGMENT),
            created_variable: true,
            vertices,
        }
    }
}

impl<const N: usize> Drawable for TestGameObject<N> {
    fn draw(&self, ctx: &mut RendererContext<'_>) -> Result<(), GlError> {
        unsafe {
            let program_id = self.program_wrapper.get_program_id()?;
            let vao_ref = self.program_wrapper.get_vao_ref();
            let vbo_ref = self.program_wrapper.get_vbo_ref();
            let vertices_len = self.vertices.len();
            let vertices_ptr = self.vertices.as_ptr();
            let created_variable = self.created_variable;
            ctx.add_commands(move || {
                gl::UseProgram(program_id);
                gl::BindVertexArray(vao_ref);
                gl::BindBuffer(gl::ARRAY_BUFFER, vbo_ref);
                if created_variable {
                    create_variable(program_id, "position").unwrap();
                }
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (vertices_len * std::mem::size_of::<f32>()) as isize,
                    vertices_ptr as *const _,
                    gl::STATIC_DRAW,
                );
                gl::PointSize(10.0);
                gl::DrawArrays(gl::POINTS, 0, vertices_len as i32);
            });
            Ok(())
        }
    }
}

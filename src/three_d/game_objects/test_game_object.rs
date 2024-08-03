use crate::common::{
    drawables::{Drawable, RendererContext},
    errors::GlError,
    gl,
    helpers::{create_program, create_variable, Fragment, Shader, Vertex},
};
use std::cell::OnceCell;

static FRAGMENT: Shader<Fragment> =
    Shader::create_fragment_shader(include_str!("shaders/fragment_shader.glsl"));

static VERTEX: Shader<Vertex> =
    Shader::create_vertex_shader(include_str!("shaders/vertex_shader.glsl"));

pub struct TestGameObject<const N: usize> {
    init_once: OnceCell<Result<(u32, u32, u32), GlError>>,
    created_variable: bool,
    vertices: [f32; N],
}

impl<const N: usize> TestGameObject<N> {
    pub fn new(vertices: [f32; N]) -> Self {
        Self {
            init_once: OnceCell::new(),
            created_variable: true,
            vertices,
        }
    }
}

impl<const N: usize> Drawable for TestGameObject<N> {
    fn draw(&self, ctx: &mut RendererContext<'_>) -> Result<(), GlError> {
        unsafe {
            let (program_id, vao_ref, vbo_ref) = self
                .init_once
                .get_or_init(|| {
                    let vertex_shader = VERTEX.get_shader_handle()?;
                    let fragment_shader = FRAGMENT.get_shader_handle()?;
                    let program_id = create_program(&vertex_shader, &fragment_shader)?;
                    let mut vao_ref = 0;
                    gl::GenVertexArrays(1, &mut vao_ref);
                    let mut vbo_ref = 0;
                    gl::GenBuffers(1, &mut vbo_ref);
                    Ok((program_id, vao_ref, vbo_ref))
                })
                .clone()?;
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

impl<const N: usize> Drop for TestGameObject<N> {
    fn drop(&mut self) {
        unsafe {
            if let Some(Ok((program_id, _, _))) = self.init_once.get() {
                gl::DeleteProgram(*program_id);
            }
        }
    }
}

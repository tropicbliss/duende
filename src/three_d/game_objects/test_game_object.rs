use crate::common::{
    drawables::{Drawable, RendererContext},
    errors::GlError,
    gl,
    helpers::{create_variable, Fragment, Shader, Vertex},
    wrappers::program_wrapper::ProgramWrapper,
};
use nalgebra::Matrix3xX;
use std::cell::Cell;

static FRAGMENT: Shader<Fragment> =
    Shader::create_fragment_shader(include_str!("shaders/fragment_shader.glsl"));

static VERTEX: Shader<Vertex> =
    Shader::create_vertex_shader(include_str!("shaders/vertex_shader.glsl"));

pub struct TestGameObject {
    program_wrapper: ProgramWrapper,
    initialized: Cell<bool>,
    vertices: Matrix3xX<f32>,
}

impl TestGameObject {
    pub fn new(vertices: Matrix3xX<f32>) -> Self {
        Self {
            program_wrapper: ProgramWrapper::new(&VERTEX, &FRAGMENT),
            initialized: Cell::new(false),
            vertices,
        }
    }
}

impl Drawable for TestGameObject {
    fn draw(&self, ctx: &mut RendererContext<'_>) -> Result<(), GlError> {
        unsafe {
            let program_id = self.program_wrapper.get_program_id()?;
            let vao_ref = self.program_wrapper.get_vao_ref();
            let vbo_ref = self.program_wrapper.get_vbo_ref();
            let vertices_len = self.vertices.len();
            let vertices_ptr = self.vertices.as_slice().as_ptr();
            let created_variable = !self.initialized.get();
            let num_points = self.vertices.ncols();
            ctx.add_commands(move || {
                gl::UseProgram(program_id);
                gl::BindBuffer(gl::ARRAY_BUFFER, vbo_ref);
                gl::BindVertexArray(vao_ref);
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
                // LINE_LOOP
                gl::DrawArrays(gl::POINTS, 0, num_points as i32);
            });
            self.initialized.set(true);
            Ok(())
        }
    }
}

impl TestGameObject {
    pub fn get_vertices_as_mut(&mut self) -> &mut Matrix3xX<f32> {
        &mut self.vertices
    }
}

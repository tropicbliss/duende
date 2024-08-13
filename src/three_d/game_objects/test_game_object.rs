use crate::common::{
    drawables::{Drawable, RendererContext},
    errors::GlError,
    gl,
    helpers::{Fragment, Shader, Vertex},
    wrappers::program_wrapper::ProgramWrapper,
};
use nalgebra::{Matrix3xX, Matrix6xX};

static FRAGMENT: Shader<Fragment> =
    Shader::create_fragment_shader(include_str!("shaders/fragment_shader.glsl"));

static VERTEX: Shader<Vertex> =
    Shader::create_vertex_shader(include_str!("shaders/vertex_shader.glsl"));

pub struct TestGameObject {
    program_wrapper: ProgramWrapper,
    vertices: Matrix6xX<f32>,
}

impl TestGameObject {
    pub fn new(vertices: Matrix3xX<f32>, colors: Matrix3xX<f32>) -> Self {
        Self {
            program_wrapper: ProgramWrapper::new(&VERTEX, &FRAGMENT),
            vertices: interleave_matrices(vertices, colors),
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
            let num_points = self.vertices.ncols();
            let variable_helper = self.program_wrapper.get_variable_helper();
            ctx.add_commands(move || {
                gl::UseProgram(program_id);
                gl::BindBuffer(gl::ARRAY_BUFFER, vbo_ref);
                gl::BindVertexArray(vao_ref);
                if let Some(ref var_helper) = variable_helper {
                    var_helper
                        .create_variables(vec!["position", "vertex_color"])
                        .unwrap();
                }
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (vertices_len * std::mem::size_of::<f32>()) as isize,
                    vertices_ptr as *const _,
                    gl::STATIC_DRAW,
                );
                gl::DrawArrays(gl::TRIANGLE_STRIP, 0, num_points as i32);
            });
            Ok(())
        }
    }
}

impl TestGameObject {
    pub fn get_data_as_mut(&mut self) -> &mut Matrix6xX<f32> {
        &mut self.vertices
    }
}

fn interleave_matrices(first: Matrix3xX<f32>, second: Matrix3xX<f32>) -> Matrix6xX<f32> {
    assert_eq!(first.ncols(), second.ncols());
    let ncols = first.ncols();
    let mut result = Matrix6xX::<f32>::zeros(ncols);
    for i in 0..ncols {
        result
            .fixed_view_mut::<3, 1>(0, i)
            .copy_from(&first.fixed_view::<3, 1>(0, i));
        result
            .fixed_view_mut::<3, 1>(3, i)
            .copy_from(&second.fixed_view::<3, 1>(0, i));
    }
    result
}

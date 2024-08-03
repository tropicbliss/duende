use crate::common::{
    errors::GlError,
    gl,
    helpers::{create_program, Fragment, Shader, Vertex},
};
use std::cell::OnceCell;

pub struct ProgramWrapper {
    program_id: OnceCell<Result<u32, GlError>>,
    vao_ref: OnceCell<u32>,
    vbo_ref: OnceCell<u32>,
    vertex_shader: &'static Shader<Vertex>,
    fragment_shader: &'static Shader<Fragment>,
}

impl ProgramWrapper {
    pub fn new(
        vertex_shader: &'static Shader<Vertex>,
        fragment_shader: &'static Shader<Fragment>,
    ) -> Self {
        Self {
            program_id: OnceCell::new(),
            vao_ref: OnceCell::new(),
            vbo_ref: OnceCell::new(),
            vertex_shader,
            fragment_shader,
        }
    }

    pub unsafe fn get_program_id(&self) -> Result<u32, GlError> {
        self.program_id
            .get_or_init(|| {
                let vertex_shader = self.vertex_shader.get_shader_handle()?;
                let fragment_shader = self.fragment_shader.get_shader_handle()?;
                let program_id = create_program(&vertex_shader, &fragment_shader)?;
                Ok(program_id)
            })
            .clone()
    }

    pub unsafe fn get_vao_ref(&self) -> u32 {
        *self.vao_ref.get_or_init(|| {
            let mut vao_ref = 0;
            gl::GenVertexArrays(1, &mut vao_ref);
            vao_ref
        })
    }

    pub unsafe fn get_vbo_ref(&self) -> u32 {
        *self.vbo_ref.get_or_init(|| {
            let mut vbo_ref = 0;
            gl::GenBuffers(1, &mut vbo_ref);
            vbo_ref
        })
    }
}

impl Drop for ProgramWrapper {
    fn drop(&mut self) {
        unsafe {
            if let Some(Ok(program_id)) = self.program_id.get() {
                gl::DeleteProgram(*program_id);
            }
        }
    }
}

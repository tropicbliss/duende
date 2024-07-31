use std::cell::OnceCell;

use crate::common::drawables::Shader;

use super::{
    drawables::{Fragment, Program, Vertex},
    errors::GlError,
    gl,
};

pub struct ProgramWrapper {
    program: OnceCell<Result<Program, GlError>>,
    vertex_shader: &'static Shader<Vertex>,
    fragment_shader: &'static Shader<Fragment>,
}

impl ProgramWrapper {
    pub fn new(
        vertex_shader: &'static Shader<Vertex>,
        fragment_shader: &'static Shader<Fragment>,
    ) -> Self {
        Self {
            program: OnceCell::new(),
            vertex_shader,
            fragment_shader,
        }
    }

    pub unsafe fn get_program_id(&self) -> Result<u32, GlError> {
        let program_result = self.program.get_or_init(|| {
            Program::new(
                self.vertex_shader.get_shader_handle()?,
                self.fragment_shader.get_shader_handle()?,
            )
        });
        let program = match program_result {
            Ok(prog) => prog,
            Err(e) => {
                return Err(e.clone());
            }
        };
        Ok(program.program_id)
    }
}

pub struct VaoWrapper {
    vao_ref: OnceCell<u32>,
}

impl VaoWrapper {
    pub const fn new() -> Self {
        Self {
            vao_ref: OnceCell::new(),
        }
    }

    pub fn get_vao_ref(&self) -> u32 {
        *self.vao_ref.get_or_init(|| unsafe {
            let mut vao_ref = 0;
            gl::GenVertexArrays(1, &mut vao_ref);
            vao_ref
        })
    }
}

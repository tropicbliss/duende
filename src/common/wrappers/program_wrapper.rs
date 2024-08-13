use crate::common::{
    errors::GlError,
    gl,
    helpers::{create_program, Fragment, Shader, Vertex},
};
use std::{
    cell::{Cell, OnceCell},
    ffi::CString,
};

pub struct ProgramWrapper {
    program_id: OnceCell<Result<u32, GlError>>,
    vao_ref: OnceCell<u32>,
    vbo_ref: OnceCell<u32>,
    vertex_shader: &'static Shader<Vertex>,
    fragment_shader: &'static Shader<Fragment>,
    variable_created: Cell<bool>,
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
            variable_created: Cell::new(false),
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

    pub fn get_variable_helper(&self) -> Option<VariableHelper> {
        if !self.variable_created.get() {
            if let Some(Ok(program_id)) = self.program_id.get() {
                self.variable_created.set(true);
                return Some(VariableHelper::new(*program_id));
            }
        }
        None
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

pub struct VariableHelper {
    program_id: u32,
}

impl VariableHelper {
    fn new(program_id: u32) -> Self {
        Self { program_id }
    }

    pub unsafe fn create_variables(
        &self,
        variable_names: Vec<&'static str>,
    ) -> Result<(), GlError> {
        let stride = (3 * variable_names.len() * std::mem::size_of::<f32>()) as i32;
        let mut offset = 0;
        for variable_name in variable_names {
            let attrib_name = CString::new(variable_name).map_err(|_| GlError::NullByte)?;
            let variable_id = gl::GetAttribLocation(self.program_id, attrib_name.as_ptr());
            if variable_id == -1 {
                return Err(GlError::NonexistantVariableName(variable_name));
            }
            gl::EnableVertexAttribArray(variable_id as u32);
            let ptr = if offset == 0 {
                std::ptr::null()
            } else {
                (offset * std::mem::size_of::<f32>()) as *const f32 as *const std::ffi::c_void
            };
            gl::VertexAttribPointer(variable_id as u32, 3, gl::FLOAT, gl::FALSE, stride, ptr);
            offset += 3;
        }
        Ok(())
    }
}

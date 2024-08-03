use super::{
    errors::GlError,
    gl::{
        self,
        types::{GLchar, GLint},
    },
};
use std::{
    ffi::CString,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, LazyLock,
    },
};

pub struct Fragment;

pub struct Vertex;

pub struct Shader<T> {
    source: &'static str,
    instance: LazyLock<Arc<AtomicU32>>,
    shader_type: std::marker::PhantomData<T>,
}

impl<T> Shader<T> {
    pub fn get_source(&self) -> &'static str {
        self.source
    }
}

impl Shader<Fragment> {
    pub const fn create_fragment_shader(source: &'static str) -> Self {
        Self {
            source,
            instance: LazyLock::new(|| Arc::new(AtomicU32::new(0))),
            shader_type: std::marker::PhantomData,
        }
    }

    pub unsafe fn get_shader_handle(&self) -> Result<ShaderHandle<Fragment>, GlError> {
        if Arc::strong_count(&self.instance) == 1 {
            let shader_id = compile_shader(gl::FRAGMENT_SHADER, self.source.as_bytes())?;
            self.instance.store(shader_id, Ordering::Relaxed);
        }
        Ok(ShaderHandle {
            shader_id: Arc::clone(&self.instance),
            shader_type: std::marker::PhantomData,
        })
    }
}

impl Shader<Vertex> {
    pub const fn create_vertex_shader(source: &'static str) -> Self {
        Self {
            source,
            instance: LazyLock::new(|| Arc::new(AtomicU32::new(0))),
            shader_type: std::marker::PhantomData,
        }
    }

    pub unsafe fn get_shader_handle(&self) -> Result<ShaderHandle<Vertex>, GlError> {
        if Arc::strong_count(&self.instance) == 1 {
            let shader_id = compile_shader(gl::VERTEX_SHADER, self.source.as_bytes())?;
            self.instance.store(shader_id, Ordering::Relaxed);
        }
        Ok(ShaderHandle {
            shader_id: Arc::clone(&self.instance),
            shader_type: std::marker::PhantomData,
        })
    }
}

#[derive(Clone)]
pub struct ShaderHandle<T> {
    shader_id: Arc<AtomicU32>,
    shader_type: std::marker::PhantomData<T>,
}

impl<T> ShaderHandle<T> {
    pub fn get_shader_id(&self) -> u32 {
        self.shader_id.load(Ordering::Relaxed)
    }
}

impl<T> Drop for ShaderHandle<T> {
    fn drop(&mut self) {
        if Arc::strong_count(&self.shader_id) == 2 {
            unsafe {
                gl::DeleteShader(self.get_shader_id());
            }
        }
    }
}

pub unsafe fn compile_shader(
    shader: gl::types::GLenum,
    source: &[u8],
) -> Result<gl::types::GLuint, GlError> {
    let shader = gl::CreateShader(shader);
    let c_str = CString::new(source).map_err(|_| GlError::NullByte)?;
    gl::ShaderSource(shader, 1, &c_str.as_ptr(), std::ptr::null());
    gl::CompileShader(shader);
    let mut compile_status = gl::FALSE as GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compile_status);
    if compile_status != (gl::TRUE as GLint) {
        let mut log_length = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_length);
        let mut log = vec![0; log_length as usize];
        gl::GetShaderInfoLog(
            shader,
            log_length,
            std::ptr::null_mut(),
            log.as_mut_ptr() as *mut GLchar,
        );
        gl::DeleteShader(shader);
        return Err(GlError::ShaderCompile(
            std::str::from_utf8(&log)
                .unwrap_or("Unknown shader compilation error")
                .to_string(),
        ));
    }
    Ok(shader)
}

pub unsafe fn create_program(
    vertex_shader: &ShaderHandle<Vertex>,
    fragment_shader: &ShaderHandle<Fragment>,
) -> Result<u32, GlError> {
    let vertex_shader_id = vertex_shader.get_shader_id();
    let fragment_shader_id = fragment_shader.get_shader_id();
    let program_id = gl::CreateProgram();
    gl::AttachShader(program_id, vertex_shader_id);
    gl::AttachShader(program_id, fragment_shader_id);
    gl::LinkProgram(program_id);
    let mut link_status = gl::FALSE as GLint;
    gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut link_status);
    if link_status != (gl::TRUE as GLint) {
        let mut log_length = 0;
        gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut log_length);
        let mut log = vec![0; log_length as usize];
        gl::GetProgramInfoLog(
            program_id,
            log_length,
            std::ptr::null_mut(),
            log.as_mut_ptr() as *mut GLchar,
        );
        gl::DeleteProgram(program_id);
        return Err(GlError::ProgramLink(
            std::str::from_utf8(&log)
                .unwrap_or("Unknown shader linking error")
                .to_string(),
        ));
    }
    Ok(program_id)
}

/// Remember, you only need to call this once
pub unsafe fn create_variable(program_id: u32, variable_name: &'static str) -> Result<(), GlError> {
    let attrib_name = CString::new(variable_name).map_err(|_| GlError::NullByte)?;
    let variable_id = gl::GetAttribLocation(program_id, attrib_name.as_ptr());
    if variable_id == -1 {
        return Err(GlError::NonexistantVariableName(variable_name));
    }
    gl::EnableVertexAttribArray(variable_id as u32);
    gl::VertexAttribPointer(
        variable_id as u32,
        3,
        gl::FLOAT,
        gl::FALSE,
        0,
        std::ptr::null(),
    );
    Ok(())
}

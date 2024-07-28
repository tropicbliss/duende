#[derive(thiserror::Error, Debug)]
pub enum DuendeError {
    #[error("internal engine error: {0:?}")]
    InternalError(Box<dyn std::error::Error>),

    #[error("improper gl call occurred: {0}")]
    GlError(GlError),

    #[error("unsupported device: {0}")]
    UnsupportedDevice(UnsupportedDevice),
}

#[derive(thiserror::Error, Debug)]
pub enum UnsupportedDevice {
    #[error("cursor grab error")]
    CursorGrab,
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum GlError {
    #[error("shader compilation error: {0}")]
    ShaderCompile(String),

    #[error("program link error: {0}")]
    ProgramLink(String),
}

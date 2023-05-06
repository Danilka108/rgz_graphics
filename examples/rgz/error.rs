use crate::ShaderKind;

#[derive(Debug)]
pub enum GlError {
    ShaderCompileError {
        shader_kind: ShaderKind,
        gl_log: String,
    },
    ProgramLinkingError {
        gl_log: String,
    },
}

impl std::fmt::Display for GlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ShaderCompileError {
                shader_kind,
                gl_log,
            } => write!(f, "failed to compile {shader_kind} shader: {gl_log}"),
            Self::ProgramLinkingError { gl_log } => {
                write!(f, "failed to link program: {gl_log}")
            }
        }
    }
}

impl std::error::Error for GlError {}

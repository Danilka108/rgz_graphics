use std::ffi::CString;

use crate::{array::AttribLocation, error::GlError, ShaderKind};

pub(crate) struct ShaderProgramBuilder<V, G, F> {
    vertex_shader: V,
    geometry_shader: G,
    fragment_shader: F,
    gl: gl::Gl,
}

pub(crate) struct None;

impl ShaderProgramBuilder<None, None, None> {
    pub fn new(gl: gl::Gl) -> ShaderProgramBuilder<None, None, None> {
        ShaderProgramBuilder {
            vertex_shader: None,
            geometry_shader: None,
            fragment_shader: None,
            gl,
        }
    }
}

impl<G, F> ShaderProgramBuilder<None, G, F> {
    pub fn vertex_shader(self, source_code: &[u8]) -> ShaderProgramBuilder<Vec<u8>, G, F> {
        let mut source_code = Vec::from(source_code);
        if !matches!(source_code.last(), Some(&b'\0')) {
            source_code.push(b'\0');
        }

        ShaderProgramBuilder {
            vertex_shader: source_code,
            geometry_shader: self.geometry_shader,
            fragment_shader: self.fragment_shader,
            gl: self.gl,
        }
    }
}

impl<V, F> ShaderProgramBuilder<V, None, F> {
    pub fn geometry_shader(self, source_code: &[u8]) -> ShaderProgramBuilder<V, Vec<u8>, F> {
        let mut source_code = Vec::from(source_code);
        if !matches!(source_code.last(), Some(&b'\0')) {
            source_code.push(b'\0');
        }

        ShaderProgramBuilder {
            vertex_shader: self.vertex_shader,
            geometry_shader: source_code,
            fragment_shader: self.fragment_shader,
            gl: self.gl,
        }
    }
}

impl<V, G> ShaderProgramBuilder<V, G, None> {
    pub fn fragment_shader(self, source_code: &[u8]) -> ShaderProgramBuilder<V, G, Vec<u8>> {
        let mut source_code = Vec::from(source_code);
        if !matches!(source_code.last(), Some(&b'\0')) {
            source_code.push(b'\0');
        }

        ShaderProgramBuilder {
            vertex_shader: self.vertex_shader,
            geometry_shader: self.geometry_shader,
            fragment_shader: source_code,
            gl: self.gl,
        }
    }
}

impl ShaderProgramBuilder<Vec<u8>, None, Vec<u8>> {
    pub fn build(self) -> Result<ShaderProgram, GlError> {
        unsafe {
            let vertex_shader_id =
                compile_shader(&self.gl, &self.vertex_shader, ShaderKind::Vertex)?;
            let fragment_shader_id =
                compile_shader(&self.gl, &self.fragment_shader, ShaderKind::Fragment)?;

            let program_id = self.gl.CreateProgram();
            self.gl.AttachShader(program_id, vertex_shader_id);
            self.gl.AttachShader(program_id, fragment_shader_id);

            self.gl.LinkProgram(program_id);
            check_linking_error(&self.gl, program_id)?;

            self.gl.DeleteShader(vertex_shader_id);
            self.gl.DeleteShader(fragment_shader_id);

            Ok(ShaderProgram {
                gl: self.gl,
                id: program_id,
            })
        }
    }
}

impl ShaderProgramBuilder<Vec<u8>, Vec<u8>, Vec<u8>> {
    pub fn build(self) -> Result<ShaderProgram, GlError> {
        unsafe {
            let vertex_shader_id =
                compile_shader(&self.gl, &self.vertex_shader, ShaderKind::Vertex)?;
            let geometry_shader_id =
                compile_shader(&self.gl, &self.geometry_shader, ShaderKind::Geometry)?;
            let fragment_shader_id =
                compile_shader(&self.gl, &self.fragment_shader, ShaderKind::Fragment)?;

            let program_id = self.gl.CreateProgram();
            self.gl.AttachShader(program_id, vertex_shader_id);
            self.gl.AttachShader(program_id, geometry_shader_id);
            self.gl.AttachShader(program_id, fragment_shader_id);

            self.gl.LinkProgram(program_id);
            check_linking_error(&self.gl, program_id)?;

            self.gl.DeleteShader(vertex_shader_id);
            self.gl.DeleteShader(geometry_shader_id);
            self.gl.DeleteShader(fragment_shader_id);

            Ok(ShaderProgram {
                gl: self.gl,
                id: program_id,
            })
        }
    }
}

unsafe fn compile_shader(
    gl: &gl::Gl,
    source_code: &[u8],
    shader_kind: ShaderKind,
) -> Result<gl::types::GLuint, GlError> {
    let shader_id = gl.CreateShader(match shader_kind {
        ShaderKind::Vertex => gl::VERTEX_SHADER,
        ShaderKind::Geometry => gl::GEOMETRY_SHADER,
        ShaderKind::Fragment => gl::FRAGMENT_SHADER,
    });

    gl.ShaderSource(
        shader_id,
        1,
        [source_code.as_ptr().cast()].as_ptr(),
        std::ptr::null(),
    );

    gl.CompileShader(shader_id);
    check_compile_error(gl, shader_id, shader_kind)?;

    Ok(shader_id)
}

unsafe fn check_compile_error(
    gl: &gl::Gl,
    shader_id: gl::types::GLuint,
    shader_kind: ShaderKind,
) -> Result<(), GlError> {
    let mut soccess: gl::types::GLint = gl::TRUE as gl::types::GLint;
    gl.GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut soccess);

    if soccess == gl::TRUE as gl::types::GLint {
        return Ok(());
    }

    Err(GlError::ShaderCompileError {
        shader_kind,
        gl_log: get_log(gl, shader_id),
    })
}

unsafe fn check_linking_error(gl: &gl::Gl, program_id: gl::types::GLuint) -> Result<(), GlError> {
    let mut soccess: gl::types::GLint = gl::TRUE as gl::types::GLint;
    gl.GetShaderiv(program_id, gl::LINK_STATUS, &mut soccess);

    if soccess == gl::TRUE as gl::types::GLint {
        return Ok(());
    }

    Err(GlError::ProgramLinkingError {
        gl_log: get_log(gl, program_id),
    })
}

pub unsafe fn get_log(gl: &gl::Gl, id: gl::types::GLuint) -> String {
    let mut log_len = 0;
    gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut log_len);

    let mut log_buff = Vec::with_capacity(log_len as usize + 1);
    log_buff.extend([b' '].iter().cycle().take(log_len as usize));

    let log = CString::from_vec_unchecked(log_buff);

    gl.GetShaderInfoLog(
        id,
        log_len,
        std::ptr::null_mut(),
        log.as_ptr() as *mut gl::types::GLchar,
    );

    log.into_string().unwrap()
}

pub(crate) struct ShaderProgram {
    gl: gl::Gl,
    id: gl::types::GLuint,
}

impl ShaderProgram {
    pub fn use_program(&self) {
        unsafe {
            self.gl.UseProgram(self.id);
        }
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn attrib_location_of(&self, name: &str) -> AttribLocation {
        let name = CString::new(name).unwrap();

        let id = unsafe { self.gl.GetAttribLocation(self.id, name.as_ptr()) };
        AttribLocation::new(id as gl::types::GLuint)
    }

    pub fn set_uniform_bool(&self, name: &str, value: bool) {
        let name = CString::new(name).unwrap();

        unsafe {
            self.gl.ProgramUniform1i(
                self.id,
                self.gl.GetUniformLocation(self.id, name.as_ptr()),
                value.into(),
            );
        }
    }

    pub fn set_uniform_i32(&self, name: &str, value: i32) {
        let name = CString::new(name).unwrap();

        unsafe {
            self.gl.ProgramUniform1i(
                self.id,
                self.gl.GetUniformLocation(self.id, name.as_ptr()),
                value,
            );
        }
    }

    pub fn set_uniform_u32(&self, name: &str, value: u32) {
        let name = CString::new(name).unwrap();

        unsafe {
            self.gl.ProgramUniform1ui(
                self.id,
                self.gl.GetUniformLocation(self.id, name.as_ptr()),
                value,
            );
        }
    }

    pub fn set_uniform_f32(&self, name: &str, value: f32) {
        let name = CString::new(name).unwrap();

        unsafe {
            self.gl.ProgramUniform1f(
                self.id,
                self.gl.GetUniformLocation(self.id, name.as_ptr()),
                value,
            );
        }
    }

    pub fn set_uniform_vec2(&self, name: &str, value: [f32; 2]) {
        let name = CString::new(name).unwrap();

        unsafe {
            self.gl.ProgramUniform2fv(
                self.id,
                self.gl.GetUniformLocation(self.id, name.as_ptr()),
                1,
                value.as_ptr(),
            );
        }
    }

    pub fn set_uniform_vec3(&self, name: &str, value: [f32; 3]) {
        let name = CString::new(name).unwrap();

        unsafe {
            self.gl.ProgramUniform3fv(
                self.id,
                self.gl.GetUniformLocation(self.id, name.as_ptr()),
                1,
                value.as_ptr(),
            );
        }
    }

    pub fn set_uniform_vec4(&self, name: &str, value: [f32; 4]) {
        let name = CString::new(name).unwrap();

        unsafe {
            self.gl.ProgramUniform4fv(
                self.id,
                self.gl.GetUniformLocation(self.id, name.as_ptr()),
                1,
                value.as_ptr(),
            );
        }
    }

    pub fn set_uniform_mat2(&self, name: &str, value: [f32; 4]) {
        let name = CString::new(name).unwrap();

        unsafe {
            self.gl.ProgramUniformMatrix2fv(
                self.id,
                self.gl.GetUniformLocation(self.id, name.as_ptr()),
                1,
                gl::FALSE,
                value.as_ptr(),
            );
        }
    }

    pub fn set_uniform_mat3(&self, name: &str, value: [f32; 9]) {
        let name = CString::new(name).unwrap();

        unsafe {
            self.gl.ProgramUniformMatrix3fv(
                self.id,
                self.gl.GetUniformLocation(self.id, name.as_ptr()),
                1,
                gl::FALSE,
                value.as_ptr(),
            );
        }
    }

    pub fn set_uniform_mat4(&self, name: &str, value: [f32; 16]) {
        let name = CString::new(name).unwrap();

        unsafe {
            self.gl.ProgramUniformMatrix4fv(
                self.id,
                self.gl.GetUniformLocation(self.id, name.as_ptr()),
                1,
                gl::FALSE,
                &value[0],
            );
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.id);
        }
    }
}

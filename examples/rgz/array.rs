pub(crate) struct Array {
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    gl: gl::Gl,
}

pub(crate) struct Location(gl::types::GLuint);

impl Location {
    pub fn new(value: gl::types::GLuint) -> Self {
        Self(value)
    }
}

pub(crate) struct AttribPointer {
    pub size: Size,
    pub stride: usize,
    pub offset: usize,
    pub ty: gl::types::GLenum,
}

pub(crate) enum Size {
    One = 1isize,
    Two = 2,
    Three = 3,
    Four = 4,
}

impl Array {
    pub fn new<T>(gl: gl::Gl, data: Vec<T>) -> Self {
        unsafe {
            let mut vao = 0;
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);

            let mut vbo = 0;
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);

            gl.BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * std::mem::size_of::<T>()) as gl::types::GLsizeiptr,
                data.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );

            gl.BindVertexArray(0);
            gl.BindBuffer(gl::ARRAY_BUFFER, 0);

            Array { gl, vao, vbo }
        }
    }

    pub fn set_attrib_pointer(&self, location: Location, value: AttribPointer) {
        unsafe {
            self.gl.BindVertexArray(self.vao);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            self.gl.VertexAttribPointer(
                location.0,
                value.size as i32,
                value.ty,
                gl::FALSE,
                value.stride as i32,
                value.offset as *const _,
            );
            self.gl.EnableVertexAttribArray(location.0);
        }
    }

    pub fn use_array(&self) {
        unsafe {
            self.gl.BindVertexArray(self.vao);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        }
    }
}

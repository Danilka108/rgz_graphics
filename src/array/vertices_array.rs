use super::{AttribLocation, AttribPointer};

pub(crate) struct VerticesArray {
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    count: usize,
    gl: gl::Gl,
}

impl VerticesArray {
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

            VerticesArray {
                gl,
                vao,
                vbo,
                count: data.len(),
            }
        }
    }

    pub fn set_attrib_pointer(
        &self,
        location: AttribLocation,
        value: AttribPointer,
        normalized: bool,
    ) {
        unsafe {
            self.gl.BindVertexArray(self.vao);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            self.gl.VertexAttribPointer(
                location.0,
                value.size as i32,
                value.ty,
                if normalized { gl::TRUE } else { gl::FALSE },
                value.stride as i32,
                value.offset as *const _,
            );
            self.gl.EnableVertexAttribArray(location.0);
        }
    }

    pub fn set_attrib_int_pointer(&self, location: AttribLocation, value: AttribPointer) {
        unsafe {
            self.gl.BindVertexArray(self.vao);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            self.gl.VertexAttribIPointer(
                location.0,
                value.size as i32,
                value.ty,
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

    pub fn len(&self) -> usize {
        self.count
    }
}

impl Drop for VerticesArray {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteVertexArrays(1, &self.vao);
            self.gl.DeleteBuffers(1, &self.vbo);
        }
    }
}

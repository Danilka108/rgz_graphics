use std::mem;

pub(crate) struct ElementsArray {
    gl: gl::Gl,
    eab: gl::types::GLuint,
    len: usize,
}

impl ElementsArray {
    pub fn new(gl: gl::Gl, indices: &[u16]) -> Self {
        let mut eab = 0;

        unsafe {
            gl.GenBuffers(1, &mut eab);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, eab);

            gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * mem::size_of::<u16>()) as gl::types::GLsizeiptr,
                indices.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );
        }

        Self {
            gl,
            eab,
            len: indices.len(),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn use_array(&self) {
        unsafe {
            self.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.eab);
        }
    }
}

impl Drop for ElementsArray {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteBuffers(1, &self.eab);
        }
    }
}

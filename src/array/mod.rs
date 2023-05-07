mod elements_array;
mod vertices_array;

pub(crate) use elements_array::ElementsArray;
pub(crate) use vertices_array::VerticesArray;

pub(crate) struct AttribLocation(gl::types::GLuint);

impl AttribLocation {
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

use std::path::Path;

pub(crate) struct Texture {
    id: u32,
    gl: gl::Gl,
}

impl Texture {
    pub fn open<P: AsRef<Path>>(gl: gl::Gl, path: P) -> Self {
        let img = image::io::Reader::open(path)
            .unwrap()
            .decode()
            .unwrap()
            .to_rgb16();

        let width = img.width();
        let height = img.height();
        let bytes = img.into_vec();
        // dbg!(width, height, bytes.len());

        let mut texture_id = 0;

        unsafe {
            gl.GenTextures(1, &mut texture_id);
            gl.BindTexture(gl::TEXTURE_2D, texture_id);

            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                width as i32,
                height as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_SHORT,
                bytes.as_ptr() as *const _,
            );
            gl.GenerateMipmap(gl::TEXTURE_2D);
        }

        Self { gl, id: texture_id }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.ActiveTexture(gl::TEXTURE0);
            self.gl.BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
}

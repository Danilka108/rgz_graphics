extern crate gl_generator;

use std::{env, fs::File, path::PathBuf};

use gl_generator::{Api, Registry, StructGenerator};

fn main() {
    let dest = PathBuf::from(&env::var("OUT_DIR").unwrap());
    let mut file = File::create(dest.join("gl_bindings.rs")).unwrap();

    Registry::new(
        Api::Gles2,
        (3, 2),
        gl_generator::Profile::Core,
        gl_generator::Fallbacks::All,
        [],
    )
    .write_bindings(StructGenerator, &mut file)
    .unwrap();
}

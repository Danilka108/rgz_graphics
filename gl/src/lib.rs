#![allow(clippy::all)]
include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));

pub use Gles2 as Gl;

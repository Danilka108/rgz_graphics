use gl_window_provider::GlWindowProvider;
use renderer::RgzRenderer;
use winit::event_loop::EventLoop;

mod error;
mod renderer;
mod shader_program;
mod array;

#[derive(Debug)]
pub enum ShaderKind {
    Vertex,
    Geometry,
    Fragment,
}

impl std::fmt::Display for ShaderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vertex => write!(f, "vertex"),
            Self::Geometry => write!(f, "geometry"),
            Self::Fragment => write!(f, "fragment"),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let handler = GlWindowProvider::new(&event_loop).build_handler::<RgzRenderer, ()>();
    event_loop.run(handler);
}

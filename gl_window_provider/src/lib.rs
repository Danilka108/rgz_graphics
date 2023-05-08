mod handler;

use glutin::{
    config::{Config, ConfigTemplateBuilder},
    context::{ContextApi, ContextAttributesBuilder, NotCurrentContext},
    display::GetGlDisplay,
    prelude::{GlConfig, GlDisplay},
};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, TouchPhase},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

use crate::handler::EventsHandler;

pub trait Renderer {
    fn new<D>(gl_display: &D) -> Self
    where
        D: GlDisplay;

    fn mouse_input_hook(&mut self, state: ElementState, button: MouseButton) {}

    fn mouse_wheel_hook(&mut self, delta: MouseScrollDelta, phase: TouchPhase) {}

    fn keyboard_input_hook(&mut self, input: KeyboardInput) {}

    fn cursor_move_hook(&mut self, pos: PhysicalPosition<f64>) {}

    fn cursor_enter_hook(&mut self) {}

    fn cursor_left_hook(&mut self) {}

    fn draw(&mut self, width: Option<u32>, height: Option<u32>);

    fn resize(&mut self, width: i32, height: i32);
}

pub struct GlWindowProvider {
    pub(crate) gl_config: Config,
    pub(crate) window: Option<Window>,
    pub(crate) not_current_context: Option<NotCurrentContext>,
}

impl GlWindowProvider {
    pub fn new<T>(event_loop: &EventLoop<T>) -> Self {
        let win_builder = if cfg!(wgl_backend) {
            Some(WindowBuilder::new())
        } else {
            None
        };

        let (window, gl_config) = Self::build_display(event_loop, win_builder);

        let not_current_context =
            Some(Self::build_not_current_context(&gl_config, window.as_ref()));

        Self {
            gl_config,
            window,
            not_current_context,
        }
    }

    fn build_display<T>(
        event_loop: &EventLoop<T>,
        win_builder: Option<WindowBuilder>,
    ) -> (Option<Window>, Config) {
        let template = ConfigTemplateBuilder::new().with_alpha_size(8);

        DisplayBuilder::new()
            .with_window_builder(win_builder)
            .build(&event_loop, template, |configs| {
                configs
                    .max_by(|a, b| a.num_samples().cmp(&b.num_samples()))
                    .unwrap()
            })
            .unwrap()
    }

    fn build_not_current_context(gl_config: &Config, window: Option<&Window>) -> NotCurrentContext {
        let gl_display = gl_config.display();
        let raw_window_handle = window.map(|window| window.raw_window_handle());

        let context_attrs = ContextAttributesBuilder::new().build(raw_window_handle);
        let fallback_context_attrs = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(raw_window_handle);

        unsafe {
            gl_display
                .create_context(&gl_config, &context_attrs)
                .unwrap_or_else(|_| {
                    gl_display
                        .create_context(&gl_config, &fallback_context_attrs)
                        .unwrap()
                })
        }
    }

    pub fn build_handler<R, T>(
        self,
    ) -> impl for<'event, 'win_target, 'control_flow> FnMut(
        Event<'event, T>,
        &'win_target EventLoopWindowTarget<T>,
        &'control_flow mut ControlFlow,
    )
    where
        R: Renderer,
        T: 'static,
    {
        let mut handler = EventsHandler {
            gl_provider: self,
            renderer: None::<R>,
            state: None,
        };

        move |event, win_target, control_flow| handler.handle(event, win_target, control_flow)
    }
}

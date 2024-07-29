#![feature(allocator_api)]

pub mod common;
pub mod three_d;
pub mod two_d;
use common::errors::DuendeError;
pub use winit::keyboard::NamedKey;

mod internal_game_loop;
mod utils;

use bumpalo::Bump;
use glutin::config::ConfigTemplateBuilder;
use glutin_winit::DisplayBuilder;
use internal_game_loop::InnerApplication;
use three_d::application_context::ThreeDApplicationContext;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize, Position, Size},
    event_loop::EventLoop,
    window::Window,
};

pub struct ApplicationBuilder {
    title: String,
    window_size: Option<(u32, u32)>,
    window_position: Option<(i32, i32)>,
    grab_mouse: bool,
    mouse_cursor_visible: bool,
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self {
            title: String::from("duende window"),
            window_size: None,
            window_position: None,
            grab_mouse: false,
            mouse_cursor_visible: true,
        }
    }
}

impl ApplicationBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title<T>(mut self, title: T) -> Self
    where
        T: Into<String>,
    {
        self.title = title.into();
        self
    }

    pub fn window_size(mut self, width: u32, height: u32) -> Self {
        self.window_size = Some((width, height));
        self
    }

    pub fn window_position(mut self, x: i32, y: i32) -> Self {
        self.window_position = Some((x, y));
        self
    }

    pub fn set_grab_mouse(mut self, grab_mouse: bool) -> Self {
        self.grab_mouse = grab_mouse;
        self
    }

    pub fn set_mouse_cursor_visible(mut self, mouse_cursor_visible: bool) -> Self {
        self.mouse_cursor_visible = mouse_cursor_visible;
        self
    }

    pub fn build(self) -> Application {
        Application::new(self)
    }
}

pub struct Application {
    builder: ApplicationBuilder,
}

impl Application {
    pub fn builder() -> ApplicationBuilder {
        ApplicationBuilder::default()
    }

    fn new(builder: ApplicationBuilder) -> Self {
        Self { builder }
    }

    pub fn render<G>(self, game_loop: G) -> Result<(), DuendeError>
    where
        G: Game,
    {
        let event_loop = EventLoop::new().unwrap();
        let mut window_attributes =
            Window::default_attributes().with_title(self.builder.title.clone());
        if let Some((width, height)) = self.builder.window_size {
            window_attributes =
                window_attributes.with_inner_size(Size::Physical(PhysicalSize::new(width, height)));
        }
        if let Some((x, y)) = self.builder.window_position {
            window_attributes =
                window_attributes.with_position(Position::Physical(PhysicalPosition::new(x, y)));
        }
        let template = ConfigTemplateBuilder::new();
        let display_builder =
            DisplayBuilder::new().with_window_attributes(Some(window_attributes.clone()));
        let bump = Bump::new();
        let mut app = InnerApplication::new(
            template,
            display_builder,
            game_loop,
            window_attributes,
            self.builder,
            &bump,
        );
        event_loop.run_app(&mut app).unwrap();
        app.exit_state
    }
}

pub trait Game {
    fn game_loop(&self, context: &mut ThreeDApplicationContext);
    fn setup(&self, _context: &mut ThreeDApplicationContext) {}
    fn teardown(&self, _context: &mut ThreeDApplicationContext) {}
}

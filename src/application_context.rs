use std::ffi::{CStr, CString};

use bumpalo::Bump;
use fnv::FnvHashSet;
use glutin::prelude::GlDisplay;
use tracing::info;
use winit::keyboard::NamedKey;

use crate::{
    drawables::Drawable, errors::GlError, gl, internal_game_loop::RendererContext,
    mut_cell::MutCell,
};

pub struct ApplicationContext<'a> {
    input_events: FnvHashSet<Event>,
    output_commands: Vec<Command, &'a Bump>,
    background_color: MutCell<InternalColor>,
    renderer_context: RendererContext<'a>,
    exit_status: Result<(), GlError>,
}

impl<'a> ApplicationContext<'a> {
    pub(crate) fn new<D>(gl_display: &D, bump: &'a Bump) -> Self
    where
        D: GlDisplay,
    {
        unsafe {
            gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });
            if let Some(renderer) = get_gl_string(gl::RENDERER) {
                info!("Running on {}", renderer.to_string_lossy());
            }
            if let Some(version) = get_gl_string(gl::VERSION) {
                info!("OpenGL Version {}", version.to_string_lossy());
            }
            if let Some(shaders_version) = get_gl_string(gl::SHADING_LANGUAGE_VERSION) {
                info!("Shaders version on {}", shaders_version.to_string_lossy());
            }
            gl::MatrixMode(gl::PROJECTION);
            gl::LoadIdentity();
            Self {
                input_events: FnvHashSet::default(),
                output_commands: Vec::new_in(bump),
                background_color: MutCell::new(InternalColor::default()),
                renderer_context: RendererContext::new(bump),
                exit_status: Ok(()),
            }
        }
    }

    pub(crate) fn pop_all_commands(&mut self) -> Vec<Command, &'a Bump> {
        let mut output = Vec::new_in(self.renderer_context.bump);
        std::mem::swap(&mut self.output_commands, &mut output);
        output
    }

    pub fn exit(&mut self) {
        self.output_commands.push(Command::Exit);
    }

    pub(crate) fn resize(&self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
    }

    pub fn set_cursor_grab(&mut self, enable: bool) {
        self.output_commands.push(Command::CursorGrab(enable));
    }

    pub fn set_background_color(&mut self, red: u8, green: u8, blue: u8, alpha: u8) {
        self.background_color.set(InternalColor(
            red as f32 / u8::MAX as f32,
            green as f32 / u8::MAX as f32,
            blue as f32 / u8::MAX as f32,
            alpha as f32 / u8::MAX as f32,
        ));
    }

    pub fn set_cursor_visible(&mut self, enable: bool) {
        self.output_commands.push(Command::CursorVisible(enable));
    }

    pub(crate) fn add_event(&mut self, event: Event) {
        self.input_events.insert(event);
    }

    pub fn draw_game_object<D>(&mut self, object: &D)
    where
        D: Drawable,
    {
        self.exit_status = object.draw(&mut self.renderer_context);
    }

    pub(crate) unsafe fn draw(&mut self) -> Result<(), GlError> {
        if let Err(e) = &self.exit_status {
            return Err(e.clone());
        }
        self.background_color.execute_on_change(|new_value| {
            gl::ClearColor(new_value.0, new_value.1, new_value.2, new_value.3);
        });
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        while let Some(commands) = self.renderer_context.command_queue.pop() {
            (commands)();
        }
        Ok(())
    }

    pub fn is_key_pressed(&self, key: NamedKey) -> bool {
        self.input_events.contains(&Event::KeyPress(key))
    }
}

#[derive(PartialEq, Eq, Hash)]
pub(crate) enum Event {
    KeyPress(NamedKey),
}

pub(crate) enum Command {
    Exit,
    CursorGrab(bool),
    CursorVisible(bool),
}

struct InternalColor(f32, f32, f32, f32);

impl Default for InternalColor {
    fn default() -> Self {
        InternalColor(0.1, 0.1, 0.1, 0.9)
    }
}

fn get_gl_string(variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl::GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

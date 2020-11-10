// https://github.com/iolivia/imgui-ggez-starter/blob/master/src/imgui_wrapper.rs

use ggez::event::{KeyCode, KeyMods, MouseButton};
use ggez::graphics;
use ggez::Context;

use gfx_core::{handle::RenderTargetView, memory::Typed};
use gfx_device_gl;

use imgui::*;
use imgui_gfx_renderer::*;

use std::time::Instant;

use clipboard::ClipboardProvider;

use super::InputEvent;

// TODO go through https://github.com/ocornut/imgui/blob/master/docs/FAQ.md and add stuff we should have

pub struct SystemClickboard;

impl imgui::ClipboardBackend for SystemClickboard {
    fn get(&mut self) -> Option<ImString> {
        if let Ok(mut context) = clipboard::ClipboardContext::new() {
            if let Ok(contents) = context.get_contents() {
                return Some(ImString::new(contents));
            }
        }
        println!("Failed to get clipboard content.");
        None
    }

    fn set(&mut self, value: &ImStr) {
        if let Ok(mut context) = clipboard::ClipboardContext::new() {
            if context.set_contents(value.to_string()).is_err() {
                println!("Failed to set clipboard content.");
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
struct MouseState {
  pos: (i32, i32),
  /// mouse buttons: (left, right, middle)
  pressed: (bool, bool, bool),
  wheel: f32,
  wheel_h: f32,
}

pub struct ImGuiFonts {
  pub open_sans_regular_22: FontId,
  pub open_sans_semi_bold_30: FontId,
}

pub struct ImGuiWrapper {
  pub imgui: imgui::Context,
  pub renderer: Renderer<gfx_core::format::Rgba8, gfx_device_gl::Resources>,
  last_frame: Instant,
  mouse_state: MouseState,
  fonts: ImGuiFonts,
}

impl ImGuiWrapper {
  pub fn new(ctx: &mut Context) -> Self {
    // Create the imgui object
    let mut imgui = imgui::Context::create();
    let (factory, gfx_device, _, _, _) = graphics::gfx_objects(ctx);

    imgui.set_clipboard_backend(Box::new(SystemClickboard));

    // Shaders
    let shaders = {
      let version = gfx_device.get_info().shading_language;
      if version.is_embedded {
        if version.major >= 3 {
          Shaders::GlSlEs300
        } else {
          Shaders::GlSlEs100
        }
      } else if version.major >= 4 {
        Shaders::GlSl400
      } else if version.major >= 3 {
        Shaders::GlSl130
      } else {
        Shaders::GlSl110
      }
    };

    // Renderer
    let mut renderer = Renderer::init(&mut imgui, &mut *factory, shaders).unwrap();

    {
      let io = imgui.io_mut();
      io[Key::Tab] = KeyCode::Tab as _;
      io[Key::LeftArrow] = KeyCode::Left as _;
      io[Key::RightArrow] = KeyCode::Right as _;
      io[Key::UpArrow] = KeyCode::Up as _;
      io[Key::DownArrow] = KeyCode::Down as _;
      io[Key::PageUp] = KeyCode::PageUp as _;
      io[Key::PageDown] = KeyCode::PageDown as _;
      io[Key::Home] = KeyCode::Home as _;
      io[Key::End] = KeyCode::End as _;
      io[Key::Insert] = KeyCode::Insert as _;
      io[Key::Delete] = KeyCode::Delete as _;
      io[Key::Backspace] = KeyCode::Back as _;
      io[Key::Space] = KeyCode::Space as _;
      io[Key::Enter] = KeyCode::Return as _;
      io[Key::Escape] = KeyCode::Escape as _;
      io[Key::KeyPadEnter] = KeyCode::NumpadEnter as _;
      io[Key::A] = KeyCode::A as _;
      io[Key::C] = KeyCode::C as _;
      io[Key::V] = KeyCode::V as _;
      io[Key::X] = KeyCode::X as _;
      io[Key::Y] = KeyCode::Y as _;
      io[Key::Z] = KeyCode::Z as _;
    }

    let open_sans_regular_bytes: &'static [u8] = include_bytes!("../../resources/fonts/open_sans/OpenSans-Regular.ttf");
    let open_sans_semi_bold_bytes: &'static [u8] = include_bytes!("../../resources/fonts/open_sans/OpenSans-SemiBold.ttf");

    let open_sans_regular_22 = imgui.fonts().add_font(&[FontSource::TtfData {
        data: open_sans_regular_bytes,
        size_pixels: 22.0,
        config: None,
    }]);

    let open_sans_semi_bold_30 = imgui.fonts().add_font(&[FontSource::TtfData {
        data: open_sans_semi_bold_bytes,
        size_pixels: 30.0,
        config: None,
    }]);

    renderer.reload_font_texture(&mut imgui, factory).unwrap();

    // Create instance
    Self {
      imgui,
      renderer,
      last_frame: Instant::now(),
      mouse_state: MouseState::default(),
      fonts: ImGuiFonts {
        open_sans_regular_22,
        open_sans_semi_bold_30,
      },
    }
  }

  pub fn render<F: FnOnce(&Ui, &ImGuiFonts)>(&mut self, ctx: &mut Context, hidpi_factor: f32, f: F) {
       let render_context = self.render_start(ctx, hidpi_factor);
       f(&render_context.ui, &render_context.fonts);
       render_context.render(ctx);
   }

  fn update_mouse(&mut self) {
    self.imgui.io_mut().mouse_pos = [self.mouse_state.pos.0 as f32, self.mouse_state.pos.1 as f32];

    self.imgui.io_mut().mouse_down = [
      self.mouse_state.pressed.0,
      self.mouse_state.pressed.1,
      self.mouse_state.pressed.2,
      false,
      false,
    ];

    self.imgui.io_mut().mouse_wheel = self.mouse_state.wheel;
    self.mouse_state.wheel = 0.0;

    self.imgui.io_mut().mouse_wheel_h = self.mouse_state.wheel_h;
    self.mouse_state.wheel_h = 0.0;
  }

  pub fn update_mouse_pos(&mut self, x: f32, y: f32) {
    self.mouse_state.pos = (x as i32, y as i32);
  }

  pub fn update_mouse_down(&mut self, button: MouseButton) {
    match button {
      MouseButton::Left => self.mouse_state.pressed.0 = true,
      MouseButton::Right => self.mouse_state.pressed.1 = true,
      MouseButton::Middle => self.mouse_state.pressed.2 = true,
      _ => ()
    }
  }

  pub fn update_mouse_up(&mut self, button: MouseButton) {
    match button {
      MouseButton::Left => self.mouse_state.pressed.0 = false,
      MouseButton::Right => self.mouse_state.pressed.1 = false,
      MouseButton::Middle => self.mouse_state.pressed.2 = false,
      _ => ()
    }
  }

  pub fn update_key_down(&mut self, key: KeyCode, mods: KeyMods) {
    self.imgui.io_mut().key_shift = mods.contains(KeyMods::SHIFT);
    self.imgui.io_mut().key_ctrl = mods.contains(KeyMods::CTRL);
    self.imgui.io_mut().key_alt = mods.contains(KeyMods::ALT);
    self.imgui.io_mut().keys_down[key as usize] = true;
  }

  pub fn update_key_up(&mut self, key: KeyCode, mods: KeyMods) {
    if mods.contains(KeyMods::SHIFT) {
      self.imgui.io_mut().key_shift = false;
    }
    if mods.contains(KeyMods::CTRL) {
      self.imgui.io_mut().key_ctrl = false;
    }
    if mods.contains(KeyMods::ALT) {
      self.imgui.io_mut().key_alt = false;
    }
    self.imgui.io_mut().keys_down[key as usize] = false;
  }

  pub fn update_text(&mut self, val: char) {
    self.imgui.io_mut().add_input_character(val);
  }

  pub fn update_scroll(&mut self, x: f32, y: f32) {
    self.mouse_state.wheel += y;
    self.mouse_state.wheel_h += x;
  }

  pub fn want_capture_mouse(&self) -> bool {
      self.imgui.io().want_capture_mouse
  }

  pub fn want_capture_keyboard(&self) -> bool {
      self.imgui.io().want_capture_keyboard
  }

  // return want capture
  pub fn handle_event(&mut self, event: &InputEvent) -> bool {
      match *event {
          InputEvent::MouseMotionEvent { x, y } => {
              self.update_mouse_pos(x, y);
              if self.want_capture_mouse() {
                  return true;
              }
          }
          // TODO?
          InputEvent::MouseDownEvent { button, .. } => {
              self.update_mouse_down(button);
              if self.want_capture_mouse() {
                  return true;
              }
          }
          // TODO?
          InputEvent::MouseUpEvent { button, .. } => {
              self.update_mouse_up(button);
              if self.want_capture_mouse() {
                  return true;
              }
          }
          InputEvent::KeyDownEvent { code, mods } => {
              self.update_key_down(code, mods);
              if self.want_capture_keyboard() {
                  return true;
              }
          }
          InputEvent::KeyUpEvent { code, mods } => {
              self.update_key_up(code, mods);
              if self.want_capture_keyboard() {
                  return true;
              }
          }
          InputEvent::TextInputEvent(val) => {
              self.update_text(val);
              if self.want_capture_keyboard() {
                  return true;
              }
          }
          InputEvent::ScrollEvent { x, y } => {
              self.update_scroll(x, y);
              if self.want_capture_mouse() {
                  return true;
              }
          }
          InputEvent::Quit => {}
      }

      false
  }

  pub fn render_start<'a, 'b>(&'a mut self, ctx: &'b mut Context, hidpi_factor: f32) -> ImGuiRenderContext {
    // Update mouse
    self.update_mouse();

    // Create new frame
    let now = Instant::now();
    let delta = now - self.last_frame;
    let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
    self.last_frame = now;

    let (draw_width, draw_height) = graphics::drawable_size(ctx);
    self.imgui.io_mut().display_size = [draw_width, draw_height];
    self.imgui.io_mut().display_framebuffer_scale = [hidpi_factor, hidpi_factor];
    self.imgui.io_mut().delta_time = delta_s;

    let ui = self.imgui.frame();
    let default_font_handle = ui.push_font(self.fonts.open_sans_regular_22);

    ImGuiRenderContext {
        ui,
        renderer: &mut self.renderer,
        default_font_handle,
        fonts: &self.fonts,
    }
  }
}

pub struct ImGuiRenderContext<'a> {
    pub ui: Ui<'a>,
    pub fonts: &'a ImGuiFonts,
    renderer: &'a mut Renderer<gfx_core::format::Rgba8, gfx_device_gl::Resources>,
    default_font_handle: FontStackToken,
}

impl<'a> ImGuiRenderContext<'a> {
    pub fn render(self, ctx: &mut Context) {
        self.default_font_handle.pop(&self.ui);
        let (factory, _, encoder, _, render_target) = graphics::gfx_objects(ctx);
        let draw_data = self.ui.render();
        self
          .renderer
          .render(
            &mut *factory,
            encoder,
            &mut RenderTargetView::new(render_target.clone()),
            draw_data,
          )
          .unwrap();
    }
}

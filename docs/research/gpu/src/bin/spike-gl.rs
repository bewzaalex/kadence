//! The spike on winit + glutin + glow (OpenGL 3.3 core, or OpenGL ES 3.0 where that is all there is).

use std::num::NonZeroU32;

use glow::HasContext;
use glutin::config::{Config, ConfigTemplateBuilder, GlConfig};
use glutin::context::{ContextApi, ContextAttributesBuilder, GlProfile, NotCurrentGlContext, PossiblyCurrentContext, Version};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use gpu_spike::{Stats, VERTEX_FLOATS};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::raw_window_handle::HasWindowHandle;
use winit::window::{Window, WindowId};

struct Gl {
    window: Window,
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,
    gl: glow::Context,
    program: glow::Program,
    vbo: glow::Buffer,
}

const VS: &str = "
in vec2 p; in vec2 uv; in vec4 rgba; in float solid;
out vec2 v_uv; out vec4 v_rgba; out float v_solid;
void main() { v_uv = uv; v_rgba = rgba; v_solid = solid; gl_Position = vec4(p, 0.0, 1.0); }";
const FS: &str = "
uniform sampler2D tex;
in vec2 v_uv; in vec4 v_rgba; in float v_solid;
out vec4 o;
void main() { o = v_solid > 0.5 ? v_rgba : texture(tex, v_uv) * v_rgba; }";

impl Gl {
    fn new(el: &ActiveEventLoop, stats: &mut Stats) -> Self {
        // a see-through window needs a config with an alpha channel the compositor uses
        let template = ConfigTemplateBuilder::new().with_alpha_size(8).with_transparency(true);
        let (window, config) = DisplayBuilder::new()
            .with_window_attributes(Some(gpu_spike::window_attributes()))
            .build(el, template, |configs| {
                configs
                    .reduce(|best, c| if c.supports_transparency().unwrap_or(false) && !best.supports_transparency().unwrap_or(false) { c } else { best })
                    .expect("a GL config")
            })
            .expect("GL display");
        let window = window.expect("window");
        let display = config.display();
        let raw = window.window_handle().ok().map(|h| h.as_raw());
        // OpenGL 3.3 core (macOS gives nothing newer than 2.1 otherwise), else OpenGL ES 3.0
        let (context, es) = unsafe {
            let core = ContextAttributesBuilder::new()
                .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
                .with_profile(GlProfile::Core)
                .build(raw);
            let gles = ContextAttributesBuilder::new().with_context_api(ContextApi::Gles(Some(Version::new(3, 0)))).build(raw);
            match display.create_context(&config, &core) {
                Ok(c) => (c, false),
                Err(_) => (display.create_context(&config, &gles).expect("GL context"), true),
            }
        };
        let attrs = window.build_surface_attributes(Default::default()).expect("surface attributes");
        let surface = unsafe { display.create_window_surface(&config, &attrs).expect("GL surface") };
        let context = context.make_current(&surface).expect("make current");
        let _ = surface.set_swap_interval(&context, SwapInterval::Wait(NonZeroU32::MIN));
        let gl = unsafe { glow::Context::from_loader_function_cstr(|s| display.get_proc_address(s)) };
        stats.info = describe(&gl, &config, es);

        unsafe {
            let header = if es { "#version 300 es\nprecision mediump float;\n" } else { "#version 330 core\n" };
            let program = gl.create_program().unwrap();
            for (kind, src) in [(glow::VERTEX_SHADER, VS), (glow::FRAGMENT_SHADER, FS)] {
                let sh = gl.create_shader(kind).unwrap();
                gl.shader_source(sh, &format!("{header}{src}"));
                gl.compile_shader(sh);
                assert!(gl.get_shader_compile_status(sh), "{}", gl.get_shader_info_log(sh));
                gl.attach_shader(program, sh);
            }
            gl.link_program(program);
            assert!(gl.get_program_link_status(program), "{}", gl.get_program_info_log(program));
            gl.use_program(Some(program));

            // the panel picture, with its mipmaps
            let tex = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            for (i, l) in gpu_spike::mip_levels(gpu_spike::load_panel()).iter().enumerate() {
                let px = glow::PixelUnpackData::Slice(Some(&l.rgba));
                gl.tex_image_2d(glow::TEXTURE_2D, i as i32, glow::RGBA8 as i32, l.w as i32, l.h as i32, 0, glow::RGBA, glow::UNSIGNED_BYTE, px);
            }
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR_MIPMAP_LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);

            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));
            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            let mut at = 0;
            for (name, n) in [("p", 2), ("uv", 2), ("rgba", 4), ("solid", 1)] {
                let loc = gl.get_attrib_location(program, name).unwrap();
                gl.enable_vertex_attrib_array(loc);
                gl.vertex_attrib_pointer_f32(loc, n, glow::FLOAT, false, (VERTEX_FLOATS * 4) as i32, at * 4);
                at += n;
            }
            gl.enable(glow::BLEND);
            gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
            Gl { window, surface, context, gl, program, vbo }
        }
    }

    fn resize(&self, w: u32, h: u32) {
        if let (Some(w), Some(h)) = (NonZeroU32::new(w), NonZeroU32::new(h)) {
            self.surface.resize(&self.context, w, h);
        }
    }

    fn draw(&self, t: f32) {
        let size = self.window.inner_size();
        let verts = gpu_spike::scene(t);
        unsafe {
            let gl = &self.gl;
            gl.viewport(0, 0, size.width as i32, size.height as i32);
            gl.clear_color(0.0, 0.0, 0.0, 0.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
            gl.use_program(Some(self.program));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            let bytes = std::slice::from_raw_parts(verts.as_ptr().cast::<u8>(), std::mem::size_of_val(&verts[..]));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes, glow::STREAM_DRAW);
            gl.draw_arrays(glow::TRIANGLES, 0, (verts.len() / VERTEX_FLOATS) as i32);
        }
        if std::env::var_os("SPIKE_NO_NOTIFY").is_none() {
            self.window.pre_present_notify();
        }
        self.surface.swap_buffers(&self.context).expect("swap");
    }
}

fn describe(gl: &glow::Context, config: &Config, es: bool) -> String {
    let (version, renderer) = unsafe { (gl.get_parameter_string(glow::VERSION), gl.get_parameter_string(glow::RENDERER)) };
    format!("{} {version} on {renderer}, alpha {} bits, transparency {:?}", if es { "GLES" } else { "GL" }, config.alpha_size(), config.supports_transparency())
}

struct App {
    gl: Option<Gl>,
    stats: Stats,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.gl.is_none() {
            self.gl = Some(Gl::new(el, &mut self.stats));
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(gl) = &mut self.gl else { return };
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(s) => gl.resize(s.width, s.height),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                if matches!(event.physical_key, PhysicalKey::Code(KeyCode::Escape | KeyCode::KeyQ)) {
                    el.exit();
                }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                let _ = gl.window.drag_window();
            }
            WindowEvent::RedrawRequested => {
                let t = self.stats.t();
                gpu_spike::resize_test(&gl.window, t);
                gl.draw(t);
                if !self.stats.frame() {
                    return el.exit();
                }
                gl.window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    let el = EventLoop::new().expect("event loop");
    el.run_app(&mut App { gl: None, stats: Stats::new("gl") }).expect("run");
}

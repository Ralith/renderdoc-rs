// Copyright 2017-2019 Eyal Kalderon
// Copyright 2015 The Gfx-rs Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate renderdoc;

use gfx::traits::FactoryExt;
use gfx::Device;
use glutin::GlProfile;
use renderdoc::{RenderDoc, V110};

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

const TRIANGLE: [Vertex; 3] = [
    Vertex {
        pos: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        pos: [0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        pos: [0.0, 0.5],
        color: [0.0, 0.0, 1.0],
    },
];

const CLEAR_COLOR: [f32; 4] = [0.1, 0.2, 0.3, 1.0];

pub fn main() {
    let mut rd: RenderDoc<V110> = RenderDoc::new().unwrap();

    let mut events_loop = glutin::EventsLoop::new();
    let window_config = glutin::WindowBuilder::new()
        .with_title("Triangle example".to_string())
        .with_dimensions((1024, 768).into());
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_gl_profile(GlProfile::Core);

    let (window, mut device, mut factory, main_color, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(window_config, context, &events_loop)
            .unwrap();

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let pso = factory
        .create_pipeline_simple(
            include_bytes!("shader/triangle_150.glslv"),
            include_bytes!("shader/triangle_150.glslf"),
            pipe::new(),
        )
        .unwrap();
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&TRIANGLE, ());
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        out: main_color,
    };

    rd.set_active_window(window.context(), std::ptr::null());;
    rd.set_focus_toggle_keys(&[glutin::VirtualKeyCode::F]);
    rd.set_capture_keys(&[glutin::VirtualKeyCode::C]);
    rd.set_capture_option_u32(renderdoc::CaptureOption::AllowVSync, 1);
    rd.set_capture_option_u32(renderdoc::CaptureOption::ApiValidation, 1);

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            if let glutin::Event::WindowEvent { event, .. } = event {
                match event {
                    glutin::WindowEvent::KeyboardInput {
                        input:
                            glutin::KeyboardInput {
                                virtual_keycode: Some(glutin::VirtualKeyCode::R),
                                state: glutin::ElementState::Pressed,
                                ..
                            },
                        ..
                    } => match rd.launch_replay_ui(true, None) {
                        Ok(pid) => println!("Launched replay UI ({}).", pid),
                        Err(err) => eprintln!("{:?}", err),
                    },
                    glutin::WindowEvent::KeyboardInput {
                        input:
                            glutin::KeyboardInput {
                                virtual_keycode: Some(glutin::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    }
                    | glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(logical_size) => {
                        let dpi_factor = window.window().get_hidpi_factor();
                        window.resize(logical_size.to_physical(dpi_factor));
                        gfx_window_glutin::update_views(&window, &mut data.out, &mut main_depth);
                    }
                    _ => (),
                }
            }
        });

        // draw a frame
        encoder.clear(&data.out, CLEAR_COLOR);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}

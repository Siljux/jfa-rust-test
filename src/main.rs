pub mod jfa;
use crate::jfa::State;
use pollster::FutureExt;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{
    DeviceEvent, ElementState, KeyEvent,
    WindowEvent::{self, *},
};
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

pub struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Window 1")
            .with_inner_size(PhysicalSize::new(2048, 1800))
            .with_cursor(winit::window::CursorIcon::Wait);

        let window = event_loop.create_window(window_attributes).unwrap();
        self.state = Some(State::new(window).block_on());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            CloseRequested
            | KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),
            RedrawRequested => {
                if let Some(state) = self.state.as_mut() {
                    match state.render() {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            state.resize(state.size)
                        }
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            println!("OutOfMemory");
                            event_loop.exit();
                        }

                        // This happens when the a frame takes too long to present
                        Err(wgpu::SurfaceError::Timeout) => {
                            println!("Surface timeout")
                        }
                    }
                }
            }
            Resized(physical_size) => {
                if let Some(state) = self.state.as_mut() {
                    state.resize(physical_size);
                }
            }
            CursorEntered { .. } => {
                if let Some(state) = self.state.as_mut() {
                    // if let Err(e) = state.window().set_cursor_grab(CursorGrabMode::Locked) {
                    //     println!("Error setting cursor grab: {e}");
                    // }
                    // state.window().set_cursor_visible(false);
                }
            }
            _ => {
                if let Some(state) = self.state.as_mut() {
                    state.input(&event);
                }
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        // match event {
        //     DeviceEvent::MouseMotion { delta } => {
        //         if let Some(state) = self.state.as_mut() {
        //             state.update();
        //             state.mouse_movement(delta.0, delta.1);
        //         }
        //     }
        //     _ => (),
        // }
    }
}

fn main() {
    // std::env::set_var("RUST_BACKTRACE", "1");
    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App { state: None };
    if let Err(e) = event_loop.run_app(&mut app) {
        println!("{:?}", e);
    }
}

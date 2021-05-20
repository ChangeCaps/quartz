use crate::input::*;
use crate::mouse::*;
use crate::render::*;
use crate::state::*;
use crate::window::*;
use futures::executor::block_on;
use glam::*;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub struct App {
    pub title: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            title: String::from("Quartz Game"),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn run<T: State + 'static>(
        self,
        state: impl Fn(&RenderResource) -> T,
    ) -> anyhow::Result<()> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(&self.title)
            .build(&event_loop)?;

        let mut render_resource = block_on(RenderResource::new(&window));

        let state = state(&render_resource);

        let mut window_descriptor = WindowDescriptor::default();
        let mut keyboard = Input::new();
        let mut mouse = MouseInput::default();

        let mut state_machine = StateMachine::new(&mut render_resource, state);
        window_descriptor.size = Vec2::new(
            window.inner_size().width as f32,
            window.inner_size().height as f32,
        );

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            if window_descriptor.cursor_grabbed {
                let new_position = Vec2::new(
                    window_descriptor.size.x / 2.0,
                    window_descriptor.size.y / 2.0,
                );
                let delta = new_position - mouse.position;

                if delta.length() > 50.0 {
                    mouse.position = new_position;
                    mouse.prev_position = new_position + delta;

                    window
                        .set_cursor_position(winit::dpi::PhysicalPosition::new(
                            new_position.x,
                            new_position.y,
                        ))
                        .unwrap();
                }
            }

            state_machine.handle_event(&render_resource, &event);

            match event {
                Event::RedrawRequested(_) => {
                    state_machine.update(
                        &mut render_resource,
                        &mut window_descriptor,
                        &keyboard,
                        &mut mouse,
                    );
                    state_machine.render(&mut render_resource);

                    window
                        .set_cursor_grab(window_descriptor.cursor_grabbed)
                        .unwrap();
                    window.set_cursor_visible(window_descriptor.cursor_visible);

                    keyboard.update();
                }
                Event::WindowEvent { event, window_id } if window_id == window.id() => {
                    match event {
                        WindowEvent::Resized(physical_size) => {
                            window_descriptor.size.x = physical_size.width as f32;
                            window_descriptor.size.y = physical_size.height as f32;
                            render_resource.resize_swapchain(physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            window_descriptor.size.x = new_inner_size.width as f32;
                            window_descriptor.size.y = new_inner_size.height as f32;
                            render_resource.resize_swapchain(*new_inner_size);
                        }
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::CursorMoved { position, .. } => {
                            mouse.position.x = position.x as f32;
                            mouse.position.y = position.y as f32;
                        }
                        WindowEvent::MouseInput { state, button, .. } => match state {
                            winit::event::ElementState::Pressed => mouse.input.press(button),
                            winit::event::ElementState::Released => mouse.input.release(button),
                        },
                        WindowEvent::Focused(focused) => {
                            if !focused {
                                window_descriptor.cursor_grabbed = false;
                                window_descriptor.cursor_visible = true;

                                window
                                    .set_cursor_grab(window_descriptor.cursor_grabbed)
                                    .unwrap();
                                window.set_cursor_visible(window_descriptor.cursor_visible);
                            }
                        }
                        WindowEvent::KeyboardInput {
                            input:
                                winit::event::KeyboardInput {
                                    virtual_keycode,
                                    state,
                                    ..
                                },
                            ..
                        } => {
                            if let Some(key) = virtual_keycode {
                                match state {
                                    winit::event::ElementState::Pressed => keyboard.press(key),
                                    winit::event::ElementState::Released => keyboard.release(key),
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Event::NewEvents(cause) => match cause {
                    _ => {
                        state_machine.update(
                            &mut render_resource,
                            &mut window_descriptor,
                            &keyboard,
                            &mut mouse,
                        );
                        state_machine.render(&mut render_resource);

                        window
                            .set_cursor_grab(window_descriptor.cursor_grabbed)
                            .unwrap();
                        window.set_cursor_visible(window_descriptor.cursor_visible);

                        keyboard.update();
                    }
                },
                _ => {}
            }
        });
    }
}

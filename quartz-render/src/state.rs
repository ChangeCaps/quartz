use crate::event::*;
use crate::input::*;
use crate::mouse::*;
use crate::render::*;
use crate::window::*;
use std::time::Instant;

pub enum Trans {
    None,
}

pub struct UpdateCtx<'a> {
    pub render_resource: &'a mut RenderResource,
    pub delta_time: f32,
    pub window: &'a mut WindowDescriptor,
    pub keyboard: &'a Input<Key>,
    pub mouse: &'a MouseInput,
}

pub trait State {
    fn start(&mut self, _render_resource: &mut RenderResource) {}

    fn update(&mut self, _ctx: UpdateCtx<'_>) -> Trans {
        Trans::None
    }

    fn handle_event(
        &mut self,
        _render_resource: &RenderResource,
        _event: &winit::event::Event<()>,
    ) -> Trans {
        Trans::None
    }

    fn render(&mut self, _render_resource: &mut RenderResource) {}
}

pub struct StateMachine {
    last_update: Instant,
    states: Vec<Box<dyn State>>,
}

impl StateMachine {
    pub fn new<T: State + 'static>(render_resource: &mut RenderResource, mut state: T) -> Self {
        state.start(render_resource);

        Self {
            last_update: Instant::now(),
            states: vec![Box::new(state)],
        }
    }

    pub fn transition(&mut self, trans: Trans) {
        match trans {
            Trans::None => {}
        }
    }

    pub fn update(
        &mut self,
        render_resource: &mut RenderResource,
        window_descriptor: &mut WindowDescriptor,
        keyboard: &Input<Key>,
        mouse: &mut MouseInput,
    ) {
        let time = Instant::now();

        mouse.pre_update();

        let ctx = UpdateCtx {
            render_resource,
            delta_time: (time - self.last_update).as_secs_f32(),
            window: window_descriptor,
            keyboard,
            mouse,
        };

        self.last_update = time;

        let trans = self.states.last_mut().unwrap().update(ctx);

        mouse.post_update();

        self.transition(trans);
    }

    pub fn handle_event(
        &mut self,
        render_resource: &RenderResource,
        event: &winit::event::Event<()>,
    ) {
        let trans = self
            .states
            .last_mut()
            .unwrap()
            .handle_event(render_resource, event);

        self.transition(trans);
    }

    pub fn render(&mut self, render_resource: &mut RenderResource) {
        self.states.last_mut().unwrap().render(render_resource);
    }
}

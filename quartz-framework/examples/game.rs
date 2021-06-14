use quartz_framework::prelude::*;

pub struct GameState {
    pipeline: RenderPipeline,
    mesh: Mesh,
    count: usize,
    generate: bool,
}

impl GameState {
    pub fn new(instance: &Instance, target_format: format::TargetFormat) -> Self {
        let shader = Shader::load("examples/shader.vert", "examples/shader.frag").unwrap();
        let pipeline = RenderPipeline::new(
            PipelineDescriptor {
                depth_stencil: None,
                ..PipelineDescriptor::default_settings(shader, target_format)
            },
            instance,
        )
        .unwrap();

        let mut mesh = Mesh::new();
        mesh.set_attribute("vertex_position", vec![Vec3::splat(1.0); 1]);
        mesh.set_indices(vec![0; 1]);

        pipeline.bind("transform", &0.0f32);

        Self {
            pipeline,
            mesh,
            count: 0,
            generate: true,
        }
    }
}

impl State for GameState {
    fn update(&mut self, ctx: UpdateCtx) -> Trans {
        if ctx.keyboard.pressed(&Key::G) {
            self.generate = !self.generate;
        }

        Trans::None
    }

    fn render(&mut self, instance: &Instance, main_view: TextureView) {
        self.count += 1;

        let mut render_ctx = instance.render();

        let desc = RenderPassDescriptor::default_settings(main_view);

        let mut pass = render_ctx.render_pass(&desc, &self.pipeline);

        let n = 10000;

        //self.mesh
        //    .set_attribute("vertex_position", vec![Vec3::splat(1.0); n]);
        //self.mesh.set_indices(vec![0; n]);

        self.pipeline.bind("transform", &20.0f32);
        pass.draw_mesh(&self.mesh);
    }
}

fn main() {
    //env_logger::builder().filter_level(log::LevelFilter::Trace).init();

    App::new().run(GameState::new).unwrap()
}

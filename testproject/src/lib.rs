use egui::*;
use quartz_engine::prelude::*;
use std::sync::Arc;

pub struct Mesh {
    pipeline: Arc<RenderPipeline<format::Rgba8UnormSrgb>>,
}

impl Init for Mesh {
    fn init(render: &mut Render) -> Self {
        Self {
            pipeline: render.pipeline.clone(),
        }
    }
}

impl Component for Mesh {
    type Plugins = Render;

    fn name() -> &'static str {
        "Mesh"
    }

    fn inspector_ui(&mut self, _: &mut Render, ctx: ComponentCtx, ui: &mut Ui) {
        if ui.button("Spawn Child").clicked() {
            ctx.tree
                .spawn_child(Mesh::new(self.pipeline.clone()), ctx.node_id);
        }
    }

    fn update(&mut self, _: &mut Render, ctx: ComponentCtx) {
        ctx.transform.rotation *= Quat::from_rotation_y(0.01);
    }

    fn render(&mut self, render: &mut Render, ctx: ComponentRenderCtx) {
        let size = ctx.render_resource.target_size();

        if let Some(view_proj) = render.camera {
            self.pipeline.bind_uniform("ViewProj", view_proj);
            self.pipeline
                .bind_uniform("Model", ctx.global_transform.matrix());
            self.pipeline.bind_uniform("Aspect", size.x / size.y);

            ctx.render_pass.with_pipeline(&self.pipeline).draw(0..3);
        }
    }
}

impl Mesh {
    pub fn new(pipeline: Arc<RenderPipeline<format::Rgba8UnormSrgb>>) -> Self {
        Self { pipeline }
    }
}

#[derive(Default)]
pub struct Camera {
    pub projection: PerspectiveProjection,
}

impl Camera {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Component for Camera {
    type Plugins = Render;

    fn name() -> &'static str {
        "Camera"
    }

    fn update(&mut self, render: &mut Render, ctx: ComponentCtx) {
        let view_proj = self.projection.matrix() * ctx.global_transform.matrix().inverse();

        render.camera = Some(view_proj);
    }
}

pub struct Render {
    pub pipeline: Arc<RenderPipeline<format::Rgba8UnormSrgb>>,
    pub camera: Option<Mat4>,
}

impl Plugin for Render {
    fn init(ctx: InitCtx) -> Self {
        let shader =
            Shader::from_glsl(include_str!("shader.vert"), include_str!("shader.frag")).unwrap();
        let pipeline = Arc::new(
            RenderPipeline::new(
                PipelineDescriptor::default_settings(shader),
                ctx.render_resource,
            )
            .unwrap(),
        );

        ctx.tree.spawn(Mesh::new(pipeline.clone()));
        ctx.tree.spawn(Camera::new());

        Self {
            pipeline: pipeline.clone(),
            camera: None,
        }
    }
}

quartz_engine::bridge! {
    components: { Mesh, Camera }
    plugins: { Render }
}

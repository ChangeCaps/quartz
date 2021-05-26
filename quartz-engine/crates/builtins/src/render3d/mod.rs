use quartz_engine_core::egui::Ui;
use quartz_engine_core::prelude::*;

mod quartz_engine {
    pub use quartz_engine_core as core;
}
use quartz_engine_core::render as quartz_render;

pub const MAX_POINT_LIGHTS: u32 = 64;
pub const MAX_DIR_LIGHTS: u32 = 8;

pub fn register_types(types: &mut Types) {
    types.register_plugin::<Render3dPlugin>();
    types.register_component::<PointLight3d>();
    types.register_component::<DirectionalLight3d>();
    types.register_component::<Camera3d>();
    types.register_component::<Mesh3d>();
    types.register_component::<ProceduralMesh3d>();
}

pub struct Render3dPlugin {
    pub pbr_pipeline: RenderPipeline,
    pub shadow_pipeline: RenderPipeline,
    pub main_camera: Option<Mat4>,
    pub shadow_map_sampler: Sampler,
    pub point_lights: UniformBuffer<PointLightRaw, MAX_POINT_LIGHTS>,
    pub directional_light_maps: Texture2dArray<format::Depth32Float>,
    pub directional_lights: UniformBuffer<DirectionalLightRaw, MAX_DIR_LIGHTS>,
}

impl Plugin for Render3dPlugin {
    fn init(ctx: PluginInitCtx) -> Self {
        let pbr_shader =
            Shader::from_glsl(include_str!("pbr.vert"), include_str!("pbr.frag")).unwrap();
        let pbr_pipeline = RenderPipeline::new(
            PipelineDescriptor::default_settings(pbr_shader),
            ctx.instance,
        )
        .unwrap();

        let shadow_shader =
            Shader::from_glsl(include_str!("shadow.vert"), include_str!("shadow.frag")).unwrap();
        let shadow_pipeline = RenderPipeline::new(
            PipelineDescriptor {
                targets: vec![],
                ..PipelineDescriptor::default_settings(shadow_shader)
            },
            ctx.instance,
        )
        .unwrap();

        let shadow_map_sampler = Sampler::new(&SamplerDescriptor::default(), ctx.instance);

        let directional_light_maps = Texture2dArray::new(
            &TextureDescriptor::default_settings(D2Array::new(2048, 2048, MAX_DIR_LIGHTS)),
            ctx.instance,
        );

        pbr_pipeline.bind("DirectionalShadowMaps", directional_light_maps.view());
        pbr_pipeline.bind("ShadowSampler", shadow_map_sampler.clone());

        Self {
            pbr_pipeline,
            shadow_pipeline,
            main_camera: None,
            shadow_map_sampler,
            point_lights: UniformBuffer::new(),
            directional_light_maps,
            directional_lights: UniformBuffer::new(),
        }
    }

    fn update(&mut self, ctx: PluginCtx) {
        self.editor_update(ctx);
    }

    fn editor_update(&mut self, _ctx: PluginCtx) {
        self.point_lights.clear();
        self.directional_lights.clear();
    }

    fn render(&mut self, ctx: PluginRenderCtx) {
        for node_id in ctx.tree.nodes() {
            let node = ctx.tree.get_node(node_id).unwrap();

            if let Some(light) = node.get_component::<DirectionalLight3d>() {
                if light.shadows {
                    let light_view_proj =
                        light.projection() * node.global_transform().matrix().inverse();

                    self.shadow_pipeline
                        .bind_uniform("Camera", &light_view_proj);
                    self.shadow_pipeline
                        .bind_uniform("CameraPos", &node.global_transform().translation);

                    let desc = RenderPassDescriptor::<format::TargetFormat> {
                        color_attachments: vec![],
                        depth_attachment: Some(DepthAttachment::default_settings(
                            self.directional_light_maps.layer_view(light.index),
                        )),
                        ..Default::default()
                    };

                    let mut pass = ctx.render_ctx.render_pass(&desc, &self.shadow_pipeline);

                    for node_id in ctx.tree.nodes() {
                        if let Some(node) = ctx.tree.get_node(node_id) {
                            if let Some(mesh) = node.get_component::<ProceduralMesh3d>() {
                                let model = node.global_transform().matrix();

                                self.shadow_pipeline.bind_uniform("Transform", &model);
                                pass.draw_mesh(&mesh.mesh);
                            }

                            if let Some(mesh) = node.get_component::<Mesh3d>() {
                                let model = node.global_transform().matrix();

                                self.shadow_pipeline.bind_uniform("Transform", &model);
                                pass.draw_mesh(&mesh.mesh);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Uniform)]
pub struct PointLightRaw {
    pub color: Color,
    pub data: Vec4,
}

#[derive(Reflect, Inspect)]
pub struct PointLight3d {
    pub color: Color,
    pub intensity: f32,
}

impl Default for PointLight3d {
    fn default() -> Self {
        Self {
            color: Color::rgb(1.0, 1.0, 1.0),
            intensity: 10.0,
        }
    }
}

impl Component for PointLight3d {
    type Plugins = Render3dPlugin;

    fn inspector_ui(&mut self, _: &mut Render3dPlugin, _: ComponentCtx, ui: &mut Ui) {
        self.inspect(ui);
    }

    fn update(&mut self, render: &mut Render3dPlugin, ctx: ComponentCtx) {
        self.editor_update(render, ctx);
    }

    fn editor_update(&mut self, render: &mut Render3dPlugin, ctx: ComponentCtx) {
        let light_raw = PointLightRaw {
            color: self.color,
            data: ctx.global_transform.translation.extend(self.intensity),
        };

        render
            .point_lights
            .push(light_raw)
            .expect("MAX_LIGHTS exceeded");
    }
}

#[derive(Uniform)]
pub struct DirectionalLightRaw {
    pub color: Color,
    pub pos: Vec3,
    pub data: Vec4,
    pub view_proj: Mat4,
    pub shadows: bool,
}

#[derive(Reflect, Inspect)]
pub struct DirectionalLight3d {
    #[inspect(ignore)]
    pub index: u32,
    pub color: Color,
    pub direction: Vec3,
    pub intensity: f32,
    pub shadows: bool,
}

impl DirectionalLight3d {
    pub fn projection(&self) -> Mat4 {
        let proj = OrthographicProjection {
            left: -60.0,
            right: 60.0,
            top: 60.0,
            bottom: -60.0,
            ..Default::default()
        };
        let rot = Quat::from_rotation_arc(Vec3::Z, self.direction.normalize());

        proj.matrix() * Mat4::from_quat(rot)
    }
}

impl Default for DirectionalLight3d {
    fn default() -> Self {
        Self {
            index: 0,
            color: Color::rgb(1.0, 1.0, 1.0),
            direction: Vec3::new(0.0, -1.0, 0.0),
            intensity: 10.0,
            shadows: true,
        }
    }
}

impl Component for DirectionalLight3d {
    type Plugins = Render3dPlugin;

    fn inspector_ui(&mut self, _: &mut Render3dPlugin, _ctx: ComponentCtx, ui: &mut Ui) {
        self.inspect(ui);
    }

    fn update(&mut self, render: &mut Render3dPlugin, ctx: ComponentCtx) {
        self.editor_update(render, ctx);
    }

    fn editor_update(&mut self, render: &mut Render3dPlugin, ctx: ComponentCtx) {
        let direction = ctx
            .global_transform
            .rotation
            .mul_vec3(self.direction.normalize());

        let proj = self.projection();
        let view_proj = proj * ctx.global_transform.matrix().inverse();

        let light_raw = DirectionalLightRaw {
            color: self.color,
            pos: ctx.global_transform.translation,
            data: direction.extend(self.intensity),
            view_proj,
            shadows: self.shadows,
        };

        self.index = render.directional_lights.len();

        render
            .directional_lights
            .push(light_raw)
            .expect("MAX_LIGHTS exceeded");
    }
}

#[derive(Default, Reflect, Inspect)]
pub struct Camera3d {
    pub projection: PerspectiveProjection,
}

impl Component for Camera3d {
    type Plugins = Render3dPlugin;

    fn inspector_ui(&mut self, _: &mut Render3dPlugin, _: ComponentCtx, ui: &mut Ui) {
        self.inspect(ui);
    }

    fn update(&mut self, plugins: &mut Render3dPlugin, ctx: ComponentCtx) {
        self.editor_update(plugins, ctx);
    }

    fn editor_update(&mut self, plugins: &mut Render3dPlugin, ctx: ComponentCtx) {
        let size = ctx.instance.target_size();
        self.projection.aspect = size.x / size.y;

        let view_proj = self.projection.matrix() * ctx.global_transform.matrix().inverse();
        plugins.main_camera = Some(view_proj);
    }
}

#[derive(Reflect, Inspect)]
pub struct Mesh3d {
    #[inspect(collapsing)]
    pub mesh: Mesh,
}

impl Default for Mesh3d {
    fn default() -> Self {
        let mut mesh = Mesh::new();

        mesh.add_attribute::<Vec3>("vertex_position");
        mesh.add_attribute::<Vec3>("vertex_normal");
        mesh.add_attribute::<Vec2>("vertex_uv");

        Self { mesh }
    }
}

impl Component for Mesh3d {
    type Plugins = Render3dPlugin;

    fn inspector_ui(&mut self, _: &mut Render3dPlugin, _: ComponentCtx, ui: &mut Ui) {
        self.inspect(ui);
    }

    fn render(&mut self, render: &mut Render3dPlugin, ctx: ComponentRenderCtx) {
        let camera = if ctx.viewport_camera.is_some() {
            ctx.viewport_camera
        } else {
            &render.main_camera
        };

        if let Some(view_proj) = camera {
            let model = ctx.global_transform.matrix();

            render.pbr_pipeline.bind_uniform("Transform", &model);
            render.pbr_pipeline.bind_uniform("Camera", view_proj);
            render
                .pbr_pipeline
                .bind_uniform("PointLights", &render.point_lights);
            render
                .pbr_pipeline
                .bind_uniform("DirectionalLights", &render.directional_lights);

            ctx.render_pass
                .with_pipeline(&render.pbr_pipeline)
                .draw_mesh(&self.mesh);
        }
    }

    fn viewport_pick_render(&mut self, _: &mut Render3dPlugin, ctx: ComponentPickCtx) {
        ctx.render_pass.draw_mesh(&self.mesh);
    }
}

#[derive(Reflect, Inspect)]
pub struct ProceduralMesh3d {
    #[reflect(ignore)]
    #[inspect(collapsing)]
    pub mesh: Mesh,
}

impl Default for ProceduralMesh3d {
    fn default() -> Self {
        let mut mesh = Mesh::new();

        mesh.add_attribute::<Vec3>("vertex_position");
        mesh.add_attribute::<Vec3>("vertex_normal");
        mesh.add_attribute::<Vec2>("vertex_uv");

        Self { mesh }
    }
}

impl Component for ProceduralMesh3d {
    type Plugins = Render3dPlugin;

    fn inspector_ui(&mut self, _: &mut Render3dPlugin, _ctx: ComponentCtx, ui: &mut Ui) {
        self.inspect(ui);
    }

    fn render(&mut self, render: &mut Render3dPlugin, ctx: ComponentRenderCtx) {
        let camera = if ctx.viewport_camera.is_some() {
            ctx.viewport_camera
        } else {
            &render.main_camera
        };

        if let Some(view_proj) = camera {
            let model = ctx.global_transform.matrix();

            render.pbr_pipeline.bind_uniform("Transform", &model);
            render.pbr_pipeline.bind_uniform("Camera", view_proj);
            render
                .pbr_pipeline
                .bind_uniform("PointLights", &render.point_lights);
            render
                .pbr_pipeline
                .bind_uniform("DirectionalLights", &render.directional_lights);

            ctx.render_pass
                .with_pipeline(&render.pbr_pipeline)
                .draw_mesh(&self.mesh);
        }
    }

    fn viewport_pick_render(&mut self, _: &mut Render3dPlugin, ctx: ComponentPickCtx) {
        ctx.render_pass.draw_mesh(&self.mesh);
    }
}

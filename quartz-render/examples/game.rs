use image::*;
use quartz_render::{framework::*, prelude::*};

pub struct Camera {
    pub projection: OrthographicProjection,
    pub transform: Transform,
}

impl Camera {
    pub fn matrix(&self) -> Mat4 {
        self.projection.matrix() * self.transform.matrix().inverse()
    }
}

struct GameState {
    main_image: RenderPipeline<format::Rgba8UnormSrgb>,
    post: RenderPipeline,
    texture: Texture2d,
    mesh: Mesh,
    mesh_transform: Transform,
    camera: Camera,
    rotation: f32,
}

impl GameState {
    pub fn new(render_resource: &RenderResource) -> Self {
        let shader = Shader::from_glsl(
            include_str!("main_image.vert"),
            include_str!("main_image.frag"),
        )
        .unwrap();
        let main_image = RenderPipeline::new(
            PipelineDescriptor::default_settings(shader),
            render_resource,
        )
        .unwrap();

        let shader =
            Shader::from_glsl(include_str!("post.vert"), include_str!("post.frag")).unwrap();
        let post = RenderPipeline::new(
            PipelineDescriptor::default_settings(shader),
            render_resource,
        )
        .unwrap();

        let texture = Texture::new(
            &TextureDescriptor::default_settings(D2::new(300, 200)),
            render_resource,
        );

        let sampler = Sampler::new(
            &SamplerDescriptor {
                filter: FilterMode::Nearest,
                ..SamplerDescriptor::default()
            },
            render_resource,
        );

        post.bind("test_sampler", sampler);
        post.bind("test_tex", texture.view());

        post.bind("Color", UniformBuffer::new(Vec4::new(0.5, 0.2, 0.6, 1.0)));

        let mut mesh = Mesh::new();
        mesh.set_attribute(
            "vertex_position",
            vec![
                Vec2::new(-4.0, -3.0),
                Vec2::new(4.0, -3.0),
                Vec2::new(0.0, 5.0),
            ],
        );
        mesh.set_indices(vec![0, 1, 2]);

        let mut camera_transform = Transform::from_translation(Vec3::new(-5.0, 5.0, 5.0));
        camera_transform.rotation = Quat::from_rotation_ypr(
            -std::f32::consts::PI / 4.0,
            -std::f32::consts::PI / 4.0,
            0.0,
        );

        Self {
            main_image,
            post,
            texture,
            mesh,
            mesh_transform: Transform::IDENTITY,
            camera: Camera {
                projection: OrthographicProjection::default(),
                transform: camera_transform,
            },
            rotation: 1.0,
        }
    }
}

impl State for GameState {
    fn update(&mut self, ctx: UpdateCtx) -> Trans {
        if ctx.keyboard.pressed(&Key::A) {
            self.rotation = -1.0;
        }

        if ctx.keyboard.pressed(&Key::D) {
            self.rotation = 1.0;
        }

        if ctx.keyboard.pressed(&Key::S) {
            self.texture.read(ctx.render_resource, |texture_data| {
                let mut data = Vec::new();

                for y in 0..self.texture.dimensions.height as usize {
                    for x in 0..self.texture.dimensions.width as usize {
                        let color = texture_data[x][y];

                        data.push((color.r * 255.0).round() as u8);
                        data.push((color.g * 255.0).round() as u8);
                        data.push((color.b * 255.0).round() as u8);
                        data.push((color.a * 255.0).round() as u8);
                    }
                }

                let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(
                    self.texture.dimensions.width,
                    self.texture.dimensions.height,
                    data,
                )
                .unwrap();

                buffer.save("image.png").unwrap();
            });
        }

        self.mesh_transform.rotation *= Quat::from_rotation_y(ctx.delta_time * self.rotation);

        Trans::None
    }

    fn render(&mut self, render_resource: &mut RenderResource) {
        let desc = RenderPassDescriptor {
            label: Some(String::from("Test pass")),
            color_attachments: vec![ColorAttachment {
                texture: TextureAttachment::Texture(self.texture.view()),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            ..Default::default()
        };

        render_resource
            .render(|ctx| {
                let mut mesh = Mesh::new();
                mesh.set_attribute(
                    "vertex_position",
                    vec![
                        Vec2::new(-4.0, -3.0),
                        Vec2::new(4.0, -3.0),
                        Vec2::new(0.0, 5.0),
                    ],
                );
                mesh.set_indices(vec![0, 1, 2]);

                self.main_image
                    .bind_uniform("Transform", self.mesh_transform.matrix());
                self.main_image
                    .bind_uniform("CameraProj", self.camera.matrix());
                ctx.render_pass(&desc, &self.main_image).draw_mesh(&mesh);

                ctx.render_pass(&RenderPassDescriptor::default(), &self.post)
                    .draw(0..6);
            })
            .unwrap();
    }
}

fn main() {
    App::new()
        .title("Pixelated Triangle")
        .run(GameState::new)
        .unwrap();
}

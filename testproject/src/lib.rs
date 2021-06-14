use noise::NoiseFn;
use quartz_engine::egui::*;
use quartz_engine::prelude::{Vec2, *};
use std::collections::HashMap;

pub fn detailed_noise(noise: impl NoiseFn<[f64; 2]>, p: Vec3, detail: u32) -> f32 {
    let mut height = 0.0;
    let mut disp = 0.0;
    let mut freq = 1.0;
    let mut amp = 0.5;

    for _ in 0..detail {
        let p = (p + Vec3::splat(disp)) * freq;
        height += noise.get([p.x as f64, p.z as f64]) as f32 * amp;

        freq *= 2.0;
        amp /= 2.0;
        disp += 50.0;
    }

    height
}

#[derive(Reflect, Inspect)]
pub struct Terrain {
    pub span: i32,
    #[inspect(collapsing)]
    pub chunks: HashMap<IVec2, NodeId>,
    pub size: f32,
    pub lods: Vec<u32>,
    pub dist_lod: f32,
    pub resolution: u32,
    #[reflect(reflect)]
    pub settings: TerrainSettings,
}

impl Default for Terrain {
    fn default() -> Self {
        Self {
            span: 3,
            size: 50.0,
            resolution: 100,
            lods: vec![1, 2, 5, 10],
            dist_lod: 25.0,
            chunks: HashMap::new(),
            settings: TerrainSettings::default(),
        }
    }
}

impl Component for Terrain {
    type Plugins = ();

    fn inspector_ui(&mut self, _: (), ctx: ComponentCtx, ui: &mut Ui) {
        if ui.button("spawn chunks").clicked() {
            for child in self.chunks.values() {
                ctx.tree.despawn(child);
            }

            self.chunks.clear();

            for x in -self.span..=self.span {
                for y in -self.span..=self.span {
                    let i_pos = IVec2::new(x, y);
                    let pos = Vec3::new(x as f32, 0.0, y as f32) * self.size;
                    let dist = pos.length();

                    let lod = ((dist / self.dist_lod).floor() as usize).min(self.lods.len() - 1);
                    let lod_mul = self.lods[lod];

                    let chunk = ctx.tree.spawn_child(ctx.node_id).unwrap();

                    let mut node = ctx.tree.get_node(chunk).unwrap();

                    node.transform.translation = pos;
                    node.name = "Chunk".to_string();

                    node.add_component(TerrainChunk {
                        size: self.size,
                        resolution: self.resolution / lod_mul,
                        settings: self.settings.clone(),
                        ..Default::default()
                    });

                    self.chunks.insert(i_pos, chunk);
                }
            }
        }

        self.inspect(ui);
    }
}

#[derive(Clone, Reflect, Inspect)]
pub struct TerrainSettings {
    /// Describes the scale of hills.
    pub scale: f32,
    pub height: f32,
    pub detail: u32,
    pub mountain_scale: f32,
    pub mountain_height: f32,
    pub mountain_detail: u32,
}

impl Default for TerrainSettings {
    fn default() -> Self {
        Self {
            scale: 1.0,
            height: 1.0,
            detail: 4,
            mountain_scale: 1.0,
            mountain_height: 1.0,
            mountain_detail: 4,
        }
    }
}

#[derive(Reflect, Inspect)]
pub struct TerrainChunk {
    pub size: f32,

    pub resolution: u32,

    #[reflect(reflect)]
    pub settings: TerrainSettings,
}

impl Default for TerrainChunk {
    fn default() -> Self {
        let mut mesh = Mesh::new();
        mesh.add_attribute::<Vec3>("vertex_position");
        mesh.add_attribute::<Vec3>("vertex_normal");
        mesh.add_attribute::<Vec2>("vertex_uv");
        mesh.add_attribute::<Color>("vertex_color");

        Self {
            size: 50.0,
            resolution: 100,
            settings: Default::default(),
        }
    }
}

impl TerrainChunk {
    pub fn generate(&mut self, ctx: ComponentCtx) {
        ctx.components.get_or_default(|mesh: &mut ProceduralMesh3d| {
            let mut positions = Vec::new(); 
            let mut indices = Vec::new();
            let mut colors = Vec::new();

            let noise = noise::OpenSimplex::new();

            for x in 0..self.resolution {
                for z in 0..self.resolution {
                    let mut pos = Vec3::new(x as f32, 0.0, z as f32);
                    pos /= self.resolution as f32 - 1.0;
                    pos *= self.size;
                    pos -= Vec3::splat(self.size as f32 / 2.0);
                    pos += ctx.transform.translation;

                    let mountains = 1.0
                        - detailed_noise(
                            &noise,
                            pos * self.settings.mountain_scale * 0.1,
                            self.settings.mountain_detail,
                        )
                        .abs();

                    let hills = detailed_noise(
                        &noise,
                        pos * self.settings.scale * 0.1 + Vec3::splat(-100.0),
                        self.settings.detail,
                    );

                    let mountain_mask = hills.max(0.0).powi(2);

                    let height = hills * self.settings.height
                        + mountain_mask * mountains * self.settings.mountain_height;

                    pos -= ctx.transform.translation;
                    pos.y = height;

                    let color = Color::rgb(0.1, 0.7, 0.2)
                        .lerp(Color::rgb(0.3, 0.3, 0.3), (mountain_mask * 20.0).min(1.0));

                    positions.push(pos);
                    colors.push(color);
                }
            }

            for x in 0..self.resolution - 1 {
                for z in 0..self.resolution - 1 {
                    let index = z * self.resolution + x;

                    indices.push(index);
                    indices.push(index + 1);
                    indices.push(index + self.resolution);
                    indices.push(index + self.resolution + 1);
                    indices.push(index + self.resolution);
                    indices.push(index + 1);
                }
            }

            let mut normals = vec![Vec3::ZERO; positions.len()];

            for i in 0..indices.len() / 3 {
                let i0 = indices[i * 3 + 0] as usize;
                let i1 = indices[i * 3 + 1] as usize;
                let i2 = indices[i * 3 + 2] as usize;

                let p0 = positions[i0];
                let p1 = positions[i1];
                let p2 = positions[i2];

                let normal = (p1 - p0).cross(p2 - p0).normalize();

                normals[i0] += normal;
                normals[i1] += normal;
                normals[i2] += normal;
            }

            for normal in &mut normals {
                *normal = normal.normalize();
            }

            mesh.mesh.set_attribute("vertex_position", positions);
            mesh.mesh.set_attribute("vertex_normal", normals);
            mesh.mesh.set_attribute("vertex_color", colors);
            mesh.mesh.set_indices(indices);
        });
    }
}

impl Component for TerrainChunk {
    type Plugins = Render3dPlugin;

    fn inspector_ui(&mut self, _: &mut Render3dPlugin, ctx: ComponentCtx, ui: &mut Ui) {
        if ui.button("Generate Mesh").clicked() || self.inspect(ui) {
            self.generate(ctx);
        }
    }

    fn start(&mut self, _: &mut Render3dPlugin, ctx: ComponentCtx) {
        self.generate(ctx);
    }

    fn editor_start(&mut self, _: &mut Render3dPlugin, ctx: ComponentCtx) {
        self.generate(ctx);
    }
}

fn register_types(types: &mut Types) {
    types.register_component::<TerrainChunk>();
    types.register_component::<Terrain>();
}

quartz_engine::register_types!(register_types);

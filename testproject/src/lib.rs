use noise::NoiseFn;
use quartz_engine::egui::*;
use quartz_engine::prelude::*;

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
    pub size: u32,
    pub scale: f32,
    pub height: f32,
    pub detail: u32,
    pub mountain_scale: f32,
    pub mountain_height: f32,
    pub mountain_detail: u32,
}

impl Default for Terrain {
    fn default() -> Terrain {
        Self {
            size: 10,
            scale: 0.2,
            height: 1.0,
            detail: 4,
            mountain_scale: 1.0,
            mountain_height: 1.0,
            mountain_detail: 4,
        }
    }
}

impl Terrain {
    pub fn generate(&self, ctx: ComponentCtx) {
        let mesh = ctx.components.get_component_mut::<ProceduralMesh3d>();

        if let Some(mut mesh) = mesh {
            println!("generating");

            let mut positions = Vec::new();
            let mut indices = Vec::new();
            let mut colors = Vec::new();

            let noise = noise::OpenSimplex::new();

            for x in 0..self.size {
                for z in 0..self.size {
                    let mut pos = Vec3::new(x as f32, 0.0, z as f32);
                    pos -= Vec3::splat(self.size as f32 / 2.0);
                    pos *= self.scale;

                    let mountains = 1.0
                        - detailed_noise(&noise, pos * self.mountain_scale * 0.1, self.mountain_detail)
                            .abs();

                    let hills = detailed_noise(&noise, pos * 0.1 + Vec3::splat(-100.0), self.detail);

                    let mountain_mask = hills.max(0.0).powi(2);

                    let height =
                        hills * self.height + mountain_mask * mountains * self.mountain_height;

                    pos.y = height;

                    let color = if mountain_mask > 0.0 {
                        Color::rgb(0.3, 0.3, 0.3)
                    } else {
                        Color::rgb(0.1, 0.7, 0.2)
                    };

                    positions.push(pos);
                    colors.push(color);
                }
            }

            for x in 0..self.size - 1 {
                for z in 0..self.size - 1 {
                    let index = z * self.size + x;

                    indices.push(index);
                    indices.push(index + 1);
                    indices.push(index + self.size);
                    indices.push(index + self.size + 1);
                    indices.push(index + self.size);
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
        }
    }
}

impl Component for Terrain {
    type Plugins = ();

    fn inspector_ui(&mut self, _: (), ctx: ComponentCtx, ui: &mut Ui) {
        if ui.button("Generate Mesh").clicked() || self.inspect(ui) {
            self.generate(ctx);
        }
    }

    fn start(&mut self, _: (), ctx: ComponentCtx) {
        self.generate(ctx);
    }

    fn editor_start(&mut self, _: (), ctx: ComponentCtx) {
        self.generate(ctx);
    }
}

fn register_types(types: &mut Types) {
    types.register_component::<Terrain>();
}

quartz_engine::register_types!(register_types);

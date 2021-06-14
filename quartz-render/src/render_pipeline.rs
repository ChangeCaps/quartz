use crate::prelude::*;
use spirv_reflect::types::{
    descriptor::ReflectDescriptorType, image::ReflectFormat, variable::ReflectDimension,
};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use wgpu::{
    util::DeviceExt, BlendState, ColorWrite, CompareFunction, DepthBiasState, FrontFace,
    PolygonMode, PrimitiveState, PrimitiveTopology, StencilState,
};

/// Descriptor binding location.
pub struct Location {
    pub set: usize,
    pub binding: usize,
}

/// The type of a binding in a shader.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingType {
    Texture {
        view_dimension: wgpu::TextureViewDimension,
        multisampled: bool,
    },
    Sampler,
    Buffer,
    UniformBuffer {
        size: u64,
    },
}

/// Shader side descriptor set binding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindGroupEntry {
    pub ident: String,
    pub binding: u32,
    pub ty: BindingType,
}

/// Shader side descriptor set.
#[derive(Clone, Debug)]
pub struct BindGroup {
    /// Entries.
    pub bindings: HashMap<u32, BindGroupEntry>,
    /// Wgpu internal [`wgpu::BindGroupLayout`].
    pub layout: Option<Arc<wgpu::BindGroupLayout>>,
}

impl BindGroup {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            layout: None,
        }
    }
}

/// Shader side input descriptor.
#[derive(Clone, Debug)]
pub struct VertexAttributeLayout {
    /// Buffer offset.
    pub offset: u64,
    /// Location in shader.
    pub shader_location: u32,
    /// Vertex format.
    pub format: wgpu::VertexFormat,
}

/// Layout of the shader side of a [`RenderPipeline`].
#[derive(Clone, Debug)]
pub struct PipelineLayout {
    /// Bind groups.
    pub bind_groups: Vec<BindGroup>,
    /// Vertex attributes.
    pub vertex_attributes: HashMap<String, VertexAttributeLayout>,
}

pub struct ColorState<F: TextureFormat> {
    pub format: F,
}

impl<F: TextureFormat> ColorState<F> {
    pub(crate) fn to_target_color_state(&self) -> wgpu::ColorTargetState {
        wgpu::ColorTargetState {
            format: self.format.format(),
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrite::ALL,
        }
    }
}

pub trait ColorTargetState {
    type State;

    fn to_color_state(state: &Self::State) -> Vec<wgpu::ColorTargetState>;
}

impl ColorTargetState for () {
    type State = ();

    fn to_color_state(state: &Self::State) -> Vec<wgpu::ColorTargetState> {
        vec![]
    }
}

macro_rules! impl_color_target_state {
    ($($ident:ident),*) => {
        #[allow(unused_parens)]
        impl<$($ident),*> ColorTargetState for ($($ident),*)
            where $($ident: TextureFormat),*
        {
            type State = ($(ColorState<$ident>),*);

            #[allow(non_snake_case)]
            fn to_color_state(($($ident),*): &Self::State) -> Vec<wgpu::ColorTargetState> {
                vec![$($ident.to_target_color_state()),*]
            }
        }
    };
}

impl_color_target_state!(A);
impl_color_target_state!(A, B);
impl_color_target_state!(A, B, C);
impl_color_target_state!(A, B, C, D);
impl_color_target_state!(A, B, C, D, E);
impl_color_target_state!(A, B, C, D, E, F);
impl_color_target_state!(A, B, C, D, E, F, G);
impl_color_target_state!(A, B, C, D, E, F, G, H);

pub struct DepthState<F: TextureFormat> {
    pub format: F,
}

pub trait DepthStencilState {
    type State;

    fn to_depth_state(state: &Self::State) -> Option<wgpu::DepthStencilState>;
}

impl DepthStencilState for () {
    type State = ();

    fn to_depth_state(state: &Self::State) -> Option<wgpu::DepthStencilState> {
        None
    }
}

impl<F: TextureFormat> DepthStencilState for F {
    type State = DepthState<F>;

    fn to_depth_state(state: &Self::State) -> Option<wgpu::DepthStencilState> {
        Some(wgpu::DepthStencilState {
            format: state.format.format(),
            depth_write_enabled: true,
            depth_compare: CompareFunction::LessEqual,
            bias: DepthBiasState::default(),
            stencil: StencilState::default(),
        })
    }
}

/// Used to create a [`RenderPipeline`].
pub struct PipelineDescriptor<T: ColorTargetState, D: DepthStencilState> {
    /// The shader for the pipeline
    pub shader: Shader,
    pub targets: T::State,
    pub depth_stencil: D::State,
    pub primitive: PrimitiveState,
}

impl<T: ColorTargetState, D: DepthStencilState> PipelineDescriptor<T, D> {
    pub fn default_settings(
        shader: Shader,
        depth_stencil: impl Into<wgpu::DepthStencilState>,
    ) -> Self
    where
        T::State: Default,
        D::State: Default,
    {
        Self {
            shader,
            targets: Default::default(),
            depth_stencil: Default::default(),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
        }
    }
}

/// Needed for rendering a renderpass.
///
/// Keeps track of [`Bindings`].
pub struct RenderPipeline<
    T: ColorTargetState = format::TargetFormat,
    D: DepthStencilState = format::Depth32Float,
> {
    pub(crate) descriptor: PipelineDescriptor<T, D>,
    pub(crate) layout: PipelineLayout,
    pub(crate) pipeline: Arc<wgpu::RenderPipeline>,
}

impl<T: ColorTargetState, D: DepthStencilState> RenderPipeline<T, D> {
    /// Creates a pipeline.
    pub fn new(
        descriptor: PipelineDescriptor<T, D>,
        instance: &Instance,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let limis = instance.device.limits();

        let mut bind_groups = (0..limis.max_bind_groups as usize)
            .into_iter()
            .map(|_| BindGroup::new())
            .collect::<Vec<_>>();

        let vs_reflection =
            spirv_reflect::ShaderModule::load_u8_data(descriptor.shader.vs_spirv.as_binary_u8())?;
        let fs_reflection =
            spirv_reflect::ShaderModule::load_u8_data(descriptor.shader.fs_spirv.as_binary_u8())?;

        for binding in vs_reflection
            .enumerate_descriptor_bindings(None)?
            .into_iter()
            .chain(fs_reflection.enumerate_descriptor_bindings(None)?)
        {
            let entry = BindGroupEntry {
                ident: match binding.descriptor_type {
                    ReflectDescriptorType::UniformBuffer => {
                        binding.type_description.unwrap().type_name
                    }
                    _ => binding.name,
                },
                binding: binding.binding,
                ty: match binding.descriptor_type {
                    ReflectDescriptorType::StorageBuffer => BindingType::Buffer,
                    ReflectDescriptorType::UniformBuffer => BindingType::UniformBuffer {
                        size: binding.block.size as u64,
                    },
                    ReflectDescriptorType::SampledImage => BindingType::Texture {
                        view_dimension: match binding.image.dim {
                            ReflectDimension::Type1d => wgpu::TextureViewDimension::D1,
                            ReflectDimension::Type2d => {
                                if binding.image.arrayed > 0 {
                                    wgpu::TextureViewDimension::D2Array
                                } else {
                                    wgpu::TextureViewDimension::D2
                                }
                            }
                            ReflectDimension::Type3d => wgpu::TextureViewDimension::D3,
                            ReflectDimension::Cube => wgpu::TextureViewDimension::Cube,
                            _ => panic!("Texture type unsupported"),
                        },
                        multisampled: binding.image.ms > 1,
                    },
                    ReflectDescriptorType::Sampler => BindingType::Sampler,
                    _ => return Err("Descriptor type unsupported".into()),
                },
            };

            if let Some(existing_entry) = bind_groups[binding.set as usize]
                .bindings
                .get(&binding.binding)
            {
                if *existing_entry != entry {
                    return Err("Overlapping bindings".into());
                }
            } else {
                bind_groups[binding.set as usize]
                    .bindings
                    .insert(binding.binding, entry);
            }
        }

        let vertex_attributes = vs_reflection
            .enumerate_input_variables(None)?
            .into_iter()
            .filter(|input| {
                !input
                    .decoration_flags
                    .contains(spirv_reflect::types::variable::ReflectDecorationFlags::BUILT_IN)
            })
            .map(|input| {
                let layout = VertexAttributeLayout {
                    offset: 0,
                    shader_location: input.location,
                    format: match input.format {
                        ReflectFormat::R32_SFLOAT => wgpu::VertexFormat::Float32,
                        ReflectFormat::R32G32_SFLOAT => wgpu::VertexFormat::Float32x2,
                        ReflectFormat::R32G32B32_SFLOAT => wgpu::VertexFormat::Float32x3,
                        ReflectFormat::R32G32B32A32_SFLOAT => wgpu::VertexFormat::Float32x4,
                        ReflectFormat::R32_SINT => wgpu::VertexFormat::Sint32,
                        ReflectFormat::R32G32_SINT => wgpu::VertexFormat::Sint32x2,
                        ReflectFormat::R32G32B32_SINT => wgpu::VertexFormat::Sint32x3,
                        ReflectFormat::R32G32B32A32_SINT => wgpu::VertexFormat::Sint32x4,
                        _ => panic!("Unsupported input format {:?}", input),
                    },
                };

                (input.name, layout)
            })
            .collect::<HashMap<_, _>>();

        let mut attributes = vertex_attributes
            .iter()
            .map(|(_, attr)| {
                vec![wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: attr.shader_location,
                    format: attr.format,
                }]
            })
            .collect::<Vec<_>>();

        attributes.sort_by(|a, b| a[0].shader_location.cmp(&b[0].shader_location));

        let buffers = attributes
            .iter()
            .map(|attr| wgpu::VertexBufferLayout {
                array_stride: attr[0].format.size(),
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &attr,
            })
            .collect::<Vec<_>>();

        let (vs_module, fs_module) = descriptor.shader.to_modules(instance);

        let layouts = bind_groups
            .iter_mut()
            .filter_map(|bind_group| {
                let entries = bind_group
                    .bindings
                    .iter()
                    .map(|(_binding, entry)| wgpu::BindGroupLayoutEntry {
                        binding: entry.binding,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: match entry.ty {
                            BindingType::Sampler => wgpu::BindingType::Sampler {
                                filtering: true,
                                comparison: false,
                            },
                            BindingType::Texture {
                                view_dimension,
                                multisampled,
                            } => wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension,
                                multisampled,
                            },
                            BindingType::UniformBuffer { .. } => wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            _ => panic!(),
                        },
                        count: None,
                    })
                    .collect::<Vec<_>>();

                if entries.len() > 0 {
                    let layout = instance.device.create_bind_group_layout(
                        &wgpu::BindGroupLayoutDescriptor {
                            label: Some("Bind Group Layout"),
                            entries: &entries,
                        },
                    );

                    bind_group.layout = Some(Arc::new(layout));

                    Some(&**bind_group.layout.as_ref().unwrap())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let layout = instance
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &layouts,
                push_constant_ranges: &[],
            });

        let targets = T::to_color_state(&descriptor.targets);
        let depth_stencil = D::to_depth_state(&descriptor.depth_stencil);

        let pipeline = instance
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &vs_module,
                    entry_point: "main",
                    buffers: &buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fs_module,
                    entry_point: "main",
                    targets: &targets,
                }),
                primitive: descriptor.primitive.clone(),
                depth_stencil,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            });

        let layout = PipelineLayout {
            bind_groups,
            vertex_attributes,
        };

        Ok(Self {
            descriptor,
            layout,
            pipeline: Arc::new(pipeline),
        })
    }
}

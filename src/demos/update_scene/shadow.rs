use super::object::Tetrahedron;
use crate::shared::ready_paint::{get_res, Ready};
use crate::shared::ready_paint::{return_res, Gfx, HashTypeId2Data, Pass};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::{util::DeviceExt, PipelineCompilationOptions};
use wgpu::{DepthStencilState, VertexState};

const SHADOW_MAP_SIZE: u32 = 1024;
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct LightUniform {
    view_proj: [[f32; 4]; 4],
}

#[derive(Default)]
pub struct ShadowScene {
    pub detpth_texture_pipeline: Option<wgpu::RenderPipeline>,
    pub second_depth_pipeline: Option<wgpu::RenderPipeline>,
    pub all_bind_group: Option<wgpu::BindGroup>,
    pub first_shadow_view: Option<wgpu::TextureView>,
    pub light_camera_bind_group: Option<wgpu::BindGroup>,
    pub depth_view: Option<wgpu::TextureView>,
}
impl Ready for ShadowScene {
    fn ready(&mut self, data: &mut HashTypeId2Data, gfx: &Gfx) {
        let light_position = Vec3::new(1.5, 2.0, 1.5);
        let light_target = Vec3::new(0.0, 0.0, 0.0);
        let light_up = Vec3::new(0.0, 1.0, 0.0);
        let light_view = Mat4::look_at_rh(light_position, light_target, light_up);
        let light_proj = Mat4::orthographic_rh(
            -2.0, 2.0, // 缩小范围以提高精度
            -2.0, 2.0, 0.1, 8.0,
        );
        let light_view_proj = light_proj * light_view;
        let light_uniform = LightUniform {
            view_proj: light_view_proj.to_cols_array_2d(),
        };
        let light_uniform_buffer =
            gfx.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("light_uniform_buffer"),
                    contents: bytemuck::cast_slice(&[light_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
        let light_camera_bind_group_layout =
            gfx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Shadow Camera Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });
        let light_camera_bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light Camera Bind Group"),
            layout: &light_camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_uniform_buffer.as_entire_binding(),
            }],
        });
        let ready_store_depth_texture = gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Texture"),
            size: wgpu::Extent3d {
                width: SHADOW_MAP_SIZE,
                height: SHADOW_MAP_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let first_shadow_view =
            ready_store_depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 创建阴影采样器
        let shadow_sampler = gfx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            // compare: Some(wgpu::CompareFunction::LessEqual),
            compare: None,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        // 创建第一个管线布局
        let first_depth_pipeline_layout =
            gfx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("first depth pipeline layout"),
                    bind_group_layouts: &[&light_camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        // 创建深度着色器
        let depth_shader = gfx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("depth_shader"),
                source: wgpu::ShaderSource::Wgsl(SHADOW_DEPTH_SHADER.into()),
            });

        // 创建深度贴图管线
        let first_detpth_texture_pipeline =
            gfx.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("first detpth pipeline"),
                    layout: Some(&first_depth_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &depth_shader,
                        entry_point: Some("vs_main"),
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 0,
                            }],
                        }],
                        compilation_options: PipelineCompilationOptions::default(),
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        front_face: wgpu::FrontFace::Cw,
                        cull_mode: Some(wgpu::Face::Back),
                        ..Default::default()
                    },
                    depth_stencil: Some(DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        stencil: Default::default(),
                        bias: wgpu::DepthBiasState {
                            constant: 2,
                            slope_scale: 2.0,
                            clamp: 0.0,
                        },
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    fragment: None,
                    multiview: None,
                    cache: None,
                });
        let all_bindgroup_layout =
            gfx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("light_bind_group_layout"),
                    entries: &[
                        // 阴影贴图纹理
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Depth,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        // 阴影采样器
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            // ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                            count: None,
                        },
                    ],
                });

        // 创建光照和阴影的绑定组
        let all_bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light_uniform_bind_group"),
            layout: &all_bindgroup_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&first_shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
            ],
        });

        // 创建第二个管线布局
        let second_depth_pipeline_layout =
            gfx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("second depth pipeline layout"),
                    bind_group_layouts: &[&all_bindgroup_layout],
                    push_constant_ranges: &[],
                });

        // 创建第二个着色器
        let second_shader = gfx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("second depth shader"),
                source: wgpu::ShaderSource::Wgsl(SECOND_DEPTH_SHADER.into()),
            });

        // 创建第二个管线
        let secone_depth_pipeline =
            gfx.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Second shadow pipeline"),
                    layout: Some(&second_depth_pipeline_layout),
                    vertex: VertexState {
                        module: &second_shader,
                        entry_point: Some("vs_main"),
                        compilation_options: PipelineCompilationOptions::default(),
                        buffers: &[],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        ..Default::default()
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        stencil: Default::default(),
                        bias: Default::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                    cache: None,
                    fragment: Some(wgpu::FragmentState {
                        module: &second_shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Bgra8Unorm,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: PipelineCompilationOptions::default(),
                    }),
                });

        // 主渲染通道的深度纹理
        let depth_texture = gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Main Render Depth Texture"),
            size: wgpu::Extent3d {
                width: 300,
                height: 300,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        return_res(
            data,
            Self {
                detpth_texture_pipeline: Some(first_detpth_texture_pipeline),
                second_depth_pipeline: Some(secone_depth_pipeline),
                all_bind_group: Some(all_bind_group),
                first_shadow_view: Some(first_shadow_view),
                light_camera_bind_group: Some(light_camera_bind_group),
                depth_view: Some(depth_view),
            },
        );
    }
}

impl ShadowScene {
    // 第一个通道：渲染深度贴图
    pub fn first_catch_depth_pass<'a>(
        data: &mut HashTypeId2Data,
        mut render_pass: wgpu::RenderPass<'a>,
    ) -> wgpu::RenderPass<'a> {
        let shadow_scene = get_res::<ShadowScene>(data);
        let tetra = get_res::<Tetrahedron>(data);
        render_pass.set_pipeline(shadow_scene.detpth_texture_pipeline.as_ref().unwrap());
        render_pass.set_bind_group(
            0,
            shadow_scene.light_camera_bind_group.as_ref().unwrap(),
            &[],
        );
        render_pass.set_vertex_buffer(0, tetra.object_buffer.as_ref().unwrap().slice(..));
        render_pass.set_index_buffer(
            tetra
                .object_triangle_list_index_buffer
                .as_ref()
                .unwrap()
                .slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(0..12, 0, 0..1);

        render_pass
    }
}

impl<'a> Pass<'a> for ShadowScene {
    fn pass(
        data: &mut HashTypeId2Data,
        mut render_pass: wgpu::RenderPass<'a>,
    ) -> wgpu::RenderPass<'a> {
        // 第二个pass
        let shadow_scene = get_res::<ShadowScene>(data);
        render_pass.set_pipeline(shadow_scene.second_depth_pipeline.as_ref().unwrap());
        render_pass.set_bind_group(0, shadow_scene.all_bind_group.as_ref().unwrap(), &[]);
        render_pass.draw(0..3, 0..1);
        render_pass
    }
}

// 第一个着色器：渲染深度贴图
const SHADOW_DEPTH_SHADER: &str = r#"
struct LightUniform {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> light: LightUniform;
struct VertexInput {
    @location(0) position: vec3<f32>,
}
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = light.view_proj * vec4<f32>(in.position, 1.0);
    return out;
}
"#;

const SECOND_DEPTH_SHADER: &str = r#"
@group(0) @binding(0)
var shadow_texture: texture_depth_2d;
@group(0) @binding(1)
// var shadow_sampler: sampler_comparison;
var shadow_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}
@vertex
fn vs_main(@builtin(vertex_index) vertex_idx: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(vertex_idx == 1u) * 4.0 - 1.0;
    let y = f32(vertex_idx == 2u) * 4.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.tex_coords = vec2<f32>(x, -y) * 0.5 + 0.5;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
   let depth = textureSample(shadow_texture, shadow_sampler, in.tex_coords);
    

// 直接显示深度值，添加一些缩放以便观察
    let adjusted_depth = 1.0 - depth; // 反转深度值，使近处物体更亮
    return vec4<f32>(vec3<f32>(adjusted_depth), 1.0);
}
"#;

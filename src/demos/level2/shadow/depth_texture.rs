use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::{util::DeviceExt, CompareFunction, PipelineCompilationOptions};

use crate::{
    console_log,
    shared::ready_paint::{get_res, return_res, Pass, Ready},
    utils::Vertex,
};

use super::{plane::ShadowPlane, ShadowScene};

#[derive(Default)]
pub struct DepthTexture {
    pub shadow_pipeline: Option<wgpu::RenderPipeline>,
    pub shadow_texture: Option<wgpu::Texture>,
    pub shadow_sampler: Option<wgpu::Sampler>,
    pub shadow_view: Option<wgpu::TextureView>,
    pub light_storage_buffer: Option<wgpu::Buffer>,
    pub light_bind_group: Option<wgpu::BindGroup>,
}

impl Ready for DepthTexture {
    fn ready(
        &mut self,
        data: &mut crate::shared::ready_paint::HashTypeId2Data,
        gfx: &crate::shared::ready_paint::Gfx,
    ) {
        let depth_texture = gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d {
                width: 1024,
                height: 1024,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let shadow_sampler = gfx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("depth sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        let shadow_texture = gfx.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 1024,
                height: 1024,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: None,
            view_formats: &[],
        });
        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let light_position = Vec3::new(-3., 5., 3.);
        let light_target = Vec3::new(0.0, 0.0, 0.0);
        let light_up = Vec3::new(0.0, 1.0, 0.0);
        let light_view = Mat4::look_at_rh(light_position, light_target, light_up);
        let light_proj = Mat4::orthographic_rh(-3.0, 3.0, -3.0, 3.0, 0.1, 10.0);
        let light_view_proj = light_proj * light_view;
        let light_storage = LightStorage {
            view_proj: light_view_proj.to_cols_array_2d(),
        };
        let light_storage_buffer =
            gfx.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("light_storage_buffer"),
                    contents: bytemuck::cast_slice(&[light_storage]),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                });
        let light_bind_group_layout =
            gfx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("light_bind_group_layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX, // 注意：修改为顶点着色器可见
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                std::mem::size_of::<LightStorage>() as _,
                            ),
                        },
                        count: None,
                    }],
                });
        let light_bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_storage_buffer.as_entire_binding(),
            }],
            label: Some("light_bind_group"),
        });
        // wgsl
        let depth_shader = gfx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("depth shader"),
                source: wgpu::ShaderSource::Wgsl(DEPTH_SHADER.into()),
            });

        let light_uniform_size = size_of::<LightStorage>() as wgpu::BufferAddress;
        // 深度管线
        let depth_pipeline =
            gfx.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("depth pipeline"),
                    layout: Some(&gfx.device.create_pipeline_layout(
                        &wgpu::PipelineLayoutDescriptor {
                            label: Some("depth_pipeline_layout"),
                            bind_group_layouts: &[&light_bind_group_layout],
                            push_constant_ranges: &[],
                        },
                    )),
                    vertex: wgpu::VertexState {
                        module: &depth_shader,
                        entry_point: Some("vs_main"),
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x3,
                            }],
                        }],
                        compilation_options: PipelineCompilationOptions::default(),
                    },
                    fragment: None,
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Cw,
                        cull_mode: Some(wgpu::Face::Back),
                        // cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                        ..Default::default()
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState {
                            constant: 2, // corresponds to bilinear filtering
                            slope_scale: 2.0,
                            clamp: 0.0,
                        },
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                    cache: None,
                });

        return_res(
            data,
            Self {
                shadow_texture: Some(depth_texture),
                shadow_view: Some(shadow_view),
                shadow_sampler: Some(shadow_sampler),
                shadow_pipeline: Some(depth_pipeline),
                light_storage_buffer: Some(light_storage_buffer),
                light_bind_group: Some(light_bind_group),
            },
        );
    }
}

impl<'a> Pass<'a> for DepthTexture {
    fn pass(
        data: &mut crate::shared::ready_paint::HashTypeId2Data,
        mut render_pass: wgpu::RenderPass<'a>,
    ) -> wgpu::RenderPass<'a> {
        let depth_thing = get_res::<Self>(data);
        let plane = get_res::<ShadowPlane>(data);
        let scene = get_res::<ShadowScene>(data);
        let depth_pipeline = depth_thing.shadow_pipeline.as_ref().unwrap();

        // 对于depth的整合的所有vertex和index
        let for_common_vertex_buffer = scene.for_common_vertex_buffer.as_ref().unwrap();
        let for_common_index_buffer = scene.for_common_index_buffer.as_ref().unwrap();
        let cube_indics_count = scene.cube_indices_count.as_ref().unwrap();
        let plane_indics_count = plane.index_count;

        let light_bind_group = depth_thing.light_bind_group.as_ref().unwrap();
        let cube_vertex_buffer = scene.cube_buffer.as_ref().unwrap();
        let cube_index_buffer = scene.cube_index_buffer.as_ref().unwrap();
        let cube_indics_count = scene.cube_indices_count.unwrap();

        render_pass.set_pipeline(depth_pipeline);
        render_pass.set_bind_group(0, light_bind_group, &[]); // 设置绑定组

        // 全部物体的深度测试
        // render_pass.set_vertex_buffer(
        //     0,
        //     scene.for_common_vertex_buffer.as_ref().unwrap().slice(..),
        // );
        // render_pass.set_index_buffer(
        //     scene.for_common_index_buffer.as_ref().unwrap().slice(..),
        //     wgpu::IndexFormat::Uint16,
        // );
        // render_pass.draw_indexed(0..(cube_indics_count + plane_indics_count), 0, 0..1);

        // 只有cube的深度测试
        render_pass.set_vertex_buffer(0, cube_vertex_buffer.slice(..));
        render_pass.set_index_buffer(cube_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..cube_indics_count, 0, 0..1);

        render_pass
    }
}

const DEPTH_SHADER: &str = r#"
struct LightStorage {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<storage, read> light: LightStorage;

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

#[repr(C)]
#[derive(Debug, Pod, Zeroable, Copy, Clone)]
pub struct LightStorage {
    pub view_proj: [[f32; 4]; 4],
}

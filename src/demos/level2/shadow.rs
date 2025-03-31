// shadow scene

use depth_texture::{DepthTexture, LightStorage};
use plane::ShadowPlane;
use wgpu::{util::DeviceExt, PipelineCompilationOptions};
use world::World;

use crate::{
    console_log,
    shared::ready_paint::{
        get_res, get_res_mut, refs_muts, return_res, Mut, Pass, Ready, Ref, Update,
    },
    utils::CubeMesh,
};
#[derive(Default)]
pub struct ShadowScene {
    pub cube_buffer: Option<wgpu::Buffer>,
    pub cube_index_buffer: Option<wgpu::Buffer>,
    pub cube_pipeline: Option<wgpu::RenderPipeline>,
    pub for_common_vertex: Option<Vec<[f32; 3]>>,
    pub for_common_index: Option<Vec<u16>>,
    pub for_common_vertex_buffer: Option<wgpu::Buffer>,
    pub for_common_index_buffer: Option<wgpu::Buffer>,
    pub cube_indices_count: Option<u32>,
    pub main_depth_view: Option<wgpu::TextureView>,
    pub depth_bind_group: Option<wgpu::BindGroup>,
    pub angle: f32,
}
impl Ready for ShadowScene {
    fn ready(
        &mut self,
        data: &mut crate::shared::ready_paint::HashTypeId2Data,
        gfx: &crate::shared::ready_paint::Gfx,
    ) {
        println!("shadow scene ready");
        let shadow_scene = get_res::<DepthTexture>(data);
        let shadow_view = shadow_scene.shadow_view.as_ref().unwrap();
        let shadow_sampler = shadow_scene.shadow_sampler.as_ref().unwrap();

        let cube = CubeMesh::default();
        let cube_indices_count = cube.indices.len() as u32;
        let cube_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("cube buffer vertex"),
                contents: bytemuck::cast_slice(&cube.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let cube_index_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("cube buffer index"),
                contents: bytemuck::cast_slice(&cube.indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        let cube_shader = gfx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("depth_shader"),
                source: wgpu::ShaderSource::Wgsl(CUBE_SHADER.into()),
            });
        let depth_bind_group_layout =
            gfx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("depth bind group layout"),
                    entries: &[
                        // depth texture
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
                        // sampler
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                            count: None,
                        },
                        // light uniform
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(
                                    size_of::<LightStorage>() as wgpu::BufferAddress
                                ),
                            },
                            count: None,
                        },
                    ],
                });
        let light_storage_buffer = shadow_scene.light_storage_buffer.as_ref().unwrap();
        let depth_bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("depth bind group"),
            layout: &depth_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(
                        light_storage_buffer.as_entire_buffer_binding(),
                    ),
                },
            ],
        });
        let world = get_res::<World>(data);
        let world_uniforms_bind_group_layout = world.uniforms_bind_group_layout.as_ref().unwrap();
        let main_layout = gfx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("main pipeline layout"),
                bind_group_layouts: &[&depth_bind_group_layout, &world_uniforms_bind_group_layout],
                push_constant_ranges: &[],
            });

        let cube_pipeline = gfx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("cube pipeline"),
                layout: Some(&main_layout),
                vertex: wgpu::VertexState {
                    module: &cube_shader,
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
                fragment: Some(wgpu::FragmentState {
                    module: &cube_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    // cull_mode: Some(wgpu::Face::Back),
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });
        let plane = get_res::<ShadowPlane>(data);
        let plane_vertex = plane
            .vertices
            .iter()
            .map(|v| [v[0], v[1], v[2]])
            .collect::<Vec<[f32; 3]>>();
        let for_depth_verticies = cube
            .vertices
            .iter()
            .map(|v| v.position)
            .collect::<Vec<[f32; 3]>>();

        let for_common_vertex = for_depth_verticies
            .iter()
            .cloned()
            .chain(plane_vertex.iter().cloned())
            .collect::<Vec<[f32; 3]>>();

        // 给阴影场景的顶点数据创建缓冲区
        let for_common_vertex_buffer =
            gfx.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("for_common_vertex_buffer"),
                    contents: bytemuck::cast_slice(&for_common_vertex),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        // 1. 获取立方体顶点数量
        let cube_vertex_count = cube.vertices.len() as u16;

        // 2. 应用偏移到平面索引
        let plane_indices_with_offset = plane
            .indices
            .iter()
            .map(|idx| idx + cube_vertex_count) // 加上立方体顶点数作为偏移
            .collect::<Vec<u16>>();

        // 3. 合并索引
        let for_common_index = cube
            .indices
            .iter()
            .cloned()
            .chain(plane_indices_with_offset.iter().cloned())
            .collect::<Vec<u16>>();
        let for_common_index_buffer =
            gfx.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("for_common_index_buffer"),
                    contents: bytemuck::cast_slice(&for_common_index),
                    usage: wgpu::BufferUsages::INDEX,
                });

        console_log!("for_common_vertex: {:?}", for_common_vertex);
        console_log!("for_common_index: {:?}", for_common_index);
        let main_depth_texture = gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Main Scene Depth Texture"),
            size: wgpu::Extent3d {
                width: 800,
                height: 600,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let main_depth_view =
            main_depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        return_res(
            data,
            Self {
                cube_buffer: Some(cube_buffer),
                cube_index_buffer: Some(cube_index_buffer),
                cube_pipeline: Some(cube_pipeline),
                for_common_vertex: Some(for_common_vertex),
                for_common_index: Some(for_common_index),
                for_common_vertex_buffer: Some(for_common_vertex_buffer),
                for_common_index_buffer: Some(for_common_index_buffer),
                cube_indices_count: Some(cube_indices_count),
                main_depth_view: Some(main_depth_view),
                depth_bind_group: Some(depth_bind_group),
                angle: 0.0,
            },
        );
    }
}

impl<'a> Pass<'a> for ShadowScene {
    fn pass(
        data: &mut crate::shared::ready_paint::HashTypeId2Data,
        mut render_pass: wgpu::RenderPass<'a>,
    ) -> wgpu::RenderPass<'a> {
        let scene = get_res::<Self>(data);
        let depth_bind_group = scene.depth_bind_group.as_ref().unwrap();
        let world = get_res::<World>(data);
        let plane = get_res::<ShadowPlane>(data);

        let for_common_vertex_buffer = scene.for_common_vertex_buffer.as_ref().unwrap();
        let for_common_index_buffer = scene.for_common_index_buffer.as_ref().unwrap();
        let cube_indics_count = scene.cube_indices_count.as_ref().unwrap();
        let plane_indics_count = plane.index_count;

        render_pass.set_pipeline(scene.cube_pipeline.as_ref().unwrap());

        render_pass.set_vertex_buffer(0, for_common_vertex_buffer.slice(..));
        render_pass.set_index_buffer(for_common_index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.set_bind_group(0, depth_bind_group, &[]);
        render_pass.set_bind_group(1, world.uniforms_bind_group.as_ref().unwrap(), &[]);

        render_pass.draw_indexed(0..(cube_indics_count + plane_indics_count), 0, 0..1);
        render_pass
    }
}
impl Update for ShadowScene {
    fn update(
        data: &mut crate::shared::ready_paint::HashTypeId2Data,
        gfx: &crate::shared::ready_paint::Gfx,
        dt: f32,
        surface: &std::sync::Arc<wgpu::Surface<'static>>,
    ) {
        // 1. 获取当前光源和深度纹理资源
        let (depth_texture, scene) = refs_muts::<(Ref<DepthTexture>, Mut<ShadowScene>)>(data);
        let light_storage_buffer = depth_texture.light_storage_buffer.as_ref().unwrap();

        // 2. 计算新的光源位置（围绕Y轴旋转）
        let rotation_speed = 0.8; // 每秒旋转的速度
        let angle = dt / 1000. * rotation_speed;

        scene.angle += angle;

        let radius = 4.0;
        let height = 5.0;

        let light_position = glam::Vec3::new(
            radius * f32::cos(scene.angle),
            height,
            radius * f32::sin(scene.angle),
        );

        // 3. 创建新的光源视图矩阵
        let light_target = glam::Vec3::ZERO; // 总是看向原点
        let light_up = glam::Vec3::Y;
        let light_view = glam::Mat4::look_at_rh(light_position, light_target, light_up);

        // 4. 创建正交投影矩阵（与原来一致）
        let light_proj = glam::Mat4::orthographic_rh(-3.0, 3.0, -3.0, 3.0, 0.1, 10.0);

        // 5. 计算新的视图投影矩阵
        let light_view_proj = light_proj * light_view;

        // 6. 创建新的LightStorage
        let light_storage = LightStorage {
            view_proj: light_view_proj.to_cols_array_2d(),
        };

        // 7. 将新数据写入buffer
        gfx.queue.write_buffer(
            light_storage_buffer,
            0,
            bytemuck::cast_slice(&[light_storage]),
        );
    }
}
const CUBE_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
}
@group(0) @binding(0) var depth_texture: texture_depth_2d;
@group(0) @binding(1) var shadow_sampler: sampler_comparison;
@group(0) @binding(2) var<storage, read> light: LightUniform;

struct LightUniform {
    view_proj: mat4x4<f32>,
}

@group(1) @binding(0) var<uniform> uniforms: Uniforms;
struct Uniforms {
    matrix: mat4x4<f32>,
    resolution: vec2<f32>,
    delta_time: f32,
    _padding: f32,
}
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) light_space_position: vec4<f32>,
}
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // 应用世界变换
    out.world_pos = in.position;
    // 摄像机下变换
    out.position = uniforms.matrix * vec4<f32>(in.position, 1.0);
    // 当前片元到光源的变换
    out.light_space_position = light.view_proj * vec4<f32>(in.position, 1.0);
    
    return out;
}
fn getShadow(coords: vec2<f32>, compare: f32) -> f32 {
    let texelSize = 1.0 / 1024.0;
    let bias = 0.02; 
    var shadow = 0.0;
    for (var y = -2; y <= 2; y += 1) {
        for (var x = -2; x <= 2; x += 1) {
            let offset = vec2<f32>(f32(x), f32(y)) * texelSize;
            shadow += textureSampleCompare(
                depth_texture, 
                shadow_sampler, 
                coords + offset, 
                compare - bias
            );
        }
    }
    return shadow / 25.0;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
         // 计算光空间深度坐标
    let proj_coords = in.light_space_position.xyz / in.light_space_position.w;
    
    // 转换到[0,1]范围
    let shadow_coords = vec2<f32>(
        proj_coords.x * 0.5 + 0.5,
        -proj_coords.y * 0.5 + 0.5
    );
    
    // 当前深度值
    let current_depth = proj_coords.z * 0.5 + 0.5;
    
    // 获取阴影值
    let shadow = getShadow(shadow_coords, current_depth);
    
    // 处理超出深度图边界的情况
    var final_shadow = shadow;
    if (shadow_coords.x < 0.0 || shadow_coords.x > 1.0 || 
        shadow_coords.y < 0.0 || shadow_coords.y > 1.0 ||
        current_depth < 0.0 || current_depth > 1.0) {
        final_shadow = 1.0; // 超出边界，不应用阴影
    }
    
    // 通过片元位置计算法线 (几何方法)
    var normal = vec3<f32>(0.0, 1.0, 0.0); // 默认为上方向
    
    // 立方体法线计算 - 基于位置判断当前面
    if (in.world_pos.y < -0.6) {
        // 平面法线为上方向
        normal = vec3<f32>(0.0, 1.0, 0.0);
    } else {
        let threshold = 0.45; // 接近0.5，但稍微小一点避免边缘问题
        if (abs(in.world_pos.x) > threshold) {
            normal = vec3<f32>(sign(in.world_pos.x), 0.0, 0.0);
        } else if (abs(in.world_pos.y) > threshold) {
            normal = vec3<f32>(0.0, sign(in.world_pos.y), 0.0);
        } else if (abs(in.world_pos.z) > threshold) {
            normal = vec3<f32>(0.0, 0.0, sign(in.world_pos.z));
        }
    }
    let radius = 4.0;
    let height = 5.0;
    let light_pos = vec3<f32>(
        radius * cos(uniforms.delta_time * 0.001),
        height,
        radius * sin(uniforms.delta_time * 0.001)
    );
    let light_dir = normalize(light_pos - in.world_pos);
    let diffuse_strength = max(dot(normal, light_dir), 0.0);
    var object_color: vec3<f32>;
    if (in.world_pos.y < -0.6) {
        // 平面的颜色
        object_color = vec3<f32>(0.2, 0.7, 0.3);
        let pattern = sin(in.world_pos.x * 3.0) * sin(in.world_pos.z * 3.0);
        if (pattern > 0.0) {
            object_color = mix(object_color, vec3<f32>(0.3, 0.8, 0.4), 0.2);
        }

        // 简单的光照计算
        let ambient_strength = 0.3;
        let ambient = ambient_strength * vec3<f32>(1.0);
        
        // 计算漫反射光照
        let light_color = vec3<f32>(1.0, 0.9, 0.8);
        let diffuse = diffuse_strength * light_color * final_shadow;
        
        // 组合光照
        let result = (ambient + diffuse) * object_color;
        
        return vec4<f32>(result, 1.0);
    } else {
        // 立方体的颜色 - 简单版本，基于法线方向
        if (abs(normal.x) > 0.5) {
            // 左右面
            object_color = vec3<f32>(0.7, 0.5, 0.3);
        } else if (abs(normal.y) > 0.5) {
            // 上下面
            object_color = vec3<f32>(0.3, 0.6, 0.8);
        } else {
            // 前后面
            object_color = vec3<f32>(0.6, 0.4, 0.7);
        }
        return vec4<f32>(object_color, 1.0);
    }
}
"#;

pub mod depth_texture;
pub mod plane;
pub mod world;

use std::f32::consts;

use crate::shared::ready_paint::{get_res, return_res, Pass, Ready, Update};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::{util::DeviceExt, PipelineCompilationOptions};

use super::{object::Tetrahedron, world::World};

// Store both eye matrices in a single buffer
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct ViewMatrices {
    left_eye: [[f32; 4]; 4],
    right_eye: [[f32; 4]; 4],
}
#[derive(Default)]
pub struct VRScene {
    pub render_pipeline: Option<wgpu::RenderPipeline>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub view_buffer: Option<wgpu::Buffer>,
    pub depth_texture_view: Option<wgpu::TextureView>,
    pub background_pipeline: Option<wgpu::RenderPipeline>,
}

// VR 配置参数
struct VRConfig {
    ipd: f32,                  // 瞳距，单位：米
    convergence_distance: f32, // 融合距离
    near_plane: f32,           // 近平面
    far_plane: f32,            // 远平面
    fov_horizontal: f32,       // 水平视场角（弧度）
    fov_vertical: f32,         // 垂直视场角（弧度）
}

impl VRScene {}

// Single pass with instanced rendering for both eyes
impl<'a> Pass<'a> for VRScene {
    fn pass(
        data: &mut crate::shared::ready_paint::HashTypeId2Data,
        mut render_pass: wgpu::RenderPass<'a>,
    ) -> wgpu::RenderPass<'a> {
        let vr_scene = get_res::<VRScene>(data);
        let tetra = get_res::<Tetrahedron>(data); // 四面体资源

        // 首先绘制背景
        render_pass.set_pipeline(vr_scene.background_pipeline.as_ref().unwrap());
        render_pass.set_bind_group(0, vr_scene.bind_group.as_ref().unwrap(), &[]);
        render_pass.draw(0..3, 0..1); // 绘制全屏三角形

        // 设置管线
        render_pass.set_pipeline(vr_scene.render_pipeline.as_ref().unwrap());

        // 设置绑定组
        render_pass.set_bind_group(0, vr_scene.bind_group.as_ref().unwrap(), &[]);

        // 设置顶点缓冲区
        render_pass.set_vertex_buffer(0, tetra.object_buffer.as_ref().unwrap().slice(..));

        // 设置索引缓冲区
        render_pass.set_index_buffer(
            tetra.object_line_index_buffer.as_ref().unwrap().slice(..),
            wgpu::IndexFormat::Uint16,
        );

        // 绘制两个实例（左右眼）
        render_pass.draw_indexed(0..12, 0, 0..2);

        render_pass
    }
}

impl Update for VRScene {
    fn update(
        data: &mut crate::shared::ready_paint::HashTypeId2Data,
        gfx: &crate::shared::ready_paint::Gfx,
        dt: f32,
        surface: &std::sync::Arc<wgpu::Surface<'static>>,
    ) {
    }
}

impl Ready for VRScene {
    fn ready(
        &mut self,
        data: &mut crate::shared::ready_paint::HashTypeId2Data,
        gfx: &crate::shared::ready_paint::Gfx,
    ) {
        // 基本 VR 配置
        let vr_config = VRConfig {
            ipd: 0.064,
            convergence_distance: 1.5,
            near_plane: 0.1,
            far_plane: 100.0,
            fov_horizontal: 120.0 * (consts::PI / 180.0),
            fov_vertical: 100.0 * (consts::PI / 180.0),
        };

        // 改进的相机设置
        let camera_position = Vec3::new(0.0, 0.3, 1.5); // 更靠近的位置
        let camera_target = Vec3::new(0.0, 0.0, 0.0); // 看向四面体中心
        let camera_forward = (camera_target - camera_position).normalize();

        // 计算上向量和右向量
        let world_up = Vec3::Z; // 假设Z轴向上
        let camera_right = camera_forward.cross(world_up).normalize();
        let camera_up = camera_right.cross(camera_forward).normalize();

        // 计算左右眼位置
        let half_ipd = vr_config.ipd * 0.5;
        let left_eye_position = camera_position - camera_right * half_ipd;
        let right_eye_position = camera_position + camera_right * half_ipd;

        // 计算左眼矩阵
        let left_view = Mat4::look_at_rh(
            left_eye_position,
            left_eye_position + camera_forward * vr_config.convergence_distance,
            camera_up,
        );

        // 计算右眼矩阵
        let right_view = Mat4::look_at_rh(
            right_eye_position,
            right_eye_position + camera_forward * vr_config.convergence_distance,
            camera_up,
        );

        // 创建非对称投影矩阵
        let aspect_ratio = 300.0 / 300.0;

        // 左眼投影矩阵
        let left_projection = create_asymmetric_projection(
            -vr_config.fov_horizontal * 0.3,
            vr_config.fov_horizontal * 0.3,
            -vr_config.fov_vertical * 0.3,
            vr_config.fov_vertical * 0.3,
            vr_config.near_plane,
            vr_config.far_plane,
        );

        // 右眼投影矩阵
        let right_projection = create_asymmetric_projection(
            -vr_config.fov_horizontal * 0.3,
            vr_config.fov_horizontal * 0.3,
            -vr_config.fov_vertical * 0.3,
            vr_config.fov_vertical * 0.3,
            vr_config.near_plane,
            vr_config.far_plane,
        );

        // 组合投影和视图矩阵
        let left_eye_matrix = left_projection * left_view;
        let right_eye_matrix = right_projection * right_view;

        // 创建最终的 ViewMatrices
        let view_matrices = ViewMatrices {
            left_eye: left_eye_matrix.to_cols_array_2d(),
            right_eye: right_eye_matrix.to_cols_array_2d(),
        };

        // 创建 uniform buffer
        let view_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("VR View Matrices Buffer"),
                contents: bytemuck::cast_slice(&[view_matrices]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        // 创建绑定组布局
        let bind_group_layout =
            gfx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("VR Bind Group Layout"),
                    entries: &[
                        // World/Model matrix
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // View matrices (left & right eye)
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        // 加载着色器
        let shader = gfx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("VR Shader"),
                source: wgpu::ShaderSource::Wgsl(VR_SHADER.into()),
            });

        // 创建管线布局
        let pipeline_layout = gfx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("VR Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
        let depth_texture = gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("VR Depth Texture"),
            size: wgpu::Extent3d {
                width: 600,
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

        // 创建渲染管线
        let render_pipeline = gfx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("VR Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress, // 位置(3) + 颜色(3)
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3, // position
                        }],
                    }],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::LineList,
                    front_face: wgpu::FrontFace::Cw,
                    ..Default::default()
                },
                depth_stencil: None,
                // Some(wgpu::DepthStencilState {
                //     format: wgpu::TextureFormat::Depth32Float,
                //     depth_write_enabled: true,
                //     depth_compare: wgpu::CompareFunction::Less,
                //     stencil: wgpu::StencilState::default(),
                //     bias: wgpu::DepthBiasState::default(),
                // }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });
        let world = get_res::<World>(data);
        let world_matrix_buffer = world.uniform_buffer.as_ref().unwrap();
        // 创建绑定组
        let bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("VR Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: world_matrix_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: view_buffer.as_entire_binding(),
                },
            ],
        });
        // 背景着色器
        let background_shader = gfx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("VR Background Shader"),
                source: wgpu::ShaderSource::Wgsl(VR_BACKGROUND_SHADER.into()),
            });

        // 背景管线
        let background_pipeline =
            gfx.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("VR Background Pipeline"),
                    layout: Some(&pipeline_layout), // 可以复用相同的布局
                    vertex: wgpu::VertexState {
                        module: &background_shader,
                        entry_point: Some("vs_main"),
                        buffers: &[], // 不需要顶点缓冲区
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &background_shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Bgra8Unorm,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                    cache: None,
                });
        return_res(
            data,
            Self {
                render_pipeline: Some(render_pipeline),
                bind_group: Some(bind_group),
                view_buffer: Some(view_buffer),
                depth_texture_view: Some(depth_view),
                background_pipeline: Some(background_pipeline),
            },
        );
    }
}
// 创建非对称投影矩阵的辅助函数
fn create_asymmetric_projection(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Mat4 {
    let x = (2.0 * near) / (right - left);
    let y = (2.0 * near) / (top - bottom);
    let a = (right + left) / (right - left);
    let b = (top + bottom) / (top - bottom);
    let c = -(far + near) / (far - near);
    let d = -(2.0 * far * near) / (far - near);

    Mat4::from_cols_array_2d(&[
        [x, 0.0, 0.0, 0.0],
        [0.0, y, 0.0, 0.0],
        [a, b, c, -1.0],
        [0.0, 0.0, d, 0.0],
    ])
}

// Updated shader that uses instance_index to select view matrix
const VR_SHADER: &str = r#"
struct Uniforms {
    matrix: mat4x4<f32>,
    resolution: vec2<f32>,
    delta_time: f32,
    padding: f32,
}

struct ViewMatrices {
    left_eye: mat4x4<f32>,
    right_eye: mat4x4<f32>,
}

struct VertexInput {
    @location(0) pos: vec3<f32>,
}

@group(0) @binding(1) var<uniform> view_matrices: ViewMatrices;

@vertex
fn vs_main(
    in: VertexInput,
    @builtin(instance_index) instance_index: u32
) -> @builtin(position) vec4<f32> {
    var position: vec4<f32>;
    if (instance_index == 0u) {
        // 左眼
        position = view_matrices.left_eye * vec4<f32>(in.pos, 1.0);
        position.x = position.x * 0.5 - 0.5 * position.w; // 移至左半屏
    } else {
        // 右眼
        position = view_matrices.right_eye * vec4<f32>(in.pos, 1.0);
        position.x = position.x * 0.5 + 0.5 * position.w; // 移至右半屏
    }
    return position;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    // return vec4<f32>(in.color, 1.0);
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
"#;

// Background shader
const VR_BACKGROUND_SHADER: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_idx: u32) -> @builtin(position) vec4<f32> {
    // 创建全屏三角形
    var pos = vec2<f32>(0.0, 0.0);
    if (vertex_idx == 0u) {
        pos = vec2<f32>(-1.0, -1.0);
    } else if (vertex_idx == 1u) {
        pos = vec2<f32>(3.0, -1.0);
    } else {
        pos = vec2<f32>(-1.0, 3.0);
    }
    return vec4<f32>(pos, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
     // 获取屏幕尺寸 (假设为600x300)
    let screen_width = 600.0;
    let screen_height = 300.0;
    
    // 计算视口中心
    let left_center = vec2<f32>(screen_width * 0.25, screen_height * 0.5);
    let right_center = vec2<f32>(screen_width * 0.75, screen_height * 0.5);
    
    // 计算当前像素到视口中心的距离
    let pixel_pos = vec2<f32>(pos.x, pos.y);
    let radius = min(screen_width * 0.25, screen_height * 0.5) * 0.9; // 视口半径稍小一些
    
    // 确定当前像素属于哪个眼睛
    let is_left_eye = pos.x < screen_width * 0.5;
    let center = select(right_center, left_center, is_left_eye);
    
    // 计算到中心的距离
    let dist = distance(pixel_pos, center);
    
    // 确定是否在视口圆圈内
    if (dist <= radius) {
        // 在视口内 - 绘制场景
        // 这里可以添加不同颜色的背景
        let bg_color = select(
            vec3<f32>(0.1, 0.2, 0.1), // 右眼背景色 - 暗绿
            vec3<f32>(0.1, 0.1, 0.2), // 左眼背景色 - 暗蓝
            is_left_eye
        );
        
        // 如果这里是四面体线条，则绘制线条，否则绘制背景
        // 简化处理：始终返回背景色，四面体线条会在此之上绘制
        return vec4<f32>(bg_color, 1.0);
    } else {
        // 在视口外 - 绘制黑色
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}
"#;

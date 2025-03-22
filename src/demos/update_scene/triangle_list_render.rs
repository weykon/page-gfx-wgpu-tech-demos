use glam::Mat4;
use wgpu::util::DeviceExt;

use crate::shared::ready_paint::{get_res, get_res_mut, return_res, Pass, Ready, Update};

use super::object::Tetrahedron;

pub struct TriangleListRender;
impl<'a> Pass<'a> for TriangleListRender {
    fn pass(
        data: &mut crate::shared::ready_paint::HashTypeId2Data,
        mut render_pass: wgpu::RenderPass<'a>,
    ) -> wgpu::RenderPass<'a> {
        let tetra = get_res::<Tetrahedron>(data);

        render_pass.set_pipeline(tetra.triangle_list_pipeline.as_ref().unwrap());
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

#[derive(Default)]
pub struct NormalTriangleListRender {
    pub object_buffer: Option<wgpu::Buffer>,
    pub pipeline: Option<wgpu::RenderPipeline>,
    pub colored_vertices: Vec<ColoredVertex>,
    pub triangle_list_normal_texture_view: Option<wgpu::TextureView>,
    pub triangle_list_normal_indices_buffer: Option<wgpu::Buffer>,
}
impl Pass<'_> for NormalTriangleListRender {
    fn pass<'a>(
        data: &mut crate::shared::ready_paint::HashTypeId2Data,
        mut render_pass: wgpu::RenderPass<'a>,
    ) -> wgpu::RenderPass<'a> {
        let here = get_res::<Self>(data);

        render_pass.set_pipeline(here.pipeline.as_ref().unwrap());
        render_pass.set_vertex_buffer(0, here.object_buffer.as_ref().unwrap().slice(..));
        render_pass.set_index_buffer(
            here.triangle_list_normal_indices_buffer
                .as_ref()
                .unwrap()
                .slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(0..12, 0, 0..1);

        render_pass
    }
}
impl Update for NormalTriangleListRender {
    fn update(
        data: &mut crate::shared::ready_paint::HashTypeId2Data,
        gfx: &crate::shared::ready_paint::Gfx,
        dt: f32,
        surface: &std::sync::Arc<wgpu::Surface<'static>>,
    ) {
        let here = get_res_mut::<Self>(data);
        let buffer = here.object_buffer.as_ref().unwrap();
        let dt_seconds = dt / 1000.0;
        let rotation_speed = std::f32::consts::PI;
        let rotation = Mat4::from_rotation_y(rotation_speed * dt_seconds);

        // 对每个顶点应用旋转变换
        for vertex in here.colored_vertices.iter_mut() {
            // 旋转位置
            let pos = glam::Vec3::from(vertex.position);
            let rotated_pos = rotation.transform_point3(pos);

            // 旋转法线 (注意：法线需要使用法线矩阵，但对于纯旋转矩阵，直接使用相同的旋转矩阵是可以的)
            let norm = glam::Vec3::from(vertex.normal);
            let rotated_norm = rotation.transform_vector3(norm).normalize();

            *vertex = ColoredVertex {
                position: [rotated_pos.x, rotated_pos.y, rotated_pos.z],
                normal: [rotated_norm.x, rotated_norm.y, rotated_norm.z],
                color: vertex.color, // 颜色不需要旋转
            }
        }
        // 更新GPU缓冲区
        gfx.queue
            .write_buffer(buffer, 0, bytemuck::cast_slice(&here.colored_vertices));
    }
}
impl Ready for NormalTriangleListRender {
    fn ready(
        &mut self,
        data: &mut crate::shared::ready_paint::HashTypeId2Data,
        gfx: &crate::shared::ready_paint::Gfx,
    ) {
        let tetra = get_res::<Tetrahedron>(data);
        let vertices = tetra.vertices.as_ref().unwrap();

        // 四面体的索引 - 四个面，每个面三个顶点
        let indices: [u16; 12] = [0, 1, 2, 0, 2, 3, 0, 3, 1, 1, 3, 2];

        // 生成带法线和颜色的顶点
        let (colored_vertices, _) = generate_colored_vertices(&vertices, &indices, true);

        let triangle_list_normal_indices_buffer =
            gfx.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Triangle List Normal Indices Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
        let triangle_list_normal_texture = gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Triangle List Normal Texture"),
            size: wgpu::Extent3d {
                width: 300,
                height: 300,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let triangle_list_normal_texture_view =
            triangle_list_normal_texture.create_view(&wgpu::TextureViewDescriptor::default());
        // 创建带颜色和法线的缓冲区
        let object_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Object Buffer"),
                contents: bytemuck::cast_slice(&colored_vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ColoredVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // 位置
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // 法线
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // 颜色
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        };
        let shader = gfx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Triangle List Shader"),
                source: wgpu::ShaderSource::Wgsl(SHADER.into()),
            });
        let world_pipeline_layout = get_res::<Tetrahedron>(data)
            .world_pipeline_layout
            .as_ref()
            .unwrap();

        let pipeline = gfx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Triangle List Pipeline"),
                layout: Some(world_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[vertex_layout],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24Plus,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        return_res(
            data,
            Self {
                object_buffer: Some(object_buffer),
                pipeline: Some(pipeline),
                colored_vertices,
                triangle_list_normal_texture_view: Some(triangle_list_normal_texture_view),
                triangle_list_normal_indices_buffer: Some(triangle_list_normal_indices_buffer),
            },
        );
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ColoredVertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 3],
}

// 根据现有顶点生成带法线和颜色的顶点数据
fn generate_colored_vertices(
    vertices: &[[f32; 3]; 4],
    indices: &[u16],
    white: bool,
) -> (Vec<ColoredVertex>, Vec<u16>) {
    // 计算每个面的法线
    let face_normals = calculate_face_normals(vertices, indices);

    // 为每个顶点计算平均法线
    let vertex_normals = calculate_vertex_normals(vertices, indices, &face_normals);

    // 生成颜色 (基于位置生成有趣的颜色)
    let colors = if white {
        gen_white_color(vertices)
    } else {
        generate_colors(vertices)
    };

    // 创建彩色顶点
    let mut colored_vertices = Vec::with_capacity(vertices.len());
    for i in 0..vertices.len() {
        colored_vertices.push(ColoredVertex {
            position: vertices[i],
            normal: vertex_normals[i],
            color: colors[i],
        });
    }

    // 索引可以保持不变
    (colored_vertices, indices.to_vec())
}

// 计算面法线
fn calculate_face_normals(vertices: &[[f32; 3]; 4], indices: &[u16]) -> Vec<[f32; 3]> {
    let mut face_normals = Vec::new();

    // 每3个索引组成一个面
    for i in (0..indices.len()).step_by(3) {
        let i0 = indices[i] as usize;
        let i1 = indices[i + 1] as usize;
        let i2 = indices[i + 2] as usize;

        let v0 = vertices[i0];
        let v1 = vertices[i1];
        let v2 = vertices[i2];

        // 计算边向量
        let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];

        let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

        // 叉乘计算法线
        let normal = [
            edge1[1] * edge2[2] - edge1[2] * edge2[1],
            edge1[2] * edge2[0] - edge1[0] * edge2[2],
            edge1[0] * edge2[1] - edge1[1] * edge2[0],
        ];

        // 归一化
        let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        let normalized = [normal[0] / length, normal[1] / length, normal[2] / length];

        face_normals.push(normalized);
    }

    face_normals
}

// 计算顶点法线 (将面法线平均)
fn calculate_vertex_normals(
    vertices: &[[f32; 3]; 4],
    indices: &[u16],
    face_normals: &[[f32; 3]],
) -> Vec<[f32; 3]> {
    let mut vertex_normals = vec![[0.0, 0.0, 0.0]; vertices.len()];
    let mut counts = vec![0; vertices.len()];

    // 将面法线添加到各个顶点
    for i in (0..indices.len()).step_by(3) {
        let face_index = i / 3;
        let face_normal = face_normals[face_index];

        // 将面法线添加到构成该面的所有顶点
        for j in 0..3 {
            let vertex_index = indices[i + j] as usize;
            vertex_normals[vertex_index][0] += face_normal[0];
            vertex_normals[vertex_index][1] += face_normal[1];
            vertex_normals[vertex_index][2] += face_normal[2];
            counts[vertex_index] += 1;
        }
    }

    // 归一化顶点法线
    for i in 0..vertex_normals.len() {
        if counts[i] > 0 {
            vertex_normals[i][0] /= counts[i] as f32;
            vertex_normals[i][1] /= counts[i] as f32;
            vertex_normals[i][2] /= counts[i] as f32;

            // 再次归一化
            let normal = vertex_normals[i];
            let length =
                (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();

            if length > 0.0 {
                vertex_normals[i][0] /= length;
                vertex_normals[i][1] /= length;
                vertex_normals[i][2] /= length;
            }
        }
    }

    vertex_normals
}

fn gen_white_color(vertices: &[[f32; 3]; 4]) -> Vec<[f32; 3]> {
    vec![[1.0, 1.0, 1.0]; vertices.len()]
}
// 生成颜色 (基于顶点位置)
pub fn generate_colors(vertices: &[[f32; 3]; 4]) -> Vec<[f32; 3]> {
    vertices
        .iter()
        .map(|v| {
            // 将位置映射到[0,1]范围的颜色
            [
                (v[0] + 1.0) * 0.5, // x坐标影响红色
                (v[1] + 1.0) * 0.5, // y坐标影响绿色
                (v[2] + 1.0) * 0.5, // z坐标影响蓝色
            ]
        })
        .collect()
}

const SHADER: &str = r#"
struct Uniforms {
    matrix: mat4x4<f32>,
    resolution: vec2<f32>,
    delta_time: f32,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.matrix * vec4<f32>(in.position, 1.0);
    out.normal = (uniforms.matrix * vec4<f32>(in.normal, 0.0)).xyz;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 简单的漫反射光照模型
    let light_dir = normalize(vec3<f32>(0.3, -1.0, 0.7));
    let normal = normalize(in.normal);
    
    // 计算漫反射系数
    let diffuse = max(dot(normal, light_dir), 0.0);
    
    // 环境光
    let ambient = 0.2;
    
    // 最终颜色计算
    let light = ambient + diffuse * 0.8;
    let result = in.color * light;
    
    return vec4<f32>(result, 1.0);
}
"#;

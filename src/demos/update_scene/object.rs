use std::sync::Arc;

use crate::{
    console_log,
    shared::ready_paint::{
        get_res, get_res_mut, return_res, Gfx, HashTypeId2Data, Pass, Ready, Update,
    },
};
use glam::Mat4;
use wgpu::{util::DeviceExt, Surface};

use super::world::World;

#[derive(Default)]
pub struct Tetrahedron {
    vertices: Option<[[f32; 3]; 4]>,
    object_buffer: Option<wgpu::Buffer>,
    object_line_index_buffer: Option<wgpu::Buffer>,
    pipeline: Option<wgpu::RenderPipeline>,
}

impl Ready for Tetrahedron {
    fn ready(&mut self, data: &mut HashTypeId2Data, gfx: &Gfx) {
        let vertices: [[f32; 3]; 4] = [
            [0.0, 1., 0.0],
            [-1., -1., -1.],
            [1., -1., -1.],
            [0.0, -1., 1.],
        ];

        let object_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Object Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            }],
        };
        let object_line_index_buffer =
            gfx.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Object Line Index Buffer"),
                    contents: bytemuck::cast_slice(&[0u16, 1, 1, 2, 2, 0, 0, 3, 1, 3, 2, 3]),
                    usage: wgpu::BufferUsages::INDEX,
                });
        let shader = gfx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("screen shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SHADER)),
            });
        let world = get_res::<World>(data);

        let world_pipeline_layout =
            gfx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("World Pipeline Layout"),
                    bind_group_layouts: &[world.uniforms_bind_group_layout.as_ref().unwrap()],
                    push_constant_ranges: &[],
                });

        let object_pipeline = gfx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Object Pipeline"),
                layout: Some(&world_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[vertex_layout],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    compilation_options: Default::default(),
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::TextureFormat::Bgra8Unorm.into())],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::LineList,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });
        return_res(
            data,
            Tetrahedron {
                vertices: Some(vertices),
                object_buffer: Some(object_buffer),
                object_line_index_buffer: Some(object_line_index_buffer),
                pipeline: Some(object_pipeline),
            },
        );
    }
}
impl<'a> Pass<'a> for Tetrahedron {
    fn pass(
        data: &mut HashTypeId2Data,
        mut render_pass: wgpu::RenderPass<'a>,
    ) -> wgpu::RenderPass<'a> {
        let tetra = get_res::<Tetrahedron>(data);
        render_pass.set_pipeline(tetra.pipeline.as_ref().unwrap());
        render_pass.set_vertex_buffer(0, tetra.object_buffer.as_ref().unwrap().slice(..));
        render_pass.set_index_buffer(
            tetra.object_line_index_buffer.as_ref().unwrap().slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(0..12, 0, 0..1);
        render_pass
    }
}
impl Update for Tetrahedron {
    fn update(data: &mut HashTypeId2Data, gfx: &Gfx, dt: f32, surface: &Arc<Surface<'static>>) {
        let tetra = get_res_mut::<Self>(data);
        let buffer = tetra.object_buffer.as_ref().unwrap();
        let dt_seconds = dt / 1000.0;
        let rotation_speed = std::f32::consts::PI;
        let rotation = Mat4::from_rotation_y(rotation_speed * dt_seconds);
        let rotated_vertices = tetra.vertices.as_ref().unwrap().map(|v| {
            let v = rotation.transform_point3(glam::Vec3::from(v));
            [v.x, v.y, v.z]
        });
        tetra.vertices = Some(rotated_vertices);
        gfx.queue.write_buffer(
            buffer,
            0,
            bytemuck::cast_slice(rotated_vertices.as_flattened()),
        );
    }
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

@vertex
fn vs_main(@location(0) pos: vec3<f32>) -> @builtin(position) vec4<f32> {
    return uniforms.matrix * vec4<f32>(pos, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
"#;

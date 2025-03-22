use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};
use std::num::NonZeroU32;
use wgpu::{util::DeviceExt, PipelineCompilationOptions};

use crate::shared::ready_paint::{get_res, get_res_mut, return_res, Pass, Ready, Update};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    model_matrix: [[f32; 4]; 4],
    resolution: [f32; 2],
    time: f32,
    _padding: f32,
}
impl Default for VRScene {
    fn default() -> Self {
        Self {
            pipeline: None,
            vertex_buffer: None,
            uniform_buffer: None,
            view_matrices_buffer: None,
            uniform_bind_group: None,
            num_vertices: 0,
            time: 0.0,
        }
    }
}
pub struct VRScene {
    pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: Option<wgpu::Buffer>,
    uniform_buffer: Option<wgpu::Buffer>,
    view_matrices_buffer: Option<wgpu::Buffer>,
    uniform_bind_group: Option<wgpu::BindGroup>,
    num_vertices: u32,
    time: f32,
}

impl VRScene {
    fn create_view_matrices() -> [[f32; 16]; 2] {
        let left_eye_position = Vec3::new(-0.03, 0.0, 0.0);
        let target = Vec3::new(0.0, 0.0, -1.0);
        let up = Vec3::new(0.0, 1.0, 0.0);
        let left_view = Mat4::look_at_rh(left_eye_position, target, up);

        let right_eye_position = Vec3::new(0.03, 0.0, 0.0);
        let right_view = Mat4::look_at_rh(right_eye_position, target, up);

        let aspect = 1.0;
        let fovy = 65.0_f32.to_radians();
        let znear = 0.1;
        let zfar = 100.0;
        let proj = Mat4::perspective_rh(fovy, aspect, znear, zfar);

        // Combine view and projection matrices
        let left_view_proj = proj * left_view;
        let right_view_proj = proj * right_view;

        // Convert to arrays
        let mut result = [[0.0; 16]; 2];
        result[0] = left_view_proj.to_cols_array();
        result[1] = right_view_proj.to_cols_array();

        result
    }
}

impl<'a> Pass<'a> for VRScene {
    fn pass(
        data: &mut crate::shared::ready_paint::HashTypeId2Data,
        mut render_pass: wgpu::RenderPass<'a>,
    ) -> wgpu::RenderPass<'a> {
        let vr_scene = get_res::<VRScene>(data);

        if let Some(pipeline) = &vr_scene.pipeline {
            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, vr_scene.uniform_bind_group.as_ref().unwrap(), &[]);
            render_pass.set_vertex_buffer(0, vr_scene.vertex_buffer.as_ref().unwrap().slice(..));
            // Draw 2 instances - one for each eye
            render_pass.draw(0..vr_scene.num_vertices, 0..2);
        }

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
        let vr_scene = get_res_mut::<VRScene>(data);

        // Update time
        vr_scene.time += dt;

        // Update uniform buffer with model matrix
        let angle = vr_scene.time * 0.5;
        let rotation = Quat::from_rotation_y(angle);
        let model = Mat4::from_quat(rotation);

        // Convert to uniform data
        let uniforms = Uniforms {
            model_matrix: model.to_cols_array_2d(),
            resolution: [
                surface.get_current_texture().unwrap().texture.size().width as f32,
                surface.get_current_texture().unwrap().texture.size().height as f32,
            ],
            time: vr_scene.time,
            _padding: 0.0,
        };

        gfx.queue.write_buffer(
            vr_scene.uniform_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&[uniforms]),
        );

        // Update view matrices if needed
        // In a real app, you'd only update these when the head position changes
        let view_matrices = VRScene::create_view_matrices();
        gfx.queue.write_buffer(
            vr_scene.view_matrices_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&view_matrices),
        );
    }
}

impl Ready for VRScene {
    fn ready(
        &mut self,
        data: &mut crate::shared::ready_paint::HashTypeId2Data,
        gfx: &crate::shared::ready_paint::Gfx,
    ) {
        // Create the vertices for a simple triangle
        let vertices = [
            Vertex {
                position: [-0.5, -0.5, 0.0],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.0, 0.5, 0.0],
                color: [0.0, 0.0, 1.0],
            },
        ];
        self.num_vertices = vertices.len() as u32;

        // Create vertex buffer
        self.vertex_buffer = Some(gfx.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));

        // Create the uniform buffer
        let uniforms = Uniforms {
            model_matrix: Mat4::IDENTITY.to_cols_array_2d(),
            resolution: [300.0, 300.0], // Default resolution
            time: 0.0,
            _padding: 0.0,
        };

        self.uniform_buffer = Some(gfx.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        ));

        // Create view matrices storage buffer
        let view_matrices = VRScene::create_view_matrices();
        self.view_matrices_buffer = Some(gfx.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("View Matrices Buffer"),
                contents: bytemuck::cast_slice(&view_matrices),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            },
        ));

        // Create bind group layout
        let bind_group_layout =
            gfx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        // Create the bind group
        self.uniform_bind_group = Some(
            gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Uniform Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uniform_buffer.as_ref().unwrap().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self
                            .view_matrices_buffer
                            .as_ref()
                            .unwrap()
                            .as_entire_binding(),
                    },
                ],
            }),
        );

        let world_pipeline_layout =
            gfx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("World Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

        let shader = gfx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(SHADER.into()),
            });

        // Use the vertex layout from our struct
        let vertex_layout_ref = &[Vertex::desc()];

        self.pipeline = Some(
            gfx.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Triangle List VR Pipeline"),
                    layout: Some(&world_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: vertex_layout_ref,
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
                    // For this approach, we don't need to set multiview directly
                    multiview: None,
                    cache: None,
                }),
        );

        return_res(
            data,
            Self {
                pipeline: self.pipeline.clone(),
                vertex_buffer: self.vertex_buffer.clone(),
                uniform_buffer: self.uniform_buffer.clone(),
                view_matrices_buffer: self.view_matrices_buffer.clone(),
                uniform_bind_group: self.uniform_bind_group.clone(),
                num_vertices: self.num_vertices,
                time: self.time,
            },
        );
    }
}

const SHADER: &str = r#"
struct Uniforms {
    model_matrix: mat4x4<f32>,
    resolution: vec2<f32>,
    time: f32,
    padding: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> view_matrices: array<mat4x4<f32>, 2>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @builtin(layer) layer: u32, // We use layer instead of view_index
}

@vertex
fn vs_main(
    in: VertexInput,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform the vertex position
    let world_pos = uniforms.model_matrix * vec4<f32>(in.position, 1.0);
    
    // For multiview, we use the renderer's capability to draw to multiple layers
    // We can use instance_index % 2 to determine which view (left or right eye)
    let view_id = instance_index % 2u;
    
    // Use the appropriate view matrix based on the view index (left or right eye)
    let view_proj = view_matrices[view_id];
    out.clip_position = view_proj * world_pos;
    
    // Set the layer for multiview rendering
    out.layer = view_id;
    
    // Pass the color to the fragment shader
    out.color = in.color;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;

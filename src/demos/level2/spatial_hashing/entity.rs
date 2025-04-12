use bytemuck::{Pod, Zeroable};
use std::mem::size_of;
use wgpu::{util::DeviceExt, PipelineCompilationOptions}; // Add this import for size_of

use crate::shared::ready_paint::{get_res, get_res_mut, return_res, Pass, Ready, Update};
#[repr(C)]
#[derive(Debug, Clone, Pod, Copy, Zeroable)]
pub struct Entity {
    position: [f32; 2],
    velocity: [f32; 2],
}

struct EntityShape {
    vertex: [[f32; 2]; 3],
}
impl Default for EntityShape {
    fn default() -> Self {
        let vertex = [[-0.1, 0.], [0.1, 0.], [0., 0.1]];
        EntityShape { vertex }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EntityRender {
    pub entities_instance_buffer: Option<wgpu::Buffer>,
    pub boid_config_buffer: Option<wgpu::Buffer>,
    pub compute_pipeline: Option<wgpu::ComputePipeline>,
    pub compute_buffers: Option<Vec<wgpu::Buffer>>,
    pub compute_bind_groups: Option<Vec<wgpu::BindGroup>>,
    pub render_pipeline: Option<wgpu::RenderPipeline>,
    pub buffer_index: usize,
    pub entity_shape_vertex_buffer: Option<wgpu::Buffer>, // New field to store the entity shape vertices
}

pub const MAX_ENTITY: usize = 5000;
impl Ready for EntityRender {
    fn ready(
        &mut self,
        data: &mut crate::shared::ready_paint::HashTypeId2Data,
        gfx: &crate::shared::ready_paint::Gfx,
    ) {
        let compute_shader = gfx
            .device
            .create_shader_module(wgpu::include_wgsl!("compute.wgsl"));
        let draw_shader = gfx
            .device
            .create_shader_module(wgpu::include_wgsl!("draw.wgsl"));
        // 实体随机位置和动量
        let mut entities = Vec::new();
        for i in 0..MAX_ENTITY {
            let id = i as u32;
            let position = random_vec2(-1.0, 1.0);
            // 确保初始速度更均衡，增加一些随机性
            let velocity = random_vec2(-0.05, 0.05);
            entities.push(Entity { position, velocity });
        }
        // instance 变化的数据
        let entities_instance_buffer =
            gfx.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Entity Buffer"),
                    contents: bytemuck::cast_slice(&entities),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });
        // 实体形状
        let entity_shape = EntityShape::default();
        // 只放在render就可以
        let entity_shape_buffer =
            gfx.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Entity Shape Buffer"),
                    contents: bytemuck::cast_slice(&entity_shape.vertex),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

        let boid_config = [
            0.04f32, // deltaT - 较小的时间步长更稳定
            0.1,     // rule1Distance - 凝聚距离
            0.02,    // rule2Distance - 分离距离(较小)
            0.08,     // rule3Distance - 对齐距离
            0.01,    // rule1Scale - 凝聚力强度(保持较弱)
            0.03,    // rule2Scale - 分离力强度(较强以避免聚集)
            0.02,    // rule3Scale - 对齐力强度(适中)
        ]
        .to_vec();
        let boid_config_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Simulation Parameter Buffer"),
                contents: bytemuck::cast_slice(&boid_config),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let entity_size: usize = size_of::<Entity>();
        let bind_group_layout =
            gfx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Entity Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(
                                    (boid_config.len() * size_of::<f32>()) as _,
                                ),
                            },
                            count: None,
                        },
                        // 读取
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(
                                    (entity_size * MAX_ENTITY) as u64,
                                ),
                            },
                            count: None,
                        },
                        // 写入
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(
                                    (entity_size * MAX_ENTITY) as u64,
                                ),
                            },
                            count: None,
                        },
                    ],
                });

        let mut compute_bind_groups = Vec::<wgpu::BindGroup>::new();
        let mut compute_buffers = Vec::<wgpu::Buffer>::new();

        for i in 0..2 {
            compute_buffers.push(gfx.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Entity Compute Buffer {i}")),
                    contents: bytemuck::cast_slice(&entities),
                    usage: wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::COPY_DST,
                },
            ));
        }

        for i in 0..2 {
            compute_bind_groups.push(gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Entity Compute Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: boid_config_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: compute_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: compute_buffers[(i + 1) % 2].as_entire_binding(),
                    },
                ],
            }));
        }
        let compute_pipeline_layout =
            gfx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Compute Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });
        let compute_pipeline =
            gfx.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Compute pipeline"),
                    layout: Some(&compute_pipeline_layout),
                    module: &compute_shader,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    cache: None,
                });

        let render_pipeline_layout =
            gfx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });
        let render_pipeline = gfx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &draw_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[
                        // 实体实例数据
                        wgpu::VertexBufferLayout {
                            array_stride: size_of::<Entity>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![ 0 => Float32x2 , 1 => Float32x2],
                        },
                        // 三角形顶点数据 - 作为第二个顶点缓冲区
                        wgpu::VertexBufferLayout {
                            array_stride: size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![ 2 => Float32x2 ],
                        },
                    ],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &draw_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::Zero,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
                cache: None,
            });

        return_res(
            data,
            EntityRender {
                entities_instance_buffer: Some(entities_instance_buffer),
                boid_config_buffer: Some(boid_config_buffer),
                compute_pipeline: Some(compute_pipeline),
                compute_buffers: Some(compute_buffers),
                compute_bind_groups: Some(compute_bind_groups),
                render_pipeline: Some(render_pipeline),
                buffer_index: 0,
                entity_shape_vertex_buffer: Some(entity_shape_buffer), // Store the entity shape buffer
            },
        );
    }
}

fn random_vec2(min: f32, max: f32) -> [f32; 2] {
    let mut bytes = [0u8; 8];
    getrandom::getrandom(&mut bytes).expect("获取随机字节失败");

    // 使用4个字节构建一个u32，然后转换为0-1范围的f32
    let rand_x =
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f32 / u32::MAX as f32;
    let rand_y =
        u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as f32 / u32::MAX as f32;

    let range = max - min;
    [min + rand_x * range, min + rand_y * range]
}

impl<'a> Pass<'a> for EntityRender {
    fn pass(
        data: &mut crate::shared::ready_paint::HashTypeId2Data,
        mut render_pass: wgpu::RenderPass<'a>,
    ) -> wgpu::RenderPass<'a> {
        let entity_render = get_res::<Self>(data);

        // 设置渲染管线
        render_pass.set_pipeline(entity_render.render_pipeline.as_ref().unwrap());

        // 实体数据缓冲区（与之前相同）
        let current_buffer_index = entity_render.buffer_index;
        let compute_buffers = entity_render.compute_buffers.as_ref().unwrap();
        render_pass.set_vertex_buffer(0, compute_buffers[current_buffer_index].slice(..));

        // 设置三角形顶点缓冲区
        render_pass.set_vertex_buffer(
            1,
            entity_render
                .entity_shape_vertex_buffer
                .as_ref()
                .unwrap()
                .slice(..),
        );

        // 绘制所有实体
        render_pass.draw(0..3, 0..MAX_ENTITY as u32);

        render_pass
    }
}

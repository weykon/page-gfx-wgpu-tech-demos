use std::{f32::consts, sync::Arc};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupLayoutEntry, ShaderStages, Surface,
};

use crate::shared::ready_paint::{
    get_res, get_res_mut, return_res, Gfx, HashTypeId2Data, Pass, Ready, Update,
};

#[derive(Default)]
pub struct World {
    uniform_buffer: Option<wgpu::Buffer>,
    pub uniforms_bind_group_layout: Option<wgpu::BindGroupLayout>,
    uniforms_bind_group: Option<wgpu::BindGroup>,
    uniforms: Option<Uniforms>,
    side_view_uniforms: Option<Uniforms>, // 侧视角uniform数据
}

impl Ready for World {
    fn ready(&mut self, data: &mut HashTypeId2Data, gfx: &Gfx) {
        println!("world ready");
        let world = generate_matrix(600. / 400.);
        let uniforms = Uniforms {
            resolution: [600., 400.],
            matrix: world.to_cols_array(),
            delta_time: 0.,
            _padding: 0.,
        };
        // 侧视角（正交投影）
        let side_view_matrix = generate_orthographic_matrix(600. / 400.);
        let side_view_uniforms = Uniforms {
            resolution: [600., 400.],
            matrix: side_view_matrix.to_cols_array(),
            delta_time: 0.,
            _padding: 0.,
        };

        // 创建主视角缓冲区
        let uniform_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("uniform buffer"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // 创建侧视角缓冲区
        let side_view_uniform_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("side view uniform buffer"),
            contents: bytemuck::bytes_of(&side_view_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniforms_bind_group_layout =
            gfx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("uniforms_bind_group_layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                std::mem::size_of::<Uniforms>() as u64,
                            ),
                        },
                        count: None,
                    }],
                });

        let uniforms_bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniforms_bind_group"),
            layout: &uniforms_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        return_res(
            data,
            World {
                uniform_buffer: Some(uniform_buffer),
                uniforms_bind_group_layout: Some(uniforms_bind_group_layout),
                uniforms_bind_group: Some(uniforms_bind_group),
                uniforms: Some(uniforms),
            },
        );
    }
}

impl Update for World {
    fn update(data: &mut HashTypeId2Data, gfx: &Gfx, dt: f32, surface: &Arc<Surface<'static>>) {
        let world = get_res_mut::<Self>(data);
        const DELTA_TIME_OFFSET: wgpu::BufferAddress = 72;
        world.uniforms.as_mut().unwrap().delta_time = dt;
        gfx.queue.write_buffer(
            &world.uniform_buffer.as_ref().unwrap(),
            DELTA_TIME_OFFSET,
            bytemuck::cast_slice(&[dt]),
        );
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    matrix: [f32; 16],
    resolution: [f32; 2],
    delta_time: f32,
    _padding: f32,
}

pub fn generate_matrix(aspect_ratio: f32) -> glam::Mat4 {
    let projection = glam::Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 1.0, 10.0);
    let view = glam::Mat4::look_at_rh(
        glam::Vec3::new(1.5f32, -5.0, 3.0),
        glam::Vec3::ZERO,
        glam::Vec3::Z,
    );
    projection * view
}

// create a side camera view orhtographic matrix
pub fn generate_orthographic_matrix(aspect_ratio: f32) -> glam::Mat4 {
    let projection = glam::Mat4::orthographic_rh(-1.0, 1.0, -1.0, 1.0, 1.0, 10.0);
    let view = glam::Mat4::look_at_rh(
        glam::Vec3::new(1.5f32, -5.0, 3.0),
        glam::Vec3::ZERO,
        glam::Vec3::Z,
    );
    projection * view
}

impl<'a> Pass<'a> for World {
    fn pass(
        data: &mut HashTypeId2Data,
        mut render_pass: wgpu::RenderPass<'a>,
    ) -> wgpu::RenderPass<'a> {
        let world = get_res::<World>(data);
        let uniforms_bind_group = world.uniforms_bind_group.as_ref().unwrap();
        render_pass.set_bind_group(0, uniforms_bind_group, &[]);
        render_pass
    }
}

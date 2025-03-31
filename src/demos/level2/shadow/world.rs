use std::f32::consts;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupLayoutEntry, ShaderStages,
};

use crate::{
    demos::update_scene::world::generate_matrix,
    shared::ready_paint::{return_res, Ready},
};

#[derive(Default)]
pub struct World {
    pub uniforms_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub uniforms_bind_group: Option<wgpu::BindGroup>,
    pub uniform_buffer: Option<wgpu::Buffer>,
}
pub fn camera(aspect_ratio: f32) -> glam::Mat4 {
    let projection = glam::Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 0.1, 100.0);
    let view = glam::Mat4::look_at_rh(
        glam::Vec3::new(-3.0, 3.0, -3.0),
        glam::Vec3::ZERO,
        glam::Vec3::Y,
    );
    projection * view
}
impl Ready for World {
    fn ready(
        &mut self,
        data: &mut crate::shared::ready_paint::HashTypeId2Data,
        gfx: &crate::shared::ready_paint::Gfx,
    ) {
        let world = camera(800. / 600.);
        let uniforms = Uniforms {
            resolution: [800., 600.],
            matrix: world.to_cols_array(),
            delta_time: 0.,
            _padding: 0.,
        };
        // 创建主视角缓冲区
        let uniform_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("uniform buffer"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniforms_bind_group_layout =
            gfx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("uniforms_bind_group_layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
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
            Self {
                uniforms_bind_group_layout: Some(uniforms_bind_group_layout),
                uniforms_bind_group: Some(uniforms_bind_group),
                uniform_buffer: Some(uniform_buffer),
            },
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

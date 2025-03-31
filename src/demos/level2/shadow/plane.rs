use wgpu::util::DeviceExt;

use crate::shared::ready_paint::{return_res, Gfx, HashTypeId2Data, Ready};

#[derive(Default)]
pub struct ShadowPlane {
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
    pub vertices: [[f32; 8]; 4],
    pub indices: [u16; 6],
    pub index_count: u32,
}

impl Ready for ShadowPlane {
    fn ready(&mut self, data: &mut HashTypeId2Data, gfx: &Gfx) {
        // 创建一个简单的平面顶点数据
        // position(x,y,z), normal(x,y,z), texcoord(u,v)
        let h = -1.;
        let vertices = [
            [5.0, h, -5.0, 0.0, 1.0, 0.0, 1.0, 0.0], // 5, -5     2        0
            [-5.0, h, 5.0, 0.0, 1.0, 0.0, 0.0, 1.0], // -5, 5
            [-5.0, h, -5.0, 0.0, 1.0, 0.0, 0.0, 0.0], // -5, -5
            [5.0, h, 5.0, 0.0, 1.0, 0.0, 1.0, 1.0],  // 5, 5      1        3
        ];

        let indices = [3, 2, 0, 1, 2, 3];

        let vertex_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Ground Plane Vertices"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Ground Plane Indices"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        return_res(
            data,
            Self {
                vertex_buffer: Some(vertex_buffer),
                index_buffer: Some(index_buffer),
                index_count: indices.len() as u32,
                vertices,
                indices,
            },
        );
    }
}

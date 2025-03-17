use std::sync::Arc;

use wgpu::Surface;

use crate::shared::ready_paint::{Gfx, HashTypeId2Data, Paint, Pass, Update};

use super::{object::Tetrahedron, world::World};
pub struct PaintScene;

impl Paint for PaintScene {
    fn paint(data: &mut HashTypeId2Data, gfx: &Gfx, dt: f32, surface: &Arc<Surface<'static>>) {
        let mut encoder = gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let frame = surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let _ = World::update(data, gfx, dt, surface);
        let _ = Tetrahedron::update(data, gfx, dt, surface);
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let mut rpass = World::pass(data, rpass);
            let _ = Tetrahedron::pass(data, rpass);
        }
        gfx.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

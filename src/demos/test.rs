use std::sync::Arc;

use crate::{console_log, shared::Shared, utils::split};

pub fn test(shared: Arc<Shared>) {
    console_log!("Demo test is ready");
    let (queue, frame, mut encoder) = split(&"canvas-1".to_string(), shared.clone());
    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    {
        let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }
    queue.submit(std::iter::once(encoder.finish()));
    frame.present();
}

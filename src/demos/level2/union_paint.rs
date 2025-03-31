use std::sync::Arc;

use crate::{
    console_log,
    shared::ready_paint::{get_res, Paint, Pass, Update},
};

use super::{
    shadow::{depth_texture::DepthTexture, ShadowScene},
    Surfaces,
};

pub struct PaintLevel2 {}

impl Paint for PaintLevel2 {
    fn paint(
        data: &mut crate::shared::ready_paint::HashTypeId2Data,
        gfx: &crate::shared::ready_paint::Gfx,
        dt: f32,
        surface: &std::sync::Arc<wgpu::Surface<'static>>,
    ) {
        let _ = ShadowScene::update(data, gfx, dt, surface);

        let surfaces = get_res::<Surfaces>(data);
        let shadow_suface = surfaces.shadow_surface.clone();
        let ray_tracing_surface = surfaces.ray_tracing_surface.clone();

        let shadow_frame = shadow_suface.get_current_texture().unwrap();

        let mut encoder = gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let shadow_scene_view = shadow_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let first_pass_thing = get_res::<DepthTexture>(data);
        let depth_thing = get_res::<DepthTexture>(data);
        // draw shadow
        {
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("depth test Render Pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &first_pass_thing.shadow_view.as_ref().unwrap(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let _ = DepthTexture::pass(data, _render_pass);
        }

        let main_depth_view = get_res::<ShadowScene>(data)
            .main_depth_view
            .as_ref()
            .unwrap();
        {
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &shadow_scene_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &main_depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let _ = ShadowScene::pass(data, _render_pass);
        }
        gfx.queue.submit(Some(encoder.finish()));
        shadow_frame.present();
    }
}

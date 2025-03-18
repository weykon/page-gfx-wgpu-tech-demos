use std::sync::Arc;

use wgpu::Surface;

use crate::shared::ready_paint::{
    get_res, get_res_mut, return_res, Gfx, HashTypeId2Data, Paint, Pass, Update,
};

use super::{
    object::Tetrahedron,
    triangle_list_render::{NormalTriangleListRender, TriangleListRender},
    world::World,
};
pub struct PaintScene {
    side_surface: Arc<Surface<'static>>,
    triangle_list_surface: Arc<Surface<'static>>,
    triangle_list_normal_surface: Arc<Surface<'static>>,
}
impl PaintScene {
    pub fn merge_surface(
        data: &mut HashTypeId2Data,
        side_surface: &Arc<Surface<'static>>,
        triangle_list_render: &Arc<Surface<'static>>,
        triangle_list_depth_surface: &Arc<Surface<'static>>,
    ) {
        return_res(
            data,
            Self {
                side_surface: side_surface.clone(),
                triangle_list_surface: triangle_list_render.clone(),
                triangle_list_normal_surface: triangle_list_depth_surface.clone(),
            },
        );
    }
}
impl Paint for PaintScene {
    fn paint(data: &mut HashTypeId2Data, gfx: &Gfx, dt: f32, surface: &Arc<Surface<'static>>) {
        let mut encoder = gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // surface
        let side_surface = get_res::<Self>(data).side_surface.clone();
        let triangle_list_surface = get_res::<Self>(data).triangle_list_surface.clone();
        let triangle_list_normal_surface =
            get_res::<Self>(data).triangle_list_normal_surface.clone();
        // surface ----

        // frame
        let frame = surface.get_current_texture().unwrap();
        let side_frame = side_surface.get_current_texture().unwrap();
        let triangle_list_frame = triangle_list_surface.get_current_texture().unwrap();
        let triangle_list_normal_frame =
            triangle_list_normal_surface.get_current_texture().unwrap();
        // frame ----

        // view
        let main_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let side_view = side_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let triangle_list_view = triangle_list_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let triangle_list_normal_view = triangle_list_normal_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        // view ----

        let _ = World::update(data, gfx, dt, surface);
        let _ = Tetrahedron::update(data, gfx, dt, surface);
        let _ = NormalTriangleListRender::update(data, gfx, dt, surface);

        // 主视角
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &main_view,
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
        // 侧视角
        {
            World::update_side_view_uniforms(data, dt, gfx);
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &side_view,
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
            let mut rpass = World::pass_for_side_view(data, rpass);
            let _ = Tetrahedron::pass(data, rpass);
        }
        // 三角形列表渲染
        // depth
        let triangle_list_depth_texture_view = get_res::<Tetrahedron>(data)
            .triangle_list_depth_texture_view
            .as_ref()
            .unwrap();
        // depth ----
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &triangle_list_view,
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: triangle_list_depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let mut rpass = World::pass(data, rpass);
            let mut rpass = TriangleListRender::pass(data, rpass);
        }
        // 三角形列表+normal渲染
        let triangle_list_normal_texture_view = get_res::<NormalTriangleListRender>(data)
            .triangle_list_normal_texture_view
            .as_ref()
            .unwrap();
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &triangle_list_normal_view,
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: triangle_list_normal_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let mut rpass = World::pass(data, rpass);
            let mut rpass = NormalTriangleListRender::pass(data, rpass);
        }
        gfx.queue.submit(Some(encoder.finish()));
        frame.present();
        side_frame.present();
        triangle_list_frame.present();
        triangle_list_normal_frame.present();
    }
}

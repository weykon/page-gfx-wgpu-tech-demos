use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

use once_cell::sync::Lazy;
use paint::PaintScene;
use shadow::ShadowScene;
use triangle_list_render::NormalTriangleListRender;
use vr::VRScene;
use wasm_bindgen::{prelude::Closure, JsCast};

use crate::{
    console_log,
    shared::{
        ready_paint::{return_res, Gfx, HashTypeId2Data, Queue, Ready, Scene},
        Shared,
    },
    utils::{create_cube, split, split_for_update, CubeMesh, Time},
};

pub struct UpdateScene {}
mod object;
mod paint;
mod shadow;
mod world;
impl Queue for UpdateScene {
    fn introduce(scene: &mut Scene) {
        scene
            .add_ready(world::World::default())
            .add_ready(object::Tetrahedron::default())
            .add_ready(ShadowScene::default())
            .add_ready(NormalTriangleListRender::default())
            .add_ready(ShadowScene::default());
        // .add_ready(VRScene::default());
        scene.add_paint::<PaintScene>();
    }
}

impl UpdateScene {
    pub fn run(shared: Arc<Shared>) {
        console_log!("UpdateScene::run");
        let mut scene = Scene::new("update_scene".to_string());
        let shared_clone = shared.clone();

        // get surfaces
        let (adapter, queue, surface) =
            split_for_update(&"canvas-3".to_string(), shared_clone, 300, 300);
        let (_, _, side_surface) =
            split_for_update(&"canvas-4".to_string(), shared.clone(), 300, 300);
        let (_, _, triangle_list_surface) =
            split_for_update(&"canvas-5".to_string(), shared.clone(), 300, 300);
        let (_, _, triangle_list_normal_surface) =
            split_for_update(&"canvas-6".to_string(), shared.clone(), 300, 300);
        let (_, _, vrscene_surface) =
            split_for_update(&"canvas-7".to_string(), shared.clone(), 300, 300);
        let (_, _, shadow_surface) =
            split_for_update(&"canvas-8".to_string(), shared.clone(), 300, 300);

        let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
        let g = f.clone();

        {
            let default_config = surface.get_default_config(&adapter, 300, 300).unwrap();
            shared.configs.borrow_mut().push(default_config);
            console_log!("Config count: {}", shared.configs.borrow().len());
        }
        UpdateScene::introduce(&mut scene);
        scene.ready(&shared);

        PaintScene::merge_surface(
            &mut scene.res,
            &side_surface,
            &triangle_list_surface,
            &triangle_list_normal_surface,
            &vrscene_surface,
            &shadow_surface, // Add shadow surface
        );
        scene.paint(&shared, 0.016, &surface); // 执行初始渲染

        let mut time = Time::new();
        let last_time = time.performance.now() as f32;
        time.last_frame_time = last_time;
        let mut accumulated_time: f32 = 0.0;
        const TARGET_FPS: f32 = 60.;
        const FRAME_TIME: f32 = 1000.0 / TARGET_FPS;
        // FPS 计算相关变量
        let mut render_frame_count: u32 = 0;
        let mut fps_update_time: f32 = last_time;
        let mut render_fps: f32 = 0.0;

        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let now = time.performance.now() as f32;
            let dt = (now - time.last_frame_time).min(100.0);
            time.last_frame_time = now;
            accumulated_time += dt;
            let mut updated = false;
            while accumulated_time >= FRAME_TIME {
                accumulated_time -= FRAME_TIME;
                updated = true;
            }
            if updated {
                scene.paint(&shared, dt, &surface);
                render_frame_count += 1;
            }
            if now - fps_update_time > 1000.0 {
                let time_diff = (now - fps_update_time) / 1000.0;
                render_fps = render_frame_count as f32 / time_diff;
                render_frame_count = 0;
                fps_update_time = now;
                console_log!(" 渲染FPS: {}", render_fps);
            }

            web_sys::window()
                .unwrap()
                .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                .expect("Failed to request animation frame");
        }) as Box<dyn FnMut()>));

        web_sys::window()
            .unwrap()
            .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .expect("Failed to request animation frame");
    }
}

mod triangle_list_render;
mod vr;

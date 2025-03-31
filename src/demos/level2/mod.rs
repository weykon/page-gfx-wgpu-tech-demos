use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

use shadow::{depth_texture::DepthTexture, plane::ShadowPlane, world::World, ShadowScene};
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast,
};

use crate::{
    console_log,
    shared::{
        ready_paint::{return_res, Queue, Ready, Scene},
        Shared,
    },
    utils::{split_for_update, Time},
};

pub struct NextLevelPage {
    time_wraper: TimeWraper,
}

struct Surfaces {
    shadow_surface: Arc<wgpu::Surface<'static>>,
    ray_tracing_surface: Arc<wgpu::Surface<'static>>,
}

impl NextLevelPage {
    pub fn run(shared: Arc<Shared>, page_2_active: Rc<RefCell<bool>>) {
        console_log!("Running level 2");
        let mut scene = Scene::new("level-page-2".to_string());
        let shared_clone = shared.clone();

        let (adapter, queue, shadow_suface) =
            split_for_update(&"canvas-1", shared.clone(), 800, 600);
        let (_, _, ray_tracing_surface) = split_for_update(&"canvas-2", shared.clone(), 800, 600);

        let mut page = NextLevelPage {
            time_wraper: TimeWraper::new(),
        };

        return_res(
            &mut scene.res,
            Surfaces {
                shadow_surface: shadow_suface.clone(),
                ray_tracing_surface,
            },
        );

        NextLevelPage::introduce(&mut scene);
        console_log!("Ready to run level 2");
        scene.ready(&shared);
        console_log!("Level 2 ready,{:?}", scene.res);

        let update_closure_func: Rc<RefCell<Option<Closure<dyn FnMut()>>>> =
            Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
        let update_closure_func_clone = update_closure_func.clone();
        let wait_browser_boot = |f: Rc<RefCell<Option<Closure<dyn FnMut()>>>>| {
            web_sys::window()
                .unwrap()
                .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                .expect("Failed to request animation frame");
        };
        *update_closure_func_clone.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            if !*page_2_active.clone().borrow() {
                console_log!("Page 2 not active");
                // wait_browser_boot(update_closure_func.clone());
                return;
            }
            page.time_wraper.update_func(Package {
                shared: shared.clone(),
                scene: &mut scene,
                surface: shadow_suface.clone(),
            });
            wait_browser_boot(update_closure_func.clone());
        })
            as Box<dyn FnMut()>));

        wait_browser_boot(update_closure_func_clone.clone());
    }
}
struct TimeWraper {
    time: Time,
    accumulated_time: f32,
    render_frame_count: u32,
    fps_update_time: f32,
    render_fps: f32,
}
impl TimeWraper {
    fn new() -> Self {
        let mut time = Time::new();
        let last_time = time.performance.now() as f32;
        time.last_frame_time = last_time;
        let accumulated_time: f32 = 0.0;
        let render_frame_count: u32 = 0;
        let fps_update_time: f32 = last_time;
        let render_fps: f32 = 0.0;
        TimeWraper {
            time,
            accumulated_time,
            render_frame_count,
            fps_update_time,
            render_fps,
        }
    }
    pub fn update_func<'a>(&mut self, p: Package<'a>) {
        let now = self.time.performance.now() as f32;
        let dt = (now - self.time.last_frame_time).min(100.0);
        self.time.last_frame_time = now;
        self.accumulated_time += dt;
        let mut updated = false;
        while self.accumulated_time >= FRAME_TIME {
            self.accumulated_time -= FRAME_TIME;
            updated = true;
        }
        if updated {
            let Package {
                shared,
                scene,
                surface,
            } = p;
            scene.paint(&shared, dt, &surface);
            self.render_frame_count += 1;
        }
        if now - self.fps_update_time > 1000.0 {
            let time_diff = (now - self.fps_update_time) / 1000.0;
            self.render_fps = self.render_frame_count as f32 / time_diff;
            self.render_frame_count = 0;
            self.fps_update_time = now;
            console_log!("渲染FPS: {}", self.render_fps);
        }
    }
}
const TARGET_FPS: f32 = 60.;
const FRAME_TIME: f32 = 1000.0 / TARGET_FPS;
struct Package<'a> {
    shared: Arc<Shared>,
    scene: &'a mut Scene,
    surface: Arc<wgpu::Surface<'static>>,
}
impl Queue for NextLevelPage {
    fn introduce(scene: &mut Scene) {
        scene
            .add_ready(World::default())
            .add_ready(ShadowPlane::default())
            .add_ready(DepthTexture::default())
            .add_ready(ShadowScene::default());
        scene.add_paint::<union_paint::PaintLevel2>();
    }
}

mod ray_tracing;
mod shadow;
mod union_paint;

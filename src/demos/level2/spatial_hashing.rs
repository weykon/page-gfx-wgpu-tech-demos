use crate::shared::ready_paint::Queue;
pub struct PatialHashScene;
impl Queue for PatialHashScene {
    fn introduce(scene: &mut super::Scene) {
        // scene
        //     .add_ready(scene::Screen::default())
        // scene.add_paint::<paint_scene::PaintScreen>();
    }
}

pub mod entity;
pub mod sapce;
use crate::shared::ready_paint::Ready;


struct Space { 
    pub width: f32,
    pub height: f32,
    pub cell_size: f32,
    pub cells: Vec<Vec<Vec<usize>>>,
}


impl Ready for Space {
    fn ready(&mut self, data: &mut crate::shared::ready_paint::HashTypeId2Data, gfx: &crate::shared::ready_paint::Gfx) {
                
    }
}
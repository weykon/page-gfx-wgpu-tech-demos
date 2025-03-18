use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    sync::Arc,
};

use wgpu::Surface;

use crate::console_log;

use super::Shared;
pub type Gfx = Arc<Shared>;
pub type HashTypeId2Data = HashMap<TypeId, Box<dyn Any>>;
pub trait Ready {
    fn ready(&mut self, data: &mut HashTypeId2Data, gfx: &Gfx);
}
pub trait Paint {
    fn paint(data: &mut HashTypeId2Data, gfx: &Gfx, dt: f32, surface: &Arc<Surface<'static>>);
}
pub trait Update {
    fn update(data: &mut HashTypeId2Data, gfx: &Gfx, dt: f32, surface: &Arc<Surface<'static>>);
}
pub trait Pass<'a> {
    fn pass(data: &mut HashTypeId2Data, render_pass: wgpu::RenderPass<'a>) -> wgpu::RenderPass<'a>;
}
pub trait Queue {
    fn introduce(scene: &mut Scene);
}

pub fn get_res<T: Any + 'static>(data: &HashTypeId2Data) -> &T {
    data.get(&TypeId::of::<T>())
        .and_then(|data| data.downcast_ref::<T>())
        .expect(&format!(
            "Failed to get resource of type: {}",
            std::any::type_name::<T>()
        ))
}
pub fn get_res_mut<T: Any + 'static>(data: &mut HashTypeId2Data) -> &mut T {
    data.get_mut(&TypeId::of::<T>())
        .and_then(|data| data.downcast_mut::<T>())
        .expect(&format!(
            "Failed to get mutable resource of type: {}",
            std::any::type_name::<T>()
        ))
}
pub fn get_ref_and_mut<Ref: Any + 'static, Mut: Any + 'static>(
    data: &mut HashTypeId2Data,
) -> (&Ref, &mut Mut) {
    assert_ne!(
        TypeId::of::<Ref>(),
        TypeId::of::<Mut>(),
        "Ref and Mut should not be the same type"
    );
    unsafe {
        let data_ptr = data as *mut HashTypeId2Data;
        let t1 = (&*data_ptr)
            .get(&TypeId::of::<Ref>())
            .and_then(|r| r.downcast_ref::<Ref>())
            .expect(&format!(
                "Failed to get resource of type: {}",
                std::any::type_name::<Ref>()
            ));
        let t2 = (&mut *data_ptr)
            .get_mut(&TypeId::of::<Mut>())
            .and_then(|d| d.downcast_mut::<Mut>())
            .expect(&format!(
                "Failed to get mutable resource of type: {}",
                std::any::type_name::<Mut>()
            ));
        (t1, t2)
    }
}

/// create a new box data of type in hashmap (directly cover)
pub fn return_res<T: Any + 'static>(data: &mut HashMap<TypeId, Box<dyn Any>>, new_data: T) {
    data.insert(TypeId::of::<T>(), Box::new(new_data));
}

pub struct Scene {
    name: String,
    pub res: HashMap<TypeId, Box<dyn Any>>,
    readys: Vec<TypeId>,
    paints: Vec<TypeId>,
    readys_hashmap: HashMap<TypeId, Box<dyn FnMut(&mut HashMap<TypeId, Box<dyn Any>>, &Gfx)>>,
    paints_hashmap: HashMap<
        TypeId,
        Box<dyn Fn(&mut HashMap<TypeId, Box<dyn Any>>, &Gfx, f32, &Arc<Surface<'static>>)>,
    >,
}

impl Scene {
    pub fn new(name: String) -> Self {
        console_log!("Scene::new");
        Scene {
            name,
            res: HashMap::new(),
            readys: Vec::new(),
            paints: Vec::new(),
            readys_hashmap: HashMap::new(),
            paints_hashmap: HashMap::new(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn add_ready<T: Ready + Default + 'static>(&mut self, mut ready_res: T) -> &mut Self {
        let type_id = TypeId::of::<T>();
        self.readys.push(type_id);
        self.res.insert(type_id, Box::new(T::default()));
        self.readys_hashmap.insert(
            type_id,
            Box::new(move |data, gfx| {
                ready_res.ready(data, gfx);
            }),
        );
        self
    }

    pub fn add_paint<T: Paint + 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.paints.push(type_id);
        self.paints_hashmap.insert(type_id, Box::new(T::paint));
    }

    pub fn ready(&mut self, gfx: &Gfx) {
        println!("<Scene>::ready");
        for ready_type_id in self.readys.iter() {
            if let Some(ready_fn) = self.readys_hashmap.get_mut(ready_type_id) {
                ready_fn(&mut self.res, gfx);
            }
        }
    }

    pub fn paint(&mut self, gfx: &Gfx, dt: f32, surface: &Arc<Surface<'static>>) {
        for paint_type_id in self.paints.iter() {
            if let Some(paint_fn) = self.paints_hashmap.get_mut(paint_type_id) {
                paint_fn(&mut self.res, gfx, dt, surface);
            }
        }
    }
}

pub trait InRefOrMut {
    type AccessMode;
    type Output;
}

pub fn refs_muts<T: TurpleAccess>(data: &mut HashTypeId2Data) -> T::Output<'_> {
    T::accesss(data)
}
pub trait AccessMode {}
pub struct Read;
pub struct Write;
impl AccessMode for Read {}
impl AccessMode for Write {}
pub trait RefOrMut {
    type Target: 'static;
    type Mode: AccessMode;
    type Output<'a>: 'a
    where
        Self::Target: 'a;
    fn process<'a>(data: &'a mut HashTypeId2Data) -> Self::Output<'a>;
}
pub struct Ref<T>(PhantomData<T>);
impl<T: 'static + Ready> RefOrMut for Ref<T> {
    type Target = T;
    type Mode = Read;
    type Output<'a> = &'a T;

    fn process<'a>(data: &'a mut HashTypeId2Data) -> Self::Output<'a> {
        data.get(&TypeId::of::<T>())
            .and_then(|r| r.downcast_ref::<T>())
            .expect(&format!(
                "Failed to get resource of type: {}",
                std::any::type_name::<T>()
            ))
    }
}

pub struct Mut<T>(PhantomData<T>);
impl<T: 'static + Ready> RefOrMut for Mut<T> {
    type Target = T;
    type Mode = Write;
    type Output<'a> = &'a mut T;

    fn process<'a>(data: &'a mut HashTypeId2Data) -> Self::Output<'a> {
        data.get_mut(&TypeId::of::<T>())
            .and_then(|m| m.downcast_mut::<T>())
            .expect(&format!(
                "Failed to get mutable resource of type: {}",
                std::any::type_name::<T>()
            ))
    }
}
pub trait TurpleAccess {
    type Output<'a>;
    fn accesss<'a>(data: &'a mut HashTypeId2Data) -> Self::Output<'a>;
}
impl<T1, T2> TurpleAccess for (T1, T2)
where
    T1: RefOrMut + 'static,
    T2: RefOrMut + 'static,
{
    type Output<'a> = (T1::Output<'a>, T2::Output<'a>);

    fn accesss<'a>(data: &'a mut HashTypeId2Data) -> Self::Output<'a> {
        assert_ne!(TypeId::of::<T1>(), TypeId::of::<T2>());
        unsafe {
            // 使用原始指针来绕过借用检查
            let data_ptr = data as *mut HashTypeId2Data;
            (T1::process(&mut *data_ptr), T2::process(&mut *data_ptr))
        }
    }
}

impl<T1, T2, T3> TurpleAccess for (T1, T2, T3)
where
    T1: RefOrMut + 'static,
    T2: RefOrMut + 'static,
    T3: RefOrMut + 'static,
{
    type Output<'a> = (T1::Output<'a>, T2::Output<'a>, T3::Output<'a>);

    fn accesss<'a>(data: &'a mut HashTypeId2Data) -> Self::Output<'a> {
        assert_ne!(TypeId::of::<T1>(), TypeId::of::<T2>());
        assert_ne!(TypeId::of::<T1>(), TypeId::of::<T3>());
        assert_ne!(TypeId::of::<T2>(), TypeId::of::<T3>());
        unsafe {
            let data_ptr = data as *mut HashTypeId2Data;
            (
                T1::process(&mut *data_ptr),
                T2::process(&mut *data_ptr),
                T3::process(&mut *data_ptr),
            )
        }
    }
}

impl<T1, T2, T3, T4> TurpleAccess for (T1, T2, T3, T4)
where
    T1: RefOrMut + 'static,
    T2: RefOrMut + 'static,
    T3: RefOrMut + 'static,
    T4: RefOrMut + 'static,
{
    type Output<'a> = (
        T1::Output<'a>,
        T2::Output<'a>,
        T3::Output<'a>,
        T4::Output<'a>,
    );

    fn accesss<'a>(data: &'a mut HashTypeId2Data) -> Self::Output<'a> {
        assert_ne!(TypeId::of::<T1>(), TypeId::of::<T2>());
        assert_ne!(TypeId::of::<T1>(), TypeId::of::<T3>());
        assert_ne!(TypeId::of::<T1>(), TypeId::of::<T4>());
        assert_ne!(TypeId::of::<T2>(), TypeId::of::<T3>());
        assert_ne!(TypeId::of::<T2>(), TypeId::of::<T4>());
        assert_ne!(TypeId::of::<T3>(), TypeId::of::<T4>());
        unsafe {
            let data_ptr = data as *mut HashTypeId2Data;
            (
                T1::process(&mut *data_ptr),
                T2::process(&mut *data_ptr),
                T3::process(&mut *data_ptr),
                T4::process(&mut *data_ptr),
            )
        }
    }
}

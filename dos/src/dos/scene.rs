use raylib::camera::Camera3D;
use raylib::math::Quaternion;
use raylib::models::Model;
use raylib::prelude::RaylibMode3DExt;
pub use raylib::prelude::{Color, Vector3};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GObject {
    pub model_name: Arc<str>,
    pub position: Vector3,
    pub rotation: Quaternion,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GLight {
    pub pos: Vector3,
    pub color: Color,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct GLightId {
    id: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct GObjectId {
    id: u32,
}

impl GObjectId {
    pub fn invalid() -> Self {
        Self { id: 0 }
    }
    pub fn get(&self) -> u32 {
        self.id
    }
    pub fn is_valid(&self) -> bool {
        self.id != 0
    }
}
impl GLightId {
    pub fn invalid() -> Self {
        Self { id: 0 }
    }
    pub fn get(&self) -> u32 {
        self.id
    }
    pub fn is_valid(&self) -> bool {
        self.id != 0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Scene {
    pub cam_pos: Vector3,
    pub cam_rot: Quaternion,
    pub objects: BTreeMap<GObjectId, GObject>,
    pub lights: BTreeMap<GLightId, GLight>,
}
impl Scene {
    pub fn new() -> Self {
        Self {
            cam_pos: Vector3::zero(),
            cam_rot: Quaternion::identity(),
            objects: BTreeMap::new(),
            lights: BTreeMap::new(),
        }
    }

    pub fn create_object(&mut self, object: GObject) -> GObjectId {
        for i in 1..u32::MAX {
            let id = GObjectId { id: i };
            if !self.objects.contains_key(&id) {
                self.objects.insert(id, object);
                return id;
            }
        }
        GObjectId { id: 0 }
    }

    pub fn destroy_object(&mut self, id: GObjectId) {
        self.objects.remove(&id);
    }

    pub fn get_object(&self, id: GObjectId) -> Option<&GObject> {
        self.objects.get(&id)
    }

    pub fn set_object(&mut self, id: GObjectId, object: GObject) -> Option<GObject> {
        if !self.objects.contains_key(&id) {
            return Some(object);
        }
        self.objects.insert(id, object);
        None
    }

    pub fn get_object_mut(&mut self, id: GObjectId) -> Option<&mut GObject> {
        self.objects.get_mut(&id)
    }
    pub fn get_object_clone(&mut self, id: GObjectId) -> Option<GObject> {
        self.objects.get_mut(&id).map(|i| i.clone())
    }

    pub fn create_light(&mut self, light: GLight) -> GLightId {
        for i in 1..u32::MAX {
            let id = GLightId { id: i };
            if !self.lights.contains_key(&id) {
                self.lights.insert(id, light);
                return id;
            }
        }
        GLightId { id: 0 }
    }

    pub fn destroy_light(&mut self, id: GObjectId) {
        self.objects.remove(&id);
    }

    pub fn get_light(&self, id: GLightId) -> Option<&GLight> {
        self.lights.get(&id)
    }

    pub fn get_light_clone(&self, id: GLightId) -> Option<GLight> {
        self.lights.get(&id).map(|i| i.clone())
    }

    pub fn set_light(&mut self, id: GLightId, light: GLight) -> Option<GLight> {
        if !self.lights.contains_key(&id) {
            return Some(light);
        }
        self.lights.insert(id, light);
        None
    }

    pub fn get_light_mut(&mut self, id: GLightId) -> Option<&mut GLight> {
        self.lights.get_mut(&id)
    }
}

pub struct SceneRenderer {
    pub loaded_meshes: HashMap<Arc<str>, Model>,
}
impl SceneRenderer{
    pub fn new()->Self{
        Self{
            loaded_meshes:HashMap::new(),
        }
    }

    pub fn render(&mut self,scene:&Scene, handle:&mut raylib::prelude::RaylibDrawHandle,_thread:raylib::prelude::RaylibThread){
        let cam = Camera3D::perspective(scene.cam_pos, Vector3::forward().transform_with(scene.cam_rot.to_matrix()), Vector3::up().transform_with(scene.cam_rot.to_matrix()), 110.0);
        let mut draw= handle.begin_mode3D(cam);
        
    }
}


use raylib::camera::Camera3D;
use raylib::ffi::RenderTexture2D;
use raylib::math::{BoundingBox, Quaternion};
use raylib::models::{Model, RaylibModel};
pub use raylib::prelude::{Color, Vector3};
use raylib::prelude::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::shaders::Shader;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GObject {
    pub model_name: Arc<str>,
    pub position: Vector3,
    pub rotation: Quaternion,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GLight {
    pub pos: Vector3,
    pub color: Color,
    pub direction: Vector3,
    pub fov: f32,
    pub casts_shadows: bool,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
            cam_rot: Quaternion::new(1.0, 0.0, 0.0, 0.0),
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
    pub shadow_map_texture: Vec<RenderTexture2D>,
    pub to_load: HashSet<Arc<str>>,
    pub shader: Option<Shader>,
}
impl SceneRenderer {
    pub fn new() -> Self {
        Self {
            loaded_meshes: HashMap::new(),
            shadow_map_texture: Vec::new(),
            to_load: HashSet::new(),
            shader: None,
        }
    }

    pub fn render(
        &mut self,
        scene: &Scene,
        handle: &mut impl RaylibDraw,
        _thread: &raylib::prelude::RaylibThread,
    ) {
        handle.clear_background(Color::BLACK);
        let cam = Camera3D::perspective(
            scene.cam_pos,
            Vector3::forward().transform_with(scene.cam_rot.to_matrix()),
            Vector3::up().transform_with(scene.cam_rot.to_matrix()),
            90.0,
        );
        let mut draw = handle.begin_mode3D(cam);
        unsafe {
            raylib::ffi::rlSetClipPlanes(0.01, 1000.0);
        }
        let mut to_load = HashSet::new();
        for (_, obj) in &scene.objects {
            if !self.loaded_meshes.contains_key(&obj.model_name) {
                to_load.insert(obj.model_name.clone());
                continue;
            }
            let mesh = self.loaded_meshes.get_mut(&obj.model_name).unwrap();
            mesh.transform = obj.rotation.to_matrix().into();
            /*  let bx = BoundingBox {
                min: Vector3 {
                    x: -1.,
                    y: -1.,
                    z: -1.,
                },
                max: Vector3 {
                    x: 1.,
                    y: 1.,
                    z: 1.,
                },
            };*/
            draw.draw_model(mesh, obj.position, 1.0, Color::WHITE);
        }
        self.to_load = to_load;
    }
}

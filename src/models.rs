use std::f32::consts::PI;

use egui::ahash::HashMap;
use image::{ImageBuffer, ImageReader, Rgba, RgbaImage};
use itertools::izip;
use nalgebra::{Matrix4, Point3, Vector3};
use tobj::{Material, Mesh};

use crate::renderer::VertexData;

#[derive(Debug, Clone)]
pub struct Camera {
    pub aspect_ratio: f32,
    fovy: f32,
    near_bound: f32,
    far_bound: f32,
    position: Point3<f32>,
    rotation: Vector3<f32>,
    look_at: Point3<f32>,
}
impl Camera {
    pub fn new(aspect_ratio: f32) -> Self {
        Self {
            aspect_ratio,
            fovy: 1.4,
            near_bound: 0.1,
            far_bound: 1000.0,
            position: Point3::new(0.0, 0.0, 2.0),
            rotation: Vector3::default(),
            look_at: Point3::new(0.0, 0.0, 0.0),
        }
    }
    pub fn rotate(&mut self, pitch: f32, yaw: f32) {
        let new_rotation = self.rotation + Vector3::new(pitch, yaw, 0.0);
        self.rotation.x = nalgebra::clamp(new_rotation.x, -PI * 89.0 / 180.0, PI * 89.0 / 180.0);
        self.rotation.y = new_rotation.y
    }
    pub fn tm(&self) -> Matrix4<f32> {
        let tm_x = Matrix4::new_rotation(Vector3::new(self.rotation.x, 0.0, 0.0));
        let tm_y = Matrix4::new_rotation(Vector3::new(0.0, self.rotation.y, 0.0));
        let position = (tm_y * tm_x).transform_point(&self.position);
        let transform_matrix =
            Matrix4::look_at_rh(&position, &self.look_at, &Vector3::new(0.0, 1.0, 0.0));
        let perspective_matrix = Matrix4::new_perspective(
            self.aspect_ratio,
            self.fovy,
            self.near_bound,
            self.far_bound,
        );
        perspective_matrix * transform_matrix
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub translation: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub scaling: Vector3<f32>,
}
impl Model {
    pub fn new(
        obj_path: &str,
        (initial_position, initial_rotation, initial_scaling): (
            Vector3<f32>,
            Vector3<f32>,
            Vector3<f32>,
        ),
    ) -> Self {
        let (models, materials_result) = tobj::load_obj(
            obj_path,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ..Default::default()
            },
        )
        .unwrap();
        let materials = materials_result.unwrap();

        Self {
            meshes: models.into_iter().map(|model| model.mesh).collect(),
            materials,
            translation: initial_position,
            rotation: initial_rotation,
            scaling: initial_scaling,
        }
    }
    pub fn tm(&self) -> Matrix4<f32> {
        Matrix4::new_rotation(self.rotation)
            .append_nonuniform_scaling(&self.scaling)
            .prepend_translation(&self.translation)
    }
    pub fn vertex_data(&self, model_idx: usize) -> Vec<Vec<VertexData>> {
        let mut vertex_data = vec![];
        for mesh in &self.meshes {
            let raw_positions = &mesh.positions;
            let positions = raw_positions.chunks_exact(3).clone();
            let raw_normals = &mesh.normals;
            let r: Vec<f32> = vec![0.0; raw_positions.len()];
            let normals = if raw_normals.is_empty() {
                r.chunks_exact(3).clone()
            } else {
                raw_normals.chunks_exact(3).clone()
            };
            let uvs = mesh.texcoords.chunks_exact(2).clone();
            vertex_data.push(
                izip!(positions, normals, uvs)
                    .map(|(position, normal, uv)| VertexData {
                        position: position.try_into().unwrap(),
                        normal: normal.try_into().unwrap(),
                        uv: [uv[0], 1.0 - uv[1]],
                        model_idx: model_idx as u32,
                    })
                    .collect::<Vec<_>>(),
            )
        }
        vertex_data
    }
    pub fn debugg(&self) {
        let vertex_data = &self.vertex_data(0)[0];
        for (i, vertex) in vertex_data.iter().enumerate() {
            println!(
                "{:?}: {:?}\t\t{:?}\t\t{:?}",
                i, &vertex.position, vertex.normal, vertex.uv
            );
        }
        println!("");
        for (i, chunk) in self.meshes[0].indices.chunks_exact(3).enumerate() {
            println!("{:?}: {:?}", i, chunk);
        }
    }
}

trait MaterialExt {
    fn texture_data<'a>(
        &self,
        textures_map: &'a HashMap<String, RgbaImage>,
    ) -> Option<&'a RgbaImage>;
}
impl MaterialExt for Material {
    fn texture_data<'a>(
        &self,
        textures_map: &'a HashMap<String, RgbaImage>,
    ) -> Option<&'a RgbaImage> {
        self.diffuse_texture
            .as_ref()
            .map(|dt_name| &textures_map[dt_name])
    }
}

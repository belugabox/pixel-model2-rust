//! Traitement de la géométrie 3D

use glam::{Vec3, Vec4, Mat4};
use anyhow::Result;

/// Triangle 3D avec tous les attributs
#[derive(Debug, Clone)]
pub struct Triangle3D {
    pub vertices: [Vertex3D; 3],
    pub texture_id: Option<u32>,
    pub material_id: u32,
}

/// Vertex 3D complet
#[derive(Debug, Clone, Copy)]
pub struct Vertex3D {
    pub position: Vec3,
    pub normal: Vec3,
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
}

/// Processeur de géométrie 3D
pub struct GeometryProcessor {
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub model_matrix: Mat4,
}

impl GeometryProcessor {
    pub fn new() -> Self {
        Self {
            view_matrix: Mat4::IDENTITY,
            projection_matrix: Mat4::IDENTITY,
            model_matrix: Mat4::IDENTITY,
        }
    }
    
    pub fn set_view_matrix(&mut self, matrix: Mat4) {
        self.view_matrix = matrix;
    }
    
    pub fn set_projection_matrix(&mut self, matrix: Mat4) {
        self.projection_matrix = matrix;
    }
    
    pub fn transform_triangle(&self, triangle: &Triangle3D) -> Result<Triangle3D> {
        let mvp_matrix = self.projection_matrix * self.view_matrix * self.model_matrix;
        
        let mut transformed = triangle.clone();
        for vertex in &mut transformed.vertices {
            let pos = mvp_matrix * Vec4::new(vertex.position.x, vertex.position.y, vertex.position.z, 1.0);
            vertex.position = Vec3::new(pos.x / pos.w, pos.y / pos.w, pos.z / pos.w);
        }
        
        Ok(transformed)
    }
}

impl Default for GeometryProcessor {
    fn default() -> Self {
        Self::new()
    }
}
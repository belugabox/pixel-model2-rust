//! Pipeline de géométrie 3D SEGA Model 2
//!
//! Implémente le système de transformation et rendu 3D authentique du SEGA Model 2,
//! incluant les matrices de transformation, projection et clipping optimisés.

use anyhow::Result;
use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};

/// Triangle 3D avec tous les attributs Model 2
#[derive(Debug, Clone)]
pub struct Triangle3D {
    pub vertices: [Vertex3D; 3],
    pub texture_id: Option<u32>,
    pub material_id: u32,
    pub flags: TriangleFlags,
}

/// Vertex 3D complet avec attributs SEGA
#[derive(Debug, Clone, Copy)]
pub struct Vertex3D {
    pub position: Vec3,
    pub normal: Vec3,
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
    pub fog_coord: f32,
    pub specular: [f32; 3],
}

/// Flags de rendu pour triangles
#[derive(Debug, Clone, Copy)]
pub struct TriangleFlags {
    pub transparent: bool,
    pub two_sided: bool,
    pub no_culling: bool,
    pub wireframe: bool,
    pub flat_shading: bool,
    pub texture_filtering: bool,
}

/// Modèle 3D complet avec LOD
#[derive(Debug, Clone)]
pub struct Model3D {
    pub name: String,
    pub triangles: Vec<Triangle3D>,
    pub bounding_box: BoundingBox,
    pub lod_levels: Vec<LodLevel>,
    pub animation_data: Option<AnimationData>,
}

/// Niveau de détail (LOD)
#[derive(Debug, Clone)]
pub struct LodLevel {
    pub distance: f32,
    pub triangle_indices: Vec<usize>,
    pub vertex_count: usize,
}

/// Boîte englobante pour culling
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

/// Données d'animation (futur)
#[derive(Debug, Clone)]
pub struct AnimationData {
    pub frames: Vec<AnimationFrame>,
    pub frame_rate: f32,
}

/// Frame d'animation
#[derive(Debug, Clone)]
pub struct AnimationFrame {
    pub vertices: Vec<Vec3>,
    pub timestamp: f32,
}

/// Processeur de géométrie 3D SEGA Model 2
pub struct GeometryProcessor {
    // Matrices de transformation
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub model_matrix: Mat4,
    pub viewport_matrix: Mat4,

    // Configuration de la caméra
    pub camera_position: Vec3,
    pub camera_target: Vec3,
    pub camera_up: Vec3,
    pub field_of_view: f32,
    pub aspect_ratio: f32,
    pub near_plane: f32,
    pub far_plane: f32,

    // Cache des matrices combinées
    view_projection_cache: Option<Mat4>,
    mvp_cache: Option<Mat4>,
    normal_matrix_cache: Option<Mat4>,

    // Paramètres de rendu
    pub frustum_culling: bool,
    pub backface_culling: bool,
    pub fog_enabled: bool,
    pub fog_start: f32,
    pub fog_end: f32,
    pub fog_color: [f32; 4],
}

/// Triangle transformé en clip space
#[derive(Debug, Clone)]
pub struct TransformedTriangle {
    pub vertices: [TransformedVertex; 3],
    pub texture_id: Option<u32>,
    pub material_id: u32,
    pub flags: TriangleFlags,
}

/// Vertex transformé en clip space
#[derive(Debug, Clone, Copy)]
pub struct TransformedVertex {
    pub clip_position: Vec4,
    pub world_position: Vec3,
    pub world_normal: Vec3,
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
    pub specular: [f32; 3],
    pub fog_factor: f32,
}

/// Triangle en coordonnées écran
#[derive(Debug, Clone)]
pub struct ScreenTriangle {
    pub vertices: [ScreenVertex; 3],
    pub texture_id: Option<u32>,
    pub material_id: u32,
    pub flags: TriangleFlags,
}

/// Vertex en coordonnées écran
#[derive(Debug, Clone, Copy)]
pub struct ScreenVertex {
    pub position: Vec3, // x, y en pixels, z en depth
    pub world_position: Vec3,
    pub world_normal: Vec3,
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
    pub specular: [f32; 3],
    pub fog_factor: f32,
    pub depth: f32, // Pour Z-buffer
}

impl Default for TriangleFlags {
    fn default() -> Self {
        Self {
            transparent: false,
            two_sided: false,
            no_culling: false,
            wireframe: false,
            flat_shading: false,
            texture_filtering: true,
        }
    }
}

impl Default for TransformedVertex {
    fn default() -> Self {
        Self {
            clip_position: Vec4::ZERO,
            world_position: Vec3::ZERO,
            world_normal: Vec3::Y,
            tex_coords: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            specular: [0.0, 0.0, 0.0],
            fog_factor: 0.0,
        }
    }
}

impl Default for ScreenVertex {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            world_position: Vec3::ZERO,
            world_normal: Vec3::Y,
            tex_coords: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            specular: [0.0, 0.0, 0.0],
            fog_factor: 0.0,
            depth: 0.0,
        }
    }
}

impl Default for Vertex3D {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            normal: Vec3::Y,
            tex_coords: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            fog_coord: 0.0,
            specular: [0.0, 0.0, 0.0],
        }
    }
}

impl BoundingBox {
    /// Crée une bounding box vide
    pub fn empty() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }

    /// Étend la bounding box pour inclure un point
    pub fn expand(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    /// Teste l'intersection avec le frustum (simplifié)
    pub fn intersects_frustum(&self, mvp_matrix: &Mat4) -> bool {
        // Test basique - transforme les coins et teste s'ils sont dans le frustum
        let corners = [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ];

        for corner in corners {
            let clip_pos = *mvp_matrix * Vec4::new(corner.x, corner.y, corner.z, 1.0);
            if clip_pos.x >= -clip_pos.w
                && clip_pos.x <= clip_pos.w
                && clip_pos.y >= -clip_pos.w
                && clip_pos.y <= clip_pos.w
                && clip_pos.z >= 0.0
                && clip_pos.z <= clip_pos.w
            {
                return true;
            }
        }

        false
    }
}

impl GeometryProcessor {
    /// Crée un nouveau processeur de géométrie avec configuration SEGA Model 2
    pub fn new(width: u32, height: u32) -> Self {
        let aspect_ratio = width as f32 / height as f32;
        let fov = 45.0_f32.to_radians();
        let near = 0.1;
        let far = 1000.0;

        let camera_position = Vec3::new(0.0, 0.0, 5.0);
        let camera_target = Vec3::ZERO;
        let camera_up = Vec3::Y;

        // Matrice de vue (look-at)
        let view_matrix = Mat4::look_at_rh(camera_position, camera_target, camera_up);

        // Matrice de projection perspective
        let projection_matrix = Mat4::perspective_rh(fov, aspect_ratio, near, far);

        // Matrice viewport (NDC vers coordonnées écran)
        let viewport_matrix =
            Mat4::from_translation(Vec3::new(width as f32 / 2.0, height as f32 / 2.0, 0.0))
                * Mat4::from_scale(Vec3::new(width as f32 / 2.0, -(height as f32) / 2.0, 1.0));

        Self {
            view_matrix,
            projection_matrix,
            model_matrix: Mat4::IDENTITY,
            viewport_matrix,
            camera_position,
            camera_target,
            camera_up,
            field_of_view: fov,
            aspect_ratio,
            near_plane: near,
            far_plane: far,
            view_projection_cache: None,
            mvp_cache: None,
            normal_matrix_cache: None,
            frustum_culling: true,
            backface_culling: true,
            fog_enabled: false,
            fog_start: 10.0,
            fog_end: 100.0,
            fog_color: [0.7, 0.7, 0.9, 1.0], // Bleu clair
        }
    }

    /// Configure la caméra avec position, cible et up vector
    pub fn set_camera(&mut self, position: Vec3, target: Vec3, up: Vec3) {
        self.camera_position = position;
        self.camera_target = target;
        self.camera_up = up;
        self.view_matrix = Mat4::look_at_rh(position, target, up);
        self.invalidate_cache();
    }

    /// Configure la projection perspective
    pub fn set_perspective(&mut self, fov: f32, aspect: f32, near: f32, far: f32) {
        self.field_of_view = fov;
        self.aspect_ratio = aspect;
        self.near_plane = near;
        self.far_plane = far;
        self.projection_matrix = Mat4::perspective_rh(fov, aspect, near, far);
        self.invalidate_cache();
    }

    /// Configure la matrice de modèle (transformation d'objet)
    pub fn set_model_matrix(&mut self, matrix: Mat4) {
        self.model_matrix = matrix;
        self.invalidate_cache();
    }

    /// Définit la matrice de vue directement
    pub fn set_view_matrix(&mut self, matrix: Mat4) {
        self.view_matrix = matrix;
        self.invalidate_cache();
    }

    /// Définit la matrice de projection directement  
    pub fn set_projection_matrix(&mut self, matrix: Mat4) {
        self.projection_matrix = matrix;
        self.invalidate_cache();
    }

    /// Obtient la matrice MVP (Model-View-Projection) avec cache
    pub fn get_mvp_matrix(&mut self) -> Mat4 {
        if let Some(cached) = self.mvp_cache {
            return cached;
        }

        let mvp = self.projection_matrix * self.view_matrix * self.model_matrix;
        self.mvp_cache = Some(mvp);
        mvp
    }

    /// Obtient la matrice View-Projection avec cache
    pub fn get_view_projection_matrix(&mut self) -> Mat4 {
        if let Some(cached) = self.view_projection_cache {
            return cached;
        }

        let vp = self.projection_matrix * self.view_matrix;
        self.view_projection_cache = Some(vp);
        vp
    }

    /// Obtient la matrice normale pour l'éclairage
    pub fn get_normal_matrix(&mut self) -> Mat4 {
        if let Some(cached) = self.normal_matrix_cache {
            return cached;
        }

        let normal_matrix = (self.view_matrix * self.model_matrix).inverse().transpose();
        self.normal_matrix_cache = Some(normal_matrix);
        normal_matrix
    }

    /// Transforme un triangle complet par le pipeline 3D
    pub fn transform_triangle(&mut self, triangle: &Triangle3D) -> Result<TransformedTriangle> {
        let mvp_matrix = self.get_mvp_matrix();
        let normal_matrix = self.get_normal_matrix();

        let mut transformed_vertices = [TransformedVertex::default(); 3];

        for (i, vertex) in triangle.vertices.iter().enumerate() {
            // Transformation de position (vers clip space)
            let clip_pos = mvp_matrix
                * Vec4::new(vertex.position.x, vertex.position.y, vertex.position.z, 1.0);

            // Transformation de normale
            let world_normal = (normal_matrix
                * Vec4::new(vertex.normal.x, vertex.normal.y, vertex.normal.z, 0.0))
            .xyz()
            .normalize();

            // Calcul du fog si activé
            let fog_factor = if self.fog_enabled {
                let view_pos = (self.view_matrix
                    * self.model_matrix
                    * Vec4::new(vertex.position.x, vertex.position.y, vertex.position.z, 1.0))
                .z;
                let fog_distance = -view_pos; // Distance à la caméra
                ((fog_distance - self.fog_start) / (self.fog_end - self.fog_start)).clamp(0.0, 1.0)
            } else {
                0.0
            };

            transformed_vertices[i] = TransformedVertex {
                clip_position: clip_pos,
                world_position: (self.model_matrix
                    * Vec4::new(vertex.position.x, vertex.position.y, vertex.position.z, 1.0))
                .xyz(),
                world_normal,
                tex_coords: vertex.tex_coords,
                color: vertex.color,
                specular: vertex.specular,
                fog_factor,
            };
        }

        Ok(TransformedTriangle {
            vertices: transformed_vertices,
            texture_id: triangle.texture_id,
            material_id: triangle.material_id,
            flags: triangle.flags,
        })
    }

    /// Effectue le culling frustum sur un triangle
    pub fn frustum_cull_triangle(&self, triangle: &TransformedTriangle) -> bool {
        if !self.frustum_culling {
            return false; // Pas de culling
        }

        // Test de culling basique : tous les vertices hors du frustum
        let mut all_outside = true;
        for vertex in &triangle.vertices {
            let pos = vertex.clip_position;
            if pos.x >= -pos.w
                && pos.x <= pos.w
                && pos.y >= -pos.w
                && pos.y <= pos.w
                && pos.z >= 0.0
                && pos.z <= pos.w
            {
                all_outside = false;
                break;
            }
        }

        all_outside
    }

    /// Effectue le backface culling
    pub fn backface_cull_triangle(&self, triangle: &TransformedTriangle) -> bool {
        if !self.backface_culling || triangle.flags.two_sided || triangle.flags.no_culling {
            return false; // Pas de culling
        }

        // Calcul de la normale du triangle en screen space
        let v0 = triangle.vertices[0].clip_position;
        let v1 = triangle.vertices[1].clip_position;
        let v2 = triangle.vertices[2].clip_position;

        let edge1 = v1.xyz() - v0.xyz();
        let edge2 = v2.xyz() - v0.xyz();
        let normal = edge1.cross(edge2);

        // Si la normale pointe vers la caméra (z positif), garder le triangle
        normal.z < 0.0
    }

    /// Clip un triangle contre les plans du frustum
    pub fn clip_triangle(&self, triangle: &TransformedTriangle) -> Vec<TransformedTriangle> {
        // Implémentation simplifiée - retourne le triangle original pour l'instant
        // Une vraie implémentation ferait du clipping contre chaque plan du frustum
        vec![triangle.clone()]
    }

    /// Projection en coordonnées écran (perspective divide + viewport)
    pub fn project_to_screen(&self, triangle: &TransformedTriangle) -> ScreenTriangle {
        let mut screen_vertices = [ScreenVertex::default(); 3];

        for (i, vertex) in triangle.vertices.iter().enumerate() {
            // Perspective divide
            let ndc = vertex.clip_position.xyz() / vertex.clip_position.w;

            // Transformation viewport vers coordonnées écran
            let screen_pos = (self.viewport_matrix * Vec4::new(ndc.x, ndc.y, ndc.z, 1.0)).xyz();

            screen_vertices[i] = ScreenVertex {
                position: screen_pos,
                world_position: vertex.world_position,
                world_normal: vertex.world_normal,
                tex_coords: vertex.tex_coords,
                color: vertex.color,
                specular: vertex.specular,
                fog_factor: vertex.fog_factor,
                depth: ndc.z, // Depth pour Z-buffer
            };
        }

        ScreenTriangle {
            vertices: screen_vertices,
            texture_id: triangle.texture_id,
            material_id: triangle.material_id,
            flags: triangle.flags,
        }
    }

    /// Active/désactive le fog
    pub fn set_fog(&mut self, enabled: bool, start: f32, end: f32, color: [f32; 4]) {
        self.fog_enabled = enabled;
        self.fog_start = start;
        self.fog_end = end;
        self.fog_color = color;
    }

    /// Invalide les caches des matrices
    fn invalidate_cache(&mut self) {
        self.view_projection_cache = None;
        self.mvp_cache = None;
        self.normal_matrix_cache = None;
    }
}

impl Default for GeometryProcessor {
    fn default() -> Self {
        Self::new(800, 600) // Résolution par défaut
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geometry_processor_creation() {
        let processor = GeometryProcessor::new(1024, 768);
        assert_eq!(processor.aspect_ratio, 1024.0 / 768.0);
        assert_eq!(processor.near_plane, 0.1);
        assert_eq!(processor.far_plane, 1000.0);
        assert!(processor.frustum_culling);
        assert!(processor.backface_culling);
    }

    #[test]
    fn test_camera_configuration() {
        let mut processor = GeometryProcessor::new(800, 600);

        let position = Vec3::new(0.0, 5.0, 10.0);
        let target = Vec3::new(0.0, 0.0, 0.0);
        let up = Vec3::Y;

        processor.set_camera(position, target, up);

        assert_eq!(processor.camera_position, position);
        assert_eq!(processor.camera_target, target);
        assert_eq!(processor.camera_up, up);
    }

    #[test]
    fn test_perspective_configuration() {
        let mut processor = GeometryProcessor::new(800, 600);

        let fov = 60.0_f32.to_radians();
        let aspect = 16.0 / 9.0;
        let near = 0.5;
        let far = 500.0;

        processor.set_perspective(fov, aspect, near, far);

        assert_eq!(processor.field_of_view, fov);
        assert_eq!(processor.aspect_ratio, aspect);
        assert_eq!(processor.near_plane, near);
        assert_eq!(processor.far_plane, far);
    }

    #[test]
    fn test_triangle_transformation() {
        let mut processor = GeometryProcessor::new(800, 600);

        // Triangle simple
        let triangle = Triangle3D {
            vertices: [
                Vertex3D {
                    position: Vec3::new(-1.0, -1.0, 0.0),
                    normal: Vec3::Z,
                    tex_coords: [0.0, 0.0],
                    color: [1.0, 0.0, 0.0, 1.0],
                    fog_coord: 0.0,
                    specular: [0.0, 0.0, 0.0],
                },
                Vertex3D {
                    position: Vec3::new(1.0, -1.0, 0.0),
                    normal: Vec3::Z,
                    tex_coords: [1.0, 0.0],
                    color: [0.0, 1.0, 0.0, 1.0],
                    fog_coord: 0.0,
                    specular: [0.0, 0.0, 0.0],
                },
                Vertex3D {
                    position: Vec3::new(0.0, 1.0, 0.0),
                    normal: Vec3::Z,
                    tex_coords: [0.5, 1.0],
                    color: [0.0, 0.0, 1.0, 1.0],
                    fog_coord: 0.0,
                    specular: [0.0, 0.0, 0.0],
                },
            ],
            texture_id: None,
            material_id: 0,
            flags: TriangleFlags::default(),
        };

        let result = processor.transform_triangle(&triangle);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert_eq!(transformed.vertices.len(), 3);

        // Vérifier que les coordonnées de texture sont préservées
        assert_eq!(transformed.vertices[0].tex_coords, [0.0, 0.0]);
        assert_eq!(transformed.vertices[1].tex_coords, [1.0, 0.0]);
        assert_eq!(transformed.vertices[2].tex_coords, [0.5, 1.0]);
    }

    #[test]
    fn test_bounding_box() {
        let mut bbox = BoundingBox::empty();

        bbox.expand(Vec3::new(1.0, 2.0, 3.0));
        bbox.expand(Vec3::new(-1.0, -2.0, -3.0));
        bbox.expand(Vec3::new(0.5, 1.5, 2.5));

        assert_eq!(bbox.min, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(bbox.max, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_fog_configuration() {
        let mut processor = GeometryProcessor::new(800, 600);

        processor.set_fog(true, 5.0, 50.0, [0.5, 0.6, 0.8, 1.0]);

        assert!(processor.fog_enabled);
        assert_eq!(processor.fog_start, 5.0);
        assert_eq!(processor.fog_end, 50.0);
        assert_eq!(processor.fog_color, [0.5, 0.6, 0.8, 1.0]);
    }

    #[test]
    fn test_triangle_flags() {
        let flags = TriangleFlags::default();

        assert!(!flags.transparent);
        assert!(!flags.two_sided);
        assert!(!flags.no_culling);
        assert!(!flags.wireframe);
        assert!(!flags.flat_shading);
        assert!(flags.texture_filtering);
    }

    #[test]
    fn test_mvp_matrix_cache() {
        let mut processor = GeometryProcessor::new(800, 600);

        // Premier appel calcule la matrice
        let mvp1 = processor.get_mvp_matrix();

        // Deuxième appel utilise le cache
        let mvp2 = processor.get_mvp_matrix();

        assert_eq!(mvp1, mvp2);

        // Changer la matrice model invalide le cache
        processor.set_model_matrix(Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)));
        let mvp3 = processor.get_mvp_matrix();

        assert_ne!(mvp1, mvp3);
    }
}

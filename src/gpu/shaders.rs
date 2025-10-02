//! Système de shaders

/// Types de shaders supportés
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

/// Gestionnaire de shaders
pub struct ShaderManager {
    // Placeholder pour future extension
}

impl ShaderManager {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ShaderManager {
    fn default() -> Self {
        Self::new()
    }
}

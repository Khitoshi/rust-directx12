#[path = "./shader.rs"]
mod shader;

// モデルデータ (ここではトライアングル)
pub struct Triangle {
    vertes_shader: shader::Shader,
    pixcel_shader: shader::Shader,
}

impl Default for Triangle {
    fn default() -> Self {
        Self {
            vertes_shader: None,
            pixcel_shader: None,
        }
    }
}

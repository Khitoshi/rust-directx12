pub struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
    uv: [f32; 2],
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            pos: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0],
            uv: [0.0, 0.0],
        }
    }
}

//set methods
impl Vertex {
    pub fn set_pos(&mut self, p: [f32; 3]) {
        self.pos = p;
    }
}

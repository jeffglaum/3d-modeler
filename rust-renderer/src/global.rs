use std::sync::RwLock;

// vertex data type
pub type Pos = [f32; 3];
pub type Norm = [f32; 3];

#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct Vertex(pub Pos, pub Norm);

// Global storage for vertices and indices
thread_local! {
    pub static MODEL_LOADED: RwLock<bool> = RwLock::new(false);
    pub static DRAW_WIREFRAME: RwLock<bool> = RwLock::new(true);
    pub static VERTICES: RwLock<Vec<Vertex>> = RwLock::new(Vec::new());
    pub static INDICES: RwLock<Vec<u32>> = RwLock::new(Vec::new());
}

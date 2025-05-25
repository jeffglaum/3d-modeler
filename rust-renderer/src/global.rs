use crate::model::ModelObject;
use std::sync::RwLock;

// vertex data type
pub type Pos = [f32; 3];
pub type Norm = [f32; 3];

#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct Vertex(pub Pos, pub Norm);

// Global storage for vertices and indices
thread_local! {
    pub static MODEL: RwLock<Option<ModelObject>> = RwLock::new(None);
    pub static GRID: RwLock<Option<ModelObject>> = RwLock::new(None);
}

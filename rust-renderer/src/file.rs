use crate::global::{Norm, Pos, Vertex, INDICES, MODEL_LOADED, VERTICES};
use crate::update_model;
use std::collections::HashMap;
use std::io::BufReader;
use wasm_bindgen::prelude::*;
use wavefront_rs::obj::{entity::*, parser::*};
use web_sys::WebGl2RenderingContext as GL;

#[wasm_bindgen]
pub fn process_file_content(content: &str, gl: GL) {
    // Log the file content to the browser console (for debugging)
    //web_sys::console::log_1(&format!("Received file content: {}", content).into());

    // Prepare to parse the OBJ content
    let mut positions: HashMap<usize, Pos> = HashMap::new();
    let mut normals: HashMap<usize, Norm> = HashMap::new();
    let mut vertices: Vec<Vertex> = vec![];
    let mut indices: Vec<u32> = vec![];

    // Parse the OBJ content from the provided string
    Parser::read_to_end(&mut BufReader::new(content.as_bytes()), |x| match x {
        Entity::Vertex { x, y, z, w: _ } => {
            let index = positions.len() + 1; // OBJ indices are 1-based
            positions.insert(index, [x as f32, y as f32, z as f32]);
            //web_sys::console::log_1(&format!("Vertex: {},{},{}", x, y, z).into());
        }
        Entity::VertexNormal { x, y, z } => {
            let index = normals.len() + 1; // OBJ indices are 1-based
            normals.insert(index, [x as f32, y as f32, z as f32]);
            //web_sys::console::log_1(&format!("Vertex Normal: {},{},{}", x, y, z).into());
        }
        Entity::Face {
            vertices: face_vertices,
        } => {
            // Fan triangulation: create triangles from v0, vi, vi+1
            if face_vertices.len() >= 3 {
                let v0 = &face_vertices[0];
                for i in 1..face_vertices.len() - 1 {
                    let v1 = &face_vertices[i];
                    let v2 = &face_vertices[i + 1];

                    let indices_set = [v0, v1, v2];

                    for v in &indices_set {
                        let pos_index = v.vertex as usize;
                        let norm_index = v.normal.unwrap_or(0) as usize;

                        if let Some(pos) = positions.get(&pos_index) {
                            let norm = normals.get(&norm_index).unwrap_or(&[0.0, 0.0, 0.0]);
                            vertices.push(Vertex(*pos, *norm));
                        }

                        // OBJ indices are 1-based, subtract 1
                        indices.push((v.vertex - 1) as u32);
                    }
                }
            }
        }

        _ => {}
    })
    .unwrap();

    // Store the vertices and indices in the global storage
    VERTICES.with(|v| {
        let mut global_vertices = v.write().unwrap();
        *global_vertices = vertices;
    });

    INDICES.with(|i| {
        let mut global_indices = i.write().unwrap();
        *global_indices = indices;
    });

    MODEL_LOADED.with(|i| {
        let mut loaded = i.write().unwrap();
        *loaded = false;
    });

    // Log the number of indices for debugging
    let indices_length = INDICES.with(|i| i.read().unwrap().len());
    web_sys::console::log_1(&format!("INFO: number of model indices: {}", indices_length).into());

    // Trigger the update_model function
    update_model(gl);
}

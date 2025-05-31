use crate::{global::{Norm, Pos, Vertex, MODEL}, trigger_draw_event};
use std::collections::HashMap;
use std::io::BufReader;
use wasm_bindgen::prelude::*;
use wavefront_rs::obj::{entity::*, parser::*};

#[wasm_bindgen]
pub fn process_file_content(content: &str) {
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

    // Update the model with the parsed vertices and indices
    MODEL.with(|v| {
        v.write()
            .unwrap()
            .as_mut()
            .unwrap()
            .update_model(vertices, indices);
    });

    // Trigger a re-render of the model
    trigger_draw_event();

}

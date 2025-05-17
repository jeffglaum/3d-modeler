use cgmath::Matrix4;

/// Converts a cgmath::Matrix4<f32> into a column-major [f32; 16] array for OpenGL.
pub fn matrix4_to_array(matrix: &Matrix4<f32>) -> [f32; 16] {
    [
        matrix.x.x, matrix.x.y, matrix.x.z, matrix.x.w, matrix.y.x, matrix.y.y, matrix.y.z,
        matrix.y.w, matrix.z.x, matrix.z.y, matrix.z.z, matrix.z.w, matrix.w.x, matrix.w.y,
        matrix.w.z, matrix.w.w,
    ]
}
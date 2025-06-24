use std::f32::consts::PI;

use crate::graphics;

pub type Vec3 = [f32; 3];
pub type Vec4 = [f32; 4];

type TriIndexes = [u32; 3];

pub struct CircleBuffer {
    vertices: Vec<Vec3>,
    indices: Vec<TriIndexes>,
}

impl CircleBuffer {
    pub fn new() -> CircleBuffer {
        let n = 64;
        let mut vertices: Vec<Vec3> = vec![[0.0, 0.0, 0.0]];
        let mut indices: Vec<TriIndexes> = vec![[0, 1, n]];
        for i in 0..n {
            let t = i as f32 / n as f32 * 2.0 * PI;
            vertices.push([t.sin(), t.cos(), 0.0]);
            indices.push([0, i + 1, i + 2]);
        }
        indices.pop();
        Self {
            vertices: vertices,
            indices: indices,
        }
    }

    pub fn vertices_buffer_data(&self) {
        graphics::buffer_data(
            graphics::BufferType::Array,
            bytemuck::cast_slice(&self.vertices),
            gl::STATIC_DRAW,
        );
        unsafe {
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Vec3>().try_into().unwrap(),
                0 as *const _,
            );
            gl::EnableVertexAttribArray(0);
        }
    }

    pub fn indeces_buffer_data(&self) {
        graphics::buffer_data(
            graphics::BufferType::ElementArray,
            bytemuck::cast_slice(&self.indices),
            gl::STATIC_DRAW,
        );
    }

    pub fn draw(&self) {
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32 * 3,
                gl::UNSIGNED_INT,
                0 as *const _,
            )
        }
    }
}

use std::f32::consts::PI;

use gl::types::*;

use crate::graphics;

pub type Vec3 = [f32; 3];
pub type Vec4 = [f32; 4];
pub type Mat4 = [f32; 16];

type Point = [f32; 8];
type TriIndexes = [u32; 3];

pub struct CircleBuffer {
    vertices: Vec<Point>,
    indices: Vec<TriIndexes>,
}

impl CircleBuffer {
    pub fn new() -> CircleBuffer {
        let n = 64;
        let mut vertices: Vec<Point> = vec![[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.2]];
        let mut indices: Vec<TriIndexes> = vec![[0, 1, n]];
        for i in 0..n {
            let t = i as f32 / n as f32 * 2.0 * PI;
            vertices.push([t.sin(), t.cos(), 0.0, 0.0, 0.0, 1.0, 0.9, 0.8]);
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
                2,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Point>().try_into().unwrap(),
                0 as *const _,
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                1,
                4,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Point>().try_into().unwrap(),
                8 as *const _,
            );
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                2,
                1,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Point>().try_into().unwrap(),
                24 as *const _,
            );
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                3,
                1,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Point>().try_into().unwrap(),
                28 as *const _,
            );
            gl::EnableVertexAttribArray(3);
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

pub fn calc_mat(u_mvp: GLint, x: f32, y: f32, scale: f32) {
    let mat: Mat4 = [
        scale / 640.0,
        0.0,
        0.0,
        x,
        0.0,
        scale / 480.0,
        0.0,
        y,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    ];
    unsafe {
        gl::UniformMatrix4fv(u_mvp, 1, gl::TRUE, mat.as_ptr());
    }
}

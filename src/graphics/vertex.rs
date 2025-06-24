extern crate gl;

use gl::types::*;

/// Basic wrapper for a [Vertex Array
/// Object](https://www.khronos.org/opengl/wiki/Vertex_Specification#Vertex_Array_Object).
pub struct VertexArray(pub GLuint);
impl VertexArray {
    /// Creates a new vertex array object
    pub fn new() -> Option<Self> {
        let mut vao = 0;
        unsafe { gl::GenVertexArrays(1, &mut vao) };
        if vao != 0 { Some(Self(vao)) } else { None }
    }

    /// Bind this vertex array as the current vertex array object
    pub fn bind(&self) {
        unsafe { gl::BindVertexArray(self.0) }
    }

    /// Clear the current vertex array object binding.
    pub fn clear_binding() {
        unsafe { gl::BindVertexArray(0) }
    }
}

/// The types of buffer object that you can have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    /// Array Buffers holds arrays of vertex data for drawing.
    Array = gl::ARRAY_BUFFER as isize,
    /// Element Array Buffers hold indexes of what vertexes to use for drawing.
    ElementArray = gl::ELEMENT_ARRAY_BUFFER as isize,
}

/// Basic wrapper for a [Buffer
/// Object](https://www.khronos.org/opengl/wiki/Buffer_Object).
pub struct Buffer(pub GLuint);
impl Buffer {
    /// Makes a new vertex buffer
    pub fn new() -> Option<Self> {
        let mut vbo = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
        }
        if vbo != 0 { Some(Self(vbo)) } else { None }
    }

    /// Bind this vertex buffer for the given type
    pub fn bind(&self, ty: BufferType) {
        unsafe { gl::BindBuffer(ty as GLenum, self.0) }
    }

    /// Clear the current vertex buffer binding for the given type.
    pub fn clear_binding(ty: BufferType) {
        unsafe { gl::BindBuffer(ty as GLenum, 0) }
    }
}

/// Places a slice of data into a previously-bound buffer.
pub fn buffer_data(ty: BufferType, data: &[u8], usage: GLenum) {
    unsafe {
        gl::BufferData(
            ty as GLenum,
            data.len().try_into().unwrap(),
            data.as_ptr().cast(),
            usage,
        );
    }
}

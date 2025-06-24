use std::io::{Error, ErrorKind, Result};

extern crate gl;

use gl::types::*;

pub struct Shader {
    type_: GLenum,
    shader: GLuint,
}

impl Shader {
    pub fn new(type_: GLenum) -> Option<Shader> {
        let shader = unsafe { gl::CreateShader(type_) };
        if shader == 0 {
            None
        } else {
            Some(Self {
                type_: type_,
                shader: shader,
            })
        }
    }
    pub fn init(&self, source: &str) -> Result<()> {
        unsafe {
            gl::ShaderSource(
                self.shader,
                1,
                &(source.as_bytes().as_ptr().cast()),
                &(source.len().try_into().unwrap()),
            );
            gl::CompileShader(self.shader);
            let mut success = 0;
            gl::GetShaderiv(self.shader, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0_i32;
                gl::GetShaderInfoLog(self.shader, 1024, &mut log_len, v.as_mut_ptr().cast());
                v.set_len(log_len.try_into().unwrap());
                Err(Error::new(ErrorKind::Other, String::from_utf8_lossy(&v)))
            } else {
                Ok(())
            }
        }
    }
    pub fn shader_type_name(&self) -> &'static str {
        match self.type_ {
            gl::VERTEX_SHADER => "vertex",
            gl::FRAGMENT_SHADER => "fragment",
            _other => "",
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.shader);
        }
    }
}

pub struct ShaderProgram<'a> {
    program: GLuint,
    vertex_shader: &'a Shader,
    fragment_shader: &'a Shader,
}

impl<'a> ShaderProgram<'a> {
    pub fn new(
        vertex_shader: &'a Shader,
        fragment_shader: &'a Shader,
    ) -> Result<ShaderProgram<'a>> {
        unsafe {
            let program = gl::CreateProgram();
            if program == 0 {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Unable to create a shader program",
                ));
            }
            gl::AttachShader(program, vertex_shader.shader);
            gl::AttachShader(program, fragment_shader.shader);
            gl::LinkProgram(program);
            let mut success = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0_i32;
                gl::GetProgramInfoLog(program, 1024, &mut log_len, v.as_mut_ptr().cast());
                v.set_len(log_len.try_into().unwrap());

                Err(Error::new(ErrorKind::Other, String::from_utf8_lossy(&v)))
            } else {
                Ok(Self {
                    program: program,
                    vertex_shader: vertex_shader,
                    fragment_shader: fragment_shader,
                })
            }
        }
    }
    pub fn use_program(&self) {
        unsafe { gl::UseProgram(self.program) }
    }
    pub fn detach_program() {
        unsafe { gl::UseProgram(0) }
    }

    pub fn get_uniform_location(&self, name: *const GLchar) -> GLint {
        unsafe { gl::GetUniformLocation(self.program, name) }
    }

    pub fn get_vertex_shader(&self) -> &'a Shader {
        self.vertex_shader
    }
    pub fn get_fragment_shader(&self) -> &'a Shader {
        self.fragment_shader
    }
}

impl Drop for ShaderProgram<'_> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}

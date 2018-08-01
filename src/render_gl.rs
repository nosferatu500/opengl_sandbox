use gl;
use std;
use std::ffi::{CString, CStr};
use resources::Resources;

pub struct Program {
    gl: gl::Gl,
    id: gl::types::GLuint,
}

impl Program {
    pub fn from_res(gl: &gl::Gl, res: &Resources, name: &str) -> Result<Program, String> {
        const POSSIBLE_EXT: [&str; 2] = [
            ".vert",
            ".frag",
        ];

        let shaders = POSSIBLE_EXT.iter()
            .map(|file_extension| {
                Shader::from_res(gl, res, &format!("{}{}", name, file_extension))
            })
            .collect::<Result<Vec<Shader>, String>>()?;

        Program::from_shaders(gl, &shaders[..])
    }

    pub fn from_shaders(gl: &gl::Gl, shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl.CreateProgram() };

        for shader in shaders {
            unsafe { gl.AttachShader(program_id, shader.id()); }
        }

        unsafe { gl.LinkProgram(program_id); }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl.GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl.GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl.GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe { gl.DetachShader(program_id, shader.id()); }
        }

        Ok(Program { gl: gl.clone(), id: program_id })
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn set_used(&self) {
        unsafe {
            self.gl.UseProgram(self.id);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.id);
        }
    }
}

pub struct Shader {
    gl: gl::Gl,
    id: gl::types::GLuint,
}

impl Shader {
    pub fn from_res(gl: &gl::Gl, res: &Resources, name: &str) -> Result<Shader, String> {
        const POSSIBLE_EXT: [(&str, gl::types::GLenum); 2] = [
            (".vert", gl::VERTEX_SHADER),
            (".frag", gl::FRAGMENT_SHADER),
        ];

        let shader_kind = POSSIBLE_EXT.iter().find(|&&(file_extension, _)| {
            name.ends_with(file_extension)
        }).map(|&(_, kind)| kind)
        .ok_or_else(|| format!("Can not determine shader type for resource {:?}", name))?;

        let source = res.load_cstring(name).map_err(|e| format!("Error loading resource {:?}: {:?}", name, e))?;

        Shader::from_source(gl, &source, shader_kind)
    }

    pub fn from_source(
        gl: &gl::Gl,
        source: &CStr,
        kind: gl::types::GLenum
    ) -> Result<Shader, String> {
        let id = shader_from_source(gl, source, kind)?;
        Ok(Shader { gl: gl.clone(), id })
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteShader(self.id);
        }
    }
}

fn shader_from_source(
    gl: &gl::Gl,
    source: &CStr,
    kind: gl::types::GLenum
) -> Result<gl::types::GLuint, String> {
    let id = unsafe { gl.CreateShader(kind) };
    unsafe {
        gl.ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl.CompileShader(id);
    }

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error = create_whitespace_cstring_with_len(len as usize);

        unsafe {
            gl.GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar
            );
        }

        return Err(error.to_string_lossy().into_owned());
    }

    Ok(id)
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    // allocate buffer of correct size
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill it with len spaces
    buffer.extend([b' '].iter().cycle().take(len));
    // convert buffer to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}
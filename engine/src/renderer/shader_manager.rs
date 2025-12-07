// ShaderManager needs to be able to compile and manage shaders for the renderer.

use anyhow::Result;
use shaderc::{
    Compiler,
    ShaderKind::{self, Fragment, Vertex},
};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use std::fs;
use std::io::{Cursor, Read};
use std::path::{PathBuf};

#[derive(EnumIter, Debug)]
pub enum ShaderId {
    BasicVertex,
    BasicFragment,
}

// Static metadata associated with each shader
struct ShaderMeta {
    path: &'static str,
    kind: ShaderKind,
}

impl ShaderId {
    fn meta(&self) -> ShaderMeta {

        match self {
            ShaderId::BasicVertex => ShaderMeta {
                path: "basic.vert",
                kind: Vertex,
            },
            ShaderId::BasicFragment => ShaderMeta {
                path: "basic.frag",
                kind: Fragment,
            },
        }
    }

    pub fn path(&self) -> PathBuf {
        PathBuf::from("shaders").join(self.meta().path)
    }

    pub fn compiled_path_str(&self) -> String {
        let shader_path = self.path();

        let spv_path = shader_path.with_extension(format!(
            "{}.spv",
            shader_path.extension().unwrap().to_string_lossy()
        ));

        spv_path.to_string_lossy().into_owned()
    }

    pub fn load_shader_bytes(&self, shader_id: ShaderId) -> Result<Vec<u32>> {
        let spv_path = shader_id.compiled_path_str();
        let bytes = std::fs::read(&spv_path)?;

        let aligned_bytes = bytemuck::cast_slice(&bytes).to_vec();
        Ok(aligned_bytes)
    }

    pub fn kind(&self) -> ShaderKind {
        self.meta().kind
    }

    pub fn all() -> impl Iterator<Item = ShaderId> {
        ShaderId::iter()
    }
}

pub struct ShaderManager {
    compiler: shaderc::Compiler, // saving this because it will need to be dynamic later for hot-reloading
}

impl ShaderManager {
    pub fn new() -> Result<Self> {
        let compiler = Compiler::new().expect("Failed to initialize shaderc compiler.");
        //let options = CompileOptions::new()?;
        // add macro definitions if needed, make options mut
        Ok(Self { compiler })
    }

    pub fn compile_all_shaders(&self) -> Result<()> {
        for shader_id in ShaderId::all() {
            self.compile_shader(shader_id)?;
        }
        Ok(())
    }

    // returns an owned string path to the compiled SPIR-V file for loading with ash
    pub fn compile_shader(&self, shader_id: ShaderId) -> Result<String> {
        let meta = shader_id.meta();
        let shader_path = shader_id.path();

        let mut file = fs::File::open(&shader_path)?;
        let mut source_bytes = Vec::new();
        file.read_to_end(&mut source_bytes)?;
        let mut cursor = Cursor::new(source_bytes);
        let mut source_string = String::new();
        cursor.read_to_string(&mut source_string)?;

        // Compile GLSL -> SPIR-V
        let compiled = self.compiler.compile_into_spirv(
            &source_string,
            meta.kind,
            shader_path.to_str().unwrap(),
            "main",
            None,
        )?;

        let spv_path = shader_id.compiled_path_str();
        fs::write(&spv_path, compiled.as_binary_u8())?;

        println!("Compiled shader {:?} -> {:?}", shader_id, spv_path);
        Ok(spv_path)
    }
}

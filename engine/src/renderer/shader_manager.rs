// ShaderManager needs to be able to compile and manage shaders for the renderer.

use anyhow::Result;
use naga::{
    back::spv,
    ShaderStage::{Fragment, Vertex},
};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use std::fs;
use std::io::{Cursor, Read};
use std::path::PathBuf;

#[derive(EnumIter, Debug)]
pub enum ShaderId {
    TriangleVertex,
    TriangleFrag,
}

// Static metadata associated with each shader
#[derive(Debug)]
struct ShaderMeta {
    path: &'static str,
    stage: naga::ShaderStage,
}

impl ShaderId {
    fn meta(&self) -> ShaderMeta {
        match self {
            ShaderId::TriangleVertex => ShaderMeta {
                path: "triangle.vert",
                stage: Vertex,
            },
            ShaderId::TriangleFrag => ShaderMeta {
                path: "triangle.frag",
                stage: Fragment,
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

    pub fn load_shader_bytes(&self) -> Result<Vec<u32>> {
        let spv_path = self.compiled_path_str();
        let bytes = std::fs::read(&spv_path)?;
        let aligned_bytes = bytemuck::cast_slice(&bytes).to_vec();
        Ok(aligned_bytes)
    }

    pub fn stage(&self) -> naga::ShaderStage {
        self.meta().stage
    }

    pub fn all() -> impl Iterator<Item = ShaderId> {
        ShaderId::iter()
    }
}

pub struct ShaderManager;

impl ShaderManager {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub fn compile_all_shaders(&self) -> Result<()> {
        for shader_id in ShaderId::all() {
            self.compile_shader(shader_id)?;
        }
        Ok(())
    }

    pub fn compile_shader(&self, shader_id: ShaderId) -> Result<String> {
        let meta = shader_id.meta();
        let shader_path = shader_id.path();

        let mut file = fs::File::open(&shader_path)?;
        let mut source_bytes = Vec::new();
        file.read_to_end(&mut source_bytes)?;
        let mut cursor = Cursor::new(source_bytes);
        let mut source_string = String::new();
        cursor.read_to_string(&mut source_string)?;

        // Parse GLSL with naga
        let mut frontend = naga::front::glsl::Frontend::default();
        let module = frontend.parse(
            &naga::front::glsl::Options::from(meta.stage),
            &source_string,
        )?;

        // Validate module
        let info = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)?;

        // Compile to SPIR-V
        let spirv = spv::write_vec(&module, &info, &spv::Options::default(), None)?;

        // Write to file
        let spv_path = shader_id.compiled_path_str();
        fs::write(&spv_path, bytemuck::cast_slice::<u32, u8>(&spirv))?;

        println!("Compiled shader {:?} -> {:?}", shader_id, spv_path);
        Ok(spv_path)
    }
}

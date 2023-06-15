use std::mem;
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D::Fxc::*, Win32::Graphics::Direct3D::*,
    Win32::Graphics::Direct3D12::*, Win32::Graphics::Dxgi::Common::*, Win32::Graphics::Dxgi::*,
    Win32::System::LibraryLoader::*, Win32::System::Threading::*,
    Win32::UI::WindowsAndMessaging::*,
};

/// エラー取得用 module
mod dx12error;

mod shader;

mod pipeline_state;

use crate::window::root_signature;

mod vertex;

/*
#[path = "./render_context.rs"]
mod render_context;
*/

#[path = "./Index_buffer.rs"]
mod index_buffer;
// モデルデータ (ここではトライアングル)
pub struct Triangle {
    vertes_shader: shader::Shader,
    pixcel_shader: shader::Shader,
    pipeline_state: pipeline_state::PipelineState,
    vertex: [vertex::Vertex; 4],
    index_buffer: index_buffer::IndexBuffer,
}

impl Default for Triangle {
    fn default() -> Self {
        Self {
            vertes_shader: shader::Shader::default(),
            pixcel_shader: shader::Shader::default(),
            pipeline_state: pipeline_state::PipelineState::default(),
            vertex: [
                vertex::Vertex::default(),
                vertex::Vertex::default(),
                vertex::Vertex::default(),
                vertex::Vertex::default(),
            ],
            index_buffer: index_buffer::IndexBuffer::default(),
        }
    }
}

/// トライアングルの初期化 methods
impl Triangle {
    pub fn init(
        device: &ID3D12Device,
        rs: &root_signature::RootSignature,
    ) -> std::result::Result<Triangle, dx12error::Dx12Error> {
        let mut triangle: Triangle = Triangle::default();

        //シェーダーの読み込み
        let (ps, vs) = match Triangle::load_shaders() {
            Ok(shaders) => shaders,
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create shader: {:?}",
                    err
                )))
            }
        };
        triangle.pixcel_shader = ps;
        triangle.vertes_shader = vs;

        //パイプラインステートの生成
        match triangle.init_pipeline_state(device, rs) {
            Ok(ps) => triangle.pipeline_state = ps,
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create pipeline state: {:?}",
                    err
                )))
            }
        };

        //頂点情報設定
        let vb = Triangle::init_vertex_buffer();
        triangle.vertex[0].set_pos(vb[0]);
        triangle.vertex[1].set_pos(vb[1]);
        triangle.vertex[2].set_pos(vb[2]);
        triangle.vertex[3].set_pos(vb[3]);

        //インデックスバッファー設定
        triangle.index_buffer = match triangle.init_index_buffer(device) {
            Ok(ib) => ib,
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create index buffer: {:?}",
                    err
                )))
            }
        };

        Ok(triangle)
    }

    ///
    fn load_shaders() -> std::result::Result<(shader::Shader, shader::Shader), dx12error::Dx12Error>
    {
        let ps_shader: shader::Shader = match shader::Shader::load_ps("./shader/dummy_shader.hlsl")
        {
            Ok(ps) => ps,
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create shader: {:?}",
                    err
                )))
            }
        };

        let vs_shader: shader::Shader = match shader::Shader::load_vs("./shader/dummy_shader.hlsl")
        {
            Ok(ps) => ps,
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create shader: {:?}",
                    err
                )))
            }
        };

        Ok((ps_shader, vs_shader))
    }

    ///
    fn init_pipeline_state(
        &self,
        device: &ID3D12Device,
        rs: &root_signature::RootSignature,
    ) -> std::result::Result<pipeline_state::PipelineState, dx12error::Dx12Error> {
        // 頂点レイアウトを定義する。
        let mut input_element_descs: [D3D12_INPUT_ELEMENT_DESC; 3] = [
            D3D12_INPUT_ELEMENT_DESC {
                SemanticName: s!("POSITION"),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: 0,
                InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
            D3D12_INPUT_ELEMENT_DESC {
                SemanticName: s!("COLOR"),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: 12,
                InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
            D3D12_INPUT_ELEMENT_DESC {
                SemanticName: s!("TEXCOORD"),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: 24,
                InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
        ];

        //ピクセルシェーダーの取得
        let ps = match self.pixcel_shader.get_shader() {
            Ok(p) => p,
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create pixcel shader: {:?}",
                    err
                )))
            }
        };

        //頂点シェーダーの取得
        let vs = match self.vertes_shader.get_shader() {
            Ok(v) => v,
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create vertex shader: {:?}",
                    err
                )))
            }
        };

        //ルートシグネチャーの取得
        let rs = match rs.get_root_signature() {
            Ok(rs) => rs,
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create root signature: {:?}",
                    err
                )))
            }
        };

        //パイプラインステート設定
        let mut desc: D3D12_GRAPHICS_PIPELINE_STATE_DESC = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
            InputLayout: D3D12_INPUT_LAYOUT_DESC {
                pInputElementDescs: input_element_descs.as_mut_ptr(),
                NumElements: input_element_descs.len() as u32,
            },
            pRootSignature: unsafe { std::mem::transmute_copy(rs) },
            VS: D3D12_SHADER_BYTECODE {
                pShaderBytecode: unsafe { vs.GetBufferPointer() },
                BytecodeLength: unsafe { vs.GetBufferSize() },
            },
            PS: D3D12_SHADER_BYTECODE {
                pShaderBytecode: unsafe { ps.GetBufferPointer() },
                BytecodeLength: unsafe { ps.GetBufferSize() },
            },
            RasterizerState: D3D12_RASTERIZER_DESC {
                CullMode: D3D12_CULL_MODE_NONE,
                ..Default::default()
            },
            BlendState: D3D12_BLEND_DESC {
                ..Default::default()
            },
            DepthStencilState: D3D12_DEPTH_STENCIL_DESC {
                DepthEnable: FALSE,
                DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ZERO,
                DepthFunc: D3D12_COMPARISON_FUNC_LESS_EQUAL,
                StencilEnable: FALSE,
                ..Default::default()
            },
            SampleMask: u32::max_value(),
            PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
            NumRenderTargets: 1,
            DSVFormat: DXGI_FORMAT_D32_FLOAT,

            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },

            ..Default::default()
        };
        desc.RTVFormats[0] = DXGI_FORMAT_R8G8B8A8_UNORM;

        //パイプラインステート生成
        match pipeline_state::PipelineState::create(device, &desc) {
            Ok(pso) => Ok(pso),
            Err(err) => Err(dx12error::Dx12Error::new(&format!(
                "Failed to create pipeline state: {:?}",
                err
            ))),
        }
    }

    //頂点情報設定
    fn init_vertex_buffer() -> [[f32; 3]; 4] {
        [
            [-0.4, -0.7, 0.0],
            [-0.4, 0.7, 0.0],
            [0.4, -0.7, 0.0],
            [0.4, 0.7, 0.0],
        ]
    }

    fn init_index_buffer(
        &mut self,
        device: &ID3D12Device,
    ) -> std::result::Result<index_buffer::IndexBuffer, dx12error::Dx12Error> {
        let mut index_buffer: index_buffer::IndexBuffer = index_buffer::IndexBuffer::default();
        let mut indices: [u16; 3] = [0, 1, 2];

        index_buffer = match index_buffer::IndexBuffer::create(device, mem::size_of::<[i8; 3]>(), 2)
        {
            Ok(ib) => ib,
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create index buffer: {:?}",
                    err
                )))
            }
        };

        index_buffer.copy(&indices);

        Ok(index_buffer)
    }
}

impl Triangle {
    /*
    pub fn draw(&mut self, rc: &mut render_context::RenderContext) {
        // rc.set
    }
    */
}

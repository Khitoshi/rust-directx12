use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D::Fxc::*, Win32::Graphics::Direct3D::*,
    Win32::Graphics::Direct3D12::*, Win32::Graphics::Dxgi::Common::*, Win32::Graphics::Dxgi::*,
    Win32::Graphics::Hlsl::*, Win32::System::LibraryLoader::*, Win32::System::Threading::*,
    Win32::UI::WindowsAndMessaging::*,
};

//エラー取得用 module
#[path = "./dx12error.rs"]
mod dx12error;

#[derive(Copy, Clone)]
enum Shader_num {
    pixcel_shader,
    vertex_shader,

    Num_shader_model_list,
}

/// シェーダデータ
///
/// # Fields
/// *  'blob' - コンパイル済みシェーダーデータ
/// *  'dxc_blob' - DXCコンパイラを使用した時のシェーダーデータ
pub struct Shader {
    blob: Option<ID3DBlob>,
    //dxc_blob: Option<ID3DBlob>,
}

///シェーダーの初期化
impl Default for Shader {
    fn default() -> Self {
        Self { blob: None }
    }
}

impl Shader {
    ///シェーダー読み込み&コンパイル
    /// # Arguments
    /// *  'shader_file_path' - シェーダーファイルパス
    /// *  'shader_num' - シェーダー番号
    ///
    /// # Returns
    /// *  'Ok(ID3DBlob)' - コンパイル済みシェーダーデータ
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    fn load_shader(
        shader_file_path: &str,
        shader_num: Shader_num,
    ) -> std::result::Result<ID3DBlob, dx12error::Dx12Error> {
        //let mut error_blob: Option<ID3DBlob> = None;

        let compile_flags = if cfg!(debug_assertions) {
            D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION
        } else {
            0
        };

        //文字列変換
        let exe_path = std::env::current_exe().ok().unwrap();
        let asset_path = exe_path.parent().unwrap();

        //shaderファイルパス 文字列 変換
        //let shaders_hlsl_path = asset_path.join(shader_file_path);
        let shaders_hlsl_path = asset_path.join(shader_file_path);
        let shaders_hlsl = shaders_hlsl_path.to_str().unwrap();
        let shaders_hlsl: HSTRING = shaders_hlsl.into();

        //シェーダーモデルリスト(文字列)
        let shader_model: [PCSTR; Shader_num::Num_shader_model_list as usize] =
            [s!("ps_5_0"), s!("vs_5_0")];
        let entry_func_name: [PCSTR; Shader_num::Num_shader_model_list as usize] =
            [s!("PSMain"), s!("VSMain")];

        //シェーダー
        let mut shader = None;

        //シェーダーコンパイル
        let shader = match unsafe {
            D3DCompileFromFile(
                &shaders_hlsl,
                None,
                None,
                entry_func_name[shader_num as usize],
                shader_model[shader_num as usize],
                compile_flags,
                0,
                &mut shader,
                None,
            )
        } {
            Ok(_) => {
                if let Some(s) = shader {
                    s
                } else {
                    return Err(dx12error::Dx12Error::new("Failed to create shader"));
                }
            }
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create shader: {:?}",
                    err
                )))
            }
        };

        Ok(shader)
    }
}

impl Shader {
    ///ピクセルシェーダーの読み込み
    ///
    /// # Arguments
    /// *  'shader_file_path' - シェーダーファイルパス
    /// *  'entry_point_name' - エントリーポイント名
    ///
    /// # Returns
    /// *  'Ok(ID3DBlob)' - コンパイル済みシェーダーデータ
    /// *  'Err(Dx12Error)' - エラーメッセージ
    pub fn load_ps(shader_file_path: &str) -> std::result::Result<Shader, dx12error::Dx12Error> {
        let mut shader: Shader = Shader::default();
        match Shader::load_shader(shader_file_path, Shader_num::pixcel_shader) {
            Ok(blob) => shader.blob = Some(blob),
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create shader: {:?}",
                    err
                )))
            }
        }

        Ok(shader)
    }

    ///頂点シェーダーの読み込み
    ///
    /// # Arguments
    /// *  'shader_file_path' - シェーダーファイルパス
    /// *  'entry_point_name' - エントリーポイント名
    ///
    /// # Returns
    /// *  'Ok(ID3DBlob)' - コンパイル済みシェーダーデータ
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    pub fn load_vs(shader_file_path: &str) -> std::result::Result<Shader, dx12error::Dx12Error> {
        let mut shader: Shader = Shader::default();
        match Shader::load_shader(shader_file_path, Shader_num::vertex_shader) {
            Ok(blob) => shader.blob = Some(blob),
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create shader: {:?}",
                    err
                )))
            }
        }

        Ok(shader)
    }
}

/// get methods
impl Shader {
    pub fn get_shader(&self) -> std::result::Result<&ID3DBlob, dx12error::Dx12Error> {
        self.blob
            .as_ref()
            .ok_or_else(|| dx12error::Dx12Error::new("Failed to get shader: blob is none"))
    }
}

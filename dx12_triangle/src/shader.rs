use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D::Fxc::*, Win32::Graphics::Direct3D::*,
    Win32::Graphics::Direct3D12::*, Win32::Graphics::Dxgi::Common::*, Win32::Graphics::Dxgi::*,
    Win32::Graphics::Hlsl::*, Win32::System::LibraryLoader::*, Win32::System::Threading::*,
    Win32::UI::WindowsAndMessaging::*,
};

//エラー取得用 module
#[path = "./dx12error.rs"]
mod dx12error;

/// シェーダデータ
///
/// # Fields
/// *  'blob' - コンパイル済みシェーダーデータ
/// *  'dxc_blob' - DXCコンパイラを使用した時のシェーダーデータ
pub struct Shader {
    blob: Option<ID3DBlob>,
    dxc_blob: Option<ID3DBlob>,
}

impl Shader {
    fn load_shader(
        shader_file_path: &str,
        entry_point_name: &str,
        shader_model: &str,
    ) -> std::result::Result<ID3DBlob, dx12error::Dx12Error> {
        let mut error_blob: Option<ID3DBlob> = None;

        let compile_flags = if cfg!(debug_assertions) {
            DXGI_CREATE_FACTORY_DEBUG
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

        //エントリー関数名 文字列 変換
        //let entry_point: HSTRING = entry_point_name.into();
        let entry_point: HSTRING = shaders_hlsl.into();

        //シェーダーモデル 文字列 変換
        //let shader_model = "5_0";
        //let shader_model: HSTRING = shader_model.into();

        //シェーダー
        let mut shader = None;

        let shader = match unsafe {
            D3DCompileFromFile(
                &shaders_hlsl,
                None,
                D3D_COMPILE_STANDARD_FILE_INCLUDE,
                shader_model,
                compile_flags,
                0,
                &mut shader,
                error_blob,
            )
        } {
            Ok(_) => {
                if let Some(s) = shader.as_ref() {
                    s
                } else {
                    return Err(dx12error::Dx12Error::new(&format!(
                        "Failed to create shader: {:?}",
                        error_blob
                    )));
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
    pub fn load_ps(
        shader_file_path: &str,
        entry_point_name: &str,
    ) -> std::result::Result<ID3DBlob, dx12error::Dx12Error> {
        match Shader::load_shader(shader_file_path, entry_point_name, "ps_5_0") {
            Ok(blob) => Ok(blob),
            Err(err) => Err(err),
        }
    }

    ///頂点シェーダーの読み込み
    pub fn load_vs(
        shader_file_path: &str,
        entry_point_name: &str,
    ) -> std::result::Result<ID3DBlob, dx12error::Dx12Error> {
        match Shader::load_shader(shader_file_path, entry_point_name, "vs_5_0") {
            Ok(blob) => Ok(blob),
            Err(err) => Err(err),
        }
    }
}

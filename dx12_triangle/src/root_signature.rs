//エラー取得用 module
#[path = "./dx12error.rs"]
mod dx12error;

use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D::Fxc::*, Win32::Graphics::Direct3D::*,
    Win32::Graphics::Direct3D12::*, Win32::Graphics::Dxgi::Common::*, Win32::Graphics::Dxgi::*,
    Win32::System::LibraryLoader::*, Win32::System::Threading::*,
    Win32::UI::WindowsAndMessaging::*,
};

pub struct RootSignature {
    root_signature: Option<ID3D12RootSignature>,
}

/// RenderTargetの初期化
impl Default for RootSignature {
    fn default() -> Self {
        Self {
            root_signature: None,
        }
    }
}

impl RootSignature {
    pub fn create(
        device: &ID3D12Device,
    ) -> std::result::Result<RootSignature, dx12error::Dx12Error> {
        let mut rs: RootSignature = RootSignature::default();
        rs.root_signature = Some(RootSignature::create_root_signature(device)?);

        Ok(rs)
    }

    fn create_root_signature(
        device: &ID3D12Device,
    ) -> std::result::Result<ID3D12RootSignature, dx12error::Dx12Error> {
        let desc = D3D12_ROOT_SIGNATURE_DESC {
            Flags: D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
            ..Default::default()
        };
        let mut serialize_signature = None;
        let serialize_signature = match unsafe {
            D3D12SerializeRootSignature(
                &desc,
                D3D_ROOT_SIGNATURE_VERSION_1,
                &mut serialize_signature,
                None,
            )
        } {
            Ok(s) => {
                if let Some(sig) = serialize_signature.as_ref() {
                    sig
                } else {
                    return Err(dx12error::Dx12Error::new(
                        "Failed to serialize root signature",
                    ));
                }
            }
            Err(err) => {
                return Err(dx12error::Dx12Error::new(
                    "Failed to serialize root signature",
                ))
            }
        };

        match unsafe {
            device.CreateRootSignature(
                0,
                std::slice::from_raw_parts(
                    serialize_signature.GetBufferPointer() as _,
                    serialize_signature.GetBufferSize(),
                ),
            )
        } {
            Ok(s) => Ok(s),
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create root signature: {:?}",
                    err
                )))
            }
        }
    }
}

impl RootSignature {
    pub fn get_root_signature(
        &self,
    ) -> std::result::Result<&ID3D12RootSignature, dx12error::Dx12Error> {
        self.root_signature
            .as_ref()
            .ok_or_else(|| dx12error::Dx12Error::new("Failed to get root signature"))
    }
}

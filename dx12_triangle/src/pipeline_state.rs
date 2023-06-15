use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D::Fxc::*, Win32::Graphics::Direct3D::*,
    Win32::Graphics::Direct3D12::*, Win32::Graphics::Dxgi::Common::*, Win32::Graphics::Dxgi::*,
    Win32::System::LibraryLoader::*, Win32::System::Threading::*,
    Win32::UI::WindowsAndMessaging::*,
};

/// エラー取得用 module
#[path = "dx12error.rs"]
mod dx12error;

pub struct PipelineState {
    pipeline_state: Option<ID3D12PipelineState>,
}

impl Default for PipelineState {
    fn default() -> Self {
        Self {
            pipeline_state: None,
        }
    }
}

impl PipelineState {
    pub fn create(
        device: &ID3D12Device,
        desc: &D3D12_GRAPHICS_PIPELINE_STATE_DESC,
    ) -> std::result::Result<PipelineState, dx12error::Dx12Error> {
        let mut pso: PipelineState = PipelineState::default();

        match unsafe { device.CreateGraphicsPipelineState(desc) } {
            Ok(pipeline_state) => {
                pso.pipeline_state = Some(pipeline_state);
            }
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create pipeline state: {:?}",
                    err
                )))
            }
        }

        Ok(pso)
    }
}

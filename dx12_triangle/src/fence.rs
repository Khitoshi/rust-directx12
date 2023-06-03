#[path = "./dx12error.rs"]
mod dx12error;

use windows::{Win32::Foundation::*, Win32::Graphics::Direct3D12::*, Win32::System::Threading::*};

//GPUとCPUの同期を取るためのオブジェクト
pub struct Fence {
    //フェンス
    fence: Option<ID3D12Fence>,
    // フェンスの値
    fence_value: u64,
    // フェンスイベント
    fence_event: Option<HANDLE>,
}

impl Default for Fence {
    fn default() -> Self {
        Self {
            fence: None,
            fence_value: 0,
            fence_event: None,
        }
    }
}

impl Fence {
    //生成
    pub fn new(device: ID3D12Device) -> std::result::Result<Fence, dx12error::Dx12Error> {
        let mut fence: Fence = Fence::default();

        //fence生成
        fence.fence = match Fence::create_fence(device) {
            Ok(f) => Some(f),
            Err(err) => return Err(err),
        };

        //fenceの値設定
        fence.fence_value = Fence::create_fence_value();

        fence.fence_event = match Fence::create_fence_event() {
            Ok(event) => Some(event),
            Err(err) => return Err(err),
        };

        Ok(fence)
    }

    //fence生成
    fn create_fence(
        device: ID3D12Device,
    ) -> std::result::Result<ID3D12Fence, dx12error::Dx12Error> {
        //GPUと同期オブジェクト(fence)生成
        match unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_NONE) } {
            Ok(f) => return Ok(f),
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create fence: {:?}",
                    err
                )))
            }
        };
    }

    //フェンスの値設定
    fn create_fence_value() -> u64 {
        let value: u64 = 1;
        return value;
    }

    //フェンスイベント
    fn create_fence_event() -> std::result::Result<HANDLE, dx12error::Dx12Error> {
        //フェンス イベントの設置
        match unsafe { CreateEventA(None, false, false, None) } {
            Ok(event) => return Ok(event),
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create fence event: {:?}",
                    err
                )))
            }
        };
    }
}

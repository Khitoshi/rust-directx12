//エラー取得用 module
#[path = "./dx12error.rs"]
mod dx12error;

use windows::{Win32::Foundation::*, Win32::Graphics::Direct3D12::*, Win32::System::Threading::*};

/// GPUとCPUの同期を取るためのオブジェクト
///
/// # Fields
/// *  'fence' - フェンス
/// *  'fence_value' - フェンスの値
/// *  'fence_event' - フェンスイベント
pub struct Fence {
    fence: Option<ID3D12Fence>,
    value: Option<u64>,
    event: Option<HANDLE>,
}

/// Fenceの初期化
impl Default for Fence {
    fn default() -> Self {
        Self {
            fence: None,
            value: None,
            event: None,
        }
    }
}

impl Fence {
    /// Fenceの生成
    ///
    /// # Arguments
    /// *  'device' - デバイス
    ///
    /// # Returns
    /// *  'Ok(Fence)' - フェンス構造体
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    pub fn create(device: &ID3D12Device) -> std::result::Result<Fence, dx12error::Dx12Error> {
        let mut fence: Fence = Fence::default();

        //fence生成
        fence.fence = match Fence::create_fence(device) {
            Ok(f) => Some(f),
            Err(err) => return Err(err),
        };

        //fenceの値設定
        fence.value = Some(Fence::create_value());

        //イベント生成
        fence.event = match Fence::create_event() {
            Ok(event) => Some(event),
            Err(err) => return Err(err),
        };

        Ok(fence)
    }

    /// fence生成
    ///
    /// # Arguments
    /// *  'device' - デバイス
    ///
    /// # Returns
    /// *  'Ok(ID3D12Fence)' - フェンス
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    fn create_fence(
        device: &ID3D12Device,
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

    ///フェンスの値設定
    ///
    /// # Returns
    /// *  'u64' - フェンスの値
    ///
    fn create_value() -> u64 {
        let value: u64 = 1;
        value
    }

    ///フェンス イベントの生成
    ///
    /// # Returns
    /// *  'Ok(HANDLE)' - フェンス イベント
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    fn create_event() -> std::result::Result<HANDLE, dx12error::Dx12Error> {
        //フェンス イベントの生成
        match unsafe { CreateEventA(None, false, false, None) } {
            Ok(event) => Ok(event),
            Err(err) => Err(dx12error::Dx12Error::new(&format!(
                "Failed to create fence event: {:?}",
                err
            ))),
        }
    }
}

///add method
impl Fence {
    /// フェンスの値を加算
    ///
    /// # Returns
    /// *  'Ok(())' - 成功
    /// *  'Err(Dx12Error)' - エラーメッセージ
    pub fn add_value(&mut self) -> std::result::Result<(), dx12error::Dx12Error> {
        if let Some(v) = self.value.as_mut() {
            *v += 1;
            Ok(())
        } else {
            Err(dx12error::Dx12Error::new(
                "Failed to add fence value: value is None",
            ))
        }
    }
}

//get method
impl Fence {
    ///フェンスを取得
    /// # Returns
    /// *  'Ok(&ID3D12Fence)' - フェンス
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    pub fn get_fence(&self) -> std::result::Result<&ID3D12Fence, dx12error::Dx12Error> {
        self.fence
            .as_ref()
            .ok_or_else(|| dx12error::Dx12Error::new("Failed to get fence: fence is None"))
    }

    ///フェンスの値を取得
    ///
    /// # Returns
    /// *  'Ok(&u64)' - フェンスの値
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    pub fn get_value(&self) -> std::result::Result<&u64, dx12error::Dx12Error> {
        self.value.as_ref().ok_or_else(|| {
            dx12error::Dx12Error::new("Failed to get fence value: fence value is None")
        })
    }

    ///フェンスイベントを取得
    ///
    /// # Returns
    /// *  'Ok(&HANDLE)' - フェンスイベント
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    pub fn get_event(&self) -> std::result::Result<&HANDLE, dx12error::Dx12Error> {
        self.event.as_ref().ok_or_else(|| {
            dx12error::Dx12Error::new("Failed to get fence event: fence event is None")
        })
    }
}

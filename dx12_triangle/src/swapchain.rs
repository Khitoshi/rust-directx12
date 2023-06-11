//エラー取得用 module
#[path = "./dx12error.rs"]
mod dx12error;

//フレームバッファの数取得用
#[path = "./graphics_settings.rs"]
mod graphics_settings;

use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D12::*,
    Win32::Graphics::Dxgi::Common::*, Win32::Graphics::Dxgi::*,
};

/// スワップチェーン(フレームバッファを管理)
///
/// # Fields
/// *  'dxgi_swapchain' - スワップチェーン
/// *  'current_back_buffer_index' - 現在のバックバッファインデックス
///
pub struct SwapChain {
    dxgi_swapchain: Option<IDXGISwapChain4>,
    current_back_buffer_index: u32,
}

/// SwapChainの初期化
impl Default for SwapChain {
    fn default() -> Self {
        Self {
            dxgi_swapchain: None,
            current_back_buffer_index: 0,
        }
    }
}

/// SwapChainのcreate methods
impl SwapChain {
    /// スワップチェーンの生成
    /// # Arguments
    /// *  'factory' - ファクトリ
    /// *  'hwnd' - window handle
    /// *  'width' - フレームバッファ幅
    /// *  'height' - フレームバッファ高さ
    ///
    pub fn create(
        factory: &IDXGIFactory4,
        hwnd: &HWND,
        width: u32,
        height: u32,
        cmd_queue: &ID3D12CommandQueue,
    ) -> std::result::Result<SwapChain, dx12error::Dx12Error> {
        let mut sc: SwapChain = SwapChain::default();

        //DXGIスワップチェインを作成
        sc.dxgi_swapchain =
            match SwapChain::create_dxgi_swapchain(factory, &hwnd, width, height, cmd_queue) {
                Ok(sc) => Some(sc),
                Err(err) => {
                    return Err(dx12error::Dx12Error::new(&format!(
                        "Failed to create DXGI swap chain: {:?}",
                        err
                    )))
                }
            };

        //現在のバックバッファインデックスを取得
        sc.current_back_buffer_index = match sc.get_back_buffer_index() {
            Ok(index) => index,
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to get back buffer index: {:?}",
                    err
                )))
            }
        };

        return Ok(sc);
    }

    /// DXGIスワップチェインを作成
    ///
    /// # Arguments
    /// *  'factory' - ファクトリ
    /// *  'hwnd' - window handle
    /// *  'frame_buffer_width' - フレームバッファ幅
    /// *  'frame_buffer_height' - フレームバッファ高さ
    /// *  'cmd_queue' - コマンドキュー
    ///
    /// # Returns
    /// *  'Ok(IDXGISwapChain4)' - スワップチェイン
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    fn create_dxgi_swapchain(
        factory: &IDXGIFactory4,
        hwnd: &HWND,
        frame_buffer_width: u32,
        frame_buffer_hegith: u32,
        cmd_queue: &ID3D12CommandQueue,
    ) -> std::result::Result<IDXGISwapChain4, dx12error::Dx12Error> {
        //スワップチェインの設定
        let desc: DXGI_SWAP_CHAIN_DESC1 = DXGI_SWAP_CHAIN_DESC1 {
            BufferCount: graphics_settings::FRAME_BUFFER_COUNT,
            Width: frame_buffer_width,
            Height: frame_buffer_hegith,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            ..Default::default()
        };

        //TODO:フルスクリーンの設定
        let scaling = DXGI_SWAP_CHAIN_FULLSCREEN_DESC {
            Windowed: TRUE,
            ..Default::default()
        };
        let scaling_option: Option<*const DXGI_SWAP_CHAIN_FULLSCREEN_DESC> =
            Some(&scaling as *const _);

        //スワップチェイン1を作成
        let swap_chain1 = unsafe {
            factory.CreateSwapChainForHwnd(cmd_queue, *hwnd, &desc, scaling_option, None)
        }
        .map_err(|err| {
            dx12error::Dx12Error::new(&format!("Failed to create swap chain: {:?}", err))
        })?;

        //スワップチェイン4にキャスト
        swap_chain1.cast::<IDXGISwapChain4>().map_err(|err| {
            dx12error::Dx12Error::new(&format!("Failed to create swap chian:{:?}", err))
        })
    }

    //バックバッファインデックス取得

    /// 現在のバックバッファインデックスを取得
    ///
    /// # Returns
    /// *  'Ok(u32)' - バックバッファインデックス
    /// *  'Err(Dx12Error)' - エラーメッセージ
    fn get_back_buffer_index(&self) -> std::result::Result<u32, dx12error::Dx12Error> {
        //現在のバックバッファインデックスを取得
        if let Some(sc4) = self.dxgi_swapchain.as_ref() {
            Ok(unsafe { sc4.GetCurrentBackBufferIndex() })
        } else {
            return Err(dx12error::Dx12Error::new("Swap chain not initialized"));
        }
    }
}

/// public methods
impl SwapChain {
    /// バックバッファとフロントバッファを入れ替える
    /// # Arguments
    /// *  'device' - デバイス
    /// *  'back_buffer_index' - バックバッファインデックス
    ///
    /// # Returns
    /// *  'Ok(ID3D12Resource)' - バックバッファ
    /// *  'Err(Dx12Error)' - エラーメッセージ
    pub fn present(&mut self) -> std::result::Result<(), dx12error::Dx12Error> {
        if let Some(sc) = self.dxgi_swapchain.as_mut() {
            match unsafe { sc.Present(1, 0) }.ok() {
                Ok(()) => (),
                Err(err) => {
                    return Err(dx12error::Dx12Error::new(&format!(
                        "Failed to present: {:?}",
                        err
                    )))
                }
            }
        } else {
            return Err(dx12error::Dx12Error::new("Swap chain not initialized"));
        }

        Ok(())
    }
}

/// get method
impl SwapChain {
    /// スワップチェーン取得
    ///
    /// # Returns
    /// *  'Ok(IDXGISwapChain4)' - スワップチェーン
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    pub fn get_dxgi_swapchain(
        &self,
    ) -> std::result::Result<&IDXGISwapChain4, dx12error::Dx12Error> {
        self.dxgi_swapchain
            .as_ref()
            .ok_or_else(|| dx12error::Dx12Error::new("Swap chain not initialized"))
    }
}

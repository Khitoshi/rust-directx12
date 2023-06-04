#[path = "./dx12error.rs"]
mod dx12error;

#[path = "./graphics_settings.rs"]
mod graphics_settings;

use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D12::*,
    Win32::Graphics::Dxgi::Common::*, Win32::Graphics::Dxgi::*,
};

// スワップチェーン (フレームバッファを管理)
pub struct SwapChain {
    // スワップチェーン
    dxgi_swapchain: Option<IDXGISwapChain4>,
    // 現在のバッグバッファインデックス
    current_back_buffer_index: u32,
}

impl Default for SwapChain {
    fn default() -> Self {
        Self {
            dxgi_swapchain: None,
            current_back_buffer_index: 0,
        }
    }
}

impl SwapChain {
    //生成
    pub fn new(
        factory: IDXGIFactory4,
        hwnd: &HWND,
        frame_buffer_width: u32,
        frame_buffer_height: u32,
        cmd_queue: ID3D12CommandQueue,
    ) -> std::result::Result<SwapChain, dx12error::Dx12Error> {
        let mut sc: SwapChain = SwapChain::default();

        //DXGIスワップチェインを作成
        sc.dxgi_swapchain = match SwapChain::create_dxgi_swapchain(
            factory,
            &hwnd,
            frame_buffer_width,
            frame_buffer_height,
            cmd_queue,
        ) {
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

    //スワップチェイン作成
    fn create_dxgi_swapchain(
        factory: IDXGIFactory4,
        hwnd: &HWND,
        frame_buffer_width: u32,
        frame_buffer_hegith: u32,
        cmd_queue: ID3D12CommandQueue,
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

        //スワップチェイン1を作成
        let mut swap_chain1: Option<IDXGISwapChain1> =
            match unsafe { factory.CreateSwapChainForHwnd(&cmd_queue, *hwnd, &desc, None, None) } {
                Ok(sc) => Some(sc),
                Err(err) => {
                    return Err(dx12error::Dx12Error::new(&format!(
                        "Failed to create swap chain: {:?}",
                        err
                    )));
                }
            };

        //swapchain1 を swapchain4に変換する
        if let Some(ref sc1) = swap_chain1.as_ref() {
            match sc1.cast::<IDXGISwapChain4>() {
                Ok(sc) => return Ok(sc),
                Err(err) => {
                    return Err(dx12error::Dx12Error::new(&format!(
                        "Failed to create swap chian:{:?}",
                        err
                    )))
                }
            };
        } else {
            return Err(dx12error::Dx12Error::new("Failed to create swap chian"));
        }
    }

    //バックバッファインデックス取得
    fn get_back_buffer_index(&self) -> std::result::Result<u32, dx12error::Dx12Error> {
        //現在のバックバッファインデックスを取得
        if let Some(sc4) = self.dxgi_swapchain.clone() {
            Ok(unsafe { sc4.GetCurrentBackBufferIndex() })
        } else {
            return Err(dx12error::Dx12Error::new("Swap chain not initialized"));
        }
    }
}

//
impl SwapChain {
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

//get method
impl SwapChain {
    pub fn get_dxgi_swapchain(&self) -> std::result::Result<IDXGISwapChain4, dx12error::Dx12Error> {
        if let Some(sc) = self.dxgi_swapchain.as_ref() {
            Ok(sc.clone())
        } else {
            return Err(dx12error::Dx12Error::new("Swap chain not initialized"));
        }
    }
}

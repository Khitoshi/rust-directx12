//エラー取得用 module
#[path = "./dx12error.rs"]
mod dx12error;

#[path = "./graphics_settings.rs"]
mod graphics_settings;

use windows::{Win32::Graphics::Direct3D12::*, Win32::Graphics::Dxgi::*};

/// レンダリングターゲット(描画結果を保存)
///
/// # Fields
/// *  'rtv_heap' - レンダリングターゲットビューのディスクリプタヒープ
/// *  'rtv_descriptor_size' - レンダリングターゲットビューのサイズ
/// *  'render_targets' - レンダリングターゲット
///
pub struct RenderTarget {
    rtv_heap: Option<ID3D12DescriptorHeap>,
    rtv_descriptor_size: u32,
    render_targets: Vec<ID3D12Resource>,
}

/// RenderTargetの初期化
impl Default for RenderTarget {
    fn default() -> Self {
        Self {
            rtv_heap: None,
            rtv_descriptor_size: 0,
            render_targets: Vec::new(),
        }
    }
}

/// RenderTargetのcreate methods
impl RenderTarget {
    /// レンダリングターゲットの生成
    ///
    /// # Arguments
    /// *  'device' - デバイス
    /// *  'swapchain' - スワップチェーン
    ///
    /// # Returns
    /// *  'Ok(RenderTarget)' - レンダリングターゲット
    /// *  'Err(Dx12Error)' - エラーメッセージ
    pub fn create(
        device: &ID3D12Device,
        swapchain: &IDXGISwapChain4,
    ) -> std::result::Result<RenderTarget, dx12error::Dx12Error> {
        let mut rt: RenderTarget = RenderTarget::default();

        rt.rtv_heap = Some(RenderTarget::create_descriptor_heap_for_frame_buffer(
            device,
        )?);

        rt.rtv_descriptor_size = RenderTarget::get_descriptor_handle_increment_size(device);

        rt.render_targets = rt.create_for_fame_buffer(swapchain, device)?;

        return Ok(rt);
    }

    /// レンダリングターゲットビューのディスクリプタヒープ生成
    ///
    /// # Arguments
    /// *  'device' - デバイス
    ///
    /// # Returns
    /// *  'Ok(ID3D12DescriptorHeap)' - レンダリングターゲットビューのディスクリプタヒープ
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    fn create_descriptor_heap_for_frame_buffer(
        device: &ID3D12Device,
    ) -> std::result::Result<ID3D12DescriptorHeap, dx12error::Dx12Error> {
        //レンダリングターゲットビューのディスクリプタヒープ用のディスクリプタヒープデスク
        let desc: D3D12_DESCRIPTOR_HEAP_DESC = D3D12_DESCRIPTOR_HEAP_DESC {
            NumDescriptors: graphics_settings::FRAME_BUFFER_COUNT,
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            ..Default::default()
        };

        //レンダリングターゲットビューのディスクリプタヒープ作成
        match unsafe { device.CreateDescriptorHeap(&desc) } {
            Ok(rtv_heap) => Ok(rtv_heap),
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create rtv descriptor heap: {:?}",
                    err
                )))
            }
        }
    }

    /// レンダリングターゲットビューのサイズ 取得
    ///
    /// # Arguments
    /// *  'device' - デバイス
    ///
    /// # Returns
    /// *  'u32' - レンダリングターゲットビューのサイズ
    ///
    fn get_descriptor_handle_increment_size(device: &ID3D12Device) -> u32 {
        unsafe { device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV) }
    }

    /// レンダリングターゲットビューの作成
    ///
    /// # Arguments
    /// *  'swap_chain' - スワップチェーン
    /// *  'device' - デバイス
    ///
    /// # Returns
    /// *  'Ok(Vec<ID3D12Resource>)' - レンダリングターゲットビュー
    /// *  'Err(Dx12Error)' - エラーメッセージ
    fn create_for_fame_buffer(
        &self,
        swap_chain: &IDXGISwapChain4,
        device: &ID3D12Device,
    ) -> std::result::Result<Vec<ID3D12Resource>, dx12error::Dx12Error> {
        //レンダリングターゲットビューのディスクリプタヒープを取得
        let rtv_heap = self
            .rtv_heap
            .as_ref()
            .ok_or(dx12error::Dx12Error::new("No RTV heap"))?;
        //レンダリングターゲットビューのディスクリプタヒープの先頭のハンドルを取得
        let mut rtv_handle = unsafe { rtv_heap.GetCPUDescriptorHandleForHeapStart() };

        //フロントバッファをバックバッファ用のRTVを作成
        let mut render_targets: Vec<ID3D12Resource> = Vec::new();
        for i in 0..graphics_settings::FRAME_BUFFER_COUNT {
            //バッファ所得
            match unsafe { swap_chain.GetBuffer(i as u32) } {
                Ok(buffer) => {
                    //バッファを保存
                    render_targets.push(buffer);
                    //レンダーターゲットビュー生成
                    if let Some(rt) = render_targets.last() {
                        unsafe { device.CreateRenderTargetView(rt, None, rtv_handle) }
                    }
                    //ポインタを渡したのでずらす
                    rtv_handle.ptr += self.rtv_descriptor_size as usize;
                }
                Err(err) => {
                    return Err(dx12error::Dx12Error::new(&format!(
                        "Failed to create_rtv_for_fame_buffer: {:?}",
                        err
                    )))
                }
            };
        }

        Ok(render_targets)
    }
}

/// get methods
impl RenderTarget {
    /// レンダリングターゲットビューのディスクリプタヒープを取得
    ///
    /// # Returns
    /// *  'Ok(&ID3D12DescriptorHeap)' - レンダリングターゲットビューのディスクリプタヒープ
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    pub fn get_heap(&self) -> std::result::Result<&ID3D12DescriptorHeap, dx12error::Dx12Error> {
        self.rtv_heap
            .as_ref()
            .ok_or_else(|| dx12error::Dx12Error::new("No RTV heap"))
    }

    /// レンダリングターゲットビューのディスクリプタヒープを取得
    ///
    /// # Arguments
    /// *  'frame_buffer_index' - フレームバッファインデックス
    ///
    /// # Returns
    /// *  'Ok(D3D12_CPU_DESCRIPTOR_HANDLE)' - レンダリングターゲットビューのディスクリプタヒープ
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    pub fn get_current_frame_buffer(
        &self,
        frame_buffer_index: usize,
    ) -> std::result::Result<D3D12_CPU_DESCRIPTOR_HANDLE, dx12error::Dx12Error> {
        //レンダリングターゲットビューのディスクリプタヒープを取得
        let rtv_heap = self
            .rtv_heap
            .as_ref()
            .ok_or(dx12error::Dx12Error::new("No RTV heap"))?;

        let mut rtv_handle = unsafe { rtv_heap.GetCPUDescriptorHandleForHeapStart() };
        //ポインタを渡したのでずらす
        rtv_handle.ptr += frame_buffer_index * self.rtv_descriptor_size as usize;
        Ok(rtv_handle)
    }

    /// レンダリングターゲットビューのサイズ 取得
    ///
    /// # Returns
    /// *  'u32' - レンダリングターゲットビューのサイズ
    pub fn get_descriptor_size(&self) -> u32 {
        self.rtv_descriptor_size
    }

    /// レンダリングターゲットビューの取得
    ///
    /// # Returns
    /// *  'Vec<ID3D12Resource>' - レンダリングターゲットビュー
    ///
    pub fn get_render_targets(&self) -> &Vec<ID3D12Resource> {
        self.render_targets.as_ref()
    }

    /// レンダリングターゲットビューの取得
    ///
    /// # Arguments
    /// *  'num' - レンダリングターゲットビューのインデックス
    ///
    /// # Returns
    /// *  'ID3D12Resource' - レンダリングターゲットビュー
    ///
    pub fn get_render_target(&self, num: usize) -> &ID3D12Resource {
        &self.render_targets[num]
    }
}

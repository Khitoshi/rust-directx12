//エラー取得用 module
#[path = "./dx12error.rs"]
mod dx12error;

use windows::{Win32::Graphics::Direct3D12::*, Win32::Graphics::Dxgi::Common::*};

/// 深度バッファやステンシルバッファを管理
///
/// # Fields
/// *  'dsv_heap' - 深度ステンシルビューのディスクリプタヒープ
/// *  'dsv_descriptor_size' - 深度ステンシルビューのサイズ
/// *  'depth_stencil_buffer' - 深度ステンシルバッファ
///
pub struct DepthStencil {
    dsv_heap: Option<ID3D12DescriptorHeap>,
    dsv_descriptor_size: Option<u32>,
    depth_stencil_buffer: Option<ID3D12Resource>,
}

/// 深度ステンシルの初期化
impl Default for DepthStencil {
    fn default() -> Self {
        Self {
            dsv_heap: None,
            dsv_descriptor_size: None,
            depth_stencil_buffer: None,
        }
    }
}

/// DepthStencilのcreate methods
///
impl DepthStencil {
    /// 深度ステンシルの生成
    ///
    /// # Arguments
    /// *  'device' - デバイス
    /// *  'width' - 幅
    /// *  'height' - 高さ
    ///
    /// # Returns
    /// * Ok(DepthStencil) - 深度ステンシル
    /// * Err(Dx12Error) - エラーメッセージ
    pub fn create(
        device: &ID3D12Device,
        width: u64,
        height: u32,
    ) -> std::result::Result<DepthStencil, dx12error::Dx12Error> {
        let mut ds: DepthStencil = DepthStencil::default();

        ds.dsv_heap = match DepthStencil::create_view_heap(device) {
            Ok(heap) => Some(heap),
            Err(err) => return Err(err),
        };

        ds.depth_stencil_buffer = match DepthStencil::create_for_frame_buffer(device, width, height)
        {
            Ok(buffer) => Some(buffer),
            Err(err) => return Err(err),
        };

        return Ok(ds);
    }

    /// 深度ステンシルビューのディスクリプタヒープ生成
    ///
    /// # Arguments
    /// *  'device' - デバイス
    ///
    /// # Returns
    /// *  'Ok(ID3D12DescriptorHeap)' - 深度ステンシルビューのディスクリプタヒープ
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    fn create_view_heap(
        device: &ID3D12Device,
    ) -> std::result::Result<ID3D12DescriptorHeap, dx12error::Dx12Error> {
        //深度ステンシルビューのディスクリプタヒープ用のディスクリプタヒープデスクを作成
        let desc: D3D12_DESCRIPTOR_HEAP_DESC = D3D12_DESCRIPTOR_HEAP_DESC {
            NumDescriptors: 1,
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            ..Default::default()
        };

        //深度ステンシルビューのディスクリプタヒープ作成
        match unsafe { device.CreateDescriptorHeap::<ID3D12DescriptorHeap>(&desc) } {
            Ok(dsv) => Ok(dsv),
            Err(err) => Err(dx12error::Dx12Error::new(&format!(
                "Failed to create dsv descriptor heap: {:?}",
                err
            ))),
        }
    }

    /// フレームバッファ用の深度ステンシルバッファの生成
    ///
    /// # Arguments
    /// *  'device' - デバイス
    /// *  'width' - 幅
    /// *  'height' - 高さ
    ///
    /// # Returns
    /// *  'Ok(ID3D12Resource)' - 深度ステンシルバッファ
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    fn create_for_frame_buffer(
        device: &ID3D12Device,
        width: u64,
        height: u32,
    ) -> std::result::Result<ID3D12Resource, dx12error::Dx12Error> {
        //画面クリア値設定
        let dsv_clear_value = D3D12_CLEAR_VALUE {
            Format: DXGI_FORMAT_D32_FLOAT,
            Anonymous: D3D12_CLEAR_VALUE_0 {
                DepthStencil: D3D12_DEPTH_STENCIL_VALUE {
                    Depth: 1.0,
                    Stencil: 0,
                },
            },
        };

        //深度ステンシルバッファの設定
        let desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            Alignment: 0,
            Width: width,
            Height: height,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: DXGI_FORMAT_D32_FLOAT,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
            Flags: D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL
                | D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE,
        };

        //ヒーププロパティ
        let heap_prop = D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_DEFAULT,
            ..Default::default()
        };

        //深度ステンシルバッファ生成
        let mut depth_stencil_buffer: Option<ID3D12Resource> = None;
        match unsafe {
            device.CreateCommittedResource(
                &heap_prop,
                D3D12_HEAP_FLAG_NONE,
                &desc,
                D3D12_RESOURCE_STATE_DEPTH_WRITE,
                Some(&dsv_clear_value),
                &mut depth_stencil_buffer,
            )
        } {
            Ok(_) => (),
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create depth stencil buffer: {:?}",
                    err
                )))
            }
        }

        depth_stencil_buffer
            .ok_or_else(|| dx12error::Dx12Error::new("Failed to get depth stencil view heap"))
    }
}

/// DepthStencilのfield取得
///
#[allow(dead_code)]
impl DepthStencil {
    /// 深度ステンシルビューのサイズを取得
    ///
    /// # Returns
    /// *  'Ok(u32)' - 深度ステンシルビューのサイズ
    /// *  'Err(Dx12Error)' - エラーメッセージ
    pub fn get_descriptor_size(&self) -> std::result::Result<u32, dx12error::Dx12Error> {
        self.dsv_descriptor_size.ok_or_else(|| {
            dx12error::Dx12Error::new("Failed to get depth stencil view size: size is none")
        })
    }

    /// 深度ステンシルビューのディスクリプタヒープを取得
    ///
    /// # Returns
    /// *  'Ok(&ID3D12DescriptorHeap)' - 深度ステンシルビューのディスクリプタヒープ
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    pub fn get_heap(&self) -> std::result::Result<&ID3D12DescriptorHeap, dx12error::Dx12Error> {
        self.dsv_heap.as_ref().ok_or_else(|| {
            dx12error::Dx12Error::new("Failed to get depth stencil view heap: heap is none")
        })
    }

    /// 深度ステンシルビューのディスクリプタヒープの開始位置を取得
    ///
    /// # Returns
    /// *  'Ok(D3D12_CPU_DESCRIPTOR_HANDLE)' - 深度ステンシルビューのディスクリプタヒープの開始位置
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    pub fn get_heap_start(
        &self,
    ) -> std::result::Result<D3D12_CPU_DESCRIPTOR_HANDLE, dx12error::Dx12Error> {
        self.dsv_heap
            .as_ref()
            .ok_or_else(|| {
                dx12error::Dx12Error::new(
                    "Failed to get depth stencil view heap start : heap start is none",
                )
            })
            .map(|dsvh| unsafe { dsvh.GetCPUDescriptorHandleForHeapStart() })
    }

    /// 深度ステンシルバッファを取得
    ///
    /// # Returns
    /// *  'Ok(&ID3D12Resource)' - 深度ステンシルバッファ
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    pub fn get_buffer(&self) -> std::result::Result<&ID3D12Resource, dx12error::Dx12Error> {
        self.depth_stencil_buffer.as_ref().ok_or_else(|| {
            dx12error::Dx12Error::new("Failed to get depth stencil buffer : buffer is none")
        })
    }
}

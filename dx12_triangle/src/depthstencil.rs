#[path = "./dx12error.rs"]
mod dx12error;

use windows::{Win32::Graphics::Direct3D12::*, Win32::Graphics::Dxgi::Common::*};

// 深度ステンシル (深度バッファやステンシルバッファを管理)
pub struct DepthStencil {
    // 深度ステンシルビューのディスクリプタヒープ
    dsv_heap: Option<ID3D12DescriptorHeap>,
    // 深度ステンシルビューのサイズ
    dsv_descriptor_size: u32,
    // 深度ステンシルバッファ
    depth_stencil_buffer: Option<ID3D12Resource>,
}

impl Default for DepthStencil {
    fn default() -> Self {
        Self {
            dsv_heap: None,
            dsv_descriptor_size: 0,
            depth_stencil_buffer: None,
        }
    }
}

impl DepthStencil {
    //DepthStencil生成
    pub fn new(
        deivce: ID3D12Device,
        frame_buffer_width: u64,
        frame_buffer_height: u32,
    ) -> std::result::Result<DepthStencil, dx12error::Dx12Error> {
        let mut ds: DepthStencil = DepthStencil::default();

        ds.dsv_heap = match DepthStencil::create_depth_stencil_view_heap(deivce.clone()) {
            Ok(heap) => Some(heap),
            Err(err) => return Err(err),
        };

        ds.depth_stencil_buffer = match DepthStencil::create_dsv_for_fame_buffer(
            deivce.clone(),
            frame_buffer_width,
            frame_buffer_height,
        ) {
            Ok(buffer) => Some(buffer),
            Err(err) => return Err(err),
        };

        return Ok(ds);
    }

    // 深度ステンシルビューのディスクリプタヒープ生成
    fn create_depth_stencil_view_heap(
        device: ID3D12Device,
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
            Ok(dsv) => return Ok(dsv),
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create dsv descriptor heap: {:?}",
                    err
                )))
            }
        }
    }

    //フレームバッファ用の深度ステンシルバッファの生成
    fn create_dsv_for_fame_buffer(
        device: ID3D12Device,
        frame_buffer_width: u64,
        frame_buffer_height: u32,
    ) -> std::result::Result<ID3D12Resource, dx12error::Dx12Error> {
        //画面クリア値設定
        let dsv_clear_value: D3D12_CLEAR_VALUE = D3D12_CLEAR_VALUE {
            Format: DXGI_FORMAT_D32_FLOAT,
            Anonymous: D3D12_CLEAR_VALUE_0 {
                DepthStencil: D3D12_DEPTH_STENCIL_VALUE {
                    Depth: 1.0,
                    Stencil: 0,
                },
            },
        };

        //リソースのデスク
        let desc: D3D12_RESOURCE_DESC = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            Alignment: 0,
            Width: frame_buffer_width,
            Height: frame_buffer_height,
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

        //深度ステンシルバッファ生成
        let heap_prop: D3D12_HEAP_PROPERTIES = D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_DEFAULT,
            ..Default::default()
        };
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
        //Failed to create depth stencil buffer:
        //生成したbufferをチェックしてからreturn
        if let Some(ref dsb) = depth_stencil_buffer {
            println!("depth stencil buffer creation succeeded");
            Ok(dsb.clone())
        } else {
            Err(dx12error::Dx12Error::new(
                "Failed to create depth stencil buffer",
            ))
        }
    }
}

//ゲットmethod
impl DepthStencil {
    //深度ステンシルビューのサイズを取得
    pub fn get_dsv_descriptor_size(&self) -> u32 {
        self.dsv_descriptor_size
    }

    //深度ステンシルビューのディスクリプタヒープを取得
    pub fn get_dsv_heap(&self) -> std::result::Result<ID3D12DescriptorHeap, dx12error::Dx12Error> {
        if let Some(dsvh) = self.dsv_heap.clone() {
            Ok(dsvh)
        } else {
            Err(dx12error::Dx12Error::new(
                "Failed to get depth stencil view heap",
            ))
        }
    }

    //深度ステンシルバッファを取得
    pub fn get_depth_stencil_buffer(
        &self,
    ) -> std::result::Result<ID3D12Resource, dx12error::Dx12Error> {
        if let Some(dsb) = self.depth_stencil_buffer.clone() {
            Ok(dsb)
        } else {
            Err(dx12error::Dx12Error::new(
                "Failed to get depth stencil buffer",
            ))
        }
    }
}

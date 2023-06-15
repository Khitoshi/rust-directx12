use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D::Fxc::*, Win32::Graphics::Direct3D::*,
    Win32::Graphics::Direct3D12::*, Win32::Graphics::Dxgi::Common::*, Win32::Graphics::Dxgi::*,
    Win32::System::LibraryLoader::*, Win32::System::Threading::*,
    Win32::UI::WindowsAndMessaging::*,
};

/// エラー取得用 module
#[path = "dx12error.rs"]
mod dx12error;
use std::slice;

pub struct IndexBuffer {
    buffer: Option<ID3D12Resource>,
    buffer_view: D3D12_INDEX_BUFFER_VIEW,
    count: usize,
    stride_in_bytes: usize,
    size_in_bytes: usize,
}

impl Default for IndexBuffer {
    fn default() -> Self {
        Self {
            buffer: None,
            buffer_view: D3D12_INDEX_BUFFER_VIEW {
                ..Default::default()
            },
            count: 0,
            stride_in_bytes: 0,
            size_in_bytes: 0,
        }
    }
}

impl IndexBuffer {
    pub fn create(
        deivce: &ID3D12Device,
        size: usize,
        stride: usize,
    ) -> std::result::Result<IndexBuffer, dx12error::Dx12Error> {
        let mut ib: IndexBuffer = IndexBuffer::default();

        ib.size_in_bytes = if stride == 2 { size * 2 } else { size };

        let heap_prop = D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_UPLOAD,
            ..Default::default()
        };

        let resouce_desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
            Alignment: D3D12_DEFAULT_RESOURCE_PLACEMENT_ALIGNMENT as u64,
            Width: ib.size_in_bytes as u64,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: DXGI_FORMAT_UNKNOWN, // Format isn't used for buffers
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: D3D12_RESOURCE_FLAG_NONE,
        };

        match unsafe {
            deivce.CreateCommittedResource(
                &heap_prop,
                D3D12_HEAP_FLAG_NONE,
                &resouce_desc,
                D3D12_RESOURCE_STATE_GENERIC_READ,
                None,
                &mut ib.buffer,
            )
        } {
            Ok(_) => {}
            Err(err) => {
                return Err(dx12error::Dx12Error::new(&format!(
                    "Failed to create index buffer: {:?}",
                    err
                )))
            }
        }

        ib.buffer_view = D3D12_INDEX_BUFFER_VIEW {
            BufferLocation: unsafe { ib.buffer.as_ref().unwrap().GetGPUVirtualAddress() },
            SizeInBytes: ib.size_in_bytes as u32,
            Format: if stride == 2 {
                DXGI_FORMAT_R16_UINT
            } else {
                DXGI_FORMAT_R32_UINT
            },
        };

        ib.count = ib.size_in_bytes / ib.stride_in_bytes;

        Ok(ib)
    }

    pub fn copy(&mut self, indecies: &[u16]) {
        unsafe {
            let mut data_ptr = std::ptr::null_mut::<std::ffi::c_void>();

            if let Some(buf) = &mut self.buffer {
                let hr = buf.Map(0, None, Some(&mut data_ptr));
                if hr.is_ok() {
                    let data_slice =
                        slice::from_raw_parts_mut(data_ptr as *mut u32, indecies.len());
                    for (i, &src_index) in indecies.iter().enumerate() {
                        data_slice[i] = src_index as u32;
                    }

                    buf.Unmap(0, None);
                }
            }
        }
    }
}

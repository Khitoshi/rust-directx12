#[path = "./dx12error.rs"]
mod dx12error;
use dx12error::Dx12Error;

use windows::{Win32::Foundation::*, Win32::Graphics::Direct3D12::*};

//ディスクリプタヒープ
const MAX_DESCRIPTOR_HEAP: u32 = 4;
//定数バッファの最大数
const MAX_CONSTANT_BUFFER: u32 = 8;
//シェーダーリソースの最大数
const MAX_SHADER_RESOURCE: u32 = 16;

pub struct RenderContext {
    //コマンドリスト
    command_list: ID3D12GraphicsCommandList4,
    //現在のビューポート
    current_viewport: D3D12_VIEWPORT,

    //ディスクリプタヒープ
    descriptor_Heap_: [ID3D12DescriptorHeap; MAX_DESCRIPTOR_HEAP as usize],
    //定数バッファ
    //constant_Buffer : [ConstantBuffer:MAX_CONSTANT_BUFFER],
    scratch_resource_list: Vec<ID3D12Resource>,
}

impl RenderContext {
    //リセット処理
    pub fn reset(
        &mut self,
        cmd_allocator: &ID3D12CommandAllocator,
        pipelineState: &ID3D12PipelineState,
    ) -> std::result::Result<(), Dx12Error> {
        match unsafe { self.command_list.Reset(cmd_allocator, pipelineState) } {
            Ok(()) => (),
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to reset RenderContext: {:?}",
                    err
                )))
            }
        }
        self.scratch_resource_list.clear();

        Ok(())
    }

    //ビューポートとシザリング矩形をセットで設定
    pub fn set_viewport_and_scissor(&mut self, vp: D3D12_VIEWPORT) {
        //シザリング矩形
        let scissor_rect: RECT = RECT {
            top: 0,
            bottom: vp.Width as i32,
            left: 0,
            right: vp.Height as i32,
        };

        //シザリング矩形を設定
        self.set_scissor_rect(scissor_rect);

        //ビューポートをコマンドリストに設定
        unsafe { self.command_list.RSSetViewports(&[vp]) };

        //viewポート設定
        self.current_viewport = vp;
    }

    //シザリング矩形 設定
    fn set_scissor_rect(&mut self, rc: RECT) {
        unsafe { self.command_list.RSSetScissorRects(&[rc]) };
    }

    //レンダリングターゲットとして使用可能になるまで待機
    /*
    pub fn wait_until_to_possible_set_render_target(rt_num: usize) {
        for i in 0..rt_num {}
    }

    fn WaitUntilToPossibleSetRenderTarget() {}
    fn WaitUntilToPossibleSetRenderTarget() {}
    */

    //レンダリングターゲットのクリア
    pub fn clear_render_target(
        &mut self,
        rtv_handle: D3D12_CPU_DESCRIPTOR_HANDLE,
        clear_color: [f32; 4],
    ) {
        unsafe {
            self.command_list
                .ClearRenderTargetView(rtv_handle, &*clear_color.as_ptr(), None);
        };
    }

    //デプスステンシルビューをクリア
    pub fn clear_depth_stencil_view(
        &mut self,
        dsv_handle: D3D12_CPU_DESCRIPTOR_HANDLE,
        clear_value: f32,
    ) {
        let rect_arr: &[RECT] = &[];
        unsafe {
            self.command_list.ClearDepthStencilView(
                dsv_handle,
                D3D12_CLEAR_FLAG_DEPTH | D3D12_CLEAR_FLAG_STENCIL,
                clear_value,
                0,
                rect_arr,
            );
        };
    }
}

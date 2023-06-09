#[path = "./dx12error.rs"]
mod dx12error;

use std::cell::RefCell;
use std::rc::Rc;
use windows::{Win32::Foundation::*, Win32::Graphics::Direct3D12::*};

pub struct RenderContext {
    //コマンドリスト
    command_list: Option<Rc<RefCell<ID3D12GraphicsCommandList4>>>,
    //現在のビューポート
    current_viewport: D3D12_VIEWPORT,
    //スクラッチリソースのリスト
    scratch_resource_list: Vec<ID3D12Resource>,
}

impl Default for RenderContext {
    fn default() -> Self {
        Self {
            command_list: None,
            current_viewport: D3D12_VIEWPORT {
                TopLeftX: 0.0,
                TopLeftY: 0.0,
                Width: 0.0,
                Height: 0.0,
                MinDepth: 0.0,
                MaxDepth: 0.0,
            },
            scratch_resource_list: Vec::new(),
        }
    }
}
#[allow(dead_code)]
impl RenderContext {
    //生成
    pub fn new(
        cmd_list: Rc<RefCell<ID3D12GraphicsCommandList4>>,
    ) -> std::result::Result<RenderContext, dx12error::Dx12Error> {
        let mut rc: RenderContext = RenderContext::default();
        rc.command_list = Some(cmd_list);
        return Ok(rc);
    }

    pub fn reset(
        &mut self,
        cmd_allocator: &ID3D12CommandAllocator,
        pipeline_state: Option<&ID3D12PipelineState>,
    ) -> std::result::Result<(), dx12error::Dx12Error> {
        //コマンドリストをリセット
        if let Some(cmd_list) = self.command_list.as_mut() {
            unsafe {
                if let Some(pipeline_state) = pipeline_state {
                    match cmd_list.borrow_mut().Reset(cmd_allocator, pipeline_state) {
                        Ok(_) => (),
                        Err(err) => {
                            return Err(dx12error::Dx12Error::new(&format!(
                                "Failed to reset command list: {:?}",
                                err
                            )))
                        }
                    }
                } else {
                    return Err(dx12error::Dx12Error::new(&format!(
                        "Failed to reset command list: {:?}",
                        "pipeline state is none"
                    )));
                }
            }
        } else {
            return Err(dx12error::Dx12Error::new(&format!(
                "Failed to reset command list: {:?}",
                "command list is none"
            )));
        }

        //スクラッチリソースをクリア
        self.scratch_resource_list.clear();

        Ok(())
    }

    //ビューポートとシザリング矩形をセットで設定
    pub fn set_viewport_and_scissor(
        &mut self,
        viewport: D3D12_VIEWPORT,
    ) -> std::result::Result<(), dx12error::Dx12Error> {
        //シザリング矩形設定
        let scissor_rect: RECT = RECT {
            left: 0,
            top: 0,
            bottom: viewport.clone().Height as i32,
            right: viewport.clone().Width as i32,
        };
        match self.set_scissor_rect(scissor_rect) {
            Ok(_) => (),
            Err(err) => return Err(err),
        }

        //ビューポート設定
        if let Some(cmd_list) = self.command_list.as_mut() {
            unsafe {
                cmd_list.borrow_mut().RSSetViewports(&[viewport.clone()]);
            }
        } else {
            return Err(dx12error::Dx12Error::new(&format!(
                "Failed to set RSSetViewports: {:?}",
                "command list is none"
            )));
        }

        //現在のビューポートを更新
        self.current_viewport = viewport.clone();
        Ok(())
    }

    //バックバッファがレンダリングターゲットとして設定可能になるまで待つ
    pub fn wait_until_to_possible_set_render_target(
        &mut self,
        resouce: &ID3D12Resource,
    ) -> std::result::Result<(), dx12error::Dx12Error> {
        //状態遷移設定
        let barrier = RenderContext::transient_barrier(
            &resouce,
            D3D12_RESOURCE_STATE_PRESENT,
            D3D12_RESOURCE_STATE_RENDER_TARGET,
        );

        //画像を表示が終わったら描画状態に遷移させる
        if let Some(cmd_list) = self.command_list.as_ref() {
            unsafe { cmd_list.borrow_mut().ResourceBarrier(&[barrier]) };
        } else {
            return Err(dx12error::Dx12Error::new(&format!(
                "Failed to set RSSetViewports: {:?}",
                "command list is none"
            )));
        }

        Ok(())
    }

    //レンダリングターゲットへの描き込み待ち
    pub fn wait_until_finish_drawing_to_render_target(
        &mut self,
        resouce: &ID3D12Resource,
    ) -> std::result::Result<(), dx12error::Dx12Error> {
        let barrier = RenderContext::transient_barrier(
            resouce,
            D3D12_RESOURCE_STATE_RENDER_TARGET,
            D3D12_RESOURCE_STATE_PRESENT,
        );

        //描画が終わった後，画像を表示するための状態に遷移させる
        if let Some(cmd_list) = self.command_list.as_ref() {
            unsafe { cmd_list.borrow_mut().ResourceBarrier(&[barrier]) };
        } else {
            return Err(dx12error::Dx12Error::new(&format!(
                "Failed to set RSSetViewports: {:?}",
                "command list is none"
            )));
        }

        Ok(())
    }

    //レンダリングターゲットのクリア
    pub fn clear_render_target_view(
        &mut self,
        rtv_handle: D3D12_CPU_DESCRIPTOR_HANDLE,
        clear_color: [f32; 4],
    ) -> std::result::Result<(), dx12error::Dx12Error> {
        if let Some(cmd_list) = self.command_list.as_mut() {
            unsafe {
                cmd_list.borrow_mut().ClearRenderTargetView(
                    rtv_handle,
                    &*clear_color.as_ptr(),
                    None,
                );
            };
        } else {
            return Err(dx12error::Dx12Error::new(&format!(
                "Failed to clear render target view: {:?}",
                "command list is none"
            )));
        }
        Ok(())
    }

    //深度ステンシルバッファのクリア
    pub fn clear_depth_stencil_view(
        &mut self,
        dsv_handle: D3D12_CPU_DESCRIPTOR_HANDLE,
        clear_value: f32,
    ) -> std::result::Result<(), dx12error::Dx12Error> {
        if let Some(cmd_list) = self.command_list.as_mut() {
            unsafe {
                cmd_list.borrow_mut().ClearDepthStencilView(
                    dsv_handle,
                    D3D12_CLEAR_FLAG_DEPTH | D3D12_CLEAR_FLAG_STENCIL,
                    clear_value,
                    0,
                    &[], // Full view is cleared
                )
            }
        } else {
            return Err(dx12error::Dx12Error::new(&format!(
                "Failed to clear depth stencil view: {:?}",
                "command list is none"
            )));
        }
        Ok(())
    }

    pub fn close(&mut self) -> std::result::Result<(), dx12error::Dx12Error> {
        if let Some(cmd_list) = self.command_list.as_ref() {
            match unsafe { cmd_list.borrow_mut().Close() } {
                Ok(_) => (),
                Err(err) => {
                    return Err(dx12error::Dx12Error::new(&format!(
                        "Failed to close command list: {:?}",
                        err
                    )))
                }
            }
        } else {
            return Err(dx12error::Dx12Error::new(&format!(
                "Failed to close command list: {:?}",
                "command list is none"
            )));
        }

        Ok(())
    }
}

//private method
impl RenderContext {
    //シザリング矩形を設定
    fn set_scissor_rect(
        &mut self,
        scissor_rect: RECT,
    ) -> std::result::Result<(), dx12error::Dx12Error> {
        //シザリング矩形を設定
        if let Some(cmd_list) = self.command_list.as_ref() {
            unsafe { cmd_list.borrow_mut().RSSetScissorRects(&[scissor_rect]) };
        } else {
            return Err(dx12error::Dx12Error::new(&format!(
                "Failed to set scissor rect: {:?}",
                "command list is none"
            )));
        }

        Ok(())
    }

    //transient barrierを設定
    fn transient_barrier(
        resource: &ID3D12Resource,
        state_before: D3D12_RESOURCE_STATES,
        state_after: D3D12_RESOURCE_STATES,
    ) -> D3D12_RESOURCE_BARRIER {
        D3D12_RESOURCE_BARRIER {
            Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
            Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
            Anonymous: D3D12_RESOURCE_BARRIER_0 {
                Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                    pResource: unsafe { std::mem::transmute_copy(resource) },
                    StateBefore: state_before,
                    StateAfter: state_after,
                    Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                }),
            },
        }
    }

    pub fn reset_pso_none(
        &mut self,
        cmd_allocator: &ID3D12CommandAllocator,
    ) -> Result<(), dx12error::Dx12Error> {
        if let Some(cmd_list) = self.command_list.as_ref() {
            match unsafe { cmd_list.borrow_mut().Reset(cmd_allocator, None) } {
                Ok(_) => (),
                Err(err) => {
                    return Err(dx12error::Dx12Error::new(&format!(
                        "Failed to reset command list: {:?}",
                        err
                    )))
                }
            }
        }

        Ok(())
    }
}

//セット method
impl RenderContext {
    //レンダリングターゲットを設定する
    pub fn set_render_target(
        &mut self,
        rtv_handle: &D3D12_CPU_DESCRIPTOR_HANDLE,
        dsv_handle: &D3D12_CPU_DESCRIPTOR_HANDLE,
    ) -> std::result::Result<(), dx12error::Dx12Error> {
        if let Some(cmd_list) = self.command_list.as_mut() {
            unsafe {
                //cmd_list.OMSetRenderTargets(1, Some(&rtv_handle), FALSE, Some(&dsv_handle));
                cmd_list
                    .borrow_mut()
                    .OMSetRenderTargets(1, Some(rtv_handle), false, None);
            }
        } else {
            return Err(dx12error::Dx12Error::new(&format!(
                "Failed to set render target: {:?}",
                "command list is none"
            )));
        }
        Ok(())
    }
}

/// エラー取得用 module
#[path = "dx12error.rs"]
mod dx12error;
use dx12error::Dx12Error;

/// sub module graphics_settings
#[path = "graphics_settings.rs"]
mod graphics_settings;

/// sub module swapchain
#[path = "swapchain.rs"]
mod swapchain;

/// sub module depthstencil
#[path = "depthstencil.rs"]
mod depthstencil;
use depthstencil::DepthStencil;

///sub module fence
#[path = "fence.rs"]
mod fence;

#[path = "render_context.rs"]
mod render_context;

#[path = "render_target.rs"]
mod render_target;

use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D::*, Win32::Graphics::Direct3D12::*,
    Win32::Graphics::Dxgi::*, Win32::System::Threading::*,
};

use std::cell::RefCell;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::rc::Rc;

/// メインレンダリングリソース
///
/// # Fields
/// *  'device' - デバイス
/// *  'command_queue' - コマンドキュー
/// *  'command_allocator' - コマンドアロケータ
/// *  'command_list' - コマンドリスト
/// *  'fence' - フェンス
/// *  'swapchain' - スワップチェーン
/// *  'render_target' - レンダーターゲット
/// *  'depth_stencil' - 深度ステンシル
/// *  'viewport' - ビューポート
/// *  'render_context' - レンダーコンテキスト
/// *  'scissor_rect' - シザー矩形
/// *  'frame_index' - 現在のフレームインデックス
pub struct MainRenderingResources {
    device: Option<ID3D12Device>,
    command_queue: Option<ID3D12CommandQueue>,
    command_allocator: Option<ID3D12CommandAllocator>,
    command_list: Option<Rc<RefCell<ID3D12GraphicsCommandList4>>>,
    fence: fence::Fence,
    swapchain: swapchain::SwapChain,
    render_target: render_target::RenderTarget,
    depth_stencil: DepthStencil,
    viewport: D3D12_VIEWPORT,
    render_context: render_context::RenderContext,
    scissor_rect: RECT,
    frame_index: u32,
}

/// MainRenderingResourcesの初期化
impl Default for MainRenderingResources {
    fn default() -> Self {
        Self {
            device: None,
            command_queue: None,
            command_allocator: None,
            command_list: None,
            fence: fence::Fence::default(),
            swapchain: swapchain::SwapChain::default(),
            render_target: render_target::RenderTarget::default(),
            depth_stencil: DepthStencil::default(),
            //pipeline_state: None,
            viewport: D3D12_VIEWPORT {
                ..Default::default()
            },
            render_context: render_context::RenderContext::default(),
            scissor_rect: RECT {
                top: 0,
                bottom: 0,
                left: 0,
                right: 0,
            },
            frame_index: 0,
        }
    }
}

/// MainRenderingResourcesのcreate methods
impl MainRenderingResources {
    /// MainRenderingResourcesの生成
    ///
    /// # Arguments
    /// *  'hwnd' - window handle
    /// *  'width' - フレームバッファ幅
    /// *  'height' - フレームバッファ高さ
    ///
    #[allow(dead_code)]
    pub fn create(
        hwnd: HWND,
        width: u64,
        height: u32,
    ) -> std::result::Result<MainRenderingResources, Box<dyn std::error::Error>> {
        let mut dx12_resources: MainRenderingResources = Default::default();

        //デバッグ用 DXGIファクトリ生成
        let dxgi_factory: Option<IDXGIFactory4> = Some(dx12_resources.create_factory()?);

        //デバイスを生成
        dx12_resources.device = Some(dx12_resources.create_device(&dxgi_factory)?);

        //コマンドキュー生成
        dx12_resources.command_queue = Some(dx12_resources.create_commandqueue()?);

        //スワップチェイン作成
        dx12_resources.swapchain =
            dx12_resources.create_swapchain(&dxgi_factory, &hwnd, width as u32, height)?;

        //ウィンドウをフルスクリーンに関連付ける
        dx12_resources.associate_the_window_with_full_screen(&dxgi_factory, &hwnd)?;

        //rtv ディスクリプタヒープ生成 & サイズ取得
        dx12_resources.render_target = dx12_resources.create_render_target()?;

        //dsv ディスクリプタヒープ生成 & サイズ取得
        dx12_resources.depth_stencil = dx12_resources.create_depth_stencil(width, height)?;

        //コマンドアロケータの生成
        dx12_resources.command_allocator = Some(dx12_resources.create_command_allocator()?);

        //コマンドリストの生成
        dx12_resources.command_list =
            Some(Rc::new(RefCell::new(dx12_resources.create_command_list()?)));

        //GPUと同期オブジェクト生成
        dx12_resources.fence = dx12_resources.create_fence()?;

        //レンダーコンテキスト作成
        dx12_resources.render_context = dx12_resources.create_render_context()?;

        //ビューポート(表示領域を設定)を初期化
        dx12_resources.viewport =
            MainRenderingResources::create_viewport(width as f32, height as f32);

        //シザリング矩形を初期化
        dx12_resources.scissor_rect =
            MainRenderingResources::create_scissor_rect(width as i32, height as i32);

        return Ok(dx12_resources);
    }
}

/// private create methods
impl MainRenderingResources {
    /// ファクトリ生成
    ///
    /// # Returns
    /// *  'Ok(IDXGIFactory4)' - ファクトリ
    /// *  'Err(Dx12Error)' - エラーメッセージ
    fn create_factory(&self) -> std::result::Result<IDXGIFactory4, Dx12Error> {
        //デバッグ時のみ入る
        if cfg!(debug_assertions) {
            unsafe {
                let mut debug: Option<ID3D12Debug> = None;
                if let Some(debug) = D3D12GetDebugInterface(&mut debug).ok().and(debug) {
                    debug.EnableDebugLayer();
                }
            }
        }

        //不変にしたいので別のifにしている
        //デバッグの場合デバッグフラグを立てる
        let dxgi_factory_flags = if cfg!(debug_assertions) {
            DXGI_CREATE_FACTORY_DEBUG
        } else {
            0
        };

        //dxgi factory生成
        match unsafe { CreateDXGIFactory2(dxgi_factory_flags) } {
            Ok(dxgi_factory) => {
                println!("Factory creation succeeded");
                Ok(dxgi_factory)
            }
            Err(err) => Err(Dx12Error::new(&format!(
                "Failed to create factory: {:?}",
                err
            ))),
        }
    }

    /// デバイス生成
    ///
    /// # Arguments
    /// *  'factory' - ファクトリ
    ///
    /// # Returns
    /// *  'Ok(ID3D12Device)' - デバイス
    /// *  'Err(Dx12Error)' - エラーメッセージ
    fn create_device(
        &self,
        factory: &Option<IDXGIFactory4>,
    ) -> std::result::Result<ID3D12Device, Dx12Error> {
        //主要なGPUベンダー定義
        enum GpuVender {
            GpuVenderNvidia, //NVIDIA
            GpuVenderAmd,    //AMD
            GpuVenderIntel,  //Intel

            NumGpuVender, //Vender数
        }

        //大手venderのGPUを持つアダプタ
        let mut adapter_vender: [Option<IDXGIAdapter1>; GpuVender::NumGpuVender as usize] =
            unsafe { std::mem::zeroed() };
        //最大のビデオサイズを持つアダプタ 主要なGPUがない場合に使用される
        let mut adapter_maximum_video_memory: Option<IDXGIAdapter1> = None;
        //ビデオメモリー比較用
        let mut video_memory_size = 0;

        //ここはグラフィックスカードが複数枚刺さっている場合にどれが一番メモリ容量が多いかを調べ一番多いものを使用する為のloop
        let mut i: u32 = 0;
        loop {
            //factoryを安全に取得
            let factory = match factory.as_ref() {
                Some(factory) => factory,
                None => return Err(Dx12Error::new("The value of factory is None")),
            };

            //アダプター取得
            let adapter = match unsafe { factory.EnumAdapters1(i) } {
                Ok(ap) => ap,
                Err(_) => {
                    break;
                }
            };

            //グラフィックス能力のあるdescを取得
            let mut desc = DXGI_ADAPTER_DESC::default();
            //desc取得チェック
            if let Err(err) = unsafe { adapter.GetDesc(&mut desc) } {
                return Err(Dx12Error::new(&format!("Failed get device {:?}", err)));
                //break;
            }

            //ビデオメモリの比較を行う
            if desc.DedicatedVideoMemory > video_memory_size {
                //ここで取得したアダプタはAMDやINTEL等のGPUがない場合に使用するアダプタ
                //現在取得しているdescのビデオメモリの方が多いので更新する
                adapter_maximum_video_memory = Some(adapter.clone());
                video_memory_size = desc.DedicatedVideoMemory;
            }

            //文字列変換処理
            //文字列内で最初のnull文字(0)を見つけるか見つからなければ配列の長さを返す
            let end_position = desc
                .Description
                .iter()
                .position(|&x| x == 0)
                .unwrap_or_else(|| desc.Description.len());
            //先ほど見つけた終端位置までの部分を取り出しOsStringに変換する
            let os_string: OsString = OsStringExt::from_wide(&desc.Description[0..end_position]);
            //OsStringをUTF-8文字列に変換する
            let description = os_string.to_string_lossy();

            //各GPUベンダーの処理
            if description.contains("NVIDIA") {
                // NVIDIAの処理
                adapter_vender[GpuVender::GpuVenderNvidia as usize] = Some(adapter.clone());
            } else if description.contains("AMD") {
                // AMDの処理
                adapter_vender[GpuVender::GpuVenderAmd as usize] = Some(adapter.clone());
            } else if description.contains("Intel") {
                // Intelの処理
                adapter_vender[GpuVender::GpuVenderIntel as usize] = Some(adapter.clone());
            }

            //インクリ
            i = i + 1;
        }

        //使用するアダプタを決める(現在はintelが最優先)
        // NVIDIA >> AMD >> intel >> other
        let use_adapter: Option<IDXGIAdapter1> = if let Some(adaptor) =
            adapter_vender[GpuVender::GpuVenderNvidia as usize].clone()
        {
            //NVIDIA
            Some(adaptor)
        } else if let Some(adaptor) = adapter_vender[GpuVender::GpuVenderAmd as usize].clone() {
            //AMD
            Some(adaptor)
        } else if let Some(adaptor) = adapter_vender[GpuVender::GpuVenderIntel as usize].clone() {
            //INTEL
            Some(adaptor)
        } else {
            //主要ベンダ以外
            adapter_maximum_video_memory
        };

        //pcによってレベルが異なるため 使用している可能性のあるFEATURE_LEVELを列挙
        const FEATURE_LEVELS: [D3D_FEATURE_LEVEL; 4] = [
            D3D_FEATURE_LEVEL_12_1, //Direct3D 12.1の機能
            D3D_FEATURE_LEVEL_12_0, //Direct3D 12.0の機能
            D3D_FEATURE_LEVEL_11_1, //Direct3D 11.1の機能
            D3D_FEATURE_LEVEL_11_0, //Direct3D 11.0の機能
        ];

        //device生成処理loop
        //TODO:ネストが深いので改善する
        for level in FEATURE_LEVELS {
            let mut device: Option<ID3D12Device> = None;

            if let Some(ref adapter) = use_adapter.as_ref() {
                match unsafe { D3D12CreateDevice(*adapter, level, &mut device) } {
                    Ok(_) => {
                        //生成に成功したのでdeviceを返す
                        if let Some(d) = device {
                            println!("Device creation succeeded");
                            return Ok(d);
                        }
                    }
                    Err(_) => {
                        //エラーの場合、次のfeature_levelで試みる
                        continue;
                    }
                }
            }
        }

        //デバイスの生成に失敗
        return Err(Dx12Error::new("Failed to generate device"));
    }

    /// コマンドキューの生成
    ///
    /// # Returns
    /// *  'Ok(ID3D12CommandQueue)' - コマンドキュー
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    fn create_commandqueue(&self) -> std::result::Result<ID3D12CommandQueue, Dx12Error> {
        // コマンドキューの設定
        let command_queue_desc = D3D12_COMMAND_QUEUE_DESC {
            Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
            Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
            ..Default::default()
        };

        //コマンドキューの生成
        if let Some(ref device) = self.device {
            match unsafe { device.CreateCommandQueue(&command_queue_desc) } {
                Ok(cmd_queue) => {
                    //成功した場合commandqueueを返す
                    println!("CommandQueue creation succeeded");
                    Ok(cmd_queue)
                }
                Err(err) => Err(Dx12Error::new(&format!(
                    "Failed to create command queue: {:?}",
                    err
                ))),
            }
        } else {
            Err(Dx12Error::new(&format!(
                "Device not owned, failed to create command queue",
            )))
        }
    }

    /// スワップチェーンの生成
    ///
    /// # Arguments
    /// *  'factory' - ファクトリ
    /// *  'hwnd' - window handle
    /// *  'width' - フレームバッファ幅
    /// *  'height' - フレームバッファ高さ
    ///
    /// # Returns
    /// *  'Ok(SwapChain)' - スワップチェーン
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    fn create_swapchain(
        &self,
        factory: &Option<IDXGIFactory4>,
        hwnd: &HWND,
        width: u32,
        height: u32,
    ) -> std::result::Result<swapchain::SwapChain, Dx12Error> {
        if let (Some(f), Some(cmd_queue)) = (factory, self.command_queue.as_ref()) {
            match swapchain::SwapChain::create(f, &hwnd, width, height, cmd_queue) {
                Ok(sp) => Ok(sp),
                Err(err) => Err(Dx12Error::new(&format!(
                    "Failed to create rtv descriptor heap: {:?}",
                    err
                ))),
            }
        } else {
            Err(Dx12Error::new("Failed to create rtv descriptor heap"))
        }
    }

    /// ウィンドウをフルスクリーンに関連付ける
    /// # Arguments
    ///
    /// *  'factory' - ファクトリ
    /// *  'hwnd' - window handle
    ///
    /// # Returns
    /// *  'Ok(())' - 成功
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    fn associate_the_window_with_full_screen(
        &self,
        factory: &Option<IDXGIFactory4>,
        hwnd: &HWND,
    ) -> std::result::Result<(), Dx12Error> {
        //TODO:フルスクリーンに対応させる
        //TODO:imguiでウィンドウ <-> フルスクリーンを行き来できるようにする

        //ウィンドウの設定をする
        if let Some(ref factory) = factory {
            match unsafe { factory.MakeWindowAssociation(*hwnd, DXGI_MWA_NO_ALT_ENTER) } {
                Ok(_) => {
                    println!("bind window succeeded");
                    Ok(())
                }
                Err(err) => Err(Dx12Error::new(&format!("Failed to bind window: {:?}", err))),
            }
        } else {
            Err(Dx12Error::new(&format!("Failed to create swap chain:")))
        }
    }

    /// rtv ディスクリプタヒープ生成
    ///
    /// # Arguments
    /// *  'device' - デバイス
    ///
    /// # Returns
    /// *  'Ok(ID3D12DescriptorHeap)' - ディスクリプタヒープ
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    fn create_render_target(&self) -> std::result::Result<render_target::RenderTarget, Dx12Error> {
        let swapchain = match self.swapchain.get_dxgi_swapchain() {
            Ok(sc) => sc,
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to create render target: {:?}",
                    err
                )))
            }
        };

        if let Some(device) = self.device.as_ref() {
            match render_target::RenderTarget::create(device, swapchain) {
                Ok(rt) => Ok(rt),
                Err(err) => Err(Dx12Error::new(&format!(
                    "Failed to create render target: {:?}",
                    err
                ))),
            }
        } else {
            Err(Dx12Error::new("Failed to create render target"))
        }
    }

    /// dsv ディスクリプタヒープ生成
    ///
    /// # Arguments
    /// *  'device' - デバイス
    /// *  'width' - フレームバッファ幅
    /// *  'height' - フレームバッファ高さ
    ///
    /// # Returns
    /// *  'Ok(DepthStencil)' - 深度ステンシル
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    fn create_depth_stencil(
        &self,
        width: u64,
        height: u32,
    ) -> std::result::Result<DepthStencil, Dx12Error> {
        if let Some(device) = self.device.as_ref() {
            match DepthStencil::create(device, width, height) {
                Ok(ds) => Ok(ds),
                Err(err) => Err(Dx12Error::new(&format!(
                    "Failed to create depth stencil: {:?}",
                    err
                ))),
            }
        } else {
            Err(Dx12Error::new("Failed to create depth stencil"))
        }
    }

    /// コマンドアロケータの生成
    ///
    /// # Returns
    /// *  'Ok(ID3D12CommandAllocator)' - コマンドアロケータ
    /// *  'Err(Dx12Error)' - エラーメッセージ
    fn create_command_allocator(&self) -> std::result::Result<ID3D12CommandAllocator, Dx12Error> {
        //コマンドアロケータの生成
        if let Some(device) = self.device.as_ref() {
            match unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT) } {
                Ok(cmda) => Ok(cmda),
                Err(err) => Err(Dx12Error::new(&format!(
                    "Failed to create command allocator: {:?}",
                    err
                ))),
            }
        } else {
            Err(Dx12Error::new(&format!(
                "Failed to create command allocator"
            )))
        }
    }

    /// コマンドリストの生成
    ///
    /// # Returns
    /// *  'Ok(ID3D12GraphicsCommandList4)' - コマンドリスト
    /// *  'Err(Dx12Error)' - エラーメッセージ
    fn create_command_list(&self) -> std::result::Result<ID3D12GraphicsCommandList4, Dx12Error> {
        //コマンドリスト生成
        let mut command_list: Option<ID3D12GraphicsCommandList4>;
        if let (Some(ref device), Some(ref cmda)) =
            (self.device.clone(), self.command_allocator.as_ref())
        {
            match unsafe {
                device.clone().CreateCommandList(
                    0,
                    D3D12_COMMAND_LIST_TYPE_DIRECT,
                    cmda.clone(),
                    None,
                )
            } {
                Ok(cmdl) => command_list = Some(cmdl),
                Err(err) => {
                    return Err(Dx12Error::new(&format!(
                        "Failed to create command list: {:?}",
                        err
                    )))
                }
            };
        } else {
            return Err(Dx12Error::new(&format!("Failed to create command list")));
        }

        //コマンドリストは開かれている状態で生成されるので，一度閉じる
        if let Some(cl) = command_list.as_mut() {
            match unsafe { cl.Close() } {
                Ok(()) => (),
                Err(err) => {
                    return Err(Dx12Error::new(&format!(
                        "Failed to close command list: {:?}",
                        err
                    )))
                }
            }
        }

        //生成したものをreturnする
        if let Some(cl) = command_list.clone() {
            println!("commandList creation succeeded");
            return Ok(cl);
        } else {
            return Err(Dx12Error::new("Command list was not properly initialized"));
        }
    }

    /// GPUと同期オブジェクトfenceの生成
    ///
    /// # Returns
    /// *  'Ok(fence::Fence)' - fence
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    fn create_fence(&self) -> std::result::Result<fence::Fence, Dx12Error> {
        //fence生成
        if let Some(device) = self.device.as_ref() {
            match fence::Fence::create(device) {
                Ok(fence) => {
                    println!("success to create fence");
                    Ok(fence)
                }
                Err(err) => Err(Dx12Error::new(&format!(
                    "Failed to create fence: {:?}",
                    err
                ))),
            }
        } else {
            Err(Dx12Error::new("Failed to create fence"))
        }
    }

    /// レンダーコンテキストの生成
    ///
    /// # Returns
    /// *  'Ok(render_context::RenderContext)' - レンダーコンテキスト
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    fn create_render_context(
        &self,
    ) -> std::result::Result<render_context::RenderContext, Dx12Error> {
        //作成
        if let Some(cmd_list) = self.command_list.as_ref() {
            match render_context::RenderContext::create(cmd_list) {
                Ok(rc) => {
                    println!("success to create render context");
                    Ok(rc)
                }
                Err(err) => Err(Dx12Error::new(&format!(
                    "Failed to create render context: {:?}",
                    err
                ))),
            }
        } else {
            Err(Dx12Error::new("Failed to create render context"))
        }
    }

    /// ビューポート(表示領域を設定)作成
    ///
    /// # Arguments
    /// *  'width' - フレームバッファ幅
    /// *  'height' - フレームバッファ高さ
    ///
    /// # Returns
    /// *  'D3D12_VIEWPORT' - ビューポート
    ///
    fn create_viewport(width: f32, height: f32) -> D3D12_VIEWPORT {
        println!("success to create viewport");
        D3D12_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: width,
            Height: height,
            MinDepth: D3D12_MIN_DEPTH,
            MaxDepth: D3D12_MAX_DEPTH,
        }
    }

    /// シザリング矩形作成
    ///
    /// # Arguments
    /// *  'width' - フレームバッファ幅
    /// *  'height' - フレームバッファ高さ
    ///
    /// # Returns
    /// *  'RECT' - シザリング矩形
    ///
    fn create_scissor_rect(width: i32, height: i32) -> RECT {
        println!("success to create scissor rect");
        RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        }
    }
}

//レンダリング 開始/終了 処理
#[allow(dead_code)]
impl MainRenderingResources {
    /// レンダリング開始処理
    ///
    /// # Arguments
    /// *  'clear_color' - クリアカラー(rgba)
    ///
    /// # Returns
    /// *  'Ok(())' - 成功
    /// *  'Err(Dx12Error)' - エラーメッセージ
    pub fn begin_render(&mut self, clear_color: [f32; 4]) -> std::result::Result<(), Dx12Error> {
        //バックバッファのインデックスを取得
        self.frame_index = match self.swapchain.get_dxgi_swapchain() {
            Ok(sc) => unsafe { sc.GetCurrentBackBufferIndex() },
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed get current back buffer index {:?}",
                    err
                )))
            }
        };

        //TODO:カメラを作成する

        //コマンドアロケータをリセット
        if let Some(cmd_allocator) = self.command_allocator.as_ref() {
            match unsafe { cmd_allocator.Reset() } {
                Ok(()) => (),
                Err(err) => {
                    return Err(Dx12Error::new(&format!(
                        "Failed to reset command allocator {:?}",
                        err
                    )))
                }
            }
        } else {
            return Err(Dx12Error::new("Failed to reset command allocator"));
        }

        //レンダリングコンテキストもリセット
        if let Some(cmd_allocator) = self.command_allocator.as_ref() {
            match self.render_context.reset_pso_none(cmd_allocator) {
                Ok(_) => (),
                Err(err) => {
                    return Err(Dx12Error::new(&format!(
                        "Failed to reset render context {:?}",
                        err
                    )))
                }
            }
        } else {
            return Err(Dx12Error::new("Failed to reset render context"));
        }

        //ビューポートを設定
        match self.render_context.set_viewport_and_scissor(self.viewport) {
            Ok(()) => (),
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed set_viewport_and_scissor {:?}",
                    err
                )))
            }
        }

        //バックバッファがレンダリングターゲットとして設定可能になるまで待機
        match self
            .render_context
            .wait_until_to_possible_set_render_target(
                self.render_target
                    .get_render_target(self.frame_index as usize),
            ) {
            Ok(()) => (),
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to wait_until_to_possible_set_render_target {:?}",
                    err
                )))
            }
        }

        //現在のレンダリングターゲットビューのフレームバッファ設定
        let current_frame_buffer_rtv_handle: D3D12_CPU_DESCRIPTOR_HANDLE = match self
            .render_target
            .get_current_frame_buffer(self.frame_index as usize)
        {
            Ok(rtvh) => rtvh,
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed get current frame buffer rtv handle {:?}",
                    err
                )))
            }
        };

        //深度ステンシルバッファのディスクリプタヒープの開始アドレスを取得
        let current_frame_buffer_dsv_handle = match self.depth_stencil.get_heap_start() {
            Ok(dh) => dh,
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed get current frame buffer dsv handle {:?}",
                    err
                )))
            }
        };

        //レンダリングターゲット設定

        match self.render_context.set_render_target(
            &current_frame_buffer_rtv_handle,
            &current_frame_buffer_dsv_handle,
        ) {
            Ok(()) => (),
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to set_render_target {:?}",
                    err
                )))
            }
        }

        //レンダリングターゲットのクリア
        match self
            .render_context
            .clear_render_target_view(current_frame_buffer_rtv_handle, clear_color)
        {
            Ok(()) => (),
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to clear render target view {:?}",
                    err
                )))
            }
        }

        /*
        //深度ステンシルバッファのクリア
        match self
        .render_context
        .clear_depth_stencil_view(current_frame_buffer_dsv_handle, 1.0_f32)
        {
            Ok(()) => (),
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to clear depth stencil view {:?}",
                    err
                )))
            }
        }
        */

        return Ok(());
    }

    /// レンダリング終了処理
    ///
    /// # Returns
    /// *  'Ok(())' - 成功
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    pub fn end_render(&mut self) -> std::result::Result<(), Dx12Error> {
        //レンダリングターゲットへの描き込み完了待ち
        match self
            .render_context
            .wait_until_finish_drawing_to_render_target(
                self.render_target
                    .get_render_target(self.frame_index as usize),
            ) {
            Ok(_) => (),
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to wait until finish drawing to render target {:?}",
                    err
                )))
            }
        }

        //レンダリングコンテキストを閉じる
        match self.render_context.close_command_list() {
            Ok(_) => (),
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to close render context {:?}",
                    err
                )))
            }
        }

        //コマンドを実行するためにコマンドリストをID3D12CommandListに変換
        let mut command_list: Option<ID3D12CommandList> = None;
        if let Some(cmd_list) = self.command_list.as_ref() {
            command_list = Some(cmd_list.borrow().can_clone_into());
        } else {
            return Err(Dx12Error::new("Failed to execute command list"));
        }

        //コマンドを実行
        if let Some(cmd_queue) = self.command_queue.as_ref() {
            unsafe { cmd_queue.ExecuteCommandLists(&[command_list]) }
        } else {
            return Err(Dx12Error::new("Failed to execute command list"));
        }

        //バックバッファとフロントバッファを入れ替える
        match self.swapchain.present() {
            Ok(_) => (),
            Err(err) => return Err(Dx12Error::new(&format!("Failed to present {:?}", err))),
        }

        //描画終了待ち
        match self.wait_draw() {
            Ok(_) => (),
            Err(err) => return Err(Dx12Error::new(&format!("Failed to wait draw {:?}", err))),
        }

        Ok(())
    }

    /// 描画終了待ち
    ///
    /// # Returns
    /// *  'Ok(())' - 成功
    /// *  'Err(Dx12Error)' - エラーメッセージ
    ///
    fn wait_draw(&mut self) -> std::result::Result<(), Dx12Error> {
        //fenceの値を取得
        let fence_value = match self.fence.get_value() {
            Ok(value) => *value,
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to get fence value {:?}",
                    err
                )))
            }
        };

        //fenceのイベントを取得
        let fence_event = match self.fence.get_event() {
            Ok(event) => *event,
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to get fence event {:?}",
                    err
                )))
            }
        };

        //描画終了待ち
        if let Some(cmd_queue) = self.command_queue.as_mut() {
            match self.fence.get_fence() {
                Ok(fence) => unsafe {
                    match cmd_queue.Signal(fence, fence_value) {
                        Ok(_) => (),
                        Err(err) => {
                            return Err(Dx12Error::new(&format!("Failed to signal {:?}", err)))
                        }
                    }
                },
                Err(err) => return Err(Dx12Error::new(&format!("Failed to get fence {:?}", err))),
            }
        } else {
            return Err(Dx12Error::new("Failed to wait draw"));
        }

        //fenceの値をインクリメント
        match self.fence.add_value() {
            Ok(()) => (),
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to add fence value {:?}",
                    err
                )))
            }
        }

        //前のフレームが終了するまでまつ
        match self.fence.get_fence() {
            Ok(fence) => {
                if unsafe { fence.GetCompletedValue() } < fence_value {
                    match unsafe { fence.SetEventOnCompletion(fence_value, fence_event) } {
                        Ok(_) => (),
                        Err(err) => {
                            return Err(Dx12Error::new(&format!(
                                "Failed to set event on completion {:?}",
                                err
                            )))
                        }
                    }

                    let result = unsafe { WaitForSingleObject(fence_event, INFINITE) };

                    match result {
                        WAIT_OBJECT_0 => (),
                        WAIT_FAILED => {
                            let error_code = unsafe { GetLastError() };
                            return Err(Dx12Error::new(&format!(
                                "Failed to wait for single object, error code: {:?}",
                                error_code
                            )));
                        }
                        _ => {
                            return Err(Dx12Error::new(&format!(
                            "Unexpected result while waiting for single object, result code: {:?}",
                                result
                            )))
                        }
                    }
                }
            }
            Err(err) => return Err(Dx12Error::new(&format!("Failed to get fence {:?}", err))),
        }

        Ok(())
    }
}

/// get
impl MainRenderingResources {
    pub fn get_device(&self) -> std::result::Result<&ID3D12Device, Dx12Error> {
        self.device
            .as_ref()
            .ok_or_else(|| Dx12Error::new("Failed to get device"))
    }
}

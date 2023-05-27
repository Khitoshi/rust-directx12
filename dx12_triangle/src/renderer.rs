use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D::Fxc::*, Win32::Graphics::Direct3D::*,
    Win32::Graphics::Direct3D12::*, Win32::Graphics::Dxgi::Common::*,
    Win32::Graphics::Dxgi::IDXGIFactory6, Win32::Graphics::Dxgi::*,
    Win32::System::LibraryLoader::*, Win32::System::Threading::*,
    Win32::UI::WindowsAndMessaging::*,
};

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
const FRAME_BUFFER_COUNT: u32 = 2;
//エラー取得用
pub struct Dx12Error {
    message: String,
}
impl Dx12Error {
    pub fn new(message: &str) -> Dx12Error {
        Dx12Error {
            message: message.to_string(),
        }
    }
    //エラー出力
    pub fn print_error(&self) {
        eprintln!("{}", self.message);
    }
}

//TODO:Dx12用のrsファイルを作成する

pub struct Dx12Resources {
    //ファクトリー デバッグ用
    dxgi_factory: IDXGIFactory4,
    //デバイス
    device: ID3D12Device,
    //コマンドキュー
    command_queue: ID3D12CommandQueue,
    //スワップチェイン
    swap_chain: IDXGISwapChain4,
    //レンダリングターゲットビューのディスクリプタヒープ
    rtv_heap: Option<ID3D12DescriptorHeap>,
    //レンダーターゲットビューのサイズ
    rtv_descriptor_size: u32,
    //深度ステンシルビューのディスクリプタヒープ
    dsv_heap: Option<ID3D12DescriptorHeap>,
    //深度ステンシルビューのサイズ
    dsv_descriptor_size: u32,
    //フレームバッファ用のレンダリングターゲット
    render_targets: [ID3D12Resource; FRAME_BUFFER_COUNT as usize],
    //深度ステンシルバッファ
    depth_stencil_buffer: Option<ID3D12Resource>,
    //コマンドアロケータ
    command_allocator: ID3D12CommandAllocator,
    //コマンドリスト
    command_list: ID3D12GraphicsCommandList,

    //GPUと同期するオブジェクト
    fence: ID3D12Fence,
    //フェンスの値
    fence_value: u32,
    //
    fence_event: HANDLE,
    //現在のバッグバッファインデックス
    current_back_buffer_index: u32,
}

impl Dx12Resources {
    //other method

    fn new() {}
}

//
impl Dx12Resources {
    fn create_factory() -> std::result::Result<IDXGIFactory4, Dx12Error> {
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
        let dxgi_factory_result = unsafe { CreateDXGIFactory2(dxgi_factory_flags) };

        //factory 生成チェック
        match dxgi_factory_result {
            Ok(dxgi_factory) => {
                // 成功した場合の処理
                println!("Factory creation succeeded");
                Ok(dxgi_factory)
            }
            Err(err) => {
                // 失敗した場合の処理
                //println!("Factory creation failed with error: {}", err);
                let errstr: String = format!("Factory creation failed with error:{}", err);
                Err(Dx12Error::new(&errstr))
            }
        }
    }

    //デバイスを生成
    fn create_device(factory: IDXGIFactory4) -> std::result::Result<ID3D12Device, Dx12Error> {
        //主要なGPUベンダー定義
        enum GpuVender {
            GpuVenderNvidia, //NVIDIA
            GpuVenderAmd,    //AMD
            GpuVenderIntel,  //Intel

            NumGpuVender, //Vender数
        }

        //大手venderのGPUを持つアダプタ
        let mut adapter_vender: [Option<IDXGIAdapter>; GpuVender::NumGpuVender as usize];
        //最大のビデオサイズを持つアダプタ 主要なGPUがない場合に使用される
        let mut adapter_maximum_video_memory: Option<IDXGIAdapter>;
        //ビデオメモリー比較用
        let mut video_memory_size = 0;
        //ここはグラフィックスカードが複数枚刺さっている場合にどれが一番メモリ容量が多いかを調べ一番多いものを使用する為のloop
        let mut i: u32 = 0;
        loop {
            let adapter_result = unsafe { factory.EnumAdapters(i) };

            //アダプター取得
            let adapter = match adapter_result {
                Ok(adapter) => adapter,
                Err(err) => {
                    println!("EnumAdapters adapter error:{}", err);
                    break;
                }
            };

            //グラフィックス能力のあるdescを取得
            let mut desc: DXGI_ADAPTER_DESC = DXGI_ADAPTER_DESC::default();
            let desc_result = unsafe { adapter.GetDesc(&mut desc) };
            //desc取得チェック
            if let Err(err) = desc_result {
                println!("GetDesc error: {}", err);
                break;
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
        let mut use_adapter: Option<IDXGIAdapter> = None;
        if adapter_vender[GpuVender::GpuVenderNvidia as usize].is_some() {
            //NVIDIA
            use_adapter = Some(
                adapter_vender[GpuVender::GpuVenderNvidia as usize]
                    .clone()
                    .unwrap(),
            );
        } else if adapter_vender[GpuVender::GpuVenderAmd as usize].is_some() {
            //AMD
            use_adapter = Some(
                adapter_vender[GpuVender::GpuVenderAmd as usize]
                    .clone()
                    .unwrap(),
            );
        } else if adapter_vender[GpuVender::GpuVenderIntel as usize].is_some() {
            //INTEL
            use_adapter = Some(
                adapter_vender[GpuVender::GpuVenderIntel as usize]
                    .clone()
                    .unwrap(),
            );
        } else {
            //主要ベンダ以外
            use_adapter = Some(adapter_maximum_video_memory.clone().unwrap());
        }

        //pcによってレベルが異なるため 使用している可能性のあるFEATURE_LEVELを列挙
        const feature_levels: [D3D_FEATURE_LEVEL; 4] = [
            D3D_FEATURE_LEVEL_12_1, //Direct3D 12.1の機能
            D3D_FEATURE_LEVEL_12_0, //Direct3D 12.0の機能
            D3D_FEATURE_LEVEL_11_1, //Direct3D 11.1の機能
            D3D_FEATURE_LEVEL_11_0, //Direct3D 11.0の機能
        ];

        //device生成処理loop
        //TODO:ネストが深いので改善する
        for level in feature_levels {
            let mut device: Option<ID3D12Device> = None;
            if let Some(ref adapter) = use_adapter {
                match unsafe { D3D12CreateDevice(adapter, level, &mut device) } {
                    Ok(_) => {
                        //生成に成功したのでdeviceを返す
                        return Ok(device.unwrap());
                    }
                    Err(err) => {
                        //エラーの場合、次のfeature_levelで試みる
                        continue;
                    }
                }
            }
        }

        Err(Dx12Error::new("デバイスの生成に失敗"))
    }

    //create_commandqueue 生成
    fn create_commandqueue(&self) -> std::result::Result<ID3D12CommandQueue, Dx12Error> {
        // コマンドキューの設定
        const command_queue_desc: D3D12_COMMAND_QUEUE_DESC = D3D12_COMMAND_QUEUE_DESC {
            Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
            Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
            ..Default::default()
        };

        //生成&生成チェック
        match unsafe {
            self.device
                .CreateCommandQueue::<ID3D12CommandQueue>(&command_queue_desc)
        } {
            Ok(cmd_queue) => Ok(cmd_queue),
            Err(err) => Err(Dx12Error::new(&format!(
                "Failed to create command queue: {:?}",
                err
            ))),
        }
    }

    //スワップチェイン作成
    fn create_swapchain(
        &self,
        hwnd: &HWND,
        frame_buffer_width: u32,
        frame_buffer_hegith: u32,
    ) -> std::result::Result<IDXGISwapChain4, Dx12Error> {
        //スワップチェインの設定
        let swap_chain_desc: DXGI_SWAP_CHAIN_DESC1 = DXGI_SWAP_CHAIN_DESC1 {
            BufferCount: FRAME_BUFFER_COUNT,
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
        //TODO:swapchain1を定義せずに直接swapchain4をcastする
        let mut swap_chain1: Option<IDXGISwapChain1> = None;
        match unsafe {
            self.dxgi_factory.CreateSwapChainForHwnd(
                &self.command_queue,
                *hwnd,
                &swap_chain_desc,
                None,
                None,
            )
        } {
            Ok(sc) => {
                swap_chain1 = Some(sc);
            }
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to create swap chain: {:?}",
                    err
                )));
            }
        }

        //swapchain1 を swapchain4に変換する
        let swap_chain4: Option<IDXGISwapChain4> =
            match swap_chain1.unwrap().cast::<IDXGISwapChain4>() {
                Ok(sc) => Some(sc),
                Err(err) => {
                    return Err(Dx12Error::new(&format!(
                        "Failed to create swap chian:{:?}",
                        err
                    )))
                }
            };

        //バッグバッファ取得
        self.current_back_buffer_index =
            unsafe { swap_chain4.unwrap().GetCurrentBackBufferIndex() };

        Ok(swap_chain4.unwrap())
    }

    //ウィンドウをフルスクリーンに関連付ける
    fn associate_the_window_with_full_screen(
        &self,
        hwnd: &HWND,
    ) -> std::result::Result<(), Dx12Error> {
        //TODO:フルスクリーンに対応させる
        //TODO:imguiでウィンドウ <-> フルスクリーンを行き来できるようにする

        //ウィンドウの設定をする
        match unsafe {
            self.dxgi_factory
                .MakeWindowAssociation(*hwnd, DXGI_MWA_NO_ALT_ENTER)
        } {
            Ok(_) => Ok(()),
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to create swap chain: {:?}",
                    err
                )))
            }
        }
    }

    //rtv ディスクリプタヒープ生成
    fn create_rtv_descriptor_heap_for_frame_buffer(&self) -> std::result::Result<(), Dx12Error> {
        //レンダリングターゲットビューのディスクリプタヒープ用のディスクリプタヒープデスクを作成
        let desc: D3D12_DESCRIPTOR_HEAP_DESC = D3D12_DESCRIPTOR_HEAP_DESC {
            NumDescriptors: FRAME_BUFFER_COUNT,
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            ..Default::default()
        };

        match unsafe { self.device.CreateDescriptorHeap(&desc) } {
            Ok(rtv) => {
                self.rtv_heap = Some(rtv);
            }
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to create rtv descriptor heap: {:?}",
                    err
                )))
            }
        }

        //ディスクリプタのサイズを取得
        //TODO:単一責任理論に反している気がするので要検討
        self.rtv_descriptor_size = unsafe {
            self.device
                .GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV)
        };

        Ok(())
    }

    //dsv ディスクリプタヒープ生成
    fn create_dsv_descriptor_heap_for_frame_buffer(&self) -> std::result::Result<(), Dx12Error> {
        //深度ステンシルビューのディスクリプタヒープ用のディスクリプタヒープデスクを作成
        let desc: D3D12_DESCRIPTOR_HEAP_DESC = D3D12_DESCRIPTOR_HEAP_DESC {
            NumDescriptors: 1,
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            ..Default::default()
        };

        //深度ステンシルビューのディスクリプタヒープ作成
        match unsafe { self.device.CreateDescriptorHeap(&desc) } {
            Ok(dsv) => {
                self.dsv_heap = Some(dsv);
            }
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to create dsv descriptor heap: {:?}",
                    err
                )))
            }
        }

        //ディスクリプタのサイズを取得。
        //TODO:単一責任理論に反している可能性があるので要検討
        self.dsv_descriptor_size = unsafe {
            self.device
                .GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_DSV)
        };

        Ok(())
    }

    //フレームバッファ用のレンダーターゲットバッファの生成
    fn create_rtv_for_fame_buffer(&self) -> std::result::Result<(), Dx12Error> {
        //ヒープの先頭を表すCPUディスクリプタハンドルを取得
        let mut rtv_handle: D3D12_CPU_DESCRIPTOR_HANDLE =
            unsafe { self.rtv_heap.unwrap().GetCPUDescriptorHandleForHeapStart() };

        //フロントバッファをバックバッファ用のRTVを作成
        for i in 0..FRAME_BUFFER_COUNT as u32 {
            //フレームバッファ用レンダーターゲット取得
            let render_target: ID3D12Resource = match unsafe { self.swap_chain.GetBuffer(i) } {
                Ok(rt) => rt,
                Err(err) => {
                    return Err(Dx12Error::new(&format!(
                        "Failed to create rtv descriptor heap: {:?}",
                        err
                    )))
                }
            };

            //レンダーターゲットビューの生成
            unsafe {
                self.device
                    .CreateRenderTargetView(&render_target, None, rtv_handle)
            };

            //生成したレンダーターゲットを登録
            self.render_targets[i as usize] = render_target;

            //ポインタを渡したのでずらす
            rtv_handle.ptr += self.rtv_descriptor_size as usize;
        }

        Ok(())
    }

    //フレームバッファ用の深度ステンシルバッファの生成
    fn create_dsv_for_fame_buffer(
        &self,
        frame_buffer_width: u64,
        frame_buffer_height: u32,
    ) -> std::result::Result<(), Dx12Error> {
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
        match unsafe {
            self.device.CreateCommittedResource(
                &heap_prop,
                D3D12_HEAP_FLAG_NONE,
                &desc,
                D3D12_RESOURCE_STATE_DEPTH_WRITE,
                Some(&dsv_clear_value),
                &mut self.depth_stencil_buffer,
            )
        } {
            Ok(_) => (),
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to create depth stencil buffer: {:?}",
                    err
                )))
            }
        }

        Ok(())
    }

    //コマンドアロケータの生成
    fn create_command_allocator(&self) -> std::result::Result<(), Dx12Error> {
        //コマンドアロケータの生成
        match unsafe {
            self.device
                .CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)
        } {
            Ok(cmda) => self.command_allocator = cmda,
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to create command allocator: {:?}",
                    err
                )))
            }
        }

        Ok(())
    }

    //コマンドリストの生成
    fn create_command_list(&self) -> std::result::Result<(), Dx12Error> {
        //コマンドリスト生成
        match unsafe {
            self.device.CreateCommandList(
                0,
                D3D12_COMMAND_LIST_TYPE_DIRECT,
                &self.command_allocator,
                None,
            )
        } {
            Ok(cmdl) => self.command_list = cmdl,
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to create command list: {:?}",
                    err
                )))
            }
        }

        //コマンドリストは開かれている状態で生成されるので，一度閉じる
        match unsafe { self.command_list.Close() } {
            Ok(()) => (),
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to close command list: {:?}",
                    err
                )))
            }
        }

        Ok(())
    }

    //GPUと同期オブジェクト生成
    fn create_synchronization_with_gpu_object(&self) -> std::result::Result<(), Dx12Error> {
        //GPUと同期オブジェクト(fence)生成
        match unsafe { self.device.CreateFence(0, D3D12_FENCE_FLAG_NONE) } {
            Ok(fence) => self.fence = fence,
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to create fence: {:?}",
                    err
                )))
            }
        }

        //フェンスの値 設定
        self.fence_value = 1;

        //フェンス イベントの設置
        match unsafe { CreateEventA(None, false, false, None) } {
            Ok(event) => self.fence_event = event,
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to create fence event: {:?}",
                    err
                )))
            }
        }

        Ok(())
    }
}

/*
//破棄処理
impl Drop for Dx12Resources {
    fn drop(&mut self) {
        unsafe {
            (*self.dxgi_factory).Release();
            (*self.device).Release();
            (*self.swap_chain).Release();
        }
    }
}
*/

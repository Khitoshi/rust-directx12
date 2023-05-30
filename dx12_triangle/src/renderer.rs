#[path = "../src/dx12error.rs"]
mod dx12error;
use dx12error::Dx12Error;

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

pub struct Dx12Resources {
    //ファクトリー デバッグ用
    dxgi_factory: Option<IDXGIFactory4>,
    //デバイス
    device: Option<ID3D12Device>,

    //スワップチェイン
    swap_chain: Option<IDXGISwapChain4>,
    //現在のバッグバッファインデックス
    current_back_buffer_index: u32,

    //レンダリングターゲットビューのディスクリプタヒープ
    rtv_heap: Option<ID3D12DescriptorHeap>,
    //レンダーターゲットビューのサイズ
    rtv_descriptor_size: u32,
    //フレームバッファ用のレンダリングターゲット
    render_targets: Option<[ID3D12Resource; FRAME_BUFFER_COUNT as usize]>,
    //レンダーターゲットハンドル
    //rtv_handle: D3D12_CPU_DESCRIPTOR_HANDLE,

    //深度ステンシルビューのディスクリプタヒープ
    dsv_heap: Option<ID3D12DescriptorHeap>,
    //深度ステンシルビューのサイズ
    dsv_descriptor_size: u32,
    //深度ステンシルバッファ
    depth_stencil_buffer: Option<ID3D12Resource>,
    //深度ステンシルハンドル
    //dsv_handle: D3D12_CPU_DESCRIPTOR_HANDLE,

    //コマンドキュー
    command_queue: Option<ID3D12CommandQueue>,
    //コマンドアロケータ
    command_allocator: Option<ID3D12CommandAllocator>,
    //コマンドリスト
    command_list: Option<ID3D12GraphicsCommandList>,

    //GPUと同期するオブジェクト
    fence: Option<ID3D12Fence>,
    //フェンスの値
    fence_value: i32,
    //
    fence_event: Option<HANDLE>,
}

impl Default for Dx12Resources {
    fn default() -> Self {
        Self {
            dxgi_factory: None,
            device: None,

            swap_chain: None,
            current_back_buffer_index: 0,

            rtv_heap: None,
            rtv_descriptor_size: 0,
            render_targets: None,

            dsv_heap: None,
            dsv_descriptor_size: 0,
            depth_stencil_buffer: None,

            command_queue: None,
            command_allocator: None,
            command_list: None,

            fence: None,
            fence_value: 0,
            fence_event: None,
        }
    }
}

impl Dx12Resources {
    //other method

    //初期化関数
    pub fn new(
        hwnd: HWND,
        frame_buffer_width: u64,
        frame_buffer_height: u32,
    ) -> std::result::Result<Dx12Resources, Box<dyn std::error::Error>> {
        let mut dx12_resources: Dx12Resources = Default::default();

        //DXGIファクトリ生成
        dx12_resources.dxgi_factory = Some(dx12_resources.create_factory()?);

        //デバイスを生成
        dx12_resources.device = Some(dx12_resources.create_device()?);

        //コマンドキュー生成
        dx12_resources.command_queue = Some(dx12_resources.create_commandqueue()?);

        //スワップチェイン作成
        let (swap_chain, current_back_buffer_index) = dx12_resources.create_swapchain(
            &hwnd,
            frame_buffer_width as u32,
            frame_buffer_height,
        )?;
        dx12_resources.swap_chain = Some(swap_chain);
        dx12_resources.current_back_buffer_index = current_back_buffer_index;

        //ウィンドウをフルスクリーンに関連付ける
        dx12_resources.associate_the_window_with_full_screen(&hwnd)?;

        //rtv ディスクリプタヒープ生成 & サイズ取得
        let (rtvdh, rtvds) = dx12_resources.create_rtv_descriptor_heap_for_frame_buffer()?;
        dx12_resources.rtv_heap = Some(rtvdh);
        dx12_resources.rtv_descriptor_size = rtvds;

        //dsv ディスクリプタヒープ生成 & サイズ取得
        let (dsvdh, dsvs) = dx12_resources.create_dsv_descriptor_heap_for_frame_buffer()?;
        dx12_resources.dsv_heap = Some(dsvdh);
        dx12_resources.dsv_descriptor_size = dsvs;

        //フレームバッファ用のレンダーターゲットバッファの生成
        dx12_resources.render_targets = Some(dx12_resources.create_rtv_for_fame_buffer()?);

        //TODO:ここから
        //フレームバッファ用の深度ステンシルバッファの生成
        dx12_resources.depth_stencil_buffer = Some(
            dx12_resources.create_dsv_for_fame_buffer(frame_buffer_width, frame_buffer_height)?,
        );

        //コマンドアロケータの生成
        dx12_resources.command_allocator = Some(dx12_resources.create_command_allocator()?);

        //コマンドリストの生成
        dx12_resources.command_list = Some(dx12_resources.create_command_list()?);

        //GPUと同期オブジェクト生成
        let (fence, fence_value, handle) =
            dx12_resources.create_synchronization_with_gpu_object()?;
        dx12_resources.fence = Some(fence);
        dx12_resources.fence_value = fence_value;
        dx12_resources.fence_event = Some(handle);

        Ok(dx12_resources)
    }
}

//生成処理の実装
impl Dx12Resources {
    //DXGIファクトリ生成
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

    //デバイス生成
    fn create_device(&self) -> std::result::Result<ID3D12Device, Dx12Error> {
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
            //アダプター取得
            let adapter: IDXGIAdapter1 =
                match unsafe { self.dxgi_factory.as_ref().unwrap().EnumAdapters1(i) } {
                    Ok(ap) => ap,
                    Err(_) => {
                        break;
                    }
                };

            //グラフィックス能力のあるdescを取得
            let mut desc: DXGI_ADAPTER_DESC = DXGI_ADAPTER_DESC::default();
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
        let use_adapter: Option<IDXGIAdapter1> =
            if adapter_vender[GpuVender::GpuVenderNvidia as usize].is_some() {
                //NVIDIA
                Some(
                    adapter_vender[GpuVender::GpuVenderNvidia as usize]
                        .clone()
                        .unwrap(),
                )
            } else if adapter_vender[GpuVender::GpuVenderAmd as usize].is_some() {
                //AMD
                Some(
                    adapter_vender[GpuVender::GpuVenderAmd as usize]
                        .clone()
                        .unwrap(),
                )
            } else if adapter_vender[GpuVender::GpuVenderIntel as usize].is_some() {
                //INTEL
                Some(
                    adapter_vender[GpuVender::GpuVenderIntel as usize]
                        .clone()
                        .unwrap(),
                )
            } else {
                //主要ベンダ以外
                Some(adapter_maximum_video_memory.clone().unwrap())
            };

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
                        println!("Device creation succeeded");
                        return Ok(device.unwrap());
                    }
                    Err(_) => {
                        //エラーの場合、次のfeature_levelで試みる
                        continue;
                    }
                }
            }
        }

        //デバイスの生成に失敗
        Err(Dx12Error::new("Failed to generate device"))
    }

    //コマンドキュー生成
    fn create_commandqueue(&self) -> std::result::Result<ID3D12CommandQueue, Dx12Error> {
        // コマンドキューの設定
        let command_queue_desc: D3D12_COMMAND_QUEUE_DESC = D3D12_COMMAND_QUEUE_DESC {
            Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
            Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
            ..Default::default()
        };

        //コマンドキューの生成
        match unsafe {
            self.device
                .as_ref()
                .unwrap()
                .CreateCommandQueue(&command_queue_desc)
        } {
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
    }

    //スワップチェイン作成
    fn create_swapchain(
        &self,
        hwnd: &HWND,
        frame_buffer_width: u32,
        frame_buffer_hegith: u32,
    ) -> std::result::Result<(IDXGISwapChain4, u32), Dx12Error> {
        //スワップチェインの設定
        let desc: DXGI_SWAP_CHAIN_DESC1 = DXGI_SWAP_CHAIN_DESC1 {
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
        let mut swap_chain1: Option<IDXGISwapChain1> = match unsafe {
            self.dxgi_factory.as_ref().unwrap().CreateSwapChainForHwnd(
                &self.command_queue.clone().unwrap(),
                *hwnd,
                &desc,
                None,
                None,
            )
        } {
            Ok(sc) => Some(sc),
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to create swap chain: {:?}",
                    err
                )));
            }
        };

        //swapchain1 を swapchain4に変換する
        let swap_chain4: Option<IDXGISwapChain4> = match swap_chain1.unwrap().cast() {
            Ok(sc) => Some(sc),
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to create swap chian:{:?}",
                    err
                )))
            }
        };

        //バッグバッファ取得
        let current_back_buffer_index: u32 =
            unsafe { swap_chain4.as_ref().unwrap().GetCurrentBackBufferIndex() };

        println!("SwapChain4 creation succeeded");
        Ok((swap_chain4.clone().unwrap(), current_back_buffer_index))
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
                .as_ref()
                .unwrap()
                .MakeWindowAssociation(*hwnd, DXGI_MWA_NO_ALT_ENTER)
        } {
            Ok(_) => {
                println!("bind window succeeded");
                Ok(())
            }
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to create swap chain: {:?}",
                    err
                )))
            }
        }
    }

    //rtv ディスクリプタヒープ生成
    fn create_rtv_descriptor_heap_for_frame_buffer(
        &self,
    ) -> std::result::Result<(ID3D12DescriptorHeap, u32), Dx12Error> {
        //レンダリングターゲットビューのディスクリプタヒープ用のディスクリプタヒープデスクを作成
        let desc: D3D12_DESCRIPTOR_HEAP_DESC = D3D12_DESCRIPTOR_HEAP_DESC {
            NumDescriptors: FRAME_BUFFER_COUNT,
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            ..Default::default()
        };

        let rtv_heap: Option<ID3D12DescriptorHeap> =
            match unsafe { self.device.as_ref().unwrap().CreateDescriptorHeap(&desc) } {
                Ok(rtv) => Some(rtv),

                Err(err) => {
                    return Err(Dx12Error::new(&format!(
                        "Failed to create rtv descriptor heap: {:?}",
                        err
                    )))
                }
            };

        //ディスクリプタのサイズを取得
        let rtv_descriptor_size: u32 = unsafe {
            self.device
                .as_ref()
                .unwrap()
                .GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV)
        };

        println!("rtv descriptor creation succeeded");
        Ok((rtv_heap.unwrap().clone(), rtv_descriptor_size.clone()))
    }

    //dsv ディスクリプタヒープ生成
    fn create_dsv_descriptor_heap_for_frame_buffer(
        &self,
    ) -> std::result::Result<(ID3D12DescriptorHeap, u32), Dx12Error> {
        //深度ステンシルビューのディスクリプタヒープ用のディスクリプタヒープデスクを作成
        let desc: D3D12_DESCRIPTOR_HEAP_DESC = D3D12_DESCRIPTOR_HEAP_DESC {
            NumDescriptors: 1,
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            ..Default::default()
        };

        //深度ステンシルビューのディスクリプタヒープ作成
        let dsv_heap: Option<ID3D12DescriptorHeap> =
            match unsafe { self.device.as_ref().unwrap().CreateDescriptorHeap(&desc) } {
                Ok(dsv) => Some(dsv),
                Err(err) => {
                    return Err(Dx12Error::new(&format!(
                        "Failed to create dsv descriptor heap: {:?}",
                        err
                    )))
                }
            };

        //ディスクリプタのサイズを取得。
        let dsv_descriptor_size: u32 = unsafe {
            self.device
                .as_ref()
                .unwrap()
                .GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_DSV)
        };

        println!("dsv descriptor creation succeeded");
        Ok((dsv_heap.unwrap().clone(), dsv_descriptor_size.clone()))
    }

    //フレームバッファ用のレンダーターゲットバッファの生成
    fn create_rtv_for_fame_buffer(
        &self,
    ) -> std::result::Result<[ID3D12Resource; FRAME_BUFFER_COUNT as usize], Dx12Error> {
        //ヒープの先頭を表すCPUディスクリプタハンドルを取得
        let mut rtv_handle: D3D12_CPU_DESCRIPTOR_HANDLE = unsafe {
            self.rtv_heap
                .as_ref()
                .unwrap()
                .GetCPUDescriptorHandleForHeapStart()
        };

        //フロントバッファをバックバッファ用のRTVを作成
        let render_targets: [ID3D12Resource; FRAME_BUFFER_COUNT as usize] =
            array_init::try_array_init(
                |i: usize| -> std::result::Result<ID3D12Resource, Dx12Error> {
                    let render_target = match unsafe {
                        self.swap_chain
                            .as_ref()
                            .ok_or_else(|| Dx12Error::new("swap_chain is None"))?
                            .GetBuffer(i as u32)
                    } {
                        Ok(resource) => resource,
                        Err(err) => {
                            return Err(Dx12Error::new(&format!(
                            "Failed to get rendertarget of frame buffer  heap at index {}: {:?}",
                            i, err
                        )))
                        }
                    };

                    //レンダーターゲットビューの生成
                    unsafe {
                        self.device
                            .as_ref()
                            .ok_or_else(|| Dx12Error::new("device is None"))?
                            .CreateRenderTargetView(&render_target, None, rtv_handle);
                    };

                    //ポインタを渡したのでずらす
                    rtv_handle.ptr += self.rtv_descriptor_size as usize;

                    //返す
                    Ok(render_target)
                },
            )?;

        println!("render targets creation succeeded");
        Ok(render_targets)
    }

    //フレームバッファ用の深度ステンシルバッファの生成
    fn create_dsv_for_fame_buffer(
        &self,
        frame_buffer_width: u64,
        frame_buffer_height: u32,
    ) -> std::result::Result<ID3D12Resource, Dx12Error> {
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
            self.device.as_ref().unwrap().CreateCommittedResource(
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
                return Err(Dx12Error::new(&format!(
                    "Failed to create depth stencil buffer: {:?}",
                    err
                )))
            }
        }

        println!("depth stencil buffer creation succeeded");
        Ok(depth_stencil_buffer.unwrap().clone())
    }

    //コマンドアロケータの生成
    fn create_command_allocator(&self) -> std::result::Result<ID3D12CommandAllocator, Dx12Error> {
        //コマンドアロケータの生成
        let command_allocator: Option<ID3D12CommandAllocator> = match unsafe {
            self.device
                .as_ref()
                .unwrap()
                .CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)
        } {
            Ok(cmda) => Some(cmda),
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to create command allocator: {:?}",
                    err
                )))
            }
        };

        println!("command allocator creation succeeded");
        Ok(command_allocator.unwrap().clone())
    }

    //コマンドリストの生成
    fn create_command_list(&self) -> std::result::Result<ID3D12GraphicsCommandList, Dx12Error> {
        //コマンドリスト生成
        let command_list: Option<ID3D12GraphicsCommandList> = match unsafe {
            self.device.as_ref().unwrap().CreateCommandList(
                0,
                D3D12_COMMAND_LIST_TYPE_DIRECT,
                self.command_allocator.as_ref().unwrap(),
                None,
            )
        } {
            Ok(cmdl) => Some(cmdl),
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to create command list: {:?}",
                    err
                )))
            }
        };

        //コマンドリストは開かれている状態で生成されるので，一度閉じる
        if let Some(command_list_ref) = command_list.as_ref() {
            match unsafe { command_list_ref.Close() } {
                Ok(()) => (),
                Err(err) => {
                    return Err(Dx12Error::new(&format!(
                        "Failed to close command list: {:?}",
                        err
                    )))
                }
            }
        }

        match command_list {
            Some(cmd_list) => {
                println!("commandList creation succeeded");
                Ok(cmd_list)
            }
            None => Err(Dx12Error::new("Command list was not properly initialized")),
        }
    }

    //GPUと同期オブジェクト生成
    fn create_synchronization_with_gpu_object(
        &self,
    ) -> std::result::Result<(ID3D12Fence, i32, HANDLE), Dx12Error> {
        //GPUと同期オブジェクト(fence)生成
        let fence: Option<ID3D12Fence> = match unsafe {
            self.device
                .as_ref()
                .unwrap()
                .CreateFence(0, D3D12_FENCE_FLAG_NONE)
        } {
            Ok(fence) => Some(fence),
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to create fence: {:?}",
                    err
                )))
            }
        };

        //フェンスの値 設定
        let fence_value: i32 = 1;

        //フェンス イベントの設置
        let handle: Option<HANDLE> = match unsafe { CreateEventA(None, false, false, None) } {
            Ok(event) => Some(event),
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed to create fence event: {:?}",
                    err
                )))
            }
        };

        println!("fence creation succeeded");
        Ok((
            fence.unwrap().clone(),
            fence_value.clone(),
            handle.unwrap().clone(),
        ))
    }
}

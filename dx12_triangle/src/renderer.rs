use mltg_bindings::Windows::Win32::Graphics::Dxgi::DXGI_CREATE_FACTORY_DEBUG;
use winapi::shared::dxgi::IDXGIFactory;
use winapi::shared::dxgi1_3::CreateDXGIFactory2;
use winapi::shared::dxgi1_4::IDXGIFactory4;
use winapi::shared::dxgi1_4::IDXGISwapChain3;
use winapi::shared::guiddef::GUID;
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::d3d12::{
    D3D12GetDebugInterface, ID3D12CommandAllocator, ID3D12CommandQueue, ID3D12DescriptorHeap,
    ID3D12Device, ID3D12Fence, ID3D12GraphicsCommandList, ID3D12Resource,
};
use winapi::um::d3d12sdklayers::ID3D12Debug;

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

pub struct Dx12Resources {
    //ファクトリー デバッグ用
    dxgi_factory: *mut IDXGIFactory,
    //デバイス
    device: *mut ID3D12Device,
    //コマンドキュー
    command_queue: *mut ID3D12CommandQueue,
    //スワップチェイン
    swap_chain: *mut IDXGISwapChain3,
    /*
    //カラーバッファ
    color_buffer:*mut ID3D12Resource,
    //深度バッファ
    depth_buffer: *mut ID3D12Resource,
    //コマンドアロケーター
    command_allocator: * mut ID3D12CommandAllocator,
    //コマンドリスト
    command_list:*mut ID3D12GraphicsCommandList,
    //深度
    descriptor_heap: *mut ID3D12DescriptorHeap,
    Heap_rtv:*mut ID3D12DescriptorHeap,
    fence:*mut ID3D12Fence,
    descriptor_heap_dsv:*mut ID3D12DescriptorHeap,
    descriptor_heap_cbv:*mut ID3D12DescriptorHeap,
    */
}

impl Dx12Resources {
    //other method

    //初期化method
    pub fn new_dx12_resources(&mut self) -> Result<Dx12Resources, Dx12Error> {
        //factory生成
        match self.create_dx12_factory() {
            Ok(factory) => self.dxgi_factory = factory,
            Err(e) => e.print_error(),
        }

        Ok(Dx12Resources {
            device,
            // 他のフィールドの初期化...
        })
    }

    //DXGIオブジェクト生成
    fn create_dx12_factory() -> Result<IDXGIFactory, Dx12Error> {
        // Define the interface IDs
        let mut dxgi_factory_flags: UINT = 0;
        const IID_IDXGIFactory4: GUID = IDXGIFactory4::uuidof();
        const IID_ID3D12Debug: GUID = ID3D12Debug::uuidof();
        // Your code...
        let mut dxgi_factory: *mut IDXGIFactory4 = std::ptr::null_mut();
        let mut debug_controller: *mut ID3D12Debug = std::ptr::null_mut();

        if cfg!(debug_assertions) {
            let hr = unsafe {
                D3D12GetDebugInterface(
                    &IID_ID3D12Debug,
                    &mut debug_controller as *mut _ as *mut *mut _,
                )
            };
            if SUCCEEDED(hr) {
                unsafe { (*debug_controller).EnableDebugLayer() };
                dxgi_factory_flags |= DXGI_CREATE_FACTORY_DEBUG;
                unsafe { (*debug_controller).Release() };
            }
        }

        let hr = unsafe {
            CreateDXGIFactory2(
                dxgi_factory_flags,
                &IID_IDXGIFactory4,
                &mut dxgi_factory as *mut _ as *mut *mut _,
            )
        };

        if SUCCEEDED(hr) {
            Ok(dxgi_factory)
        } else {
            Err(Dx12Error::new("Failed to create DX12 factory"))
        }
    }

    //使用するデバイス情報取得
    fn create_dx12_device() -> Result<ID3D12Device, Dx12Error> {
        //使用している可能性のあるFEATURE_LEVELを列挙
        levels = D3D_FEATURE_LEVEL {
            D3D_FEATURE_LEVEL_12_1,
            D3D_FEATURE_LEVEL_12_0,
            D3D_FEATURE_LEVEL_11_1,
            D3D_FEATURE_LEVEL_11_0,
        };

        //GPUベンダー定義。
        enum GPU_Vender {
            GPU_VENDER_NVIDIA, //NVIDIA
            GPU_VENDER_AMD,    //Intel
            GPU_VENDER_INTEL,  //AMD

            NUM_GPU_VENDER, //Vender数
        };

        //アダプター定義
        let adapter: *mut IDXGIAdapter;
        //各ベンダーのアダプター
        let adapter_vender: [*mut IDXGIAdapter; NUM_GPU_VENDER];
        //最大ビデオメモリのアダプタ
        let adapter_max_video_memory: *mut IDXGIAdapter;
        //最終的に使用するアダプタ
        let use_adapter: *mut IDXGIAdapter;

        //グラフィックスカードが複数枚刺さっている場合にどれが一番メモリ容量が多いかを調べ一番多いものを使用する為のloop
        let mut i = 0;
        loop {
            let result = unsafe { dxgi_factory.EnumAdapters(i, &mut adapter) };
            if result == DXGI_ERROR_NOT_FOUND {
                break;
            }

            let &mut desc: DXGI_ADAPTER_DESC;
            (*adapter).GetDesc(&desc);

            if (desc.DedicatedVideoMemory > video_memory_size) {
                //こちらのビデオメモリの方が多いので、こちらを使う。
                if (adapter_max_video_memory != nullptr) {
                    (*adapter_max_video_memory).Release();
                }
                adapter_max_video_memory = adapter_temp;
                //IDXGIAdapterを登録するたびにインクリメントしないといけないのでaddref(インクリメント)している
                (*adapter_max_video_memory).AddRef();
                video_memory_size = desc.DedicatedVideoMemory;
            }

            //インクリメント i++はcargoに怒られた...
            i += 1
        }
        Ok()
    }
}

//破棄処理
impl Drop for Dx12Resources {
    fn drop(&mut self) {
        unsafe {
            (*self.device).Release();
            (*self.swap_chain).Release();
            (*self.dxgi_factory).Release();
        }
    }
}

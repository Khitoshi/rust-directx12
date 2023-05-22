use com_ptr::{hresult, ComPtr};
use winapi::shared::dxgi::*;
use winapi::um::d3d12::{
    ID3D12CommandAllocator, ID3D12CommandQueue, ID3D12DescriptorHeap, ID3D12Device, ID3D12Fence,
    ID3D12GraphicsCommandList, ID3D12Resource, ID3D12SwapChain3,ID3D12Debug
};
//エラー取得用
pub struct Dx12Error{
    message: String,
}
impl Dx12Error {
    pub fn new(message: &str) -> Dx12Error {
        Dx12Error {
            message: message.to_string(),
        }
    }
    //エラー出力
    pub fn print_error(){
        eprintln!("{}", message);
    }
}

pub struct Dx12Resources {
    //ファクトリー デバッグ用
    dxgi_factory: *mut ID3D12DXGIFactory,
    //デバイス
    device: *mut ID3D12Device,
    //コマンドキュー
    command_queue: *mut ID3D12CommandQueue,
    //スワップチェイン
    swap_chain: *mut ID3D12SwapChain3,
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
    pub fn new_dx12_resources() -> Result<Dx12Resources, Dx12Error> {

        //factory生成
        match self::create_dx12_factory(){
            Ok(factory) => dxgi_factory = factory,
            Err(e) =>  e.print_error(),
        }

        Ok(Dx12Resources {
            device,
            // 他のフィールドの初期化...
        })
    }

    fn create_dx12_factory() -> Result<ID3D12DXGIFactory,Dx12Error>{
        let dxgi_factory_flags : i64;
        dxgi_factory_flags = 1;
        if cfg!(debug_assertions){
            let debugController:*mut ID3D12Debug;
            //デバッグコントローラーがあればバグレイヤーがあるDXGIを作成する
            if SUCCEEDED(D3D12GetDebugInterface(IID_PPV_ARGS(&debugController))){
                (*debugController).EnableDebugLayer();

                //デバッグレイヤーの追加を有効にする
                dxgi_factory_flags |= DXGI_CREATE_FACTORY_DEBUG;
                (*debugController).Release();
            }
        }

        //factory生成
        let hr = CreateDXGIFactory2(dxgi_factory_flags,IID_PPV_ARGS(dxgi_factory));

        //生成チェック
        if SUCCEEDED(hr) {
            Ok(dxgi_factory)
        } else {
            Err(Dx12Error::new("Failed to create DX12 factory"))
        }
    }

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
        let result = unsafe{factory}
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

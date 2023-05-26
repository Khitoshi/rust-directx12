use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D::Fxc::*, Win32::Graphics::Direct3D::*,
    Win32::Graphics::Direct3D12::*, Win32::Graphics::Dxgi::Common::*,
    Win32::Graphics::Dxgi::IDXGIFactory6, Win32::Graphics::Dxgi::*,
    Win32::System::LibraryLoader::*, Win32::System::Threading::*,
    Win32::UI::WindowsAndMessaging::*,
};

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

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
    dxgi_factory: *mut IDXGIFactory4,
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
            use_adapter = adapter_vender[GpuVender::GpuVenderNvidia as usize].clone();
        } else if adapter_vender[GpuVender::GpuVenderAmd as usize].is_some() {
            //AMD
            use_adapter = adapter_vender[GpuVender::GpuVenderAmd as usize].clone();
        } else if adapter_vender[GpuVender::GpuVenderIntel as usize].is_some() {
            //INTEL
            use_adapter = adapter_vender[GpuVender::GpuVenderIntel as usize].clone();
        } else {
            //主要ベンダ以外
            use_adapter = adapter_maximum_video_memory.clone();
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

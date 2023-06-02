//定数バッファ用構造体
pub struct ConstantBuffer {
    //定数バッファ
    constant_buffer: Option<[ID3D12Resource; 2]>,
    //CPU側からアクセスできるする定数バッファのアドレス

    //定数バッファのサイズ
    constant_buffer_size: i64,

    //利用可能フラグ
    is_valid_: boon,
}

impl Default for ConstantBuffer {
    fn default() -> Self {
        Self {
            constant_buffer: None,
            constant_buffer_size: 0,
            is_valid_: false,
        }
    }
}

impl ConstantBuffer {
    //値を譲渡
    pub fn transfer_constant_buffer(&mut self, cb: &ConstantBuffer) {
        //定数バッファ譲渡
        self.constant_buffer[0] = cb.constant_buffer[0];
        self.constant_buffer[1] = cb.constant_buffer[1];

        //CPUからアクセスできる定数バッファの譲渡

        //定数バッファのサイズを譲渡
        self.constant_buffer_size = cb.constant_buffer_size;

        //コピー元定数バッファの削除
        cb.constant_buffer[0] = None;
        cb.constant_buffer[1] = None;
    }

    pub fn new(
        rc: RenderContext,
        constant_buffer_size: i64,
    ) -> std::result::Result<ConstantBuffer, Dx12Error> {
        let mut cb: ConstantBuffer;

        //定数バッファの初期化
        cb.constant_buffer_size = constant_buffer_size;

        //定数バッファは256バイトアライメントが要求されるので,256の倍率に切り上げる
    }
}

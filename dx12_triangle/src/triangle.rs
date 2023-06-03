// モデルデータ (ここではトライアングル)
pub struct Triangle {
    // 頂点バッファ
    vertex_buffer: Option<ID3D12Resource>,
    // 頂点バッファビュー
    vertex_buffer_view: D3D12_VERTEX_BUFFER_VIEW,
}

impl Default for Triangle {
    fn default() -> Self {
        Self {
            vertex_buffer: None,
            vertex_buffer_view: None,
        }
    }
}

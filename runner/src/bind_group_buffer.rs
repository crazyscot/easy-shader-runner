pub struct BufferDescriptor<'a> {
    pub buffer_type: BindGroupBufferType<'a>,
    pub shader_stages: wgpu::ShaderStages,
}

#[allow(dead_code)]
pub enum BindGroupBufferType<'a> {
    Uniform(Uniform<'a>),
    SSBO(SSBO<'a>),
}

pub struct SSBO<'a> {
    pub data: &'a [u8],
    pub read_only: bool,
}

pub struct Uniform<'a> {
    pub data: &'a [u8],
}

use super::*;
#[derive(Clone, Copy, PartialEq)]
pub enum BufferUsage {
  Vertex = gl::ARRAY_BUFFER as isize,
  Index = gl::ELEMENT_ARRAY_BUFFER as isize,
  Uniform = gl::UNIFORM_BUFFER as isize,
  TransformFeedback = gl::TRANSFORM_FEEDBACK_BUFFER as isize,
  TransferSrc = gl::COPY_READ_BUFFER as isize,
  TransferDst = gl::COPY_WRITE_BUFFER as isize,
}
fn usage_to_store_type(usage: BufferUsage) -> u32 {
  // https://developer.mozilla.org/ja/docs/Web/API/WebGLRenderingContext/bufferData
  match usage {
    BufferUsage::Vertex => gl::STATIC_DRAW,
    BufferUsage::Index => gl::STATIC_DRAW,
    BufferUsage::Uniform => gl::STREAM_DRAW,
    BufferUsage::TransformFeedback => gl::STREAM_COPY,
    BufferUsage::TransferSrc => gl::STATIC_DRAW,
    BufferUsage::TransferDst => gl::STATIC_READ,
  }
}

pub struct RawGpuBuffer {
  gl: Rc<GlContext>,
  buffer: web_sys::WebGlBuffer,
  size: i32,
  usage: BufferUsage,
}
impl RawGpuBuffer {
  pub fn new<T: Sized>(gl: &Rc<GlContext>, data: &[T], usage: BufferUsage) -> Self {
    let result = Self::new_uninitialized::<T>(gl, data.len(), usage);
    result.write(0, data);
    result
  }
  pub fn new_uninitialized<T: Sized>(gl: &Rc<GlContext>, count: usize, usage: BufferUsage) -> Self {
    let u8_size = std::mem::size_of::<T>() * count;
    Self::new_uninitialized_untyped(gl, u8_size as i32, usage)
  }
  pub fn new_uninitialized_untyped(gl: &Rc<GlContext>, size: i32, usage: BufferUsage) -> Self {
    let buffer = gl.create_buffer().expect("failed to craete buffer");
    let target = usage as u32;
    gl.bind_buffer(target, Some(&buffer));
    gl.buffer_data_with_i32(target, size, usage_to_store_type(usage));
    if SET_BIND_NONE_AFTER_WORK {
      gl.bind_buffer(target, None);
    }
    Self {
      gl: Rc::clone(gl),
      buffer,
      size,
      usage,
    }
  }
  pub fn write<T: Sized>(&self, offset: usize, data: &[T]) {
    use core::slice;
    let u8_size = std::mem::size_of::<T>() * data.len();
    let ptr = data.as_ptr() as *const u8;
    let u8_data: &[u8] = unsafe { slice::from_raw_parts(ptr, u8_size) };
    let u8_offset = std::mem::size_of::<T>() * offset;
    self.write_untyped(u8_offset as i32, u8_data);
  }
  pub fn write_untyped(&self, offset: i32, data: &[u8]) {
    let size = offset + data.len() as i32;
    if offset < 0 || size > self.size {
      log::error(format!(
        "invalid buffer write size: offset:{}, size:{}, reserved:{}",
        offset, size, self.size
      ));
      return;
    }
    let target = self.usage as u32;
    self.gl.bind_buffer(target, Some(&self.buffer));
    self
      .gl
      .buffer_sub_data_with_i32_and_u8_array(target, offset, data);
    if SET_BIND_NONE_AFTER_WORK {
      self.gl.bind_buffer(target, None);
    }
  }
  pub fn raw_buffer(&self) -> &web_sys::WebGlBuffer {
    &self.buffer
  }
  pub fn raw_target(&self) -> u32 {
    self.usage as u32
  }
}
impl Drop for RawGpuBuffer {
  fn drop(&mut self) {
    self.gl.delete_buffer(Some(&self.buffer));
  }
}

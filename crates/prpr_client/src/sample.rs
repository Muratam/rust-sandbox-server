// hoge_client に逃がす前段階でのサンプル
use super::*;
use prgl;
use std::sync::Arc;
crate::shader_attr! {
  mapping PbrMapping {
    normal_map : sampler2D,
    roughness_map : sampler2D
  }
}
pub struct SampleSystem {
  surface: Arc<prgl::Texture>,
  renderpass: prgl::RenderPass,
  pipeline: prgl::Pipeline,
  camera: prgl::Camera,
}
/* TODO:
- キーボード入力 / タッチ入力を受け取る
  - https://rustwasm.github.io/docs/wasm-bindgen/examples/paint.html
- RenderPassにPipelineを登録する形式にする
  - ステートの変更関数呼び出しを減らしたい
- fullscreenのテンプレートほしい
  - VAOは最後だけに設定できる方がいい (nil -> Vao?)
  - MRTしてポストプロセスをかけてみる
- texture2darray, texture3d 対応する
  - texture として扱いたい？
    - https://ics.media/web3d-maniacs/webgl2_texture2darray/
    - https://ics.media/web3d-maniacs/webgl2_texture3d/
  - texStorage2D
    - https://developer.mozilla.org/en-US/docs/Web/API/WebGL2RenderingContext/copyBufferSubData
    - https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/texImage2D
  - https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#teximagetexsubimage_uploads_esp._videos_can_cause_pipeline_flushes
- client_wait_sync ?
  - https://ics.media/entry/19043/
  - https://inside.pixiv.blog/petamoriken/5853
  - 描画だけをメインスレッドにすればいいかも
  - https://rustwasm.github.io/wasm-bindgen/examples/wasm-in-web-worker.html
- renderbuffer
  - MSAA: https://ics.media/web3d-maniacs/webgl2_renderbufferstoragemultisample/
- zoom-in/outの解像度耐えたい
  - pinch-in/out も
  - window.visualViewport
  - cssの方でscaleいじれば強引にいけそう
- Async Computeしたい
  - tf
- 複数のカメラで描画したい
  - 同じのを別カメラで２回やればOK
  - Selection はカメラから？
  - 指操作はカメラに紐付ける？
  - デバッグ用のが欲しくはなるかも
  - 結局ズーム操作はエミュレーションすることになるのでは
*/
impl System for SampleSystem {
  fn new(core: &Core) -> Self {
    let ctx = core.main_prgl().ctx();
    let surface = Arc::new(Texture::new_rgba_map(ctx, 640, 640, |x, y| {
      Vec4::new(x, y, 1.0, 1.0)
    }));
    let normal_map = Arc::new(Texture::new_rgba_map(ctx, 100, 100, |x, y| {
      Vec4::new(x, y, 0.0, 1.0)
    }));
    let roughness_map = normal_map.clone();
    let mut renderpass = RenderPass::new(ctx);
    renderpass.set_use_default_framebuffer(true);
    // renderpass.set_color_target(Some(&surface));
    let mut pipeline = Pipeline::new(ctx);
    let template = crate::shader_template! {
      attrs: [CameraAttribute, PbrMapping],
      vs_attr: ShapeFactoryVertex,
      fs_attr: { in_color: vec4 },
      out_attr: { out_color: vec4 }
      vs_code: {
        in_color = vec4(position, 1.0) + texture(roughness_map, vec2(0.5, 0.5));
        gl_Position = view_proj_mat * vec4(position, 1.0);
      },
      fs_code: {
        out_color = in_color + texture(normal_map, vec2(0.5, 0.5));
      }
    };
    let vao = ShapeFactory::new(ctx).create_cube();
    pipeline.set_draw_vao(&Arc::new(vao));
    let camera = Camera::new(ctx);
    pipeline.add_uniform_buffer(&camera.ubo);
    if let Some(shader) = Shader::new(ctx, template) {
      pipeline.set_shader(&Arc::new(shader));
    }
    pipeline.add_texture_mapping(&Arc::new(TextureMapping::new(
      ctx,
      PbrMapping {
        normal_map,
        roughness_map,
      },
    )));
    Self {
      surface,
      renderpass,
      pipeline,
      camera,
    }
  }
  fn update(&mut self, core: &Core) {
    let frame = core.frame();
    let prgl = core.main_prgl();
    {
      // update world
      let v = ((frame as f32) / 100.0).sin() * 0.25 + 0.75;
      let color = Vec4::new(v, v, v, 1.0);
      self.renderpass.set_clear_color(Some(color));
      self.renderpass.set_viewport(Some(&prgl.full_viewport()));
      let rad = (frame as f32) / 100.0;
      self.camera.camera_pos = Vec3::new(rad.sin(), rad.cos(), rad.cos()) * 5.0;
      self.camera.aspect_ratio = prgl.aspect_ratio();
      self.camera.update();
    }
    {
      // update draw
      self.renderpass.bind();
      self.pipeline.draw();
      prgl.flush();
    }
    self.render_sample(core);
  }
}

impl SampleSystem {
  fn render_sample(&mut self, core: &Core) {
    // TODO: 2D
    {
      let ctx = core.main_2d_context();
      let width = 0;
      // note use: `?;` for Result
      use std::f64::consts::PI;
      ctx.begin_path();
      ctx.arc(75.0, 75.0, 50.0, 0.0, PI * 2.0).ok();
      ctx.move_to(110.0, 75.0);
      ctx.arc(75.0, 75.0, 35.0, 0.0, PI).ok();
      ctx.move_to(65.0, 65.0);
      ctx.arc(60.0, 65.0, 5.0, 0.0, PI * 2.0).ok();
      ctx.move_to(95.0, 65.0);
      ctx.arc(90.0, 65.0, 5.0, 0.0, PI * 2.0).ok();
      ctx.stroke();
    }
    // TODO: HTML
    {
      let frame = core.frame();
      let html_layer = core.html_layer();
      if frame > 1000 {
        html_layer.set_text_content(None);
      }
      let frame = frame % 200;
      let text = format!("{} ", frame);
      let pre_text = html_layer.text_content().unwrap();
      html_layer.set_text_content(Some(&format!("{}{}", &pre_text, &text)));
    }
  }
}

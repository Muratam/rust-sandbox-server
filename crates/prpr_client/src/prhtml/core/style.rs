use super::*;

// 小さすぎると崩れるので、ある程度の大きさのheightを仮定している
pub const EXPECTED_BROWSER_HEIGHT: f32 = 1000.0;
pub fn convert_percent(x: f32) -> f32 {
  x * EXPECTED_BROWSER_HEIGHT * 0.01
}
pub fn convert_percent_str(x: f32) -> String {
  format!("{}px", convert_percent(x))
}

fn to_css(v: Vec4) -> String {
  fn clamp255(x: f32) -> i32 {
    ((x * 255.0) as i32).clamp(0, 255)
  }
  format!(
    "rgba({},{},{},{:.4})",
    clamp255(v.x),
    clamp255(v.y),
    clamp255(v.z),
    v.w.clamp(0.0, 1.0)
  )
}
pub enum Gradation {
  Linear(f32, Vec<Vec4>),            // degree, rgbas
  Radial(bool, f32, f32, Vec<Vec4>), // is_circle, x, y, rgbas
}
impl Gradation {
  fn to_css(&self) -> String {
    match self {
      Self::Linear(degree, rgbas) => {
        let mut result = format!("linear-gradient({}deg ", *degree as i32);
        for rgba in rgbas {
          result += &format!(", {}", to_css(*rgba));
        }
        result += ")";
        result
      }
      Self::Radial(is_circle, x, y, rgbas) => {
        let mut result = format!(
          "radial-gradient({} at {:.2}% {:.2}%, ",
          if *is_circle { "circle" } else { "ellipse" },
          *x * 100.0,
          *y * 100.0,
        );
        for rgba in rgbas {
          result += &format!(", {}", to_css(*rgba));
        }
        result += ")";
        result
      }
    }
  }
}

pub enum Filter {
  Blur(f32),                       // px per
  Brightness(f32),                 // 1.0: Identity
  Contrast(f32),                   // 1.0: Identity
  DropShadow(f32, f32, f32, Vec4), // x, y, r, rgba
  GrayScale(f32),                  // 0.0: Identity
  HueRotate(f32),                  // Degree
  Invert(f32),                     // 0.0: Identity
  Opacity(f32),                    // 1.0: Identity
  Saturate(f32),                   // 1.0: Identity
  Sepia(f32),                      // 1.0: Identity
}
impl Filter {
  fn value(&self) -> String {
    match self {
      Self::Blur(per) => format!("blur({})", convert_percent_str(*per)),
      Self::Brightness(x) => format!("brightness({:.4})", x),
      Self::Contrast(x) => format!("contrast({:.4})", x),
      Self::GrayScale(x) => format!("grayscale({:.4})", x),
      Self::HueRotate(degree) => format!("hue-rotate({:.4}deg)", degree),
      Self::Invert(x) => format!("invert({:.4})", x),
      Self::Opacity(x) => format!("opacity({:.4})", x),
      Self::Saturate(x) => format!("saturate({:.4})", x),
      Self::Sepia(x) => format!("sepia({:.4})", x),
      Self::DropShadow(x, y, r, rgba) => format!(
        "drop-shadow({} {} {} {} ",
        convert_percent_str(*x),
        convert_percent_str(*y),
        convert_percent_str(*r),
        to_css(*rgba)
      ),
    }
  }
}

#[derive(Clone, Copy)]
pub enum BorderStyle {
  Solid,
  Double,
  Hidden,
  Dashed,
}
impl BorderStyle {
  fn value(&self) -> &'static str {
    match self {
      Self::Solid => "solid",
      Self::Double => "double",
      Self::Hidden => "hidden",
      Self::Dashed => "dashed",
    }
  }
}

#[derive(Clone, Copy)]
pub enum Align {
  Left,
  Right,
  Center,
}
impl Align {
  fn value(&self) -> &'static str {
    match self {
      Self::Left => "left",
      Self::Right => "right",
      Self::Center => "center",
    }
  }
}
#[derive(Clone, Copy)]
pub enum Cursor {
  Auto,
  Default,
  Pointer,
  Wait,
  Text,
  NotAllowed,
  Move,
  CrossHair,
  ColResize,
  RowResize,
}
impl Cursor {
  fn value(&self) -> &'static str {
    match self {
      Self::Auto => "auto",
      Self::Default => "default",
      Self::Pointer => "pointer",
      Self::Wait => "wait",
      Self::Text => "text",
      Self::NotAllowed => "not-allowed",
      Self::Move => "move",
      Self::CrossHair => "crosshair",
      Self::ColResize => "col-resize",
      Self::RowResize => "row-resize",
    }
  }
}
pub trait HtmlElementHolderTrait {
  fn get_raw_element(&self) -> &web_sys::HtmlElement;
  // OVERALL ATTRIBUTE
  fn set_cursor(&self, cursor: Cursor) {
    self.set_by_name_impl("cursor", cursor.value());
  }
  fn set_filter(&self, filter: &Vec<Filter>) {
    self.set_filter_impl(filter);
  }

  fn set_by_name_impl(&self, key: &str, value: &str) {
    let style = self.get_raw_element().style();
    if style.set_property(key, value).is_err() {
      log::error(format!("failted to set_property: {} -> {}", key, value));
    }
  }
  fn set_float_percentage_parameter_impl(&self, key: &str, value: f32) {
    self.set_by_name_impl(key, &convert_percent_str(value));
  }
  fn set_color_impl(&self, key: &str, rgba: Vec4) {
    self.set_by_name_impl(key, &to_css(rgba));
  }
  fn set_shadow_impl(&self, key: &str, dx: f32, dy: f32, blur_radius: f32, rgba: Vec4) {
    self.set_by_name_impl(
      key,
      &format!(
        "{} {} {} {}",
        convert_percent_str(dx),
        convert_percent_str(dy),
        convert_percent_str(blur_radius),
        &to_css(rgba)
      ),
    );
  }
  fn set_filter_impl(&self, filter: &Vec<Filter>) {
    if filter.len() == 0 {
      self.set_by_name_impl("filter", "none");
    } else {
      self.set_by_name_impl(
        "filter",
        &filter
          .iter()
          .map(|x| x.value())
          .collect::<Vec<_>>()
          .join(" "),
      );
    }
  }
}
pub trait HtmlTextHolderTrait
where
  Self: HtmlElementHolderTrait,
{
  // TEXT
  fn set_text_color(&self, rgba: Vec4) {
    self.set_color_impl("color", rgba);
  }
  fn set_text_shadow(&self, dx: f32, dy: f32, blur_radius: f32, rgba: Vec4) {
    self.set_shadow_impl("text-shadow", dx, dy, blur_radius, rgba);
  }
  fn set_text_size(&self, percent: f32) {
    self.set_float_percentage_parameter_impl("font-size", percent);
  }
  fn set_text_line_height(&self, percent: f32) {
    self.set_float_percentage_parameter_impl("line-height", percent);
  }
  fn set_text_letter_spacing(&self, percent: f32) {
    self.set_float_percentage_parameter_impl("letter-spacing", percent);
  }
  fn set_text_bold(&self, is_bold: bool) {
    self.set_by_name_impl("font-weight", if is_bold { "bold" } else { "normal" });
  }
  fn set_text_italic(&self, is_italic: bool) {
    self.set_by_name_impl("font-style", if is_italic { "italic" } else { "normal" });
  }
}
pub trait HtmlContainerTrait
where
  Self: HtmlElementHolderTrait,
{
  // OVERALL
  fn set_align(&self, align: Align) {
    self.set_by_name_impl("text-align", align.value());
  }
  fn set_padding(&self, percent: f32) {
    self.set_padding_left(percent);
    self.set_padding_right(percent);
    self.set_padding_top(percent);
    self.set_padding_bottom(percent);
  }

  // BORDER
  fn set_border_radius(&self, percent: f32) {
    self.set_float_percentage_parameter_impl("border-radius", percent);
  }
  fn set_border_color(&self, rgba: Vec4) {
    self.set_border_left_color(rgba);
    self.set_border_right_color(rgba);
    self.set_border_bottom_color(rgba);
    self.set_border_top_color(rgba);
  }
  fn set_border_width(&self, percent: f32) {
    self.set_border_left_width(percent);
    self.set_border_right_width(percent);
    self.set_border_bottom_width(percent);
    self.set_border_top_width(percent);
  }
  fn set_border_style(&self, border_style: BorderStyle) {
    self.set_border_left_style(border_style);
    self.set_border_right_style(border_style);
    self.set_border_bottom_style(border_style);
    self.set_border_top_style(border_style);
  }

  // BACKGROUND
  fn set_background_color(&self, rgba: Vec4) {
    self.set_color_impl("background-color", rgba);
  }
  fn set_background_gradation(&self, gradation: &Gradation) {
    self.set_by_name_impl("background", &gradation.to_css());
  }
  fn set_background_shadow(&self, dx: f32, dy: f32, blur_radius: f32, rgba: Vec4) {
    self.set_shadow_impl("box-shadow", dx, dy, blur_radius, rgba);
  }

  // EXPERIMENTAL
  fn set_background_textclip(&self) {
    // to clip to gradation
    self.set_by_name_impl("background-clip", "text");
    self.set_by_name_impl("-webkit-background-clip", "text");
    self.set_by_name_impl("color", "transparent");
    self.set_by_name_impl("text-shadow", "none");
  }

  // LRTB
  fn set_padding_left(&self, percent: f32) {
    self.set_float_percentage_parameter_impl("padding-left", percent);
  }
  fn set_padding_right(&self, percent: f32) {
    self.set_float_percentage_parameter_impl("padding-right", percent);
  }
  fn set_padding_top(&self, percent: f32) {
    self.set_float_percentage_parameter_impl("padding-top", percent);
  }
  fn set_padding_bottom(&self, percent: f32) {
    self.set_float_percentage_parameter_impl("padding-bottom", percent);
  }
  fn set_border_left_color(&self, rgba: Vec4) {
    self.set_color_impl("border-left-color", rgba);
  }
  fn set_border_left_width(&self, percent: f32) {
    self.set_float_percentage_parameter_impl("border-left-width", percent);
  }
  fn set_border_left_style(&self, border_style: BorderStyle) {
    self.set_by_name_impl("border-left-style", border_style.value());
  }
  fn set_border_right_color(&self, rgba: Vec4) {
    self.set_color_impl("border-right-color", rgba);
  }
  fn set_border_right_width(&self, percent: f32) {
    self.set_float_percentage_parameter_impl("border-right-width", percent);
  }
  fn set_border_right_style(&self, border_style: BorderStyle) {
    self.set_by_name_impl("border-right-style", border_style.value());
  }
  fn set_border_top_color(&self, rgba: Vec4) {
    self.set_color_impl("border-top-color", rgba);
  }
  fn set_border_top_width(&self, percent: f32) {
    self.set_float_percentage_parameter_impl("border-top-width", percent);
  }
  fn set_border_top_style(&self, border_style: BorderStyle) {
    self.set_by_name_impl("border-top-style", border_style.value());
  }
  fn set_border_bottom_color(&self, rgba: Vec4) {
    self.set_color_impl("border-bottom-color", rgba);
  }
  fn set_border_bottom_width(&self, percent: f32) {
    self.set_float_percentage_parameter_impl("border-bottom-width", percent);
  }
  fn set_border_bottom_style(&self, border_style: BorderStyle) {
    self.set_by_name_impl("border-bottom-style", border_style.value());
  }
}

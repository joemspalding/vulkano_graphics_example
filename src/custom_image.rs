use image::{
    GenericImage,
    GenericImageView,
    ImageBuffer,
    RgbaImage,
    DynamicImage,
    Rgba
};

pub struct CustomImage {
    img: DynamicImage,
    rgba_img: Option<RgbaImage>
}

impl CustomImage {
    pub fn new(path: &str) -> Self {
        CustomImage {
            img: image::open(path).unwrap(),
            rgba_img: None
        }
    }

    pub fn to_rgba_img(&mut self) {
        self.rgba_img = Some(self.img.to_rgba());
    }
    pub fn clear_rgba_img(&mut self) {
        self.rgba_img = None;
    }
    pub fn get_rgba_img(self) -> Option<RgbaImage>{
        self.rgba_img
    }
}
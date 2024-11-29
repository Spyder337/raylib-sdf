use std::{ffi::CString, mem::ManuallyDrop};

use lazy_static::lazy_static;
use raylib::prelude::*;

lazy_static! {
    static ref ZED_FONT: &'static [u8] =
        include_bytes!("data/ZedMono/ZedMonoNerdFontMono-Extended.ttf");
    static ref ZED_SIZE: i32 = (*ZED_FONT).len() as i32;
}

pub struct Text(pub Font, pub ffi::Shader);

pub fn load_font(rl: &mut RaylibHandle, thread: &RaylibThread) -> Font {
    let font = rl
        .load_font(&thread, "data/ZedMono/ZedMonoNerdFontMono-Extended.ttf")
        .expect("Couldn't load font file.");
    let mut texture = font.texture();
    // rl.load_font_ex(_, filename, font_size, chars)
    // texture.set_texture_filter(thread, TextureFilter::);
    font
}

pub fn load_sdf_font(rl: &mut RaylibHandle, thread: &RaylibThread) -> Option<Text> {
    let mut font = rl.get_font_default();
    font.baseSize = 16;
    font.glyphCount = 95;

    if let Some(mut glyphs) = load_sdf_data(rl, *ZED_FONT, font.baseSize, None) {
        font.glyphs = glyphs.0.as_mut_ptr();
        let mut chars: Vec<ffi::GlyphInfo> = glyphs.into();
        let atlas = gen_image_font_atlas(thread, &mut chars, font.baseSize, font.glyphPadding, 0);
        let mut texture = rl.load_texture_from_image(thread, &atlas.0).unwrap();
        texture.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        font.texture = texture.to_raw();
    } else {
        return None;
    }

    let f: Font = unsafe { Font::from_raw(font.to_raw()) };

    let shader = rl
        .load_shader(thread, None, Some("data/shaders/sdf.fs"))
        .expect("Error loading the shader file.");

    Some(Text(f, shader.to_raw()))
}

fn load_sdf_data(
    rl: &mut RaylibHandle,
    data: &[u8],
    font_size: i32,
    chars: Option<&str>,
) -> Option<SliceGlyphInfo> {
    let glyphs = unsafe {
        let glyphs: *mut ffi::GlyphInfo = match chars {
            Some(c) => {
                let mut co = load_codepoints(rl, c);
                ffi::LoadFontData(
                    data.as_ptr(),
                    data.len() as i32,
                    font_size,
                    co.0.as_mut_ptr(),
                    c.len() as i32,
                    2,
                )
            }
            None => ffi::LoadFontData(
                data.as_ptr(),
                data.len() as i32,
                font_size,
                std::ptr::null_mut(),
                0,
                2,
            ),
        };
        let glyph_count = if let Some(c) = chars { c.len() } else { 95 };
        if glyphs.is_null() {
            None
        } else {
            let slice = SliceGlyphInfo(std::mem::ManuallyDrop::new(Box::from_raw(
                std::slice::from_raw_parts_mut(glyphs, glyph_count),
            )));
            Some(slice)
        }
    };
    glyphs
}

struct CodePoints(ManuallyDrop<Box<[i32]>>);

impl Drop for CodePoints {
    fn drop(&mut self) {
        unsafe { ffi::UnloadCodepoints(self.0.as_mut_ptr()) };
    }
}

fn load_codepoints(rl: &mut RaylibHandle, text: &str) -> CodePoints {
    let ptr = CString::new(text).unwrap();
    let mut len = 0;
    //  Load the code points into a buffer.
    let u = unsafe { ffi::LoadCodepoints(ptr.as_ptr(), &mut len) };

    unsafe {
        CodePoints(std::mem::ManuallyDrop::new(Box::from_raw(
            std::slice::from_raw_parts_mut(u, text.len()),
        )))
    }
}

#[derive(Clone)]
struct SliceGlyphInfo(std::mem::ManuallyDrop<std::boxed::Box<[ffi::GlyphInfo]>>);

impl Drop for SliceGlyphInfo {
    fn drop(&mut self) {
        unsafe {
            let inner = std::mem::ManuallyDrop::take(&mut self.0);
            let len = inner.len();
            ffi::UnloadFontData(
                std::boxed::Box::leak(inner).as_mut_ptr() as *mut _,
                len as i32,
            );
        }
    }
}

impl Into<*mut ffi::GlyphInfo> for SliceGlyphInfo {
    fn into(mut self) -> *mut ffi::GlyphInfo {
        self.0.as_mut_ptr()
    }
}

impl Into<Vec<ffi::GlyphInfo>> for SliceGlyphInfo {
    fn into(self) -> Vec<ffi::GlyphInfo> {
        self.0.to_vec()
    }
}

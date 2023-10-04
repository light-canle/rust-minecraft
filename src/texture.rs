use crate::gl_call;

use gl;
use std::os::raw::c_void;
use image::{GenericImageView, ColorType};

pub fn create_texture(path: &str) -> u32{
    let mut id = 0;
    gl_call!(gl::CreateTextures(gl::TEXTURE_2D, 1, &mut id));
    gl_call!(gl::TextureParameteri(id, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_NEAREST as i32));
    gl_call!(gl::TextureParameteri(id, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32));

    let img = image::open(path);
    let img = match img{
        Ok(img) => img.flipv(),
        Err(err) => panic!("Filename: {path}, error : {}", err.to_string())
    };

    match img.color() {
        ColorType::Rgba8 =>{},
        _ => panic!("Texture format not supported.")
    };

    // 텍스쳐를 저장할 스토리지 생성
    gl_call!(gl::TextureStorage2D(
        id, 1, gl::RGBA8, img.width() as i32, img.height() as i32
    ));

    // 여러 애셋이 들어있는 이미지를 불러와 일부만 텍스쳐로 씀
    gl_call!(gl::TextureSubImage2D(
        id, 0,
        0, 0, img.width() as i32, img.height() as i32,
        gl::RGBA, gl::UNSIGNED_BYTE,
        img.as_bytes().as_ptr() as *mut c_void
    ));

    gl_call!(gl::GenerateTextureMipmap(id));
    
    id
}


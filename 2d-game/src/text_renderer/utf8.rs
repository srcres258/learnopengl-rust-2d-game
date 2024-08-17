// SPDX-License-Identifier: Apache-2.0

// Copyright 2024 src_resources
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate nalgebra_glm as glm;

use std::collections::HashMap;
use std::{mem, ptr};
use std::cell::RefCell;
use std::ffi::CString;
use learnopengl_shared::{filesystem, util};
use crate::resource_manager;
use crate::shader::Shader;
use freetype::freetype::{
    FT_Done_Face, FT_Done_FreeType, FT_Face, FT_Init_FreeType,
    FT_Library, FT_Load_Char, FT_LOAD_RENDER, FT_New_Face,
    FT_Set_Pixel_Sizes
};
use crate::text_renderer::ITextRenderer;

/// Holds all state information relevant to a character as loaded using FreeType
#[derive(Copy, Clone, Default)]
pub struct Character {
    texture_id: u32, // ID handle of the glyph texture
    size: glm::IVec2, // size of glyph
    bearing: glm::IVec2, // offset from baseline to left/top of glyph
    advance: u32 // horizontal offset to advance to next glyph
}

/// A utility struct used to connect FreeType library with OpenGL.
struct FTHelper {
    initialized: bool,
    font_size: u32,

    ft: FT_Library,
    face: FT_Face,

    // holds a list of pre-compiled Characters
    characters: HashMap<char, Character>,
}

// A renderer class for rendering text displayed by a font loaded using the
// FreeType library. A single font is loaded, processed into a list of Character
// items for later rendering.
pub struct TextRenderer {
    // shader used for text rendering
    pub text_shader: Shader,

    // render state
    vao: u32,
    vbo: u32,
    // the FreeType library
    ft_helper: RefCell<FTHelper>
}

impl FTHelper {
    fn init(&mut self, font: String, font_size: u32) {
        unsafe {
            if FT_Init_FreeType(&mut self.ft) != 0 { // all functions return a value different than 0 whenever an error occurred
                log::error!("ERROR::FREETYPE: Could not init FreeType Library");
            }
            // load font as face
            let font = CString::new(font).unwrap();
            if FT_New_Face(self.ft, font.as_ptr(), 0, &mut self.face) != 0 {
                log::error!("ERROR::FREETYPE: Failed to load font");
            }
            // set size to load glyphs as
            FT_Set_Pixel_Sizes(self.face, 0, font_size);
        }

        self.font_size = font_size;
        self.initialized = true;
    }

    fn load(&mut self, c: char) -> bool {
        // Ensure the character has not been loaded yet at first.
        if self.characters.iter().any(|it| c == *it.0) {
            return false;
        }

        unsafe {
            // disable byte-alignment restriction
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            // load character glyph
            let face = self.face;
            if FT_Load_Char(face, c as _, FT_LOAD_RENDER as _) != 0 {
                log::error!("ERROR::FREETYPE: Failed to load Glyph");
                return false;
            }
            // generate texture
            let mut texture = 0u32;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RED as _,
                (*(*face).glyph).bitmap.width as _,
                (*(*face).glyph).bitmap.rows as _,
                0,
                gl::RED,
                gl::UNSIGNED_BYTE,
                (*(*face).glyph).bitmap.buffer as _
            );
            // set texture options
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);

            // now store character for later use
            let character = Character {
                texture_id: texture,
                size: glm::vec2((*(*face).glyph).bitmap.width as _, (*(*face).glyph).bitmap.rows as _),
                bearing: glm::vec2((*(*face).glyph).bitmap_left, (*(*face).glyph).bitmap_top),
                advance: (*(*face).glyph).advance.x as _
            };
            self.characters.insert(c, character);

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        true
    }

    fn get_or_load(&mut self, c: char) -> Character {
        if !self.characters.iter().any(|it| c == *it.0) {
            self.load(c);
        }
        self.characters[&c]
    }
}

impl Default for FTHelper {
    fn default() -> Self {
        Self {
            initialized: false,
            font_size: 0,
            ft: ptr::null_mut(),
            face: ptr::null_mut(),
            characters: HashMap::new()
        }
    }
}

impl Drop for FTHelper {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                // destroy FreeType once we're finished
                FT_Done_Face(self.face);
                FT_Done_FreeType(self.ft);
            }
        }
    }
}

impl TextRenderer {
    // constructor
    pub fn new(width: u32, height: u32) -> Self {
        let mut result = Self {
            text_shader: Shader::new(),
            vao: u32::default(),
            vbo: u32::default(),
            ft_helper: RefCell::new(FTHelper::default())
        };

        // load and configure shader
        result.text_shader = resource_manager::load_shader(
            filesystem::get_path("resources/shaders/text_2d.vs".to_string()).as_str(),
            filesystem::get_path("resources/shaders/text_2d.fs".to_string()).as_str(),
            None,
            "text".to_string()
        );
        result.text_shader.set_matrix4_ex("projection", &util::glm::ortho(0.0, width as _, height as _, 0.0), true);
        result.text_shader.set_integer("text", 0);
        unsafe {
            // configure VAO/VBO for texture quads
            gl::GenVertexArrays(1, &mut result.vao);
            gl::GenBuffers(1, &mut result.vbo);
            gl::BindVertexArray(result.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, result.vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (mem::size_of::<f32>() * 6 * 4) as _, ptr::null(), gl::DYNAMIC_DRAW);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, (4 * mem::size_of::<f32>()) as _, ptr::null());
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        result
    }

    // pre-compiles a list of characters from the given font
    pub fn load(&self, font: String, font_size: u32) {
        self.ft_helper.borrow_mut().init(font, font_size);
    }
}

impl ITextRenderer for TextRenderer {
    // renders a string of text using the precompiled list of characters
    fn render_text(
        &self,
        text: String,
        x: f32,
        y: f32,
        scale: f32
    ) {
        self.render_text_ex(
            text,
            x,
            y,
            scale,
            util::glm::scale_vec3(1.0)
        );
    }

    fn render_text_ex(
        &self,
        text: String,
        mut x: f32,
        y: f32,
        scale: f32,
        color: glm::TVec3<f32>
    ) {
        // activate corresponding render state
        self.text_shader.use_shader();
        self.text_shader.set_vector3f("textColor", &color);
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindVertexArray(self.vao);

            // iterate through all characters
            for c in text.chars() {
                let ch = self.ft_helper.borrow_mut().get_or_load(c);

                let xpos = x + ch.bearing.x as f32 * scale;
                let ypos = y + (self.ft_helper.borrow_mut().get_or_load('H').bearing.y - ch.bearing.y) as f32 * scale;

                let w = ch.size.x as f32 * scale;
                let h = ch.size.y as f32 * scale;
                // update VBO for each character
                let vertices = [
                    [xpos    , ypos + h, 0.0, 1.0],
                    [xpos + w, ypos    , 1.0, 0.0],
                    [xpos    , ypos    , 0.0, 0.0],

                    [xpos    , ypos + h, 0.0, 1.0],
                    [xpos + w, ypos + h, 1.0, 1.0],
                    [xpos + w, ypos    , 1.0, 0.0],
                ];
                // render glyph texture over quad
                gl::BindTexture(gl::TEXTURE_2D, ch.texture_id);
                // update content of VBO memory
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                gl::BufferSubData(gl::ARRAY_BUFFER, 0, mem::size_of_val(&vertices) as _, ptr::addr_of!(vertices) as _); // be sure to use glBufferSubData and not glBufferData
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                // render quad
                gl::DrawArrays(gl::TRIANGLES, 0, 6);
                // now advance cursors for next glyph
                x += (ch.advance >> 6) as f32 * scale; // bitshift by 6 to get value in pixels (1/64th times 2^6 = 64)
            }
            gl::BindVertexArray(0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}
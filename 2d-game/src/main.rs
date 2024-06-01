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

use std::ffi::{c_void, CStr};
use std::ptr;
use gl::types::{GLchar, GLenum, GLsizei, GLuint};
use glfw::{Action, Context, Glfw, Key, Modifiers, OpenGlProfileHint, Scancode, Window, WindowHint};
use crate::game::Game;

pub mod game;
pub mod game_level;
pub mod game_object;
pub mod texture;
pub mod power_up;
pub mod sprite_renderer;
pub mod shader;
pub mod resource_manager;
pub mod ball_object;
pub mod particle_generator;
pub mod post_processor;
pub mod text_renderer;
pub mod sound_engine;

// The Width of the screen
const SCREEN_WIDTH: u32 = 800;
// The height of the screen
const SCREEN_HEIGHT: u32 = 600;

static mut GAME_OBJ_PTR: *mut Game = ptr::null_mut();

extern "system" fn gl_debug_output(
    source: GLenum,
    gltype: GLenum,
    id: GLuint,
    severity: GLenum,
    _length: GLsizei,
    message: *const GLchar,
    _user_param: *mut c_void
) {
    if id == 131169 || id == 131185 || id == 131218 || id == 131204 { // ignore these non-significant error codes
        return;
    }

    let message_c_str;
    unsafe {
        message_c_str = CStr::from_ptr(message);
    }
    let message_str = message_c_str.to_str().unwrap();
    log::info!("Debug message ({}): {}", id, message_str);
    match source {
        gl::DEBUG_SOURCE_API => log::info!("Source: API"),
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => log::info!("Source: Window System"),
        gl::DEBUG_SOURCE_SHADER_COMPILER => log::info!("Source: Shader Compiler"),
        gl::DEBUG_SOURCE_THIRD_PARTY => log::info!("Source: Third Party"),
        gl::DEBUG_SOURCE_APPLICATION => log::info!("Source: Application"),
        gl::DEBUG_SOURCE_OTHER => log::info!("Source: Other"),
        _ => {}
    }
    match gltype {
        gl::DEBUG_TYPE_ERROR => log::info!("Type: Error"),
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => log::info!("Type: Deprecated Behaviour"),
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => log::info!("Type: Undefined Behaviour"),
        gl::DEBUG_TYPE_PORTABILITY => log::info!("Type: Portability"),
        gl::DEBUG_TYPE_PERFORMANCE => log::info!("Type: Performance"),
        gl::DEBUG_TYPE_MARKER => log::info!("Type: Marker"),
        gl::DEBUG_TYPE_PUSH_GROUP => log::info!("Type: Push Group"),
        gl::DEBUG_TYPE_POP_GROUP => log::info!("Type: Pop Group"),
        gl::DEBUG_TYPE_OTHER => log::info!("Type: Other"),
        _ => {}
    }
    match severity {
        gl::DEBUG_SEVERITY_HIGH => log::info!("Severity: high"),
        gl::DEBUG_SEVERITY_MEDIUM => log::info!("Severity: medium"),
        gl::DEBUG_SEVERITY_LOW => log::info!("Severity: low"),
        gl::DEBUG_SEVERITY_NOTIFICATION => log::info!("Severity: notification"),
        _ => {}
    }
    log::info!("---------------");
}

fn main() {
    env_logger::init();

    // glfw: initialize and configure
    // ------------------------------
    let mut glfw = glfw::init(glfw::fail_on_errors)
        .expect("Failed to initialise GLFW.");

    glfw.window_hint(WindowHint::ContextVersionMajor(3));
    glfw.window_hint(WindowHint::ContextVersionMinor(3));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(WindowHint::Resizable(false));

    // glfw window creation
    // --------------------
    let (mut window, _) = glfw.create_window(
        SCREEN_WIDTH, SCREEN_HEIGHT,
        "Breakout", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    // load all OpenGL function pointers
    // ---------------------------------
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    window.set_key_callback(key_callback);
    window.set_framebuffer_size_callback(framebuffer_size_callback);

    // OpenGL configuration
    // --------------------
    unsafe {
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS); // makes sure errors are displayed synchronously
        gl::DebugMessageCallback(Some(gl_debug_output), ptr::null());
        gl::DebugMessageControl(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0, ptr::null(), gl::TRUE);

        gl::Viewport(0, 0, SCREEN_WIDTH as _, SCREEN_HEIGHT as _);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    // initialize game
    // ---------------
    init_game_obj(glfw.clone(), SCREEN_WIDTH, SCREEN_HEIGHT);
    game_obj_mut().init();

    // deltaTime variables
    // -------------------
    let mut last_frame = 0f32;

    while !window.should_close() {
        // calculate delta time
        // --------------------
        let current_frame = glfw.get_time() as f32;
        let delta_time = current_frame - last_frame;
        last_frame = current_frame;
        glfw.poll_events();

        // manage user input
        // -----------------
        game_obj_mut().process_input(delta_time);

        // update game state
        // -----------------
        game_obj_mut().update(delta_time);

        // render
        // ------
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        game_obj().render();

        window.swap_buffers();
    }

    // delete all resources as loaded using the resource manager
    // ---------------------------------------------------------
    resource_manager::clear();

    drop_game_obj();
}

fn init_game_obj(glfw: Glfw, width: u32, height: u32) {
    let game = Game::new(glfw, width, height);
    let game = Box::new(game);
    unsafe {
        GAME_OBJ_PTR = Box::leak(game);
    }
}

fn drop_game_obj() {
    let game;
    unsafe {
        game = Box::from_raw(GAME_OBJ_PTR);
    }
    drop(game);
}

pub fn game_obj() -> &'static Game {
    unsafe { &*GAME_OBJ_PTR }
}

pub fn game_obj_mut() -> &'static mut Game {
    unsafe { &mut *GAME_OBJ_PTR }
}

fn key_callback(
    window: &mut Window,
    key: Key,
    _: Scancode,
    action: Action,
    _: Modifiers
) {
    // when a user presses the escape key, we set the WindowShouldClose property to true, closing the application
    if key == Key::Escape && action == Action::Press {
        window.set_should_close(true);
    }
    if (0..1024).contains(&(key as i32)) {
        if action == Action::Press {
            game_obj_mut().keys[key as usize] = true;
        } else if action == Action::Release {
            game_obj_mut().keys[key as usize] = false;
            game_obj_mut().keys_processed[key as usize] = false;
        }
    }
}

fn framebuffer_size_callback(
    _: &mut Window,
    width: i32,
    height: i32
) {
    unsafe {
        // make sure the viewport matches the new window dimensions; note that width and
        // height will be significantly larger than specified on retina displays
        gl::Viewport(0, 0, width, height);
    }
}
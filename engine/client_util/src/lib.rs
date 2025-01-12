// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

#![feature(drain_filter)]
#![feature(must_not_suspend)]
#![feature(hash_raw_entry)]
#![feature(hash_drain_filter)]
#![feature(array_zip)]
#![feature(label_break_value)]
#![feature(mixed_integer_ops)]

extern crate core;

pub mod apply;
#[cfg(feature = "audio")]
pub mod audio;
pub mod browser_storage;
pub mod context;
pub mod entry_point;
pub mod fps_monitor;
pub mod frontend;
pub mod game_client;
pub mod infrastructure;
pub mod joystick;
pub mod js_hooks;
pub mod keyboard;
pub mod mouse;
pub mod rate_limiter;
pub mod reconn_web_socket;
pub mod renderer;
pub mod rgb;
pub mod setting;
pub mod visibility;
pub mod web_socket;

/// Log to javascript console. Use this instead of println!()
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => {
        {
            use wasm_bindgen::prelude::*;

            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(js_namespace = console)]
                pub fn log(s: &str);
            }

            log(&format_args!($($t)*).to_string());
        }
    }
}

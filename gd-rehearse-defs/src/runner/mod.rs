/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use godot::builtin::GString;

pub(crate) mod class;
pub mod config;
pub(crate) mod print;

pub use class::GdTestRunner;

pub(crate) fn is_headless_run() -> bool {
    godot::engine::DisplayServer::singleton().get_name() == GString::from("headless")
}

pub(crate) fn extract_file_subtitle(file: &str) -> &str {
    if let Some(sep_pos) = file.rfind(&['/', '\\']) {
        &file[sep_pos + 1..]
    } else {
        file
    }
}

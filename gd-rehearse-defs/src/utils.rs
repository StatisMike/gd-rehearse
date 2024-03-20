use godot::builtin::GString;

pub(crate) fn is_writable(path: &str) -> bool {
    let res = std::fs::File::create(path);
    if res.is_err() {
        return false;
    }
    if std::fs::remove_file(path).is_err() {
        return false;
    }
    true
}

pub(crate) fn is_headless_run() -> bool {
  godot::engine::DisplayServer::singleton().get_name() == GString::from("headless")
}

pub(crate) fn is_godot_debug() -> bool {
  godot::engine::Os::singleton().is_debug_build()
}

pub(crate) fn is_rust_debug() -> bool {
  cfg!(debug_assertions)
}

pub(crate) fn extract_file_subtitle(file: &str) -> &str {
  if let Some(sep_pos) = file.rfind(&['/', '\\']) {
      &file[sep_pos + 1..]
  } else {
      file
  }
}
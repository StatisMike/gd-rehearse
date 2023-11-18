use std::io::Write;

pub struct GdExtensionWriter {
    gd_extension_path: String,
    godot_path: String,
    libname: String,
}

impl GdExtensionWriter {
    pub (crate) fn write_gd_extension(&self) -> Result<(), GdSetupError> {
        let file_res = std::fs::File::create(&self.gd_extension_path);

        if let Err(error) = file_res {
            return Err(GdSetupError::new(&format!(
                "Can't create file for gdextension: {}",
                error
            )));
        }

        let mut file = file_res.unwrap();

        write!(
            file,
            r#"
[configuration]
entry_symbol = "{libname}"
compatibility_minimum = 4.1

[libraries]
linux.debug.x86_64 = "{godot_path}/debug/lib{libname}.so"
linux.release.x86_64 = "{godot_path}/release/lib{libname}.so"
windows.debug.x86_64 = "{godot_path}/target/debug/{libname}.dll"
windows.release.x86_64 = "{godot_path}/target/release/{libname}.dll"
macos.debug = "{godot_path}/target/debug/lib{libname}.dylib"
macos.release = "{godot_path}/target/release/lib{libname}.dylib"
macos.debug.arm64 = "{godot_path}/target/debug/lib{libname}.dylib"
macos.release.arm64 = "{godot_path}/target/release/lib{libname}.dylib"    
"#,
            libname = self.libname,
            godot_path = self.godot_path
        )
        .unwrap();

        Ok(())
    }

    fn figure_gd_extension_path() -> String {
      String::new()
    }

    fn figure_godot_path() -> String {
      String::new()
    }

    fn figure_libname() -> String {
      String::new()
    }

    fn is_workspace() -> bool {
      false
    }

    pub fn new() -> Self {
      Self {
        gd_extension_path: Self::figure_gd_extension_path(),
        godot_path: Self::figure_godot_path(),
        libname: Self::figure_libname()
      }
    }
}



pub struct GdSetupError {
    message: String,
}

impl GdSetupError {
    pub (crate) fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for GdSetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Problem during godot_test setup: {}",
            self.message
        )
    }
}
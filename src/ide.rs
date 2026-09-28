#![windows_subsystem = "windows"]

#[cfg(windows)]
mod windows_ide {
    use native_windows_gui as nwg;
    use native_windows_derive as nwd;
    use nwd::NwgUi;
    use nwg::NativeUi;
    use std::process::Command;
    use std::fs;
    use std::env;

    fn ypp_exe_path() -> std::path::PathBuf {
        // Look next to the IDE binary first
        if let Ok(mut p) = env::current_exe() {
            p.pop();
            p.push("ypp.exe");
            if p.exists() {
                return p;
            }
        }
        // Fall back to PATH
        std::path::PathBuf::from("ypp.exe")
    }

    fn temp_ypp_path() -> std::path::PathBuf {
        let mut p = env::temp_dir();
        p.push("ypp_ide_temp.ypp");
        p
    }

    #[derive(Default, NwgUi)]
    pub struct YppIde {
        #[nwg_control(size: (1000, 700), position: (200, 100), title: "Y++ IDE v1.2.1")]
        #[nwg_events( OnWindowClose: [YppIde::exit] )]
        window: nwg::Window,

        #[nwg_control(text: "Run", size: (100, 32), position: (10, 460))]
        #[nwg_events( OnButtonClick: [YppIde::run_code] )]
        run_button: nwg::Button,

        #[nwg_control(text: "Open File", size: (100, 32), position: (120, 460))]
        #[nwg_events( OnButtonClick: [YppIde::open_file] )]
        open_button: nwg::Button,

        #[nwg_control(text: "Save File", size: (100, 32), position: (230, 460))]
        #[nwg_events( OnButtonClick: [YppIde::save_file] )]
        save_button: nwg::Button,

        #[nwg_control(text: "Clear Output", size: (110, 32), position: (340, 460))]
        #[nwg_events( OnButtonClick: [YppIde::clear_output] )]
        clear_button: nwg::Button,

        #[nwg_control(
            text: "Import ycomponents *\n\nPRINT: \"Hello, World!\";\n",
            size: (980, 440),
            position: (10, 10),
            flags: "VISIBLE|TAB_STOP|AUTOHSCROLL|AUTOVSCROLL"
        )]
        editor: nwg::TextBox,

        #[nwg_control(
            text: "Output will appear here...",
            size: (980, 180),
            position: (10, 504),
            readonly: true,
            flags: "VISIBLE|TAB_STOP|AUTOHSCROLL|AUTOVSCROLL"
        )]
        console: nwg::TextBox,

        #[nwg_resource]
        file_dialog: nwg::FileDialog,
    }

    impl YppIde {
        fn run_code(&self) {
            let code = self.editor.text();
            let tmp = temp_ypp_path();

            if let Err(e) = fs::write(&tmp, &code) {
                self.console.set_text(&format!("Failed to write temp file: {}", e));
                return;
            }

            let ypp = ypp_exe_path();
            match Command::new(&ypp).arg(&tmp).output() {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    let combined = format!("{}{}", stdout, stderr);
                    self.console.set_text(if combined.is_empty() { "(no output)" } else { &combined });
                }
                Err(e) => {
                    self.console.set_text(&format!(
                        "Failed to run ypp ({}):\n{}\n\nMake sure ypp.exe is in the same folder as ypp-ide.exe.",
                        ypp.display(), e
                    ));
                }
            }
        }

        fn open_file(&self) {
            // Simple: use input box workaround since full dialog needs more setup
            // For now show a message
            self.console.set_text("To open a file: drag and drop a .ypp file, or manually paste its path above.");
        }

        fn save_file(&self) {
            let code = self.editor.text();
            let tmp = temp_ypp_path();
            match fs::write(&tmp, &code) {
                Ok(_) => self.console.set_text(&format!("Saved to: {}", tmp.display())),
                Err(e) => self.console.set_text(&format!("Save failed: {}", e)),
            }
        }

        fn clear_output(&self) {
            self.console.set_text("");
        }

        fn exit(&self) {
            nwg::stop_thread_dispatch();
        }
    }

    pub fn run() {
        nwg::init().expect("Failed to init Native Windows GUI");
        nwg::Font::set_global_family("Consolas").ok();
        let _app = YppIde::build_ui(Default::default()).expect("Failed to build UI");
        nwg::dispatch_thread_events();
    }
}

#[cfg(not(windows))]
fn main() {
    println!("Y++ IDE is currently only supported on Windows.");
    println!("Use 'ypp <file.ypp>' from the terminal instead.");
}

#[cfg(windows)]
fn main() {
    windows_ide::run();
}

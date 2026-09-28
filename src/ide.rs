#![windows_subsystem = "windows"]

use native_windows_gui as nwg;
use native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;
use std::process::Command;
use std::fs;

#[derive(Default, NwgUi)]
pub struct YppIde {
    #[nwg_control(size: (800, 600), position: (300, 300), title: "Y++ IDE")]
    #[nwg_events( OnWindowClose: [YppIde::exit] )]
    window: nwg::Window,

    #[nwg_control(text: "Run", size: (80, 30), position: (10, 420))]
    #[nwg_events( OnButtonClick: [YppIde::run_code] )]
    run_button: nwg::Button,

    #[nwg_control(text: "Import ycomponents *\nImport yGUI *\n", size: (780, 400), position: (10, 10))]
    editor: nwg::TextBox,

    #[nwg_control(text: "Output...", size: (780, 130), position: (10, 460), readonly: true)]
    console: nwg::TextBox,
}

impl YppIde {
    fn run_code(&self) {
        let code = self.editor.text();
        fs::write("temp.ypp", code).unwrap();
        let output = Command::new("ypp.exe")
            .arg("temp.ypp")
            .output();
        
        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                self.console.set_text(&format!("{}{}", stdout, stderr));
            }
            Err(e) => {
                self.console.set_text(&format!("Failed to run: {}", e));
            }
        }
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    let _app = YppIde::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}

/// Built-in Y++ packages. Imports never load files from disk — only these
/// named runtimes can be enabled. That keeps `Import` from becoming a path
/// or code-loading primitive.

pub const PKG_COMPONENTS: &str = "ycomponents";
pub const PKG_NETWORKING: &str = "ynetworking";
pub const PKG_GUI: &str = "ygui";

/// Normalize a user-facing import name (`yGUI`, `YNetworking`, …).
pub fn normalize_package(name: &str) -> String {
    name.trim().to_lowercase()
}

pub fn is_components(name: &str) -> bool {
    normalize_package(name) == PKG_COMPONENTS
}

pub fn is_networking(name: &str) -> bool {
    matches!(normalize_package(name).as_str(), "ynetworking" | "ynetwork")
}

pub fn is_gui(name: &str) -> bool {
    matches!(
        normalize_package(name).as_str(),
        "ygui" | "ygaming" | "ygame" | "ygames"
    )
}

pub fn is_known_package(name: &str) -> bool {
    is_components(name) || is_networking(name) || is_gui(name)
}

pub fn canonical_package(name: &str) -> String {
    if is_networking(name) {
        PKG_NETWORKING.to_string()
    } else if is_gui(name) {
        PKG_GUI.to_string()
    } else {
        normalize_package(name)
    }
}

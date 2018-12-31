pub fn get_target_os() -> Option<String> {
    // https://stackoverflow.com/questions/41742046/is-there-a-list-of-all-cfg-features

    if cfg!(target_os = "windows"){
        Some("windows".to_string())
    } else if cfg!(target_os = "linux") {
        Some("linux".to_string())
    } else if cfg!(target_os = "macos") {
        Some("macos".to_string())
    } else {
        None
    }
}
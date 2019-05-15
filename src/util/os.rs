use os_info;
use platforms::{guess_current, Platform};

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

/// 操作系统信息。
#[derive(Debug)]
pub struct OSInfo {
    pub(crate) os_type: String,
    pub(crate) version: String,
    pub(crate) edition: Option<String>,
    pub(crate) target_os: String,
    pub(crate) target_arch: String,
}

/// 获取操作系统信息。
/// 
/// 包括操作系统类型、系统名称（可选）和版本号信息。
pub fn get_os_info() -> OSInfo {
    let info = os_info::get();

    // TODO: 有没有将 Option<&str> 转换为 Option<String> 的 map 函数？
    let edition = match info.version().edition() {
        Some(x) => Some(x.to_string()),
        None => None,
    };

    let Platform {
        target_os,
        target_arch,
        ..
    } = guess_current().unwrap();

    OSInfo {
        os_type: info.os_type().to_string(),
        version: info.version().version().to_string(),
        edition,
        target_os: target_os.as_str().to_string(),
        target_arch: target_arch.as_str().to_string(),
    }
}
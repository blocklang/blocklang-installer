//! 程序中有两类配置信息，一类是不需要用户修改的，存在 `config.rs` 文件中;
//! 一类是需要用户修改的，约定存在 `install_config.toml` 等 toml 文件中。

pub const ROOT_PATH_APP: &str = "apps";
pub const ROOT_PATH_PROD: &str = "prod";
pub const INSTALLER_CONFIG_FILE_NAME: &str = "installer_config.toml";
pub const DOWNLOAD_CONFIG_FILE_NAME: &str = "download_config.toml";

// 存放 REST API 区
pub const REST_API_INSTALLERS: &str = "installers";
pub const REST_API_APPS: &str = "apps";
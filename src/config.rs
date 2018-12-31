//! 程序中有两类配置信息，一类是不需要用户修改的，存在 `config.r` 文件中;
//! 一类是需要用户修改的，约定存在 `config.toml` 文件中。

use std::fs::File;
use std::io::prelude::*;
use serde_derive::{Deserialize, Serialize};
use toml;

use crate::http::client::InstallerInfo;
use crate::util::net;

#[cfg(test)]
use mockito;

#[cfg(not(test))]
pub const URL: &str = "https://www.blocklang.com";
#[cfg(test)]
pub const URL: &str = mockito::SERVER_URL;

pub const ROOT_PATH_SOFTWARE: &str = "softwares";
pub const ROOT_PATH_PROD: &str = "prod";
pub const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub installers: Option<Vec<InstallerConfig>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallerConfig {
    pub url: String,
    pub token: String,
    pub server_token: String,
    pub software_name: String,
    pub software_version: String,
    pub software_file_name: String,
    pub software_run_port: u32,
    pub jdk_name: String,
    pub jdk_version: String,
    pub jdk_file_name: String,
}

/// 将 Installer 信息存储在 config.toml 文件中。
pub fn save(installer_info: InstallerInfo) {
    let interface_addr = net::get_interface_address().expect("获取不到能联网的有线网络");
    let server_token = interface_addr.mac_address;
    // 设置配置信息
    let config = Config {
        installers: Some(vec!(InstallerConfig {
            url: installer_info.url,
            token: installer_info.token,
            server_token: server_token,
            software_name: installer_info.software_name,
            software_version: installer_info.software_version,
            software_file_name: installer_info.software_file_name,
            software_run_port: installer_info.software_run_port,
            jdk_name: installer_info.jdk_name,
            jdk_version: installer_info.jdk_version,
            jdk_file_name: installer_info.jdk_file_name,
        })),
    };
    let toml_content = toml::to_vec(&config).unwrap();

    // 在 config.toml 文件中存储配置信息
    let mut file = File::create(CONFIG_FILE_NAME).expect("failed to create config.toml file");
    file.write_all(toml_content.as_slice()).expect("failed to save config.toml content");
}

/// 从 config.toml 文件中读取 Installer 信息。
pub fn read() -> Result<Config, Box<std::error::Error>> {
    let mut file = File::open(CONFIG_FILE_NAME)?;
    // TODO: 如何修改默认的提示信息，并能往外传递，如果使用 expect 的话，就地退出了，并没有传到 main 函数中。
    // .expect(&format!("找不到 {} 文件，请先执行 register 命令，注册一个 installer", CONFIG_FILE_NAME));
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

#[cfg(test)]
mod tests {

    use std::path::Path;
    use std::fs::{self, File};
    use std::io::prelude::*;
    use crate::http::client::InstallerInfo;
    use super::{save, Config, CONFIG_FILE_NAME};

    #[test]
    fn save_config_success() -> Result<(), Box<std::error::Error>> {
        let installer_info = InstallerInfo {
            url: "1".to_string(),
            token: "12".to_string(),
            software_name: "3".to_string(),
            software_version: "4".to_string(),
            software_file_name: "5".to_string(),
            software_run_port: 6_u32,
            jdk_name: "7".to_string(),
            jdk_version: "8".to_string(),
            jdk_file_name: "9".to_string(),
        };
        save(installer_info);

        // 断言存在 config.toml 文件
        assert!(Path::new(CONFIG_FILE_NAME).exists());
        // 读取文件中的内容，并比较部分内容
        let mut file = File::open(CONFIG_FILE_NAME)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        assert!(buffer.contains("[[installers]]"));

        // 删除 config.toml 文件
        fs::remove_file(CONFIG_FILE_NAME)?;

        Ok(())
    }

    // 当前只支持配置一个 installers，所以如果多次保存，则只存储最后一个配置信息。
    #[test]
    fn save_config_twice() -> Result<(), Box<std::error::Error>> {
        let installer_info_1 = InstallerInfo {
            url: "1".to_string(),
            token: "2".to_string(),
            software_name: "3".to_string(),
            software_version: "4".to_string(),
            software_file_name: "5".to_string(),
            software_run_port: 6_u32,
            jdk_name: "7".to_string(),
            jdk_version: "8".to_string(),
            jdk_file_name: "9".to_string(),
        };
        let installer_info_2 = InstallerInfo {
            url: "a".to_string(),
            token: "b".to_string(),
            software_name: "c".to_string(),
            software_version: "d".to_string(),
            software_file_name: "e".to_string(),
            software_run_port: 66_u32,
            jdk_name: "f".to_string(),
            jdk_version: "g".to_string(),
            jdk_file_name: "h".to_string(),
        };

        save(installer_info_1);
        save(installer_info_2);

        // 断言存在 config.toml 文件
        assert!(Path::new(CONFIG_FILE_NAME).exists());
        
        // 读取文件中的内容，并比较部分内容
        let mut file = File::open(CONFIG_FILE_NAME)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        
        let config: Config = toml::from_str(buffer.as_str()).unwrap();

        let installers = config.installers;
        let installers = installers.unwrap();
        assert_eq!(1, installers.len());
        assert_eq!("a", installers.get(0).unwrap().url);
        assert_eq!(66, installers.get(0).unwrap().software_run_port);

        // 删除 config.toml 文件
        fs::remove_file(CONFIG_FILE_NAME)?;

        Ok(())
    }

}
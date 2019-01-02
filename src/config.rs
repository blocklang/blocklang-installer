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

    save_to(config, CONFIG_FILE_NAME);
}

fn save_to(config: Config, file_name: &str) {
    let toml_content = toml::to_vec(&config).unwrap();

    // 在 config.toml 文件中存储配置信息
    let mut file = File::create(file_name).expect("failed to create config.toml file");
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
    use super::{save_to, Config, InstallerConfig};

    /// 注意，测试用例中的 config file name 不能相同，
    /// 因为用例中有删除 config file 的代码，
    /// 而测试用例是平行运行的，因此会出现干扰。
    #[test]
    fn save_config_success() -> Result<(), Box<std::error::Error>> {
        let config_file_name = "config1.toml";

        let config = Config {
            installers: Some(vec!(InstallerConfig {
                url: "1".to_string(),
                token: "2".to_string(),
                server_token: "3".to_string(),
                software_name: "3".to_string(),
                software_version: "4".to_string(),
                software_file_name: "5".to_string(),
                software_run_port: 6_u32,
                jdk_name: "7".to_string(),
                jdk_version: "8".to_string(),
                jdk_file_name: "9".to_string(),
            })),
        };
        save_to(config, config_file_name);

        // 断言存在 config.toml 文件
        assert!(Path::new(config_file_name).exists());
        // 读取文件中的内容，并比较部分内容
        let mut file = File::open(config_file_name)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        assert!(buffer.contains("[[installers]]"));

        // 删除 config.toml 文件
        fs::remove_file(config_file_name)?;

        Ok(())
    }

    // 当前只支持配置一个 installers，所以如果多次保存，则只存储最后一个配置信息。
    #[test]
    fn save_config_twice() -> Result<(), Box<std::error::Error>> {
        // 每个测试用例中的 config file name 不能相同。
        let config_file_name = "config2.toml";

        let config_1 = Config {
            installers: Some(vec!(InstallerConfig {
                url: "1".to_string(),
                token: "2".to_string(),
                server_token: "3".to_string(),
                software_name: "3".to_string(),
                software_version: "4".to_string(),
                software_file_name: "5".to_string(),
                software_run_port: 6_u32,
                jdk_name: "7".to_string(),
                jdk_version: "8".to_string(),
                jdk_file_name: "9".to_string(),
            })),
        };

        let config_2 = Config {
            installers: Some(vec!(InstallerConfig {
                url: "a".to_string(),
                token: "b".to_string(),
                server_token: "c".to_string(),
                software_name: "d".to_string(),
                software_version: "e".to_string(),
                software_file_name: "f".to_string(),
                software_run_port: 77_u32,
                jdk_name: "g".to_string(),
                jdk_version: "h".to_string(),
                jdk_file_name: "i".to_string(),
            })),
        };

        save_to(config_1, config_file_name);
        save_to(config_2, config_file_name);

        // 断言存在 config.toml 文件
        assert!(Path::new(config_file_name).exists());
        
        // 读取文件中的内容，并比较部分内容
        let mut file = File::open(config_file_name)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        
        let config: Config = toml::from_str(buffer.as_str()).unwrap();

        let installers = config.installers;
        let installers = installers.unwrap();
        assert_eq!(1, installers.len());
        assert_eq!("a", installers.get(0).unwrap().url);
        assert_eq!(77, installers.get(0).unwrap().software_run_port);

        // 删除 config.toml 文件
        fs::remove_file(config_file_name)?;

        Ok(())
    }

}
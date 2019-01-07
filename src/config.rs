//! 程序中有两类配置信息，一类是不需要用户修改的，存在 `config.r` 文件中;
//! 一类是需要用户修改的，约定存在 `config.toml` 文件中。

use std::fs::File;
use std::path::Path;
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
    /// 服务器 token，为每个服务器生成唯一的 token
    /// 此 token 一旦生成就不能修改，目前使用的是 MAC 地址。
    pub server_token: String,
    pub installers: Option<Vec<InstallerConfig>>,
}

/// 注意，虽然 `InstallerInfo` 的字段和 InstallerConfig 的字段一样，
/// 但是因为 `InstallerInfo` 是用于从服务中获取数据，需要做字段名的驼峰转换，
/// 所以这里又定义了一个对应的 Config 类。
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct InstallerConfig {
    pub url: String,
    /// 为每个 installer 生成唯一的 token
    /// 一个应用服务器上可安装多个 installer。
    /// 注意，在 config 中存储的是 installer token，不是 registration token。
    pub installer_token: String,
    pub software_name: String,
    pub software_version: String,
    pub software_file_name: String,
    pub software_run_port: u32,
    pub jdk_name: String,
    pub jdk_version: String,
    pub jdk_file_name: String,
}

// TODO: 第一次创建 `config.toml` 文件时，要生成一个 server_token
/// 将 Installer 信息存储在 config.toml 文件中。
pub fn save(config_info: Config) {
    save_to(config_info, CONFIG_FILE_NAME);
}

fn save_to(config: Config, file_name: &str) {
    let toml_content = toml::to_vec(&config).unwrap();

    // 在 config.toml 文件中存储配置信息
    let mut file = File::create(file_name).expect("failed to create config.toml file");
    file.write_all(toml_content.as_slice()).expect("failed to save config.toml content");
}

pub fn add_installer(config_info: &mut Config, installer_info: InstallerInfo) {
    let installer_config = InstallerConfig {
        url: installer_info.url,
        installer_token: installer_info.installer_token,
        software_name: installer_info.software_name,
        software_version: installer_info.software_version,
        software_file_name: installer_info.software_file_name,
        software_run_port: installer_info.software_run_port,
        jdk_name: installer_info.jdk_name,
        jdk_version: installer_info.jdk_version,
        jdk_file_name: installer_info.jdk_file_name,
    };

    let installers = config_info.installers.as_mut();

    match installers {
        None => {config_info.installers = Some(vec!(installer_config))},
        Some(x) => {x.push(installer_config)}
    }
}

// TODO: remove installer



/// 从 config.toml 文件中读取 Installer 信息。
pub fn read() -> Result<Config, Box<std::error::Error>> {
    read_from(CONFIG_FILE_NAME)
}

fn read_from(file_name: &str) -> Result<Config, Box<std::error::Error>> {
    let mut file = File::open(file_name)?;
    // TODO: 如何修改默认的提示信息，并能往外传递，如果使用 expect 的话，就地退出了，并没有传到 main 函数中。
    // .expect(&format!("找不到 {} 文件，请先执行 register 命令，注册一个 installer", CONFIG_FILE_NAME));
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

pub fn get() -> Result<Config, Box<std::error::Error>> {
    let config_path = Path::new(CONFIG_FILE_NAME);
    if config_path.exists() {
        return read_from(CONFIG_FILE_NAME);
    }

    let net_interface = net::get_interface_address().unwrap();
    Ok(Config {
        server_token: net_interface.mac_address,
        installers: None,
    })
}

#[cfg(test)]
mod tests {

    use std::path::Path;
    use std::fs::{self, File};
    use std::io::prelude::*;
    use crate::util::net;
    use crate::http::client::InstallerInfo;
    use super::{save_to, get, add_installer, Config, InstallerConfig};

    /// 默认是没有 `config.toml` 配置文件的，所以第一次不会读取 `config.toml` 文件，
    /// 而是会设置一些初始值。
    #[test]
    fn get_config_first_time() -> Result<(), Box<std::error::Error>> {
        let config_info = get()?;

        let net_interface = net::get_interface_address().unwrap();
        assert_eq!(net_interface.mac_address, config_info.server_token);
        assert_eq!(None, config_info.installers);

        Ok(())
    }

    #[test]
    fn add_installer_once() {
        let mut config_info = Config {
            server_token: "1".to_string(),
            installers: None,
        };

        let installer_info = InstallerInfo {
            url: "1".to_string(),
            installer_token: "2".to_string(),
            software_name: "3".to_string(),
            software_version: "4".to_string(),
            software_file_name: "5".to_string(),
            software_run_port: 6_u32,
            jdk_name: "7".to_string(),
            jdk_version: "8".to_string(),
            jdk_file_name: "9".to_string(),
        };
        add_installer(&mut config_info, installer_info);

        assert_eq!(1, config_info.installers.unwrap().len());
    }

    #[test]
    fn add_installer_twice() {
        let mut config_info = Config {
            server_token: "1".to_string(),
            installers: None,
        };

        let installer_info_1 = InstallerInfo {
            url: "1".to_string(),
            installer_token: "2".to_string(),
            software_name: "3".to_string(),
            software_version: "4".to_string(),
            software_file_name: "5".to_string(),
            software_run_port: 6_u32,
            jdk_name: "7".to_string(),
            jdk_version: "8".to_string(),
            jdk_file_name: "9".to_string(),
        };
        add_installer(&mut config_info, installer_info_1);

        let installer_info_2 = InstallerInfo {
            url: "11".to_string(),
            installer_token: "22".to_string(),
            software_name: "33".to_string(),
            software_version: "44".to_string(),
            software_file_name: "55".to_string(),
            software_run_port: 66_u32,
            jdk_name: "77".to_string(),
            jdk_version: "88".to_string(),
            jdk_file_name: "99".to_string(),
        };
        add_installer(&mut config_info, installer_info_2);

        assert_eq!(2, config_info.installers.unwrap().len());
    }

    /// 注意，测试用例中的 config file name 不能相同，
    /// 因为用例中有删除 config file 的代码，
    /// 而测试用例是平行运行的，因此会出现干扰。
    #[test]
    fn save_config_success() -> Result<(), Box<std::error::Error>> {
        let config_file_name = "config1.toml";

        let config = Config {
            server_token: "server_1".to_string(),
            installers: Some(vec!(InstallerConfig {
                url: "1".to_string(),
                installer_token: "2".to_string(),
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

        assert!(buffer.contains(r#"server_token = "server_1""#));
        assert!(buffer.contains("[[installers]]"));

        // 删除 config.toml 文件
        fs::remove_file(config_file_name)?;

        Ok(())
    }

    // 如果调用 save_to 函数多次，则覆盖之前的配置信息，只存储最后一个配置信息。
    
    #[test]
    fn save_config_twice() -> Result<(), Box<std::error::Error>> {
        // 每个测试用例中的 config file name 不能相同。
        let config_file_name = "config2.toml";

        let config_1 = Config {
            server_token: "server_1".to_string(),
            installers: Some(vec!(InstallerConfig {
                url: "1".to_string(),
                installer_token: "2".to_string(),
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
            server_token: "server_2".to_string(),
            installers: Some(vec!(InstallerConfig {
                url: "a".to_string(),
                installer_token: "b".to_string(),
                software_name: "c".to_string(),
                software_version: "d".to_string(),
                software_file_name: "e".to_string(),
                software_run_port: 66_u32,
                jdk_name: "f".to_string(),
                jdk_version: "g".to_string(),
                jdk_file_name: "h".to_string(),
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
        assert_eq!(66, installers.get(0).unwrap().software_run_port);

        // 删除 config.toml 文件
        fs::remove_file(config_file_name)?;

        Ok(())
    }

}
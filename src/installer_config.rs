use std::fs::File;
use std::io::prelude::*;
use serde_derive::{Deserialize, Serialize};
use toml;

use crate::http::client::InstallerInfo;
use crate::util::net;
use crate::config::INSTALLER_CONFIG_FILE_NAME;

pub struct InstallerConfig {
    file_name: String,
    data: InstallerData,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct InstallerData {
    /// 服务器 token，为每个服务器生成唯一的 token
    /// 此 token 一旦生成就不能修改，目前使用的是 MAC 地址。
    pub server_token: String,
    pub installers: Vec<Installer>,
}

/// 注意，虽然 `InstallerInfo` 的字段和 Installer 的字段一样，
/// 但是因为 `InstallerInfo` 是用于从服务中获取数据，需要做字段名的驼峰转换，
/// 所以这里又定义了一个对应的 Config 类。
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Installer {
    pub url: String,
    /// 为每个 installer 生成唯一的 token
    /// 一个应用服务器上可安装多个 installer。
    /// 注意，在 config 中存储的是 installer token，不是 registration token。
    pub installer_token: String,
    pub app_name: String,
    pub app_version: String,
    pub app_file_name: String,
    pub app_run_port: u32,
    pub jdk_name: String,
    pub jdk_version: String,
    pub jdk_file_name: String,
}

impl Default for InstallerConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallerConfig {

    // 使用默认的配置文件
    pub fn new() -> Self {
        Self::from(INSTALLER_CONFIG_FILE_NAME)
    }

    pub fn from(file_name: &str) -> Self {
        File::open(file_name).map(|mut file| {
            // 如果文件存在，则读取文件内容
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap_or(0);
            content
        }).map(|content| {
            toml::from_str::<InstallerData>(&content).unwrap_or_else(|_| {
                // 不是预期的 toml 格式，则创建默认设置
                Self::create_default_config(file_name).data
            })
        }).map(|data| {
                InstallerConfig {
                file_name: file_name.to_string(),
                data,
            }
        }).unwrap_or_else(|_|{
            // 如果文件不存在，则创建一个文件，并返回文件的内容
            Self::create_default_config(file_name)
        })
    }

    pub fn add(&mut self, installer_info: InstallerInfo) {
        let installer_config = Installer {
            url: installer_info.url.unwrap(),
            installer_token: installer_info.installer_token,
            app_name: installer_info.app_name,
            app_version: installer_info.app_version,
            app_file_name: installer_info.app_file_name,
            app_run_port: installer_info.app_run_port,
            jdk_name: installer_info.jdk_name,
            jdk_version: installer_info.jdk_version,
            jdk_file_name: installer_info.jdk_file_name,
        };

        self.data.installers.push(installer_config);
        self.save();
    }

    pub fn update(&mut self, app_run_port: u32, installer_info: InstallerInfo) {
       if let Some(mut elem) = self.data.installers.iter_mut().find(|elem| {
            elem.app_run_port == app_run_port
       }) {
			elem.url = installer_info.url.unwrap();
            elem.installer_token = installer_info.installer_token;
            elem.app_name = installer_info.app_name;
            elem.app_version = installer_info.app_version;
            elem.app_file_name = installer_info.app_file_name;
            elem.jdk_name = installer_info.jdk_name;
            elem.jdk_version = installer_info.jdk_version;
            elem.jdk_file_name = installer_info.jdk_file_name;
            self.save();
		}
    }

    /// 注意，一台主机上的一个端口上只能部署一个应用，所以可以根据 port 唯一定义一个 installer
    pub fn get_by_port(&self, app_run_port: u32) -> Option<&Installer>{
        self.data.installers.iter().find_map(|installer| {
            if installer.app_run_port == app_run_port {
                Some(installer)
            } else { 
                None
            }
        })
    }

    pub fn remove_by_installer_token(&mut self, installer_token: &str) {
        let installers = &mut self.data.installers;

        match installers.iter().position(|item| item.installer_token == installer_token) {
            None => {},
            Some(index) => {
                installers.remove(index);
            }
        };

        self.save();
    }

    /// 删除所有 installer
    /// 可通过函数来判断每一个 installer 是否可以删除，如果返回 true，则删除；如果返回 false 则不删除
    /// 在每一个操作中删除配置信息，因此可以不需要统一删除
    pub fn remove_all<F>(&mut self, mut f: F) where F: FnMut(&Installer) -> bool {
        let installers = &mut self.data.installers;

        installers.retain(|installer| {
           !f(installer)
        });

        // 因为在 f 函数中删除了配置信息，所以这里可以不在执行 save 操作
        self.save();
    }

    pub fn get_data(&self) -> &InstallerData {
        &self.data
    }

    /// 创建一个默认的配置
    fn create_default_config(file_name: &str) -> Self {
        let net_interface = net::get_interface_address().unwrap();
        let data = InstallerData {
            server_token: net_interface.mac_address,
            installers: Vec::<Installer>::new()
        };
        let installer_config = InstallerConfig {
            file_name: file_name.to_string(),
            data,
        };

        installer_config.save();

        installer_config
    }

    fn save(&self) {
        let toml_content = toml::to_vec(&self.data).unwrap();

        let mut file = File::create(&self.file_name).expect("failed to create installer_config.toml file");
        file.write_all(toml_content.as_slice()).expect("failed to save installer_config.toml content");
    }
}

#[cfg(test)]
mod tests {

    use std::path::Path;
    use std::fs::{self, File};
    use std::io::prelude::*;

    use crate::util::net;
    use crate::http::client::InstallerInfo;
    use super::{InstallerConfig, InstallerData, Installer};

    /// 注意，测试用例中的 config file name 不能相同，
    /// 因为用例中有删除 config file 的代码，
    /// 而测试用例是平行运行的，因此会出现干扰。
    #[test]
    fn from_config_file_not_exist() -> Result<(), Box<std::error::Error>>  {
        let file_name = "not-exist-installer-config.toml";

        assert!(!Path::new(file_name).exists());

        let mac_address = net::get_interface_address().unwrap().mac_address;

        let installer_config = InstallerConfig::from(file_name);
        assert_eq!(file_name, installer_config.file_name);
        assert_eq!(InstallerData {
            server_token: mac_address,
            installers: Vec::<Installer>::new(),
        }, installer_config.data);
        
        assert!(Path::new(file_name).exists());

        // 判断文件中的内容
        let mut file = File::open(file_name)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mac_address = net::get_interface_address().unwrap().mac_address;
        let server_token_part = &format!("server_token = \"{}\"", mac_address);
        assert!(content.contains(server_token_part));
        assert!(!content.contains("[[installers]]"));

        // 测试完成后，删除文件
        fs::remove_file(file_name)?;
        Ok(())
    }

    #[test]
    fn from_config_file_exist_but_content_is_empty() -> Result<(), Box<std::error::Error>>  {
        let file_name = "exist-installer-config_empty.toml";
        File::create(file_name)?;

        assert!(Path::new(file_name).exists());

        let installer_config = InstallerConfig::from(file_name);
        assert_eq!(file_name, installer_config.file_name);
        assert_eq!(InstallerData {
            server_token: net::get_interface_address().unwrap().mac_address,
            installers: Vec::<Installer>::new(),
        }, installer_config.data);

        assert!(Path::new(file_name).exists());

        // 判断文件中的内容
        let mut file = File::open(file_name)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mac_address = net::get_interface_address().unwrap().mac_address;
        let server_token_part = &format!("server_token = \"{}\"", mac_address);
        assert!(content.contains(server_token_part));
        assert!(!content.contains("[[installers]]"));

        // 删除 installer_config.toml 文件
        fs::remove_file(file_name)?;
        Ok(())
    }

    #[test]
    fn from_config_file_exist_has_server_token_and_empty_installer() -> Result<(), Box<std::error::Error>>  {
        let file_name = "exist-installer-config_has_server_token_and_empty_installer.toml";
        // 注意，installers 的值为空时，toml 是按数组解析的
        let toml_content = r#"
        server_token = "11"
        installers = []
        "#;
        let mut file = File::create(file_name)?;
        file.write_all(toml_content.as_bytes()).expect("failed to save installer_config.toml content");

        assert!(Path::new(file_name).exists());

        let installer_config = InstallerConfig::from(file_name);
        assert_eq!(file_name, installer_config.file_name);
        assert_eq!(InstallerData {
            server_token: "11".to_string(),
            installers: Vec::<Installer>::new(),
        }, installer_config.data);

        assert!(Path::new(file_name).exists());

        // 判断文件中的内容
        let mut file = File::open(file_name)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        assert!(content.contains("server_token = \"11\""));
        assert!(!content.contains("[[installers]]"));

        // 删除 installer_config.toml 文件
        fs::remove_file(file_name)?;
        Ok(())
    }

    #[test]
    fn from_config_file_exist_has_invalid_toml() -> Result<(), Box<std::error::Error>>  {
        let file_name = "exist-installer-config_invalid.toml";
        // 注意，installers 的值为空时，toml 是按数组解析的
        let toml_content = r#"
        a = "11"
        b = []
        "#;
        let mut file = File::create(file_name)?;
        file.write_all(toml_content.as_bytes()).expect("failed to save installer_config.toml content");

        assert!(Path::new(file_name).exists());

        let installer_config = InstallerConfig::from(file_name);
        assert_eq!(file_name, installer_config.file_name);
        assert_eq!(InstallerData {
            server_token: net::get_interface_address().unwrap().mac_address,
            installers: Vec::<Installer>::new(),
        }, installer_config.data);

        assert!(Path::new(file_name).exists());

        // 判断文件中的内容
        let mut file = File::open(file_name)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mac_address = net::get_interface_address().unwrap().mac_address;
        let server_token_part = &format!("server_token = \"{}\"", mac_address);
        assert!(content.contains(server_token_part));
        assert!(!content.contains("[[installers]]"));

        // 删除 installer_config.toml 文件
        fs::remove_file(file_name)?;
        Ok(())
    }

    #[test]
    fn add_a_installer_success() -> Result<(), Box<std::error::Error>> {
        let file_name = "add_a_installer_success.toml";
        let mut installer_config = InstallerConfig::from(file_name);

        let installer_info = InstallerInfo {
            url: Some("1".to_string()),
            installer_token: "2".to_string(),
            app_name: "3".to_string(),
            app_version: "4".to_string(),
            app_file_name: "5".to_string(),
            app_run_port: 6_u32,
            jdk_name: "7".to_string(),
            jdk_version: "8".to_string(),
            jdk_file_name: "9".to_string(),
        };
        installer_config.add(installer_info);

        assert_eq!(1, installer_config.get_data().installers.len());

        // 判断文件中的内容
        let mut file = File::open(file_name)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mac_address = net::get_interface_address().unwrap().mac_address;
        let server_token_part = &format!("server_token = \"{}\"", mac_address);
        assert!(content.contains(server_token_part));
        assert!(content.contains("[[installers]]"));
        assert!(content.contains("url = \"1\""));
        assert!(content.contains("installer_token = \"2\""));
        assert!(content.contains("app_name = \"3\""));
        assert!(content.contains("app_version = \"4\""));
        assert!(content.contains("app_file_name = \"5\""));
        assert!(content.contains("app_run_port = 6"));
        assert!(content.contains("jdk_name = \"7\""));
        assert!(content.contains("jdk_version = \"8\""));
        assert!(content.contains("jdk_file_name = \"9\""));
        
        // 删除 installer_config.toml 文件
        fs::remove_file(file_name)?;
        Ok(())
    }

    #[test]
    fn update_a_installer_success() -> Result<(), Box<std::error::Error>> {
        let file_name = "update_a_installer_success.toml";
        let mut installer_config = InstallerConfig::from(file_name);

        let installer_info = InstallerInfo {
            url: Some("1".to_string()),
            installer_token: "2".to_string(),
            app_name: "3".to_string(),
            app_version: "4".to_string(),
            app_file_name: "5".to_string(),
            app_run_port: 6_u32,
            jdk_name: "7".to_string(),
            jdk_version: "8".to_string(),
            jdk_file_name: "9".to_string(),
        };
        installer_config.add(installer_info);

        let updated_installer_info = InstallerInfo {
            url: Some("11".to_string()),
            installer_token: "22".to_string(),
            app_name: "33".to_string(),
            app_version: "44".to_string(),
            app_file_name: "55".to_string(),
            app_run_port: 6_u32,
            jdk_name: "77".to_string(),
            jdk_version: "88".to_string(),
            jdk_file_name: "99".to_string(),
        };
        installer_config.update(6, updated_installer_info);

        // 判断文件中的内容
        let mut file = File::open(file_name)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mac_address = net::get_interface_address().unwrap().mac_address;
        let server_token_part = &format!("server_token = \"{}\"", mac_address);
        assert!(content.contains(server_token_part));
        assert!(content.contains("[[installers]]"));

        assert!(!content.contains("url = \"1\""));
        assert!(!content.contains("installer_token = \"2\""));
        assert!(!content.contains("app_name = \"3\""));
        assert!(!content.contains("app_version = \"4\""));
        assert!(!content.contains("app_file_name = \"5\""));
        assert!(!content.contains("jdk_name = \"7\""));
        assert!(!content.contains("jdk_version = \"8\""));
        assert!(!content.contains("jdk_file_name = \"9\""));

        assert!(content.contains("url = \"11\""));
        assert!(content.contains("installer_token = \"22\""));
        assert!(content.contains("app_name = \"33\""));
        assert!(content.contains("app_version = \"44\""));
        assert!(content.contains("app_file_name = \"55\""));
        assert!(content.contains("app_run_port = 6"));
        assert!(content.contains("jdk_name = \"77\""));
        assert!(content.contains("jdk_version = \"88\""));
        assert!(content.contains("jdk_file_name = \"99\""));
        
        // 删除 installer_config.toml 文件
        fs::remove_file(file_name)?;
        Ok(())
    }

    #[test]
    fn get_by_port_not_exist() -> Result<(), Box<std::error::Error>> {
        let file_name = "get_by_port_not_exist.toml";
        let installer_config = InstallerConfig::from(file_name);

        assert_eq!(None, installer_config.get_by_port(8080));

        // 删除 installer_config.toml 文件
        fs::remove_file(file_name)?;

        Ok(())
    }

    #[test]
    fn get_by_port_one_installer_success() -> Result<(), Box<std::error::Error>> {
        let file_name = "get_by_port_one_installer_success.toml";
        let mut installer_config = InstallerConfig::from(file_name);

        let installer_info = InstallerInfo {
            url: Some("1".to_string()),
            installer_token: "2".to_string(),
            app_name: "3".to_string(),
            app_version: "4".to_string(),
            app_file_name: "5".to_string(),
            app_run_port: 6_u32,
            jdk_name: "7".to_string(),
            jdk_version: "8".to_string(),
            jdk_file_name: "9".to_string(),
        };
        installer_config.add(installer_info);

        assert_eq!("1", installer_config.get_by_port(6).unwrap().url);

        // 删除 installer_config.toml 文件
        fs::remove_file(file_name)?;
        
        Ok(())
    }

    #[test]
    fn get_by_port_two_installer_success() -> Result<(), Box<std::error::Error>> {
        let file_name = "get_by_port_two_installer_success.toml";
        let mut installer_config = InstallerConfig::from(file_name);

        let installer_info = InstallerInfo {
            url: Some("1".to_string()),
            installer_token: "2".to_string(),
            app_name: "3".to_string(),
            app_version: "4".to_string(),
            app_file_name: "5".to_string(),
            app_run_port: 6_u32,
            jdk_name: "7".to_string(),
            jdk_version: "8".to_string(),
            jdk_file_name: "9".to_string(),
        };
        installer_config.add(installer_info);

        let installer_info = InstallerInfo {
            url: Some("11".to_string()),
            installer_token: "22".to_string(),
            app_name: "33".to_string(),
            app_version: "44".to_string(),
            app_file_name: "55".to_string(),
            app_run_port: 66_u32,
            jdk_name: "77".to_string(),
            jdk_version: "88".to_string(),
            jdk_file_name: "99".to_string(),
        };
        installer_config.add(installer_info);

        assert_eq!("11", installer_config.get_by_port(66).unwrap().url);

        // 删除 installer_config.toml 文件
        fs::remove_file(file_name)?;
        
        Ok(())
    }

    #[test]
    fn remove_by_installer_token_not_exist() -> Result<(), Box<std::error::Error>> {
        let file_name = "remove_by_installer_token_not_exist.toml";
        let mut installer_config = InstallerConfig::from(file_name);

        assert_eq!(0, installer_config.get_data().installers.len());
        installer_config.remove_by_installer_token("not-exist-installer-token");
        assert_eq!(0, installer_config.get_data().installers.len());
        // 删除 installer_config.toml 文件
        fs::remove_file(file_name)?;

        Ok(())
    }

    #[test]
    fn remove_all_success() -> Result<(), Box<std::error::Error>> {
        let file_name = "remove_all_success.toml";
        let mut installer_config = InstallerConfig::from(file_name);

        let installer_info = InstallerInfo {
            url: Some("1".to_string()),
            installer_token: "2".to_string(),
            app_name: "3".to_string(),
            app_version: "4".to_string(),
            app_file_name: "5".to_string(),
            app_run_port: 6_u32,
            jdk_name: "7".to_string(),
            jdk_version: "8".to_string(),
            jdk_file_name: "9".to_string(),
        };
        installer_config.add(installer_info);

        let installer_info = InstallerInfo {
            url: Some("11".to_string()),
            installer_token: "22".to_string(),
            app_name: "33".to_string(),
            app_version: "44".to_string(),
            app_file_name: "55".to_string(),
            app_run_port: 66_u32,
            jdk_name: "77".to_string(),
            jdk_version: "88".to_string(),
            jdk_file_name: "99".to_string(),
        };
        installer_config.add(installer_info);

        assert_eq!(2, installer_config.get_data().installers.len());

        installer_config.remove_all(|_| {
            true
        });

        assert_eq!(0, installer_config.get_data().installers.len());

        // 判断文件中的内容
        let mut file = File::open(file_name)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mac_address = net::get_interface_address().unwrap().mac_address;
        let server_token_part = &format!("server_token = \"{}\"", mac_address);
        assert!(content.contains(server_token_part));
        assert!(!content.contains("[[installers]]"));

        // 删除 installer_config.toml 文件
        fs::remove_file(file_name)?;
        
        Ok(())
    }

}
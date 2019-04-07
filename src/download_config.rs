use crate::config::DOWNLOAD_CONFIG_FILE_NAME;
use std::fs::File;
use std::io::prelude::*;
use serde_derive::{Deserialize, Serialize};
use toml;

/// 记录断点续传的配置信息
pub struct DownloadConfig {
    file_name: String,
    data: DownloadData,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DownloadData {
    pub files: Vec<FileMd5Info>,
}

/// 记录文件的 MD5 信息
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct FileMd5Info {
    pub name: String,
    pub version: String,
    pub md5: String,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadConfig {
    
    pub fn new() -> Self {
        Self::from(DOWNLOAD_CONFIG_FILE_NAME)
    }

    pub fn from(file_name: &str) -> Self {
        File::open(file_name).map(|mut file| {
            // 如果文件存在，则读取文件内容
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap_or(0);
            content
        }).map(|content| {
            toml::from_str::<DownloadData>(&content).unwrap_or_else(|_| {
                // 不是预期的 toml 格式，则创建默认设置
                Self::create_default_config(file_name).data
            })
        }).map(|data| {
                DownloadConfig {
                file_name: file_name.to_string(),
                data,
            }
        }).unwrap_or_else(|_|{
            // 如果文件不存在，则创建一个文件，并返回文件的内容
            Self::create_default_config(file_name)
        })
    }

    pub fn put(&mut self, app_name: &str, app_version:  &str, md5_value:  &str) {
        let file_md5_info = FileMd5Info {
            name: app_name.to_string(),
            version: app_version.to_string(),
            md5: md5_value.to_string(),
        };

        let files = &mut self.data.files;
        match files.iter().position(|file| file.name == app_name && file.version == app_version) {
            None => {},
            Some(index) => {
                files.remove(index);
            }
        };

        files.push(file_md5_info);

        self.save();
    }

    pub fn get(self, app_name: &str, app_version: &str) -> Option<FileMd5Info> {
        let files = self.data.files;

        files.into_iter().find(|file| file.name == app_name && file.version == app_version)
    }

    pub fn remove(&mut self, app_name: &str, app_version: &str) {
        let files = &mut self.data.files;
        match files.iter().position(|file| file.name == app_name && file.version == app_version) {
            None => {},
            Some(index) => {
                files.remove(index);
            }
        }

        self.save();
    }

    /// 创建一个默认的配置
    fn create_default_config(file_name: &str) -> Self {
        let data = DownloadData {
            files: Vec::<FileMd5Info>::new(),
        };
        let config = DownloadConfig {
            file_name: file_name.to_string(),
            data,
        };

        config.save();

        config
    }

    fn save(&self) {
        let toml_content = toml::to_vec(&self.data).unwrap();

        let mut file = File::create(&self.file_name).expect("failed to create download_config.toml file");
        file.write_all(toml_content.as_slice()).expect("failed to save download_config.toml content");
    }
}

#[cfg(test)]
mod tests {

    use std::path::Path;
    use std::fs::{self, File};
    use std::io::prelude::*;
    use super::{DownloadConfig, DownloadData, FileMd5Info};

    #[test]
    fn from_download_config_file_not_exist() -> Result<(), Box<std::error::Error>>  {
        let file_name = "from_download_config_file_not_exist.toml";

        assert!(!Path::new(file_name).exists());

        let download_config = DownloadConfig::from(file_name);
        assert_eq!(file_name, download_config.file_name);
        assert_eq!(DownloadData {
            files: Vec::<FileMd5Info>::new(),
        }, download_config.data);
        
        assert!(Path::new(file_name).exists());

        // 判断文件中的内容
        let mut file = File::open(file_name)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        assert!(content.contains("files = []"));
        assert!(!content.contains("[[files]]"));

        // 测试完成后，删除文件
        fs::remove_file(file_name)?;
        Ok(())
    }

    #[test]
    fn from_config_file_exist_but_content_is_empty() -> Result<(), Box<std::error::Error>>  {
        let file_name = "from_download_config_file_exist_but_content_is_empty.toml";
        File::create(file_name)?;

        assert!(Path::new(file_name).exists());

        let download_config = DownloadConfig::from(file_name);
        assert_eq!(file_name, download_config.file_name);
        assert_eq!(DownloadData {
            files: Vec::<FileMd5Info>::new(),
        }, download_config.data);

        assert!(Path::new(file_name).exists());

        // 判断文件中的内容
        let mut file = File::open(file_name)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        assert!(content.contains("files = []"));
        assert!(!content.contains("[[files]]"));

        // 删除 download_config.toml 文件
        fs::remove_file(file_name)?;
        Ok(())
    }

    #[test]
    fn from_config_file_exist_has_empty_files() -> Result<(), Box<std::error::Error>>  {
        let file_name = "from_download_config_file_exist_has_server_token_and_empty_installer.toml";
        // 注意，installers 的值为空时，toml 是按数组解析的
        let toml_content = r#"
        files = []
        "#;
        let mut file = File::create(file_name)?;
        file.write_all(toml_content.as_bytes()).expect("failed to save config.toml content");

        assert!(Path::new(file_name).exists());

        let download_config = DownloadConfig::from(file_name);
        assert_eq!(file_name, download_config.file_name);
        assert_eq!(DownloadData {
            files: Vec::<FileMd5Info>::new(),
        }, download_config.data);

        assert!(Path::new(file_name).exists());

        // 判断文件中的内容
        let mut file = File::open(file_name)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        assert!(content.contains("files = []"));
        assert!(!content.contains("[[files]]"));

        // 删除 config.toml 文件
        fs::remove_file(file_name)?;
        Ok(())
    }

    #[test]
    fn from_config_file_exist_has_invalid_toml() -> Result<(), Box<std::error::Error>>  {
        let file_name = "from_download_config_file_exist_has_invalid_toml.toml";
        // 注意，installers 的值为空时，toml 是按数组解析的
        let toml_content = r#"
        [[files]]
        a = "a"
        b = "b"
        "#;
        let mut file = File::create(file_name)?;
        file.write_all(toml_content.as_bytes()).expect("failed to save download_config.toml content");

        assert!(Path::new(file_name).exists());

        let download_config = DownloadConfig::from(file_name);
        assert_eq!(file_name, download_config.file_name);
        assert_eq!(DownloadData {
            files: Vec::<FileMd5Info>::new(),
        }, download_config.data);

        assert!(Path::new(file_name).exists());

        // 判断文件中的内容
        let mut file = File::open(file_name)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        assert!(content.contains("files = []"));
        assert!(!content.contains("[[files]]"));

        // 删除 download_config.toml 文件
        fs::remove_file(file_name)?;
        Ok(())
    }

    #[test]
    fn put_one_file_success() -> Result<(), Box<std::error::Error>> {
        let config_file_name = "put_one_file_success.toml";
        let mut download_config = DownloadConfig::from(config_file_name);

        download_config.put("app_name", "app_version", "md5_value");

        // 断言存在 toml 文件
        assert!(Path::new(config_file_name).exists());
        // 读取文件中的内容，并比较部分内容
        let mut file = File::open(config_file_name)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        assert!(buffer.contains("[[files]]"));
        assert!(buffer.contains(r#"name = "app_name""#));
        assert!(buffer.contains(r#"version = "app_version""#));
        assert!(buffer.contains(r#"md5 = "md5_value""#));

        // 删除 download_config.toml 文件
        fs::remove_file(config_file_name)?;

        Ok(())
    }

    #[test]
    fn put_one_if_exists_then_override() -> Result<(), Box<std::error::Error>> {
        let config_file_name = "put_one_if_exists_then_override.toml";
        let mut download_config = DownloadConfig::from(config_file_name);

        download_config.put("app_name", "app_version", "md5_value");
        download_config.put("app_name", "app_version", "md5_value_1");

        // 读取文件中的内容，并比较部分内容
        let mut file = File::open(config_file_name)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        assert!(buffer.contains("[[files]]"));
        assert!(buffer.contains(r#"name = "app_name""#));
        assert!(buffer.contains(r#"version = "app_version""#));
        assert!(!buffer.contains(r#"md5 = "md5_value""#));
        assert!(buffer.contains(r#"md5 = "md5_value_1""#));

        // 删除 download_config.toml 文件
        fs::remove_file(config_file_name)?;

        Ok(())
    }

    #[test]
    fn get_one_file_success() -> Result<(), Box<std::error::Error>> {
        let config_file_name = "get_one_file_success.toml";
        let content = br#"
            [[files]]
            name = "name_1"
            version = "version_1"
            md5 = "m5d_1"
        "#;
        let mut file = File::create(config_file_name).unwrap();
        file.write_all(content).unwrap();

        let download_config = DownloadConfig::from(config_file_name);
        let file_md5_info = download_config.get("name_1", "version_1").unwrap();
        assert_eq!("m5d_1", file_md5_info.md5);

        // 删除 download_config.toml 文件
        fs::remove_file(config_file_name)?;
        Ok(())
    }

    #[test]
    fn remove_one_file_success() -> Result<(), Box<std::error::Error>> {
        let config_file_name = "remove_one_file_success.toml";
        let content = br#"
            [[files]]
            name = "name_1"
            version = "version_1"
            md5 = "m5d_1"
        "#;
        let mut file = File::create(config_file_name).unwrap();
        file.write_all(content).unwrap();

        let mut download_config = DownloadConfig::from(config_file_name);
        download_config.remove("name_1", "version_1");
        
        let mut file = File::open(config_file_name)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        assert!(content.contains("files = []"));
        assert!(!content.contains("[[files]]"));

        // 删除 download_config.toml 文件
        fs::remove_file(config_file_name)?;
        Ok(())
    }
}
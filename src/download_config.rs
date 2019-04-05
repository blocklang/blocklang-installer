use std::fs::File;
use std::io::prelude::*;
use serde_derive::{Deserialize, Serialize};
use toml;

const CONFIG_FILE_NAME: &str = "download_config.toml";

/// 记录断点续传的配置信息
#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadConfig {
    pub files: Vec<FileMd5Info>,
}

/// 记录文件的 MD5 信息
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct FileMd5Info {
    pub name: String,
    pub version: String,
    pub md5: String,
}

pub fn put(app_name: &str, app_version:  &str, md5_value:  &str) {
    put_to(CONFIG_FILE_NAME, app_name, app_version, md5_value);
}

pub fn get(app_name: &str, app_version: &str) -> Option<FileMd5Info> {
    get_from(CONFIG_FILE_NAME, app_name, app_version)
}

pub fn remove(app_name: &str, app_version: &str) {
    remove_from(CONFIG_FILE_NAME, app_name, app_version);
}

fn put_to(config_file_name: &str, app_name: &str, app_version:  &str, md5_value:  &str) {
     let mut files_config = match read_from(config_file_name) {
        Ok(config) => {config},
        Err(_) => {
            DownloadConfig {
                files: Vec::new()
            }
        }
    };

    let file_md5_info = FileMd5Info {
        name: app_name.to_string(),
        version: app_version.to_string(),
        md5: md5_value.to_string(),
    };

    let files = &mut files_config.files;
    match files.iter().position(|file| file.name == app_name && file.version == app_version) {
        None => {},
        Some(index) => {
            files.remove(index);
        }
    };

    files.push(file_md5_info);

    let toml_content = toml::to_vec(&files_config).unwrap();
    let mut file = File::create(config_file_name).expect("failed to create download_config.toml file");
    file.write_all(toml_content.as_slice()).expect("failed to save download_config.toml content");
}

fn get_from(config_file_name: &str, app_name: &str, app_version:  &str) -> Option<FileMd5Info> {
     match read_from(config_file_name) {
        Ok(config) => {
            let files = config.files;
            files.into_iter().find(|file| file.name == app_name && file.version == app_version)
        },
        Err(_) => {
            None
        }
    }
}

fn read_from(config_file_name: &str) -> Result<DownloadConfig, Box<std::error::Error>> {
    let mut file = File::open(config_file_name)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: DownloadConfig = toml::from_str(&contents)?;
    Ok(config)
}

fn remove_from(config_file_name: &str, app_name: &str, app_version:  &str) {
    match read_from(config_file_name) {
        Ok(mut files_config) => {
            let files = &mut files_config.files;
            match files.iter().position(|file| file.name == app_name && file.version == app_version) {
                None => {},
                Some(index) => {
                    files.remove(index);
                }
            }

            let toml_content = if !files.is_empty() {
                toml::to_vec(&files_config).unwrap()
            } else {
                vec![]
            };

            let mut file = File::create(config_file_name).expect("failed to create download_config.toml file");
            file.write_all(toml_content.as_slice()).expect("failed to save download_config.toml content");
            
        },
        Err(_) => {
        }
    }
}

#[cfg(test)]
mod tests {

    use std::path::Path;
    use std::fs::{self, File};
    use std::io::prelude::*;
    use super::{put_to, get_from, remove_from};

    #[test]
    fn put_one_file_success() -> Result<(), Box<std::error::Error>> {
        let config_file_name = "download_config_1.toml";
        put_to(config_file_name, "app_name", "app_version", "md5_value");

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

        // 删除 config.toml 文件
        fs::remove_file(config_file_name)?;

        Ok(())
    }

    #[test]
    fn put_one_if_exists_then_override() -> Result<(), Box<std::error::Error>> {
        let config_file_name = "download_config_2.toml";
        put_to(config_file_name, "app_name", "app_version", "md5_value");
        put_to(config_file_name, "app_name", "app_version", "md5_value_1");

        // 读取文件中的内容，并比较部分内容
        let mut file = File::open(config_file_name)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        assert!(buffer.contains("[[files]]"));
        assert!(buffer.contains(r#"name = "app_name""#));
        assert!(buffer.contains(r#"version = "app_version""#));
        assert!(!buffer.contains(r#"md5 = "md5_value""#));
        assert!(buffer.contains(r#"md5 = "md5_value_1""#));

        // 删除 config.toml 文件
        fs::remove_file(config_file_name)?;

        Ok(())
    }

    #[test]
    fn get_one_file_success() -> Result<(), Box<std::error::Error>> {
        let config_file_name = "download_config_3.toml";
        let content = br#"
            [[files]]
            name = "name_1"
            version = "version_1"
            md5 = "m5d_1"
        "#;
        let mut file = File::create(config_file_name).unwrap();
        file.write_all(content).unwrap();

        let file_md5_info = get_from(config_file_name, "name_1", "version_1").unwrap();
        assert_eq!("m5d_1", file_md5_info.md5);

        // 删除 config.toml 文件
        fs::remove_file(config_file_name)?;
        Ok(())
    }

    #[test]
    fn remove_one_file_success() -> Result<(), Box<std::error::Error>> {
        let config_file_name = "download_config_4.toml";
        let content = br#"
            [[files]]
            name = "name_1"
            version = "version_1"
            md5 = "m5d_1"
        "#;
        let mut file = File::create(config_file_name).unwrap();
        file.write_all(content).unwrap();

        remove_from(config_file_name, "name_1", "version_1");
        
        let mut file = File::open(config_file_name)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        assert!(buffer.is_empty());

        // 删除 config.toml 文件
        fs::remove_file(config_file_name)?;
        Ok(())
    }
}
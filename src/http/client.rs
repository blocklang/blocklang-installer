use std::fs::{self, File};
use std::path::Path;
use std::collections::HashMap;
use reqwest::{self, Method};
use serde_derive::{Deserialize};

use crate::util::{net, os};
use crate::config;

/// 软件安装信息
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallerInfo {
    pub url: String,
    pub token: String,
    pub software_name: String,
    pub software_version: String,
    pub software_file_name: String,
    pub software_run_port: u32,
    pub jdk_name: String,
    pub jdk_version: String,
    pub jdk_file_name: String,
}

/// 使用 Block Lang 提供的部署 token，向 Block Lang 平台注册部署服务器信息。
/// 
/// Block Lang 和部署服务器之间是通过 token 建立连接的。
/// 
/// 注意：连接建立后，Block Lang 平台默认打开连接，但是如果遇到盗用 token 的情况，
/// 可以在 Block Lang 平台关闭该连接。
pub fn register_installer(url: &str, token: &str) -> Result<InstallerInfo, Box<std::error::Error>> {
    request_installers(Method::POST, url, token)
}

/// 使用 Block Lang 提供的部署 token，向 Block Lang 平台更新部署服务器信息，并获取软件的最新发布信息。
/// 
/// Block Lang 和部署服务器之间是通过 token 建立连接的。
/// 
/// 注意：连接建立后，Block Lang 平台默认打开连接，但是如果遇到盗用 token 的情况，
/// 可以在 Block Lang 平台关闭该连接。
pub fn update_installer(url: &str, token: &str) -> Result<InstallerInfo, Box<std::error::Error>> {
    request_installers(Method::PUT, url, token)
}

fn request_installers(
    http_method: Method, 
    url: &str, 
    token: &str) -> Result<InstallerInfo, Box<std::error::Error>> {
    let url = &format!("{}/installers", url);

    let mut json_data = HashMap::new();
    let interface_addr = net::get_interface_address().expect("获取不到能联网的有线网络");
    
    json_data.insert("token", token);
    json_data.insert("serverToken", &interface_addr.mac_address);
    json_data.insert("ip", &interface_addr.ip_address);
    // TODO: 设置以下参数
    // json_data.insert("port", "");
    // json_data.insert("platform_name", "");
    // json_data.insert("platform_version", "");
    // json_data.insert("architecture", "");
    // println!("{:?}", json_data);

    let client = reqwest::Client::new();
    let mut response = client.request(http_method, url)
        .json(&json_data)
        .send()?;

    let result: InstallerInfo = response.json()?;

    Ok(result)
}

/// 从软件中心下载软件。
/// 
/// `download` 函数将根据 `software_name` 指定的软件名，
/// `software_version` 指定的软件版本号，从软件发布中心下载软件。
/// 然后将下载的软件存到应用服务器指定的目录中，并将文件名设置为 `software_file_name`。
/// 
/// 如果在指定的文件夹下找到对应的文件，则中断下载，直接使用已存在文件。
/// 
/// 下载完成后，会返回新下载文件的完整路径。
/// 
/// 应用服务器的目录结构为
/// 
/// * softwares
///     * software_name
///         * software_version
///             * software_file_name
/// 
/// # Examples
/// 
/// ```no_run
/// use installer::http::client::download;
/// 
/// fn main() {
///     download("app", "0.1.0", "app-0.1.0.zip").unwrap();
/// }
/// ```
pub fn download(software_name: &str, 
    software_version: &str, 
    software_file_name: &str) -> Option<String> {
    
    let saved_dir_path = &format!("{}/{}/{}", 
        config::ROOT_PATH_SOFTWARE, 
        software_name, 
        software_version);

    fs::create_dir_all(saved_dir_path).expect("在创建存储下载文件的目录结构时出错");

    let saved_file_path = &format!("{}/{}", saved_dir_path, software_file_name);

    let path = Path::new(saved_file_path);
    // 如果文件已存在，则直接返回文件名
    if path.exists() {
        return Some(saved_file_path.to_string());
    }

    println!("开始下载文件：{}", software_file_name);

    let os = os::get_target_os().expect("不支持的操作系统");
    let url = &format!("{}/softwares?name={}&version={}&os={}", 
        config::URL, 
        software_name, 
        software_version,
        os);

    let client = reqwest::Client::new();
    match client.get(url).send() {
        Err(e) => {
            println!("下载失败，出现了其他错误，状态码为：{:?}", e);
            None
        },
        Ok(mut response) => {
            if !response.status().is_success() {
                println!("下载失败，{:?}", response.status());
                return None;
            }

            println!("返回成功，开始在本地写入文件");
            let mut file = File::create(saved_file_path).unwrap();
            response.copy_to(&mut file).unwrap();
            println!("下载完成。");
            Some(saved_file_path.to_string())
        }
    }
}


#[cfg(test)]
mod tests {

    use std::path::Path;
    use std::fs;
    use std::io::prelude::*;
    use mockito::mock;
    use tempfile::NamedTempFile;
    use crate::config;
    use super::{register_installer, 
                download};

    #[test]
    fn register_installer_success() -> Result<(), Box<std::error::Error>> {
        // 模拟一个 installers POST 服务
        let mock = mock("POST", "/installers")
            .with_body(r#"{
                            "url": "1",
                            "token": "2", 
                            "softwareName": "3", 
                            "softwareVersion": "4",
                            "softwareFileName": "5",
                            "softwareRunPort": 6,
                            "jdkName": "7", 
                            "jdkVersion": "8",
                            "jdkFileName": "9"
                        }"#)
            .with_status(201)
            .create();

        // 请求 installers 服务
        let installer_info = register_installer(config::URL, "1")?;
        println!("{:#?}", installer_info);
        // 断言返回的结果
        assert_eq!("1", installer_info.url);
        assert_eq!("2", installer_info.token);
        assert_eq!("3", installer_info.software_name);
        assert_eq!("4", installer_info.software_version);
        assert_eq!("5", installer_info.software_file_name);
        assert_eq!(6, installer_info.software_run_port);
        assert_eq!("7", installer_info.jdk_name);
        assert_eq!("8", installer_info.jdk_version);
        assert_eq!("9", installer_info.jdk_file_name);

        // 断言已执行过 mock 的 http 服务
        mock.assert();

        Ok(())
    }

    #[test]
    fn download_fail() {
        assert_eq!(None, download("app", "0.1.0", "app-0.1.0.zip"));
    }

    #[test]
    fn download_success() -> Result<(), Box<std::error::Error>> {
        // 创建一个临时文件，当作下载文件
        let mut file = NamedTempFile::new()?;
        writeln!(file, "I am a software!")?;
        let path = file.path();
        let path = path.to_str().unwrap();

        // mock 下载文件的 http 服务
        let mock = mock("GET", "/softwares?name=app&version=0.1.1&os=windows")
            .with_body_from_file(path)
            .with_status(200)
            .create();
        
        {
            // 执行下载文件方法
            let downloaded_file_path = download("app", "0.1.1", "app-0.1.1.zip").unwrap();

            // 断言文件已下载成功
            assert!(Path::new(&downloaded_file_path).exists());

            // 删除已下载的文件
            fs::remove_dir_all(config::ROOT_PATH_SOFTWARE)?;
        }

        // 断言已执行过 mock 的 http 服务
        mock.assert();

        Ok(())
    }
}
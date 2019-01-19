use std::fs::{self, File};
use std::path::Path;
use std::collections::HashMap;
use reqwest::{self, Method, StatusCode};
use serde_derive::{Deserialize};

use crate::util::{net, os};
use crate::config;

/// 软件安装信息
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallerInfo {
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

/// 使用 Block Lang 提供的项目注册 token，向 Block Lang 平台注册部署服务器信息。
/// `server_token` 用于标识一台服务器，支持在一台应用服务器上注册多个 installer。
/// 
/// Block Lang 和部署服务器之间是通过 token 建立连接的。
/// 
/// 注意：连接建立后，Block Lang 平台默认打开连接，但是如果遇到盗用 token 的情况，
/// 可以在 Block Lang 平台关闭该连接。
pub fn register_installer(
    url: &str, 
    registration_token: &str, 
    app_run_port: u32,  
    server_token: &str) -> Result<InstallerInfo, Box<std::error::Error>> {

    let url = &format!("{}/installers", url);

    let mut json_data = HashMap::new();
    let interface_addr = net::get_interface_address().expect("获取不到能联网的有线网络");
    let os_info = os::get_os_info();

    let app_run_port = app_run_port.to_string();
    
    json_data.insert("token", registration_token);
    json_data.insert("serverToken", server_token);
    json_data.insert("ip", &interface_addr.ip_address);
    json_data.insert("appRunPort", &app_run_port);
    json_data.insert("osType", &os_info.os_type);
    json_data.insert("osVersion", &os_info.version);
    json_data.insert("arch", &os_info.target_arch);

    let client = reqwest::Client::new();
    let mut response = client.request(Method::POST, url)
        .json(&json_data)
        .send()?;

    let result: InstallerInfo = response.json()?;

    Ok(result)
}

/// 向 Block Lang 平台注销指定的 installer
pub fn unregister_installer(url: &str, installer_token: &str) -> Result<(), Box<std::error::Error>> {
    let url = &format!("{}/installers/{}", url, installer_token);
    let client = reqwest::Client::new();
    client.request(Method::DELETE, url).send()?;
    Ok(())
}

/// 使用 Block Lang 提供的部署 token，向 Block Lang 平台更新部署服务器信息，并获取软件的最新发布信息。
/// 
/// Block Lang 和部署服务器之间是通过 token 建立连接的。
/// 
/// 注意：连接建立后，Block Lang 平台默认打开连接，但是如果遇到盗用 token 的情况，
/// 可以在 Block Lang 平台关闭该连接。
/// TODO: 不能再调用同一个方法，待修复，需要重新设计 update
pub fn update_installer(url: &str, token: &str) -> Result<InstallerInfo, Box<std::error::Error>> {
    request_installers(Method::PUT, url, token)
}
// TODO:xxxx
fn request_installers(
    http_method: Method, 
    url: &str, 
    token: &str) -> Result<InstallerInfo, Box<std::error::Error>> {
    let url = &format!("{}/installers", url);

    let mut json_data = HashMap::new();
    let interface_addr = net::get_interface_address().expect("获取不到能联网的有线网络");
    let os_info = os::get_os_info();
    
    json_data.insert("token", token);
    json_data.insert("serverToken", &interface_addr.mac_address);
    json_data.insert("ip", &interface_addr.ip_address);
    json_data.insert("os_type", &os_info.os_type);
    json_data.insert("os_version", &os_info.version);
    json_data.insert("arch", &os_info.target_arch);
    // TODO: 设置以下参数
    // json_data.insert("port", ""); // port 指在部署服务器上运行的服务，当前未开发此功能。
    
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
/// `download` 函数将根据 `app_name` 指定的软件名，
/// `app_version` 指定的软件版本号，从软件发布中心下载软件。
/// 然后将下载的软件存到应用服务器指定的目录中，并将文件名设置为 `app_file_name`。
/// 
/// 如果在指定的文件夹下找到对应的文件，则中断下载，直接使用已存在文件。
/// 
/// 下载完成后，会返回新下载文件的完整路径。
/// 
/// 应用服务器的目录结构为
/// 
/// * apps
///     * app_name
///         * app_version
///             * app_file_name
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
pub fn download(app_name: &str, 
    app_version: &str, 
    app_file_name: &str) -> Option<String> {
    
    let saved_dir_path = &format!("{}/{}/{}", 
        config::ROOT_PATH_APP, 
        app_name, 
        app_version);

    fs::create_dir_all(saved_dir_path).expect("在创建存储下载文件的目录结构时出错");

    let saved_file_path = &format!("{}/{}", saved_dir_path, app_file_name);

    let path = Path::new(saved_file_path);
    // 如果文件已存在，则直接返回文件名
    if path.exists() {
        return Some(saved_file_path.to_string());
    }

    println!("开始下载文件：{}", app_file_name);

    let os_info = os::get_os_info();

    println!("服务器信息：{:?}", os_info);

    let url = &format!("{}/apps?appName={}&version={}&targetOs={}&arch={}", 
        config::URL, 
        app_name, 
        app_version,
        os_info.target_os,
        os_info.target_arch);

    let client = reqwest::Client::new();
    match client.get(url).send() {
        Err(e) => {
            println!("下载失败，出现了其他错误，状态码为：{:?}", e);
            None
        },
        Ok(mut response) => {
            match response.status() {
                StatusCode::OK => {
                    println!("返回成功，开始在本地写入文件");
                    let mut file = File::create(saved_file_path).unwrap();
                    response.copy_to(&mut file).unwrap();
                    println!("下载完成。");
                    Some(saved_file_path.to_string())
                }
                StatusCode::NOT_FOUND => {
                    println!("下载失败，没有找到要下载的文件 {}", 404);
                    println!("下载地址为 {}", response.url().as_str());
                    None
                }
                s => {
                    println!("下载失败，状态码为 {:?}", s);
                    println!("下载地址为 {}", response.url().as_str());
                    return None;
                }
            }
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
    use crate::util::os;
    use super::{register_installer,
                unregister_installer, 
                download};

    #[test]
    fn register_installer_success() -> Result<(), Box<std::error::Error>> {
        // 模拟一个 installers POST 服务
        let mock = mock("POST", "/installers")
            .with_body(r#"{
                            "url": "1",
                            "installerToken": "2", 
                            "appName": "3", 
                            "appVersion": "4",
                            "appFileName": "5",
                            "appRunPort": 6,
                            "jdkName": "7", 
                            "jdkVersion": "8",
                            "jdkFileName": "9"
                        }"#)
            .with_status(201)
            .create();

        // 请求 installers 服务
        let installer_info = register_installer(config::URL, "1", 80, "server_1")?;
        println!("{:#?}", installer_info);
        // 断言返回的结果
        assert_eq!("1", installer_info.url);
        assert_eq!("2", installer_info.installer_token);
        assert_eq!("3", installer_info.app_name);
        assert_eq!("4", installer_info.app_version);
        assert_eq!("5", installer_info.app_file_name);
        assert_eq!(6, installer_info.app_run_port);
        assert_eq!("7", installer_info.jdk_name);
        assert_eq!("8", installer_info.jdk_version);
        assert_eq!("9", installer_info.jdk_file_name);

        // 断言已执行过 mock 的 http 服务
        mock.assert();

        Ok(())
    }

    #[test]
    fn unregister_installer_success() -> Result<(), Box<std::error::Error>> {
        let installer_token = "1";
        // 模拟一个 installers DELETE 服务
        let url = format!("/installers/{}", installer_token);
        let mock = mock("DELETE", &*url)
            .with_status(204)
            .create();

        // 请求 installers 的注销服务
        unregister_installer(config::URL, installer_token)?;

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
        writeln!(file, "I am a app!")?;
        let path = file.path();
        let path = path.to_str().unwrap();

        // mock 下载文件的 http 服务
        let os_info = os::get_os_info();
        let url = format!("/apps?appName=app&version=0.1.1&targetOs={}&arch={}", 
            os_info.target_os,
            os_info.target_arch);

        let mock = mock("GET", &*url) // FIXME: 为什么 &url 不起作用？
            .with_body_from_file(path)
            .with_status(200)
            .create();
        
        {
            // 执行下载文件方法
            let downloaded_file_path = download("app", "0.1.1", "app-0.1.1.zip").unwrap();

            // 断言文件已下载成功
            assert!(Path::new(&downloaded_file_path).exists());

            // 删除已下载的文件
            fs::remove_dir_all(config::ROOT_PATH_APP)?;
        }

        // 断言已执行过 mock 的 http 服务
        mock.assert();

        Ok(())
    }
}
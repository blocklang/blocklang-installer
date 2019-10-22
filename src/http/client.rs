use std::fs::{self, File};
use std::path::Path;
use std::io::{self, copy, Read};
use std::collections::HashMap;
use std::time::Instant;
use reqwest::{Client, StatusCode};
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde_derive::{Deserialize};
use serde_json;
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};

use crate::util::{net, os};
use crate::config::{self, REST_API_INSTALLERS, REST_API_APPS};
use crate::download_config::DownloadConfig;


/// 先显示字段级错误，然后显示全局错误
fn print_errors(errors: serde_json::Value, mut writer: impl std::io::Write) {
    let error_map = errors["errors"].as_object().unwrap();
    let mut num = 0;
    for(key, value) in error_map.iter() {
        if key != "globalErrors" {
            for error_msg in value.as_array().unwrap().iter() {
                num += 1;
                writeln!(writer, "> [ERROR]: {}. {}", num, error_msg.as_str().unwrap()).unwrap();
            }
        }
    }

    // 最后打印 globalErrors
    if let Some(global_errors) = errors["errors"]["globalErrors"].as_array() {
        for error_msg in global_errors.iter() {
            num += 1;
            writeln!(writer, "> [ERROR]: {}. {}", num, error_msg.as_str().unwrap()).unwrap();
        }
    }
  
}

/// 软件安装信息
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallerInfo {
    pub url: Option<String>,
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
    root_url: &str, 
    registration_token: &str, 
    app_run_port: u32,  
    server_token: &str) -> Result<InstallerInfo, Box<dyn std::error::Error>> {

    let url = &format!("{}/{}", root_url, REST_API_INSTALLERS);
    
    let interface_addr = net::get_interface_address().expect("获取不到能联网的有线网络");
    let os_info = os::get_os_info();

    let app_run_port = app_run_port.to_string();
    
    let mut json_data = HashMap::new();
    json_data.insert("registrationToken", registration_token);
    json_data.insert("serverToken", server_token);
    json_data.insert("ip", &interface_addr.ip_address);
    json_data.insert("appRunPort", &app_run_port);
    json_data.insert("osType", &os_info.os_type);
    json_data.insert("osVersion", &os_info.version);
    json_data.insert("arch", &os_info.target_arch);
    json_data.insert("targetOs", &os_info.target_os);

    let client = Client::new();

    client.post(url)
        .json(&json_data)
        .send()
        .map_err(|err| {
            eprintln!("> [ERROR]: 无法访问 {}, 可能是 url 输入有误", url);
            Box::from(err)
        })
        .and_then(|mut response| {
            match response.status() {
                StatusCode::CREATED => {
                    match response.json::<InstallerInfo>() {
                        Ok(mut result) => {
                            result.url = Some(root_url.to_string());
                            Ok(result)
                        },
                        Err(e) => {
                            eprintln!("> [ERROR]: 从 {} 未能获取有效的安装器数据", url);
                            Err(Box::from(e))
                        }
                    }
                }
                StatusCode::UNPROCESSABLE_ENTITY => {
                    eprintln!("> [ERROR]: 往 Block Lang 平台注册主机失败!");
                    eprintln!("> [ERROR]: 请修复以下问题后再安装：");

                    match response.json::<serde_json::Value>() {
                        Ok(errors) => {
                            print_errors(errors, &mut std::io::stderr());
                        },
                        Err(_) => {
                            eprintln!("> [ERROR]: 从 {} 未能获取有效错误信息", url);
                        }
                    };

                    Err(Box::from("未通过数据有效性校验"))
                }
                s => {
                    eprintln!("> [ERROR]: 从 {} 未能获取有效数据, 可能是 url 输入有误", url);
                    Err(Box::from(format!("未知错误，状态码是 {:?}", s)))
                }
            }
        })
}

/// 向 Block Lang 平台注销指定的 installer
pub fn unregister_installer(root_url: &str, installer_token: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = &format!("{}/{}/{}", root_url, REST_API_INSTALLERS, installer_token);
    let client = Client::new();
    client.delete(url)
        .send()
        .map_err(|err| {
            eprintln!("> [ERROR]: 无法访问 {}", url);
            Box::from(err)
        })
        .and_then(|response| {
            match response.status() {
                StatusCode::NO_CONTENT => {
                    // 注销成功，不做任何操作
                    Ok(())
                },
                StatusCode::NOT_FOUND => {
                    println!("> [WARN]: 根据installer token 没有找到注册器信息");
                    Err(Box::from("根据installer token 没有找到注册器信息"))
                }
                s => {
                    eprintln!("> [ERROR]: 返回的状态码无效，url 为 {}, 状态码为：{}", url, s);
                    Err(Box::from(format!("未知错误，状态码是 {:?}", s)))
                }
            }
        })
}

/// 使用 Block Lang 提供的部署 token，向 Block Lang 平台更新部署服务器信息，并获取软件的最新发布信息。
/// 
/// Block Lang 和部署服务器之间是通过 token 建立连接的。
/// 
/// 注意：连接建立后，Block Lang 平台默认打开连接，但是如果遇到盗用 token 的情况，
/// 可以在 Block Lang 平台关闭该连接。
/// TODO: 不能再调用同一个方法，待修复，需要重新设计 update
pub fn update_installer(root_url: &str, token: &str) -> Result<InstallerInfo, Box<dyn std::error::Error>> {
    let url = &format!("{}/{}", root_url, REST_API_INSTALLERS);

    let mut json_data = HashMap::new();
    let interface_addr = net::get_interface_address().expect("获取不到能联网的有线网络");
    let os_info = os::get_os_info();
    
    json_data.insert("installerToken", token);
    json_data.insert("serverToken", &interface_addr.mac_address);
    json_data.insert("ip", &interface_addr.ip_address);
    json_data.insert("osType", &os_info.os_type);
    json_data.insert("osVersion", &os_info.version);
    json_data.insert("arch", &os_info.target_arch);
    json_data.insert("targetOs", &os_info.target_os);

    let client = Client::new();
    client.put(url)
        .json(&json_data)
        .send()
        .map_err(|err| {
            eprintln!("> [ERROR]: 无法访问 {}", url);
            Box::from(err)
        })
        .and_then(|mut response| {
            match response.status() {
                StatusCode::OK => {
                    match response.json::<InstallerInfo>() {
                        Ok(mut result) => {
                            result.url = Some(root_url.to_string());
                            Ok(result)
                        },
                        Err(e) => {
                            eprintln!("> [ERROR]: 从 {} 未能获取有效的安装器数据", url);
                            Err(Box::from(e))
                        }
                    }
                }
                StatusCode::UNPROCESSABLE_ENTITY => {
                    eprintln!("> [ERROR]: 往 Block Lang 平台获取项目最新信息失败!");
                    eprintln!("> [ERROR]: 请修复以下问题后再升级：");

                    match response.json::<serde_json::Value>() {
                        Ok(errors) => {
                            print_errors(errors, &mut std::io::stderr());
                        },
                        Err(_) => {
                            eprintln!("> [ERROR]: 从 {} 未能获取有效错误信息", url);
                        }
                    };

                    Err(Box::from("未通过数据有效性校验"))
                }
                s => {
                    eprintln!("> [ERROR]: 从 {} 未能获取有效数据, 可能是 url 输入有误", url);
                    Err(Box::from(format!("未知错误，状态码是 {:?}", s)))
                }
            }
        })
}

struct DownloadProgress<R> {
    inner: R,
    progress_bar: ProgressBar,
}

impl<R: Read> Read for DownloadProgress<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf).map(|n| {
            self.progress_bar.inc(n as u64);
            n
        })
    }
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
///     download("https://blocklang.com", "app", "0.1.0", "app-0.1.0.zip").unwrap();
/// }
/// ```
pub fn download(
    root_url: &str,
    app_name: &str, 
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
        println!("> 文件已存在");
        return Some(saved_file_path.to_string());
    }

    // 在下载过程中，将文件命名后面添加 .part
    let saved_file_part_name = &format!("{}.part", saved_file_path);
    let saved_file_part_path = Path::new(saved_file_part_name);
    let mut downloaded_size = 0;

    let mut headers = HeaderMap::new();

    if saved_file_part_path.exists() { // 已下载部分内容，进行断点续传
        downloaded_size = saved_file_part_path.metadata().unwrap().len();
        headers.insert(header::RANGE, HeaderValue::from_str(&format!("bytes={}-", downloaded_size)).unwrap());

        let download_config = DownloadConfig::new();
        if let Some(file_md5_info) = download_config.get(app_name, app_version) {
            headers.insert(header::IF_RANGE, HeaderValue::from_str(&file_md5_info.md5).unwrap());
        }
        
    } else {
        // 全新下载
    }

    let os_info = os::get_os_info();

    let url = &format!("{}/{}?appName={}&version={}&targetOs={}&arch={}", 
        root_url, 
        REST_API_APPS,
        app_name, 
        app_version,
        os_info.target_os,
        os_info.target_arch);

    let client = Client::new();
    match client.get(url).headers(headers).send() {
        Err(e) => {
            println!("> [ERROR]: 下载失败，出现了其他错误，状态码: {:?}", e);
            None
        },
        Ok(response) => {
            match response.status() {
                StatusCode::OK => {
                    // 只有开始下载时，才需要显示进度条
                    let total_size = response
                        .headers()
                        .get(header::CONTENT_LENGTH)
                        .and_then(|ct_len| ct_len.to_str().ok())
                        .and_then(|ct_len| ct_len.parse().ok())
                        .unwrap_or(0);

                    let etag = response
                        .headers()
                        .get(header::ETAG)
                        .and_then(|value| value.to_str().ok())
                        .unwrap_or("");

                    // 在开始下载前，缓存 etag 的值
                    if !etag.trim().is_empty() {
                        // 去掉外围的双引号
                        let mut download_config = DownloadConfig::new();
                        download_config.put(app_name, app_version, etag.trim().trim_matches('"'));
                    }

                    let pb = ProgressBar::new(total_size);

                    pb.set_style(ProgressStyle::default_bar()
                        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                        .progress_chars("=>-"));
                    
                    let mut source = DownloadProgress {
                        progress_bar: pb,
                        inner: response,
                    };

                    // 下载整个文件
                    // 如果文件已存在，说明文件被改动过，需删除之前下载过的文件，重新下载
                    // 直接使用 File::create 就可删除之前下载过的内容
                    let mut file = File::create(saved_file_part_path).unwrap();

                    let started = Instant::now();
                    copy(&mut source, &mut file).unwrap();
                    source.progress_bar.finish_and_clear();
                   
                    // 下载完成后，将文件名中的 .part 去掉
                    fs::rename(saved_file_part_path, saved_file_path).unwrap();

                    // 下载完成后，清除 download_config 配置项
                    let mut download_config = DownloadConfig::new();
                    download_config.remove(app_name, app_version);

                     println!("> [INFO]: 下载完成，耗时 {}", HumanDuration(started.elapsed()));

                    Some(saved_file_path.to_string())
                }
                StatusCode::PARTIAL_CONTENT => {
                    // 断点续传
                    // 只有开始下载时，才需要显示进度条
                    // let total_size = response
                    //     .headers()
                    //     .get(header::CONTENT_LENGTH)
                    //     .and_then(|ct_len| ct_len.to_str().ok())
                    //     .and_then(|ct_len| ct_len.parse().ok())
                    //     .unwrap_or(0);
                    // 当是断点续传时，CONTENT_LENGTH 中存的是剩余大小
                    // 需要从 CONTENT_RANGE 中获取总大小
                    let total_size = response
                        .headers()
                        .get(header::CONTENT_RANGE)
                        .and_then(|ct_range| ct_range.to_str().ok())
                        .and_then(|ct_range| ct_range.split('/').collect::<Vec<_>>()[1].parse::<u64>().ok())
                        .unwrap_or(0);

                    println!("{:?}", response
                        .headers()
                        .get(header::CONTENT_RANGE));

                    println!("{}", total_size);
                    
                    let accept_ranges = response
                        .headers()
                        .get(header::ACCEPT_RANGES)
                        .and_then(|value| value.to_str().ok())
                        .unwrap_or("none");

                    let pb = ProgressBar::new(total_size);
                    pb.set_style(ProgressStyle::default_bar()
                        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                        .progress_chars("=>-"));

                    // 先判断服务器端是否支持断点续传
                    if accept_ranges == "bytes" && downloaded_size > 0 {
                        pb.inc(downloaded_size);
                    }
                    
                    let mut source = DownloadProgress {
                        progress_bar: pb,
                        inner: response,
                    };

                    // 参考资料：https://www.cnblogs.com/amyzhu/p/8060451.html
                    let mut dest = fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&saved_file_part_path).unwrap();
                    
                    let started = Instant::now();
                    copy(&mut source, &mut dest).unwrap();
                    source.progress_bar.finish_and_clear();
                    // 下载完成后，将文件名中的 .part 去掉
                    fs::rename(saved_file_part_path, saved_file_path).unwrap();
                    println!("> [INFO]: 下载完成，耗时 {}", HumanDuration(started.elapsed()));
                    Some(saved_file_path.to_string())
                }
                StatusCode::NOT_FOUND => {
                    println!("> [ERROR]: 下载失败，没有找到要下载的文件，状态码: 404");
                    println!("> [ERROR]: 下载地址: {}", response.url().as_str());

                    None
                }
                s => {
                    println!("> [ERROR]: 下载失败，状态码: {:?}", s);
                    println!("> [ERROR]: 下载地址: {}", response.url().as_str());

                    None
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use mockito;
    use std::path::Path;
    use std::{fs};
    use std::io::prelude::*;
    use mockito::mock;
    use tempfile::NamedTempFile;
    use crate::config::{self, REST_API_INSTALLERS, REST_API_APPS};
    use crate::util::os;
    use super::{print_errors,
                register_installer,
                unregister_installer, 
                download};
    use serde_json;

    use reqwest;
    use std::collections::HashMap;

    fn get_root_url() -> String{
        mockito::server_url()
    }

    #[test]
    fn test_mock() {
        let mock = mock("POST", "/hello")
            .with_header("Content-Type", "application/json")
            .with_status(201)
            .create();

        let mut map = HashMap::new();
        map.insert("lang", "rust");

        let client = reqwest::Client::new();
        client.post(&format!("{}/hello", &get_root_url())).json(&map).send().unwrap();

        mock.assert();
    }

    #[test]
    fn register_installer_success() -> Result<(), Box<dyn std::error::Error>> {
        // 模拟一个 installers POST 服务
        let url = format!("/{}", REST_API_INSTALLERS);
        let mock = mock("POST", &*url)
            .with_header("content-type", "application/json")
            .with_body(r#"{
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
        let installer_info = register_installer(&get_root_url(), "registration_token", 80, "server_token")?;
        println!("{:#?}", installer_info);
        // 断言返回的结果
        assert_eq!(get_root_url(), installer_info.url.unwrap());
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
    fn register_installer_params_not_valid() -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("/{}", REST_API_INSTALLERS);
        let mock = mock("POST", &*url)
            .with_status(422)
            .with_body(r#"{
                            "errors": {
                                "registrationToken": ["注册 Token 不能为空"]
                            }
                        }"#)
            .create();

        // 请求 installers 服务
        assert!(register_installer(&get_root_url(), 
            "not-exist-registration-token", 
            80, 
            "server_token_1").is_err());

        // 断言已执行过 mock 的 http 服务
        mock.assert();

        Ok(())
    }

    #[test]
    fn unregister_installer_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let installer_token = "1";
        // 模拟一个 installers DELETE 服务
        let url = format!("/{}/{}", REST_API_INSTALLERS, installer_token);
        let mock = mock("DELETE", &*url)
            .with_status(404)
            .create();

        // 请求 installers 的注销服务
        assert!(unregister_installer(&get_root_url(), installer_token).is_err());

        // 断言已执行过 mock 的 http 服务
        mock.assert();

        Ok(())
    }

    #[test]
    fn unregister_installer_success() -> Result<(), Box<dyn std::error::Error>> {
        let installer_token = "1";
        // 模拟一个 installers DELETE 服务
        let url = format!("/{}/{}", REST_API_INSTALLERS, installer_token);
        let mock = mock("DELETE", &*url)
            .with_status(204)
            .create();

        // 请求 installers 的注销服务
        assert!(unregister_installer(&get_root_url(), installer_token).is_ok());

        // 断言已执行过 mock 的 http 服务
        mock.assert();

        Ok(())
    }

    #[test]
    fn download_fail() {
        assert_eq!(None, download(&get_root_url(), "app", "0.1.0", "app-0.1.0.zip"));
    }

    #[test]
    fn download_success() -> Result<(), Box<dyn std::error::Error>> {
        // 创建一个临时文件，当作下载文件
        let mut file = NamedTempFile::new()?;
        writeln!(file, "I am a app!")?;
        let path = file.path();
        let path = path.to_str().unwrap();

        // mock 下载文件的 http 服务
        let os_info = os::get_os_info();
        let url = format!("/{}?appName=app&version=0.1.1&targetOs={}&arch={}",
            REST_API_APPS,
            os_info.target_os,
            os_info.target_arch);

        let mock = mock("GET", &*url) // FIXME: 为什么 &url 不起作用？
            .with_body_from_file(path)
            .with_status(200)
            .create();
        
        {
            // 执行下载文件方法
            let downloaded_file_path = download(&get_root_url(), "app", "0.1.1", "app-0.1.1.zip").unwrap();

            // 断言文件已下载成功
            assert!(Path::new(&downloaded_file_path).exists());

            // 删除已下载的文件
            fs::remove_dir_all(config::ROOT_PATH_APP)?;
        }

        // 断言已执行过 mock 的 http 服务
        mock.assert();

        Ok(())
    }

    #[test]
    fn print_errors_only_has_global_errors_success() -> Result<(), Box<dyn std::error::Error>> {
        let data = r#"{"errors": {
                "globalErrors": ["first global error", "second global error"]
            }}"#;
        let v: serde_json::Value = serde_json::from_str(data)?;
        let mut actual = Vec::new();
        print_errors(v, &mut actual);
        assert_eq!(String::from_utf8(actual).unwrap(), String::from("> [ERROR]: 1. first global error\n> [ERROR]: 2. second global error\n"));
        Ok(())
    }

    #[test]
    fn print_errors_only_has_field_errors_success() -> Result<(), Box<dyn std::error::Error>> {
        let data = r#"{"errors": {
                "field1Errors": ["first field1 error", "second field1 error"]
            }}"#;
        let v: serde_json::Value = serde_json::from_str(data)?;
        let mut actual = Vec::new();
        print_errors(v, &mut actual);
        assert_eq!(String::from_utf8(actual).unwrap(), String::from("> [ERROR]: 1. first field1 error\n> [ERROR]: 2. second field1 error\n"));
        Ok(())
    }

    #[test]
    fn print_errors_has_field_and_global_errors_success() -> Result<(), Box<dyn std::error::Error>> {
        let data = r#"{"errors": {
                "globalErrors": ["first global error", "second global error"],
                "field1Errors": ["first field1 error", "second field1 error"]
            }}"#;
        let v: serde_json::Value = serde_json::from_str(data)?;
        let mut actual = Vec::new();
        print_errors(v, &mut actual);
        assert_eq!(String::from_utf8(actual).unwrap(), String::from("> [ERROR]: 1. first field1 error\n> [ERROR]: 2. second field1 error\n> [ERROR]: 3. first global error\n> [ERROR]: 4. second global error\n"));
        Ok(())
    }
}
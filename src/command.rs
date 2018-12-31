use std::path::Path;
use std::fs;

use crate::config;
use crate::http::client;
use crate::jar;
use crate::util::{zip, process};

/// 注册命令
pub fn register(url: &str, token: &str) -> Result<(), Box<std::error::Error>> {
    // 向 Block Lang 平台发送注册请求
    let installer_info = client::register_installer(url, token)?;
    // 存储安装信息
    config::save(installer_info);

    Ok(())
}

/// 启动命令
/// 
/// 在启动时会使用 `config.toml` 中的 `software_name` 和 `software_version` 等信息
/// 在 `prod` 文件夹下检查 Spring boot jar 和 JDK 文件是否已存在，如果不存在则先下载。
/// 下载并解压成功后，启动 Spring Boot jar。
pub fn start() -> Result<(), Box<std::error::Error>> {
    let config_info = config::read()?;
    let installers = config_info.installers.unwrap();
    assert!(installers.len() < 1, "没有找到 installer。请先执行 `blocklang-installer register` 注册 installer");

    // 当前版本只支持一个服务器上配置一个 installer。
    let first_installer = installers.get(0).unwrap();

    // 有两条检查路径，一是先检查下载文件夹，然后检查 prod 文件夹；
    // 二是先检查 prod 文件夹，然后检查下载文件夹。
    // 这里选用第一条检查路径。

    // 检查 Spring Boot Jar
    // 1. 检查 Spring Boot Jar 是否已下载
    let download_spring_boot_jar_path = Path::new(config::ROOT_PATH_SOFTWARE)
        .join(&first_installer.software_name)
        .join(&first_installer.software_version)
        .join(&first_installer.software_file_name);
    if !download_spring_boot_jar_path.exists() {
        client::download(&first_installer.software_name,
                         &first_installer.software_version,
                         &first_installer.software_file_name);
    }
    // 2. 检查 prod 下是否有 Spring Boot Jar
    let prod_spring_boot_jar_path = Path::new(config::ROOT_PATH_PROD)
        .join(&first_installer.software_name)
        .join(&first_installer.software_version)
        .join(&first_installer.software_file_name);
    if !prod_spring_boot_jar_path.exists() {
        // 复制文件
        fs::create_dir_all(prod_spring_boot_jar_path.parent().unwrap())?;
        fs::copy(download_spring_boot_jar_path, &prod_spring_boot_jar_path)?;
    }

    // 检查 JDK
    // 1. 检查 JDK 是否已下载
    let download_jdk_path = Path::new(config::ROOT_PATH_SOFTWARE)
        .join(&first_installer.jdk_name)
        .join(&first_installer.jdk_version)
        .join(&first_installer.jdk_file_name);
    if !download_jdk_path.exists() {
        client::download(&first_installer.jdk_name,
                         &first_installer.jdk_version,
                         &first_installer.jdk_file_name);
    }
    // 2. 检查 prod 中是否有 JDK
    let prod_jdk_path = Path::new(config::ROOT_PATH_PROD)
        .join(&first_installer.jdk_name)
        .join(&first_installer.jdk_version)
        // 注意，因为 jdk 的命名规范是 jdk-11.0.1
        .join(format!("jdk-{}", first_installer.jdk_version));
    if !prod_jdk_path.exists() {
        zip::unzip_to(download_jdk_path.to_str().unwrap(), 
                      prod_jdk_path.parent().unwrap().to_str().unwrap())
            .expect("解压 JDK 出错");
    }

    // 运行 Spring Boot Jar
    jar::run_spring_boot(prod_spring_boot_jar_path.to_str().unwrap(), 
                         prod_jdk_path.to_str().unwrap());

    Ok(())
}

/// 关闭命令
pub fn stop() -> Result<(), Box<std::error::Error>> {
    let config = config::read()?;
    let installers = config.installers.unwrap();
    assert!(installers.len() < 1, "没有找到 installer。请先执行 `blocklang-installer register` 注册 installer");

    // 当前版本只支持一个服务器上配置一个 installer。
    let first_installer = installers.get(0).unwrap();

    // 根据在 config.toml 中登记的 spring boot jar 的运行端口来找到进程，并 kill 掉进程，
    // 以此来关闭 spring boot jar。
    match process::get_id(first_installer.software_run_port) {
        Some(x) => {
            process::kill(x);
        }
        None => {
            println!("没有发现运行端口 {} 的进程", first_installer.software_run_port);
        }
    }

    Ok(())
}

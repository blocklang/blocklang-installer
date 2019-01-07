use std::path::{Path, PathBuf};
use std::fs;
use version_compare::Version;
use crate::config;
use crate::http::client;
use crate::jar;
use crate::util::{zip, process};

/// 注册命令
pub fn register(url: &str,
    registration_token: &str,
    software_run_port: u32) -> Result<(), Box<std::error::Error>> {

    let mut config_info = config::get()?;
    let server_token = &config_info.server_token;
    // 向 Block Lang 平台发送注册请求
    let installer_info = client::register_installer(url, registration_token, software_run_port, server_token)?;
    // 添加安装信息
    config::add_installer(&mut config_info, installer_info);
    config::save(config_info);

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

    let prod_spring_boot_jar_path = ensure_spring_boot_jar_exists(
        &first_installer.software_name,
        &first_installer.software_version,
        &first_installer.software_file_name)?;
    let prod_jdk_path = ensure_jdk_exists(
        &first_installer.jdk_name,
        &first_installer.jdk_version,
        &first_installer.jdk_file_name)?;

    // 运行 Spring Boot Jar
    jar::run_spring_boot(
        prod_spring_boot_jar_path.to_str().unwrap(), 
        prod_jdk_path.to_str().unwrap());

    Ok(())
}

/// 更新命令
pub fn update() -> Result<(), Box<std::error::Error>> {
    // 读取配置文件中的 url 和 token
    let config_info = config::read()?;
    let installers = config_info.installers.unwrap();
    assert!(installers.len() < 1, "没有找到 installer。请先执行 `blocklang-installer register` 注册 installer");

    // 当前版本只支持一个服务器上配置一个 installer。
    let first_installer = installers.get(0).unwrap();

    // 从 Block Lang 软件发布中心获取软件最新版信息
    let installer_info = client::update_installer(&first_installer.url, &first_installer.installer_token)?;

    // 检查 spring boot jar 是否有升级
    let jar_old_ver = Version::from(&first_installer.software_version).unwrap();
    let jar_new_ver = Version::from(&installer_info.software_version).unwrap();
    let jar_upgraded = jar_new_ver > jar_old_ver;

    // 检查 jdk 是否有升级
    let jdk_old_ver = Version::from(&first_installer.jdk_version).unwrap();
    let jdk_new_ver = Version::from(&installer_info.jdk_version).unwrap();
    let jdk_upgraded = jdk_new_ver > jdk_old_ver;

    // 更新 config.toml
    // 不管是否有升级新版本，都要更新
    // FIXME:
    // config::save(installer_info.clone());

    // 如果软件版本没有变化，则提示当前运行的 spring boot jar 已是最新版本
    if !jar_upgraded && !jdk_upgraded {
        println!("已是最新版本。{} 的版本是 {}，JDK 的版本是 {}。", 
            installer_info.software_name,
            installer_info.software_version,
            installer_info.jdk_version);
        return Ok(());
    }

    // 如果版本已有新版本，则更新并运行最新版本(只要 jdk 或 jar 有一个升级就重启)
    // 1. 更新 JDK
    let prod_jdk_path = if jdk_upgraded {
        ensure_jdk_exists(
            &first_installer.jdk_name,
            &first_installer.jdk_version,
            &first_installer.jdk_file_name)?
    } else {
        get_prod_jdk_path(&first_installer.jdk_name, 
            &first_installer.jdk_version)
    };

    // 2. 更新 spring boot jar
    let prod_spring_boot_jar_path =  if jar_upgraded {
        ensure_spring_boot_jar_exists(
            &first_installer.software_name,
            &first_installer.software_version,
            &first_installer.software_file_name)?
    } else {
        get_prod_spring_boot_jar_path(
            &first_installer.software_name,
            &first_installer.software_version,
            &first_installer.software_file_name)
    };

    // 3. 停止旧版 jar
    stop_jar(first_installer.software_run_port);
    // 4. 启动新版 jar
    jar::run_spring_boot(
        prod_spring_boot_jar_path.to_str().unwrap(), 
        prod_jdk_path.to_str().unwrap());

    println!("更新完成。{} 的版本是 {}，JDK 的版本是 {}。", 
        installer_info.software_name,
        installer_info.software_version,
        installer_info.jdk_version);

    Ok(())
}

/// 关闭命令
pub fn stop() -> Result<(), Box<std::error::Error>> {
    let config = config::read()?;
    let installers = config.installers.unwrap();
    assert!(installers.len() < 1, "没有找到 installer。请先执行 `blocklang-installer register` 注册 installer");

    // 当前版本只支持一个服务器上配置一个 installer。
    let first_installer = installers.get(0).unwrap();

    stop_jar(first_installer.software_run_port);

    Ok(())
}

/// 停止运行 spring boot jar。
fn stop_jar(run_port: u32) {
    // 根据在 config.toml 中登记的 spring boot jar 的运行端口来找到进程，并 kill 掉进程，
    // 以此来关闭 spring boot jar。
    match process::get_id(run_port) {
        Some(x) => {
            process::kill(x);
        }
        None => {
            println!("没有发现运行端口 {} 的进程", run_port);
        }
    }
}

/// 确认 JDK 是否已成功解压到 prod 文件夹。
/// 
/// 有两条检查路径，一是先检查下载文件夹，然后检查 prod 文件夹；
/// 二是先检查 prod 文件夹，然后检查下载文件夹。
/// 这里选用第一条检查路径。
fn ensure_jdk_exists(jdk_name: &str,
                     jdk_version: &str,
                     jdk_file_name: &str) -> Result<PathBuf, Box<std::error::Error>>  {
    // 1. 检查 JDK 是否已下载
    let download_jdk_path = Path::new(config::ROOT_PATH_SOFTWARE)
        .join(jdk_name)
        .join(jdk_version)
        .join(jdk_file_name);
    if !download_jdk_path.exists() {
        client::download(jdk_name,
                         jdk_version,
                         jdk_file_name);
    }
    // 2. 检查 prod 中是否有 JDK
    let prod_jdk_path = get_prod_jdk_path(jdk_name, jdk_version);
    if !prod_jdk_path.exists() {
        zip::unzip_to(download_jdk_path.to_str().unwrap(), 
                      prod_jdk_path.parent().unwrap().to_str().unwrap())
            .expect("解压 JDK 出错");
    }

    Ok(prod_jdk_path)
}

/// 确认 Spring Boot Jar 是否已成功复制到 prod 文件夹。
/// 
/// 有两条检查路径，一是先检查下载文件夹，然后检查 prod 文件夹；
/// 二是先检查 prod 文件夹，然后检查下载文件夹。
/// 这里选用第一条检查路径。
fn ensure_spring_boot_jar_exists (software_name: &str,
                                  software_version: &str,
                                  software_file_name: &str) -> Result<PathBuf, Box<std::error::Error>> {
    // 1. 检查 Spring Boot Jar 是否已下载
    let download_spring_boot_jar_path = Path::new(config::ROOT_PATH_SOFTWARE)
        .join(software_name)
        .join(software_version)
        .join(software_file_name);
    if !download_spring_boot_jar_path.exists() {
        client::download(software_name,
                         software_version,
                         software_file_name);
    }
    // 2. 检查 prod 下是否有 Spring Boot Jar
    let prod_spring_boot_jar_path = get_prod_spring_boot_jar_path(
        software_name, 
        software_version, 
        software_file_name);
    if !prod_spring_boot_jar_path.exists() {
        // 复制文件
        fs::create_dir_all(prod_spring_boot_jar_path.parent().unwrap())?;
        fs::copy(download_spring_boot_jar_path, &prod_spring_boot_jar_path)?;
    }

    Ok(prod_spring_boot_jar_path)
}

/// 获取 prod 文件夹中 Spring boot jar 的路径。
fn get_prod_spring_boot_jar_path(software_name: &str,
    software_version: &str,
    software_file_name: &str) -> PathBuf {
    Path::new(config::ROOT_PATH_PROD)
        .join(software_name)
        .join(software_version)
        .join(software_file_name)
}

/// 获取 prod 文件夹中 JDK 的路径。
fn get_prod_jdk_path(jdk_name: &str,
    jdk_version: &str) -> PathBuf {
    Path::new(config::ROOT_PATH_PROD)
        .join(jdk_name)
        .join(jdk_version)
        // 注意，因为 jdk 的命名规范是 jdk-11.0.1
        .join(format!("jdk-{}", jdk_version))
}

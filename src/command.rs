use std::path::{Path, PathBuf};
use std::fs;
use std::time::Instant;
use version_compare::Version;
use crate::config;
use crate::http::client;
use crate::jar;
use crate::util::{zip, process};
use prettytable::{Table, Row, Cell, row, cell};
use indicatif::HumanDuration;

/// 注册命令
pub fn register_installer(url: &str,
    registration_token: &str,
    app_run_port: u32) -> Result<(), Box<std::error::Error>> {

    let mut config_info = config::get()?;
    let server_token = &config_info.server_token;
    // 向 Block Lang 平台发送注册请求
    let installer_info = client::register_installer(url, registration_token, app_run_port, server_token)?;
    // 添加安装信息
    config::add_installer(&mut config_info, installer_info);
    config::save(config_info);

    Ok(())
}

pub fn list_installers() -> Result<(), Box<std::error::Error>> {
    let config_info = config::get()?;
    let installers = config_info.installers;
    if installers.is_empty() {
        println!("还没有注册 installer，请使用 `blocklang-installer register` 命令注册 installer。");
    } else {
        // 获取每一列文本的最大长度，然后在此基础上加四个空格
        // 端口号  Installer Token    URL
        let mut table = Table::new();
        // 标题行
        table.add_row(row!["Port", "Installer Token", "URL"]);
        // 数据行
        installers.iter().for_each(|installer| {
            table.add_row(Row::new(vec![
                Cell::new(&installer.app_run_port.to_string()),
                Cell::new(&installer.installer_token),
                Cell::new(&installer.url),
            ]));
        });
        table.printstd();
    }

    Ok(())
}

pub fn unregister_single_installer(app_run_port: u32) -> Result<(), Box<std::error::Error>> {
    // 根据运行 APP 实例的端口号获取 installer 信息
    let mut config_info = config::read()?;
    // 以下大段代码都是因为 `config_info` 先被不可变借用，然后被可变借用，违反了 rust 的规则，
    // 所以将这两个操作分成两段，放在两块作用域中，然后第一个作用域只返回克隆的值。
    // TODO: 此处写的代码过多，感觉将简单问题复杂化了，当理解后要优化。
    let (installer_token, url) = 
    {
        let installer = config::get_installer_by_port(&config_info, app_run_port);
        match installer {
            None => {
                // 如果没有找到，则给出提示
                println!("提示：在配置文件中没有找到端口号 {}", app_run_port);
                return Ok(())
            },
            Some(v) => {
                (v.installer_token.clone(), v.url.clone())
            }
        }
    };

    println!("开始注销对应 {} 端口的 installer", app_run_port);
    // 向 Block Lang 平台注销 installer
    print!("开始向 Block Lang 平台注销 installer");
    client::unregister_installer(&url, &installer_token)?;
    println!(" ---- Ok");
    // TODO 添加校验，如果 APP 处于运行状态，则关闭该 APP
    print!("开始关闭 APP");
    stop_jar(app_run_port);
    println!(" ---- Ok");
    // 在 `config.toml` 文件中删除此 installer 的配置信息
    print!("开始从配置文件中删除");
    config::remove_installer(&mut config_info, &installer_token);
    config::save(config_info);
    println!(" ---- Ok");
    println!("注销完成！");
    
    Ok(())
}

pub fn unregister_all_installers() -> Result<(), Box<std::error::Error>> {
    let mut config_info = config::read()?;
    {
        let installers = &mut config_info.installers;
        if installers.is_empty() {
            println!("提示：配置文件中没有找到 installer。");
            return Ok(());
        }

        installers.retain(|installer| {
            println!("开始注销对应 {} 端口的 installer", installer.app_run_port);
            // 向 Block Lang 平台注销 installer
            print!("开始向 Block Lang 平台注销 installer");
            match client::unregister_installer(&installer.url, &installer.installer_token) {
                Ok(_) => {
                    println!(" ---- Ok");
                    // TODO 添加校验，如果 APP 处于运行状态，则关闭该 APP
                    print!("开始关闭 APP");
                    stop_jar(installer.app_run_port);
                    println!(" ---- Ok");
                    println!("注销完成！");
                    false
                },
                Err(e) => {
                    println!(" ---- 向 Block Lang 平台注销失败 {}", e);
                    true
                }
            }
        });
    }
    // 将调整后的配置信息保存起来。
    config::save(config_info);

    Ok(())
}

/// 启动命令，启动单个 APP
/// 
/// 在启动时会使用 `config.toml` 中的 `app_name` 和 `app_version` 等信息
/// 在 `prod` 文件夹下检查 Spring boot jar 和 JDK 文件是否已存在，如果不存在则先下载。
/// 下载并解压成功后，启动 Spring Boot jar。
pub fn run_single_app(app_run_port: u32) -> Result<(), Box<std::error::Error>> {
    let config_info = config::read()?;

    let installer_option = config::get_installer_by_port(&config_info, app_run_port);

    match installer_option {
        None => {
            println!("没有找到 installer。请先执行 `blocklang-installer register` 注册 installer");
        },
        Some(installer) => {
            run_app(installer)?;
        }
    };

    Ok(())
}

/// 启动命令，启动所有注册的 APP
pub fn run_all_apps() -> Result<(), Box<std::error::Error>> {
    let config_info = config::read()?;
    let installers = config_info.installers;
    if installers.is_empty() {
        println!("没有找到 installer。请先执行 `blocklang-installer register` 注册 installer");
        return Ok(());
    }
    for installer in installers.iter() {
        run_app(installer)?;
    }

    Ok(())
}

fn run_app(installer: &config::InstallerConfig) -> Result<(), Box<std::error::Error>>  {
    let started = Instant::now();

    println!("开始下载并安装 {}-{}，使用 {} 端口", 
        installer.app_name,
        installer.app_version,
        installer.app_run_port);

    println!("[1/3] 下载 Jar 包: {}...", installer.app_file_name);
    let prod_spring_boot_jar_path = ensure_spring_boot_jar_exists(
            &installer.app_name,
            &installer.app_version,
            &installer.app_file_name)?;

    println!("[2/3] 下载 Oracle JDK: {}...", installer.jdk_file_name);
    let prod_jdk_path = ensure_jdk_exists(
        &installer.jdk_name,
        &installer.jdk_version,
        &installer.jdk_file_name)?;

    println!("[3/3] 在 {} 端口上启动项目...", installer.app_run_port);
    // 假定运行在所有端口上的项目，都是 installer 管理的
    // 这样就不会出现在端口上运行的不是我们期望的 APP

    // 如果端口被占用，则认为程序已启动，不需重启
    if process::get_id(installer.app_run_port) == None {
        // 运行 Spring Boot Jar
        jar::run_spring_boot(
            prod_spring_boot_jar_path.to_str().unwrap(), 
            prod_jdk_path.to_str().unwrap(),
            installer.app_run_port);
        println!("> [INFO]: 项目启动成功");
    } else {
        println!("> [INFO]: 项目已处于运行状态");
    }
    println!("完成！耗时 {}", HumanDuration(started.elapsed()));
    Ok(())
}

/// 升级单个 APP
pub fn update_single_app(app_run_port: u32) -> Result<(), Box<std::error::Error>> {
    let config_info = config::read()?;
    let installer_option = config::get_installer_by_port(&config_info, app_run_port);

    match installer_option {
        None => {
            println!("没有找到 installer。请先执行 `blocklang-installer register` 注册 installer");
        },
        Some(installer) => {
            update_app(installer)?;
        }
    }

    Ok(())
}

/// 升级所有 APP
pub fn update_all_apps() -> Result<(), Box<std::error::Error>> {
    let config_info = config::read()?;
    let installers = config_info.installers;
    if installers.is_empty() {
        println!("没有找到 APP。请先执行 `blocklang-installer register` 注册 installer");
        return Ok(());
    }

    for installer in installers.iter() {
        update_app(installer)?;
    }

    Ok(())
}

fn update_app(installer: &config::InstallerConfig) -> Result<(), Box<std::error::Error>> {
    // 从 Block Lang 软件发布中心获取软件最新版信息
    let installer_info = client::update_installer(&installer.url, &installer.installer_token)?;

    // 检查 spring boot jar 是否有升级
    let jar_old_ver = Version::from(&installer.app_version).unwrap();
    let jar_new_ver = Version::from(&installer.app_version).unwrap();
    let jar_upgraded = jar_new_ver > jar_old_ver;

    // 检查 jdk 是否有升级
    let jdk_old_ver = Version::from(&installer.jdk_version).unwrap();
    let jdk_new_ver = Version::from(&installer_info.jdk_version).unwrap();
    let jdk_upgraded = jdk_new_ver > jdk_old_ver;

    // 更新 config.toml
    // 不管是否有升级新版本，都要更新
    // FIXME:
    // config::save(installer_info.clone());

    // 如果软件版本没有变化，则提示当前运行的 spring boot jar 已是最新版本
    if !jar_upgraded && !jdk_upgraded {
        println!("已是最新版本。{} 的版本是 {}，JDK 的版本是 {}。", 
            installer_info.app_name,
            installer_info.app_version,
            installer_info.jdk_version);
        return Ok(());
    }

    // 如果版本已有新版本，则更新并运行最新版本(只要 jdk 或 jar 有一个升级就重启)
    // 1. 更新 JDK
    let prod_jdk_path = if jdk_upgraded {
        ensure_jdk_exists(
            &installer.jdk_name,
            &installer.jdk_version,
            &installer.jdk_file_name)?
    } else {
        get_prod_jdk_path(&installer.jdk_name, 
            &installer.jdk_version)
    };

    // 2. 更新 spring boot jar
    let prod_spring_boot_jar_path =  if jar_upgraded {
        ensure_spring_boot_jar_exists(
            &installer.app_name,
            &installer.app_version,
            &installer.app_file_name)?
    } else {
        get_prod_spring_boot_jar_path(
            &installer.app_name,
            &installer.app_version,
            &installer.app_file_name)
    };

    if process::get_id(installer.app_run_port) == None {
        // 如果 APP 没有运行，则提示程序的运行状态
        println!("{} 没有运行", installer_info.app_name);
    } else {
        // 如果 APP 正在运行，则重启 APP
        // 3. 停止旧版 jar
        stop_jar(installer.app_run_port);
        // 4. 启动新版 jar
        jar::run_spring_boot(
            prod_spring_boot_jar_path.to_str().unwrap(), 
            prod_jdk_path.to_str().unwrap(),
            installer.app_run_port);
        
        println!("{} 正运行在 {} 端口上", 
            installer_info.app_name,
            installer.app_run_port);
    }
    println!("更新完成。{} 的版本是 {}，JDK 的版本是 {}。", 
        installer_info.app_name,
        installer_info.app_version,
        installer_info.jdk_version);

    Ok(())
}

/// 停止单个 APP
pub fn stop_single_app(app_run_port: u32) -> Result<(), Box<std::error::Error>> {
    let config_info = config::read()?;
    let installer_option = config::get_installer_by_port(&config_info, app_run_port);

    match installer_option {
        None => {
            println!("没有找到注册到 {} 上的 APP。请先执行 `blocklang-installer register` 注册 installer", 
                app_run_port);
        },
        Some(_) => {
            stop_jar(app_run_port);
        }
    }

    Ok(())
}

/// 停止所有 APP
pub fn stop_all_apps() -> Result<(), Box<std::error::Error>> {
    let config_info = config::read()?;
    let installers = config_info.installers;
    if installers.is_empty() {
        println!("没有找到 APP。请先执行 `blocklang-installer register` 注册 installer");
        return Ok(());
    }

    for installer in installers.iter() {
        stop_jar(installer.app_run_port);
    }

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
    let download_jdk_path = Path::new(config::ROOT_PATH_APP)
        .join(jdk_name)
        .join(jdk_version)
        .join(jdk_file_name);
    if !download_jdk_path.exists() {
        client::download(jdk_name,
                         jdk_version,
                         jdk_file_name);
    } else {
        println!("> [INFO]: 文件已存在");
    }
    // 2. 检查 prod 中是否有 JDK
    let prod_jdk_path = get_prod_jdk_path(jdk_name, jdk_version);
    if !prod_jdk_path.exists() {
        print!("> [INFO]: 正在解压 JDK...");
        zip::unzip_to(download_jdk_path.to_str().unwrap(), 
                      prod_jdk_path.parent().unwrap().to_str().unwrap())
            .expect("解压 JDK 出错");
        println!("...完成");
    } else {
        println!("> [INFO]: 文件已解压");
    }

    Ok(prod_jdk_path)
}

/// 确认 Spring Boot Jar 是否已成功复制到 prod 文件夹。
/// 
/// 有两条检查路径，一是先检查下载文件夹，然后检查 prod 文件夹；
/// 二是先检查 prod 文件夹，然后检查下载文件夹。
/// 这里选用第一条检查路径。
fn ensure_spring_boot_jar_exists (app_name: &str,
                                  app_version: &str,
                                  app_file_name: &str) -> Result<PathBuf, Box<std::error::Error>> {
    
    // 1. 检查 Spring Boot Jar 是否已下载
    let download_spring_boot_jar_path = Path::new(config::ROOT_PATH_APP)
        .join(app_name)
        .join(app_version)
        .join(app_file_name);
    if !download_spring_boot_jar_path.exists() {
        client::download(app_name,
                         app_version,
                         app_file_name);
    } else {
        println!("> [INFO]: 文件已存在");
    }
    // 2. 检查 prod 下是否有 Spring Boot Jar
    let prod_spring_boot_jar_path = get_prod_spring_boot_jar_path(
        app_name, 
        app_version, 
        app_file_name);
    if !prod_spring_boot_jar_path.exists() {
        // 复制文件
        fs::create_dir_all(prod_spring_boot_jar_path.parent().unwrap())?;
        fs::copy(download_spring_boot_jar_path, &prod_spring_boot_jar_path)?;
    }

    Ok(prod_spring_boot_jar_path)
}

/// 获取 prod 文件夹中 Spring boot jar 的路径。
fn get_prod_spring_boot_jar_path(app_name: &str,
    app_version: &str,
    app_file_name: &str) -> PathBuf {
    Path::new(config::ROOT_PATH_PROD)
        .join(app_name)
        .join(app_version)
        .join(app_file_name)
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
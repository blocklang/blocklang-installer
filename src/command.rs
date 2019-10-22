use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::time::Instant;
use std::io::{self, Write};
use version_compare::Version;

use crate::config;
use crate::installer_config::{Installer, InstallerConfig};
use crate::http::client;
use crate::jar;
use crate::util::{zip, process};
use prettytable::{Table, Row, Cell, row, cell};
use indicatif::HumanDuration;

/// 注册命令
pub fn register_installer(url: &str,
    registration_token: &str,
    app_run_port: u32) -> Result<(), Box<dyn std::error::Error>> {
    
    let mut installer_config = InstallerConfig::new();

    let server_token = &installer_config.get_data().server_token;
    // 向 Block Lang 平台发送注册请求
    let installer_info = client::register_installer(url, registration_token, app_run_port, server_token)?;
    // 添加安装信息
    installer_config.add(installer_info);

    Ok(())
}

pub fn list_installers() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始查找已注册的安装器");

    let installer_config = InstallerConfig::new();

    let installers = &installer_config.get_data().installers;
    if installers.is_empty() {
        println!("> [INFO]: 共找到 0 个 installer，请使用 `blocklang-installer register` 命令注册。");
    } else {
        println!("> [INFO]: 共找到 {} 个 installer。", installers.len());
        print_installers(&installers);
    }

    Ok(())
}

pub fn unregister_single_installer(app_run_port: u32) -> Result<(), Box<dyn std::error::Error>> {
    println!("开始注销 {} 端口上的 installer", app_run_port);
    let installer_config = InstallerConfig::new();

    // 注意：不能关闭未注册的端口，防止误关安装在应用服务器上的其他应用。
    if let Some(installer) = installer_config.get_by_port(app_run_port) {
        println!("> [INFO]: 端口号 {} 上注册的 installer 信息如下：", app_run_port);
        
        print_one_installer(installer);

        // 询问用户是否要注销
        println!("> [WARN]: 注销之后，项目将无法访问，确定要注销吗？输入 Y 确定注销，输入 N 退出(默认为 N)：");

        if !confirm_to_continue() {
            println!("> [INFO]: 已退出");
            return Ok(());
        }

        // 向 Block Lang 平台注销 installer
        unregister_installer(&installer)?;

        println!("注销完成！");
    } else {
        println!("> [WARN]: 端口 {} 上未注册 installer，可执行 `blocklang-installer --list` 命令查看已注册的 installer", app_run_port);
    }
    
    Ok(())
}

pub fn unregister_all_installers() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始注销所有 installer");

    let mut installer_config = InstallerConfig::new();

    let installers = &installer_config.get_data().installers;
    if installers.is_empty() {
        println!("> [INFO]: 共找到 0 个 installer");
        return Ok(());
    }

    // 展示所有注册的 installer
    let installer_len = installers.len();
    println!("> [INFO]: 共找到 {} 个 installer。", installer_len);
    print_installers(&installers);

    // 向用户确认，是否要注销
    println!("> [WARN]: 注销之后，项目将无法访问，确定要全部注销吗？输入 Y 确定注销，输入 N 退出(默认为 N)：");
    if !confirm_to_continue() {
        println!("> [INFO]: 已退出");
        return Ok(());
    }

    let mut num = 1;
    installer_config.remove_all(|installer| {
        println!();
        println!("===== [{}/{}] 开始注销 {} 端口上的 installer =====", num, installer_len, installer.app_run_port);
        num += 1;
        // 向 Block Lang 平台注销 installer
        println!("开始向 Block Lang 平台注销 installer");

        unregister_installer(&installer).is_ok()
    });

    Ok(())
}

fn unregister_installer(installer: &Installer) -> Result<(), Box<dyn std::error::Error>> {
    // 向 Block Lang 平台注销 installer
    println!("[1/3] 向 Block Lang 平台注销 installer");
    if client::unregister_installer(&installer.url, &installer.installer_token).is_ok() {
        println!("> [INFO]: 完成");
    }else {
        return Err(Box::from("注销失败"));
    }

    println!("[2/3] 关闭端口 {}", installer.app_run_port);
    // 如果 APP 处于运行状态，则关闭该 APP，此逻辑在 stop_jar 函数中
    stop_jar(installer.app_run_port);

    // 在配置文件中删除此 installer 的配置信息
    println!("[3/3] 从配置文件中删除配置信息");
    // 注意：因为 rustc 提示不可变借用了，不能再可变借用，只有暂时重新 new 一个对象了。
    // TODO: 有没有更好的办法，让只需要 new 一次？
    let mut installer_config = InstallerConfig::new();
    installer_config.remove_by_installer_token(&installer.installer_token);
    println!("> [INFO]: 完成");
    Ok(())
}

fn confirm_to_continue() -> bool {
    let mut io_y_or_n = String::new();
    io::stdin().read_line(&mut io_y_or_n).unwrap();
    io_y_or_n = io_y_or_n.trim().to_string();
    if io_y_or_n.is_empty() {
        io_y_or_n.push_str("N");
    }
    io_y_or_n.to_uppercase() == "Y"
}

fn print_one_installer(installer: &Installer) {
    let mut table = Table::new();
    // 标题行
    table.add_row(row!["端口号", "Installer Token", "URL", "项目名", "版本号"]);
    // 数据行
    table.add_row(Row::new(vec![
        Cell::new(&installer.app_run_port.to_string()),
        Cell::new(&installer.installer_token),
        Cell::new(&installer.url),
        Cell::new(&installer.app_name),
        Cell::new(&installer.app_version),
    ]));
    table.printstd();
}

fn print_installers(installers: &[Installer]) {
    let mut table = Table::new();
    // 标题行
    table.add_row(row!["端口号", "Installer Token", "URL", "项目名", "版本号"]);
    // 数据行
    installers.iter().for_each(|installer| {
        table.add_row(Row::new(vec![
            Cell::new(&installer.app_run_port.to_string()),
            Cell::new(&installer.installer_token),
            Cell::new(&installer.url),
            Cell::new(&installer.app_name),
            Cell::new(&installer.app_version),
        ]));
    });
    table.printstd();
}

/// 启动命令，启动单个 APP
/// 
/// 在启动时会使用 `installer_config.toml` 中的 `app_name` 和 `app_version` 等信息
/// 在 `prod` 文件夹下检查 Spring boot jar 和 JDK 文件是否已存在，如果不存在则先下载。
/// 下载并解压成功后，启动 Spring Boot jar。
pub fn run_single_app(app_run_port: u32) -> Result<(), Box<dyn std::error::Error>> {
    let installer_config = InstallerConfig::new();

    match installer_config.get_by_port(app_run_port) {
        Some(installer) => {
            run_app(&installer)?;
        },
        None => {
            println!("> [INFO]: 没有找到 installer。请先执行 `blocklang-installer register` 注册 installer");
        }
    };

    Ok(())
}

/// 启动命令，启动所有注册的 APP
pub fn run_all_apps() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始启动所有项目");

    let installer_config = InstallerConfig::new();

    let installers = &installer_config.get_data().installers;
    if installers.is_empty() {
        println!("> [INFO]: 没有找到 installer。请先执行 `blocklang-installer register` 注册 installer");
        return Ok(());
    }

    let installer_len = installers.len();
    println!("> [INFO]: 共找到 {} 个 installer。", installer_len);

    for (index, installer) in installers.iter().enumerate() {
        println!();
        println!("===== [{}/{}] 开始启动 {} 端口上的项目 {}-{} =====", 
            index + 1, 
            installer_len, 
            installer.app_run_port, 
            installer.app_name, 
            installer.app_version);

        run_app(installer)?;
    }

    Ok(())
}

fn run_app(installer: &Installer) -> Result<(), Box<dyn std::error::Error>>  {
    let started = Instant::now();

    println!("开始下载并安装 {}-{}，使用 {} 端口", 
        installer.app_name,
        installer.app_version,
        installer.app_run_port);

    println!("[1/3] 下载 Jar 包: {}...", installer.app_file_name);
    let prod_spring_boot_jar_path = ensure_spring_boot_jar_exists(
        &installer.url,
        &installer.app_name,
        &installer.app_version,
        &installer.app_file_name)?;

    println!("[2/3] 下载 Oracle JDK: {}...", installer.jdk_file_name);
    let prod_jdk_path = ensure_jdk_exists(
        &installer.url,
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
pub fn update_single_app(app_run_port: u32) -> Result<(), Box<dyn std::error::Error>> {
    println!("开始升级运行在端口 {} 上的项目", app_run_port);

    let installer_config = InstallerConfig::new();

    match installer_config.get_by_port(app_run_port) {
        Some(installer) => {
            println!("> [INFO]: 端口 {} 上正在运行 {}-{}，使用的 JDK 版本是 {}", 
                app_run_port, 
                &installer.app_name, 
                &installer.app_version, 
                &installer.jdk_version);

            update_app(&installer)?;
        }
        None => {
            println!("> [INFO]: 端口 {} 上未注册 installer。请先执行 `blocklang-installer register` 注册 installer", app_run_port);
        }
    }

    Ok(())
}

/// 升级所有 APP
pub fn update_all_apps() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始升级所有项目");

    let installer_config = InstallerConfig::new();
    let installers = &installer_config.get_data().installers;
    if installers.is_empty() {
        println!("> [INFO]: 没有找到 installer。请先执行 `blocklang-installer register` 注册 installer");
        return Ok(());
    }

    let installer_len = installers.len();
    println!("> [INFO]: 共找到 {} 个 installer。", installer_len);

    for (index, installer) in installers.iter().enumerate() {
        println!();
        println!("===== [{}/{}] 开始升级 {} 端口上的项目 {}-{} =====", 
            index + 1, 
            installer_len, 
            installer.app_run_port, 
            installer.app_name, 
            installer.app_version);

        update_app(installer)?;
    }

    Ok(())
}

fn update_app(installer: &Installer) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();

    println!("[1/4] 获取 {} 的最新版本和使用的 JDK 最新版本", &installer.app_name);
    // 从 Block Lang 软件发布中心获取软件最新版信息
    let new_installer = client::update_installer(&installer.url, &installer.installer_token)?;

    // 检查 spring boot jar 是否有升级
    let jar_old_ver = Version::from(&installer.app_version).unwrap();
    let jar_new_ver = Version::from(&new_installer.app_version).unwrap();
    let jar_upgraded = jar_new_ver > jar_old_ver;

    // 检查 jdk 是否有升级
    let jdk_old_ver = Version::from(&installer.jdk_version).unwrap();
    let jdk_new_ver = Version::from(&new_installer.jdk_version).unwrap();
    let jdk_upgraded = jdk_new_ver > jdk_old_ver;

    // 如果软件版本没有变化，则提示当前运行的 spring boot jar 已是最新版本
    if !jar_upgraded && !jdk_upgraded {
        println!("> [INFO]: 已是最新版本。{} 的版本是 {}，JDK 的版本是 {}", 
            new_installer.app_name,
            new_installer.app_version,
            new_installer.jdk_version);
        return Ok(());
    }

    println!("[2/4] 开始升级 Oracle JDK");
    // 如果版本已有新版本，则更新并运行最新版本(只要 jdk 或 jar 有一个升级就重启)
    // 1. 更新 JDK
    let prod_jdk_path = if jdk_upgraded {
        println!("> [INFO]: 从 {} 升级到 {}", &installer.jdk_version, &new_installer.jdk_version);
        ensure_jdk_exists(
            &installer.url, // 注意，url 注册之后就不会再改变。
            &new_installer.jdk_name,
            &new_installer.jdk_version,
            &new_installer.jdk_file_name)?
    } else {
        println!("> [INFO]: 文件已存在");
        get_prod_jdk_path(&installer.jdk_name, &installer.jdk_version)
    };

    println!("[3/4] 开始升级 {}", &new_installer.app_name);
    // 2. 更新 spring boot jar
    let prod_spring_boot_jar_path =  if jar_upgraded {
        println!("> [INFO]: 从 {} 升级到 {}", &installer.app_version, &new_installer.app_version);
        ensure_spring_boot_jar_exists(
            &installer.url,
            &new_installer.app_name,
            &new_installer.app_version,
            &new_installer.app_file_name)?
    } else {
        println!("> [INFO]: 文件已存在");
        get_prod_spring_boot_jar_path(
            &installer.app_name,
            &installer.app_version,
            &installer.app_file_name)
    };

    println!("[4/4] 检查端口 {} 上 {}-{} 的运行状态", 
        installer.app_run_port,
        &new_installer.app_name, 
        &new_installer.app_version);

    if process::get_id(installer.app_run_port).is_none() {
        // 如果 APP 没有运行，则提示程序的运行状态
        println!("> [INFO]: {}-{} 没有运行。依然保持未运行状态", installer.app_name, installer.app_version);
    } else {
        println!("> [INFO]: {}-{} 运行在 {} 端口上，开始重启", installer.app_name, installer.app_version, installer.app_run_port);
        // 如果 APP 正在运行，则重启 APP
        // 3. 停止旧版 jar
        stop_jar(installer.app_run_port);
        // 4. 启动新版 jar
        print!("> [INFO]: 开始重启...");
        io::stdout().flush()?;

        jar::run_spring_boot(
            prod_spring_boot_jar_path.to_str().unwrap(), 
            prod_jdk_path.to_str().unwrap(),
            installer.app_run_port);
        
        println!("完成");
    }

    // 更新 installer_config.toml 中的配置信息
    let mut installer_config = InstallerConfig::new();
    installer_config.update(installer.app_run_port, new_installer);

    println!("升级完成！耗时 {}", HumanDuration(started.elapsed()));
    Ok(())
}

/// 停止单个 APP
pub fn stop_single_app(app_run_port: u32) -> Result<(), Box<dyn std::error::Error>> {
    println!("开始停止运行在 {} 端口上的项目，并关闭此端口", app_run_port);

    let installer_config = InstallerConfig::new();

    // 注意：只关闭注册 installer 的端口，防止误关安装在应用服务器上的其他应用。
    match installer_config.get_by_port(app_run_port) {
        Some(_) => {
            stop_jar(app_run_port);
        }
        None => {
            println!("> [INFO]: {} 端口上未注册项目", app_run_port);
        }
    }

    Ok(())
}

/// 停止所有 APP
pub fn stop_all_apps() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始关闭所有项目");

    let installer_config = InstallerConfig::new();

    let installers = &installer_config.get_data().installers;
    if installers.is_empty() {
        println!("> [INFO]: 没有找到 installer。请先执行 `blocklang-installer register` 注册 installer");
        return Ok(());
    }

    let installer_len = installers.len();
    println!("> [INFO]: 共找到 {} 个 installer。", installer_len);

    for (index, installer) in installers.iter().enumerate() {
        println!();
        println!("===== [{}/{}] 开始关闭 {} 端口上的项目 {}-{} =====", 
            index + 1, 
            installer_len, 
            installer.app_run_port, 
            installer.app_name, 
            installer.app_version);

        stop_jar(installer.app_run_port);
    }

    Ok(())
}

/// 停止运行 spring boot jar。
fn stop_jar(run_port: u32) {
    // 根据在 installer_config.toml 中登记的 spring boot jar 的运行端口来找到进程，并 kill 掉进程，
    // 以此来关闭 spring boot jar。
    match process::get_id(run_port) {
        Some(x) => {
            println!("> [INFO]: 端口 {} 运行在 {} 进程上", run_port, x);
            process::kill(x);
            println!("> [INFO]: 端口 {} 已关闭", run_port);
        }
        None => {
            println!("> [INFO]: 端口 {} 未使用", run_port);
        }
    }
}

/// 确认 JDK 是否已成功解压到 prod 文件夹。
/// 
/// 有两条检查路径，一是先检查下载文件夹，然后检查 prod 文件夹；
/// 二是先检查 prod 文件夹，然后检查下载文件夹。
/// 这里选用第一条检查路径。
fn ensure_jdk_exists(
    root_url: &str,
    jdk_name: &str,
    jdk_version: &str,
    jdk_file_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>>  {
    // 1. 检查 JDK 是否已下载
    let download_jdk_path = Path::new(config::ROOT_PATH_APP)
        .join(jdk_name)
        .join(jdk_version)
        .join(jdk_file_name);
    if !download_jdk_path.exists() {
        client::download(
            root_url,
            jdk_name,
            jdk_version,
            jdk_file_name);
    } else {
        println!("> [INFO]: 文件已存在");
    }

    // 2. 检查 prod 中是否有 JDK
    let prod_jdk_path = &get_prod_jdk_path(jdk_name, jdk_version);
    // 当已解压的文件夹已存在时，校验是否全部解压
    // 如果没有全部解压完成，则删除之前解压的文件，重新解压
    let zipping_status_file_name = &format!("{}_zipping", jdk_file_name);
    let zipping_status_path = &prod_jdk_path.parent().unwrap().join(zipping_status_file_name);
    if prod_jdk_path.exists() {
        if zipping_status_path.exists() {
            // 文件未完全解压完成
            fs::remove_dir_all(prod_jdk_path)?;
            fs::remove_file(zipping_status_path)?;
        } else {
            // 确认文件解压完成
            println!("> [INFO]: 文件已解压");
            return Ok(prod_jdk_path.to_path_buf());
        }
    }

    print!("> [INFO]: 正在解压 JDK...");
    // 强制输出
    io::stdout().flush()?;
    let started = Instant::now();
    // 文件解压前，创建一个标识解压状态的文件
    fs::create_dir_all(&prod_jdk_path.parent().unwrap())?;
    File::create(zipping_status_path)?;

    zip::unzip_to(
        download_jdk_path.to_str().unwrap(), 
        prod_jdk_path.parent().unwrap().to_str().unwrap()
    )?;

    // 文件解压完成后，删除标识解压状态的文件
    fs::remove_file(zipping_status_path)?;

    println!("完成！耗时 {}", HumanDuration(started.elapsed()));

    Ok(prod_jdk_path.to_path_buf())
}

/// 确认 Spring Boot Jar 是否已成功复制到 prod 文件夹。
/// 
/// 有两条检查路径，一是先检查下载文件夹，然后检查 prod 文件夹；
/// 二是先检查 prod 文件夹，然后检查下载文件夹。
/// 这里选用第一条检查路径。
fn ensure_spring_boot_jar_exists (
    root_url: &str,
    app_name: &str,
    app_version: &str,
    app_file_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    
    // 1. 检查 Spring Boot Jar 是否已下载
    let download_spring_boot_jar_path = Path::new(config::ROOT_PATH_APP)
        .join(app_name)
        .join(app_version)
        .join(app_file_name);
    if !download_spring_boot_jar_path.exists() {
        client::download(
            root_url,
            app_name,
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
use std::io;
use structopt::StructOpt;
use url::Url;
use url::ParseError::{EmptyHost};
use installer::command::{
        // installer 相关命令
        register_installer, 
        list_installers,
        unregister_single_installer,
        unregister_all_installers,
        // app 相关命令
        run_single_app, 
        run_all_apps,
        update_single_app,
        update_all_apps,
        stop_single_app,
        stop_all_apps};
use installer::installer_config::InstallerConfig;
use installer::util::process;

fn main() {
    let args = Cli::from_args();

    match args {
        // 支持多次调用 register 命令，最后的设置会覆盖之前的设置。
        Cli::Register => {
            ask_register_installer();
        },
        Cli::List => {
            ask_list_installers();
        },
        Cli::Unregister { port, all } => {
            if let Some(v) = port {
                ask_unregister_single_installer(v);
            } else if all {
                ask_unregister_all_installers();
            } else {
                println!("提示：请输入 --port <port> 选项注销单个 installer，或输入 --all 注销所有 installer。");
            }
        },
        Cli::Run { port, all } => {
            if let Some(v) = port {
                ask_run_single_app(v);
            } else if all {
                ask_run_all_apps();
            } else {
                println!("提示：请输入 --port <port> 选项运行单个 APP，或输入 --all 运行所有 APP。");
            }
        },
        Cli::Update { port, all } => {
            if let Some(v) = port {
                ask_update_single_app(v);
            } else if all {
                ask_update_all_apps();
            } else {
                println!("提示：请输入 --port <port> 选项升级单个 APP，或输入 --all 升级所有 APP。");
            }
        },
        Cli::Stop { port, all } => {
            if let Some(v) = port {
                ask_stop_single_app(v);
            } else if all {
                ask_stop_all_apps();
            } else {
                println!("提示：请输入 --port <port> 选项停止单个 APP，或输入 --all 停止所有 APP。");
            }
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "blocklang-installer", about = "Block Lang 安装程序")]
enum Cli {
    /// 将 installer 注册到 Block Lang 平台。
    #[structopt(name = "register")]
    Register,

    /// 显示所有在此服务器上注册的 installer。
    #[structopt(name = "list")]
    List,

    /// 从 Block Lang 平台注销 installer。
    #[structopt(name = "unregister")]
    Unregister {
        /// 根据指定的端口号定位到 installer，然后注销此 installer
        #[structopt(long = "port", short = "p")]
        port: Option<u32>,

        /// 注销配置文件中的所有 installer
        #[structopt(long = "all", short = "a")]
        all: bool,
    },

    /// 启动 Installer REST 服务(未实现)，并运行绑定的 Spring Boot jar。
    #[structopt(name = "run")]
    Run {
        /// 根据指定的端口号定位到 installer，然后运行此 installer
        #[structopt(long = "port", short = "p")]
        port: Option<u32>,

        /// 运行配置文件中的所有 installer
        #[structopt(long = "all", short = "a")]
        all: bool,        
    },

    /// 升级并运行最新版的 Spring Boot jar。
    #[structopt(name = "update")]
    Update {
        /// 根据指定的端口号定位到 installer，然后升级此 installer 管理的 APP
        #[structopt(long = "port", short = "p")]
        port: Option<u32>,

        /// 升级配置文件中的所有 installer 管理的所有 APP
        #[structopt(long = "all", short = "a")]
        all: bool,        
    },

    /// 停止 Installer Rest 服务，并停止运行 Spring Boot jar。
    #[structopt(name = "stop")]
    Stop {
        /// 根据指定的端口号定位到 installer，然后停止此 installer 管理的 APP
        #[structopt(long = "port", short = "p")]
        port: Option<u32>,

        /// 停止配置文件中的所有 installer 管理的所有 APP
        #[structopt(long = "all", short = "a")]
        all: bool,        
    },
}

fn ask_register_installer() {
    println!("开始往 Block Lang 平台注册主机：");
    println!("[1/3] 请输入 Block Lang 平台 URL(默认值为 https://blocklang.com)");
    let url: &str;
    let mut io_url; // 存储用户输入的值
    loop {
        io_url = String::new();
        io::stdin().read_line(&mut io_url).unwrap();
        io_url = io_url.trim().to_string();
        if io_url.is_empty() {
            io_url.push_str("https://blocklang.com");
        }

        match Url::parse(&io_url) {
            Ok(value) => {
                if value.scheme() != "http" && value.scheme() != "https" {
                    println!("> [ERROR]: URL 必须使用 http 或 https 协议，请重新输入 URL(默认值为 https://blocklang.com)：");  
                    continue;
                }
                
                url = &io_url;
                break;
            }
            Err(e) => {
                match e {
                    EmptyHost => {
                        println!("> [ERROR]: URL 不能为空，请重新输入 URL(默认值为 https://blocklang.com)：");
                    },
                    _ => {
                        println!("> [ERROR]: 无效的 URL，请重新输入 URL(默认值为 https://blocklang.com)：");
                    }
                }
                continue;
            }
        }
    }

    println!("[2/3] 请输入部署项目的注册 token");
    let mut token = String::new();
    io::stdin().read_line(&mut token).unwrap();
    token = token.trim().to_string();

    let installer_config = InstallerConfig::new();

    // 运行端口应该在部署时来定，跟发布无关，而是跟部署环境有关
    println!("[3/3] 请输入运行项目的端口号(默认为80)");
    let mut app_run_port: u32;
    loop {
        let mut in_app_run_port = String::new();
        io::stdin().read_line(&mut in_app_run_port).unwrap();
        in_app_run_port = in_app_run_port.trim().to_string();
        if in_app_run_port.is_empty() {
            in_app_run_port = "80".to_string();
        }

        // 校验是否能转换为数字
        match in_app_run_port.parse::<u32>() {
            Ok(value) => {
                app_run_port = value;
            },
            Err(_) => {
                println!("> [INFO]: 端口号只能由数字组成，请重新输入(默认为80)：", );
                continue;
            }
        };

        // 校验端口是否已被注册
        if let Some(installer) = installer_config.get_by_port(app_run_port) {
            println!("> [WARN]: {} 端口下已注册 {} 项目", installer.app_run_port, installer.app_name);
            println!("> [INFO]: 确定要在 {} 端口下重新注册项目，请：", installer.app_run_port);
            println!("> [INFO]: 1. 先执行 `blocklang-installer unregister --port {}` 命令注销", installer.app_run_port);
            println!("> [INFO]: 2. 再执行 `blocklang-installer register --port {}` 命令重新注册", installer.app_run_port);
            println!("> [INFO]: 按 CTRL + C 退出，或重新输入端口号(默认为80)：");
            continue;
        }

        // 如果端口未被 installer 注册，则再校验端口号是否被主机上其他应用占用
        if process::get_id(app_run_port).is_some() {
            // 端口被占用，则提醒用户
            println!("> [INFO]: 端口 {} 已被占用，请重新输入(默认为80)：", app_run_port);
            continue;
        }

        // 前面的校验都通过了，则跳出循环
        break;
    }
    
    // 输入完成后，开始注册
    if register_installer(&url, &token, app_run_port).is_ok() {
        if cfg!(target_os = "windows"){
            println!("注册成功，请执行 `blocklang-installer.exe run --port <port>` 命令运行项目。");
        } else if cfg!(target_os = "linux") {
            println!("注册成功，请执行 `./blocklang-installer run --port <port>` 命令运行项目。");
        }
    }

    // 出错后, 不打印任何内容
}

fn ask_list_installers() {
    match list_installers() {
        Ok(_) => {},
        Err(e) => {
            println!("查找 installer 清单时出错！{}", e);
        }
    }
}

fn ask_unregister_single_installer(app_run_port: u32) {
    match unregister_single_installer(app_run_port) {
        Ok(_) => {
            // 不做任何处理
        },
        Err(e) => {
            println!("注销单个 installer 失败！{}", e);
        },
    }
}

fn ask_unregister_all_installers() {
    match unregister_all_installers() {
        Ok(_) => {
            // 不做任何处理
        },
        Err(e) => {
            println!("注销所有 installer 失败！{}", e);
        },
    }
}

fn ask_run_single_app(app_run_port: u32) {
    match run_single_app(app_run_port) {
        Ok(_) => {
            // 不做任何处理
        },
        Err(e) => {
            println!("启动单个 APP 失败！{}", e);
        },
    }
}

fn ask_run_all_apps() {
    match run_all_apps() {
        Ok(_) => {
            // 不做任何处理
        },
        Err(e) => {
            println!("启动所有 APP 失败！{}", e);
        },
    }
}

fn ask_update_single_app(app_run_port: u32) {
    match update_single_app(app_run_port) {
        Ok(_) => {
            // 不做任何处理
        },
        Err(e) => {
            println!("升级项目失败！{}", e);
        },
    }
}

fn ask_update_all_apps() {
    match update_all_apps() {
        Ok(_) => {
            // 不做任何处理
        },
        Err(e) => {
            println!("升级所有 APP 失败！{}", e);
        },
    }
}

fn ask_stop_single_app(app_run_port: u32) {
    match stop_single_app(app_run_port) {
        Ok(_) => {
            // 不做任何处理
        },
        Err(e) => {
            println!("停止单个 APP 失败！{}", e);
        },
    }
}

fn ask_stop_all_apps() {
    match stop_all_apps() {
        Ok(_) => {
            // 不做任何处理
        },
        Err(e) => {
            println!("停止所有 APP 失败！{}", e);
        },
    }
}

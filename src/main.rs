use std::io;
use structopt::StructOpt;
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
                println!("提示：请输入 --port <port> 选项注销单个 installer，或输入 --all 注销所有 installer。", )
            }
        },
        Cli::Run { port, all } => {
            if let Some(v) = port {
                ask_run_single_app(v);
            } else if all {
                ask_run_all_apps();
            } else {
                println!("提示：请输入 --port <port> 选项运行单个 APP，或输入 --all 运行所有 APP。", )
            }
        },
        Cli::Update { port, all } => {
            if let Some(v) = port {
                ask_update_single_app(v);
            } else if all {
                ask_update_all_apps();
            } else {
                println!("提示：请输入 --port <port> 选项升级单个 APP，或输入 --all 升级所有 APP。", )
            }
        },
        Cli::Stop { port, all } => {
            if let Some(v) = port {
                ask_stop_single_app(v);
            } else if all {
                ask_stop_all_apps();
            } else {
                println!("提示：请输入 --port <port> 选项停止单个 APP，或输入 --all 停止所有 APP。", )
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
    println!("请输入 Block Lang 平台 URL(默认值为 https://blocklang.com)");
    let mut url = String::new();
    io::stdin().read_line(&mut url).unwrap();
    url = url.trim().to_string();
    if url.is_empty() {
        url.push_str("https://blocklang.com");
    }

    println!("请输入待绑定项目的注册 token");
    let mut token = String::new();
    io::stdin().read_line(&mut token).unwrap();
    token = token.trim().to_string();

    // 运行端口应该在部署时来定，跟发布无关，而是跟部署环境有关
    println!("请输入运行 APP 的端口号(默认为80)");
    let mut app_run_port = String::new();
    io::stdin().read_line(&mut app_run_port).unwrap();
    app_run_port = app_run_port.trim().to_string();
    if app_run_port.is_empty() {
        app_run_port = "80".to_string();
    }
    let app_run_port = app_run_port.parse::<u32>().unwrap();

    // 输入完成后，开始注册
    match register_installer(&url, &token, app_run_port) {
        Ok(_) => {
            println!("注册成功，请执行 `blocklang-installer run` 命名运行 APP。");
        },
        Err(e) => {
            println!("注册失败！{}", e);
        },
    }
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
            println!("注销成功，{} 端口上运行的 APP 已关闭。", app_run_port);
        },
        Err(e) => {
            println!("注销单个 installer 失败！{}", e);
        },
    }
}

fn ask_unregister_all_installers() {
    match unregister_all_installers() {
        Ok(_) => {
            println!("注销成功，所有运行的 APP 已关闭。");
        },
        Err(e) => {
            println!("注销所有 installer 失败！{}", e);
        },
    }
}

fn ask_run_single_app(app_run_port: u32) {
    match run_single_app(app_run_port) {
        Ok(_) => {
            println!("启动成功，{} 端口上运行的 APP 正在运行。", app_run_port);
        },
        Err(e) => {
            println!("启动单个 APP 失败！{}", e);
        },
    }
}

fn ask_run_all_apps() {
    match run_all_apps() {
        Ok(_) => {
            println!("启动成功，所有 APP 正在运行。");
        },
        Err(e) => {
            println!("启动所有 APP 失败！{}", e);
        },
    }
}

fn ask_update_single_app(app_run_port: u32) {
    match update_single_app(app_run_port) {
        Ok(_) => {
            println!("升级单个 APP 成功。");
        },
        Err(e) => {
            println!("升级单个 APP 失败！{}", e);
        },
    }
}

fn ask_update_all_apps() {
    match update_all_apps() {
        Ok(_) => {
            println!("升级所有 APP 成功。");
        },
        Err(e) => {
            println!("升级所有 APP 失败！{}", e);
        },
    }
}

fn ask_stop_single_app(app_run_port: u32) {
    match stop_single_app(app_run_port) {
        Ok(_) => {
            println!("停止单个 APP 成功。");
        },
        Err(e) => {
            println!("停止单个 APP 失败！{}", e);
        },
    }
}

fn ask_stop_all_apps() {
    match stop_all_apps() {
        Ok(_) => {
            println!("停止所有 APP 成功。");
        },
        Err(e) => {
            println!("停止所有 APP 失败！{}", e);
        },
    }
}

use std::io;
use structopt::StructOpt;
use installer::command::{register, start, update, stop};

fn main() {
    let args = Cli::from_args();

    match args {
        // 支持多次调用 register 命令，最后的设置会覆盖之前的设置。
        Cli::Register => {
            ask_register_installer();
        },
        Cli::Start => {
            ask_install();
        },
        Cli::Update => {
            ask_update();
        },
        Cli::Stop => {
            ask_stop();
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "blocklang-installer", about = "Block Lang 安装程序")]
enum Cli {
    /// 将应用服务器注册到 Block Lang 平台。
    #[structopt(name = "register")]
    Register,

    /// 启动 Installer REST 服务，并运行绑定的 Spring Boot jar。
    #[structopt(name = "start")]
    Start,

    /// 安装并运行最新版的 Spring Boot jar。
    #[structopt(name = "update")]
    Update,

    /// 停止 Installer Rest 服务，并停止运行 Spring Boot jar。
    #[structopt(name = "stop")]
    Stop,
}

fn ask_register_installer() {
    println!("请输入 Block Lang 平台 URL(默认值为 https://blocklang.com)");
    let mut url = String::new();
    io::stdin().read_line(&mut url).unwrap();
    url = url.trim().to_string();
    if url.is_empty() {
        url.push_str("https://blocklang.com");
    }

    println!("请输入待绑定项目的 token");
    let mut token = String::new();
    io::stdin().read_line(&mut token).unwrap();
    token = token.trim().to_string();

    // 输入完成后，开始注册
    match register(&url, &token) {
        Ok(_) => {
            println!("注册成功，请执行 `blocklang-installer start` 命名启动 BlockLang Installer。");
        },
        Err(e) => {
            println!("注册失败！{}", e);
        },
    }
}

fn ask_install() {
    match start() {
        Ok(_) => {
            println!("启动成功，Spring boot jar 项目已运行。");
        },
        Err(e) => {
            println!("启动失败！{}", e);
        },
    }
}

fn ask_update() {
    match update() {
        Ok(_) => {
            println!("更新成功，新版的 Spring boot jar 项目已运行。");
        },
        Err(e) => {
            println!("更新失败！{}", e);
        },
    }
}

fn ask_stop() {
    match stop() {
        Ok(_) => {
            println!("已成功停止，Spring boot jar 项目已停止运行。");
        },
        Err(e) => {
            println!("停止失败！{}", e);
        },
    }
}
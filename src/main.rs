use std::io;
use structopt::StructOpt;
use installer::register;

fn main() {
    let args = Cli::from_args();

    match args {
        // 支持多次调用 register 命令，最后的设置会覆盖之前的设置。
        Cli::Register => {
            ask_register_installer();
        },
        Cli::Update => {
            println!("{}", "更新成功");
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "blocklang-installer", about = "Block Lang 安装程序")]
enum Cli {
    /// 将应用服务器注册到 Block Lang 平台。
    #[structopt(name = "register")]
    Register,

    /// 从软件中心更新软件，并安装到应用服务器上。
    #[structopt(name = "update")]
    Update,
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

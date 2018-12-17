use structopt::StructOpt;

fn main() {
    let args = Cli::from_args();

    match args {
        Cli::Update => {
            println!("{}", "更新成功");
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "installer", about = "Block Lang 安装程序")]
enum Cli {
    
    /// 从软件中心更新软件，并安装到应用服务器上。
    #[structopt(name = "update")]
    Update,
}

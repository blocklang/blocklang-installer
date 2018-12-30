use std::path::{Path};
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::process::{Command, Child};
use std::collections::HashMap;
use reqwest;
use zip::ZipArchive;
use serde_derive::{Deserialize, Serialize};
use toml;

mod util;

/// 注册命令
pub fn register(url: &str, token: &str) -> Result<(), Box<std::error::Error>> {
    // 向 Block Lang 平台发送注册请求
    let installer_info = register_installer(url, token)?;
    // 存储安装信息
    save_config(installer_info);

    Ok(())
}

/// 启动命令
/// 
/// 在启动时会使用 `config.toml` 中的 `software_name` 和 `software_version` 等信息
/// 在 `prod` 文件夹下检查 Spring boot jar 和 JDK 文件是否已存在，如果不存在则先下载。
/// 下载并解压成功后，启动 Spring Boot jar。
pub fn start() -> Result<(), Box<std::error::Error>> {
    let config = read_config()?;
    let installers = config.installers.unwrap();
    assert!(installers.len() < 1, "没有找到 installer。请先执行 `blocklang-installer register` 注册 installer");

    // 当前版本只支持一个服务器上配置一个 installer。
    let first_installer = installers.get(0).unwrap();

    // 有两条检查路径，一是先检查下载文件夹，然后检查 prod 文件夹；
    // 二是先检查 prod 文件夹，然后检查下载文件夹。
    // 这里选用第一条检查路径。

    // 检查 Spring Boot Jar
    // 1. 检查 Spring Boot Jar 是否已下载
    let download_spring_boot_jar_path = Path::new(ROOT_PATH_SOFTWARE)
        .join(&first_installer.software_name)
        .join(&first_installer.software_version)
        .join(&first_installer.software_file_name);
    if !download_spring_boot_jar_path.exists() {
        download(&first_installer.software_name,
                 &first_installer.software_version,
                 &first_installer.software_file_name);
    }
    // 2. 检查 prod 下是否有 Spring Boot Jar
    let prod_spring_boot_jar_path = Path::new(ROOT_PATH_PROD)
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
    let download_jdk_path = Path::new(ROOT_PATH_SOFTWARE)
        .join(&first_installer.jdk_name)
        .join(&first_installer.jdk_version)
        .join(&first_installer.jdk_file_name);
    if !download_jdk_path.exists() {
        download(&first_installer.jdk_name,
                 &first_installer.jdk_version,
                 &first_installer.jdk_file_name);
    }
    // 2. 检查 prod 中是否有 JDK
    let prod_jdk_path = Path::new(ROOT_PATH_PROD)
        .join(&first_installer.jdk_name)
        .join(&first_installer.jdk_version)
        // 注意，因为 jdk 的命名规范是 jdk-11.0.1
        .join(format!("jdk-{}", first_installer.jdk_version));
    if !prod_jdk_path.exists() {
        unzip_to(download_jdk_path.to_str().unwrap(), 
                 prod_jdk_path.parent().unwrap().to_str().unwrap())
                 .expect("解压 JDK 出错");
    }

    // 运行 Spring Boot Jar
    run_spring_boot_jar(prod_spring_boot_jar_path.to_str().unwrap(), 
                        prod_jdk_path.to_str().unwrap());

    Ok(())
}


#[cfg(test)]
use mockito;

#[cfg(not(test))]
const URL: &str = "https://www.blocklang.com";

#[cfg(test)]
const URL: &str = mockito::SERVER_URL;

const ROOT_PATH_SOFTWARE: &str = "softwares";
const ROOT_PATH_PROD: &str = "prod";
const CONFIG_FILE_NAME: &str = "config.toml";

/// 软件安装信息
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct InstallerInfo {
    token: String,
    software_name: String,
    software_version: String,
    software_file_name: String,
    software_run_port: u32,
    jdk_name: String,
    jdk_version: String,
    jdk_file_name: String,
}

/// 使用 Block Lang 提供的部署 token，向 Block Lang 平台注册部署服务器信息。
/// 
/// Block Lang 和部署服务器之间是通过 token 建立连接的。
/// 
/// 注意：连接建立后，Block Lang 平台默认打开连接，但是如果遇到盗用 token 的情况，
/// 可以在 Block Lang 平台关闭该连接。
fn register_installer(url: &str, token: &str) -> Result<InstallerInfo, Box<std::error::Error>> {
    let url = &format!("{}/installers", url);

    let mut json_data = HashMap::new();
    let interface_addr = util::get_interface_address().expect("获取不到能联网的有线网络");
    
    json_data.insert("token", token);
    json_data.insert("serverToken", &interface_addr.mac_address);
    json_data.insert("ip", &interface_addr.ip_address);
    // TODO: 设置以下参数
    // json_data.insert("port", "");
    // json_data.insert("platform_name", "");
    // json_data.insert("platform_version", "");
    // json_data.insert("architecture", "");
    // println!("{:?}", json_data);

    let client = reqwest::Client::new();
    let mut response = client.post(url)
        .json(&json_data)
        .send()?;

    let result: InstallerInfo = response.json()?;

    Ok(result)
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    installers: Option<Vec<InstallerConfig>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InstallerConfig {
    token: String,
    server_token: String,
    software_name: String,
    software_version: String,
    software_file_name: String,
    software_run_port: u32,
    jdk_name: String,
    jdk_version: String,
    jdk_file_name: String,
}

/// 将 Installer 信息存储在 config.toml 文件中。
fn save_config(installer_info: InstallerInfo) {
    let interface_addr = util::get_interface_address().expect("获取不到能联网的有线网络");
    let server_token = interface_addr.mac_address;
    // 设置配置信息
    let config = Config {
        installers: Some(vec!(InstallerConfig {
            token: installer_info.token,
            server_token: server_token,
            software_name: installer_info.software_name,
            software_version: installer_info.software_version,
            software_file_name: installer_info.software_file_name,
            software_run_port: installer_info.software_run_port,
            jdk_name: installer_info.jdk_name,
            jdk_version: installer_info.jdk_version,
            jdk_file_name: installer_info.jdk_file_name,
        })),
    };
    let toml_content = toml::to_vec(&config).unwrap();

    // 在 config.toml 文件中存储配置信息
    let mut file = File::create(CONFIG_FILE_NAME).expect("failed to create config.toml file");
    file.write_all(toml_content.as_slice()).expect("failed to save config.toml content");
}

/// 从 config.toml 文件中读取 Installer 信息。
fn read_config() -> Result<Config, Box<std::error::Error>> {
    let mut file = File::open(CONFIG_FILE_NAME)?;
    // TODO: 如何修改默认的提示信息，并能往外传递，如果使用 expect 的话，就地退出了，并没有传到 main 函数中。
    // .expect(&format!("找不到 {} 文件，请先执行 register 命令，注册一个 installer", CONFIG_FILE_NAME));
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

/// 从软件中心下载软件。
/// 
/// `download` 函数将根据 `software_name` 指定的软件名，
/// `software_version` 指定的软件版本号，从软件发布中心下载软件。
/// 然后将下载的软件存到应用服务器指定的目录中，并将文件名设置为 `software_file_name`。
/// 
/// 如果在指定的文件夹下找到对应的文件，则中断下载，直接使用已存在文件。
/// 
/// 下载完成后，会返回新下载文件的完整路径。
/// 
/// 应用服务器的目录结构为
/// 
/// * softwares
///     * software_name
///         * software_version
///             * software_file_name
/// 
/// # Examples
/// 
/// ```no_run
/// use installer::download;
/// 
/// fn main() {
///     download("app", "0.1.0", "app-0.1.0.zip").unwrap();
/// }
/// ```
pub fn download(software_name: &str, 
    software_version: &str, 
    software_file_name: &str) -> Option<String> {
    
    let saved_dir_path = &format!("{}/{}/{}", 
        ROOT_PATH_SOFTWARE, 
        software_name, 
        software_version);

    fs::create_dir_all(saved_dir_path).expect("在创建存储下载文件的目录结构时出错");

    let saved_file_path = &format!("{}/{}", saved_dir_path, software_file_name);

    let path = Path::new(saved_file_path);
    // 如果文件已存在，则直接返回文件名
    if path.exists() {
        return Some(saved_file_path.to_string());
    }

    println!("开始下载文件：{}", software_file_name);

    let os = util::get_target_os().expect("不支持的操作系统");
    let url = &format!("{}/softwares?name={}&version={}&os={}", 
        URL, 
        software_name, 
        software_version,
        os);

    let mut response = reqwest::get(url).unwrap();

    if !response.status().is_success() {
        println!("下载失败，出现了其他错误，状态码为：{:?}", response.status());
        return None;
    }

    println!("返回成功，开始在本地写入文件");
    let mut file = File::create(saved_file_path).unwrap();
    response.copy_to(&mut file).unwrap();
    println!("下载完成。");
    Some(saved_file_path.to_string())
}

/// 将 `source_file_path` 的压缩文件解压到 `target_dir_path` 目录下。
/// 
/// # Examples
/// 
/// ```no_run
/// use installer::unzip_to;
/// 
/// fn main() -> Result<(), Box<std::error::Error>> {
///     unzip_to("test.zip", "another/folder")?;
///     Ok(())
/// }
/// ```
pub fn unzip_to(source_file_path: &str, target_dir_path: &str) -> Result<(), Box<std::error::Error>> {
    let source_path = Path::new(source_file_path);

    let file_name = source_path.file_name().unwrap().to_str().unwrap();
    let target_path = Path::new(target_dir_path).join(file_name);

    let is_in_same_dir = source_path == target_path;

    // 如果源目录跟目标目录相同，则不复制
    if !is_in_same_dir {
        // 将压缩文件复制到指定的目录
        fs::create_dir_all(target_dir_path)?;
        fs::copy(source_path, &target_path)?;
    }

    // 解压文件
    unzip_file(target_path.to_str().unwrap())?;

    // 删除目标文件夹中的压缩文件
    if !is_in_same_dir {
        fs::remove_file(target_path)?;
    }

    Ok(())
}

/// 将压缩文件解压到当前目录，即存放压缩文件的目录中。
/// 
/// 注意：解压完成后，并不会删除之前的压缩文件 `source_file_path`
fn unzip_file(source_file_path: &str) -> Result<(), Box<std::error::Error>> {
    let source_file = File::open(source_file_path)?;
    let source_reader = BufReader::new(source_file);
    let mut archive = ZipArchive::new(source_reader)?;

    // 获取被压缩文件所在的文件夹
    let parent_dir = Path::new(source_file_path).parent().unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = parent_dir.join(&file.sanitized_name());

        if (&*file.name()).ends_with('/') {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(p) = out_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }
            let mut out_file = fs::File::create(&out_path)?;
            io::copy(&mut file, &mut out_file)?;
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&out_path, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}

/// 在后台运行 Spring boot jar 文件，并返回进程
/// 
/// 默认将 JDK 与要运行的 spring boot jar 放在同一个文件夹中,
/// 其中 `jar_file_path` 指 spring boot jar 的存放路径，
/// `jdk_path` 指 jdk 的存放路径。
/// 
/// Examples
/// 
/// ```no_run
/// use installer::run_spring_boot_jar;
/// 
/// fn main() {
///     run_spring_boot_jar("prod/app1/demo-0.0.1-SNAPSHOT.jar", "prod/app1/temp/jdk-11.0.1");
/// }
/// ```
pub fn run_spring_boot_jar(
    jar_file_path: &str, 
    jdk_path: &str) -> Child {

    let child = if cfg!(target_os = "windows") {
        // 注意，在 windows 操作系统中，使用 `javaw`，不使用 `java`
        // 因为 `java` 会在命令行中启动新的命令行，因此无法直接跟踪到
        // 使用 `java` 命令运行的 jar。而 `javaw` 正是为解决此类问题诞生的。
        // 而在 linux 环境下， `java` 命令不会启动一个新的命令行，
        // 所以在 linux 环境下，并不存在 `javaw` 命令。
        // 详见 https://stackoverflow.com/questions/14331406/why-javaw-is-not-found-on-my-java-installation-on-ubuntu
        Command::new("javaw")
            .env("PATH", Path::new(jdk_path).join("bin"))
            .arg("-jar")
            .arg(jar_file_path)
            .spawn()
            .expect("failed to run javaw -jar")
    } else {
        // 设置权限，初次运行时会提示没有权限
        let java_executable_path = Path::new(jdk_path).join("bin").join("java");
        let java_executable_path = java_executable_path
            .to_str()
            .expect("parse jdk/bin/java path error");

	    // 为 jdk/bin/java 设置可执行权限
        set_executable_permission(java_executable_path)
            .expect("failed to set jdk/bin/java permission");
        // 为 spring boot jar 设置可执行权限
        set_executable_permission(jar_file_path)
            .expect("failed to set sping boot jar permission");
	
        // linux 下直接运行 java -jar 就是生成一个新的后台进程
        // 当关闭 installer 进程后，运行 java -jar 的进程依然存在
        Command::new("java")
            .env("PATH", Path::new(jdk_path).join("bin"))
            .arg("-jar")
            .arg(jar_file_path)
            .spawn()
            .expect("failed to run java -jar")
    };

    child
}

// 在 linux 等操作系统下，让文件具有可执行权限。
fn set_executable_permission(path: &str) -> std::io::Result<()> {
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_readonly(false);
    fs::set_permissions(path, perms)?;

    Ok(())
}

/// 停止运行 spring boot jar
/// 
/// 这里是通过直接杀死进程来停止 spring boot 项目的。
/// 
/// Examples
/// 
/// ```no_run
/// use installer::run_spring_boot_jar;
/// use installer::stop_spring_boot_jar;
/// 
/// fn main() {
///     let mut process = run_spring_boot_jar("prod/app1/demo-0.0.1-SNAPSHOT.jar", "prod/app1/temp/jdk-11.0.1");
///     stop_spring_boot_jar(&mut process);
/// }
/// ```
pub fn stop_spring_boot_jar(process: &mut Child) -> std::io::Result<()> {
    process.kill()
}

#[cfg(test)]
mod tests {
    
    use std::io::Write;
    use std::io::prelude::*;
    use std::fs::{self, File};
    use std::path::Path;
    use tempfile::NamedTempFile;
    use mockito::mock;
    use zip::CompressionMethod::Stored;
    use zip::result::{ZipResult};
    use zip::write::{ZipWriter, FileOptions};
    use toml;
    use super::{register_installer, 
                download, 
                unzip_to, 
                save_config,
                InstallerInfo,
                Config,
                URL,
                CONFIG_FILE_NAME,
                ROOT_PATH_SOFTWARE};

    const TEMP_FILE_NAME: &str = "hello_world.txt";

    #[test]
    fn register_installer_success() -> Result<(), Box<std::error::Error>> {
        // 模拟一个 installers POST 服务
        let mock = mock("POST", "/installers")
            .with_body(r#"{
                            "token": "1", 
                            "softwareName": "2", 
                            "softwareVersion": "3",
                            "softwareFileName": "4",
                            "softwareRunPort": 5,
                            "jdkName": "6", 
                            "jdkVersion": "7",
                            "jdkFileName": "8"
                        }"#)
            .with_status(201)
            .create();

        // 请求 installers 服务
        let installer_info = register_installer(URL, "1")?;
        println!("{:#?}", installer_info);
        // 断言返回的结果
        assert_eq!("1", installer_info.token);
        assert_eq!("2", installer_info.software_name);
        assert_eq!("3", installer_info.software_version);
        assert_eq!("4", installer_info.software_file_name);
        assert_eq!(5, installer_info.software_run_port);
        assert_eq!("6", installer_info.jdk_name);
        assert_eq!("7", installer_info.jdk_version);
        assert_eq!("8", installer_info.jdk_file_name);

        // 断言已执行过 mock 的 http 服务
        mock.assert();

        Ok(())
    }

    #[test]
    fn save_config_success() -> Result<(), Box<std::error::Error>> {
        let installer_info = InstallerInfo {
            token: "1".to_string(),
            software_name: "2".to_string(),
            software_version: "3".to_string(),
            software_file_name: "4".to_string(),
            software_run_port: 5_u32,
            jdk_name: "6".to_string(),
            jdk_version: "7".to_string(),
            jdk_file_name: "8".to_string(),
        };
        save_config(installer_info);

        // 断言存在 config.toml 文件
        assert!(Path::new(CONFIG_FILE_NAME).exists());
        // 读取文件中的内容，并比较部分内容
        let mut file = File::open(CONFIG_FILE_NAME)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        assert!(buffer.contains("[[installers]]"));

        // 删除 config.toml 文件
        fs::remove_file(CONFIG_FILE_NAME)?;

        Ok(())
    }

    // 当前只支持配置一个 installers，所以如果多次保存，则只存储最后一个配置信息。
    #[test]
    fn save_config_twice() -> Result<(), Box<std::error::Error>> {
        let installer_info_1 = InstallerInfo {
            token: "1".to_string(),
            software_name: "2".to_string(),
            software_version: "3".to_string(),
            software_file_name: "4".to_string(),
            software_run_port: 5_u32,
            jdk_name: "6".to_string(),
            jdk_version: "7".to_string(),
            jdk_file_name: "8".to_string(),
        };
        let installer_info_2 = InstallerInfo {
            token: "a".to_string(),
            software_name: "b".to_string(),
            software_version: "c".to_string(),
            software_file_name: "d".to_string(),
            software_run_port: 55_u32,
            jdk_name: "e".to_string(),
            jdk_version: "f".to_string(),
            jdk_file_name: "g".to_string(),
        };

        save_config(installer_info_1);
        save_config(installer_info_2);

        // 断言存在 config.toml 文件
        assert!(Path::new(CONFIG_FILE_NAME).exists());
        
        // 读取文件中的内容，并比较部分内容
        let mut file = File::open(CONFIG_FILE_NAME)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        
        let config: Config = toml::from_str(buffer.as_str()).unwrap();

        let installers = config.installers;
        let installers = installers.unwrap();
        assert_eq!(1, installers.len());
        assert_eq!("a", installers.get(0).unwrap().token);
        assert_eq!(55, installers.get(0).unwrap().software_run_port);

        // 删除 config.toml 文件
        fs::remove_file(CONFIG_FILE_NAME)?;

        Ok(())
    }

    #[test]
    fn download_fail() {
        assert_eq!(None, download("app", "0.1.0", "app-0.1.0.zip"));
    }

    #[test]
    fn download_success() -> Result<(), Box<std::error::Error>> {
        // 创建一个临时文件，当作下载文件
        let mut file = NamedTempFile::new()?;
        writeln!(file, "I am a software!")?;
        let path = file.path();
        let path = path.to_str().unwrap();

        // mock 下载文件的 http 服务
        let mock = mock("GET", "/softwares?name=app&version=0.1.0&os=windows")
            .with_body_from_file(path)
            .with_status(200)
            .create();
        
        {
            // 执行下载文件方法
            let downloaded_file_path = download("app", "0.1.0", "app-0.1.0.zip").unwrap();

            // 断言文件已下载成功
            assert!(Path::new(&downloaded_file_path).exists());

            // 删除已下载的文件
            fs::remove_dir_all(ROOT_PATH_SOFTWARE)?;
        }

        // 断言已执行过 mock 的 http 服务
        mock.assert();

        Ok(())
    }

    #[test]
    fn unzip_to_success() -> Result<(), Box<std::error::Error>> {
        let zip_file_name = "test.zip";
        // 生成一个 zip 文件
        generate_zip_file(zip_file_name)?;
        // 将文件 test.zip 解压到 test_folder/ 文件夹下
        let target_dir = "test_folder";
        unzip_to(zip_file_name, target_dir)?;

        // 如果不将以下代码放在单独放在一个作用域中，
        // 在执行 `fs::remove_dir_all(target_dir)?;` 时
        // 总是会报“目录不为空”的错误，但实际上已经将目录中的文件删除了
        {
            // 断言文件解压成功
            let unzip_file_path = Path::new(target_dir).join(TEMP_FILE_NAME);
            assert!(unzip_file_path.exists());
            // 读取文件的内容，断言内容为“Hello, World!”
            let mut unzip_file = File::open(&unzip_file_path)?;
            let mut unzip_file_content = String::new();
            unzip_file.read_to_string(&mut unzip_file_content)?;
            assert_eq!(unzip_file_content, "Hello, World!");
        }
        
        // 删除 test.zip 文件
        fs::remove_file(zip_file_name)?;
        // 删除 test_folder 目录
        fs::remove_dir_all(target_dir)?;
        Ok(())
    }

    fn generate_zip_file(zip_file_name: &str) -> ZipResult<()> {
        //  1. 生成一个临时文件
        //  2. 将临时文件压缩成 zip
        let file = File::create(zip_file_name)?;
        let mut zip = ZipWriter::new(file);

        let options = FileOptions::default().compression_method(Stored);
        zip.start_file(TEMP_FILE_NAME, options)?;
        zip.write_all(b"Hello, World!")?;

        zip.finish()?;
        Ok(())
    }

}

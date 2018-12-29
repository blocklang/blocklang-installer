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


#[cfg(test)]
use mockito;

#[cfg(not(test))]
const URL: &str = "https://www.blocklang.com";

#[cfg(test)]
const URL: &str = mockito::SERVER_URL;

const ROOT_PATH_SOFTWARE: &str = "softwares";

/// 软件安装信息
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct InstallerInfo {
    token: String,
    software_name: String,
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
    jdk_name: String,
    jdk_version: String,
    jdk_file_name: String,
}

/// 将 installer 信息存储在 config.toml 文件中。
fn save_config(installer_info: InstallerInfo) {
    // 设置配置信息
    let config = Config {
        installers: Some(vec!(InstallerConfig {
            token: installer_info.token,
            server_token: "xx".to_string(),
            software_name: installer_info.software_name,
            jdk_name: installer_info.jdk_name,
            jdk_version: installer_info.jdk_version,
            jdk_file_name: installer_info.jdk_file_name,
        })),
    };
    let toml_content = toml::to_vec(&config).unwrap();

    // 在 config.toml 文件中存储配置信息
    let mut file = File::create("config.toml").expect("failed to create config.toml file");
    file.write_all(toml_content.as_slice()).expect("failed to save config.toml content");
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

    let url = &format!("{}/softwares?name={}&version={}", 
        URL, 
        software_name, 
        software_version);
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
                ROOT_PATH_SOFTWARE};

    const TEMP_FILE_NAME: &str = "hello_world.txt";

    #[test]
    fn register_installer_success() -> Result<(), Box<std::error::Error>> {
        // 模拟一个 installers POST 服务
        let mock = mock("POST", "/installers")
            .with_body(r#"{
                            "token": "1", 
                            "softwareName": "2", 
                            "jdkName": "3", 
                            "jdkVersion": "4",
                            "jdkFileName": "5"
                        }"#)
            .with_status(201)
            .create();

        // 请求 installers 服务
        let installer_info = register_installer(URL, "1")?;
        println!("{:#?}", installer_info);
        // 断言返回的结果
        assert_eq!("1", installer_info.token);
        assert_eq!("2", installer_info.software_name);
        assert_eq!("3", installer_info.jdk_name);
        assert_eq!("4", installer_info.jdk_version);
        assert_eq!("5", installer_info.jdk_file_name);

        // 断言已执行过 mock 的 http 服务
        mock.assert();

        Ok(())
    }

    #[test]
    fn save_config_success() -> Result<(), Box<std::error::Error>> {
        let installer_info = InstallerInfo {
            token: "1".to_string(),
            software_name: "2".to_string(),
            jdk_name: "3".to_string(),
            jdk_version: "4".to_string(),
            jdk_file_name: "5".to_string(),
        };
        save_config(installer_info);

        // 断言存在 config.toml 文件
        assert!(Path::new("config.toml").exists());
        // 读取文件中的内容，并比较部分内容
        let mut file = File::open("config.toml")?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        assert!(buffer.contains("[[installers]]"));

        // 删除 config.toml 文件
        fs::remove_file("config.toml")?;

        Ok(())
    }

    // 当前只支持配置一个 installers，所以如果多次保存，则只存储最后一个配置信息。
    #[test]
    fn save_config_twice() -> Result<(), Box<std::error::Error>> {
        let installer_info_1 = InstallerInfo {
            token: "1".to_string(),
            software_name: "2".to_string(),
            jdk_name: "3".to_string(),
            jdk_version: "4".to_string(),
            jdk_file_name: "5".to_string(),
        };
        let installer_info_2 = InstallerInfo {
            token: "a".to_string(),
            software_name: "b".to_string(),
            jdk_name: "c".to_string(),
            jdk_version: "d".to_string(),
            jdk_file_name: "e".to_string(),
        };

        save_config(installer_info_1);
        save_config(installer_info_2);

        // 断言存在 config.toml 文件
        assert!(Path::new("config.toml").exists());
        
        // 读取文件中的内容，并比较部分内容
        let mut file = File::open("config.toml")?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        
        let config: Config = toml::from_str(buffer.as_str()).unwrap();

        let installers = config.installers;
        let installers = installers.unwrap();
        assert_eq!(1, installers.len());
        assert_eq!("a", installers.get(0).unwrap().token);

        // 删除 config.toml 文件
        fs::remove_file("config.toml")?;

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
        let mock = mock("GET", "/softwares?name=app&version=0.1.0")
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

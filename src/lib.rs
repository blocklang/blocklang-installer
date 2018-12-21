use reqwest;
use std::path::Path;
use std::fs::{self, File};

#[cfg(test)]
use mockito;

#[cfg(not(test))]
const URL: &str = "https://www.blocklang.com";

#[cfg(test)]
const URL: &str = mockito::SERVER_URL;

const ROOT_PATH_SOFTWARE: &str = "softwares";

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
/// ```
/// fn main() -> Result<String, Box<std::error::Error>> {
///     let downloaded_file_path = download("app", "0.1.0", "app-0.1.0.zip")?;
///     Ok(downloaded_file_path)
/// }
/// ```
pub fn download(software_name: &str, 
    software_version: &str, 
    software_file_name: &str) -> Result<String, Box<std::error::Error>> {
    
    let saved_dir_path = &format!("{}/{}/{}", 
        ROOT_PATH_SOFTWARE, 
        software_name, 
        software_version);

    fs::create_dir_all(saved_dir_path)?;

    let saved_file_path = &format!("{}/{}", saved_dir_path, software_file_name);

    let path = Path::new(saved_file_path);
    // 如果文件已存在，则直接返回文件名
    if path.exists() {
        return Ok(saved_file_path.to_string());
    }

    println!("开始下载文件：{}", software_file_name);

    let url = &format!("{}/softwares?name={}&version={}", 
        URL, 
        software_name, 
        software_version);
    let mut response = reqwest::get(url)?;

    if response.status().is_success() {
        println!("返回成功，开始在本地写入文件");
        let mut file = File::create(saved_file_path)?;
        response.copy_to(&mut file)?;
        println!("下载完成。");
    } else {
        println!("出现了其他错误，状态码为：{:?}", response.status());
    }

    Ok(saved_file_path.to_string())
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;
    use std::io::Write;
    use mockito::mock;
    use std::fs;
    use std::path::Path;
    use super::{download, ROOT_PATH_SOFTWARE};

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
            let downloaded_file_path = download("app", "0.1.0", "app-0.1.0.zip")?;

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
    #[should_panic]
    fn download_server_not_work() {
        match download("app", "0.1.0", "app-0.1.0.zip") {
            Err(why) => panic!("{:?}", why),
            _ => (),
        };
    }
}

use std::path::Path;
use std::fs::{self, File};
use std::io::{self, BufReader};
use zip::ZipArchive;

/// 将 `source_file_path` 的压缩文件解压到 `target_dir_path` 目录下。
/// 
/// # Examples
/// 
/// ```no_run
/// use installer::util::zip::unzip_to;
/// 
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     unzip_to("test.zip", "another/folder")?;
///     Ok(())
/// }
/// ```
pub fn unzip_to(source_file_path: &str, target_dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
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
fn unzip_file(source_file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
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

#[cfg(test)]
mod tests {

    use std::fs::{self, File};
    use std::path::Path;
    use std::io::prelude::*;
    use zip::result::{ZipResult};
    use zip::write::{ZipWriter, FileOptions};
    use zip::CompressionMethod::Stored;
    use super::unzip_to;

    const TEMP_FILE_NAME: &str = "hello_world.txt";

    #[test]
    fn unzip_to_success() -> Result<(), Box<dyn std::error::Error>> {
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
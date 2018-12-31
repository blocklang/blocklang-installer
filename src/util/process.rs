use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

/// 根据指定的端口号获取进程 id
pub fn get_id(port: u32) -> Option<u32> {
    if cfg!(target_os = "windows") {
        // netstat -ano | findstr 8080
        let child = Command::new("cmd")
                 .args(&["/C", &format!("netstat -ano | findstr {}", port)])
                 .stdout(Stdio::piped())
                 .spawn()
                 .expect("Failed to execute netstat -ano | findstr xxx");

        let mut first_line_content = String::new();
        BufReader::new(child.stdout.unwrap()).read_line(&mut first_line_content).unwrap();
        if first_line_content.trim().is_empty() {
            return None;
        }
        let parts: Vec<&str> = first_line_content
            .trim()
            .split(|c: char| c.is_whitespace() || c.is_control())
            .filter(|&s| s.len() > 0)
            .collect();
        let pid = parts[4].parse::<u32>().expect("将字符串类型的 pid 转换为 u32 类型时出错");
        Some(pid)
    } else if cfg!(target_os = "linux") {
        unimplemented!();
    } else {
        unimplemented!();
    }
}

/// 根据进程 id 杀死进程
pub fn kill(process_id: u32) {
    if cfg!(target_os = "windows") {
        // taskkill /F /PID xxx
        Command::new("cmd")
            .args(&["/C", &format!("taskkill /F /PID {}", process_id)])
            .output()
            .expect(&format!("执行 taskkill /F /PID {} 时出错", process_id));
        println!("进程 {} 已成功关闭", process_id);
    } else if cfg!(target_os = "linux") {
        // kill -9 xxx
        Command::new("sh")
            .args(&["-c", &format!("kill -9 {}", process_id)])
            .output()
            .expect(&format!("执行 kill -9 {} 时出错", process_id));
    } else {
        unimplemented!();
    }
}

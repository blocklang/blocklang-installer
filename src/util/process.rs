use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

/// 根据指定的端口号获取进程 id，
/// 如果返回 `None`，说明端口没有被占用。
pub fn get_id(port: u32) -> Option<u32> {
    let child = if cfg!(target_os = "windows") {
        // netstat -ano | findstr 8080
        Command::new("cmd")
            .args(&["/C", &format!("netstat -ano | findstr {}", port)])
            .stdout(Stdio::piped())
            .spawn()
            .unwrap_or_else(|_| panic!("Failed to execute netstat -ano | findstr {}", port))
    } else if cfg!(target_os = "linux") {
        // netstat -apn | grep 8080
        Command::new("sh")
            .args(&["-c", &format!("netstat -apn | grep {}", port)])
            .stdout(Stdio::piped())
            .spawn()
            .unwrap_or_else(|_| panic!("Failed to execute netstat -apn | grep {}", port))
    } else {
        unimplemented!();
    };

    BufReader::new(child.stdout.unwrap())
        .lines()
        .find_map(|line| extract_process_id(line.unwrap(), port))
}

#[cfg(target_os = "windows")]
pub fn extract_process_id(line: String, port: u32) -> Option<u32> {
    // TCP    0.0.0.0:8080    0.0.0.0:0    LISTENING    1
    let trimed_line = line.trim();
    if trimed_line.is_empty() {
        return None;
    }
    let parts: Vec<&str> = trimed_line
        .split(|c: char| c.is_whitespace() || c.is_control())
        .filter(|&s| !s.is_empty())
        .collect();
    if parts[0] != "TCP" {
        return None;
    }
    if !parts[1].ends_with(&format!(":{}", port)) {
        return None;
    }
    if parts[3] != "LISTENING" {
        return None;
    }
    let pid = parts[4].parse::<u32>().expect("将字符串类型的 pid 转换为 u32 类型时出错");
    Some(pid)
}

#[cfg(target_os = "linux")]
pub fn extract_process_id(line: String, port: u32) -> Option<u32> {
    // tcp    0    0 0.0.0.0:8080    0.0.0.0:*    LISTEN    1/java
    let trimed_line = line.trim();
    if trimed_line.is_empty() {
        return None;
    }
    let parts: Vec<&str> = trimed_line
        .split(|c: char| c.is_whitespace() || c.is_control())
        .filter(|&s| s.len() > 0)
        .collect();
    if parts[0] != "tcp" {
        return None;
    }
    if !parts[3].ends_with(&format!(":{}", port)) {
        return None;
    }
    if parts[5] != "LISTEN" {
        return None;
    }
    let pid_parts: Vec<&str> = parts[6].split("/").collect();
    let pid = pid_parts[0].parse::<u32>().expect("将字符串类型的 pid 转换为 u32 类型时出错");
    Some(pid)
}

// 搜 80 端口时，需要精确匹配，不要匹配到 8080 端口

/// 根据进程 id 杀死进程
pub fn kill(process_id: u32) {
    if cfg!(target_os = "windows") {
        // taskkill /F /PID xxx
        Command::new("cmd")
            .args(&["/C", &format!("taskkill /F /PID {}", process_id)])
            .output()
            .unwrap_or_else(|_| panic!("执行 taskkill /F /PID {} 时出错", process_id));
        println!("进程 {} 已成功关闭", process_id);
    } else if cfg!(target_os = "linux") {
        // kill -9 xxx
        Command::new("sh")
            .args(&["-c", &format!("kill -9 {}", process_id)])
            .output()
            .unwrap_or_else(|_| panic!("执行 kill -9 {} 时出错", process_id));
    } else {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {

    use super::{get_id, extract_process_id};

    #[test]
    fn get_id_none() {
        let not_exist_port = 12_345_678;
        assert_eq!(None, get_id(not_exist_port));
    }

    #[test]
    fn extract_process_id_input_is_empty() {
        let input = "";
        
        assert_eq!(None, extract_process_id(input.to_string(), 80));
    }

    #[test]
    fn extract_process_id_port_not_match() {
        let input = if cfg!(target_os = "windows") {
             r#"TCP    0.0.0.0:8080    0.0.0.0:0    LISTENING    1"#
        } else if cfg!(target_os = "linux") {
             r#"tcp    0    0 0.0.0.0:8080    0.0.0.0:*    LISTEN    1/java"#
        } else {
            unimplemented!();
        };
        
        assert_eq!(None, extract_process_id(input.to_string(), 80));
    }

    #[test]
    fn extract_process_id_not_tcp() {
        let input = if cfg!(target_os = "windows") {
             r#"NOT_TCP    0.0.0.0:80    0.0.0.0:0    LISTENING    1"#
        } else if cfg!(target_os = "linux") {
             r#"not_tcp    0    0 0.0.0.0:80    0.0.0.0:*    LISTEN    1/java"#
        } else {
            unimplemented!();
        };
        
        assert_eq!(None, extract_process_id(input.to_string(), 80));
    }

    #[test]
    fn extract_process_id_not_listening() {
        let input = if cfg!(target_os = "windows") {
             r#"TCP    0.0.0.0:80    0.0.0.0:0    NOT_LISTENING    1"#
        } else if cfg!(target_os = "linux") {
             r#"tcp    0    0 0.0.0.0:80    0.0.0.0:*    NOT_LISTEN    1/java"#
        } else {
            unimplemented!();
        };
        
        assert_eq!(None, extract_process_id(input.to_string(), 80));
    }

    #[test]
    fn extract_process_id_success() {
        let input = if cfg!(target_os = "windows") {
             r#"TCP    0.0.0.0:80      0.0.0.0:0    LISTENING    1"#
        } else if cfg!(target_os = "linux") {
             r#"tcp    0    0 0.0.0.0:80      0.0.0.0:*    LISTEN    1/java"#
        } else {
            unimplemented!();
        };
        
        assert_eq!(Some(1), extract_process_id(input.to_string(), 80));
    }
}

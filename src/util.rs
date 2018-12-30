use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use mac_address;

#[derive(Debug, PartialEq)]
pub struct InterfaceAddr {
    pub ip_address: String,
    pub mac_address: String,
}

#[cfg(target_os = "windows")]
use ipconfig::{self, IfType, OperStatus};
#[cfg(target_os = "windows")]
/// 获取本服务器的 IP 地址和 MAC 地址。
pub fn get_interface_address() -> Option<InterfaceAddr> {
    // MAC 地址
    let adapters = ipconfig::get_adapters().expect("没有获取到网络适配器信息！");
    let mut matched_filter = adapters.into_iter().filter(|adapter| {
        adapter.oper_status() == OperStatus::IfOperStatusUp
            && adapter.if_type() == IfType::EthernetCsmacd
    });

    let matched = matched_filter.next();

    // 寻找到一个处于连接状态的有线网络，
    // 如果找不到，则返回 `None`
    if matched.is_none() {
        return None;
    }

    let adapter = matched.unwrap();
    let ip_address = adapter.ip_addresses().get(1);
    let mac_address = adapter.physical_address().clone().unwrap();
    let mac_address: Vec<String> = mac_address.iter().map(|x| format!("{:x}", x)).collect();
    let mac_address = mac_address.join(":");

    Some(InterfaceAddr {
        ip_address: ip_address.unwrap().to_string(),
        mac_address: mac_address,
    })
}

#[cfg(not(target_os = "windows"))]
pub fn get_interface_address() -> InterfaceAddr {
    // MAC 地址

    InterfaceAddr {
        ip_address: String::from("a"),
        mac_address: String::from("b"),
    }
}

pub fn get_target_os() -> Option<String> {
    // https://stackoverflow.com/questions/41742046/is-there-a-list-of-all-cfg-features

    if cfg!(target_os = "windows"){
        Some("windows".to_string())
    } else if cfg!(target_os = "linux") {
        Some("linux".to_string())
    } else if cfg!(target_os = "macos") {
        Some("macos".to_string())
    } else {
        None
    }
}

/// 根据指定的端口号获取进程 id
pub fn get_process_id(port: u32) -> Option<u32> {
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
pub fn kill_process(process_id: u32) {
    if cfg!(target_os = "windows") {
        // taskkill /F /PID xxx
        Command::new("cmd")
            .args(&["/C", &format!("taskkill /F /PID {}", process_id)])
            .output()
            .expect("taskkill /F /PID xxx");
        println!("进程 {} 已成功关闭", process_id);
    } else if cfg!(target_os = "linux") {
        unimplemented!();
    } else {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {

    use super::get_interface_address;

    // 注意，如果电脑仅通过无线网络联网，则此测试用例会失败
    #[test]
    fn get_interface_address_not_empty() {
        let interface_addr = get_interface_address();
        assert_ne!(interface_addr, None);
    }

}

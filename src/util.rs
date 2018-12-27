use mac_address;
use local_ip;

/// 获取本服务器的 MAC 地址。
pub fn get_mac_address() -> String {
    // MAC 地址
    let hwaddr = mac_address::get_mac_address()
        .expect("failed to get server's mac address!")
        .unwrap()
        .to_string();
    hwaddr
}

/// 获取本服务器的 IP 地址。
pub fn get_ip_address() {
    let ip = local_ip::get().unwrap();
    println!("local ip address: {:?}", ip.to_string());
    
}


#[cfg(test)]
mod tests {

    use super::{get_mac_address, get_ip_address};

    #[test]
    fn get_mac_address_not_empty() {
        let hwaddr = get_mac_address();
        assert!(!hwaddr.is_empty());
    }

    #[test]
    fn get_ip_address_not_empty() {
        get_ip_address();
    }
}
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
    let adapters = ipconfig::get_adapters().unwrap();
    let mut matched = adapters.into_iter().filter(|adapter| {
        adapter.oper_status() == OperStatus::IfOperStatusUp
            && adapter.if_type() == IfType::EthernetCsmacd
    });

    // 寻找到一个处于连接状态的有线网络，
    // 如果找不到，则返回 `None`
    if let None = matched.next() {
        return None;
    }

    let adapter = matched.next().unwrap();
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

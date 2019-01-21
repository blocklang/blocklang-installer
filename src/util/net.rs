#[derive(Debug, PartialEq)]
pub struct InterfaceAddr {
    pub ip_address: String,

    /// MAC 地址
    /// 16进制统一使用大写字母
    pub mac_address: String,
}

#[cfg(target_os = "windows")]
use ipconfig::{self, IfType, OperStatus};
#[cfg(target_os = "windows")]
/// 获取 Windows 服务器的 IP 地址和 MAC 地址。
/// 优先获取有线网络，如果没有找到有线网络，再获取无线网络
pub fn get_interface_address() -> Option<InterfaceAddr> {
    let adapters = ipconfig::get_adapters().expect("没有获取到网络适配器信息！");
    // 先获取有线网络
    let mut matched = adapters.iter().find(|adapter| {
        adapter.oper_status() == OperStatus::IfOperStatusUp
            && adapter.if_type() == IfType::EthernetCsmacd
    });

    // 如果没有找到有线网络，则获取无线网络
    if matched.is_none() {
        matched = adapters.iter().find(|adapter| {
            adapter.oper_status() == OperStatus::IfOperStatusUp
                && adapter.if_type() == IfType::Ieee80211
        });
    }

    matched.map(|adapter| {
            let ip_address = adapter.ip_addresses().get(1);
            let mac_address = adapter.physical_address().clone().unwrap();
            let mac_address: Vec<String> = mac_address.iter().map(|x| format!("{:x}", x)).collect();
            let mac_address = mac_address.join(":");

            InterfaceAddr {
                ip_address: ip_address.unwrap().to_string(),
                mac_address: mac_address.to_uppercase(),
            }
        })
}

#[cfg(not(target_os = "windows"))]
use get_if_addrs;
#[cfg(not(target_os = "windows"))]
use mac_address::mac_address_by_name;

#[cfg(not(target_os = "windows"))]
/// 获取 Linux 服务器的 IP 地址和 MAC 地址。
pub fn get_interface_address() -> Option<InterfaceAddr> {
    // IP 地址
    let ifaces = get_if_addrs::get_if_addrs().unwrap();
    let iface = ifaces.iter().find(|&interface| !interface.is_loopback() && interface.ip().is_ipv4());
    if iface == None {
        return None;
    }

    // MAC 地址
    // 注意，在 windows 平台下要传入的 name 不是适配器的 name，而是 Friendly Name
    // 但是 get_if_addrs 获取的却是适配器的 name。
    let interface_name = &iface.unwrap().name;
    let mac_address = match mac_address_by_name(interface_name) {
        Ok(Some(mac)) => Some(mac),
        Ok(None) => None,
        Err(_) => None,
    };

    Some(InterfaceAddr {
        ip_address: iface.unwrap().ip().to_string(),
        mac_address: mac_address.unwrap().to_string().to_uppercase(),
    })
}

#[cfg(test)]
mod tests {

    use super::get_interface_address;

    // 注意，如果电脑没有联网，则此测试用例会失败
    #[test]
    fn get_interface_address_not_none() {
        let interface_addr = get_interface_address();
        assert_ne!(interface_addr, None);
    }

    #[test]
    fn get_interface_address_not_empty() {
        let interface_addr = get_interface_address().unwrap();
        assert!(!interface_addr.ip_address.is_empty());
        assert!(!interface_addr.mac_address.is_empty());
    }

}

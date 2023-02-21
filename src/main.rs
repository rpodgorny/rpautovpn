fn is_public(addr: &std::net::Ipv6Addr) -> bool {
    addr.to_string().starts_with('2')
}

// the imperative way
fn has_public_ipv6_addr_(prefixes: &[&str]) -> bool {
    for i in pnet::datalink::interfaces() {
        log::trace!("IFACE {:?} {:?}", i.name, i.ips);
        if !prefixes.iter().any(|x| i.name.starts_with(x)) {
            continue;
        }
        for ip in i.ips {
            log::trace!("ADDR {:?}", ip);
            if let pnet::ipnetwork::IpNetwork::V6(a) = ip {
                log::trace!("ADDR6 {:?}", a);
                if is_public(&a.ip()) {
                    return true;
                }
            }
        }
    }
    false
}

// the functional way
fn has_public_ipv6_addr(prefixes: &[&str]) -> bool {
    pnet::datalink::interfaces()
        .iter()
        .filter(|i| prefixes.iter().any(|x| i.name.starts_with(x)))
        .flat_map(|x| {
            x.ips
                .iter()
                .filter_map(|x| match x {
                    pnet::ipnetwork::IpNetwork::V6(v) => Some(v),
                    pnet::ipnetwork::IpNetwork::V4(_) => None,
                })
        })
        .filter(|x| is_public(&x.ip()))
        .count()
        > 0
}

fn is_service_active(service_name: &str) -> bool {
    let rc = std::process::Command::new("systemctl")
        .args(["is-active", service_name])
        .stdout(std::process::Stdio::null())
        .status()
        .unwrap();
    rc.code() == Some(0)
}

fn start_stop_service(service_name: &str, action: &str) {
    log::info!("START_STOP {service_name} {action}");
    std::process::Command::new("systemctl")
        .args([action, service_name])
        .status()
        .unwrap();
}

fn main() {
    simplelog::TermLogger::init(
        //simplelog::LevelFilter::Trace,
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        simplelog::TerminalMode::default(),
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    log::info!("starting rpautovpn v{}", env!("CARGO_PKG_VERSION"));

    let prefixes = vec!["eth", "eno", "wlan"];
    let vpn_iface = "wg0";
    let service_name = format!("wg-quick@{vpn_iface}.service");

    loop {
        let has_ipv6 = has_public_ipv6_addr(&prefixes);
        let is_vpn = is_service_active(&service_name);
        log::debug!("STATE ipv6: {:?} vpn: {:?}", has_ipv6, is_vpn);
        if has_ipv6 && is_vpn {
            start_stop_service(&service_name, "stop");
        } else if !has_ipv6 && !is_vpn {
            start_stop_service(&service_name, "start");
        }
        std::thread::sleep(std::time::Duration::from_secs(10));  // TODO: hard-coded shit
    }
}

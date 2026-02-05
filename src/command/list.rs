use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::process::Command;

use clap::Args;

use crate::config::vm_dir;
use crate::util::json;

#[derive(Args)]
pub struct List;

impl List {
    pub fn execute(&self) {
        let home_dir = vm_dir::home_dir();
        if !home_dir.exists() {
            panic!("home dir does not exist, dir={}", home_dir.to_string_lossy());
        }

        let ip_addrs = ip_addrs();

        println!(
            "{:<16}{:<16}{:<8}{:<8}{:<8}{:<16}{:<16}",
            "name", "status", "os", "cpu", "ram", "disk", "ip"
        );
        for entry in fs::read_dir(home_dir).unwrap_or_else(|err| panic!("failed to read dir, err={err}")) {
            let path = entry.unwrap_or_else(|err| panic!("failed to read dir, err={err}")).path();
            if path.is_dir() {
                let dir = vm_dir::vm_dir(&path.file_name().unwrap().to_string_lossy());
                if dir.initialized() {
                    let name = dir.name();

                    let config = dir.load_config();
                    let os = json::to_json_value(&config.os);
                    let cpu = config.cpu;
                    let ram = format!("{:.2}G", config.ram as f32 / (1024 * 1024 * 1024) as f32);
                    let metadata = dir.disk_path.metadata().unwrap_or_else(|err| panic!("failed to get metadata, err={err}"));
                    let disk = format!(
                        "{:0.2}G/{:.2}G",
                        metadata.blocks() as f32 * 512.0 / 1_000_000_000.0,
                        metadata.len() as f32 / 1_000_000_000.0
                    );
                    let ip = ip_addrs.get(&config.mac_address).map(String::as_str).unwrap_or("-");
                    let status = if dir.pid().is_some() { "running" } else { "stopped" };
                    println!("{:<16}{:<16}{:<8}{:<8}{:<8}{:<16}{:<16}", name, status, os, cpu, ram, disk, ip)
                }
            }
        }
    }
}

fn ip_addrs() -> HashMap<String, String> {
    let output = Command::new("arp").arg("-anl").output().expect("failed to execute arp");
    if !output.status.success() {
        panic!("failed to execute arp, err={}", String::from_utf8_lossy(&output.stderr))
    }
    let output = String::from_utf8(output.stdout).expect("output should be in utf-8");
    parse_arp_output(&output)
}

fn parse_arp_output(output: &str) -> HashMap<String, String> {
    let mut ip_addrs = HashMap::new();
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let mac = parse_mac(parts[1]);
            let ip = parts[0].to_string();
            ip_addrs.insert(mac, ip);
        }
    }
    ip_addrs
}

// in the arp output, the mac address 'fa:5d:0b:89:61:16' is displayed as 'fa:5d:b:89:61:16', with the leading zeroes removed.
fn parse_mac(mac: &str) -> String {
    let parts: Vec<String> = mac
        .split(':')
        .map(|part| if part.len() == 1 { format!("0{part}") } else { part.to_string() })
        .collect();
    parts.join(":")
}

#[cfg(test)]
mod tests {
    use super::parse_arp_output;

    #[test]
    fn test_parse_arp_output() {
        let ip_addrs = parse_arp_output(
            r#"Neighbor                Linklayer Address Expire(O) Expire(I)          Netif Refs Prbs
            10.11.101.76            f0:18:98:3c:4a:cc expired   expired        en0    1
            192.168.64.3            f6:db:b3:ec:f9:3f 2m42s     2m34s     bridge10    1
            192.168.64.8            fa:5d:b:89:61:16  2m33s     1m21s     bridge10    1
            224.0.0.251             1:0:5e:0:0:fb     (none)    (none)         en0"#,
        );
        assert_eq!("192.168.64.3", ip_addrs.get("f6:db:b3:ec:f9:3f").unwrap());
        assert_eq!("192.168.64.8", ip_addrs.get("fa:5d:0b:89:61:16").unwrap());
    }
}

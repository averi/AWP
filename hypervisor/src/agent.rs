# Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
# GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

use sysinfo::System;
use gethostname::gethostname;
use virt::connect::Connect;
use virt::sys::{VIR_CONNECT_LIST_DOMAINS_RUNNING,VIR_CONNECT_LIST_DOMAINS_SHUTOFF,VIR_DOMAIN_INTERFACE_ADDRESSES_SRC_AGENT};
use virt::domain::Interface;
use std::net::IpAddr;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Hypervisor {
    hostname: String,
    memory: u64,
    cpu: usize,
    arch: String,
    vms: Vec<VirtualMachine>,
}

#[derive(Serialize, Deserialize)]
pub struct VirtualMachine {
    name: String,
    memory: u64,
    cpu: u32,
    state: String,
    ip_addresses: Vec<String>,
}

const MAIN_INTERFACE_NAME: &str = "eth0";

impl Hypervisor {
    pub fn new() -> Result<Self, String> {
        let mut system = System::new_all();
        system.refresh_all();

        let hostname = gethostname().to_string_lossy().to_string();
        let memory = system.total_memory() / 1024 / 1024 / 1024;
        let cpu = system.cpus().len();
        let arch: String = std::env::consts::ARCH.to_string();
        let mut vms = Vec::new();

        let conn = Connect::open(Some("qemu:///system")).map_err(|e| format!("Failed to connect to libvirt: {}", e))?;
        let domains = conn.list_all_domains(VIR_CONNECT_LIST_DOMAINS_RUNNING | VIR_CONNECT_LIST_DOMAINS_SHUTOFF).map_err(|e| format!("Failed to list domains: {}", e))?;
        for domain in domains {
            let vm_info = domain.get_info().map_err(|e| format!("Failed to get definition for domain: {}", e))?;
            let vm_name = domain.get_name().map_err(|e| format!("Failed to get name for domain: {}", e))?;
            let interfaces_result = domain.interface_addresses(
                VIR_DOMAIN_INTERFACE_ADDRESSES_SRC_AGENT,
                0,
            );

            let ip_addresses: Vec<String> = match interfaces_result {
                Ok(interfaces) => {
                    Self::get_main_interface_ip(interfaces).unwrap_or_default()
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Could not get interfaces via agent for domain '{}': {}. Using empty IP list.",
                        vm_name, e
                    );
                    Vec::new()
                }
            };

            vms.push(VirtualMachine {
                name: vm_name,
                memory: vm_info.memory / 1024 / 1024,
                cpu: vm_info.nr_virt_cpu,
                ip_addresses,
                state: VirtualMachine::convert_status_codes(vm_info.state),
            });
        }

        Ok(Hypervisor { hostname, memory, cpu, arch, vms })
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("Failed to serialize to JSON: {}", e))
    }

    fn get_main_interface_ip(interfaces: Vec<Interface>) -> Option<Vec<String>> {
        let mut ip_addresses = Vec::new();

        for iface in interfaces {
            if iface.name == MAIN_INTERFACE_NAME {
                for addr_info in &iface.addrs {
                    match IpAddr::from_str(&addr_info.addr) {
                        Ok(ip) => {
                            ip_addresses.push(format!("{}/{}", ip, addr_info.prefix));
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Could not parse IP string '{}' for interface {}: {}",
                                addr_info.addr, iface.name, e
                            );
                        }
                    }
                }
            }
        }

        if !ip_addresses.is_empty() {
            return Some(ip_addresses);
        } else {
            None
        }
    }
}

impl VirtualMachine {
    pub fn convert_status_codes(status: u32) -> String {
        match status {
            0 => "No state".to_string(),
            1 => "Running".to_string(),
            2 => "Blocked".to_string(),
            3 => "Paused".to_string(),
            4 => "Shutdown".to_string(),
            5 => "Shutoff".to_string(),
            6 => "Crashed".to_string(),
            7 => "Suspended".to_string(),
            _ => "Unknown".to_string(),
        }
    }
}
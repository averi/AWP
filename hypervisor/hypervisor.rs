# Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
# GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

use sysinfo::System;
use gethostname::gethostname;
use virt::connect::Connect;
use virt::domain::Domain;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize)]
struct Hypervisor {
    hostname: String,
    memory: u64,
    cpu: usize,
    vms: Vec<VirtualMachine>,
}

#[derive(Serialize, Deserialize)]
struct VirtualMachine {
    name: String,
    memory: u64,
    cpu: u32,
    state: String,
}

impl Hypervisor {
    fn new(hostname: String, memory: u64, cpu: usize, vms: Vec<VirtualMachine>) -> Hypervisor {
        Hypervisor {
            hostname,
            memory,
            cpu,
            vms
        }
    }
}

impl VirtualMachine {
    fn convert_status_codes(status: u32) -> String {
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

fn main() {
    let mut system = System::new_all();
    system.refresh_all();

    let hostname = gethostname().to_string_lossy().to_string();
    let memory = system.total_memory() / 1024 / 1024 / 1024;
    let cpu = system.cpus().len();

    let mut hyperv = Hypervisor::new(hostname, memory, cpu, vec![]);

    let conn = match Connect::open(Some("qemu:///system")) {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Failed to connect to libvirt: {}", err);
            return;
        }
    };

    match conn.list_defined_domains() {
        Ok(domains) => {
            for domain in domains {
                let dom = match Domain::lookup_by_name(&conn, &domain) {
                    Ok(dom) => dom,
                    Err(err) => {
                        eprintln!("Failed to lookup domain {}: {}", domain, err);
                        continue;
                    }
                };
                match dom.get_info() {
                    Ok(vm) => {
                        hyperv.vms.push(VirtualMachine {
                            name: domain,
                            memory: vm.memory / 1024 / 1024,
                            cpu: vm.nr_virt_cpu,
                            state: VirtualMachine::convert_status_codes(vm.state),
                        });
                    }
                    Err(err) => {
                        eprintln!("Failed to get info for domain {}: {}", domain, err);
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to list domains: {}", err);
        }
    }

    match serde_json::to_string(&hyperv) {
        Ok(json) => println!("{}", json),
        Err(err) => eprintln!("Failed to serialize to JSON: {}", err),
    }
}

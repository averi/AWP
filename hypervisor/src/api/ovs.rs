// Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
// GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

use std::io;
use serde::{Serialize, Deserialize};
use std::process::Command;


#[derive(Serialize, Deserialize, Debug)]
pub struct OvsDbRequest {
    method: String,
    params: Vec<serde_json::Value>,
    id: u64,
}

impl OvsDbRequest {
    pub async fn add_vnet_to_provider_bridge(port_name: &str, provider_bridge: &str, tenant: &str) -> Result<(), io::Error> {
        let vnet_name = tenant.to_string() + "-" + port_name;

        let add_vnet = Command::new("ovs-vsctl")
            .args(&[
                "add-port",
                provider_bridge,
                &vnet_name
            ])
            .output()?;

        if !add_vnet.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("ovs-vsctl add-port: {}", String::from_utf8_lossy(&add_vnet.stderr)),
            ));
        }

        Ok(())
    }

    pub async fn add_port(port_name: &str, bridge_name: Option<String>, tenant: String) -> Result<(), io::Error> {
        let bridge_name = bridge_name.unwrap_or("br-int".to_string());
        let port_name = tenant + "-" + port_name;

        let add_port = Command::new("ovs-vsctl")
            .args(&[
                "add-port",
                &bridge_name,
                &port_name,
                &format!("external_ids:iface-id={}", &port_name),
            ])
            .output()?;

        if !add_port.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("ovs-vsctl add-port: {}", String::from_utf8_lossy(&add_port.stderr)),
            ));
        }

        let set_interface = Command::new("ovs-vsctl")
            .args(&[
                "set",
                "Interface",
                &port_name,
                &format!("external_ids:iface-id={}", &port_name),
            ])
            .output()?;

        if !set_interface.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("ovs-vsctl set Interface failure: {}", String::from_utf8_lossy(&set_interface.stderr)),
            ));
        }

        Ok(())
    }

    pub async fn delete_port(port_name: String, bridge_name: Option<String>, tenant: String) -> Result<(), io::Error> {
        let bridge_name = bridge_name.unwrap_or("br-int".to_string());
        let port_name = tenant + "-" + &port_name;

        let remove_port = Command::new("ovs-vsctl")
            .args(&[
                "del-port",
                &bridge_name,
                &port_name,
            ])
            .output()?;

        if !remove_port.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("ovs-vsctl del-port failed with: {}", String::from_utf8_lossy(&remove_port.stderr)),
            ));
        }

        Ok(())
    }
}
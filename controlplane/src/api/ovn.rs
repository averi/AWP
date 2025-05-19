// Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
// GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

use chrono;
use serde_json::{json, self, Value};
use serde_yaml;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use rand::{rng, Rng};


#[derive(serde::Deserialize, Debug, Clone)]
struct Config {
    ovn: OvnAPI,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct OvnAPI {
    host: String,
    port: u16,
}

fn read_conf_file(config_file: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let file = std::fs::read_to_string(config_file)?;
    let config: Config = serde_yaml::from_str(&file)?;
    Ok(config)
}

pub async fn create_l2_switch(name: &str, cidr: &str) -> Result<(), std::io::Error> {
    let conf_file: Config = read_conf_file("config.yaml").unwrap();

    let request_body = json!({
        "method": "transact",
        "params": [
            "OVN_Northbound",
            {
                "op": "insert",
                "table": "Logical_Switch",
                "row": {
                    "name": name,
                    "other_config": ["map", [["subnet", cidr]]]
                }
            },
            {
                "op": "comment",
                "comment": format!("Added by create_l2_switch {} at {}", name, chrono::Utc::now())
            }
        ],
        "id": 1
    }).to_string() + "\n"; // OVSDB expects newline-delimited JSON messages
    
    write_to_ovsdb(&request_body, conf_file).await?;
    Ok(())
}

pub async fn delete_l2_switch(name: &str) -> Result<(), std::io::Error> {
    let conf_file: Config = read_conf_file("config.yaml").unwrap();

    let request_body = json!({
        "method": "transact",
        "params": [
            "OVN_Northbound",
            {
                "op": "delete",
                "table": "Logical_Switch",
                "where": [["name", "==", name]]
            },
            {
                "op": "comment",
                "comment": format!("Deleted by delete_l2_switch {} at {}", name, chrono::Utc::now())
            }
        ],
        "id": 1
    }).to_string() + "\n";

    write_to_ovsdb(&request_body, conf_file).await?;
    Ok(())
}

pub async fn add_lsp_to_ls(port_name: &str, switch_name: &str) -> Result<String, std::io::Error> {
    let conf_file: Config = read_conf_file("config.yaml").unwrap();

    let fetch_ls_uuid = json!({
        "method": "transact",
        "params": [
            "OVN_Northbound",
            {
            "op": "select",
            "table": "Logical_Switch",
            "where": [["name", "==", &switch_name]]
            }
        ],
        "id": 3
    }).to_string() + "\n";

    let ls_uuid_response = write_to_ovsdb(&fetch_ls_uuid, conf_file.clone()).await?;
    let switch_uuid = extract_uuid_from_row(&ls_uuid_response).await?;

    let lsp_add_request = json!({
        "method": "transact",
        "params": [
            "OVN_Northbound",
            {
                "op": "insert",
                "table": "Logical_Switch_Port",
                "row": { "name": format!("{}", &port_name)},
                "uuid-name": "row1"
            },
            {
                "op": "mutate",
                "table": "Logical_Switch",
                "where": [
                    ["_uuid", "==", ["uuid", &switch_uuid]]
                ],
                "mutations": [
                    [
                        "ports",
                        "insert",
                        [
                            "set",
                            [["named-uuid", "row1"]]
                        ]
                    ]
                ]
            },
            {
                "op": "comment",
                "comment": format!("Added by add_lsp_to_ls for {}", &port_name)
            }
        ],
        "id": 4
    }).to_string() + "\n";

    let response = write_to_ovsdb(&lsp_add_request, conf_file).await?;
    Ok(response)
}

pub async fn extract_uuid_from_row(response: &str) -> Result<String, std::io::Error> {
    // Northd response: {"error":null,"id":3,"result":[{"rows":[{"_uuid":["uuid","640ac16a-9d76-424d-a35f-e8cc39d0d41a"],
    // "_version":["uuid","a419722e-f685-4ba2-8616-3d7289e064c6"],"acls":["set",[]],"copp":["set",[]],
    // "dns_records":["set",[]],"external_ids":["map",[]],"forwarding_groups":["set",[]],
    // "load_balancer":["set",[]],"load_balancer_group":["set",[]],"name":"averi",
    // "other_config":["map",[]],"ports":["set",[]],"qos_rules":["set",[]]}]}]}  
    let json_response: serde_json::Value = serde_json::from_str(response)?;
    if let Some(uuid) = json_response["result"][0]["rows"][0]["_uuid"][1].as_str(){
        Ok(uuid.to_string())
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "UUID could not be parsed"))
    }
}

pub async fn extract_uuid_from_response(response: &str) -> Result<String, std::io::Error> {
    // Northd response: "error":null,"id":1,"result":[{"uuid":["uuid","b19bc163-1325-4036-9dfc-f8155518e41e"]},{}]}
    let json_response: serde_json::Value = serde_json::from_str(response)?;
    if let Some(uuid) = json_response["result"][0]["uuid"][1].as_str() {
        Ok(uuid.to_string())
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "UUID could not be parsed"))
    }
}

pub async fn generate_mac_address() -> [u8; 6] {
    // let mac_addr_as_string = format!(
    //     "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
    //     mac_addr[0], mac_addr[1], mac_addr[2],
    //     mac_addr[3], mac_addr[4], mac_addr[5]
    //   );
    let mut rng = rng();

    // Fixed prefix for KVM (52:54:00)
    let mut mac_address = [0x52, 0x54, 0x00, 0, 0, 0];

    for i in 3..6 {
        mac_address[i] = rng.random_range(0..=255);
    }
    
    mac_address
}

pub async fn get_dhcpv4_options_id(cidr: &str) -> Result<String, std::io::Error> {
    let conf_file: Config = read_conf_file("config.yaml").unwrap();

    let fetch_dhcp_options_uuid = json!({
        "method": "transact",
        "params": [
            "OVN_Northbound",
            {
                "op": "select",
                "table": "DHCP_Options",
                "where": [["cidr", "==", cidr]]
            }
        ],
        "id": 20
    }).to_string() + "\n";

    let dhcp_options_uuid_response = write_to_ovsdb(&fetch_dhcp_options_uuid, conf_file).await?;
    let dhcp_options_uuid = extract_uuid_from_row(&dhcp_options_uuid_response).await?;

    Ok(dhcp_options_uuid)
}

pub async fn delete_dhcpv4_options(cidr: &str) -> Result<(), std::io::Error> {
    let conf_file: Config = read_conf_file("config.yaml").unwrap();

    let dhcp_options_uuid = get_dhcpv4_options_id(&cidr).await?;

    let delete_dhcp_options_request = json!({
        "method": "transact",
        "params": [
            "OVN_Northbound",
            {
                "op": "delete",
                "table": "DHCP_Options",
                "where": [["_uuid", "==", ["uuid", &dhcp_options_uuid]]]
            },
            {
                "op": "comment",
                "comment": format!("Deleted by delete_dhcpv4_options with cidr={} at {}", &cidr, chrono::Utc::now())
            }
        ],
        "id": 12
    }).to_string() + "\n";

    write_to_ovsdb(&delete_dhcp_options_request, conf_file).await?;
    Ok(())
}

pub async fn create_dhcpv4_options(cidr: &str) -> Result<String, std::io::Error> {
    let conf_file: Config = read_conf_file("config.yaml").unwrap();
    let mac_addr = generate_mac_address().await;
    let mac_addr_as_string = format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        mac_addr[0], mac_addr[1], mac_addr[2],
        mac_addr[3], mac_addr[4], mac_addr[5]
      ).to_lowercase();

      let router_ip = cidr.split(".").take(3).collect::<Vec<&str>>().join(".") + ".254";
      let dhcpv4_options_request = json!({
        "method": "transact",
        "params": [
            "OVN_Northbound",
            {
                "op": "insert",
                "table": "DHCP_Options",
                "row": {
                    "cidr": &cidr,
                    "options": [
                        "map",
                        [
                            ["lease_time", "3600"],
                            ["router", &router_ip],
                            ["server_id", &router_ip],
                            ["server_mac", &mac_addr_as_string]
                        ]
                    ]
                },
                "uuid-name": "row1"
            },
            {
                "op": "comment",
                "comment": format!("Created by create_dhcpv4_options with cidr={} at {}", &cidr, chrono::Utc::now())
            }
        ],
        "id": 6
    }).to_string() + "\n";    

    let response = write_to_ovsdb(&dhcpv4_options_request, conf_file).await?;
    Ok(response)
}

pub async fn add_dhcp_options_to_lsp(port_uuid: &str, dhcp_options_uuid: &str) -> Result<(), std::io::Error> {
    let conf_file: Config = read_conf_file("config.yaml").unwrap();

    let lsp_set_dhcp_request = json!({
        "method": "transact",
        "params": [
            "OVN_Northbound",
            {
                "op": "update",
                "table": "Logical_Switch_Port",
                "row": {
                    "dhcpv4_options": ["uuid", &dhcp_options_uuid]
                },
                "where": [
                    [
                        "_uuid",
                        "==",
                        ["uuid", &port_uuid]
                    ]
                ]
            },
            {
                "op": "comment",
                "comment": format!("Added by add_dhcp_options_to_lsp with: port_uuid={},dhcp_options_uuid={}", &port_uuid, &dhcp_options_uuid)
            }
        ],
        "id": 7
    }).to_string() + "\n";

    write_to_ovsdb(&lsp_set_dhcp_request, conf_file).await?;
    Ok(())
}

pub async fn add_mac_to_lsp(port_uuid: &str, mac_address: &str) -> Result<(), std::io::Error> {
    let conf_file: Config = read_conf_file("config.yaml").unwrap();
    let mac_address = mac_address.to_owned() + " dynamic";

    let lsp_set_addresses_request = json!({
            "method": "transact",
            "params": [
                "OVN_Northbound",
                {
                    "op": "update",
                    "table": "Logical_Switch_Port",
                    "row": {
                        "addresses": &mac_address
                    },
                    "where": [
                        [
                            "_uuid",
                            "==",
                            [
                                "uuid",
                                &port_uuid
                            ]
                        ]
                    ]
                },
                {
                    "op": "comment",
                    "comment": format!("MAC Address defined by add_mac_to_lsp for {}", &port_uuid)
                }
            ],
            "id": 5
    }).to_string() + "\n";

    write_to_ovsdb(&lsp_set_addresses_request, conf_file).await?;

    Ok(())
}

pub async fn remove_lsp(port_name: &str, switch_name: &str) -> Result<(), std::io::Error> {
    let conf_file: Config = read_conf_file("config.yaml").unwrap();

    let fetch_ls_uuid = json!({
        "method": "transact",
        "params": [
            "OVN_Northbound",
            {
            "op": "select",
            "table": "Logical_Switch",
            "where": [["name", "==", switch_name]]
            }
        ],
        "id": 8
    }).to_string() + "\n";

    let fetch_lsp_uuid = json!({
        "method": "transact",
        "params": [
            "OVN_Northbound",
            {
            "op": "select",
            "table": "Logical_Switch_Port",
            "where": [["name", "==", port_name]]
            }
        ],
        "id": 9
    }).to_string() + "\n";

    let ls_uuid_response = write_to_ovsdb(&fetch_ls_uuid, conf_file.clone()).await?;
    let lsp_uuid_response = write_to_ovsdb(&fetch_lsp_uuid, conf_file.clone()).await?;

    let ls_uuid = extract_uuid_from_row(&ls_uuid_response).await?;
    let lsp_uuid = extract_uuid_from_row(&lsp_uuid_response).await?;

    let lsp_delete_request = json!({
        "method": "transact",
        "params": [
            "OVN_Northbound",
            {
                "mutations": [
                    [
                        "ports",
                        "delete",
                        [
                            "set",
                            [
                                [
                                    "uuid",
                                    &lsp_uuid
                                ]
                            ]
                        ]
                    ]
                ],
                "op": "mutate",
                "table": "Logical_Switch",
                "where": [
                    [
                        "_uuid",
                        "==",
                        [
                            "uuid",
                            &ls_uuid
                        ]
                    ]
                ]
            },
            {
                "comment": "",
                "op": "comment"
            }
        ],
        "id": 10
    }).to_string() + "\n";    
    
    write_to_ovsdb(&lsp_delete_request, conf_file).await?;

    Ok(())
}

async fn write_to_ovsdb(request: &str, conf_file: Config) -> Result<String, io::Error> {
    match TcpStream::connect(format!("{}:{}", conf_file.ovn.host, conf_file.ovn.port)) {
        Ok(mut stream) => {
            stream.write_all(request.as_bytes())?;
            
            let mut response = Vec::new();
            let mut buffer = [0; 1024];

            loop {
                match stream.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        response.extend_from_slice(&buffer[..n]);

                        match serde_json::from_slice::<Value>(&response) {
                            Ok(json) => {
                                println!("Response: {}", json);
                                let json_str = json.to_string();
                                return Ok(json_str);
                            }
                            Err(e) => {
                                if e.is_syntax() || e.is_data() {
                                    eprintln!("Failed to parse JSON: {}", e);
                                    return Err(io::Error::new(io::ErrorKind::InvalidData, e));
                                } else {
                                    continue;
                                }
                            }
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        eprintln!("Read timed out.");
                        break;
                    }
                    Err(e) => {
                        eprintln!("Failed to read response: {}", e);
                        return Err(e);
                    }
                }
            }
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Incomplete response"))
        }
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            Err(e)
        }
    }
}
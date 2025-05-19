// Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
// GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

mod database;
mod ovn;

use axum::{
    extract::Json, http::StatusCode, response::IntoResponse, routing::{get,post}, Router
};
use ovn::{delete_dhcpv4_options, extract_uuid_from_response, get_dhcpv4_options_id, remove_lsp};
use sqlx::{prelude::FromRow, types::ipnetwork::IpNetwork};
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse};
use sqlx::types::Uuid;
use crate::api::database::Database;
use crate::api::ovn::{create_l2_switch,add_lsp_to_ls,add_mac_to_lsp, generate_mac_address, create_dhcpv4_options, add_dhcp_options_to_lsp};

use serde_json::json;
use reqwest::Client;

use tower_http::cors::{Any,CorsLayer};
use axum::http::header::{ORIGIN, ACCEPT, CONTENT_TYPE};
use axum::http::Method;

use ssh_key::PublicKey;

use std::collections::HashSet;


#[derive(serde::Serialize, serde::Deserialize, FromRow)]
pub struct Vpc {
    id: Option<Uuid>,
    name: Option<String>,
    cidr: Option<String>,
    nat: Option<bool>,
    tenant: Option<Uuid>
}

#[derive(serde::Serialize, serde::Deserialize, FromRow)]
pub struct VpcDelete {
    id: Uuid,
    tenant: Uuid
}

#[derive(serde::Serialize, serde::Deserialize, FromRow)]
pub struct Tenant {
    name: Option<String>,
    id: Option<Uuid>,
}

#[derive(serde::Serialize, serde::Deserialize, FromRow, Debug)]
pub struct Port {
    id: Option<Uuid>,
    name: String,
    vpc: String,
    hypervisor: String,
}

#[derive(serde::Serialize, serde::Deserialize, FromRow)]
pub struct Hypervisor {
    id: Option<Uuid>,
    hostname: String,
    total_ram: i32,
    total_cpu: i32,
    used_ram: i32,
    used_cpu: i32,
    vms: Vec<VirtualMachine>,
}

#[derive(serde::Serialize, serde::Deserialize, FromRow)]
pub struct HypervisorAgent {
    hostname: String,
    memory: i32,
    cpu: i32,
    arch: String,
    vms: Vec<HypervisorSchedulerVM>,
}

#[derive(FromRow, serde::Serialize, serde::Deserialize)]
pub struct HypervisorScheduler {
    id: Uuid,
    hostname: String,
    total_ram: i32,
    total_cpu: i32,
    used_ram: i32,
    used_cpu: i32,
    hosted_vms: i32,
    arch: String,
}

#[derive(serde::Serialize, serde::Deserialize, FromRow)]
pub struct HypervisorSchedulerVM {
    name: String,
    memory: i32,
    cpu: i32,
    state: String,
    ip_addresses: Vec<IpNetwork>
}

#[derive(serde::Serialize, serde::Deserialize, FromRow, Debug)]
pub struct VirtualMachine {
    name: String,
    ram: i32,
    cpu: i32,
    state: String,
    os: String,
    disk_size: i32,
    vpc: Uuid,
    ssh_pub_key: Uuid,
    tenant: Uuid,
    hypervisor: Uuid,
    ip_addresses: Vec<IpNetwork>
}

#[derive(serde::Serialize, serde::Deserialize, FromRow)]
pub struct VirtualMachineCreate {
    name: String,
    ram: i32,
    cpu: i32,
    os: String,
    disk_size: i32,
    vpc: String,
    ssh_pub_key: String,
    tenant: String,
    arch: String,
    networking: String,
    network: Option<String>
}

#[derive(serde::Serialize, serde::Deserialize, FromRow)]
pub struct VirtualMachineDelete {
    name: String,
    tenant: String,
}

#[derive(serde::Serialize, serde::Deserialize, FromRow)]
pub struct SSHKey {
    name: Option<String>,
    ssh_pub_key: Option<String>,
    fingerprint: Option<String>,
    tenant: Option<Uuid>,
}

#[derive(serde::Serialize, serde::Deserialize, FromRow)]
pub struct ProviderNetwork {
    name: String,
    vlan: i32,
    subnet: String
}

#[derive(serde::Serialize, serde::Deserialize, FromRow)]
pub struct ProviderNetworkDelete {
    name: String,
}

pub struct ControlPlaneAPI {}

impl ControlPlaneAPI {
    pub async fn router() -> Router {
        tracing_subscriber::fmt()
        .with_env_filter("axum=debug,tower_http=debug")
        .init();

        let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers(vec![ORIGIN, ACCEPT, CONTENT_TYPE])
        .allow_origin(Any);

        Router::new()
            .route("/tenant/create", post(create_tenant_handler))
            .route("/tenant/delete", post(delete_tenant_handler))
            .route("/tenants/list", get(list_tenants_handler))
            .route("/vpc/create", post(create_vpc_handler))
            .route("/vpc/delete", post(delete_vpc_handler))
            .route("/vpcs/list", post(list_vpcs_handler))
            .route("/ports/list", get(list_ports_handler))
            .route("/hypervisor/stats", post(hypervisor_stats_handler))
            .route("/hypervisors/list", get(list_hypervisors_handler))
            .route("/ssh_pub_key/create", post(create_ssh_pub_key))
            .route("/ssh_pub_key/delete", post(delete_ssh_pub_key))
            .route("/ssh_pub_keys/list", post(list_ssh_pub_keys))
            .route("/virtualmachine/create", post(virtual_machine_scheduler))
            .route("/virtualmachine/delete", post(delete_vm_handler))
            .route("/virtualmachines/list", post(list_vm_handler))
            .route("/provider_network/create", post(create_provider_network_handler))
            .route("/provider_network/delete", post(delete_provider_network_handler))
            .route("/provider_networks/list", get(list_provider_networks_handler))
            .fallback(handler_404)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::new().level(tracing::Level::DEBUG)) 
                    .on_response(DefaultOnResponse::new().level(tracing::Level::DEBUG)),
            )
            .layer(cors)
    }

    pub async fn start_server(app: Router) -> Result<(), Box<dyn std::error::Error>> {
        let listener = match tokio::net::TcpListener::bind("0.0.0.0:8080").await {
            Ok(listener) => {
                println!("Listening on: {}", listener.local_addr().unwrap());
                listener
            }
            Err(e) => {
                eprintln!("Failed to bind to address: {}", e);
                return Err(Box::new(e));
            }
        };
    
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("Server failed: {}", e);
            return Err(Box::new(e));
        }
    
        Ok(())
    }
}

async fn create_tenant_handler(Json(payload): Json<Tenant>) -> impl IntoResponse {
    let db = match Database::new().await {
        Ok(db) => db,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error").into_response();
        }
    };

    match payload.name {
        Some(name) => {
            let existing = Database::get_tenant_by_name(&db, &name).await.unwrap();

            if ! existing.is_none() {
                return (StatusCode::BAD_REQUEST, format!("Tenant '{}' already exists.", &name)).into_response();
            }

            let create_tenant: Result<(), sqlx::Error> = Database::create_tenant(&db, &name).await;
            match create_tenant {
                Ok(_) => (StatusCode::OK, format!("Tenant '{}' created successfully.", &name)).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create tenant: {}", e)).into_response(),
            }
        },
        _ => (StatusCode::BAD_REQUEST, format!("Tenant create request must include a name.")).into_response(),
    }
}

async fn delete_tenant_handler(Json(payload): Json<Tenant>) -> impl IntoResponse {
    let db = match Database::new().await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Database connection error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error").into_response();
        }
    };

    let tenant_id_to_delete: Uuid;
    let tenant_identifier_for_msg: String;

    if let Some(id) = payload.id {
        match Database::get_tenant_by_id(&db, &id).await {
            Ok(Some(_tenant_record)) => {
                tenant_id_to_delete = id;
                tenant_identifier_for_msg = id.to_string();
            }
            Ok(None) => {
                return (StatusCode::NOT_FOUND, format!("Tenant with UUID '{}' not found.", id)).into_response();
            }
            Err(e) => {
                eprintln!("Error fetching tenant by ID '{}': {}", id, e);
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error checking tenant ID: {}", e)).into_response();
            }
        }
    } else if let Some(name) = payload.name {
        match Database::get_tenant_by_name(&db, &name).await {
            Ok(Some(tenant_record)) => {
                tenant_id_to_delete = tenant_record;
                tenant_identifier_for_msg = name.clone();
            }
            Ok(None) => {
                return (StatusCode::NOT_FOUND, format!("Tenant with name '{}' not found.", name)).into_response();
            }
            Err(e) => {
                eprintln!("Error fetching tenant by name '{}': {}", name, e);
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error checking tenant name: {}", e)).into_response();
            }
        }
    } else {
        return (StatusCode::BAD_REQUEST, "Tenant delete request must include either 'id' (UUID) or 'name'.".to_string()).into_response();
    }

    match Database::get_virtual_machine_by_tenant(&db, &tenant_id_to_delete).await {
        Ok(vms) => {
            match vms {
                Some(_) => {
                    return (StatusCode::BAD_REQUEST, format!("Tenant '{}' has associated VMs. Please delete associated VMs first.", tenant_identifier_for_msg)).into_response();
                }
                None => {}
            }
        }
        Err(e) => {
            eprintln!("Error fetching VMs for tenant '{}': {}", tenant_id_to_delete, e);
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error checking associated VMs: {}", e)).into_response();
        }
    }

    match Database::list_ssh_pub_keys(&db, &tenant_id_to_delete).await {
        Ok(ssh_keys) => {
            if !ssh_keys.is_empty() {
                return (
                    StatusCode::BAD_REQUEST,
                    format!("Tenant '{}' has associated SSH keys ({} found). Please delete associated SSH keys first.",
                            tenant_identifier_for_msg, ssh_keys.len())
                ).into_response();
            }
        }
        Err(e) => {
            eprintln!("Error fetching SSH keys for tenant '{}': {}", tenant_id_to_delete, e);
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error checking associated SSH keys: {}", e)).into_response();
        }
    }

    match Database::delete_tenant(&db, &tenant_id_to_delete).await {
        Ok(_) => {
            (StatusCode::OK, format!("Tenant '{}' deleted successfully.", tenant_identifier_for_msg)).into_response()
        }
        Err(e) => {
            eprintln!("Failed to delete tenant '{}' (ID: {}): {}", tenant_identifier_for_msg, tenant_id_to_delete, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete tenant '{}': {}", tenant_identifier_for_msg, e)).into_response()
        }
    }
}

async fn list_tenants_handler() -> impl IntoResponse {
    let db = match Database::new().await {
        Ok(db) => db,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error").into_response();
        }
    };

    let tenants = match Database::list_tenants(&db).await {
        Ok(tenants) => tenants,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error fetching tenants").into_response();
        }
    };

    let tenants_json = match serde_json::to_string(&tenants) {
        Ok(json) => json,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error serializing tenants").into_response();
        }
    };

    (StatusCode::OK, tenants_json).into_response()
}

async fn create_vpc_handler(Json(payload): Json<Vpc>) -> impl IntoResponse {
    let db = match Database::new().await {
        Ok(db) => db,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error").into_response();
        }
    };

    match (payload.name, payload.cidr, payload.nat, payload.tenant) {
        (Some(name), Some(cidr), Some(nat), Some(tenant)) => {
            let is_existing = Database::get_vpc_by_name(&db, &name, &tenant).await.unwrap();
            if ! is_existing.is_none() {
                return (StatusCode::BAD_REQUEST, format!("VPC '{}' already exists.", &name)).into_response();
            }

            let switch_name = format!("{}-{}", &tenant, &name);
            let l2_switch = create_l2_switch(&switch_name, &cidr).await;
            match l2_switch {
                Ok(_) => (),
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create L2 switch: {}", e)).into_response(),
            }

            match create_dhcpv4_options(&cidr).await {
                Ok(_) => (),
                Err(e) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create DHCPv4 options: {}", e)).into_response();
                }
            }

            let create_vpc: Result<(), sqlx::Error> = Database::create_vpc(&db, &name, &cidr, &nat, &tenant).await;
            match create_vpc {
                Ok(_) => (StatusCode::OK, format!("VPC '{}' created successfully.", name)).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create VPC: {}", e)).into_response(),
            }
        },
        _ => (StatusCode::BAD_REQUEST, format!("VPC create request must include a name and CIDR.")).into_response(),
    }
}

async fn delete_vpc_handler(Json(payload): Json<VpcDelete>) -> impl IntoResponse {
    let db = match Database::new().await {
        Ok(db) => db,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error").into_response();
        }
    };

    if let Err(e) = Database::get_vpc_by_id(&db, &payload.id, &payload.tenant).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)).into_response();
    }    

    match Database::list_ports(&db, &payload.id).await {
        Ok(ports) => {
            if !ports.is_empty() {
                return (
                    StatusCode::BAD_REQUEST,
                    format!("VPC '{}' has ports associated with it. Please delete associated ports first.", &payload.id),
                ).into_response();
            }
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ).into_response();
        }
    }

    let vpc_name = Database::get_vpc_by_id(&db, &payload.id, &payload.tenant).await.unwrap();
    match vpc_name {
        Some(vpc) => {
            let vpc_name = format!("{}-{}", &payload.tenant, &vpc);
            if let Err(e) = ovn::delete_l2_switch(&vpc_name).await {
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete L2 switch: {}", e)).into_response();
            }

            let vpc_object = Database::get_vpc_object(&db, &vpc).await;
            match vpc_object {
                Ok(Some(vpc)) => {
                    match vpc.cidr {
                        Some(cidr) => {
                            if let Err(e) = delete_dhcpv4_options(&cidr).await {
                                return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete DHCPv4 options: {}", e)).into_response();
                            }
                        }
                        None => return (StatusCode::BAD_REQUEST, "VPC CIDR not found".to_string()).into_response(),
                        
                    }
                }
                Ok(None) => return (StatusCode::BAD_REQUEST, "VPC not found".to_string()).into_response(),
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)).into_response(),
            }
        }
        None => return (StatusCode::BAD_REQUEST, format!("VPC '{}' does not exist.", &payload.id)).into_response(),
    }

    let delete_vpc: Result<(), sqlx::Error> = Database::delete_vpc(&db, &payload.id).await;
    match delete_vpc {
        Ok(_) => (StatusCode::OK, format!("VPC '{}' deleted successfully.", &payload.id)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete VPC: {}", e)).into_response(),
    }
}

async fn list_vpcs_handler(Json(payload): Json<Tenant>) -> impl IntoResponse {
    let db = match Database::new().await {
        Ok(db) => db,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error").into_response();
        }
    };

    match (payload.name, payload.id) {
        (Some(name), _) => {
            let vpcs = Database::list_vpcs_by_tenantname(&db, &name).await.unwrap();
            let vpcs = serde_yaml::to_string(&vpcs).unwrap();
            (StatusCode::OK, vpcs).into_response()
        },
        (_, Some(id)) => {
            let vpcs = Database::list_vpcs_by_tenantid(&db, &id).await.unwrap();
            let vpcs = serde_json::to_string(&vpcs).unwrap();
            (StatusCode::OK, vpcs).into_response()
        },
        _ => (StatusCode::BAD_REQUEST, format!("VPC list request must include a tenant name or UUID.")).into_response(),
    }
}

async fn list_ports_handler(Json(payload): Json<Vpc>) -> impl IntoResponse {
    let db = match Database::new().await {
        Ok(db) => db,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error").into_response();
        }
    };

    match payload.id {
        Some(id) => {
            let ports = Database::list_ports(&db, &id).await.unwrap();
            let ports = serde_json::to_string(&ports).unwrap();
            (StatusCode::OK, ports).into_response()
        },
        _ => (StatusCode::BAD_REQUEST, format!("VPC delete request must use a valid VPC UUID.")).into_response(),
    }
}

async fn hypervisor_stats_handler(Json(payload): Json<HypervisorAgent>) -> impl IntoResponse {
    let db = match Database::new().await {
        Ok(db) => db,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error").into_response();
        }
    };

    let arch = &payload.arch;

    let archs = vec!["x86_64", "aarch64"];
    if !archs.contains(&arch.as_str()) {
        return (
        StatusCode::BAD_REQUEST,
        format!(
            "Invalid architecture specified. Only {} are supported.",
            archs.join(" and ")
        ),
        ).into_response();
    }
    
    let mut used_ram: i32 = 0;
    let mut used_cpu: i32 = 0;

    match payload.vms.len() {
        0 => {
            used_ram = 1;
            used_cpu = 1;
        }
        _ => {
            for vm in payload.vms.iter() {
                used_ram += 1 + vm.memory;
                used_cpu += 1 + vm.cpu;
            }
        }
    }

    match Database::get_hypervisor_by_hostname(&db, &payload.hostname).await {
        Ok(Some(id)) => {
            if let Err(e) = Database::update_hypervisor(&db, &id, &used_ram, &used_cpu, payload.vms.len() as i32).await {
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update hypervisor: {}", e)).into_response();
            }
            
            let mut errors = Vec::new();
            for vm in &payload.vms {
                let agent_state = vm.state.clone();
                
                let name_tenant: Vec<&str> = vm.name.split('-').collect();
                if name_tenant.len() != 2 {
                    continue;
                }

                let tenant = name_tenant[0];
                let name = name_tenant[1];

                match Database::get_tenant_by_name(&db, &tenant).await {
                    Ok(Some(tenant_uuid)) => {
                        match Database::get_virtual_machine_by_name(&db, &name, &tenant_uuid).await {
                            Ok(vm_on_db) => {
                                match vm_on_db {
                                    Some(vm_on_db) => {
                                        if vm_on_db.state != agent_state {
                                            if let Err(e) = Database::update_vm_state(&db, &name, &tenant_uuid, &agent_state.to_lowercase()).await {
                                                errors.push(format!("Failed to update VM '{}': {}", name, e));
                                            }
                                        }

                                        if vm_on_db.ip_addresses.iter().collect::<HashSet<_>>() != vm.ip_addresses.iter().collect::<HashSet<_>>() {
                                            if let Err(e) = Database::update_vm_ip_addr(&db, &name, &tenant_uuid, &vm.ip_addresses).await {
                                                errors.push(format!("Failed to update VM '{}' IP addresses: {}", name, e));
                                            }
                                        }
                                    },
                                    None => {}
                                }
                            },
                            Err(e) => {
                                errors.push(format!("Failed to fetch VM state '{}': {}", name, e));
                            }
                        }                    },
                    Ok(None) => {
                        errors.push(format!("Tenant '{}' not found.", tenant));
                        continue;
                    },
                    Err(e) => {
                        errors.push(format!("Failed to fetch tenant '{}': {}", tenant, e));
                        continue;
                    }
                }
            }
            
            if errors.is_empty() {
                (StatusCode::OK, "Hypervisor and all VMs updated successfully.".to_string()).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Hypervisor updated but had VM errors: {}", errors.join("; "))).into_response()
            }
        },
        Ok(None) => {
            match Database::hypervisor_register(&db, &payload.hostname, &payload.memory, &payload.cpu,
                                              used_ram, used_cpu, &arch, payload.vms.len() as i32).await {
                Ok(_) => (StatusCode::OK, format!("Hypervisor '{}' registered successfully.", &payload.hostname)).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to register hypervisor: {}", e)).into_response()
            }
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get hypervisor: {}", e)).into_response()
    }
}

async fn create_ssh_pub_key(Json(payload): Json<SSHKey>) -> impl IntoResponse {
    let db = match Database::new().await {
        Ok(db) => db,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error").into_response();
        }
    };

    match (payload.name, payload.ssh_pub_key, payload.tenant) {
        (Some(name), Some(ssh_pub_key), Some(tenant)) => {
            let public_key = PublicKey::from_openssh(&ssh_pub_key);
            let fingerprint: ssh_key::Fingerprint = match public_key {
                Ok(public_key) => {
                    public_key.fingerprint(Default::default())
                }
                Err(e) => return (StatusCode::BAD_REQUEST, format!("Invalid SSH public key: {}", e)).into_response(),
            };

            let create_ssh_pub_key: Result<(), sqlx::Error> = Database::create_ssh_pub_key(&db, &name, &ssh_pub_key, &fingerprint.to_string(), &tenant).await;
            match create_ssh_pub_key {
                Ok(_) => (StatusCode::OK, format!("SSH public key '{}' created successfully.", &name)).into_response(),
                // Potential here for: insert or update on table "ssh_pub_keys" violates foreign key constraint "fk_resource_tenant"
                // in case for example the tenant UUID doesn't exist
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create SSH public key: {}", e)).into_response(),
            }
        },
        _ => (StatusCode::BAD_REQUEST, format!("SSH public key create request must include a name and SSH public key.")).into_response(),
    }
}

async fn delete_ssh_pub_key(Json(payload): Json<SSHKey>) -> impl IntoResponse {
    let db = match Database::new().await {
        Ok(db) => db,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error").into_response();
        }
    };

    match payload.name {
        Some(name) => {
            let delete_ssh_pub_key: Result<(), sqlx::Error> = Database::delete_ssh_pub_key(&db, &name).await;
            match delete_ssh_pub_key {
                Ok(_) => (StatusCode::OK, format!("SSH public key with name '{}' deleted successfully.", &name)).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete SSH public key: {}", e)).into_response(),
            }
        },
        _ => (StatusCode::BAD_REQUEST, format!("SSH public key delete request must use a valid SSH public key name.")).into_response(),
    }
}

async fn list_ssh_pub_keys(Json(payload): Json<Tenant>) -> impl IntoResponse {
    let db = match Database::new().await {
        Ok(db) => db,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error").into_response();
        }
    };

    match (payload.name, payload.id) {
        (_, Some(id)) => {
            let ssh_pub_keys = Database::list_ssh_pub_keys(&db, &id).await.unwrap();
            let ssh_pub_keys = serde_json::to_string(&ssh_pub_keys).unwrap();
            (StatusCode::OK, ssh_pub_keys).into_response()
        },
        _ => (StatusCode::BAD_REQUEST, format!("SSH public key list request must include tenant's UUID.")).into_response(),
    }
}

async fn virtual_machine_scheduler(Json(payload): Json<VirtualMachineCreate>) -> impl IntoResponse {
    let db = match Database::new().await {
        Ok(db) => db,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error").into_response();
        }
    };

    let tenant_uuid = match Database::get_tenant_by_name(&db, &payload.tenant).await {
        Ok(Some(uuid)) => uuid,
        Ok(None) => return (StatusCode::BAD_REQUEST, "Tenant not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)).into_response(),
    };

    let is_vm_existing = Database::get_virtual_machine_by_name(&db, &payload.name, &tenant_uuid).await;
    match is_vm_existing {
        Ok(Some(_)) => return (StatusCode::BAD_REQUEST, format!("VM '{}' already exists.", &payload.name)).into_response(),
        Ok(None) => (),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)).into_response(),
    }

    let valid_networking = vec!["l2-tenant", "l2-bridged"];
    if !valid_networking.contains(&payload.networking.as_str()) {
        return (StatusCode::BAD_REQUEST, format!("Invalid networking type, valid modes are: {}", valid_networking.join(", "))).into_response();
    }

    let mut provider_network_name = Option::None;
    let mut provider_network_vlan: Option<String> = Option::None;
    match &payload.network {
        Some(network) => {
            if payload.networking != "l2-bridged" {
                return (StatusCode::BAD_REQUEST, "Network can only be specified for the l2-bridged networking mode").into_response();
            }

            let is_network_existing = Database::get_provider_network(&db, &network).await;
            match is_network_existing {
                Ok(Some(provider)) => {
                    provider_network_name = Some(provider.name);
                    provider_network_vlan = Some(provider.vlan.to_string());
                }
                Ok(None) => return (StatusCode::BAD_REQUEST, format!("Network '{}' does not exist.", &network)).into_response(),
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)).into_response(),
            }
        }
        None => {
            if payload.networking == "l2-bridged" {
                return (StatusCode::BAD_REQUEST, "Network must be specified for 'l2-bridged' networking.").into_response();
            }
        }
    }

    let mut target_hypervisor: String = String::new();
    let mut target_hypervisor_uuid: Uuid = Uuid::nil();
    let target_hypervisors = Database::get_hypervisors_min_hosted_vms(&db, &payload.arch).await.unwrap();

    for hypervisor in target_hypervisors.iter() {
        if hypervisor.total_ram - hypervisor.used_ram >= payload.ram && hypervisor.total_cpu - hypervisor.used_cpu >= payload.cpu {
            target_hypervisor = hypervisor.hostname.clone();
            target_hypervisor_uuid = hypervisor.id;
            break;
        }
    }

    if target_hypervisor.is_empty() && target_hypervisor_uuid.is_nil() {
        return (StatusCode::BAD_REQUEST, "No hypervisor available with enough resources to schedule VM.").into_response();
    }

    let mac_addr = generate_mac_address().await;
    let mac_addr_as_string = format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        mac_addr[0], mac_addr[1], mac_addr[2],
        mac_addr[3], mac_addr[4], mac_addr[5]
      ).to_lowercase();

    let mut create_vm_query = json!({
        "name": payload.name,
        "memory": payload.ram * 1024,
        "cpu": payload.cpu,
        "os": payload.os,
        "disk": payload.disk_size,
        "ssh_pub_key": payload.ssh_pub_key,
        "tenant": payload.tenant,
        "mac_addr": mac_addr_as_string,
        "networking": payload.networking,
    });

    if payload.networking == "l2-tenant" {
        let ls_name = format!("{}-{}", &tenant_uuid, &payload.vpc);
        let lsp_port_name = format!("{}-{}", &payload.tenant, &payload.name);
        let response = add_lsp_to_ls(&lsp_port_name, &ls_name).await;
        match response {
            Ok(response) => {
                let port_uuid = extract_uuid_from_response(&response).await;
                match port_uuid {
                    Ok(port_uuid) => {
                        let add_mac_to_lsp = add_mac_to_lsp(&port_uuid, &mac_addr_as_string).await;
                        match add_mac_to_lsp {
                            Ok(_) => (),
                            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to add MAC to LSP: {}", e)).into_response(),
                        }

                        let cidr = Database::get_vpc_cidr(&db, &payload.vpc, &tenant_uuid).await;
                        match cidr {
                            Ok(Some(cidr)) => {
                                let dhcpv4_options = get_dhcpv4_options_id(&cidr).await;
                                match dhcpv4_options {
                                    Ok(dhcpv4_options) => {
                                        let add_dhcpv4_options = add_dhcp_options_to_lsp(&port_uuid, &dhcpv4_options).await;
                                        match add_dhcpv4_options {
                                            Ok(_) => (),
                                            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to add DHCPv4 options to LSP: {}", e)).into_response(),
                                        }                            
                                    }
                                    Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)).into_response(),
                                }                     
                            }
                            Ok(None) => return (StatusCode::BAD_REQUEST, "VPC CIDR not found").into_response(),
                            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)).into_response(),
                        }
                    }
                    Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to extract UUID from OVN response: {}", e)).into_response(),
                }
            }
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create logical port: {}", e)).into_response(),
        }
    } else if payload.networking == "l2-bridged" {
        create_vm_query["network"] = json!(provider_network_vlan);
    }

    let client = Client::new();
    let create_vm_response = client.post(format!("http://{}:3000/virtualmachine/create", target_hypervisor))
        .header("Content-Type", "application/json")
        .body(create_vm_query.to_string())
        .send()
        .await;

    match create_vm_response {
        Ok(response) => {
            match response.status() {
                StatusCode::OK => {
                    let vpc_uuid = match Database::get_vpc_by_name(&db, &payload.vpc, &tenant_uuid).await {
                        Ok(Some(uuid)) => uuid,
                        Ok(None) => return (StatusCode::BAD_REQUEST, "VPC not found").into_response(),
                        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)).into_response(),
                    };
                
                    let tenant_uuid = match Database::get_tenant_by_name(&db, &payload.tenant).await {
                        Ok(Some(uuid)) => uuid,
                        Ok(None) => return (StatusCode::BAD_REQUEST, "Tenant not found").into_response(),
                        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)).into_response(),
                    };

                    let pub_ssh_key_uuid = match Database::get_ssh_key(&db, &payload.ssh_pub_key).await {
                        Ok(Some(uuid)) => uuid,
                        Ok(None) => return (StatusCode::BAD_REQUEST, "SSH public key not found").into_response(),
                        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)).into_response(),
                    };
                
                    match Database::create_virtual_machine(
                        &db, &payload.name, &payload.cpu, &payload.ram,
                        &tenant_uuid, &vpc_uuid, &pub_ssh_key_uuid,
                        &payload.disk_size, &target_hypervisor_uuid,
                        &payload.os, "created", &payload.networking,
                        provider_network_name
                    ).await {
                        Ok(_) => (StatusCode::OK, format!("VM '{}' created successfully.", &payload.name)).into_response(),
                        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to add VM to database: {}", e)).into_response(),
                    }
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create VM on hypervisor '{}'.", &target_hypervisor)).into_response(),
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to connect to hypervisor compute API '{}': {}", &target_hypervisor, e)).into_response(),
    }
}

async fn delete_vm_handler(Json(payload): Json<VirtualMachineDelete>) -> impl IntoResponse {
    let db = match Database::new().await {
        Ok(db) => db,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error").into_response();
        }
    };
    
    let tenant_uuid: Uuid = match Database::get_tenant_by_name(&db, &payload.tenant).await {
        Ok(Some(uuid)) => uuid,
        Ok(None) => return (StatusCode::BAD_REQUEST, "Tenant not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)).into_response(),
    };

    match Database::get_virtual_machine_by_name(&db, &payload.name, &tenant_uuid).await {
        Ok(Some(vm)) => {
            let tenant_name = Database::get_tenant_by_id(&db, &vm.tenant).await;
            match tenant_name {
                Ok(Some(tenant_name)) => {
                    let port_name = format!("{}-{}", &tenant_name, &vm.name);
                    if let Ok(Some(vpc)) = Database::get_vpc_by_id(&db, &vm.vpc, &tenant_uuid).await {
                        let ls_name = format!("{}-{}", &tenant_uuid, &vpc);
                        let delete_lsp = remove_lsp(&port_name, &ls_name).await;
                        match delete_lsp {
                            Ok(_) => (),
                            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete LSP: {}", e)).into_response(),
                        }
                    } else {
                        return (StatusCode::BAD_REQUEST, "VPC not found").into_response();
                    }
                }
                Ok(None) => return (StatusCode::BAD_REQUEST, "Tenant not found").into_response(),
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)).into_response(),
            }

            let delete_vm_query = json!({
                "name": payload.name,
                "tenant": payload.tenant,
            });

            let hypervisor_hostname = match Database::get_hypervisor_by_id(&db, &vm.hypervisor).await {
                Ok(Some(hypervisor_hostname)) => hypervisor_hostname,
                Ok(None) => return (StatusCode::BAD_REQUEST, format!("Hypervisor '{}' not found.", &vm.hypervisor)).into_response(),
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)).into_response(),
            };

            let client = Client::new();
            let delete_vm_response = client.post(format!("http://{}:3000/virtualmachine/delete", &hypervisor_hostname))
                .header("Content-Type", "application/json")
                .body(delete_vm_query.to_string())
                .send()
                .await;

            match delete_vm_response {
                Ok(response) => {
                    match response.status() {
                        StatusCode::OK => {
                            match Database::delete_virtual_machine(&db, &payload.name).await {
                                Ok(_) => (StatusCode::OK, format!("VM '{}' deleted successfully.", &payload.name)).into_response(),
                                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete VM from database: {}", e)).into_response(),
                            }
                        }
                        _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete VM on hypervisor '{}'.", &vm.hypervisor)).into_response(),
                    }
                }
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to connect to hypervisor compute API '{}': {}", &vm.hypervisor, e)).into_response(),
            }
        }
        Ok(None) => (StatusCode::BAD_REQUEST, format!("VM '{}' not found.", &payload.name)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)).into_response(),
    }
}

async fn list_vm_handler(Json(payload): Json<Tenant>) -> impl IntoResponse {
    let db = match Database::new().await {
        Ok(db) => db,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error").into_response();
        }
    };

    match payload.id {
        Some(id) => {
            let vms = Database::list_virtual_machines(&db, &id).await.unwrap();
            let vms = serde_json::to_string(&vms).unwrap();
            (StatusCode::OK, vms).into_response()
        },
        _ => (StatusCode::BAD_REQUEST, format!("VM list request must include a tenant ID.")).into_response(),
    }
}

async fn list_provider_networks_handler() -> impl IntoResponse {
    let db = match Database::new().await {
        Ok(db) => db,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error").into_response();
        }
    };

    let provider_networks = match Database::list_provider_networks(&db).await {
        Ok(networks) => networks,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error fetching networks").into_response();
        }
    };

    let provider_networks_json = match serde_json::to_string(&provider_networks) {
        Ok(json) => json,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error serializing data").into_response();
        }
    };

    (StatusCode::OK, provider_networks_json).into_response()
}

async fn create_provider_network_handler(Json(payload): Json<ProviderNetwork>) -> impl IntoResponse {
    let db = match Database::new().await {
        Ok(db) => db,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error").into_response();
        }
    };

    if payload.name.trim().is_empty() || !(1..=4094).contains(&payload.vlan) || payload.subnet.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "Provider network must include a non-empty name and a valid VLAN (1–4094).".to_string(),
        ).into_response();
    }

    match Database::create_provider_network(&db, &payload.name, &payload.vlan, &payload.subnet).await {
        Ok(_) => (
            StatusCode::OK,
            format!("Provider network '{}' created successfully.", payload.name),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create provider network: {}", e),
        ).into_response(),
    }
}

async fn delete_provider_network_handler(Json(payload): Json<ProviderNetworkDelete>) -> impl IntoResponse {
    let db = match Database::new().await {
        Ok(db) => db,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error").into_response();
        }
    };

    if payload.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "Provider network delete request must include a non-empty name.".to_string(),
        ).into_response();
    }

    match Database::delete_provider_network(&db, &payload.name).await {
        Ok(_) => (
            StatusCode::OK,
            format!("Provider network '{}' deleted successfully.", payload.name),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to delete provider network: {}", e),
        ).into_response(),
    }
}

async fn list_hypervisors_handler() -> impl IntoResponse {
    let db = match Database::new().await {
        Ok(db) => db,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error").into_response();
        }
    };

    let hypervisors = match Database::list_hypervisors(&db).await {
        Ok(hypervisors) => hypervisors,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error fetching hypervisors").into_response();
        }
    };

    let hypervisors_json = match serde_json::to_string(&hypervisors) {
        Ok(json) => json,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error serializing data").into_response();
        }
    };

    (StatusCode::OK, hypervisors_json).into_response()
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 - Not Found")
}

// Once the scheduler will provision a VM against the target hypervisor
// the hypervisor should also return the MAC address it used as it's required
// by the logical switcvh to create a port, even better, to generate the MAC
// in controlplane and feed it to the hypervisor to avoid the back and forth.
// Idea is to get the VM request from the API, generate the MAC, append to the
// JSON and send the payload to the target hypervisor.

// The scheduler once identified a target hypervisor will also have to handle
// OVN control plane pieces (i.e LS and LSP creation). At the same time when a VM
// will be deleted OVN cleanup also has to be performed.
// Primarily: scheduler --> generate MAC --> create LS and LSP --> generate VM payload
// --> send to hypervisor --> hypervisor creates VM

//payload {"hostname":"sweetrevenge","memory":7,"cpu":4,"vms":[]}
//payload {"hostname":"averi-thinkpadp1gen4i.newyork.csb","memory":62,"cpu":16,"vms":[{"name":"crc","memory":9,"cpu":4,"state":"Shutoff"},{"name":"fedora40","memory":4,"cpu":2,"state":"Shutoff"}]}
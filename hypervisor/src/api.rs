// Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
// GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

mod libvirt;
mod ovs;

use crate::api::libvirt::VmDomain;

use axum::{
    extract::Json, http::StatusCode, response::IntoResponse, routing::post, Router
};
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse};


#[derive(serde::Serialize, serde::Deserialize)]
struct VirtualMachine {
    name: String,
    memory: u64,
    cpu: u32,
    os: String,
    ssh_pub_key: String,
    disk: u32,
    tenant: String,
    mac_addr: String,
    networking: String,
    network: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct VirtualMachineDelete {
    name: String,
    tenant: String,
}

pub struct HypervisorApi {}

impl HypervisorApi {
    pub async fn router() -> Router {
        tracing_subscriber::fmt()
        .with_env_filter("axum=debug,tower_http=debug")
        .init();

        Router::new()
            .route("/virtualmachine/create", post(create_vm_handler))
            .route("/virtualmachine/delete", post(delete_vm_handler))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::new().level(tracing::Level::DEBUG)) 
                    .on_response(DefaultOnResponse::new().level(tracing::Level::DEBUG)),
            )
    }

    pub async fn start_server(app: Router) -> Result<(), Box<dyn std::error::Error>> {
        let listener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
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

async fn create_vm_handler(Json(payload): Json<VirtualMachine>) -> impl IntoResponse {
    let vm = serde_json::to_string(&payload).unwrap();

    let os = match payload.os.as_str() {
        "rhel9" | "fedora41" => payload.os,
        _ => return (StatusCode::BAD_REQUEST, "Invalid OS specified. Only 'rhel9' and 'fedora41' are supported.".to_string()),
    };

    let create_vm = VmDomain::create_vm(payload.name, payload.memory, payload.cpu, os, payload.ssh_pub_key, payload.disk, payload.tenant, payload.mac_addr, payload.networking, payload.network).await;
    match create_vm {
        Ok(_) => (StatusCode::OK, format!("VM creation started successfully with specs: {}", vm)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create VM: {}", e)),
    }
}

async fn delete_vm_handler(Json(payload): Json<VirtualMachineDelete>) -> impl IntoResponse {
    let vm = serde_json::to_string(&payload).unwrap();

    let delete_vm: Result<(), virt::error::Error> = VmDomain::delete_vm(payload.name, payload.tenant).await;
    match delete_vm {
        Ok(_) => (StatusCode::OK, format!("VM with name '{}' deleted successfully.", vm)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete VM: {}", e)),
    }
}
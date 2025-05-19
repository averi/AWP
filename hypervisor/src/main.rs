// Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
// GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

mod agent;
mod api;

use agent::Hypervisor;
use api::HypervisorApi;

use reqwest::Client;


#[derive(serde::Deserialize, Debug)]
struct Config {
    compute: ComputeAPI,
}

#[derive(serde::Deserialize, Debug)]
struct ComputeAPI {
    host: String,
    port: u16,
    path: String,
    protocol: String,
}

fn read_conf_file(config_file: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let file = std::fs::read_to_string(config_file)?;
    let config: Config = serde_yaml::from_str(&file)?;
    Ok(config)
}

async fn post_agent_stats(json: &str) -> Result<(), reqwest::Error> {
    let config = match read_conf_file("config.yaml") {
        Ok(config) => {
            //println!("{:?}", config);
            config
        }
        Err(e) => {
            eprintln!("Failed to read config file: {}", e);
            return Ok(());
        }
    };

    let client = Client::new();
    let url = format!("{}://{}:{}{}", config.compute.protocol, config.compute.host, config.compute.port, config.compute.path);
    let response = client.post(&url)
        .header("Content-Type", "application/json")
        .body(json.to_string())
        .send()
        .await?;

    response.error_for_status()?;

    Ok(())
}

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;

            match Hypervisor::new() {
                Ok(hv) => match hv.to_json() {
                    Ok(json) => {
                        if let Err(e) = post_agent_stats(&json).await {
                            eprintln!("Cannot POST hypervisor stats: {} with payload {}", e, &json);
                        }
                    }
                    Err(err) => eprintln!("Error serializing JSON: {}", err),
                },
                Err(err) => eprintln!("Error creating Hypervisor: {}", err),
            }
        }
    });

    let app = HypervisorApi::router().await;
    let server = HypervisorApi::start_server(app).await;
    match server {
        Ok(_) => println!("Server started!"),
        Err(err) => eprintln!("Error starting server: {}", err),
    }
}
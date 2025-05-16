# Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
# GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

mod api;

use api::ControlPlaneAPI;


#[tokio::main]
async fn main() {
    let app = ControlPlaneAPI::router().await;
    let server = ControlPlaneAPI::start_server(app).await;
    match server {
        Ok(_) => println!("Server started!"),
        Err(err) => eprintln!("Error starting server: {}", err),
    }
}
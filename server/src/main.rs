use serde::{self, Deserialize};
use std::path::Path;
use std::fs::File;
use actix_web::{HttpServer, web};
use flow_utils::exit_with;
use core::State;
use std::time::Instant;
use std::collections::HashMap;

const SETTINGS_FILE_PATH: &'static str = "settings.toml";

#[derive(Deserialize)]
struct Settings {
    ip: String,
    port: String
}

fn read_bind_address_from_settings(settings_file_path: &Path) -> Result<String, String> {
    let mut file = File::open(settings_file_path)
        .map_err(|_| format!("Unable to open the file \"{}\"", settings_file_path.display()))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|_| format!("Unable to read the file \"{}\"", settings_file_path.display()))?;
    let mut settings: Settings = toml::from_str(&contents)
        .map_err(|_| format!("Invalid syntax in settings file \"{}\". Syntax must follow the TOML specification.", settings_file_path.display()))?;
    let port = settings.port.parse::<u16>()
        .map_err(|_| format!("Invalid port!"))?;
    Ok(format!("{}:{}", settings.ip, port))
}

struct InfrastructureStatus(HashMap<String, GroupStatus>);

impl InfrastructureStatus {
    fn refresh(&mut self, group_name: &str, computer_name: &str) -> State {
        State::Online
    }

    fn shutdown_computer(&mut self, group_name: &str, computer_name: &str) -> () {

    }

    fn shutdown_group(&mut self, group_name: &str, computer_name: &str) -> () {

    }

    fn cleanup(&mut self) -> () {

    }
}

struct GroupStatus(HashMap<String, ComputerStatus>);

impl GroupStatus {
    fn may_request_shutdown(&self) -> bool {
        for (computer_name, computer_status) in self.0.iter() {
            if computer_status.may_request_shutdown() {
                return true
            }
        }
        false
    }

    fn cleanup(&mut self) -> () {

    }
}

struct ComputerStatus {
    state: State,
    last_pong_date: Instant
}

impl ComputerStatus {
    fn update_pong_date(&mut self) -> () {
        self.last_pong_date = Instant::now();
    }

    fn may_request_shutdown(&self) -> bool {
        return self.state == State::Online;
    }

    fn request_shutdown(&mut self) -> () {
        if self.state == State::Online {
            self.state = State::ShutdownRequested
        }
    }

    fn accept_shutdown(&mut self) -> () {
        if self.state == State::ShutdownRequested {
            self.state = State::ShutdownAccepted
        }
    }
}

struct Statuses(HashMap<String, HashMap<String, Status>>);



#[actix_rt::main]
async fn main() {
    // TODO: actix middleware for authentication
    let bind_address = read_bind_address_from_settings(Path::new(SETTINGS_FILE_PATH))
        .unwrap_or_else(exit_with!(1, "{}"));

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index_handler))
            .route("/api/pong", web::get().to(pong_handler))
            .route("/api/shutdown", web::post().to(shutdown_handler))
            .route("/api/status", web::get().to(status_handler))
    })
        .bind(&bind_address)
        .run()
        .await
}

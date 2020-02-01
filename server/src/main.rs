use std::{
    mem,
    io::{
        self,
        Read
    },
    fs::File,
    collections::HashMap,
    sync::{
        Mutex,
        Arc,
    },
    path::Path,
    convert::TryFrom,
    time::{
        Instant,
        Duration,
    },
};

use actix_web::{
    App,
    Responder,
    HttpResponse,
    HttpServer,
    web::{
        self,
        Data,
        Query,
        Json,
    },
};
use actix_files::{
    self,
    NamedFile,
};
use actix_cors::Cors;
use actix_htpasswd::{
    AuthControl,
    HtpasswdDatabase,
    user_control_policy::{
        AnyLoggedUser,
        Anyone,
    },
};
use serde::{
    self,
    Serialize,
    Deserialize,
};
use serde_json;
use flow_utils::exit_with;

use core::{
    State,
    PONG_SLEEP_SECONDS
};

const SETTINGS_FILE_PATH: &'static str = "settings.toml";
const HTPASSWD_FILE_PATH: &'static str = ".htpasswd";

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
    let settings: Settings = toml::from_str(&contents)
        .map_err(|_| format!("Invalid syntax in settings file \"{}\". Syntax must follow the TOML specification.", settings_file_path.display()))?;
    let port = settings.port.parse::<u16>()
        .map_err(|_| format!("Invalid port!"))?;
    Ok(format!("{}:{}", settings.ip, port))
}

#[derive(Serialize)]
struct InfrastructureStatus(HashMap<String, GroupStatus>);

impl InfrastructureStatus {
    fn new() -> InfrastructureStatus {
        InfrastructureStatus(HashMap::new())
    }

    fn add_fresh(&mut self, group_name: &str, computer_name: &str) -> () {
        match self.get_mut_group_status(group_name) {
            Ok(group_status) => group_status.add_fresh(computer_name),
            Err(_) => {
                self.0.insert(group_name.to_owned(), GroupStatus::new());
                self.add_fresh(group_name, computer_name)
            }
        }
    }

    fn refresh(&mut self, group_name: &str, computer_name: &str) -> State {
        match self.get_mut_computer_status(group_name, computer_name) {
            Ok(computer_status) => {
                computer_status.update_pong_date();
                if computer_status.should_shutdown() {
                    computer_status.accept_shutdown();
                    State::ShutdownRequested
                } else {
                    computer_status.state
                }
            },
            Err(_) => {
                self.add_fresh(group_name, computer_name);
                self.refresh(group_name, computer_name)
            }
        }
    }

    fn request_shutdown_computer(&mut self, group_name: &str, computer_name: &str) -> Result<(), String> {
        self.get_mut_computer_status(group_name, computer_name)
            .and_then(|computer_status| computer_status.request_shutdown())
    }

    fn request_shutdown_group(&mut self, group_name: &str) -> Result<(), String> {
        self.get_mut_group_status(group_name)
            .and_then(|group_status| group_status.request_shutdown())
    }

    fn get_mut_group_status(&mut self, group_name: &str) -> Result<&mut GroupStatus, String> {
        self.0.get_mut(group_name)
            .ok_or_else(|| format!("No group with name \"{}\"", group_name))
    }

    fn get_mut_computer_status(&mut self, group_name: &str, computer_name: &str) -> Result<&mut ComputerStatus, String> {
        self.get_mut_group_status(group_name)
            .and_then(|group_status| group_status.get_mut_computer_status(computer_name))
    }

    fn cleanup(&mut self) -> () {
        let mut _self = Self::new();
        mem::swap(self, &mut _self);
        self.0 = _self.0.into_iter()
            .filter(|(_, group_status)| group_status.has_members())
            .collect();
        self.0.iter_mut()
            .for_each(|(_, group_status)| group_status.cleanup());
    }
}

#[derive(Serialize)]
struct GroupStatus(HashMap<String, ComputerStatus>);

impl GroupStatus {
    fn new() -> GroupStatus {
        GroupStatus(HashMap::new())
    }

    fn add_fresh(&mut self, computer_name: &str) -> () {
        if self.get_mut_computer_status(computer_name).is_err() {
            self.0.insert(computer_name.to_owned(), ComputerStatus::new());
        }
    }

    fn may_request_shutdown(&self) -> bool {
        self.0.iter()
            .any(|(_, computer_status)| computer_status.may_request_shutdown())
    }

    fn request_shutdown(&mut self) -> Result<(), String> {
        if self.may_request_shutdown() {
            self.0.iter_mut()
                .for_each(|(_, computer_status)| {
                    if computer_status.may_request_shutdown() { computer_status.request_shutdown().unwrap() }
                });
            Ok(())
        } else {
            Err(format!("Unable to shutdown a group where no computer is in the online state"))
        }
    }

    fn has_members(&self) -> bool {
        return !self.0.is_empty();
    }

    fn get_mut_computer_status(&mut self, computer_name: &str) -> Result<&mut ComputerStatus, String> {
        self.0.get_mut(computer_name)
            .ok_or_else(|| format!("No computer with name \"{}\" in this group", computer_name))
    }

    fn cleanup(&mut self) -> () {
        let cleanup_threshold = Duration::from_secs(4 * PONG_SLEEP_SECONDS);
        let mut _self = Self::new();
        mem::swap(self, &mut _self);
        self.0 = _self.0.into_iter()
            .filter(|(_, computer_status)| {
                computer_status.last_pong_date.elapsed() < cleanup_threshold
            })
            .collect();
    }
}

#[derive(Serialize)]
struct ComputerStatus {
    state: State,
    #[serde(skip_serializing)]
    last_pong_date: Instant
}

impl ComputerStatus {
    fn new() -> ComputerStatus {
        ComputerStatus {
            state: State::Online,
            last_pong_date: Instant::now()
        }
    }

    fn update_pong_date(&mut self) -> () {
        self.last_pong_date = Instant::now();
    }

    fn may_request_shutdown(&self) -> bool {
        self.state == State::Online
    }

    fn request_shutdown(&mut self) -> Result<(), String> {
        if self.may_request_shutdown() {
            self.state = State::ShutdownRequested;
            Ok(())
        } else {
            Err(format!("Unable to shutdown a computer which is not in the online state"))
        }
    }

    fn should_shutdown(&self) -> bool {
        self.state == State::ShutdownRequested
    }

    fn accept_shutdown(&mut self) -> () {
        self.state = State::ShutdownAccepted
    }
}

async fn index_handler(_auth_control: AuthControl<Anyone>) -> impl Responder {
    NamedFile::open("static/index.html")
}

#[derive(Deserialize)]
struct PongSelector {
    group_name: String,
    computer_name: String
}

async fn pong_handler(data: InfrastructureData, pong_selector: Query<PongSelector>, _auth_control: AuthControl<Anyone>) -> impl Responder {
    match data.lock() {
        Ok(mut infrastructure_status) => {
            let state = infrastructure_status.refresh(&pong_selector.group_name, &pong_selector.computer_name);
            HttpResponse::Ok()
                .body(serde_json::to_string(&state).unwrap())
        },
        Err(_) => {
            HttpResponse::InternalServerError()
                .body(format!("Unable to acquire the lock"))
        }
    }
}

#[derive(Deserialize)]
struct ShutdownSelector {
    group_name: String,
    computer_name: Option<String>
}

async fn shutdown_handler(data: InfrastructureData, shutdown_selector: Json<ShutdownSelector>, _auth_control: AuthControl<AnyLoggedUser>) -> impl Responder {
    match data.lock() {
        Ok(mut infrastructure_status) => {
            let result = match shutdown_selector.computer_name.as_ref() {
                Some(computer_name) => infrastructure_status.request_shutdown_computer(&shutdown_selector.group_name, &computer_name),
                None => infrastructure_status.request_shutdown_group(&shutdown_selector.group_name)
            };
            match result {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(msg) => HttpResponse::BadRequest().body(msg)
            }
        },
        Err(_) => {
            HttpResponse::InternalServerError()
                .body(format!("Unable to acquire the lock"))
        }
    }
}

async fn status_handler(data: InfrastructureData, _auth_control: AuthControl<AnyLoggedUser>) -> impl Responder {
    match data.lock() {
        Ok(mut infrastructure_status) => {
            infrastructure_status.cleanup();
            HttpResponse::Ok().body(serde_json::to_string(&*infrastructure_status).unwrap())
        },
        Err(_) => {
            HttpResponse::InternalServerError()
                .body(format!("Unable to acquire the lock"))
        }
    }
}

type InfrastructureData = Data<Arc<Mutex<InfrastructureStatus>>>;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    let bind_address = read_bind_address_from_settings(Path::new(SETTINGS_FILE_PATH))
        .unwrap_or_else(exit_with!(1, "{}"));

    let data = Arc::new(Mutex::new(InfrastructureStatus::new()));

    HttpServer::new(move || {
        let htpasswd_database = HtpasswdDatabase::try_from(Path::new(HTPASSWD_FILE_PATH))
            .unwrap_or_else(exit_with!(1, "{}"));

        App::new()
            // TODO: remove
            .wrap(
                Cors::new()
                    .allowed_origin("http://localhost:10081")
                    .supports_credentials()
                    .finish()
            )
            .data(data.clone())
            .data(htpasswd_database)
            .service(actix_files::Files::new("/static", "static"))
            .route("/api/pong", web::get().to(pong_handler))
            .route("/api/shutdown", web::post().to(shutdown_handler))
            .route("/api/status", web::get().to(status_handler))
            .route("/", web::get().to(index_handler))
    })
        .bind(&bind_address)?
        .run()
        .await
}

mod status;

use std::{
    io::Read,
    fs::File,
    sync::{
        Mutex,
        Arc,
    },
    path::Path,
    convert::TryFrom,
};

use futures::future::TryFutureExt;
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
use actix_files;
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
    Deserialize,
};
use serde_json;
use flow_utils::process::{
    ProcessResult,
    ProcessError::{
        Unrecoverable,
        InvalidData,
    },
};

use status::InfrastructureStatus;
use std::convert::TryInto;

const SETTINGS_FILE_PATH: &'static str = "settings.toml";
const HTPASSWD_FILE_PATH: &'static str = ".htpasswd";

static INDEX_HTML_TEMPLATE: &'static str = r#"
<!DOCTYPE html>
<html lang="fr">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no, minimal-ui">
        <title>Shutdown On Lan</title>
    </head>
    <body>
        <div id="app-container"></div>
        <script type="text/javascript">
            window.API_ROOT_URL = '%% public_address %%/api';
        </script>
        <script type="text/javascript" src="static/bundle.js"></script>
    </body>
</html>
"#;

#[derive(Deserialize)]
struct RawSettings {
    bind_ip: String,
    bind_port: String,
    public_full_address: String,
}

struct Settings {
    bind_ip: String,
    bind_port: u16,
    public_full_address: String,
}

impl TryFrom<RawSettings> for Settings {
    type Error = String;

    fn try_from(raw_settings: RawSettings) -> Result<Self, Self::Error> {
        let numeric_bind_port: u16 = raw_settings.bind_port.parse()
            .map_err(|_| "Bind port must be an integer between 0 and 65535")?;
        Ok(Settings {
            bind_ip: raw_settings.bind_ip,
            bind_port: numeric_bind_port,
            public_full_address: raw_settings.public_full_address
        })
    }
}

fn read_settings(settings_file_path: &Path) -> Result<Settings, String> {
    let mut file = File::open(settings_file_path)
        .map_err(|_| format!("Unable to open the file \"{}\"", settings_file_path.display()))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|_| format!("Unable to read the file \"{}\"", settings_file_path.display()))?;
    let raw_settings: RawSettings = toml::from_str(&contents)
        .map_err(|_| format!("Invalid syntax in settings file \"{}\". Syntax must follow the TOML specification.", settings_file_path.display()))?;
    let settings = raw_settings.try_into()?;
    Ok(settings)
}

async fn index_handler(public_address_data: Data<String>, _auth_control: AuthControl<Anyone>) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(INDEX_HTML_TEMPLATE.replace("%% public_address %%", public_address_data.get_ref()))
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
async fn main() -> ProcessResult<()> {
    let settings = read_settings(Path::new(SETTINGS_FILE_PATH))
        .map_err(|msg| InvalidData(msg))?;

    let bind_address = format!("{}:{}", settings.bind_ip, settings.bind_port);
    let public_full_address = settings.public_full_address;

    let htpasswd_database = HtpasswdDatabase::try_from(Path::new(HTPASSWD_FILE_PATH))
        .map_err(|msg| InvalidData(msg.to_string()))?;

    let infrastructure_data = Arc::new(Mutex::new(InfrastructureStatus::new()));

    HttpServer::new(move || {
        App::new()
            .data(infrastructure_data.clone())
            .data(public_full_address.clone())
            .data(htpasswd_database.clone())
            .service(actix_files::Files::new("/static", "static"))
            .route("/api/pong", web::get().to(pong_handler))
            .route("/api/shutdown", web::post().to(shutdown_handler))
            .route("/api/status", web::get().to(status_handler))
            .route("/index.html", web::get().to(index_handler))
            .route("/", web::get().to(index_handler))
    })
        .bind(&bind_address)
        .map_err(|msg| Unrecoverable(format!("Unable to bind the server to \"{}\": {}", bind_address, msg)))?
        .run()
        .map_err(|msg| Unrecoverable(format!("Error during server runtime: {}", msg)))
        .await?;
    Ok(())
}

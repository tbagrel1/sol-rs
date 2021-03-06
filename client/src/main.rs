use std::{
    path::Path,
    fs::File,
    time::Duration,
    thread::sleep,
    process::{
        Command,
        Output,
    },
    io::{
        self,
        Read,
    },
};

use percent_encoding::utf8_percent_encode;
use flow_utils::process::{
    run_main_process,
    ProcessResult,
    ProcessError::{
        InvalidData,
        Unrecoverable,
    },
};
use serde::{
    self,
    Deserialize,
};

use core::{
    State,
    PONG_SLEEP_SECONDS,
};


const SETTINGS_FILE_PATH: &'static str = "settings.toml";

#[derive(Deserialize)]
struct Settings {
    api_pong_url: String,
    group_name: String,
    computer_name: String
}

fn read_settings(settings_file_path: &Path) -> Result<Settings, String> {
    let mut file = File::open(settings_file_path)
        .map_err(|_| format!("Unable to open the file \"{}\"", settings_file_path.display()))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|_| format!("Unable to read the file \"{}\"", settings_file_path.display()))?;
    let mut settings: Settings = toml::from_str(&contents)
        .map_err(|_| format!("Invalid syntax in settings file \"{}\". Syntax must follow the TOML specification.", settings_file_path.display()))?;
    if settings.api_pong_url.ends_with("/") {
        settings.api_pong_url.pop();
    }
    Ok(settings)
}

fn pong(settings: &Settings) -> Result<State, String> {
    let url = format!(
        "{}?group_name={}&computer_name={}",
        &settings.api_pong_url,
        utf8_percent_encode(&settings.group_name, percent_encoding::NON_ALPHANUMERIC),
        utf8_percent_encode(&settings.computer_name, percent_encoding::NON_ALPHANUMERIC),
    );
    let state = reqwest::blocking::get(&url)
        .map_err(|msg| format!("Unable to pong the API: <{}>", msg))?
        .json::<State>()
        .map_err(|_| format!("Invalid response format from the API"))?;
    Ok(state)
}

fn shutdown() -> io::Result<Output> {
    let mut cmd;
    if cfg!(target_os = "windows") {
        cmd = Command::new("cmd");
        cmd.args(&["/C", "shutdown -s -t 5"]);
    } else {
        cmd = Command::new("sh");
        cmd.args(&["-c", "poweroff"]);
    }
    cmd.output()
}

#[allow(unreachable_code)]
fn run() -> ProcessResult<()> {
    let settings = read_settings(Path::new(SETTINGS_FILE_PATH))
        .map_err(|msg| InvalidData(format!("{}", msg)))?;
    let error_sleep_duration = Duration::from_secs(1);
    let pong_sleep_duration = Duration::from_secs(PONG_SLEEP_SECONDS);

    loop {
        let state = match pong(&settings) {
            Ok(state) => state,
            Err(msg) => {
                eprintln!("{}", msg);
                sleep(error_sleep_duration);
                continue;
            }
        };

        if state == State::ShutdownRequested {
            println!("Shutdown requested...");
            shutdown()
                .map_err(|msg| Unrecoverable(format!("Unable to shutdown the computer: {}", msg)))?;
        }

        sleep(pong_sleep_duration);
    }
    Ok(())
}

fn main() -> () {
    run_main_process(run);
}

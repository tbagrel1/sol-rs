use serde_utils::serde_str_enum;

pub const PONG_SLEEP_SECONDS: u64 = 4;

serde_str_enum! {
    State {
        Online("online"),
        ShutdownRequested("shutdown_requested"),
        ShutdownAccepted("shutdown_accepted"),
    }
}

use std::{
    collections::HashMap,
    time::{
        Duration,
        Instant,
    },
};

use flow_utils::updatable::Updatable;
use serde::Serialize;

use core::{
    State,
    PONG_SLEEP_SECONDS
};

#[derive(Serialize)]
pub(crate) struct InfrastructureStatus(HashMap<String, GroupStatus>);

impl InfrastructureStatus {
    pub(crate) fn new() -> InfrastructureStatus {
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

    pub(crate) fn refresh(&mut self, group_name: &str, computer_name: &str) -> State {
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

    pub(crate) fn request_shutdown_computer(&mut self, group_name: &str, computer_name: &str) -> Result<(), String> {
        self.get_mut_computer_status(group_name, computer_name)
            .and_then(|computer_status| computer_status.request_shutdown())
    }

    pub(crate) fn request_shutdown_group(&mut self, group_name: &str) -> Result<(), String> {
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

    pub(crate) fn cleanup(&mut self) -> () {
        self.0.self_update(|map| {
            map.into_iter()
                .filter(|(_, group_status)| group_status.has_members())
                .collect()
        });
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
        self.0.self_update(|map| {
            map.into_iter()
                .filter(|(_, computer_status)| {
                    computer_status.last_pong_date.elapsed() < cleanup_threshold
                })
                .collect()
        });
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

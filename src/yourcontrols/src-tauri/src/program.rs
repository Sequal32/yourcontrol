use anyhow::Result;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

use crate::aircraft::DefinitionsUpdater;
use crate::network::{Network, NetworkEvent};
use crate::simulator::Simulator;
use crate::ui::cmd::UiEvents;
use crate::ui::Ui;

use yourcontrols_definitions::DefinitionsParser;
use yourcontrols_types::Payloads;

const DEFINITIONS_PATH: &str = "definitions";

pub struct Program<U> {
    ui: U,
    definitions_parser: DefinitionsParser,
    definitions_updater: DefinitionsUpdater,
    network: Network,
    simulator: Simulator,
}

impl<U: Ui> Program<U> {
    pub fn setup() -> Self {
        Self {
            ui: Ui::run(),
            network: Network::new(),
            definitions_parser: DefinitionsParser::new(),
            definitions_updater: DefinitionsUpdater::new(),
            simulator: Simulator::new(),
        }
    }

    pub fn load_definitions(&mut self, target_definition_sub_path: impl AsRef<Path>) -> Result<()> {
        let mut core_path = PathBuf::from(DEFINITIONS_PATH);
        core_path.push("templates");

        for entry in read_dir(core_path)? {
            let entry = entry?;
            self.definitions_parser.load_file(entry.path())?
        }

        let mut scripts_path = PathBuf::from(DEFINITIONS_PATH);
        scripts_path.push("scripts");

        self.definitions_parser.load_scripts(scripts_path)?;

        let mut definition_path = PathBuf::from(DEFINITIONS_PATH);
        definition_path.push(target_definition_sub_path);

        self.definitions_parser.load_file(definition_path)?;

        Ok(())
    }

    pub fn start_server(&mut self) -> Result<()> {
        self.network.start_direct(27015)
    }

    /// Connects to the simulator and readies the definitions
    pub fn connect_to_simulator(&mut self) -> bool {
        let success = self.simulator.connect("YourControls");

        if !success {
            return false;
        }

        self.simulator.send_message(Payloads::SetMappings {
            datums: self.definitions_parser.get_parsed_datums().clone(),
            vars: self.definitions_parser.get_parsed_vars().clone(),
            events: self.definitions_parser.get_parsed_events().clone(),
            scripts: self.definitions_parser.get_parsed_scripts(),
        });

        return true;
    }

    pub fn process_sim_events(&mut self) -> Result<()> {
        match self.simulator.poll() {
            Ok(Payloads::VariableChange { changed }) => println!("{:?}", changed),
            _ => {}
        };

        Ok(())
    }

    pub fn process_network_events(&mut self) -> Result<()> {
        let events = match self.network.step() {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        for event in events {
            println!("{:?}", event);
            match event {
                NetworkEvent::SessionReceived { session_id } => {
                    self.ui.send_message(UiEvents::LobbyInfo {
                        session_code: Some(session_id),
                        server_ip: None,
                        clients: None,
                    });
                }
                NetworkEvent::Update { changed, time } => {
                    self.simulator.send_message(Payloads::SendIncomingValues {
                        data: changed,
                        time, // TODO: pre-process time
                    });
                }
            }
        }

        Ok(())
    }

    pub fn process_ui_events(&mut self) -> Result<()> {
        let event = match self.ui.next_event() {
            Some(e) => e,
            None => return Ok(()),
        };

        match event {
            UiEvents::UiReady => {
                self.ui.send_message(UiEvents::StartUpText {
                    text: "Starting...".to_string(),
                });

                // self.definitions_updater.fetch_data().expect("OK"); TODO: update repository then uncomment

                self.ui.send_message(UiEvents::InitData {
                    version: std::env::var("CARGO_PKG_VERSION").unwrap(),
                    aircraft: self.definitions_updater.get_all_aircraft_info(),
                });

                self.ui.send_message(UiEvents::LoadingComplete);
            }
            UiEvents::InstallAircraft { names } => {}
            UiEvents::TestNetwork { port } => {}
            UiEvents::Host { port, username } => {
                if let Some(port) = port {
                    self.network.start_direct(port)?;
                } else {
                    self.network.request_session()?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn poll(&mut self) -> Result<()> {
        self.process_ui_events()?;
        self.process_network_events()?;
        self.process_sim_events()?;

        Ok(())
    }
}
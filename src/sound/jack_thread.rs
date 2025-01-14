//! function to create the jack audio thread that will drive audio through the system
//!
//! This thread have a JamEngine moved into it.  That JamEngine will then be called
//! with audio frames from the jack audio system.
//!
//! If a different audio interface were to be used, this code will be replace with some other.
//! The main thing is that the system will callback with frames of input/output audio that get
//! passed into the JamEngine.
use crate::common::box_error::BoxError;

use jack;
use log::{info, error};
// use serde_json::json;
use std::thread::sleep;
use std::time::Duration;

use super::jam_engine::JamEngine;
use super::SoundCallback;

/// call this to start up jack and feed the engine
///
/// it will never return (unless jack blows up)  TODO: Add jack failure recovery
pub fn run(mut engine: JamEngine) -> Result<(), BoxError> {
    // let's open up a jack port
    while engine.is_running() {
        match jack::Client::new("rtjam_rust", jack::ClientOptions::NO_START_SERVER) {
            Ok((client, _status)) => {
                let in_a = client.register_port("rtjam_in_1", jack::AudioIn::default())?;
                let in_b = client.register_port("rtjam_in_2", jack::AudioIn::default())?;
                let mut out_a = client.register_port("rtjam_out_l", jack::AudioOut::default())?;
                let mut out_b = client.register_port("rtjam_out_r", jack::AudioOut::default())?;
                let midi_in = client.register_port("rtjam_midi_input", jack::MidiIn::default())?;
        
                // The callback gets called by jack whenever we have a frame
                let process_callback =
                    move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
                        let in_a_p = in_a.as_slice(ps);
                        let in_b_p = in_b.as_slice(ps);
                        let out_a_p = out_a.as_mut_slice(ps);
                        let out_b_p = out_b.as_mut_slice(ps);

                        // Let the engine process it
                        engine.process_inputs(in_a_p, in_b_p);
                        engine.get_playback_data(out_a_p, out_b_p);
                        let show_p = midi_in.iter(ps);
                        for e in show_p {
                            engine.send_midi_event(e);
                        }
                        jack::Control::Continue
                    };
                let process = jack::ClosureProcessHandler::new(process_callback);

                // Activate the client, which starts the processing.
                let active_client = client.activate_async(Notifications, process)?;

                // Connect system inputs to us and our puts to playback
                active_client
                    .as_client()
                    .connect_ports_by_name("system:capture_1", "rtjam_rust:rtjam_in_1")?;
                // Just catch and ignore error if second input channel connect fails.  Just run with one
                match active_client
                    .as_client()
                    .connect_ports_by_name("system:capture_2", "rtjam_rust:rtjam_in_2")
                {
                    Ok(_) => (),
                    Err(e) => {
                        dbg!(e);
                    }
                }
                active_client
                    .as_client()
                    .connect_ports_by_name("rtjam_rust:rtjam_out_l", "system:playback_1")?;
                active_client
                    .as_client()
                    .connect_ports_by_name("rtjam_rust:rtjam_out_r", "system:playback_2")?;

                loop {
                    sleep(Duration::new(2, 0));
                }
            }
            Err(e) => {
                error!("jack start error: {}", e);
            }
        }
        sleep(Duration::new(2, 0));
    }

    // active_client.deactivate()?;
    Ok(())
}

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        info!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        info!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        info!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        info!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        info!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        info!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        info!(
            "JACK: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        info!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        info!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        info!("JACK: xrun occurred");
        jack::Control::Continue
    }
}

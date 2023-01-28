use crate::common::box_error::BoxError;
use jack;
use serde_json::json;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;

pub fn run(
    status_data_tx: mpsc::Sender<serde_json::Value>, // channel for us to send status data
    command_rx: mpsc::Receiver<serde_json::Value>,   // channel for us to receive commands
) -> Result<(), BoxError> {
    // let's open up a jack port
    let (client, _status) = jack::Client::new("rtjam_rust", jack::ClientOptions::NO_START_SERVER)?;

    let in_a = client.register_port("rtjam_in_1", jack::AudioIn::default())?;
    let in_b = client.register_port("rtjam_in_2", jack::AudioIn::default())?;
    let mut out_a = client.register_port("rtjam_out_l", jack::AudioOut::default())?;
    let mut out_b = client.register_port("rtjam_out_r", jack::AudioOut::default())?;

    let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let out_a_p = out_a.as_mut_slice(ps);
        let out_b_p = out_b.as_mut_slice(ps);
        let in_a_p = in_a.as_slice(ps);
        let in_b_p = in_b.as_slice(ps);
        out_a_p.clone_from_slice(in_a_p);
        out_b_p.clone_from_slice(in_b_p);
        jack::Control::Continue
    };
    let process = jack::ClosureProcessHandler::new(process_callback);

    // Activate the client, which starts the processing.
    let active_client = client.activate_async(Notifications, process)?;

    // Connect system inputs to us and our puts to playback
    active_client
        .as_client()
        .connect_ports_by_name("system:capture_1", "rtjam_rust:rtjam_in_1")?;
    active_client
        .as_client()
        .connect_ports_by_name("system:capture_2", "rtjam_rust:rtjam_in_2")?;
    active_client
        .as_client()
        .connect_ports_by_name("rtjam_rust:rtjam_out_l", "system:playback_1")?;
    active_client
        .as_client()
        .connect_ports_by_name("rtjam_rust:rtjam_out_r", "system:playback_2")?;

    loop {
        sleep(Duration::new(2, 0));
    }
    active_client.deactivate()?;
    Ok(())
}

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        println!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        println!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        println!(
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
        println!(
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
        println!(
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
        println!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: xrun occurred");
        jack::Control::Continue
    }
}

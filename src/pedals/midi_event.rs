use jack::RawMidi;
use serde::Serialize;

#[derive(Serialize)]
pub struct MidiEvent {
    #[serde(rename = "type")]
    e_type: usize,
    channel: usize,
    note: usize,
    velocity: usize
}

impl MidiEvent {
    pub fn new(e: RawMidi) -> MidiEvent {
        match e.bytes[0] {
            192 => {
                MidiEvent { e_type: 4, channel: 0, note: e.bytes[1] as usize, velocity: 0 }
            }
            176 => {
                MidiEvent { e_type: 3, channel: 0, note: e.bytes[1] as usize, velocity: e.bytes[2] as usize }
            }
            _ => {
                println!("Midi: {:?}", e);
                MidiEvent { e_type: 8, channel: 0, note: 0, velocity: 0 }
            }
        }
    }
}

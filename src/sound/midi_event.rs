use jack::RawMidi;
use serde::Serialize;

#[derive(Serialize)]
pub struct MidiEvent {
    #[serde(rename = "type")]
    e_type: usize,
    channel: u8,
    note: u8,
    velocity: u8
}

impl MidiEvent {
    pub fn new(e: RawMidi) -> MidiEvent {
        let res = Self::handle_midi_message(e.bytes);
        match res {
            Ok(m) => {
                println!("{:?}", m);
                match m {
                    wmidi::MidiMessage::ProgramChange(chan, prog) => {
                        MidiEvent { e_type: 4, channel: chan as u8, note: prog.into(), velocity: 0 }
                    }
                    wmidi::MidiMessage::ControlChange(_chan, note , val) => {
                        MidiEvent { e_type: 4, channel: 0, note: note.into(), velocity: val.into() }
                    }
                    _ => {
                        MidiEvent { e_type: 8, channel: 0, note: 0, velocity: 0 }
                    }
                }
            }
            Err(e) => {
                dbg!(e);
                MidiEvent { e_type: 8, channel: 0, note: 0, velocity: 0 }
            }
        }
        // match e.bytes[0] {
        //     192 => {
        //         MidiEvent { e_type: 4, channel: 0, note: e.bytes[1] as usize, velocity: 0 }
        //     }
        //     176 => {
        //         MidiEvent { e_type: 3, channel: 0, note: e.bytes[1] as usize, velocity: e.bytes[2] as usize }
        //     }
        //     _ => {
        //         println!("Midi: {:?}", e);
        //         MidiEvent { e_type: 8, channel: 0, note: 0, velocity: 0 }
        //     }
        // }
    }

    fn handle_midi_message(bytes: &[u8]) -> Result<wmidi::MidiMessage, wmidi::FromBytesError> {
        Ok(wmidi::MidiMessage::try_from(bytes)?)
    }
    
}

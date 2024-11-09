//! container for the various pedals.
//!
//! The PedalBoard will sequentially call through its collection of pedals applying them to the
//! audio data frames.  It will build the pedals (factory) and can arrange them in order.
//!
//! It can also save itself out as a bunch of json that can also be used to construct a pedal
//! board.  This json has all the pedals and their individual controls.  Saved pedal boards in
//! the rtjam-nation website can be reloaded using that interface.
//!
//! When creating a new pedal
//! * the pedal name (unique) and Description must be added to get_pedal_types
//! * the pedal must be added to the private make_pedal function.
//! * the pedal implementation *MUST* use that unique name when it serializes (so the factory can create it)
//! * The pedal struct has to be added to exports in pedals.rs
//! Each pedal must have a unique pedal type that will be used to construct a default pedal of that type.
//!
use std::str::FromStr;

use crate::dsp::biquad::{BiQuadFilter, FilterType};

use super::{
    bass_di::BassDI, bass_envelope::BassEnvelope, chorus::Chorus, compressor::Compressor,
    delay::Delay, guitar_envelope::GuitarEnvelope, noise_gate::NoiseGate, pedal::Pedal,
    sigma_reverb::SigmaReverb, soul_drive::SoulDrive, speaker_sim_iir::SpeakerSimIIR,
    tone_stack::ToneStack, tremelo::Tremelo, tube_drive::TubeDrive, template_pedal::TemplatePedal,
    champ::Champ, princeton::Princeton, tube_screamer::TubeScreamer
};
use serde_json::{json, Value};

type BoxedPedal = std::boxed::Box<
    dyn Pedal
        + std::marker::Send // needed for threads
        + std::marker::Sync, // needed for threads
>;

pub struct PedalBoard {
    pedals: Vec<BoxedPedal>,
    board_id: i64,
    channel: usize, // Channel this board will be used on
    dc_removal: BiQuadFilter,
}

impl PedalBoard {
    pub fn new(chan: usize) -> PedalBoard {
        let mut filter = BiQuadFilter::new();
        filter.init(FilterType::HighPass, 6.0, 1.0, 0.707, 48000.0 );
        PedalBoard {
            pedals: vec![],
            board_id: -1,
            channel: chan,
            dc_removal: filter,
        }
    }
    pub fn get_channel(&self) -> usize {
        self.channel
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) -> () {

        let mut buf1: Vec<f32> = vec![0.0; input.len()];
        let mut buf2: Vec<f32> = vec![0.0; input.len()];

        // Remove any dc offset in the input
        let mut j : usize = 0;
        for s in input {
            buf1[j] = self.dc_removal.get_sample(s);
            j += 1;
        }

        let mut i: usize = 0;
        for pedal in &mut self.pedals {
            if i % 2 == 0 {
                pedal.process(&buf1, &mut buf2);
            } else {
                pedal.process(&buf2, &mut buf1);
            }
            i += 1;
        }
        for n in 0..=input.len() - 1 {
            if i % 2 == 0 {
                output[n] = buf1[n];
            } else {
                output[n] = buf2[n];
            }
        }
    }
    pub fn add_tone_stack(&mut self) -> () {
        self.pedals.push(Box::new(ToneStack::new()));
    }

    pub fn get_pedal_types() -> serde_json::Value {
        json!({
           "Template Pedal": "Sample pedal template",
           "Tone Stack": "Tone controls (3 band)",
           "Noise Gate": "Noise Gate",
           "Bass DI": "Bass Guitar Tone Shaping",
           "Speaker Sim": "Speaker Cabinet Simulator",
           "Sigma Reverb": "Sigma Reverb",
           "Compressor": "Compressor Pedal",
           "Tremelo": "Tremelo ala Fender",
           "Delay": "Delay Pedal",
           "TubeDrive": "Tube Overdrive",
           "Tube Screamer": "Tube Overdrive Modeled From The Classic",
           "SoulDrive": "Soul Overdrive",
           "Chorus": "Chorus",
           "Bass Envelope": "Bass Envelope Filter Pedal",
           "Guitar Envelope": "Guitar Envelope Filter Pedal (auto-wah)",
           "Champ": "Fender Champ",
           "Princeton": "Fender Princeton",
        })
    }

    pub fn num_pedals(&self) -> usize {
        self.pedals.len()
    }

    fn make_pedal(type_name: &str) -> Option<BoxedPedal> {
        match type_name {
            "Template Pedal" => Some(Box::new(TemplatePedal::new())),
            "Tone Stack" => Some(Box::new(ToneStack::new())),
            "Noise Gate" => Some(Box::new(NoiseGate::new())),
            "Bass DI" => Some(Box::new(BassDI::new())),
            "Speaker Sim" => Some(Box::new(SpeakerSimIIR::new())),
            "Sigma Reverb" => Some(Box::new(SigmaReverb::new())),
            "Compressor" => Some(Box::new(Compressor::new())),
            "Tremelo" => Some(Box::new(Tremelo::new())),
            "Delay" => Some(Box::new(Delay::new())),
            "SoulDrive" => Some(Box::new(SoulDrive::new())),
            "TubeDrive" => Some(Box::new(TubeDrive::new())),
            "Chorus" => Some(Box::new(Chorus::new())),
            "Bass Envelope" => Some(Box::new(BassEnvelope::new())),
            "Guitar Envelope" => Some(Box::new(GuitarEnvelope::new())),
            "Champ" => Some(Box::new(Champ::new())),
            "Princeton" => Some(Box::new(Princeton::new())),
            "Tube Screamer" => Some(Box::new(TubeScreamer::new())),
            _ => {
                // No pedal for that name
                println!("Can't create pedal {}", type_name);
                None
            }
        }
    }
    pub fn load_from_json(&mut self, raw: &str) -> () {
        // First thing clear out existing pedals
        self.pedals.clear();
        // parse the json
        match serde_json::Value::from_str(raw) {
            Ok(v) => {
                if let Some(i) = v["id"].as_i64() {
                    self.board_id = i;
                }
                if let Some(configs) = v["config"].as_array() {
                    // We have an array of pedals configs
                    for config in configs {
                        if let Some(ptype) = config["name"].as_str() {
                            // The config entry has a pedal name.  Try to construct one
                            if let Some(mut pedal) = Self::make_pedal(ptype) {
                                // We have a pedal.  Now let's set the settings
                                if let Some(settings) = config["settings"].as_array() {
                                    // We have an array of settings to apply
                                    for setting in settings {
                                        pedal.change_setting(setting);
                                    }
                                }
                                // Put the pedal at the end of the chain
                                // TODO:  This really should sort the configs by config["index"] before runnint the list
                                self.pedals.push(pedal);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                // error parsing json to modify a setting
                dbg!(e);
            }
        }
    }
    pub fn insert_pedal(&mut self, type_name: &str, idx: usize) -> () {
        if let Some(p) = Self::make_pedal(type_name) {
            if idx > self.pedals.len() {
                self.pedals.push(p)
            } else {
                self.pedals.insert(idx, p)
            }
        }
    }
    pub fn delete_pedal(&mut self, idx: usize) -> () {
        if idx < self.pedals.len() {
            self.pedals.remove(idx);
        }
    }
    pub fn move_pedal(&mut self, from_idx: usize, to_idx: usize) -> () {
        if from_idx != to_idx && from_idx < self.pedals.len() && to_idx < self.pedals.len() {
            self.pedals.swap(from_idx, to_idx);
        }
    }
    pub fn change_value(&mut self, pedal_index: usize, setting: &Value) -> () {
        // change the value of a setting on a pedal
        if pedal_index < self.pedals.len() {
            self.pedals[pedal_index].change_setting(setting);
        }
    }
    pub fn as_json(&self, idx: usize) -> serde_json::Value {
        let mut rval: Vec<serde_json::Value> = vec![];
        let mut i = 0;
        for p in &self.pedals {
            rval.push(p.as_json(i));
            i += 1;
        }
        json!({
            "boardId": self.board_id,
            "channel": idx,
            "name": format!("channel_{}", idx),
            "effects": rval,
        })
    }
}

#[cfg(test)]
mod test_pedal_board {
    use super::*;

    #[test]
    fn get_types() {
        let types = PedalBoard::get_pedal_types();
        assert_eq!(types["Tone Stack"], "Tone controls (3 band)");
    }

    #[test]
    fn can_add_one() {
        let mut board = PedalBoard::new(0);
        assert_eq!(board.num_pedals(), 0);
        board.insert_pedal("Tone Stack", 0);
        assert_eq!(board.num_pedals(), 1);
        board.insert_pedal("BogusPedalThatCannotBeMade", 0);
        assert_eq!(board.num_pedals(), 1);
    }
    #[test]
    fn can_delete_one() {
        let mut board = PedalBoard::new(1);
        board.insert_pedal("Tone Stack", 0);
        assert_eq!(board.num_pedals(), 1);
        board.delete_pedal(0);
        assert_eq!(board.num_pedals(), 0);
    }
    #[test]
    fn can_build_muliple() {
        let mut board = PedalBoard::new(0);
        board.insert_pedal("Tone Stack", 0);
        board.insert_pedal("Noise Gate", 0);
        assert_eq!(board.num_pedals(), 2);
        println!(
            "board: {}",
            serde_json::to_string_pretty(&board.as_json(1)).unwrap()
        );
    }
}

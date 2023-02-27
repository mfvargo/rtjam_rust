use std::str::FromStr;

use super::{noise_gate::NoiseGate, pedal::Pedal, tone_stack::ToneStack};
use serde_json::json;

type BoxedPedal = std::boxed::Box<
    dyn Pedal
        + std::marker::Send // needed for threads
        + std::marker::Sync, // needed for threads
>;

pub struct PedalBoard {
    pedals: Vec<BoxedPedal>,
}

impl PedalBoard {
    pub fn new() -> PedalBoard {
        PedalBoard { pedals: vec![] }
    }
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) -> () {
        let mut buf1: Vec<f32> = input.to_vec();
        let mut buf2: Vec<f32> = vec![0.0; input.len()];

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
          "Tone Stack": "Tone controls (3 band)",
          "Noise Gate": "Noise Gate",
        })
    }

    pub fn num_pedals(&self) -> usize {
        self.pedals.len()
    }

    fn make_pedal(type_name: &str) -> Option<BoxedPedal> {
        match type_name {
            "Tone Stack" => Some(Box::new(ToneStack::new())),
            "Noise Gate" => Some(Box::new(NoiseGate::new())),
            _ => {
                // No pedal for that name
                println!("Can't create pedal {}", type_name);
                None
            }
        }
    }
    pub fn insert_pedal(&mut self, type_name: &str, idx: usize) -> () {
        match Self::make_pedal(type_name) {
            Some(p) => {
                if idx > self.pedals.len() {
                    self.pedals.push(p)
                } else {
                    self.pedals.insert(idx, p)
                }
            }
            None => (),
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
    pub fn change_value(&mut self, pedal_index: usize, setting: &str) -> () {
        // change the value of a setting on a pedal

        // Check range on pedal_index
        if pedal_index >= self.pedals.len() {
            return;
        }

        match serde_json::Value::from_str(setting) {
            Ok(v) => {
                self.pedals[pedal_index].change_setting(v);
            }
            Err(e) => {
                // error parsing json to modify a setting
                dbg!(e);
            }
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
            "boardId": -1,
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
        let mut board = PedalBoard::new();
        assert_eq!(board.num_pedals(), 0);
        board.insert_pedal("Tone Stack", 0);
        assert_eq!(board.num_pedals(), 1);
        board.insert_pedal("BogusPedalThatCannotBeMade", 0);
        assert_eq!(board.num_pedals(), 1);
    }
    #[test]
    fn can_delete_one() {
        let mut board = PedalBoard::new();
        board.insert_pedal("Tone Stack", 0);
        assert_eq!(board.num_pedals(), 1);
        board.delete_pedal(0);
        assert_eq!(board.num_pedals(), 0);
    }
    #[test]
    fn can_build_muliple() {
        let mut board = PedalBoard::new();
        board.insert_pedal("Tone Stack", 0);
        board.insert_pedal("Noise Gate", 0);
        assert_eq!(board.num_pedals(), 2);
        println!(
            "board: {}",
            serde_json::to_string_pretty(&board.as_json(1)).unwrap()
        );
    }
}

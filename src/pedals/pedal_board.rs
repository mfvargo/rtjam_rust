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

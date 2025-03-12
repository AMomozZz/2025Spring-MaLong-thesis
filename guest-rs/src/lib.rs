wit_bindgen::generate!({
    world: "component",
});

use exports::pkg::component::nexmark::Guest as NexmarkGuest;
use serde_json::Result;
use nexmark::event::Event;

struct Component;

export!(Component);

const USD_TO_EUR_RATE: f64 = 0.85;

fn parse_event_from_json(json_str: &Vec<u8>) -> Result<Event> {
    // eprintln!("{:?}", String::from_utf8_lossy(json_str));
    serde_json::from_slice::<Event>(json_str)
}

fn serialize_event_to_json(event: &Event) -> Vec<u8> {
    match serde_json::to_vec(event) {
        Ok(json_output) => json_output,
        Err(e) => format!("!Failed to serialize event: {}", e).into_bytes(),
    }
}

impl NexmarkGuest for Component {
    #[doc = "convert-currency"]
    fn q1(json_str: Vec<u8>,) -> Vec<u8> {
        match parse_event_from_json(&json_str) {
            Ok(event) => {
                match event {
                    Event::Bid(mut bid) => {
                        bid.price = ((bid.price as f64) * USD_TO_EUR_RATE) as usize;
                        serialize_event_to_json(&Event::Bid(bid))
                    }
                    _ => [].to_vec(), // json_str,
                }
            },
            Err(e) => {
                format!("!Failed to parse bid: {}", e).into_bytes()
            }
        }
    }
}





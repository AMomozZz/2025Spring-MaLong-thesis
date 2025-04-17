pub mod q1;
pub mod data;

// use crate::data::Auction;
use crate::data::Bid;
// use crate::data::Person;

use std::{fs::File, io::BufReader};
use runtime::prelude::{serde::de::DeserializeOwned, *};
use runtime::traits::Timestamp;
use ::csv::ReaderBuilder;
use wasi::cli::environment;

const USAGE: &str = "Usage: cargo run <data-dir> <query-id>";
const WATERMARK_FREQUENCY: usize = 1000;
const SLACK: Duration = Duration::from_milliseconds(100);

fn main() {
    let binding = environment::get_arguments();
    println!("{:?}", binding);
    let mut args = binding.iter().skip(1);
    let Some(query) = args.next() else {
        println!("{USAGE}");
        return;
    };

    let bids = std::fs::File::open(&format!("../nexmark-data/bid/bids.csv")).map(iter::<Bid>);
    // let auctions = std::fs::File::open(&format!("data/auctions.csv")).map(iter::<Auction>);
    // let persons = std::fs::File::open(&format!("data/persons.csv")).map(iter::<Person>);

    match query.as_str() {
        "q1" => timed(move |ctx| q1::run(stream(ctx, bids), ctx)),
        "q1-opt" => timed(move |ctx| q1::run_opt(stream(ctx, bids), ctx)),
        "io" => timed(move |ctx| stream(ctx, bids).drain(ctx)),
        _ => panic!("unknown query"),
    }
}

fn iter<T: Data + DeserializeOwned + 'static>(file: File) -> impl Iterator<Item = T> {
    let reader = BufReader::new(file);
    let csv_reader = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(reader);

    csv_reader
        .into_deserialize::<T>() 
        .map(move |result| match result {
            Ok(data) => {
                data
            },
            Err(e) => {
                panic!("CSV deserialization failed: {:?}", e);
            }
        })
}

fn timed(f: impl FnOnce(&mut Context) + Send + 'static) {
    let time = std::time::Instant::now();
    CurrentThreadRunner::run(f);
    eprintln!("{}", time.elapsed().as_millis());
}

// Stream from iterator
fn stream_with<T: Data + Timestamp>(
    ctx: &mut Context,
    iter: std::io::Result<impl Iterator<Item = T> + Send + 'static>,
    frequency: usize,
) -> Stream<T> {
    Stream::from_iter(ctx, iter.unwrap(), T::timestamp, frequency, SLACK)
}

fn stream<T: Data + Timestamp>(
    ctx: &mut Context,
    iter: std::io::Result<impl Iterator<Item = T> + Send + 'static>,
) -> Stream<T> {
    stream_with(ctx, iter, WATERMARK_FREQUENCY)
}
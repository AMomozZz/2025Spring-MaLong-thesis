pub mod q1;
pub mod q2;
pub mod q3;
pub mod q4;
pub mod q5;
pub mod q6;
pub mod q7;
// pub mod q8;
// pub mod qw;

pub mod data;

use crate::data::Auction;
use crate::data::Bid;
use crate::data::Person;

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
    let Some(dir) = args.next() else {
        println!("{USAGE}");
        return;
    };
    let Some(query) = args.next() else {
        println!("{USAGE}");
        return;
    };

    let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
    let auctions = std::fs::File::open(&format!("{dir}/auctions.csv")).map(iter::<Auction>);
    let persons = std::fs::File::open(&format!("{dir}/persons.csv")).map(iter::<Person>);

    match query.as_str() {
        "q1" => timed(move |ctx| q1::run(stream(ctx, bids), ctx)),
        "q2" => timed(move |ctx| q2::run(stream(ctx, bids), ctx)),
        "q3" => timed(move |ctx| q3::run(stream(ctx, auctions), stream(ctx, persons), ctx)),
        "q4" => timed(move |ctx| q4::run(stream(ctx, auctions), stream(ctx, bids), ctx)),
        "q5" => timed(move |ctx| q5::run(stream(ctx, bids), ctx)),
        "q6" => timed(move |ctx| q6::run(stream(ctx, auctions), stream(ctx, bids), ctx)),
        "q7" => timed(move |ctx| q7::run(stream(ctx, bids), ctx)),
        // "q8" => timed(move |ctx| q8::run(stream(ctx, auctions), stream(ctx, persons), ctx)),
        // "qw" => {
        //     let Some(size) = args.next() else {
        //         println!("{USAGE} <size> <step>");
        //         return;
        //     };
        //     let Some(step) = args.next() else {
        //         println!("{USAGE} <size> <step>");
        //         return;
        //     };
        //     let size = size.parse().unwrap();
        //     let step = step.parse().unwrap();
        //     timed(move |ctx| qw::run(stream(ctx, bids), size, step, ctx))
        // },
        // Optimised
        "q1-opt" => timed(move |ctx| q1::run_opt(stream(ctx, bids), ctx)),
        "q2-opt" => timed(move |ctx| q2::run_opt(stream(ctx, bids), ctx)),
        "q3-opt" => timed(move |ctx| q3::run_opt(stream(ctx, auctions), stream(ctx, persons), ctx)),
        "q4-opt" => timed(move |ctx| q4::run_opt(stream(ctx, auctions), stream(ctx, bids), ctx)),
        "q5-opt" => timed(move |ctx| q5::run_opt(stream(ctx, bids), ctx)),
        "q6-opt" => timed(move |ctx| q6::run_opt(stream(ctx, auctions), stream(ctx, bids), ctx)),
        "q7-opt" => timed(move |ctx| q7::run_opt(stream(ctx, bids), ctx)),
        // "q8-opt" => timed(move |ctx| q8::run_opt(stream(ctx, auctions), stream(ctx, persons), ctx)),
        // "qw-opt" => {
        //     let size = args.next().unwrap().parse().unwrap();
        //     let step = args.next().unwrap().parse().unwrap();
        //     timed(move |ctx| qw::run_opt(stream(ctx, bids), size, step, ctx))
        // },
        
        "io" => {
            timed(move |ctx| {
                if bids.is_ok() {
                    stream(ctx, bids).drain(ctx);
                }
                if persons.is_ok() {
                    stream(ctx, persons).drain(ctx);
                }
                if auctions.is_ok() {
                    stream(ctx, auctions).drain(ctx);
                }
            });
        },
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
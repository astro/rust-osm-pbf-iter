use std::env::args;
use std::fs::File;
use std::io::{BufReader, Seek};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread;
use std::time::Instant;

use osm_pbf_iter::*;

type Stats = [u64; 3];

fn blobs_worker(req_rx: Receiver<Blob>, res_tx: SyncSender<Stats>) {
    let mut stats = [0; 3];

    while let Ok(blob) = req_rx.recv() {
        let data = blob.into_data();
        let primitive_block = PrimitiveBlock::parse(&data);
        for primitive in primitive_block.primitives() {
            match primitive {
                Primitive::Node(_) => stats[0] += 1,
                Primitive::Way(_) => stats[1] += 1,
                Primitive::Relation(_) => stats[2] += 1,
            }
        }
    }

    res_tx.send(stats).unwrap();
}

fn main() {
    let cpus: usize = thread::available_parallelism().unwrap().into();

    for arg in args().skip(1) {
        let mut workers = Vec::with_capacity(cpus);
        for _ in 0..cpus {
            let (req_tx, req_rx) = sync_channel(2);
            let (res_tx, res_rx) = sync_channel(0);
            workers.push((req_tx, res_rx));
            thread::spawn(move || {
                blobs_worker(req_rx, res_tx);
            });
        }

        println!("Open {}", arg);
        let f = File::open(&arg).unwrap();
        let mut reader = BlobReader::new(BufReader::new(f));
        let start = Instant::now();

        let mut w = 0;
        for blob in &mut reader {
            let req_tx = &workers[w].0;
            w = (w + 1) % cpus;

            req_tx.send(blob).unwrap();
        }

        let mut stats = [0; 3];
        for (req_tx, res_rx) in workers.into_iter() {
            drop(req_tx);
            let worker_stats = res_rx.recv().unwrap();
            for i in 0..worker_stats.len() {
                stats[i] += worker_stats[i];
            }
        }

        let stop = Instant::now();
        let duration = stop.duration_since(start);
        let duration = duration.as_secs() as f64 + (duration.subsec_nanos() as f64 / 1e9);
        let mut f = reader.into_inner();
        if let Ok(pos) = f.stream_position() {
            let rate = pos as f64 / 1024f64 / 1024f64 / duration;
            println!(
                "Processed {} MB in {:.2} seconds ({:.2} MB/s)",
                pos / 1024 / 1024,
                duration,
                rate
            );
        };

        println!(
            "{} - {} nodes, {} ways, {} relations",
            arg, stats[0], stats[1], stats[2]
        );
    }
}

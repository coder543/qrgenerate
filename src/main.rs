extern crate indicatif;

use indicatif::{ProgressBar, ProgressStyle};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

static TASKS: AtomicUsize = AtomicUsize::new(0);

fn inc() {
    let mut tasks = TASKS.load(Ordering::Relaxed);
    loop {
        if tasks > 500 {
            tasks = TASKS.load(Ordering::Relaxed);
            thread::sleep(Duration::from_millis(25));
            continue;
        }
        match TASKS.compare_exchange_weak(tasks, tasks + 1, Ordering::SeqCst, Ordering::Relaxed) {
            Ok(_) => break,
            Err(x) => tasks = x,
        }
    }
}

fn dec() {
    TASKS.fetch_sub(1, Ordering::SeqCst);
}

fn generate_code(word: String) {
    inc();
    thread::spawn(move || {
        let filename = format!("./qrcodes/{}.png", word);
        Command::new("qrencode")
            .arg("-o")
            .arg(&filename)
            .arg(word)
            .status()
            .unwrap_or_else(|e| {
                dec();
                panic!("qrencode failed! {:#?}", e)
            });

        Command::new("mogrify")
            .arg("-resize")
            .arg("512x512")
            .arg(filename)
            .status()
            .unwrap_or_else(|e| {
                dec();
                panic!("mogrify failed! {:#?}", e)
            });

        dec();
    });
}

fn generate(word: String, bar: &ProgressBar) {
    if word.len() == 3 {
        return;
    }
    for c in ('a' as u8)..('z' as u8) {
        let mut new_word = word.clone();
        new_word.push(c as char);
        generate_code(new_word.clone());
        bar.inc(1);
        generate(new_word, bar);
    }
}

fn main() {
    let bar = ProgressBar::new(17576);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} {eta_precise}"),
    );
    generate(String::new(), &bar);
}

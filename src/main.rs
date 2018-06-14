use std::io::{stdout, Write};
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

fn main() {
    let mut c = 'a';
    let mut word = String::new();
    for i in 0..50000 {
        print!("\r{}  {}", i, TASKS.load(Ordering::Relaxed));
        stdout().flush().unwrap();
        let mut new_word = word.clone();
        new_word.push(c);
        generate_code(new_word);
        if c == 'z' {
            c = 'A';
        } else if c == 'Z' {
            c = 'a';
            word.push(c);
        } else {
            c = (c as u8 + 1) as char;
        }
    }
}

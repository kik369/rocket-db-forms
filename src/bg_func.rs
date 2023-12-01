use std::{thread, time::Duration};

pub fn print_hello() {
    loop {
        println!("Background function running... 🚀");
        thread::sleep(Duration::from_secs(5));
    }
}


mod executor;

use std::env;
fn main() {
    let args:Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Arguments missing");
        return;
    }
    executor::execute(args);
}
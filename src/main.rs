mod file_scanner;
mod model;
use std::error::Error;
use tokio::sync::mpsc;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let (tx, mut rx) = mpsc::channel(100);
    //file_rx: mpsc::Receiver<(String, Event)>
    let pth = String::from("/home/sus/work/testground");
    let mut fsc = file_scanner::FileScanner::new(&pth, rx);
    loop {
        fsc.read_and_compare().await?;
    }
    println!("Hello, world!");
    return Ok(());
}

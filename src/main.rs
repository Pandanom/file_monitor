mod file_scanner;
mod model;
use std::error::Error;
use tokio::sync::mpsc;
use tokio::task;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let (tx, mut rx) = mpsc::channel::<model::Event>(100);
    //file_rx: mpsc::Receiver<(String, Event)>
    let pth = String::from("/home/sus/work/testground");
    let mut fsc = file_scanner::FileScanner::new(&pth, tx);
    //OBJ = file_scanner::FileScannerController::new(&pth, tx);
    //loop {
    //    fsc.read_and_compare().await?;
    //}
    //fsc.start();

    let reader_task = task::spawn(async move{
        loop {
            fsc.read_and_compare().await.unwrap();
        }
    });
    
    while let Some(ev) = rx.recv().await {
        println!("{ev:?}");
    }

    reader_task.abort();

    return Ok(());
}

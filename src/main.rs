mod file_scanner;
mod model;
use std::error::Error;
use std::sync::atomic;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task;
use tokio::signal;
use tokio::io::AsyncWriteExt;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use clap::Parser;
use std::str::FromStr;

static DEFAULT_PATH: &'static str = "~/inbox/.";

#[derive(Parser)]
#[command(version, about = "Program for monitoring files in INBOX folder", long_about = None)]
struct Cli {
    /// Path to INBOX folder, defaults to "~/inbox/.".
    /// If "~/inbox/." does not exist and no valid path was provided "~/inbox/." will be created.
    #[arg(short = 'd', long)]
    path: Option<String>,
}

fn handle_args() -> String {
    let cli = Cli::parse();

    match cli.path {
        Some(pth) => {
            //let mut abs = abspath(&pth);
            match abspath(&pth) {
                Some(abs) => {
                    match PathBuf::from_str(&abs) {
                        Ok(d) => {
                            if !d.is_dir() {
                                eprintln!("Provided path {pth} is not directory. Will use default path");
                            }
                            return abs;
                        },
                        Err(e) => {
                            eprintln!("Problem with reading {pth}: {e}. Will use default path");
                        },
                    }
                },
                None => {
                    eprintln!("Provided path {pth} is invalid. Will use default path");
                },
            }
        },
        None => {},
    };
    
    let abs_path = abspath(&String::from(DEFAULT_PATH));
    //path does not exist
    if abs_path == None {
        println!("Creating INBOX folder: {DEFAULT_PATH}");
        //panic if we can't create INBOX
        let mut pb = PathBuf::from_str(abspath(&String::from("~/")).unwrap().as_str()).unwrap();
        pb.push("inbox");
        std::fs::create_dir(pb).unwrap();
        return abspath(&String::from(DEFAULT_PATH)).unwrap();
    }
    else {
        return abs_path.unwrap();
    }

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pth = handle_args();
    let (tx, rx) = mpsc::channel::<model::Event>(100);
   
    let mut fsc = file_scanner::FileScanner::new(&pth, tx);
    
    // do initial read
    fsc.read_path_recursive().await.unwrap();
    let init = fsc.get_curr_read_copy();
    write_files(&init, &pth).await;

    // used to stop reader_task when needed
    let reader_run = Arc::new(AtomicBool::new(true));
    let reader_task = task::spawn(reader(fsc, Arc::clone(&reader_run)));
    // writes events from rx to stdout
    let writer = task::spawn(event_write(rx, pth.clone()));
    // await ctrl + c signal from user
    signal::ctrl_c().await?;
    // stop internal loop of reader_task
    reader_run.store(false, Ordering::Release);
    // await for FileScanner to get final state of buffer
    fsc = reader_task.await.unwrap();
    // stop writer task
    writer.abort();

    //current buffer could be half filled if we stop program in the middle of reading process
    //therefore we take previous read, which is definitely finished 
    let last = fsc.get_prev_read_copy();
    write_files(&last, &pth).await;
    return Ok(());
}


// get absolute path
fn abspath(p: &String) -> Option<String> {
    let exp_path = shellexpand::full(p).ok()?;
    let can_path = std::fs::canonicalize(exp_path.as_ref()).ok()?;
    can_path.into_os_string().into_string().ok()
}

async fn write_files(files: &BTreeSet<model::File>, base: &String) {
    let mut stdout = tokio::io::stdout();
    // delimiter
    stdout.write_all(b"--------------------------------------\n").await.unwrap();
    for f in files {
        let relative = f.get_relative_path(base);
        // “[Date Time] PATH“
        let msg =  format!("[{}] /{}\n", f.last_mod_date.format("%d.%m.%Y %H:%M:%S"), relative);
        stdout.write_all(msg.as_bytes()).await.unwrap();
    }
    stdout.flush().await.unwrap();
}

async fn event_write(mut rx: mpsc::Receiver<model::Event>, base: String) {
    let mut stdout = tokio::io::stdout();
    while let Some(ev) = rx.recv().await {
        // “[EVENT] PATH”
        let msg =  format!("[{}] /{}\n", ev.ev_type, ev.file.get_relative_path(&base));
        stdout.write_all(msg.as_bytes()).await.unwrap();
        stdout.flush().await.unwrap();
    }
}

//returns moved FileScanner, so we can use it later 
async fn reader (mut fsc : file_scanner::FileScanner, run : Arc<atomic::AtomicBool>) -> file_scanner::FileScanner {
    while run.load(Ordering::Acquire) {
        fsc.read_and_compare().await.unwrap();
    }
    return fsc;
}
extern crate chrono;
extern crate tokio;

use crate::model::{Event, File};

use tokio::sync::mpsc;
use std::{collections::LinkedList, collections::BTreeSet, str::FromStr};
use std::path::PathBuf;
use tokio::task::JoinSet;
use std::error::Error;


#[derive(Debug)]

pub struct FileScanner {
    file_tx: mpsc::Sender<(String, Event)>,
    folder_path: String,
    curr_buff: usize, // should be 0 or 1.
    //stores 2 Sets of files. One is for new read and other for old to compare with and find deleted/created files.
    buff: Vec<BTreeSet<File>>,
}

//Reads content of directory given by folder_path argument
impl FileScanner {

    pub fn new(path: &String, tx: mpsc::Sender<(String, Event)>) -> FileScanner {
        
        FileScanner{
            file_tx: tx,
            folder_path: path.clone(),
            curr_buff: 0,
            buff: vec![BTreeSet::new(), BTreeSet::new()],
        }
    }

/*     async fn files_reciver(&mut self) -> Result<(), Box<dyn Error>> {
        while let Some((path, file)) = self.file_rx.recv().await {
            let old_val = self.files.insert(file);

        }

        Ok(())
    } */

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        let mut rac = task::spawn(async {
            loop {
                
            }
        });
    }

    pub async fn read_and_compare(&mut self) -> Result<(), Box<dyn Error>> {
        //switch to next buffer to not rewrite previous read
        let prev_buff = self.curr_buff;
        self.curr_buff = (self.curr_buff + 1) % 2;
        self.read_path_recursive().await.unwrap();

        // difference returns elements that are in self but not in other
        // therefore
        // new files
        let new = self.buff[self.curr_buff].difference(&self.buff[prev_buff]);
        for n in new {
            println!("New file {n:?}");
        }
        // deleted files
        let del = self.buff[prev_buff].difference(&self.buff[self.curr_buff]);
        for d in del {
            println!("Deleted file {d:?}");
        }
        // those intersection has files with same path
        // but last_mod_date can be different as we don't use it in File comparator
        let prev_it = self.buff[prev_buff].intersection(&self.buff[self.curr_buff]);
        let curr_it = self.buff[self.curr_buff].intersection(&self.buff[prev_buff]);

        let p_iter = prev_it.zip(curr_it);
        // to find modified files we simply need to compare last_mod_date on those 2 intersections
        // not perfect solution, but its only ~O(n) + difference and intersection costs - good enough
        // would be easier if those functions had ability to use custom comparator function.
        for (prev, curr) in p_iter {
            if (prev.path == curr.path && prev.last_mod_date != curr.last_mod_date)
            {
                println!("Mod date was changed! {prev:?} | {curr:?}");
            }
        }    
        return Ok(());
    }

// reads folder_path recursively. Writes results into buff[self.curr_buff]
    pub async fn read_path_recursive(&mut self) -> Result<(), Box<dyn Error>> {
        let mut dir_read_tasks = JoinSet::new();
        self.buff[self.curr_buff].clear();
        //spawn initial task to read "inbox"
        dir_read_tasks.spawn(Self::read_path(PathBuf::from_str(&self.folder_path)?));

        while let Some(res) = dir_read_tasks.join_next().await {
            let (mut children_dirs, mut dir_files) = res?.unwrap();
            //spawn tasks to read each child dir 
            for _ in 0..children_dirs.len() {
                dir_read_tasks.spawn(Self::read_path(children_dirs.pop_front().unwrap()));
            }
            self.buff[self.curr_buff].append(&mut dir_files);
        }

        //for f in &self.buff[self.curr_buff % 2] {
        //    println!("{:?} | {}", f.path, f.last_mod_date);
        //}
        return Ok(());
    }

// reads given folder_path. Returns found directories in List and files BTreeSet
    async fn read_path(folder_path: PathBuf ) -> std::io::Result<(LinkedList<PathBuf>, BTreeSet<File>)> {
        let mut paths = tokio::fs::read_dir(&folder_path).await?;
        let mut dirs: LinkedList<PathBuf> = LinkedList::new();
        let mut files: BTreeSet<File> = BTreeSet::new();
        loop {
            let dir = match paths.next_entry().await {
                Ok(d) => {
                    match d {
                       Some(data) =>  data,
                       None => break, // Directory has no more items
                    }
                },
                Err(e) => {
                    eprintln!("Problem while reading directory {folder_path:?}: {e}");
                    continue
                },
            };
            let ft = match dir.file_type().await {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Problem while reading directory entry {}: {e}", dir.path().display());
                    continue
                },
            };
            //if it's symlink - we ignore it.  
            if ft.is_dir() {
                //save to list and return for next reading cycle
                dirs.push_back(dir.path());
            }
            else if ft.is_file() {
                if let Ok(md) = dir.metadata().await {
                    // md.modified() will return error if this field is not available on platform 
                    // in that case we want to just panic
                    files.insert(File { path: dir.path(), last_mod_date: md.modified().unwrap().into()});
                }
                else {
                    // its possible file was created and deleted very fast (like when using VIM and it creates temp files)
                    // in that case we would get error "No such file or directory"
                    // another possible error is "The user lacks permissions to perform metadata call on path."
                    // We will ignore both those cases and continue
                    continue;
                }
            }
        };
        return Ok((dirs, files));
    }
}


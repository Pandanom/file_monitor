use std::cmp::Ordering;
use std::path::PathBuf;
use chrono::DateTime;
use std::fmt;
use std::collections::BTreeSet;
use std::str::FromStr;
#[derive(Debug, Eq, Clone)]
pub struct File {
    pub path: PathBuf,
    pub last_mod_date: DateTime<chrono::offset::Local>,
}

impl Ord for File {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.cmp(&other.path)
    }
}

impl PartialOrd for File {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.last_mod_date == other.last_mod_date
    }
}

impl File {

    pub fn get_relative_path(&self, base: &String) -> String {
        let relative = self.path.strip_prefix(PathBuf::from_str(&base).unwrap()).unwrap();
        let ret = relative.display().to_string();
        return ret;
    }
}


#[derive(Debug)]
pub enum EventType {
    NEW, 
    MOD,
    DEL,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct Event {
    pub ev_type: EventType,
    pub file: File,
}

// File buffer for changes analizing 
// Stores 2 BTreeSets of T
#[derive(Debug)]
pub struct DBuffer<T> {
    curr_buff: usize, // should be 0 or 1.
    //stores 2 Sets of files. One is for new read and other for old to compare with and find deleted/created files.
    buff: Vec<BTreeSet<T>>,
}

impl<T> DBuffer<T> {

    pub fn new() -> DBuffer<T> {
        DBuffer {
            curr_buff: 0,
            buff: vec![BTreeSet::new(), BTreeSet::new()],
        }
    }

    #[inline]
    fn prev_buff(&self) -> usize { (self.curr_buff + 1)%2 }

    pub fn get_curr<'a>(&'a mut self) -> &'a mut BTreeSet<T> {
        return &mut self.buff[self.curr_buff];
    }

    //returns (prev, curr) non mutable buffers
    pub fn get_buffers<'a>(&'a self) -> (&'a BTreeSet<T>, &'a BTreeSet<T>) {
        let prev = self.prev_buff();
        return (&self.buff[prev], &self.buff[self.curr_buff]);
    }
    // moves to next iteration
    // "swaps" buffers and clear old data 
    pub fn next(&mut self) { 
        self.curr_buff = (self.curr_buff + 1) % 2; 
        self.buff[self.curr_buff].clear();
    }
}

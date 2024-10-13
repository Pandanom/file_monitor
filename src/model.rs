use std::cmp::Ordering;
use std::path::PathBuf;
use chrono::DateTime;
use std::fmt;

#[derive(Debug, Eq, Clone)]
pub struct File {
    pub path: PathBuf,
    pub last_mod_date: DateTime<chrono::offset::Local>,
}

impl Ord for File {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.cmp(&other.path)
        /* match self.path.cmp(&other.path) {
            Ordering::Equal => self.last_mod_date.cmp(&other.last_mod_date),
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less
        } */
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


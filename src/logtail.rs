use std::mem;
use std::io::{self, BufReader, SeekFrom};
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

/// Iterates over a file line by line, blocking until new lines are appended.
pub struct LogTail<'a> {
    path: &'a Path,
    file: Option<BufReader<File>>,
    pos: u64
}

impl<'a, P: ?Sized> From<&'a P> for LogTail<'a> where P: AsRef<Path> {
    fn from(path: &'a P) -> LogTail<'a> {
        LogTail {
            path: path.as_ref(),
            file: None,
            pos: 0
        }
    }
}

impl<'a> Iterator for LogTail<'a> {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<io::Result<String>> {
        // main calculation is in a function that returns Result, for cleaner code using try!
        let inner = &mut || -> io::Result<Option<String>> {
            if self.file.is_none() {
                let mut f = BufReader::new(try!(File::open(self.path)));
                self.pos = try!(f.seek(SeekFrom::End(0)));
                // seek back to last newline
                while self.pos > 0 {
                    self.pos = try!(f.seek(SeekFrom::Current(-1)));
                    let mut buf = [0];
                    try!(f.read_exact(&mut buf));
                    if buf[0] == b'\n' {
                        break;
                    } else {
                        self.pos = try!(f.seek(SeekFrom::Current(-1)));
                    }
                }
                self.file = Some(f);
            }
            let mut buf = String::default();
            let mut f = mem::replace(&mut self.file, None).unwrap();
            loop {
                //TODO watch for new file in logs archive
                self.pos += try!(f.read_line(&mut buf)) as u64;
                if buf.chars().last().map_or(false, |c| c == '\n') {
                    buf.pop();
                    break;
                }
            }
            self.file = Some(f);
            Ok(Some(buf))
        };

        match inner() {
            Ok(Some(s)) => Some(Ok(s)),
            Ok(None) => None,
            Err(e) => Some(Err(e))
        }
    }
}

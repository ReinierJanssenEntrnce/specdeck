//! The session's terminal history: every output batch and resize, in order, with gap-free
//! sequence numbers (design decision 4).
//!
//! Late join, reconnect and reattaching the host window all read the log from a sequence number,
//! so every terminal processes exactly the bytes the host's did, at the sizes the host had.

use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};

use tempfile::NamedTempFile;
use tokio::sync::watch;

/// How long output is collected into one record before it is appended.
pub const BATCH_INTERVAL: Duration = Duration::from_millis(16);

/// Output is appended early when a batch grows this large.
pub const MAX_BATCH_BYTES: usize = 64 * 1024;

pub use specdeck_protocol::frames::Record;

const KIND_OUTPUT: u8 = 1;
const KIND_RESIZE: u8 = 2;

/// Where a record's payload is in the file.
#[derive(Clone, Copy)]
struct Entry {
    kind: u8,
    offset: u64,
    len: u32,
}

struct Writer {
    file: File,
    offset: u64,
}

/// Append-only, file-backed log of a session's terminal records. The file has user-only
/// permissions and is deleted when the log is dropped.
pub struct OutputLog {
    writer: Mutex<Writer>,
    reader: File,
    index: RwLock<Vec<Entry>>,
    latest: watch::Sender<Option<u64>>,
    /// Owns the file and deletes it on drop. Declared last so it is dropped after the handles
    /// above, which Windows needs to delete the file right away.
    file: NamedTempFile,
}

impl OutputLog {
    /// Creates a new, empty log file in `dir`.
    pub fn create(dir: &Path) -> io::Result<Self> {
        std::fs::create_dir_all(dir)?;
        // Created with user-only permissions (0600 on Unix, the user's ACL on Windows).
        let file = tempfile::Builder::new().prefix("session-").suffix(".log").tempfile_in(dir)?;
        let writer = file.as_file().try_clone()?;
        let reader = file.reopen()?;
        Ok(Self {
            writer: Mutex::new(Writer { file: writer, offset: 0 }),
            reader,
            index: RwLock::new(Vec::new()),
            latest: watch::channel(None).0,
            file,
        })
    }

    pub fn path(&self) -> &Path {
        self.file.path()
    }

    /// Appends terminal output and returns its sequence number.
    pub fn append_output(&self, bytes: &[u8]) -> io::Result<u64> {
        self.append(KIND_OUTPUT, bytes)
    }

    /// Appends a terminal size change and returns its sequence number.
    pub fn append_resize(&self, cols: u16, rows: u16) -> io::Result<u64> {
        let mut payload = [0; 4];
        payload[..2].copy_from_slice(&cols.to_be_bytes());
        payload[2..].copy_from_slice(&rows.to_be_bytes());
        self.append(KIND_RESIZE, &payload)
    }

    fn append(&self, kind: u8, payload: &[u8]) -> io::Result<u64> {
        let len = u32::try_from(payload.len()).map_err(|_| io::ErrorKind::InvalidInput)?;
        let mut writer = self.writer.lock().unwrap();
        writer.file.write_all(payload)?;
        let entry = Entry { kind, offset: writer.offset, len };
        writer.offset += u64::from(len);
        // Index under the writer lock, so sequence numbers follow file order.
        let mut index = self.index.write().unwrap();
        let seq = index.len() as u64;
        index.push(entry);
        drop(index);
        drop(writer);
        self.latest.send_replace(Some(seq));
        Ok(seq)
    }

    /// Sequence number of the newest record, or `None` while the log is empty.
    pub fn latest(&self) -> Option<u64> {
        *self.latest.borrow()
    }

    /// Notifies whenever a record is appended.
    pub fn subscribe(&self) -> watch::Receiver<Option<u64>> {
        self.latest.subscribe()
    }

    /// Reads records starting at `from`, stopping once about `max_bytes` of payload were read.
    /// Returns at least one record if `from` exists, and none if it does not exist yet.
    pub fn read(&self, from: u64, max_bytes: usize) -> io::Result<Vec<Record>> {
        let entries: Vec<(u64, Entry)> = {
            let index = self.index.read().unwrap();
            let start = usize::try_from(from).unwrap_or(usize::MAX).min(index.len());
            let mut total = 0;
            index[start..]
                .iter()
                .enumerate()
                .take_while(|(i, entry)| {
                    let keep = *i == 0 || total + entry.len as usize <= max_bytes;
                    total += entry.len as usize;
                    keep
                })
                .map(|(i, entry)| ((start + i) as u64, *entry))
                .collect()
        };
        entries.into_iter().map(|(seq, entry)| self.load(seq, entry)).collect()
    }

    fn load(&self, seq: u64, entry: Entry) -> io::Result<Record> {
        let mut payload = vec![0; entry.len as usize];
        read_exact_at(&self.reader, &mut payload, entry.offset)?;
        Ok(match entry.kind {
            KIND_RESIZE => Record::Resize {
                seq,
                cols: u16::from_be_bytes([payload[0], payload[1]]),
                rows: u16::from_be_bytes([payload[2], payload[3]]),
            },
            _ => Record::Output { seq, bytes: payload },
        })
    }
}

/// Collects pty output into batches of at most [`BATCH_INTERVAL`] or [`MAX_BATCH_BYTES`].
#[derive(Default)]
pub struct Batcher {
    pending: Vec<u8>,
    started: Option<Instant>,
}

impl Batcher {
    /// Adds output; returns a full batch when the size limit is reached.
    pub fn push(&mut self, bytes: &[u8], now: Instant) -> Option<Vec<u8>> {
        if self.pending.is_empty() {
            self.started = Some(now);
        }
        self.pending.extend_from_slice(bytes);
        (self.pending.len() >= MAX_BATCH_BYTES).then(|| self.take())
    }

    /// Returns the pending batch once it is [`BATCH_INTERVAL`] old.
    pub fn poll(&mut self, now: Instant) -> Option<Vec<u8>> {
        let due = self.started.is_some_and(|started| now >= started + BATCH_INTERVAL);
        due.then(|| self.take())
    }

    /// When the pending batch is due, if there is one.
    pub fn deadline(&self) -> Option<Instant> {
        self.started.map(|started| started + BATCH_INTERVAL)
    }

    /// Returns whatever is pending, for example before a resize record or when the pty closes.
    pub fn take(&mut self) -> Vec<u8> {
        self.started = None;
        std::mem::take(&mut self.pending)
    }
}

#[cfg(unix)]
fn read_exact_at(file: &File, buf: &mut [u8], offset: u64) -> io::Result<()> {
    std::os::unix::fs::FileExt::read_exact_at(file, buf, offset)
}

#[cfg(windows)]
fn read_exact_at(file: &File, mut buf: &mut [u8], mut offset: u64) -> io::Result<()> {
    use std::os::windows::fs::FileExt;
    while !buf.is_empty() {
        match file.seek_read(buf, offset)? {
            0 => return Err(io::ErrorKind::UnexpectedEof.into()),
            n => {
                buf = &mut buf[n..];
                offset += n as u64;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output(seq: u64, bytes: &[u8]) -> Record {
        Record::Output { seq, bytes: bytes.to_vec() }
    }

    #[test]
    fn records_have_gap_free_sequence_numbers_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let log = OutputLog::create(dir.path()).unwrap();
        assert_eq!(log.latest(), None);
        assert_eq!(log.append_output(b"a").unwrap(), 0);
        assert_eq!(log.append_resize(120, 40).unwrap(), 1);
        assert_eq!(log.append_output(b"bc").unwrap(), 2);
        assert_eq!(log.latest(), Some(2));

        assert_eq!(
            log.read(0, usize::MAX).unwrap(),
            vec![output(0, b"a"), Record::Resize { seq: 1, cols: 120, rows: 40 }, output(2, b"bc")]
        );
    }

    #[test]
    fn reads_from_any_sequence_number() {
        let dir = tempfile::tempdir().unwrap();
        let log = OutputLog::create(dir.path()).unwrap();
        for i in 0..5u8 {
            log.append_output(&[b'0' + i]).unwrap();
        }
        assert_eq!(log.read(3, usize::MAX).unwrap(), vec![output(3, b"3"), output(4, b"4")]);
        assert!(log.read(5, usize::MAX).unwrap().is_empty());
        assert!(log.read(u64::MAX, usize::MAX).unwrap().is_empty());
    }

    #[test]
    fn read_stops_near_max_bytes_but_returns_at_least_one_record() {
        let dir = tempfile::tempdir().unwrap();
        let log = OutputLog::create(dir.path()).unwrap();
        log.append_output(&[1; 10]).unwrap();
        log.append_output(&[2; 10]).unwrap();
        log.append_output(&[3; 10]).unwrap();
        assert_eq!(log.read(0, 20).unwrap().len(), 2);
        assert_eq!(log.read(0, 1).unwrap().len(), 1);
    }

    #[test]
    fn subscribers_see_new_records() {
        let dir = tempfile::tempdir().unwrap();
        let log = OutputLog::create(dir.path()).unwrap();
        let mut latest = log.subscribe();
        log.append_output(b"x").unwrap();
        assert!(latest.has_changed().unwrap());
        assert_eq!(*latest.borrow_and_update(), Some(0));
    }

    #[test]
    fn file_is_deleted_when_the_log_is_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let log = OutputLog::create(dir.path()).unwrap();
        log.append_output(b"secret").unwrap();
        let path = log.path().to_owned();
        assert!(path.exists());
        drop(log);
        assert!(!path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn file_is_readable_by_the_user_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let log = OutputLog::create(dir.path()).unwrap();
        let mode = std::fs::metadata(log.path()).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn two_logs_in_one_directory_do_not_clash() {
        let dir = tempfile::tempdir().unwrap();
        let a = OutputLog::create(dir.path()).unwrap();
        let b = OutputLog::create(dir.path()).unwrap();
        assert_ne!(a.path(), b.path());
    }

    #[test]
    fn batcher_flushes_after_the_interval() {
        let start = Instant::now();
        let mut batcher = Batcher::default();
        assert_eq!(batcher.push(b"ab", start), None);
        assert_eq!(batcher.push(b"c", start + Duration::from_millis(5)), None);
        assert_eq!(batcher.poll(start + Duration::from_millis(10)), None);
        assert_eq!(batcher.deadline(), Some(start + BATCH_INTERVAL));
        assert_eq!(batcher.poll(start + BATCH_INTERVAL), Some(b"abc".to_vec()));
        assert_eq!(batcher.deadline(), None);
        assert_eq!(batcher.poll(start + Duration::from_secs(1)), None);
    }

    #[test]
    fn batcher_flushes_early_when_full() {
        let mut batcher = Batcher::default();
        let full = batcher.push(&vec![0; MAX_BATCH_BYTES], Instant::now()).unwrap();
        assert_eq!(full.len(), MAX_BATCH_BYTES);
        assert_eq!(batcher.deadline(), None);
    }
}

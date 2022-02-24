use anes::*;
use humantime;
use std::io::Write;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread::spawn;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;

const SLEEP_DURATION: Duration = Duration::from_millis(300);

pub struct Status {
    stop: Arc<(Mutex<bool>, Condvar)>,
    at: Arc<AtomicU64>,
    handle: Option<JoinHandle<()>>,
}

impl Drop for Status {
    fn drop(&mut self) {
        {
            let (lock, cvar) = &*self.stop;
            let mut stopped = lock.lock().unwrap();
            *stopped = true;
            // We notify the condvar that the value has changed.
            cvar.notify_one();
        }
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}

fn clamp_duration(dur: Duration) -> Duration {
    Duration::from_secs(dur.as_secs())
}

impl Status {
    pub fn new(total: u64) -> Status {
        let at = Arc::new(AtomicU64::new(0));
        let at2 = at.clone();
        // let stop = Condvar::new();
        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        let pair2 = Arc::clone(&pair);
        let handle = spawn(move || {
            let now = Instant::now();
            loop {
                let (lock, cvar) = &*pair;
                let stopped = lock.lock().unwrap();
                let result = cvar.wait_timeout(stopped, SLEEP_DURATION).unwrap();
                let progress = at.load(Ordering::Relaxed) as f64 / total as f64;
                if at.load(Ordering::Relaxed) == 0 {
                    continue;
                }
                let elapsed = clamp_duration(now.elapsed());
                let total = clamp_duration(now.elapsed().div_f64(progress));
                let left = total - elapsed;
                print!(
                    "\r{}Progress: {:.1}%, elapsed: {}, eta: {}, total: {}",
                    ClearLine::All,
                    progress * 100.0,
                    humantime::format_duration(elapsed),
                    humantime::format_duration(left),
                    humantime::format_duration(total),
                );
                std::io::stdout().flush().unwrap();
                if *result.0 {
                    println!("");
                    break;
                }
            }
        });
        Status {
            stop: pair2,
            at: at2,
            handle: Some(handle),
        }
    }

    pub fn _set(&self, at: u64) {
        self.at.store(at, Ordering::Relaxed);
    }
    pub fn add(&self, inc: u64) {
        self.at.fetch_add(inc, Ordering::Relaxed);
    }
}

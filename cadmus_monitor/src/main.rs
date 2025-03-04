mod handler;
mod change;

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::{select, task::JoinHandle};
use std::path::PathBuf;
use std::{path::Path, time::UNIX_EPOCH};
use common::{dbobjects::file::File, hbam2};
use std::time::{Duration, SystemTime};
use handler::{Handler, DummyHandler};

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;

use change::Change;

mod monitor_proto {
    tonic::include_proto!("monitor");
}

async fn watch<P: AsRef<Path>>(
    path: P,
    file_map: Arc<Mutex<HashMap<PathBuf, Option<SystemTime>>>>
    ) -> notify::Result<()> {
    let (watch_tx, watch_rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(watch_tx, Config::default()).unwrap();
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in watch_rx {
        let e = match res {
            Ok(inner) => inner,
            Err(e) => panic!("{}", e),
        };

        match e.kind {
            EventKind::Modify(..) => {
                let mut file_times_handle = file_map.lock().unwrap();
                file_times_handle.insert(e.paths[0].clone(), Some(SystemTime::now()));
            }
            _ => {}
        }
    }
    Ok(())
}

async fn handle(path: PathBuf) {
    println!("HELLO FROM HANDLER for: {:?}", path);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "test_data";
    println!("Watching path: {:?}", path);

    let file_map = Arc::new(Mutex::new(<HashMap::<PathBuf, Option<SystemTime>>>::new()));

    tokio::spawn(watch(path, file_map.clone()));
    const DEBOUNCE_INTERVAL: Duration = Duration::new(2, 0);
    let mut handler_jobs = vec![];
    loop {
        std::thread::sleep(DEBOUNCE_INTERVAL);
        println!("[#1] Checking MAP...");
        let file_times_handle = &mut file_map.lock();
        let file_times_handle = file_times_handle.as_mut().unwrap();
        let now = SystemTime::now();
        let mut to_be_removed = vec![];
        for (file, time) in file_times_handle.iter_mut() {
            if time.is_some() && now.duration_since(*time.as_ref().unwrap()).unwrap() > DEBOUNCE_INTERVAL {
                println!("Starting job for {:?}", file);
                *time = None;
                to_be_removed.push(file);
                handler_jobs.push((file.to_path_buf(), tokio::spawn(handle(file.to_path_buf()))))
            }
        }
    }
}

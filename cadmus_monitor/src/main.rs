
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;

fn handle_modification(path: &Path) {
    // Maintain a write ahead log for case where we can't connect to the server right now.
}

fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                match event.kind {
                    notify::EventKind::Modify(_) => {
                        // If we see a modify, start counting down a timer
                        // If there hasn't been another modification for that
                        // path in the next 2 seconds, then we're good to do some 
                        eprintln!("modified: {:?}", event.paths);
                        handle_modification(&event.paths[0]);
                    }
                    _ => {}
                }
                println!("Change: {event:?}")
            }
            Err(error) => eprintln!("Error: {error:?}"),
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "../test_data/input/";
    println!("Watching path: {:?}", path);

    if let Err(error) = watch(path) {
        eprintln!("error: {error:?}");
    }
    Ok(())
}

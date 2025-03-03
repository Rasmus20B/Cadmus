
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;

fn handle_modification() {
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
                        eprintln!("modified: {:?}", event.paths);
                    }
                    _ => {}
                }
                //println!("Change: {event:?}")
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

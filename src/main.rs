extern crate crossbeam;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::channel;

use crossbeam::sync::ArcCell;

struct Put {
    key: String,
    val: String,
}

fn main() {
    let current = ArcCell::new(Arc::new(HashMap::new()));
    crossbeam::scope(|scope| {
        let (writer, sender) = {
            let (tx, rx) = channel();
            let current = &current;
            let writer = scope.spawn(move || {
                let mut db = HashMap::new();
                while let Ok(Put { key, val }) = rx.recv() {
                    db.insert(key, val);
                    // Full clone is required for snapshot.
                    // To fix it, we'll need a persistent (immutable) map.
                    current.set(Arc::new(db.clone()));
                }
            });
            (writer, tx)
        };

        let readers: Vec<_> = (0..5).map(|r_idx| {
            let sender = sender.clone();
            let current = &current;
            scope.spawn(move || {
                for i in 0..3 {
                    sender.send(Put {
                        key: format!("Reader {}", r_idx),
                        val: i.to_string(),
                    }).unwrap();
                    let snapshot = current.get(); // not guaranteed to receive an update
                    println!("Reader {}'s snapshot: {:?}\n", r_idx, snapshot)
                }
            })
        }).collect();
        drop(sender);
        for reader in readers {
            reader.join();
        }
        writer.join()
    });
}

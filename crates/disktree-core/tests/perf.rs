//! Performance budgets on a home-directory-sized tree.
//!
//! These are `#[ignore]`d so `cargo test` stays quick; run them with
//! `make perf` (a release build: debug timings mean nothing). Each budget is
//! what keeps the window from stalling, measured against a tree of about four
//! million entries — the size of a real developer's home directory.

#![allow(clippy::unwrap_used, reason = "tests")]

use std::time::{Duration, Instant};

use disktree_core::classify::classify;
use disktree_core::filter::filter;
use disktree_core::insights::worth_a_look;
use disktree_core::tree::{Metric, Node, NodeKind, aggregate};
use disktree_core::treemap::{LayoutOptions, Rect, layout};

/// A deterministic tree shaped like a home directory: a few wide top-level
/// directories, uneven fan-out, and files at every level.
fn synthetic(target: usize) -> Node {
    let mut root = Node::directory("home");
    let mut made = 0usize;
    let mut seed = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    fill(&mut root, 0, target, &mut made, &mut next);
    aggregate(&mut root, Metric::Bytes);
    classify(&mut root);
    root
}

fn fill(
    node: &mut Node,
    depth: u32,
    target: usize,
    made: &mut usize,
    next: &mut impl FnMut() -> u64,
) {
    const NAMES: [&str; 8] = [
        "src",
        "node_modules",
        "cache",
        "Library",
        ".git",
        "build",
        "Documents",
        "target",
    ];
    let files = 4 + (next() % 24) as usize;
    for index in 0..files {
        if *made >= target {
            return;
        }
        let bytes = 512 + next() % (1 << (10 + next() % 16));
        let mut file = Node::entry(
            format!("file-{depth}-{index}.dat"),
            NodeKind::File,
            bytes,
        );
        file.modified =
            1_700_000_000 + i64::try_from(next() % 50_000_000).unwrap();
        node.children.push(file);
        *made += 1;
    }
    if depth >= 8 {
        return;
    }
    let dirs = if depth == 0 {
        40
    } else {
        1 + (next() % 7) as usize
    };
    for index in 0..dirs {
        if *made >= target {
            return;
        }
        let name = NAMES[(next() % NAMES.len() as u64) as usize];
        let mut dir = if depth < 2 {
            Node::directory(format!("{name}-{index}"))
        } else {
            Node::directory(name)
        };
        *made += 1;
        fill(&mut dir, depth + 1, target, made, next);
        node.children.push(dir);
    }
}

fn count(node: &Node) -> usize {
    1 + node.children.iter().map(count).sum::<usize>()
}

fn timed<T>(label: &str, budget: Duration, work: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let out = work();
    let took = start.elapsed();
    eprintln!("{label:<34} {:>8.1} ms  (budget {budget:?})", ms(took));
    assert!(took <= budget, "{label} took {took:?}, budget {budget:?}");
    out
}

fn ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

const ENTRIES: usize = 4_000_000;

#[test]
#[ignore = "performance: run with `make perf`"]
fn a_four_million_entry_tree_stays_interactive() {
    let tree = timed("build synthetic tree", Duration::from_secs(30), || {
        synthetic(ENTRIES)
    });
    let entries = count(&tree);
    eprintln!("entries: {entries}");
    assert!(entries >= ENTRIES * 9 / 10);

    // One frame: laying out the visible levels. This runs on the UI thread
    // whenever the view changes, so it has to fit comfortably in a frame.
    let area = Rect::new(0.0, 0.0, 1600.0, 1000.0);
    for depth in [3, 6] {
        let options = LayoutOptions {
            max_depth: depth,
            ..LayoutOptions::default()
        };
        let tiles = timed(
            &format!("layout, depth {depth}"),
            Duration::from_millis(16),
            || layout(&tree, &[], area, Metric::Bytes, &options),
        );
        assert!(!tiles.is_empty());
    }

    // Worth a look is recomputed on the UI thread after a scan and a
    // metric change.
    let found = timed("worth a look", Duration::from_millis(250), || {
        worth_a_look(&tree, 1_760_000_000, 8)
    });
    assert!(!found.is_empty());

    // Metric switch: a full copy and re-sort, done off the UI thread; the
    // budget is how long the old ordering stays on screen.
    let mut copy = timed("clone tree", Duration::from_secs(3), || tree.clone());
    timed("re-aggregate by files", Duration::from_secs(3), || {
        aggregate(&mut copy, Metric::Files);
    });
    timed("drop tree", Duration::from_secs(3), || drop(copy));

    // Typing in the filter runs off the UI thread, but each keystroke
    // restarts it, so it must finish quickly to feel live.
    let matches = timed(
        "filter \"node_modules\"",
        Duration::from_millis(900),
        || filter(&tree, &[], "node_modules"),
    );
    assert!(matches.is_some());
}

/// Re-ordering a real scan by file count, the step behind the Files mode.
/// Scans `DISKTREE_PERF_ROOT`, or the home directory.
#[test]
#[ignore = "performance: scans a real directory; run with `make perf`"]
fn reordering_a_real_scan_is_quick() {
    use disktree_core::scan::{ScanOptions, scan};
    let root = std::env::var_os("DISKTREE_PERF_ROOT")
        .or_else(|| std::env::var_os("HOME"))
        .map(std::path::PathBuf::from)
        .unwrap();
    let tree = timed("scan", Duration::from_mins(10), || {
        scan(&root, ScanOptions::default()).unwrap()
    });
    eprintln!("entries: {}", count(&tree));
    for (label, metric) in [
        ("by files", Metric::Files),
        ("by bytes", Metric::Bytes),
        ("by files again", Metric::Files),
    ] {
        let mut copy = tree.clone();
        timed(label, Duration::from_millis(1500), || {
            aggregate(&mut copy, metric);
        });
    }
}

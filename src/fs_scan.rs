use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

use walkdir::WalkDir;

use crate::engine::Engine;
use crate::report::Report;
use crate::Arguments;

pub(crate) fn start(args: Arguments, engine: Engine, mut report: Report) -> Result<(), String> {
    log::info!("scanning {} ...", &args.root);

    let start = Instant::now();
    let num_scanned = AtomicU32::new(0);
    let num_detected = AtomicU32::new(0);

    for entry in WalkDir::new(&args.root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let f_path = entry.path();
        if !f_path.is_file() {
            continue;
        }

        let mut do_scan = args.ext.is_empty();
        if !do_scan {
            if let Some(ext) = f_path.extension() {
                for filter_ext in &args.ext {
                    if filter_ext.to_lowercase() == *ext.to_string_lossy().to_lowercase() {
                        do_scan = true;
                        break;
                    }
                }
            }
        }

        if do_scan {
            let res = engine.scan(&f_path.to_path_buf());
            if res.detected {
                num_detected.fetch_add(1, Ordering::SeqCst);
            }
            num_scanned.fetch_add(1, Ordering::SeqCst);

            if let Err(e) = report.report(res) {
                log::error!("reporting error: {:?}", e);
            }
        }
    }

    log::info!(
        "{:?} files scanned in {:?}, {:?} positive detections",
        num_scanned,
        start.elapsed(),
        num_detected
    );

    Ok(())
}

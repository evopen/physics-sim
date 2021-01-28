#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use anyhow::{bail, Context, Result};
use log::{debug, error, info, trace, warn};

fn init_logger() -> Result<()> {
    let log_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(false)
        .truncate(true)
        .open(format!("{}.log", env!("CARGO_PKG_NAME")))?;

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Error)
        .level_for(env!("CARGO_CRATE_NAME"), log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .chain(log_file)
        .apply()?;
    Ok(())
}

fn main() -> Result<()> {
    init_logger()?;

    println!("Hello, world!");

    Ok(())
}

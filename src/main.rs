#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use log::info;
use std::{panic, process};

mod egui_app;
mod models;
mod renderer;
mod winit_app;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    set_panic_hook();
    info!("Starting...");
    //egui_app::run();
    winit_app::run()?;
    Ok(())
}

fn set_panic_hook() {
    // Set up custom panic hook before initializing the app
    //let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Log the panic info
        eprintln!("Application panicked: {}", panic_info);

        // You can also show a dialog or write to a log file here

        // Call the original hook
        //original_hook(panic_info);
        process::exit(1);
    }));
}

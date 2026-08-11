#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod commands;
mod gui;
mod headless;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();
    match cli.command {
        None => gui::run(),
        Some(cli::Command::Render(args)) => headless::run_render(args),
    }
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::App;
use bird_snake::args::*;
use clap::Parser;

fn main() {
    let args = Args::parse();
    let mut app = App::new();

    bird_snake::run(&mut app, &args);
}

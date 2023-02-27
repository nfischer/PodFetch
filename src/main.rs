#![feature(plugin, decl_macro, proc_macro_hygiene)]
#![allow(proc_macro_derive_resolution_fallback, unused_attributes)]


#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use dotenv::dotenv;
use std::{env, thread};
use std::env::var;
use actix_files as fs;
use actix_web::{App, http, HttpServer, web};
use std::process::Command;
use std::time::Duration;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use clokwerk::{Scheduler, TimeUnits};
use feed_rs::parser;
use reqwest::blocking::{Client, ClientBuilder};
use rusqlite::Connection;

mod controllers;
pub use controllers::user_controller::*;
use crate::constants::constants::{DB_NAME, PODCASTS_ROOT_DIRECTORY};
use crate::models::itunes_models::Podcast;
use crate::service::rust_service::{insert_podcast_episodes, schedule_episode_download};
use crate::service::file_service::create_podcast_root_directory_exists;
mod db;
mod models;
mod constants;
mod service;
use crate::db::DB;
use crate::service::environment_service::EnvironmentService;

mod config;

#[actix_web::main]
async fn main()-> std::io::Result<()> {
    DB::new().unwrap();
    create_podcast_root_directory_exists();

    thread::spawn(||{
        let mut scheduler = Scheduler::new();
        let env = EnvironmentService::new();
        let polling_interval = env.get_polling_interval();
        scheduler.every(polling_interval.minutes()).run(||{
            let db = DB::new().unwrap();
            //check for new episodes
            let podcasts = db.get_podcasts().unwrap();
            println!("Checking for new episodes: {:?}", podcasts);
            for podcast in podcasts {
                let podcast_clone = podcast.clone();
                insert_podcast_episodes(podcast);
                schedule_episode_download(podcast_clone)
            }
        });
        loop {
            scheduler.run_pending();
            thread::sleep(Duration::from_millis(1000));
        }
    });

    // Start WebSocket server
    let websocket_server = thread::spawn(||HttpServer::new(|| {
        App::new()
            .service(web::resource("/ws/"))
    })
        .bind("127.0.0.1:8080")
        .unwrap()
        .run());


    HttpServer::new(|| {
        let public_url = var("PUBLIC_URL").unwrap_or("http://localhost:5173".to_string());
        let cors = Cors::default()
            .allowed_origin(&public_url)
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);


        let api = web::scope("/api/v1")
            .service(find_podcast)
            .service(add_podcast)
            .service(find_all_podcasts)
            .service(find_all_podcast_episodes_of_podcast)
            .service(find_podcast_by_id)
            .service(log_watchtime)
            .service(get_last_watched)
            .service(get_watchtime)

            .wrap(Logger::default());
        App::new().service(fs::Files::new
            ("/podcasts", "podcasts").show_files_listing())
            .wrap(cors)
            .service(api)
            .wrap(Logger::default())
    }
    )
        .bind(("0.0.0.0", 8000))?
        .run()
        .await
}
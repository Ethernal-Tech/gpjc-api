// #[macro_use]
extern crate actix_web;

use actix_cors::Cors;
use actix_web::{middleware, App, HttpServer};
use dotenv::dotenv;
use std::env;

mod handler;
mod types;

fn set_env() {
    let binding = std::env::current_dir().unwrap();
    let current_dir_path = binding.to_str().unwrap();
    let pdir = current_dir_path.to_owned() + "/private-join-and-compute";
    std::env::set_current_dir(pdir).unwrap();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Command line argument missing: Address for running the gpjc server is missing, 
                  consider running the program with Makefile command run.
                  Check the README.md file in case you want to use different address for gpjc server.");
        return Ok(());
    }

    dotenv().ok();
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    set_env();

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            // register HTTP requests handlers
            .service(handler::start_client_process)
            .service(handler::start_server_process)
    })
    .bind((args[1].as_str(), 9090))?
    .run()
    .await
}

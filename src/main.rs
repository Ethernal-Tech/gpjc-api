// #[macro_use]
extern crate actix_web;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use dotenv::dotenv;
use std::env;

mod constants;
mod db;
mod handler;
mod types;

fn set_env() {
    let binding = std::env::current_dir().unwrap();
    let current_dir_path = binding.to_str().unwrap();
    let pdir = current_dir_path.to_owned().clone() + "/private-join-and-compute";
    std::env::set_current_dir(pdir).unwrap();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    #[allow(unused)]
    let mut mssql_password = "".to_string();

    #[cfg(target_os = "linux")]
    {
        if args.len() != 3 {
            println!("Command line argument missing: Expected <address> and <mssql-password>");
            return Ok(());
        }
        mssql_password = args[2].clone();
    }

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

    match db::execute_query(mssql_password.clone(), db::Query::CreateTable, Vec::new()) {
        Ok(_) => println!("Table created"),
        Err(err) => println!("Error {err}"),
    }

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(mssql_password.clone()))
            .wrap(Cors::permissive())
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            // register HTTP requests handlers
            .service(handler::start_client_process)
            .service(handler::start_server_process)
            .service(handler::get_proof)
    })
    .bind((args[1].as_str(), 9090))?
    .run()
    .await
}

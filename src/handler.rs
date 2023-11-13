use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::process::Command;
use std::str::FromStr;

use actix_web::web::{self, Json};
use actix_web::{get, post, HttpResponse, Responder};
use csv::Writer;

use crate::constants::APPLICATION_JSON;
use crate::db;
use crate::types::{ClientStartRequest, ProofRequest, Response, ServerStartRequest};

fn get_path(file_name: &str) -> String {
    let binding = std::env::current_dir().unwrap();
    let current_dir_path = binding.to_str().unwrap();
    let sanctions_dir_path =
        current_dir_path.replace("private-join-and-compute", "") + "sanction-lists/";

    let paths = fs::read_dir(sanctions_dir_path)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    paths
        .iter()
        .find(|&x| x.contains(file_name))
        .unwrap()
        .to_string()
}

fn create_csv(receiver: String) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(get_path("UN_test.csv"))?;
    wtr.write_record(&[receiver, 0.to_string()])?;
    wtr.flush()?;
    Ok(())
}

#[allow(unused)]
pub fn start_client(
    mssql_password: web::Data<String>,
    transaction_id: i32,
    destination_address: String,
) -> Response {
    let client_csv_path = get_path("UN_test.csv");

    let output = Command::new("bazel-bin/private_join_and_compute/client")
        .arg(format!("--client_data_file={client_csv_path}"))
        .arg(format!("--port={destination_address}"))
        .output()
        .unwrap();

    // Feature not used for testing on local machine
    #[cfg(feature = "multiple-machines")]
    {
        let params = vec![transaction_id.to_string(), "1".to_string()];
        match db::execute_query(mssql_password.to_string(), db::Query::InsertLog, params) {
            Ok(_) => println!("Entered log in db"),
            Err(err) => println!("Insert into gpjc_logs failed with error: {err}"),
        };
    }
    match output.status.code() {
        Some(code) => {
            if code == 0 {
                let output_text = String::from_utf8_lossy(&output.stdout).into_owned();
                #[cfg(feature = "multiple-machines")]
                {
                    let sliced_text: Vec<&str> = output_text.split(',').collect();

                    let params = vec![
                        sliced_text[0].to_string(),
                        sliced_text[1].to_string(),
                        transaction_id.to_string(),
                    ];
                    match db::execute_query(
                        mssql_password.to_string(),
                        db::Query::UpdateLog,
                        params,
                    ) {
                        Ok(_) => println!("Updated log in db"),
                        Err(err) => println!("Insert into gpjc_logs failed with error: {err}"),
                    };
                }

                return Response {
                    exit_code: code,
                    data: output_text,
                };
            } else {
                println!("{}", String::from_utf8_lossy(&output.stdout));
                println!("{}", String::from_utf8_lossy(&output.stderr));
                return Response {
                    exit_code: code,
                    data: String::from_utf8_lossy(&output.stdout).into_owned(),
                };
            }
        }
        None => {
            return Response {
                exit_code: 1,
                data: "Error in client execution".to_string(),
            }
        }
    }
}

pub fn start_server(mssql_password: web::Data<String>, transaction_id: String) -> Response {
    let params = vec![transaction_id.clone(), "0".to_string()];
    match db::execute_query(mssql_password.to_string(), db::Query::InsertLog, params) {
        Ok(_) => println!("Entered log in db"),
        Err(err) => println!("Insert into gpjc_logs failed with error: {err}"),
    };

    let server_csv_path = get_path("UN_List.csv");
    let dns_name = std::env::var("DNS_NAME").expect("DNS_NAME in .env file must be set.");

    let output = Command::new("bazel-bin/private_join_and_compute/server")
        .arg(format!("--server_data_file={server_csv_path}"))
        .arg(format!("--port={dns_name}:10501"))
        .output()
        .unwrap();

    match output.status.code() {
        Some(code) => {
            if code == 0 {
                let output_text = String::from_utf8_lossy(&output.stdout).into_owned();
                let sliced_text: Vec<&str> = output_text.split(',').collect();

                let params = vec![
                    sliced_text[0].to_string(),
                    sliced_text[1].to_string(),
                    transaction_id,
                ];
                match db::execute_query(mssql_password.to_string(), db::Query::UpdateLog, params) {
                    Ok(_) => println!("Updated log in db"),
                    Err(err) => println!("Insert into gpjc_logs failed with error: {err}"),
                };

                return Response {
                    exit_code: code,
                    data: output_text,
                };
            } else {
                return Response {
                    exit_code: code,
                    data: String::from_utf8_lossy(&output.stdout).into_owned(),
                };
            }
        }
        None => {
            return Response {
                exit_code: 1,
                data: format!("Error in gpjc server execution"),
            }
        }
    }
}

#[post("/api/start-client")]
pub async fn start_client_process(
    mssql_password: web::Data<String>,
    request_data: Json<ClientStartRequest>,
) -> impl Responder {
    tokio::spawn(async move {
        match create_csv(request_data.receiver.clone()) {
            Ok(()) => {
                let _resp = start_client(
                    mssql_password,
                    FromStr::from_str(request_data.tx_id.as_str()).unwrap(),
                    request_data.to.clone(),
                );
            }
            Err(err) => println!("Creation of csv file failed {}", err),
        }
    });

    return HttpResponse::Ok();
}

#[post("/api/start-server")]
pub async fn start_server_process(
    mssql_password: web::Data<String>,
    request_data: Json<ServerStartRequest>,
) -> impl Responder {
    tokio::spawn(async move {
        let resp = start_server(mssql_password, request_data.tx_id.clone());
        if resp.exit_code != 0 {
            println!("ERROR: GPJC failed with error: {}", resp.data);
            return;
        }

        let sliced_text: Vec<&str> = resp.data.split(',').collect();

        let mut map = HashMap::new();
        map.insert("TransactionId", request_data.tx_id.clone());
        map.insert("PolicyId", request_data.policy_id.clone());
        map.insert("Value", sliced_text[0].to_string());

        let client = reqwest::Client::new();
        let intermediary_address =
            std::env::var("INTERMEDIARY").expect("INTERMEDIARY in .env file must be set.");
        let _res = client
            .post(format!(
                "http://{}/api/submitTransactionProof",
                intermediary_address
            ))
            .json(&map)
            .send()
            .await;
    });

    return HttpResponse::Ok();
}

#[get("/api/proof")]
pub async fn get_proof(
    mssql_password: web::Data<String>,
    request_data: Json<ProofRequest>,
) -> impl Responder {
    let params = vec![FromStr::from_str(request_data.tx_id.as_str()).unwrap()];
    match db::execute_query(mssql_password.to_string(), db::Query::GetLog, params) {
        Ok(val) => match val {
            Some(resp) => return HttpResponse::Ok().content_type(APPLICATION_JSON).json(resp),
            None => {
                return HttpResponse::Ok()
                    .content_type(APPLICATION_JSON)
                    .json("Log with this transaction id does not exist")
            }
        },
        Err(_err) => {
            return HttpResponse::BadRequest()
                .content_type(APPLICATION_JSON)
                .json("")
        }
    };
}

use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::process::Command;
use std::sync::Arc;

use actix_web::web::{self, Json};
use actix_web::{post, HttpResponse, Responder};
use csv::Writer;

use crate::crypto::{self, Keys};
use crate::types::{ClientStartRequest, Response, ServerStartRequest};

fn get_path(file_name: &str) -> String {
    let binding = std::env::current_dir().unwrap();
    let current_dir_path = binding.to_str().unwrap();
    let sanctions_dir_path =
        current_dir_path.replace("private-join-and-compute", "") + "sanction-lists/";

    let paths = fs::read_dir(&sanctions_dir_path)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    match paths.iter().find(|&x| x.contains(file_name)) {
        Some(path) => path.to_string(),
        None => {
            let path = format!("{}/{}", sanctions_dir_path, file_name);
            File::create(&path).unwrap();
            path
        }
    }
}

fn create_csv(participants: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(get_path("UN_test.csv"))?;
    let mut i = 1;
    for participant in participants {
        wtr.write_record(&[participant, i.to_string()])?;
        i += 1;
    }
    wtr.flush()?;
    Ok(())
}

#[allow(unused)]
pub fn start_client(destination_address: String) -> Response {
    let client_csv_path = get_path("UN_test.csv");

    let output = Command::new("bazel-bin/private_join_and_compute/client")
        .arg(format!("--client_data_file={client_csv_path}"))
        .arg(format!("--port={destination_address}"))
        .output()
        .unwrap();

    match output.status.code() {
        Some(code) => {
            if code == 0 {
                let output_text = String::from_utf8_lossy(&output.stdout).into_owned();

                Response {
                    exit_code: code,
                    data: output_text,
                }
            } else {
                println!("{}", String::from_utf8_lossy(&output.stdout));
                println!("{}", String::from_utf8_lossy(&output.stderr));
                return Response {
                    exit_code: code,
                    data: String::from_utf8_lossy(&output.stdout).into_owned(),
                };
            }
        }
        None => Response {
            exit_code: 1,
            data: "Error in client execution".to_string(),
        },
    }
}

pub fn start_server() -> Response {
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

                Response {
                    exit_code: code,
                    data: output_text,
                }
            } else {
                Response {
                    exit_code: code,
                    data: String::from_utf8_lossy(&output.stdout).into_owned(),
                }
            }
        }
        None => Response {
            exit_code: 1,
            data: format!("Error in gpjc server execution {:#?}", output),
        },
    }
}

#[post("/api/start-client")]
pub async fn start_client_process(
    request_data: Json<ClientStartRequest>,
    #[cfg(feature = "multiple-machines")] data: web::Data<Arc<Keys>>,
) -> impl Responder {
    tokio::spawn(async move {
        #[allow(unused)]
        let mut resp: Option<Response> = None;
        match create_csv(request_data.participants.clone()) {
            Ok(()) => {
                resp = Some(start_client(request_data.to.clone()));

                if let Some(resp) = &resp {
                    if resp.exit_code != 0 {
                        println!("ERROR: GPJC failed with error: {}", resp.data);
                    }
                } else {
                    println!("ERROR: GPJC failed")
                }
            }
            Err(err) => {
                println!("Creation of csv file failed {}", err);
            }
        }

        #[cfg(feature = "multiple-machines")]
        {
            if let Some(response) = resp {
                let signer_addr = crypto::public_key_to_address(data.public_key.as_str()).unwrap();
                let signed_msg = vec![
                    request_data.compliance_check_id.as_str(),
                    response.data.as_str(),
                ]
                .join(",");
                let mut sig = "".to_string();

                match crypto::sign_message(&signed_msg, &data.secret_key) {
                    Ok(signature) => {
                        sig = signature;
                    }
                    Err(e) => println!("Error signing message: {}", e),
                }

                notify_caller(
                    request_data.compliance_check_id.clone(),
                    request_data.policy_id.clone(),
                    vec![signed_msg, sig, signer_addr].join(";"),
                )
                .await;
            }
        }
    });

    HttpResponse::Ok()
}

#[post("/api/start-server")]
pub async fn start_server_process(
    request_data: Json<ServerStartRequest>,
    data: web::Data<Arc<Keys>>,
) -> impl Responder {
    tokio::spawn(async move {
        let resp = start_server();
        if resp.exit_code != 0 {
            println!("ERROR: GPJC failed with error: {}", resp.data);
            return;
        }

        let signer_addr = crypto::public_key_to_address(data.public_key.as_str()).unwrap();
        let signed_msg = [
            request_data.compliance_check_id.as_str(),
            resp.data.as_str(),
        ]
        .join(",");
        let mut sig = "".to_string();

        match crypto::sign_message(&signed_msg, &data.secret_key) {
            Ok(signature) => {
                sig = signature;
            }
            Err(e) => println!("Error signing message: {}", e),
        }

        notify_caller(
            request_data.compliance_check_id.clone(),
            request_data.policy_id.clone(),
            [signed_msg, sig, signer_addr].join(";"),
        )
        .await;
    });

    HttpResponse::Ok()
}

async fn notify_caller(compliance_check_id: String, policy_id: String, resulting_value: String) {
    let mut map = HashMap::new();
    map.insert("compliance_check_id", compliance_check_id);
    map.insert("policy_id", policy_id);
    map.insert("value", resulting_value);

    let client = reqwest::Client::new();
    let api_address =
        std::env::var("GPJC_PUBLISH_ADDR").expect("GPJC_PUBLISH_ADDR in .env file must be set.");
    let res = client
        .post(format!("http://{}/proof/interactive", api_address))
        .json(&map)
        .send()
        .await;

    match res {
        Ok(_) => println!("Result and signature sent"),
        Err(e) => println!("Error sending the result: {}", e),
    }
}

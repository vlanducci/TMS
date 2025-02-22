use ::log::warn;
use rocket::*;
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use tms_utils::{TmsRespond, TmsRouteResponse, TmsClients, TmsClient, security::encrypt, schemas::create_permissions, network_schemas::{RegisterResponse, RegisterRequest}, with_clients_write};

fn register_client(user_id: String, key: String, clients: TmsClients) {
  with_clients_write(&clients, |client_map| {
    client_map.insert(
      user_id.clone(),
      TmsClient {
        user_id: user_id.clone(),
        key: key.clone(),
        auth_token: String::from(""),
        permissions: create_permissions(),
        last_timestamp: std::time::SystemTime::now(),
        ws_sender: None
      },
    );
  }).unwrap();
}

pub fn unregister_client(user_id: String, clients: TmsClients) {
  with_clients_write(&clients, |client_map| {
    client_map.remove(&user_id);
  }).unwrap();
  warn!("Unregistered Client {}", user_id);
}

#[post("/register", data = "<register_request>")]
pub fn register_route(clients: &State<TmsClients>, s_public_key: &State<Vec<u8>>, ws_port: &State<u16>, register_request: Json<RegisterRequest>) -> TmsRouteResponse<()> {
  let user_id = register_request.user_id.clone();

  let res = RegisterResponse {
    key: String::from_utf8(s_public_key.to_vec()).unwrap(),
    url_scheme: String::from("ws://"),
    url_path: format!(":{}/ws/{}", ws_port, user_id),
    version: option_env!("VERSION").unwrap_or("0.0.0").to_string()
  };
  
  if clients.read().unwrap().contains_key(&user_id) {
    warn!("Already registered client");

    TmsRespond!(
      Status::AlreadyReported, 
      res,
      register_request.key.clone()
    );
  }

  register_client(user_id.to_owned(), register_request.key.to_owned(), clients.inner().clone());
  warn!("Registered New Client: {}", user_id);
  
  TmsRespond!(
    Status::Ok,
    res,
    register_request.key.clone()
  );
}

#[delete("/register/<user_id>")]
pub fn unregister_route(clients: &State<TmsClients>, user_id: String) -> TmsRouteResponse<()> {
  unregister_client(user_id, clients.inner().clone());
  TmsRespond!();
}
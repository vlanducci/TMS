
use log::error;
use rocket::{State, get, http::Status, post};
use tms_macros::tms_private_route;
use tms_utils::{security::Security, security::encrypt, TmsClients, TmsRouteResponse, schemas::{GameMatch, create_permissions}, TmsRespond, network_schemas::{MatchesResponse, MatchRequest, MatchResponse, MatchLoadRequest, MatchUpdateRequest, SocketMessage, MatchDeleteRequest, MatchAddRequest}, TmsRequest, check_permissions, tms_clients_ws_send};

use crate::{db::{db::TmsDB, tree::{UpdateTree, UpdateError}}, event_service::TmsEventServiceArc};

#[get("/matches/get/<uuid>")]
pub fn matches_get_route(clients: &State<TmsClients>, db: &State<std::sync::Arc<TmsDB>>, uuid: String) -> TmsRouteResponse<()> {
  // get matches from db and put into MatchesResponse
  let mut matches:Vec<GameMatch> = vec![];

  for match_raw in db.tms_data.matches.iter() {
    let game_match = match match_raw {
      Ok(game_match) => game_match.1,
      _ => {
        TmsRespond!(Status::BadRequest, "Failed to find match".to_string());
      }
    };
    
    matches.push(game_match.clone());
  }

  let matches_response = MatchesResponse {
    matches
  };

  TmsRespond!(
    Status::Ok,
    matches_response,
    clients,
    uuid
  )
}

#[tms_private_route]
#[post("/match/get/<uuid>", data = "<message>")]
pub fn match_get_route(message: String) -> TmsRouteResponse<()> {
  let match_request: MatchRequest = TmsRequest!(message.clone(), security);
  match db.tms_data.matches.get(match_request.match_number.clone()).unwrap() {
    Some(m) => {
      let match_response: MatchResponse = MatchResponse { game_match: m.clone() };

      TmsRespond!(
        Status::Ok,
        match_response,
        clients,
        uuid
      )
    },
    None => {
      TmsRespond!(Status::NotFound, "Failed to find match".to_string());
    }
  };
}

#[tms_private_route]
#[post("/match/update/<uuid>", data = "<message>")]
pub fn match_update_route(message: String) -> TmsRouteResponse<()> {
  let message: MatchUpdateRequest = TmsRequest!(message.clone(), security);

  let mut perms = create_permissions();
  perms.head_referee = Some(true);
  perms.referee = Some(true); // referees can update if a score has been submitted or not
  if check_permissions(clients, uuid, message.auth_token, perms) {
    match db.tms_data.matches.get(message.match_number.clone()).unwrap() {
      Some(m) => {
        let origin_match_number = m.match_number.clone();
        match db.tms_data.matches.update(origin_match_number.as_bytes(), message.match_data.match_number.as_bytes(), message.match_data.clone()) {
          Ok(_) => {},
          Err(e) => {
            match e {
              UpdateError::KeyExists => {
                error!("Failed to update match, match number already exists");
                TmsRespond!(Status::BadRequest, "Failed to update match, match number already exists".to_string());
              },
              _ => {
                TmsRespond!(Status::BadRequest, "Failed to update match".to_string());
              }
            }
          }
        }
        // send updates to clients
        tms_clients_ws_send(SocketMessage {
          from_id: None,
          topic: String::from("match"),
          sub_topic: String::from("update"),
          message: origin_match_number.clone(),
        }, clients.inner().clone(), None);

        // send another update for the new match (if the new match number changed from origin)
        tms_clients_ws_send(SocketMessage {
          from_id: None,
          topic: String::from("match"),
          sub_topic: String::from("update"),
          message: message.match_data.match_number.clone(),
        }, clients.inner().clone(), None);
        TmsRespond!();
      },
      None => {
        TmsRespond!(Status::NotFound, "Failed to find match".to_string());
      }
    };
  }

  TmsRespond!(Status::Unauthorized)
}

#[tms_private_route]
#[post("/match/load/<uuid>", data = "<message>")]
pub fn match_load_route(tms_event_service: &State<TmsEventServiceArc>, message: String) -> TmsRouteResponse<()> {
  let message: MatchLoadRequest = TmsRequest!(message.clone(), security);
  let mut perms = create_permissions();
  perms.head_referee = Some(true);
  if check_permissions(clients, uuid, message.auth_token, perms) {
    tms_event_service.lock().unwrap().match_control.load_matches(message.match_numbers);
    TmsRespond!()
  }
  
  TmsRespond!(Status::Unauthorized)
}

#[tms_private_route]
#[post("/match/unload/<uuid>", data = "<message>")]
pub fn match_unload_route(tms_event_service: &State<TmsEventServiceArc>, message: String) -> TmsRouteResponse<()> {
  let message: MatchLoadRequest = TmsRequest!(message.clone(), security);
  let mut perms = create_permissions();
  perms.head_referee = Some(true);
  if check_permissions(clients, uuid, message.auth_token, perms) {
    tms_event_service.lock().unwrap().match_control.unload_matches();
    TmsRespond!()
  }
  
  TmsRespond!(Status::Unauthorized)
}

#[tms_private_route]
#[post("/match/delete/<uuid>", data = "<message>")]
pub fn match_delete_route(message: String) -> TmsRouteResponse<()> {
  let message: MatchDeleteRequest = TmsRequest!(message.clone(), security);
  let mut perms = create_permissions();
  perms.head_referee = Some(true);

  if check_permissions(clients, uuid, message.auth_token, perms) {
    match db.tms_data.matches.get(message.match_number.clone()).unwrap() {
      Some(_) => {
        let _ = db.tms_data.matches.remove(message.match_number.as_bytes());
        // send updates to clients
        tms_clients_ws_send(SocketMessage {
          from_id: None,
          topic: String::from("matches"),
          sub_topic: String::from("update"),
          message: "".to_string(),
        }, clients.inner().clone(), None);
        TmsRespond!()
      },
      None => {
        error!("Failed to get match (delete) {}", message.match_number);
        TmsRespond!(Status::NotFound, "Failed to get match".to_string());
      }
    };
  }

  TmsRespond!(Status::Unauthorized)
}

#[tms_private_route]
#[post("/match/add/<uuid>", data = "<message>")]
pub fn match_add_route(message: String) -> TmsRouteResponse<()> {
  let message: MatchAddRequest = TmsRequest!(message.clone(), security);
  let mut perms = create_permissions();
  perms.head_referee = Some(true);

  if check_permissions(clients, uuid, message.auth_token, perms) {
    let match_number = message.match_data.match_number.clone();
    let match_data = message.match_data.clone();
    let _ = db.tms_data.matches.insert(match_number.as_bytes(), match_data);
    // send updates to clients
    tms_clients_ws_send(SocketMessage {
      from_id: None,
      topic: String::from("matches"),
      sub_topic: String::from("update"),
      message: "".to_string(),
    }, clients.inner().clone(), None);
    TmsRespond!()
  }

  TmsRespond!(Status::Unauthorized)
}
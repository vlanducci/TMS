use log::{info, error};
use rocket::{State, post, get, http::Status};
use tms_macros::tms_private_route;
use tms_utils::security::encrypt;
use tms_utils::{TmsRouteResponse, TmsRespond, security::Security, TmsClients, TmsRequest, schemas::create_permissions, check_permissions, network_schemas::{SetupRequest, PurgeRequest, EventResponse, SocketMessage, ApiLinkRequest, ApiLinkResponse}, tms_clients_ws_send};

use crate::db::db::TmsDB;

#[get("/event/get/<uuid>")]
pub fn event_get_route(clients: &State<TmsClients>, db: &State<std::sync::Arc<TmsDB>>, uuid: String) -> TmsRouteResponse<()> {
  let event = match db.tms_data.event.get().unwrap() {
    Some(event) => event,
    None => {
      println!("Event not found");
      TmsRespond!(Status::NotFound, "Event not found".to_string());
    }
  };

  let event_response = EventResponse {
    event
  };

  TmsRespond!(
    Status::Ok,
    event_response,
    clients,
    uuid
  )
}

#[tms_private_route]
#[post("/event/get/api_link/<uuid>", data = "<message>")]
pub fn event_get_api_link_route(message: String) -> TmsRouteResponse<()> {
  let message: ApiLinkRequest = TmsRequest!(message.clone(), security);

  let perms = create_permissions(); // only admin can get api link
  if check_permissions(clients, uuid.clone(), message.auth_token, perms) {
    match db.tms_data.api_link.get().unwrap() {
      Some(api_link) => {
        let api_response = ApiLinkResponse {
          api_link
        };
        TmsRespond!(
          Status::Ok,
          api_response,
          clients,
          uuid
        )
      },
      None => {
        println!("API Link not found");
        TmsRespond!(Status::NotFound, "API Link not found".to_string());
      }
    };
  }
  TmsRespond!(Status::Unauthorized)
}

#[tms_private_route]
#[post("/event/purge/<uuid>", data = "<message>")]
pub fn event_purge_route(message: String) -> TmsRouteResponse<()> {
  let message: PurgeRequest = TmsRequest!(message.clone(), security);

  let perms = create_permissions(); // only admin can purge
  if check_permissions(clients, uuid, message.auth_token, perms) {
    match db.purge() {
      Ok(_) => {
        info!("Database purged successfully");
        db.setup_default();
        

        // send event update
        tms_clients_ws_send(SocketMessage {
          from_id: None,
          topic: String::from("event"),
          sub_topic: String::from("update"),
          message: String::from("")
        }, clients.inner().to_owned(), None);
        
        // send teams update
        tms_clients_ws_send(SocketMessage {
          from_id: None,
          topic: String::from("teams"),
          sub_topic: String::from("update"),
          message: String::from("")
        }, clients.inner().to_owned(), None);

        // send matches update
        tms_clients_ws_send(SocketMessage {
          from_id: None,
          topic: String::from("matches"),
          sub_topic: String::from("update"),
          message: String::from("")
        }, clients.inner().to_owned(), None);

        // send judging sessions update
        tms_clients_ws_send(SocketMessage {
          from_id: None,
          topic: String::from("judging_sessions"),
          sub_topic: String::from("update"),
          message: String::from("")
        }, clients.inner().to_owned(), None);

        // good response
        TmsRespond!()
      },
      Err(e) => {
        error!("Failed to purge database: {}", e);
        TmsRespond!(Status::BadRequest, format!("Failed to purge database: {}", e));
      }
    }
  }

  TmsRespond!(Status::Unauthorized)
}

#[tms_private_route]
#[post("/event/setup/<uuid>", data = "<message>")]
pub fn event_setup_route(message: String) -> TmsRouteResponse<()> {
  let message: SetupRequest = TmsRequest!(message.clone(), security);

  let perms = create_permissions(); // only admin can setup
  if check_permissions(clients, uuid, message.auth_token, perms) {
    // Put data into database

    // supply new admin password
    if message.admin_password != "" {
      let mut user = match db.tms_data.users.get(String::from("admin")).unwrap() {
        Some(user) => user,
        None => {
          TmsRespond!(Status::NotFound, "Admin user not found".to_string());
        }
      };

      user.password = message.admin_password;
      let _ = db.tms_data.users.insert("admin".as_bytes(), user);
    }

    // supply new teams
    for team in message.teams {
      match db.tms_data.teams.insert(team.team_number.as_bytes(), team.clone()) {
        Ok(_) => {
          info!("Team {} setup successfully", team.team_number);
        },
        Err(e) => {
          error!("Failed to setup team {}: {}", team.team_number, e);
          TmsRespond!(Status::BadRequest, format!("Failed to setup team {}", team.team_number));
        }
      }
    }

    // supply new matches
    for game_match in message.matches {
      match db.tms_data.matches.insert(game_match.match_number.as_bytes(), game_match.clone()) {
        Ok(_) => {
          info!("Match {} setup successfully", game_match.match_number);
        },
        Err(e) => {
          error!("Failed to setup match {}: {}", game_match.match_number, e);
          TmsRespond!(Status::BadRequest, format!("Failed to setup match {}", game_match.match_number));
        }
      }
    }

    // supply new judging sessions (the team number is the key)
    for judging_session in message.judging_sessions {
      match db.tms_data.judging_sessions.insert(judging_session.session_number.as_bytes(), judging_session.clone()) {
        Ok(_) => {
          info!("Successfully setup Judging Session {}", judging_session.session_number);
        },
        Err(e) => {
          error!("Failed to setup judging session {}: {}", judging_session.session_number, e);
          TmsRespond!(Status::BadRequest, format!("Failed to setup judging session {}", judging_session.session_number));
        }
      }
    }

    // setup users
    for user in message.users {
      match db.tms_data.users.insert(user.username.as_bytes(), user.clone()) {
        Ok(_) => {
          info!("User {} setup successfully", user.username);
        },
        Err(e) => {
          error!("Failed to setup user {}: {}", user.username, e);
          TmsRespond!(Status::BadRequest, format!("Failed to setup user {}", user.username));
        }
      }
    }

    // supply new event
    match db.tms_data.event.set(message.event) {
      Ok(_) => {
        info!("Event setup successfully");
      },
      Err(e) => {
        error!("Failed to setup event: {}", e);
        TmsRespond!(Status::BadRequest, "Failed to setup event".to_string());
      }
    }


    // send event update
    tms_clients_ws_send(SocketMessage {
      from_id: None,
      topic: String::from("event"),
      sub_topic: String::from("update"),
      message: String::from("")
    }, clients.inner().to_owned(), None);

    // send game update
    tms_clients_ws_send(SocketMessage {
      from_id: None,
      topic: String::from("game"),
      sub_topic: String::from("update"),
      message: String::from("")
    }, clients.inner().to_owned(), None);
    
    // send teams update
    tms_clients_ws_send(SocketMessage {
      from_id: None,
      topic: String::from("teams"),
      sub_topic: String::from("update"),
      message: String::from("")
    }, clients.inner().to_owned(), None);

    // send matches update
    tms_clients_ws_send(SocketMessage {
      from_id: None,
      topic: String::from("matches"),
      sub_topic: String::from("update"),
      message: String::from("")
    }, clients.inner().to_owned(), None);

    // send judging sessions update
    tms_clients_ws_send(SocketMessage {
      from_id: None,
      topic: String::from("judging_sessions"),
      sub_topic: String::from("update"),
      message: String::from("")
    }, clients.inner().to_owned(), None);

    // good response
    TmsRespond!()
  }
  
  TmsRespond!(Status::Unauthorized)
}
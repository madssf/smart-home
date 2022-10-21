use std::sync::Arc;

use actix_web::{delete, get, post, web, HttpResponse, Responder, Scope};
use chrono::NaiveDateTime;
use log::error;
use serde::Deserialize;
use uuid::Uuid;

use crate::db::temp_actions::TempActionsClient;
use crate::domain::{ActionType, TempAction};

pub fn temp_actions(temp_actions_client: Arc<TempActionsClient>) -> Scope {
    let temp_actions_client = web::Data::new(temp_actions_client);
    web::scope("/temp_actions")
        .app_data(temp_actions_client)
        .service(get_temp_actions)
        .service(create_temp_action)
        .service(update_temp_action)
        .service(delete_temp_action)
}

#[get("/")]
async fn get_temp_actions(
    temp_actions_client: web::Data<Arc<TempActionsClient>>,
) -> impl Responder {
    match temp_actions_client.get_ref().get_temp_actions().await {
        Ok(temp_actions) => HttpResponse::Ok().json(temp_actions),
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Deserialize)]
struct TempActionRequest {
    pub room_ids: Vec<Uuid>,
    pub action_type: ActionType,
    pub expires_at: NaiveDateTime,
}

impl TryInto<TempAction> for TempActionRequest {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<TempAction, Self::Error> {
        TempAction::new(&self.expires_at, &self.action_type, self.room_ids)
    }
}

#[post("/")]
async fn create_temp_action(
    temp_actions_client: web::Data<Arc<TempActionsClient>>,
    body: web::Json<TempActionRequest>,
) -> impl Responder {
    let new_action = match body.into_inner().try_into() {
        Ok(temp_action) => temp_action,
        Err(e) => {
            error!("{}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    match temp_actions_client
        .get_ref()
        .create_temp_action(new_action)
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/{id}")]
async fn update_temp_action(
    temp_actions_client: web::Data<Arc<TempActionsClient>>,
    id: web::Path<Uuid>,
    body: web::Json<TempActionRequest>,
) -> impl Responder {
    match temp_actions_client
        .get_ref()
        .update_temp_action(TempAction {
            id: id.into_inner(),
            room_ids: body.room_ids.clone(),
            action_type: body.action_type,
            expires_at: body.expires_at,
        })
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/{id}")]
async fn delete_temp_action(
    temp_actions_client: web::Data<Arc<TempActionsClient>>,
    id: web::Path<Uuid>,
) -> impl Responder {
    match temp_actions_client
        .get_ref()
        .delete_temp_action(&id.into_inner())
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

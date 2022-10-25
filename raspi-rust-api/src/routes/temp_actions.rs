use actix_web::{delete, get, post, web, HttpResponse, Responder, Scope};
use chrono::NaiveDateTime;
use log::error;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db;
use crate::domain::{ActionType, TempAction};

pub fn temp_actions() -> Scope {
    web::scope("/temp_actions")
        .service(get_temp_actions)
        .service(create_temp_action)
        .service(update_temp_action)
        .service(delete_temp_action)
}

#[get("/")]
async fn get_temp_actions(pool: web::Data<PgPool>) -> impl Responder {
    match db::temp_actions::get_temp_actions(pool.get_ref()).await {
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
    pool: web::Data<PgPool>,
    body: web::Json<TempActionRequest>,
) -> impl Responder {
    let new_action = match body.into_inner().try_into() {
        Ok(temp_action) => temp_action,
        Err(e) => {
            error!("{}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    match db::temp_actions::create_temp_action(pool.get_ref(), new_action).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/{id}")]
async fn update_temp_action(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
    body: web::Json<TempActionRequest>,
) -> impl Responder {
    match db::temp_actions::update_temp_action(
        pool.get_ref(),
        TempAction {
            id: id.into_inner(),
            room_ids: body.room_ids.clone(),
            action_type: body.action_type,
            expires_at: body.expires_at,
        },
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/{id}")]
async fn delete_temp_action(pool: web::Data<PgPool>, id: web::Path<Uuid>) -> impl Responder {
    match db::temp_actions::delete_temp_action(pool.get_ref(), &id.into_inner()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

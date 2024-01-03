use std::sync::Arc;

use actix_web::{delete, get, post, web, HttpResponse, Responder, Scope};
use log::error;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db;
use crate::domain::{
    ActionType, TempAction, TempActionRequest, TempActionResponse, TempActionType,
};

pub fn temp_actions() -> Scope {
    web::scope("/temp_actions")
        .service(get_temp_actions)
        .service(create_temp_action)
        .service(update_temp_action)
        .service(delete_temp_action)
}

#[get("/")]
async fn get_temp_actions(pool: web::Data<Arc<PgPool>>) -> impl Responder {
    match db::temp_actions::get_temp_actions(pool.get_ref()).await {
        Ok(temp_actions) => {
            let json: Vec<TempActionResponse> =
                temp_actions.into_iter().map(|t| t.into()).collect();
            HttpResponse::Ok().json(json)
        }
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/")]
async fn create_temp_action(
    pool: web::Data<Arc<PgPool>>,
    body: web::Json<TempActionRequest>,
) -> impl Responder {
    let new_action: TempAction = body.into_inner().into();
    match db::temp_actions::create_temp_action(pool.get_ref(), new_action).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/{id}")]
async fn update_temp_action(
    pool: web::Data<Arc<PgPool>>,
    id: web::Path<Uuid>,
    body: web::Json<TempActionRequest>,
) -> impl Responder {
    let action_type = match body.action {
        ActionType::ON => TempActionType::ON(body.temp),
        ActionType::OFF => TempActionType::OFF,
    };
    match db::temp_actions::update_temp_action(
        pool.get_ref(),
        TempAction {
            id: id.into_inner(),
            room_ids: body.room_ids.clone(),
            action_type,
            expires_at: body.expires_at,
            starts_at: body.starts_at,
        },
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/{id}")]
async fn delete_temp_action(pool: web::Data<Arc<PgPool>>, id: web::Path<Uuid>) -> impl Responder {
    match db::temp_actions::delete_temp_action(pool.get_ref(), &id.into_inner()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

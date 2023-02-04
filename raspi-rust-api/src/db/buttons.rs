use sqlx::types::ipnetwork::IpNetwork;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::DbError;
use crate::domain::Button;

#[derive(Clone, Debug)]
struct ButtonEntity {
    id: Uuid,
    ip: IpNetwork,
    name: String,
    username: String,
    password: String,
}

impl ButtonEntity {
    fn to_domain(&self, plugs: &[ButtonPlugEntity]) -> Button {
        Button {
            id: self.id,
            name: self.name.clone(),
            ip: self.ip,
            username: self.username.clone(),
            password: self.password.clone(),
            plug_ids: plugs
                .iter()
                .filter_map(|p| match p.button_id == self.id {
                    true => Some(p.plug_id),
                    false => None,
                })
                .collect(),
        }
    }
}
#[derive(Copy, Clone, Debug)]
struct ButtonPlugEntity {
    button_id: Uuid,
    plug_id: Uuid,
}

fn to_entity(button: &Button) -> (ButtonEntity, Vec<ButtonPlugEntity>) {
    let button_entity = ButtonEntity {
        id: button.id,
        ip: button.ip,
        name: button.name.clone(),
        username: button.username.clone(),
        password: button.password.clone(),
    };
    let button_plug_entities = button
        .plug_ids
        .iter()
        .map(|p| ButtonPlugEntity {
            button_id: button.id,
            plug_id: *p,
        })
        .collect();
    (button_entity, button_plug_entities)
}

pub async fn get_buttons(pool: &PgPool) -> Result<Vec<Button>, DbError> {
    let entities: Vec<ButtonEntity> = sqlx::query_as!(ButtonEntity, "SELECT * FROM buttons")
        .fetch_all(pool)
        .await?;

    let button_plugs = sqlx::query_as!(ButtonPlugEntity, "SELECT * FROM button_plugs")
        .fetch_all(pool)
        .await?;

    Ok(entities
        .iter()
        .map(|entity| entity.to_domain(&button_plugs))
        .collect())
}

pub async fn create_button(pool: &PgPool, new_button: &Button) -> Result<(), DbError> {
    let (button_entity, button_plug_entities) = to_entity(new_button);

    let mut tx = pool.begin().await?;
    sqlx::query!(
        r#"
    INSERT INTO buttons (id, ip, name, password, username)
    VALUES ($1, $2, $3, $4, $5)
    "#,
        button_entity.id,
        button_entity.ip,
        button_entity.name,
        button_entity.password,
        button_entity.username,
    )
    .execute(&mut tx)
    .await?;

    for plug in button_plug_entities {
        sqlx::query!(
            r#"
        INSERT INTO button_plugs (button_id, plug_id)
        VALUES ($1, $2)
        "#,
            plug.button_id,
            plug.plug_id,
        )
        .execute(&mut tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

pub async fn update_button(pool: &PgPool, button: &Button) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;

    let (button_entity, _button_plug_entities) = to_entity(button);

    sqlx::query!(
        r#"
        UPDATE buttons
        SET ip = $2, name = $3, password = $4, username = $5
        WHERE id = $1
        "#,
        button_entity.id,
        button_entity.ip,
        button_entity.name,
        button_entity.password,
        button_entity.username,
    )
    .execute(&mut tx)
    .await?;

    let existing_button_plugs: Vec<ButtonPlugEntity> = sqlx::query_as!(
        ButtonPlugEntity,
        "SELECT * FROM button_plugs WHERE button_id = $1",
        &button.id
    )
    .fetch_all(&mut tx)
    .await?;

    for plug_id in &button.plug_ids {
        if !existing_button_plugs.iter().any(|x| x.plug_id == *plug_id) {
            sqlx::query!(
                r#"
            INSERT INTO button_plugs (button_id, plug_id)
            VALUES ($1, $2)
            "#,
                &button.id,
                plug_id,
            )
            .execute(&mut tx)
            .await?;
        }
    }

    for existing in existing_button_plugs {
        if !button.plug_ids.contains(&existing.plug_id) {
            sqlx::query!(
                r#"
                DELETE FROM button_plugs WHERE plug_id = $1 AND button_id = $2
                "#,
                existing.plug_id,
                existing.button_id,
            )
            .execute(&mut tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

pub async fn delete_button(pool: &PgPool, id: &Uuid) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;

    sqlx::query!("DELETE FROM button_plugs WHERE button_id = $1", id)
        .execute(&mut tx)
        .await?;

    sqlx::query!("DELETE FROM buttons WHERE id = $1", id)
        .execute(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

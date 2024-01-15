use crate::AppState;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::body::AsyncReadBody;
use entity::uploads as Uploads;
use sea_orm::EntityTrait;
use tokio::fs::File;

pub async fn get_file(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let file = match Uploads::Entity::find_by_id(id).one(state.get_pool()).await {
        Ok(Some(f)) => f,
        _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let f: Vec<&str> = file.filename.split(".").collect();

    match f.get(1) {
        Some(ext) => match ext {
            &"png" => {
                let file = match File::open(format!("uploads/{}", file.filename)).await {
                    Ok(f) => f,
                    _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                };

                (
                    StatusCode::OK,
                    [
                        (header::CONTENT_TYPE, "image/png"),
                        (header::ACCEPT_RANGES, "bytes, bytes"),
                    ],
                    AsyncReadBody::new(file),
                )
                    .into_response()
            }
            &"gif" => {
                let file = match File::open(format!("uploads/{}", file.filename)).await {
                    Ok(f) => f,
                    _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                };

                (
                    StatusCode::OK,
                    [
                        (header::CONTENT_TYPE, "image/gif"),
                        (header::ACCEPT_RANGES, "bytes, bytes"),
                    ],
                    AsyncReadBody::new(file),
                )
                    .into_response()
            }
            _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
        _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

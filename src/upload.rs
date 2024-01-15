use std::{fs::File, io::Cursor};

use crate::AppState;
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use entity::uploads as Uploads;
use image::{
    codecs::gif::{GifDecoder, GifEncoder},
    io::Reader,
};
use image::{AnimationDecoder, ImageFormat};
use sea_orm::{ActiveModelTrait, Set};
use serde_json::json;

pub async fn upload(State(state): State<AppState>, mut multipart: Multipart) -> Response {
    let mut is_gif = false;

    let field = match multipart.next_field().await {
        Ok(Some(field)) => field,
        _ => return StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS.into_response(),
    };

    if let Some(content_type) = field.content_type() {
        if content_type == "image/gif" {
            is_gif = true;
        } else if !content_type.contains("image") {
            return StatusCode::UNPROCESSABLE_ENTITY.into_response();
        }
    }

    let image_bytes = match field.bytes().await {
        Ok(b) => b,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let id = cuid2::create_id();

    match is_gif {
        true => {
            let gif = match GifDecoder::new(std::io::Cursor::new(image_bytes)) {
                Ok(g) => g,
                _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

            let gif_out = match File::create(format!("./uploads/{}.gif", id)) {
                Ok(g) => g,
                _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

            let mut encoder = GifEncoder::new(gif_out);

            let frames = match gif.into_frames().collect_frames() {
                Ok(g) => g,
                _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

            match encoder.set_repeat(image::codecs::gif::Repeat::Infinite) {
                Ok(_) => (),
                _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

            match encoder.encode_frames(frames.into_iter()) {
                Ok(_) => {
                    Uploads::ActiveModel {
                        id: Set(id.clone()),
                        filename: Set(format!("{}.gif", id)),
                    }
                    .insert(state.get_pool())
                    .await
                    .unwrap();
                    return (
                        StatusCode::OK,
                        Json(json!({
                                "url": format!("http://localhost/uploads/{}", id),
                                "display_message": "".to_string(),
                        })),
                    )
                        .into_response();
                }
                _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };
        }
        false => {
            let img = match Reader::new(Cursor::new(image_bytes))
                .with_guessed_format()
                .unwrap()
                .decode()
            {
                Ok(img) => img,
                _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

            match img.save_with_format(format!("./uploads/{}.png", id), ImageFormat::Png) {
                Ok(_) => {
                    Uploads::ActiveModel {
                        id: Set(id.clone()),
                        filename: Set(format!("{}.png", id)),
                    }
                    .insert(state.get_pool())
                    .await
                    .unwrap();
                    return (
                        StatusCode::OK,
                        Json(json!({
                                "url": format!("http://localhost/uploads/{}", id),
                                "display_message": "".to_string(),
                        })),
                    )
                        .into_response();
                }
                _ => return StatusCode::IM_A_TEAPOT.into_response(),
            }
        }
    }

    // StatusCode::OK.into_response()
}

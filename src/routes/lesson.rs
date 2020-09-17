use actix_web::{delete, get, patch, put, web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::vec::Vec;
use chrono::NaiveDate;

use crate::error::{APIError, Result};
use crate::middleware::{Authentication, CheckLessonPermission, PathExtractor};
use crate::model::{
    account::AccountID,
    lesson::{Lesson, LessonID},
    permission::PermissionType,
    repeat::Repeat,
    single_occurrence::SingleOccurrence,
};
use crate::payload::Payload;
use crate::util::deserialize_optional_field;

#[get(
    "/lesson/{id}",
    wrap = "CheckLessonPermission::new(PermissionType::Read)",
    wrap = "PathExtractor::<LessonID>::new()",
    wrap = "Authentication"
)]
pub async fn get_lesson(db: web::Data<PgPool>, lesson_id: LessonID) -> Result<Lesson> {
    Lesson::by_id(db.get_ref(), lesson_id)
        .await?
        .ok_or(APIError::LessonDosNotExist)
        .map(Payload::from)
}

#[derive(Deserialize)]
pub struct LessonCreateRequest {
    title: String,
    description: Option<String>,
    repeats: Option<Vec<Repeat>>,
    singles: Option<Vec<SingleOccurrence>>,
}

#[put("/lesson", wrap = "Authentication")]
pub async fn put_lesson(
    db: web::Data<PgPool>,
    lesson: web::Json<LessonCreateRequest>,
    account_id: AccountID,
) -> Result<Lesson> {
    let LessonCreateRequest {
        title,
        description,
        repeats,
        singles,
    } = lesson.into_inner();

    Ok(Lesson::create(
        db.get_ref(),
        title,
        description,
        repeats.unwrap_or_default(),
        singles.unwrap_or_default(),
        &account_id,
    )
    .await?
    .into())
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct LessonUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_field")]
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repeats: Option<Vec<Repeat>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    singles: Option<Vec<SingleOccurrence>>,
}

#[patch(
    "/lesson/{id}",
    wrap = "CheckLessonPermission::new(PermissionType::ReadWrite)",
    wrap = "PathExtractor::<LessonID>::new()",
    wrap = "Authentication"
)]
pub async fn patch_lesson(
    db: web::Data<PgPool>,
    lesson_id: LessonID,
    patch: web::Json<LessonUpdateRequest>,
) -> std::result::Result<HttpResponse, APIError> {
    let LessonUpdateRequest {
        title,
        repeats,
        description,
        singles,
    } = patch.into_inner();

    Lesson::update(
        db.get_ref(),
        &lesson_id,
        &title,
        &repeats,
        &singles,
        &description,
    )
    .await?;
    Ok(HttpResponse::NoContent().finish())
}

#[delete(
    "/lesson/{id}",
    wrap = "CheckLessonPermission::new(PermissionType::ReadWrite)",
    wrap = "PathExtractor::<LessonID>::new()",
    wrap = "Authentication"
)]
pub async fn delete_lesson(
    db: web::Data<PgPool>,
    lesson_id: LessonID,
) -> std::result::Result<HttpResponse, APIError> {
    Lesson::delete(db.get_ref(), &lesson_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize)]
pub struct GetLessonsQuery {
    date: NaiveDate
}

#[get("/lessons", wrap = "Authentication")]
pub async fn get_lessons(
    db: web::Data<PgPool>,
    date: web::Query<GetLessonsQuery>,
    account_id: AccountID
) -> Result<Vec<Lesson>> {
    let GetLessonsQuery { date } = date.into_inner();
    Ok(Lesson::for_date(db.get_ref(), &date, &account_id).await?.into())
}

pub fn configure_lesson_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_lesson)
        .service(put_lesson)
        .service(patch_lesson)
        .service(delete_lesson)
        .service(get_lessons);
}

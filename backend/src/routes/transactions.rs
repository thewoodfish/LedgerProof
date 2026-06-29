use crate::{
    error::{AppError, AppResult},
    models::transaction::Transaction,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ListQuery {
    pub merchant_id: Option<Uuid>,
    pub category: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Vec<Transaction>>> {
    let merchant_id = q.merchant_id.unwrap_or_else(|| {
        crate::routes::statements::merchant_id_from_headers(&headers)
    });
    let limit = q.limit.unwrap_or(500).min(1000);
    let offset = q.offset.unwrap_or(0);

    let rows = if let Some(cat) = &q.category {
        sqlx::query_as::<_, Transaction>(
            "SELECT * FROM transactions
             WHERE merchant_id = $1 AND category = $2
             ORDER BY date ASC
             LIMIT $3 OFFSET $4",
        )
        .bind(merchant_id)
        .bind(cat)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, Transaction>(
            "SELECT * FROM transactions
             WHERE merchant_id = $1
             ORDER BY date ASC
             LIMIT $2 OFFSET $3",
        )
        .bind(merchant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(rows))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Transaction>> {
    let row = sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("transaction not found".into()))?;

    Ok(Json(row))
}

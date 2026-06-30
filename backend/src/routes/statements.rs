use crate::{
    error::{AppError, AppResult},
    models::statement::UploadStatementQuery,
    routes::AuthUser,
    services::{classifier, parser},
    AppState,
};
use axum::{
    extract::{Multipart, Path, Query, State},
    Json,
};
use chrono::NaiveDate;
use serde_json::{json, Value};
use uuid::Uuid;

/// POST /upload-statement
pub async fn upload(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<UploadStatementQuery>,
    mut multipart: Multipart,
) -> AppResult<Json<Value>> {
    let merchant_id = auth.id;
    let mut filename = String::from("statement.xlsx");
    let mut file_bytes: Vec<u8> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        if field.name() == Some("file") {
            filename = field.file_name().unwrap_or("statement.xlsx").to_string();
            file_bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(e.to_string()))?
                .to_vec();
        }
    }

    if file_bytes.is_empty() {
        return Err(AppError::BadRequest("No file provided".into()));
    }

    let month = q.month.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m").to_string());
    if !is_valid_month(&month) {
        return Err(AppError::BadRequest("month must be in YYYY-MM format".into()));
    }

    let stmt_id: Uuid = sqlx::query_scalar(
        "INSERT INTO statements (merchant_id, filename, month, status)
         VALUES ($1, $2, $3, 'pending') RETURNING id",
    )
    .bind(merchant_id)
    .bind(&filename)
    .bind(&month)
    .fetch_one(&state.db)
    .await?;

    let db = state.db.clone();
    let config = state.config.clone();

    tokio::spawn(async move {
        match run_parse(stmt_id, merchant_id, &month, &file_bytes, &config, &db).await {
            Ok(_) => {
                let _ = sqlx::query(
                    "UPDATE statements SET status = 'parsed', updated_at = NOW() WHERE id = $1",
                )
                .bind(stmt_id)
                .execute(&db)
                .await;
            }
            Err(e) => {
                tracing::error!("Parse failed for {}: {}", stmt_id, e);
                let _ = sqlx::query(
                    "UPDATE statements SET status = 'error', error_msg = $2, updated_at = NOW()
                     WHERE id = $1",
                )
                .bind(stmt_id)
                .bind(e.to_string())
                .execute(&db)
                .await;
            }
        }
    });

    Ok(Json(json!({
        "statement_id": stmt_id,
        "status": "pending",
        "message": "Statement uploaded. Parsing in background.",
    })))
}

/// POST /parse/:statement_id — re-trigger parse status check
pub async fn parse(
    State(state): State<AppState>,
    Path(stmt_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let status: Option<String> =
        sqlx::query_scalar("SELECT status FROM statements WHERE id = $1")
            .bind(stmt_id)
            .fetch_optional(&state.db)
            .await?;

    let status = status.ok_or_else(|| AppError::NotFound("Statement not found".into()))?;
    Ok(Json(json!({ "statement_id": stmt_id, "status": status })))
}

// ── internals ──────────────────────────────────────────────────────────────

async fn run_parse(
    stmt_id: Uuid,
    merchant_id: Uuid,
    month: &str,
    file_bytes: &[u8],
    _config: &std::sync::Arc<crate::config::Config>,
    db: &sqlx::PgPool,
) -> anyhow::Result<()> {
    let raw_txns = parser::parse_statement(file_bytes)?;

    for raw in &raw_txns {
        let date = parse_date(&raw.date)?;
        let credit = (raw.credit * 100.0) as i64;
        let debit = (raw.debit * 100.0) as i64;
        let balance = (raw.balance * 100.0) as i64;
        let category = classifier::classify(&raw.description, credit, debit);

        sqlx::query(
            "INSERT INTO transactions
             (statement_id, merchant_id, date, description, credit, debit, balance, category)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(stmt_id)
        .bind(merchant_id)
        .bind(date)
        .bind(&raw.description)
        .bind(credit)
        .bind(debit)
        .bind(balance)
        .bind(category.to_string())
        .execute(db)
        .await?;
    }

    let summary = format!("Parsed {} transactions for {}", raw_txns.len(), month);
    sqlx::query("UPDATE statements SET raw_text = $2 WHERE id = $1")
        .bind(stmt_id)
        .bind(summary)
        .execute(db)
        .await?;

    Ok(())
}

fn parse_date(s: &str) -> anyhow::Result<NaiveDate> {
    let formats = ["%Y-%m-%d", "%d/%m/%Y", "%d-%m-%Y", "%m/%d/%Y", "%d %b %Y", "%d %B %Y"];
    for fmt in &formats {
        if let Ok(d) = NaiveDate::parse_from_str(s.trim(), fmt) {
            return Ok(d);
        }
    }
    Err(anyhow::anyhow!("unparseable date: {s}"))
}

fn is_valid_month(s: &str) -> bool {
    let parts: Vec<&str> = s.splitn(2, '-').collect();
    parts.len() == 2
        && parts[0].len() == 4
        && parts[0].chars().all(|c| c.is_ascii_digit())
        && parts[1].len() == 2
        && parts[1].chars().all(|c| c.is_ascii_digit())
}

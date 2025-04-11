use async_trait::async_trait;
use sqlx::{PgPool, postgres::PgQueryResult, Row};
use chrono::{DateTime, Utc};

use crate::model::advertisement::advertisement::{Advertisement, AdvertisementStatus};
use crate::dto::advertisement::advertisement::AdvertisementQueryParams;
use crate::repository::advertisement::advertisement_repository::AdvertisementRepository;

pub struct PostgresAdvertisementRepository {
    pool: PgPool,
}

impl PostgresAdvertisementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    fn build_query(&self, params: &AdvertisementQueryParams) -> (String, Vec<String>) {
        let mut conditions = Vec::new();
        let mut values = Vec::new();
        
        if let Some(status) = &params.status {
            conditions.push(format!("status = '{}'", status));
        }
        
        if let Some(start_date_from) = params.start_date_from {
            conditions.push(format!("start_date >= '{}'", start_date_from));
        }
        
        if let Some(start_date_to) = params.start_date_to {
            conditions.push(format!("start_date <= '{}'", start_date_to));
        }
        
        if let Some(end_date_from) = params.end_date_from {
            conditions.push(format!("end_date >= '{}'", end_date_from));
        }
        
        if let Some(end_date_to) = params.end_date_to {
            conditions.push(format!("end_date <= '{}'", end_date_to));
        }
        
        if let Some(search) = &params.search {
            conditions.push(format!("title ILIKE '%{}%'", search));
        }
        
        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };
        
        (where_clause, values)
    }
}

#[async_trait]
impl AdvertisementRepository for PostgresAdvertisementRepository {
    async fn find_all(&self, params: &AdvertisementQueryParams) -> Result<(Vec<Advertisement>, i64), anyhow::Error> {
        let (where_clause, _) = self.build_query(params);
        
        let limit = params.limit.unwrap_or(10).min(50);
        let offset = (params.page.unwrap_or(1) - 1) * limit;
        
        // Count total records
        let count_query = format!(
            "SELECT COUNT(*) FROM advertisements {}",
            where_clause
        );
        
        let total: i64 = sqlx::query(&count_query)
            .fetch_one(&self.pool)
            .await?
            .get(0);
        
        // Get paginated results
        let query = format!(
            "SELECT id, title, image_url, start_date, end_date, status, click_url, created_at, updated_at 
             FROM advertisements 
             {} 
             ORDER BY created_at DESC 
             LIMIT {} OFFSET {}",
            where_clause, limit, offset
        );
        
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await?;
        
        let advertisements = rows.into_iter().map(|row| {
            Advertisement {
                id: row.get("id"),
                title: row.get("title"),
                image_url: row.get("image_url"),
                start_date: row.get("start_date"),
                end_date: row.get("end_date"),
                status: AdvertisementStatus::from(row.get::<String, _>("status")),
                click_url: row.get("click_url"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }).collect();
        
        Ok((advertisements, total))
    }
    
    async fn find_by_id(&self, id: &str) -> Result<Option<Advertisement>, anyhow::Error> {
        let query = 
            "SELECT id, title, image_url, start_date, end_date, status, click_url, created_at, updated_at 
             FROM advertisements 
             WHERE id = $1";
        
        let row = sqlx::query(query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        
        let advertisement = match row {
            Some(row) => Some(Advertisement {
                id: row.get("id"),
                title: row.get("title"),
                image_url: row.get("image_url"),
                start_date: row.get("start_date"),
                end_date: row.get("end_date"),
                status: AdvertisementStatus::from(row.get::<String, _>("status")),
                click_url: row.get("click_url"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }),
            None => None,
        };
        
        Ok(advertisement)
    }
    
    async fn create(&self, advertisement: &Advertisement) -> Result<Advertisement, anyhow::Error> {
        let query = 
            "INSERT INTO advertisements (id, title, image_url, start_date, end_date, status, click_url, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
             RETURNING id";
        
        sqlx::query(query)
            .bind(&advertisement.id)
            .bind(&advertisement.title)
            .bind(&advertisement.image_url)
            .bind(&advertisement.start_date)
            .bind(&advertisement.end_date)
            .bind(advertisement.status.to_string())
            .bind(&advertisement.click_url)
            .bind(&advertisement.created_at)
            .bind(&advertisement.updated_at)
            .execute(&self.pool)
            .await?;
        
        Ok(advertisement.clone())
    }
    
    async fn update(&self, advertisement: &Advertisement) -> Result<Advertisement, anyhow::Error> {
        let query = 
            "UPDATE advertisements 
             SET title = $2, image_url = $3, start_date = $4, end_date = $5, 
                 status = $6, click_url = $7, updated_at = $8 
             WHERE id = $1";
        
        sqlx::query(query)
            .bind(&advertisement.id)
            .bind(&advertisement.title)
            .bind(&advertisement.image_url)
            .bind(&advertisement.start_date)
            .bind(&advertisement.end_date)
            .bind(advertisement.status.to_string())
            .bind(&advertisement.click_url)
            .bind(Utc::now())
            .execute(&self.pool)
            .await?;
        
        Ok(advertisement.clone())
    }
    
    async fn delete(&self, id: &str) -> Result<bool, anyhow::Error> {
        let result = sqlx::query("DELETE FROM advertisements WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
}
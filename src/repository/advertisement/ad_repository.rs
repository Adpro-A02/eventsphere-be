use async_trait::async_trait;
use sqlx::{Pool, Postgres, query_builder::QueryBuilder, Row};
use std::error::Error as StdError;
use chrono::Utc;
use crate::dto::advertisement::advertisement::AdvertisementQueryParams;
use crate::model::advertisement::advertisement::{Advertisement, AdvertisementStatus};

#[async_trait]
pub trait AdvertisementRepository: Send + Sync {
    async fn find_all(&self, params: &AdvertisementQueryParams) -> Result<(Vec<Advertisement>, i64), Box<dyn StdError>>;
    async fn find_by_id(&self, id: &str) -> Result<Option<Advertisement>, Box<dyn StdError>>;
    async fn create(&self, advertisement: &Advertisement) -> Result<Advertisement, Box<dyn StdError>>;
    async fn update(&self, advertisement: &Advertisement) -> Result<Advertisement, Box<dyn StdError>>;
    async fn delete(&self, id: &str) -> Result<bool, Box<dyn StdError>>;
    async fn increment_impression(&self, id: &str) -> Result<(), Box<dyn StdError>>;
    async fn increment_click(&self, id: &str) -> Result<(), Box<dyn StdError>>;
    async fn find_active(&self, limit: u32) -> Result<Vec<Advertisement>, Box<dyn StdError>>;
}

pub struct PostgresAdvertisementRepository {
    pool: Pool<Postgres>,
}

impl PostgresAdvertisementRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
    
    // Helper to map database row to Advertisement
    fn row_to_advertisement(&self, row: sqlx::postgres::PgRow) -> Advertisement {
        Advertisement {
            id: row.get("id"),
            title: row.get("title"),
            description: row.get("description"),
            image_url: row.get("image_url"),
            start_date: row.get("start_date"),
            end_date: row.get("end_date"),
            status: AdvertisementStatus::from(row.get::<String, _>("status")),
            click_url: row.get("click_url"),
            position: row.get("position"),
            impressions: row.get("impressions"),
            clicks: row.get("clicks"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl AdvertisementRepository for PostgresAdvertisementRepository {
    async fn find_all(&self, params: &AdvertisementQueryParams) -> Result<(Vec<Advertisement>, i64), Box<dyn StdError>> {
        let mut query_builder = QueryBuilder::new(
            "SELECT id, title, description, image_url, start_date, end_date, 
             status, click_url, position, impressions, clicks, 
             created_at, updated_at FROM advertisements WHERE 1=1"
        );
                
        // Add filters based on params
        if let Some(status) = &params.status {
            query_builder.push(" AND status = ").push_bind(status);
        }
        
        if let Some(start_date_from) = params.start_date_from {
            query_builder.push(" AND start_date >= ").push_bind(start_date_from);
        }
        
        if let Some(start_date_to) = params.start_date_to {
            query_builder.push(" AND start_date <= ").push_bind(start_date_to);
        }
        
        if let Some(end_date_from) = params.end_date_from {
            query_builder.push(" AND end_date >= ").push_bind(end_date_from);
        }
        
        if let Some(end_date_to) = params.end_date_to {
            query_builder.push(" AND end_date <= ").push_bind(end_date_to);
        }
        
        if let Some(search) = &params.search {
            query_builder.push(" AND (title ILIKE ").push_bind(format!("%{}%", search))
                         .push(" OR description ILIKE ").push_bind(format!("%{}%", search))
                         .push(")");
        }
        
        // Get total count
        let count_sql = query_builder.sql();
        let count_sql = format!("SELECT COUNT(*) FROM ({}) as cnt", count_sql);
        let total: i64 = sqlx::query_scalar(&count_sql)
            .fetch_one(&self.pool)
            .await?;
            
        // Add pagination
        let limit = params.limit.unwrap_or(10).min(50);
        let offset = (params.page.unwrap_or(1) - 1) * limit;
        
        query_builder.push(" ORDER BY created_at DESC LIMIT ")
                    .push_bind(limit as i64)
                    .push(" OFFSET ")
                    .push_bind(offset as i64);
        
        // Execute query and map results
        let rows = query_builder.build().fetch_all(&self.pool).await?;
        let advertisements = rows.into_iter()
            .map(|row| self.row_to_advertisement(row))
            .collect();
        
        Ok((advertisements, total))
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Advertisement>, Box<dyn StdError>> {
        let query = "SELECT * FROM advertisements WHERE id = $1";
        let row = sqlx::query(query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
            
        Ok(row.map(|row| self.row_to_advertisement(row)))
    }

    async fn create(&self, ad: &Advertisement) -> Result<Advertisement, Box<dyn StdError>> {
        let query = "INSERT INTO advertisements
            (id, title, description, image_url, start_date, end_date, 
            status, click_url, position, impressions, clicks)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *";
        
        let row = sqlx::query(query)
            .bind(&ad.id)
            .bind(&ad.title)
            .bind(&ad.description)
            .bind(&ad.image_url)
            .bind(&ad.start_date)
            .bind(&ad.end_date)
            .bind(&ad.status.to_string())
            .bind(&ad.click_url)
            .bind(&ad.position)
            .bind(&ad.impressions)
            .bind(&ad.clicks)
            .fetch_one(&self.pool)
            .await?;
            
        Ok(self.row_to_advertisement(row))
    }

    async fn update(&self, ad: &Advertisement) -> Result<Advertisement, Box<dyn StdError>> {
        let query = "UPDATE advertisements SET
            title = $1, description = $2, image_url = $3, start_date = $4,
            end_date = $5, status = $6, click_url = $7, position = $8
            WHERE id = $9
            RETURNING *";
        
        let row = sqlx::query(query)
            .bind(&ad.title)
            .bind(&ad.description)
            .bind(&ad.image_url)
            .bind(&ad.start_date)
            .bind(&ad.end_date)
            .bind(&ad.status.to_string())
            .bind(&ad.click_url)
            .bind(&ad.position)
            .bind(&ad.id)
            .fetch_one(&self.pool)
            .await?;
            
        Ok(self.row_to_advertisement(row))
    }

    async fn delete(&self, id: &str) -> Result<bool, Box<dyn StdError>> {
        let result = sqlx::query("DELETE FROM advertisements WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
            
        Ok(result.rows_affected() > 0)
    }
    
    async fn increment_impression(&self, id: &str) -> Result<(), Box<dyn StdError>> {
        sqlx::query("UPDATE advertisements SET impressions = impressions + 1 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    async fn increment_click(&self, id: &str) -> Result<(), Box<dyn StdError>> {
        sqlx::query("UPDATE advertisements SET clicks = clicks + 1 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    async fn find_active(&self, limit: u32) -> Result<Vec<Advertisement>, Box<dyn StdError>> {
        let rows = sqlx::query("SELECT * FROM advertisements
            WHERE status = 'active'
            AND start_date <= $1
            AND (end_date IS NULL OR end_date >= $1)
            ORDER BY created_at DESC
            LIMIT $2")
            .bind(Utc::now())
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?;
            
        Ok(rows.into_iter().map(|row| self.row_to_advertisement(row)).collect())
    }
}
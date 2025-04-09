use rocket::serde::{Deserialize,Serialize};

use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum EventStatus {
    Published,
    Completed,
    Cancelled,
    Draft,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub event_date: NaiveDateTime,
    pub location: String,
    pub base_price: f64,
    pub status: EventStatus,
}

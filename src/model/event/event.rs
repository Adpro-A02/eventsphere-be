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


impl Event {
   
    pub fn new(
        title: String, 
        description: String, 
        event_date: NaiveDateTime, 
        location: String, 
        base_price: f64
    ) -> Self {
        Event {
            id: Uuid::new_v4(),
            title,
            description,
            event_date,
            location,
            base_price,
            status: EventStatus::Draft, 
        }
    }
    
    // Method untuk mengupdate properti event
    pub fn update(
        &mut self,
        title: Option<String>,
        description: Option<String>,
        event_date: Option<NaiveDateTime>,
        location: Option<String>,
        base_price: Option<f64>,
    ) {
        if let Some(title) = title {
            self.title = title;
        }
        
        if let Some(description) = description {
            self.description = description;
        }
        
        if let Some(event_date) = event_date {
            self.event_date = event_date;
        }
        
        if let Some(location) = location {
            self.location = location;
        }
        
        if let Some(base_price) = base_price {
            self.base_price = base_price;
        }
    }
    
    // Method untuk mengubah status event
    pub fn change_status(&mut self, new_status: EventStatus) {
        self.status = new_status;
    }
    
    // Method untuk mempublikasikan event
    pub fn publish(&mut self) -> Result<(), &'static str> {
        // Validasi: event harus memiliki title yang tidak kosong
        if self.title.is_empty() {
            return Err("Event title cannot be empty");
        }
        
        // Validasi: event harus memiliki tanggal yang valid (masa depan)
        let now = chrono::Local::now().naive_local();
        if self.event_date <= now {
            return Err("Event date must be in the future");
        }
        
        // Mengubah status menjadi Published
        self.status = EventStatus::Published;
        Ok(())
    }
    
    // Method untuk membatalkan event
    pub fn cancel(&mut self) -> Result<(), &'static str> {
        // Tidak bisa membatalkan event yang sudah completed
        if matches!(self.status, EventStatus::Completed) {
            return Err("Cannot cancel a completed event");
        }
        
        self.status = EventStatus::Cancelled;
        Ok(())
    }
    
    // Method untuk menandai event sebagai selesai
    pub fn complete(&mut self) -> Result<(), &'static str> {
        // Hanya event yang published yang bisa diubah menjadi completed
        if !matches!(self.status, EventStatus::Published) {
            return Err("Only published events can be marked as completed");
        }
        
        self.status = EventStatus::Completed;
        Ok(())
    }
    
    // Method untuk mengecek apakah harga tiket gratis / harga diabwah 0
    pub fn is_free(&self) -> bool {
        self.base_price == 0.0
    }
    pub fn is_err(&self) -> bool {
        self.base_price < 0.0
    }
}
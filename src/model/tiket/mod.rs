use uuid::Uuid;

#[derive(Debug, PartialEq, Clone)]
pub enum TicketStatus {
    AVAILABLE,
    SOLD_OUT,
    EXPIRED,
}

#[derive(Debug, Clone)]
pub struct Ticket {
    pub id: Option<Uuid>,
    pub event_id: Uuid,
    pub ticket_type: String,
    pub price: f64,
    pub quota: u32,
    pub status: TicketStatus,
}

impl Ticket {
    pub fn new(event_id: Uuid, ticket_type: String, price: f64, quota: u32) -> Self {
        Ticket {
            id: None,
            event_id,
            ticket_type,
            price,
            quota,
            status: TicketStatus::AVAILABLE,
        }
    }

    pub fn update_quota(&mut self, new_quota: u32) {
        self.quota = new_quota;
        
        if self.quota == 0 {
            self.status = TicketStatus::SOLD_OUT;
        }
    }

    pub fn mark_as_expired(&mut self) {
        self.status = TicketStatus::EXPIRED;
    }

    pub fn update_price(&mut self, new_price: f64) {
        self.price = new_price;
    }

    pub fn is_available(&self, quantity: u32) -> bool {
        self.status == TicketStatus::AVAILABLE && self.quota >= quantity
    }
}

#[cfg(test)]
pub mod tests;

use crate::model::review::Review;

pub struct NotificationService;

impl NotificationService {
    pub fn new() -> Self {
        NotificationService
    }

    pub fn notify_created(&self, review: &Review) -> Result<(), String> {
        println!("Review created for event {}: {:?}", review.event_id, review);
        Ok(())
    }

    pub fn notify_updated(&self, review: &Review) -> Result<(), String> {
        println!("Review updated for event {}: {:?}", review.event_id, review);
        Ok(())
    }

    pub fn notify_deleted(&self, review: &Review) -> Result<(), String> {
        println!("Review deleted for event {}: {:?}", review.event_id, review);
        Ok(())
    }

    pub fn notify_approved(&self, review: &Review) -> Result<(), String> {
        println!("Review approved for event {}: {:?}", review.event_id, review);
        Ok(())
    }

    pub fn notify_rejected(&self, review: &Review) -> Result<(), String> {
        println!("Review rejected for event {}: {:?}", review.event_id, review);
        Ok(())
    }
}

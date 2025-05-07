use crate::models::review::Review;

pub struct NotificationService;

impl NotificationService {
    pub fn new() -> Self {
        NotificationService
    }

    pub fn send_review_created_notification(&self, review: &Review) {
        println!("Notification: A new review with ID {} has been created.", review.id);
    }

    pub fn send_review_approved_notification(&self, review: &Review) {
        println!("Notification: Review with ID {} has been approved.", review.id);
    }

    pub fn send_review_rejected_notification(&self, review: &Review) {
        println!("Notification: Review with ID {} has been rejected.", review.id);
    }
}

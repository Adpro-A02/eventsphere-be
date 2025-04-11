#[cfg(test)]
mod tests {
    use chrono::{NaiveDateTime, Local, Duration};
    use crate::model::event::{Event,EventStatus};
    use crate::service::event::event_service::ServiceError;
    use crate::service::event::EventService;
    
    use super::*;
    use mockall::predicate::*;
    use mockall::*;
    use tokio::runtime::Runtime;
    use uuid::Uuid;

    // Mock Repository
    mock! {
        pub EvenetServiceMock{}
      
        impl EventService for EvenetServiceMock{
            fn create_event(&self, event: Event) -> Result<Event, ServiceError>;
            fn get_event(&self, event_id: &str) -> Result<Event, ServiceError>;
            fn update_event(&self, event_id: &str, event: Event) -> Result<Event, ServiceError>;
            fn delete_event(&self, event_id: &str) -> Result<(), ServiceError>;
        
        }
    }

    // Test helper untuk membuat event dummy
    fn create_test_event(id: Option<Uuid>) -> Event {
        let future_date = Local::now().naive_local() + Duration::days(30);
        Event {

            id: Uuid::new_v4(),
            title: String::from("Test Event"),
            description: String::from("Description for test event"),
            event_date: future_date,
            location: String::from("Test Location"),
            base_price: 100.0,
            status: EventStatus::Draft,
        }
    }

    // Helper untuk membuat service dengan mock repo
    fn setup_mock_service() -> MockEvenetServiceMock{
        MockEvenetServiceMock::new()
    }

    #[test]
    fn test_create_event_success() {
        let mut mock_service = setup_mock_service();
        let test_event = create_test_event(None);

        // Setup mock expectation
        mock_service.expect_create_event()
            .with(eq(test_event.clone()))
            .returning({
                let test_event = test_event.clone();
                move |_| Ok(test_event.clone())
            });

        // Execute
        let result = mock_service.create_event(test_event.clone());

        // Verify
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.title, "Test Event");
    }

    #[test]
    fn test_create_event_error() {
        let mut mock_service = setup_mock_service();
        let test_event = create_test_event(None);

        // Setup mock to return error
        mock_service.expect_create_event()
            .return_once(|_| Err(ServiceError::RepositoryError("DB error".to_string())));

        let result = mock_service.create_event(test_event);

        // Verify
        assert!(matches!(result, Err(ServiceError::RepositoryError(_))));
    }
    #[test]
    fn test_get_event_success() {
        let mut mock_service = setup_mock_service();
        let test_event = create_test_event(None);

        // Setup mock
        let cloned_event = test_event.clone();
        mock_service.expect_get_event()
            .with(eq(test_event.id.to_string()))
            .return_once(move |_| Ok(cloned_event));

        // Execute
        let result = mock_service.get_event(&test_event.id.to_string());

        // Verify
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, test_event.id);
    }
    #[test]
    fn test_get_event_not_found() {
        let mut mock_service = setup_mock_service();
        let test_id = Uuid::new_v4();

        // Setup mock to return not found
        mock_service.expect_get_event()
            .return_once(|_| Err(ServiceError::NotFound));

        let result = mock_service.get_event(&test_id.to_string());

        // Verify
        assert!(matches!(result, Err(ServiceError::NotFound)));
    }

    #[test]
    fn test_update_event_success() {
        let mut mock_service = setup_mock_service();
        let test_event = create_test_event(None);
        let updated_event = Event {
            title: "Updated Title".to_string(),
            ..test_event.clone()
        };

        // Setup mock
        mock_service.expect_update_event()
            .with(
                eq(test_event.id.to_string()),
                eq(updated_event.clone())
            )
            .return_once({
                let updated_event = updated_event.clone();
                move |_, _| Ok(updated_event)
            });

        // Execute
        let result = mock_service.update_event(
            &test_event.id.to_string(),
            updated_event.clone(),
        );

        // Verify
        assert!(result.is_ok());
        assert_eq!(result.unwrap().title, "Updated Title");
    }

    #[test]
    fn test_delete_event_success() {
        let mut mock_service = setup_mock_service();
        let test_id = Uuid::new_v4();

        // Setup mock
        mock_service.expect_delete_event()
            .with(eq(test_id.to_string()))
            .return_once(|_| Ok(()));

        // Execute
        let result = mock_service.delete_event(&test_id.to_string());

        // Verify
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_event_error() {
        let mut mock_service = setup_mock_service();
        let test_id = Uuid::new_v4();

        // Setup mock to return error
        mock_service.expect_delete_event()
            .return_once(|_| Err(ServiceError::RepositoryError("DB error".to_string())));

        let result = mock_service.delete_event(&test_id.to_string());

        assert!(matches!(result, Err(ServiceError::RepositoryError(_))));
    }

    
    #[test]
    fn test_invalid_uuid_format() {
        let mut mock_service = setup_mock_service();

        // Mock tidak diharapkan dipanggil karena error validasi terlebih dahulu
        mock_service.expect_get_event()
            .never();

        let result = mock_service.get_event("invalid-uuid");

        assert!(matches!(result, Err(ServiceError::InvalidInput(_))));
    }

}
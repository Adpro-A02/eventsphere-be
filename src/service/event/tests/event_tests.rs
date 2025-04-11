#[cfg(test)]
mod tests {
    use crate::model::event::{Event, EventStatus};
    use crate::service::event::event_service::{EventService,ServiceError};
   
    use mockall::{mock, predicate::*};
  
    use uuid::Uuid;
    use chrono::{Duration, Local};

   
    mock! {
        pub EventRepo {}

        
        impl EventService for EventRepo {
            fn create_event(&self, event: Event) -> Result<Event, ServiceError>;
            fn list_events(&self) -> Result<Vec<Event>, ServiceError>;
            fn get_event(&self, event_id: &str) -> Result<Event, ServiceError>;
            fn update_event(&self, event_id: &str, event: Event) -> Result<Event, ServiceError>;
            fn delete_event(&self, event_id: &str) -> Result<(), ServiceError>;
           
        }
    }


    fn handle_list_events(service: &dyn EventService) -> Vec<String> {
        match service.list_events() {
            Ok(events) => events.into_iter().map(|e| e.title).collect(),
            Err(_) => vec![],
        }
    }

    fn sample_event(id: &str, title: &str) -> Event {
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

    #[test]
    fn test_handle_list_events_success() {
        let mut mock_service = MockEventRepo::new();

        mock_service
            .expect_list_events()
            .returning(|| Ok(vec![
                sample_event("1", "Event 1"),
                sample_event("2", "Event 2"),
            ]));

        let names = handle_list_events(&mock_service);

        assert_eq!(names, vec!["Event 1", "Event 2"]);
    }

    #[test]
    fn test_handle_list_events_error() {
        let mut mock_service = MockEventRepo::new();

        mock_service
            .expect_list_events()
            .returning(|| Err(ServiceError::RepositoryError("DB error".to_string())));

        let names = handle_list_events(&mock_service);

        assert_eq!(names.len(), 0);
    }
}

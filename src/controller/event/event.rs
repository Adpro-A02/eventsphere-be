use rocket::{get, post, put, delete};
use rocket::serde::json::Json;
use crate::models::event::Event;

#[get("/events")]
pub fn get_events() -> Json<Vec<Event>> {
    
}

#[post("/events", format = "json", data = "<new_event>")]
pub fn create_event(new_event: Json<Event>) -> Json<Event> {
   
}

#[put("/events/<id>", format = "json", data = "<updated_event>")]
pub fn update_event(id: String, updated_event: Json<Event>) -> Json<Event> {
   
}

#[delete("/events/<id>")]
pub fn delete_event(id: String) -> Json<String> {
  
}

use serde_json::Value;

pub fn load_mock_response(action: &str) -> Option<Value> {
    match action {
        "search" => Some(serde_json::from_str(include_str!("response.search.json")).unwrap()),
        "select" => Some(serde_json::from_str(include_str!("response.select.json")).unwrap()),
        "init" => Some(serde_json::from_str(include_str!("response.init.json")).unwrap()),
        "confirm" => Some(serde_json::from_str(include_str!("response.confirm.json")).unwrap()),
        "status" => Some(serde_json::from_str(include_str!("response.status.json")).unwrap()),
        // ... add other actions
        _ => None,
    }
}

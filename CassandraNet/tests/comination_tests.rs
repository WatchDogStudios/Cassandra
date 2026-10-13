use cnbackend::requests::request::{CnCustomRequest, CnRequest, Method};

// TODO: Add integration tests for UGC and Messaging endpoints.
// Requires running gateway and reqwest dependency.
//
// Tests to add:
// 1. Create content: POST /ugc/content
// 2. List content: GET /ugc/content
// 3. Update content state: PUT /ugc/content/:id/state
// 4. Publish message: POST /messaging/publish
// 5. List messages: GET /messaging/messages
// 6. Ack message: POST /messaging/messages/:id/ack

#[test]
fn test_requests() {
    let mut headers = std::collections::HashMap::new();
    headers.insert("Host".to_string(), "localhost".to_string());
    headers.insert("User-Agent".to_string(), "curl/7.68.0".to_string());
    headers.insert("Accept".to_string(), "*/*".to_string());

    let request = CnRequest::new(0, Method::Get, Some("/".to_string()), headers);

    assert_eq!(request.request_id, 0);
    assert_eq!(request.request_type, Method::Get);
    assert_eq!(request.request_data, Some("/".to_string()));
    assert_eq!(request.headers.len(), 3);

    let mut headers = std::collections::HashMap::new();
    headers.insert("Host".to_string(), "localhost".to_string());
    headers.insert("User-Agent".to_string(), "curl/7.68.0".to_string());
    headers.insert("Accept".to_string(), "*/*".to_string());

    let mut request = CnRequest::new(0, Method::Get, Some("/".to_string()), headers);

    request.register_custom_request(Method::Get);
}

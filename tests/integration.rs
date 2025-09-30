use tui_chat::{ChatArea, ChatMessage};

#[test]
fn test_chat_area_add_message() {
    let mut chat_area = ChatArea::new();
    let message = ChatMessage {
        sender: "Test".to_string(),
        content: "Hello World".to_string(),
    };
    chat_area.add_message(message);
    // Basic check that it doesn't panic
    assert!(true);
}
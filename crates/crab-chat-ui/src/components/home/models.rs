#[derive(Debug, Default)]
pub struct Room {
    name: String,
    chat: Chat,
}

#[derive(Debug, Default)]
pub struct Chat {
    messages: Vec<String>,
    input: String,
}

#[async_trait::async_trait]
pub trait EventEmitter<U, T> {
    async fn emit_event(mut client: T, event: U) -> std::result::Result<(), Box<dyn std::error::Error>>;
}
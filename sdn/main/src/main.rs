use rtnetlink::new_connection;

#[tokio::main]
async fn create_veth() -> Result<(), String> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);
    handle
        .link()
        .add()
        .veth("veth-rs-1".into(), "veth-rs-2".into())
        .execute()
        .await
        .map_err(|e| format!("{}", e))
}

fn main() -> Result<(), String> {
    let _ = create_veth();
    Ok(())
}

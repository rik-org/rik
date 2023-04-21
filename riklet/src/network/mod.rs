pub mod net;
pub mod tap;

pub mod netutils {
    use futures_util::TryStreamExt;
    use rtnetlink::new_connection;
    use tracing::{trace, warn};

    #[tracing::instrument()]
    pub async fn set_link_up(iface_name: String) -> Result<(), rtnetlink::Error> {
        trace!("link {} up", &iface_name);
        let (connection, handle, _) = new_connection().unwrap();
        tokio::spawn(connection);

        let mut links = handle.link().get().match_name(iface_name.clone()).execute();
        if let Some(link) = links.try_next().await? {
            handle.link().set(link.header.index).up().execute().await?;

            return Ok(());
        }

        warn!("Could not get the interface {}", iface_name);
        return Err(rtnetlink::Error::RequestFailed);
    }
}

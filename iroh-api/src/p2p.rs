use anyhow::Result;
use async_trait::async_trait;
use iroh_rpc_client::P2pClient;
use libp2p::{Multiaddr, PeerId};
#[cfg(feature = "testing")]
use mockall::automock;

pub struct ClientP2p {
    client: P2pClient,
}

pub struct Lookup {
    pub peer_id: PeerId,
    pub listen_addrs: Vec<Multiaddr>,
    pub local_addrs: Vec<Multiaddr>,
}

#[derive(Debug, Clone)]
pub enum PeerIdOrAddr {
    PeerId(PeerId),
    Multiaddr(Multiaddr),
}

impl ClientP2p {
    pub fn new(client: P2pClient) -> Self {
        Self { client }
    }
}

#[cfg_attr(feature = "testing", automock)]
#[async_trait]
pub trait P2p: Sync {
    async fn lookup(&self, addr: &PeerIdOrAddr) -> Result<Lookup>;
}

#[async_trait]
impl P2p for ClientP2p {
    /// XXX really should be an API that intos a peer id, and then also accepts
    /// an address, or two separate methods, one for peer id, one for address
    async fn lookup(&self, _addr: &PeerIdOrAddr) -> Result<Lookup> {
        let (_, listen_addrs) = self.client.get_listening_addrs().await?;
        Ok(Lookup {
            peer_id: self.client.local_peer_id().await?,
            listen_addrs,
            local_addrs: self.client.external_addresses().await?,
        })
    }
}

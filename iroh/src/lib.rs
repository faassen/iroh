mod api;
mod config;
mod getadd;
mod p2p;

#[cfg(feature = "testing")]
pub use crate::api::MockApi;
pub use crate::api::{Api, Iroh};
#[cfg(feature = "testing")]
pub use crate::p2p::MockP2p;
pub use crate::p2p::P2p as P2pApi;
pub use crate::p2p::{Lookup, PeerIdOrAddr};
pub use bytes::Bytes;
pub use cid::Cid;
pub use iroh_resolver::resolver::{CidOrDomain, Path as IpfsPath};
pub use iroh_rpc_client::{ServiceStatus, StatusRow, StatusTable};
pub use libp2p::gossipsub::MessageId;
pub use libp2p::{Multiaddr, PeerId};

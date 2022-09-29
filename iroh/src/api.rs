use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::rc::Rc;

use crate::getadd::{add, get};
use crate::p2p::{ClientP2p, P2p};
use crate::store::{ClientStore, Store};
use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use iroh_resolver::resolver::Path as IpfsPath;
use iroh_rpc_client::{Client, P2pClient, StoreClient};
use libp2p::gossipsub::{MessageId, TopicHash};
use libp2p::{Multiaddr, PeerId};
use mockall::automock;
use tokio::{fs::File, io::stdin, io::AsyncReadExt};

pub struct ClientApi<'a> {
    client: &'a Client,
}

#[automock]
#[async_trait(?Send)]
pub trait Api<P: P2p, S: Store> {
    fn p2p(&self) -> Result<P>;
    fn store(&self) -> Result<S>;
    async fn get<'a>(&self, ipfs_path: &IpfsPath, output: Option<&'a Path>) -> Result<()>;
    async fn add(&self, path: &Path, recursive: bool, no_wrap: bool) -> Result<Cid>;
}

impl<'a> ClientApi<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }
}

#[async_trait(?Send)]
impl<'a> Api<ClientP2p<'a>, ClientStore<'a>> for ClientApi<'a> {
    fn p2p(&self) -> Result<ClientP2p<'a>> {
        let p2p_client = self.client.try_p2p()?;
        Ok(ClientP2p::new(p2p_client))
    }

    fn store(&self) -> Result<ClientStore<'a>> {
        let store_client = self.client.try_store()?;
        Ok(ClientStore::new(store_client))
    }

    async fn get<'b>(&self, ipfs_path: &IpfsPath, output: Option<&'b Path>) -> Result<()> {
        get(self.client, ipfs_path, output).await
    }

    async fn add(&self, path: &Path, recursive: bool, no_wrap: bool) -> Result<Cid> {
        add(self.client, path, recursive, no_wrap).await
    }
}

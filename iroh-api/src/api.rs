use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::{Config, CONFIG_FILE_NAME, ENV_PREFIX};
#[cfg(feature = "testing")]
use crate::p2p::MockP2p;
use crate::p2p::{ClientP2p, P2p};
use anyhow::Result;
use cid::Cid;
use futures::future::{BoxFuture, LocalBoxFuture};
use futures::stream::LocalBoxStream;
use futures::FutureExt;
use futures::StreamExt;
use iroh_resolver::resolver::Path as IpfsPath;
use iroh_resolver::{resolver, unixfs_builder};
use iroh_rpc_client::Client;
use iroh_rpc_client::StatusTable;
use iroh_util::{iroh_config_path, make_config};
#[cfg(feature = "testing")]
use mockall::automock;

pub struct Iroh {
    client: Client,
}

pub enum OutType<T: resolver::ContentLoader> {
    Dir,
    Reader(resolver::OutPrettyReader<T>),
}

#[cfg_attr(feature= "testing", automock(type P = MockP2p;))]
pub trait Api {
    type P: P2p;

    fn p2p(&self) -> Result<Self::P>;

    fn get<'a>(
        &'a self,
        ipfs_path: &'a IpfsPath,
        output: Option<&'a Path>,
    ) -> BoxFuture<'_, Result<()>>;
    fn add<'a>(
        &'a self,
        path: &'a Path,
        recursive: bool,
        no_wrap: bool,
    ) -> LocalBoxFuture<'_, Result<Cid>>;
    fn check(&self) -> BoxFuture<'_, StatusTable>;
    fn watch(&self) -> LocalBoxFuture<'static, LocalBoxStream<'static, StatusTable>>;
}

impl Iroh {
    pub async fn new(
        config_path: Option<&Path>,
        overrides_map: HashMap<String, String>,
    ) -> Result<Self> {
        let cfg_path = iroh_config_path(CONFIG_FILE_NAME)?;
        let sources = vec![Some(cfg_path), config_path.map(PathBuf::from)];
        let config = make_config(
            // default
            Config::default(),
            // potential config files
            sources,
            // env var prefix for this config
            ENV_PREFIX,
            // map of present command line arguments
            overrides_map,
        )
        .unwrap();

        let client = Client::new(config.rpc_client).await?;

        Ok(Iroh::from_client(client))
    }

    fn from_client(client: Client) -> Self {
        Self { client }
    }

    pub(crate) fn get_client(&self) -> &Client {
        &self.client
    }
}

impl Api for Iroh {
    type P = ClientP2p;

    fn p2p(&self) -> Result<ClientP2p> {
        let p2p_client = self.client.try_p2p()?;
        Ok(ClientP2p::new(p2p_client.clone()))
    }

    fn get<'a>(
        &'a self,
        ipfs_path: &'a IpfsPath,
        output: Option<&'a Path>,
    ) -> BoxFuture<'_, Result<()>> {
        async move {
            let blocks = self.get_stream(ipfs_path, output);
            tokio::pin!(blocks);
            while let Some(block) = blocks.next().await {
                let (path, out) = block?;
                match out {
                    OutType::Dir => {
                        tokio::fs::create_dir_all(path).await?;
                    }
                    OutType::Reader(mut reader) => {
                        if let Some(parent) = path.parent() {
                            tokio::fs::create_dir_all(parent).await?;
                        }
                        let mut f = tokio::fs::File::create(path).await?;
                        tokio::io::copy(&mut reader, &mut f).await?;
                    }
                }
            }
            Ok(())
        }
        .boxed()
    }

    fn add<'a>(
        &'a self,
        path: &'a Path,
        recursive: bool,
        no_wrap: bool,
    ) -> LocalBoxFuture<'_, Result<Cid>> {
        async move {
            let providing_client = iroh_resolver::unixfs_builder::StoreAndProvideClient {
                client: Box::new(self.get_client()),
            };
            if path.is_dir() {
                unixfs_builder::add_dir(Some(&providing_client), path, !no_wrap, recursive).await
            } else if path.is_file() {
                unixfs_builder::add_file(Some(&providing_client), path, !no_wrap).await
            } else {
                anyhow::bail!("can only add files or directories");
            }
        }
        .boxed_local()
    }

    fn check(&self) -> BoxFuture<'_, StatusTable> {
        async { self.client.check().await }.boxed()
    }

    fn watch(
        &self,
    ) -> LocalBoxFuture<'static, LocalBoxStream<'static, iroh_rpc_client::StatusTable>> {
        let client = self.client.clone();
        async { client.watch().await.boxed_local() }.boxed_local()
    }
}

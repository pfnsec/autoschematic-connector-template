use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use autoschematic_core::{
    connector::{Connector, ConnectorOutbox, FilterResponse, GetResourceResponse, OpExecResponse, PlanResponseElement},
    diag::DiagnosticResponse,
};

/// This template starts as one file on purpose so the overall control flow is
/// easy to follow before you break it apart.
pub struct DummyConnector {
    /// Connector methods receive repo-relative addresses like
    /// `aws/s3/us-east-1/buckets/main.ron`, not `{prefix}/aws/...`, so you might store the
    /// prefix here for E.G. loading config files later.
    prefix: PathBuf,
}

#[async_trait]
impl Connector for DummyConnector {
    async fn new(_name: &str, prefix: &Path, _outbox: ConnectorOutbox) -> Result<Arc<dyn Connector>, anyhow::Error>
    where
        Self: Sized,
    {
        // In new(), you'll create and return a default instance with whatever
        // empty structures the connector needs.
        // Even if the configuration is invalid, new() should never fail unless something is seriously messed up.
        // Loading and validating configuration is for init(), not new()! If new() fails, the connector
        // won't even be alive to tell the client that the configuration is wrong.
        // You can ignore `name` or interpret it, not unlike argv[0]. It's the `shortname` as specified in autoschematic.ron .
        Ok(Arc::new(DummyConnector { prefix: prefix.into() }))
    }

    async fn init(&self) -> Result<(), anyhow::Error> {
        // In init(), you'll load and validate prefix-wide configuration files if needed,
        // for example, the AWS S3 Connector (autoschematic-connector-aws-s3) loads ./${prefix}/aws/config.ron
        // in addition to ./${prefix}/aws/s3/config.ron in order to load AWS and S3-specific config params.
        // The Kubernetes Connector (autoschematic-connector-k8s) loads ./${prefix}/k8s/config.yaml to find its
        // kubeconfig(s) and set max concurrency per cluster connection, for instance.
        //
        // If you support config files, remember that `filter()` must return
        // `FilterResponse::Config` for those paths so upstream caches are
        // invalidated when config changes.
        Ok(())
    }

    async fn filter(&self, addr: &Path) -> Result<FilterResponse, anyhow::Error> {
        // In filter(), you'll define which files belong to your connector.
        // We'll use an example to explain what we mean.
        // For the SnowflakeConnector, here's how we respond to various addresses.
        // "snowflake/warehouses/data_team.sql" => FilterResponse::Resource
        // "snowflake/warehouses/dummy_file.txt" => FilterResponse::None
        // "snowflake/databases/customer_db/database.sql" => FilterResponse::Resource
        // "snowflake/databases/customer_db/primary/schema.sql" => FilterResponse::Resource
        // "snowflake/databases/customer_db/primary/tables/customer_orgs.sql" => FilterResponse::Resource
        // In other words, this address decoding logic is the main mechanism by which connectors describe an
        // ontology of nested objects.
        // Note that addresses never include the prefix here; connectors run with their working directory
        // at the root of the git repo, but they are informed of their prefix at new() and should save it if they need it (for loading configs etc).
        // In other words, if the full path is "./${prefix}/${addr}", connectors are only passed "./${addr}" in operations like filter, get, plan, etc.
        Ok(FilterResponse::None)
    }

    async fn list(&self, subpath: &Path) -> Result<Vec<PathBuf>, anyhow::Error> {
        // Scan remote infra and list the addresses of remote resources that currently exist.
        //
        // `subpath` is a hint for narrowing the query space. If your connector
        // can truly partition listing by region/account/cluster, implement
        // `subpaths()` as well. Otherwise, it is fine to ignore `subpath` and
        // list everything.
        // list() is a hefty operation and is only invoked during `autoschematic import`.
        let _ = subpath;
        Ok(Vec::new())
    }

    async fn get(&self, addr: &Path) -> Result<Option<GetResourceResponse>, anyhow::Error> {
        // Fetch one remote resource and serialize it into its on-disk representation (e.g. RON, JSON, YAML...)
        //
        // `get()` is called with a physical address and returns
        // `Ok(None)` when the remote object does not exist.
        //
        // Your serialized resource definition (E.G. RON, JSON bytes...)
        // goes into `GetResourceResponse.resource_definition`.
        //
        // If the API gives you stable IDs or other stored values, you may also
        // return them in the hashmap `GetResourceResponse.outputs` to store them.
        // The point of storing them may be to virt-phy or phy-virt address mapping.
        // See https://github.com/autoschematic-sh/autoschematic-connector-aws/tree/main/vpc
        //  for a reference implementation of `Connector::addr_virt_to_phy` and `Connector::addr_phy_to_virt` .
        let _ = addr;
        Ok(None)
    }

    async fn plan(
        &self,
        addr: &Path,
        current: Option<Vec<u8>>,
        desired: Option<Vec<u8>>,
    ) -> Result<Vec<PlanResponseElement>, anyhow::Error> {
        // Given the current state of a resource at addr (as returned by get(addr)) and the desired state (the file on disk at ./{prefix}/{addr}),
        // produce an ordered list of "connector ops".
        // Just like resource bodies, connector ops are also serialized when returned here, and deserialized within `Connector::op_exec` later on.
        //
        // Handle all four cases:
        // current -> desired
        // - `None -> Some`: create
        // - `Some -> None`: delete
        // - `Some -> Some`: update or no-op
        // - `None -> None`: no-op
        //
        // Your `ConnectorOp` definitions should only contain the changing data.
        // Do not duplicate fields that already live in `addr`.
        let _ = (addr, current, desired);
        Ok(Vec::new())
    }

    async fn op_exec(&self, addr: &Path, op: &str) -> Result<OpExecResponse, anyhow::Error> {
        // Parse `op` back into your `ConnectorOp` type and perform exactly one
        // mutation against the remote API.
        //
        // If execution discovers values Autoschematic should persist, such as a
        // remote ID created at runtime, return them in `outputs`. They will be
        // stored under `.autoschematic/{prefix}/{addr}.out.ron`.
        let _ = (addr, op);
        Ok(OpExecResponse {
            outputs: None,
            friendly_message: None,
        })
    }

    async fn eq(&self, addr: &Path, a: &[u8], b: &[u8]) -> Result<bool, anyhow::Error> {
        // Once you have typed resource bodies, compare parsed values here
        // instead of raw bytes if formatting differences should be ignored.
        let _ = addr;
        Ok(a == b)
    }

    async fn diag(&self, addr: &Path, a: &[u8]) -> Result<Option<DiagnosticResponse>, anyhow::Error> {
        // If you're using RON as your resource format, you can
        // ```
        // let parsed_addr = SnowflakeResourceAddress::from_path(addr)?;

        // match parsed_addr {
        //     // RON-based resources (users, roles)
        //     SnowflakeResourceAddress::User { .. } => ron_check_eq::<SnowflakeUser>(a, b),
        //     SnowflakeResourceAddress::Role { .. } => ron_check_eq::<SnowflakeRole>(a, b),
        //     // SQL-based resources (warehouses, databases, schemas, tables)
        //     _ => Ok(a == b),
        // }
        // ```
        let _ = (addr, a);
        Ok(None)
    }
}

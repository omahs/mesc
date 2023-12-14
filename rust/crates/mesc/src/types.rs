use crate::validate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Endpoint {
    pub name: String,
    pub url: String,
    pub chain_id: Option<ChainId>,
    pub endpoint_metadata: HashMap<String, serde_json::Value>,
}

impl Endpoint {
    pub fn chain_id_string(&self) -> String {
        self.chain_id
            .clone()
            .map(|x| x.to_string())
            .unwrap_or("-".to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    pub default_endpoint: Option<String>,
    pub network_defaults: HashMap<ChainId, String>,
}

#[derive(Debug)]
pub enum ConfigMode {
    Path,
    Env,
    Disabled,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcConfig {
    pub mesc_version: String,
    pub default_endpoint: Option<String>,
    pub network_defaults: HashMap<ChainId, String>,
    pub endpoints: HashMap<String, Endpoint>,
    pub network_names: HashMap<String, ChainId>,
    pub profiles: HashMap<String, Profile>,
    pub global_metadata: HashMap<String, serde_json::Value>,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            mesc_version: env!("CARGO_PKG_VERSION").to_string(),
            default_endpoint: None,
            network_defaults: HashMap::new(),
            endpoints: HashMap::new(),
            network_names: HashMap::new(),
            profiles: HashMap::new(),
            global_metadata: HashMap::new(),
        }
    }
}

impl RpcConfig {
    pub fn serialize(&self) -> Result<String, MescError> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn validate(&self) -> Result<(), MescError> {
        validate::validate_config(self)
    }
}

#[derive(Debug)]
pub enum MescError {
    MescNotEnabled,
    InvalidConfigMode,
    InvalidChainId(String),
    IntegrityError(String),
    MissingEndpoint(String),
    IOError(std::io::Error),
    InvalidJson,
    EnvReadError(std::env::VarError),
    NotImplemented(String),
    SerdeError(serde_json::Error),
    InvalidInput,
}

impl From<std::io::Error> for MescError {
    fn from(value: std::io::Error) -> MescError {
        MescError::IOError(value)
    }
}

impl From<serde_json::Error> for MescError {
    fn from(value: serde_json::Error) -> MescError {
        MescError::SerdeError(value)
    }
}

impl From<std::env::VarError> for MescError {
    fn from(value: std::env::VarError) -> MescError {
        MescError::EnvReadError(value)
    }
}

/// ChainId is a string representation of an integer chain id
/// - TryFrom conversions allow specifying as String, &str, uint, or binary data
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct ChainId(String);

impl ChainId {
    pub fn null_chain_id() -> ChainId {
        ChainId("0".to_string())
    }
}

impl PartialOrd for ChainId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ChainId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let ChainId(self_str) = self;
        let ChainId(other_str) = other;
        let self_str = format!("{:>079}", self_str);
        let other_str = format!("{:>079}", other_str);
        self_str.cmp(&other_str)
    }
}

impl std::fmt::Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

macro_rules! impl_from_uint_for_chainid {
    ($($t:ty),*) => {
        $(
            impl From<$t> for ChainId {
                fn from(value: $t) -> ChainId {
                    ChainId(value.to_string())
                }
            }
        )*
    };
}

impl_from_uint_for_chainid!(u8, u16, u32, u64, u128, usize);

/// use custom trait instead of TryInto so that Error type is always the same
pub trait TryIntoChainId {
    fn try_into_chain_id(self) -> Result<ChainId, MescError>;
}

impl TryIntoChainId for ChainId {
    fn try_into_chain_id(self) -> Result<ChainId, MescError> {
        Ok(self)
    }
}

impl TryIntoChainId for String {
    fn try_into_chain_id(self) -> Result<ChainId, MescError> {
        if self.chars().all(|c| c.is_ascii_digit()) {
            Ok(ChainId(self))
        } else {
            Err(MescError::InvalidChainId(self))
        }
    }
}

impl TryIntoChainId for &str {
    fn try_into_chain_id(self) -> Result<ChainId, MescError> {
        if self.chars().all(|c| c.is_ascii_digit()) {
            Ok(ChainId(self.to_string()))
        } else {
            Err(MescError::InvalidChainId(self.to_string()))
        }
    }
}

macro_rules! impl_try_into_chain_id_for_integer {
    ($($t:ty),*) => {
        $(
            impl TryIntoChainId for $t {
                fn try_into_chain_id(self) -> Result<ChainId, MescError> {
                    Ok(ChainId(self.to_string()))
                }
            }
        )*
    };
}

impl_try_into_chain_id_for_integer!(u8, u16, u32, u64, u128, usize);

impl TryIntoChainId for &[u8] {
    fn try_into_chain_id(self) -> Result<ChainId, MescError> {
        Err(MescError::NotImplemented("binary chain_id".to_string()))
    }
}

#[derive(Debug, Default, Clone)]
pub struct EndpointQuery {
    pub chain_id: Option<ChainId>,
    pub name_contains: Option<String>,
    pub url_contains: Option<String>,
}

impl EndpointQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn chain_id<T: TryIntoChainId>(mut self, chain_id: T) -> Result<Self, MescError> {
        self.chain_id = Some(chain_id.try_into_chain_id()?);
        Ok(self)
    }

    pub fn name<T: AsRef<str>>(mut self, query: T) -> Result<Self, MescError> {
        self.name_contains = Some(query.as_ref().to_string());
        Ok(self)
    }

    pub fn url<T: AsRef<str>>(mut self, query: T) -> Result<Self, MescError> {
        self.url_contains = Some(query.as_ref().to_string());
        Ok(self)
    }
}
use serde::{Deserialize, Serialize};

use crate::provider::auth_profiles::{agent_broker, agent_sign_in};
use crate::runtime::plans::default_network_mode;
use crate::runtime::profiles::AgentProfile;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCatalogData {
    pub agents: Vec<AgentCatalogItemData>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCatalogItemData {
    pub name: String,
    pub description: String,
    pub image: String,
    pub sign_in: String,
    pub broker: String,
    pub default_network: String,
    pub provider_host_count: usize,
}

impl AgentCatalogData {
    pub fn from_profiles(profiles: impl IntoIterator<Item = AgentProfile>) -> Self {
        Self {
            agents: profiles
                .into_iter()
                .map(|profile| AgentCatalogItemData::from_profile(&profile))
                .collect(),
        }
    }
}

impl AgentCatalogItemData {
    pub fn from_profile(profile: &AgentProfile) -> Self {
        Self {
            name: profile.name.to_string(),
            description: profile.description.to_string(),
            image: profile.image.to_string(),
            sign_in: agent_sign_in(profile.name).to_string(),
            broker: agent_broker(profile.name).to_string(),
            default_network: default_network_mode(profile).as_str().to_string(),
            provider_host_count: profile.provider_hosts.len(),
        }
    }
}

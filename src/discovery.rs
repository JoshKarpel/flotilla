use std::{collections::HashMap, rc::Rc};

use http::Request;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::APIResource;
use kube::{
    api::ApiResource,
    core::{gvk::ParseGroupVersionError, GroupVersion},
    Client,
};

use crate::DynResult;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct DiscoveredAPIResource {
    /// Resource group, empty for core group.
    pub group: String,
    /// group version
    pub version: String,
    /// apiVersion of the resource (v1 for core group,
    /// groupName/groupVersions for other).
    pub api_version: String,
    /// Singular PascalCase name of the resource
    pub kind: String,
    /// Plural name of the resource
    pub plural: String,
    /// Singular name of the resource
    pub singular: String,
    /// Short names for the resource
    pub short_names: Option<Vec<String>>,
    /// Verbs that can be applied to this resource
    pub verbs: Vec<String>,
    /// Whether the resource is namespaced or not
    pub namespaced: bool,
}

impl DiscoveredAPIResource {
    fn parse_api_resource(
        api_resource: &APIResource,
        group_version: &str,
    ) -> Result<Self, ParseGroupVersionError> {
        let gv: GroupVersion = group_version.parse()?;
        Ok(Self {
            group: api_resource
                .group
                .clone()
                .unwrap_or_else(|| gv.group.clone()),
            version: api_resource
                .version
                .clone()
                .unwrap_or_else(|| gv.version.clone()),
            api_version: gv.api_version(),
            kind: api_resource.kind.to_string(),
            plural: api_resource.name.clone(),
            singular: api_resource.singular_name.clone(),
            short_names: api_resource.short_names.clone(),
            verbs: api_resource.verbs.clone(),
            namespaced: api_resource.namespaced,
        })
    }

    pub fn url_path(&self, namespace: Option<&str>) -> String {
        let n = if let Some(ns) = namespace {
            format!("namespaces/{ns}/")
        } else {
            "".into()
        };
        format!(
            "/{group}/{api_version}/{namespaces}{plural}",
            group = if self.group.is_empty() { "api" } else { "apis" },
            api_version = self.api_version,
            namespaces = n,
            plural = self.plural,
        )
    }

    pub fn table_request(&self, namespace: Option<&str>) -> Request<Vec<u8>> {
        Request::builder()
            .uri(self.url_path(namespace))
            .header("Accept", "application/json;as=Table;g=meta.k8s.io;v=v1")
            .body(vec![])
            .unwrap()
    }
}

impl From<&DiscoveredAPIResource> for ApiResource {
    fn from(value: &DiscoveredAPIResource) -> Self {
        Self {
            group: value.group.clone(),
            version: value.version.clone(),
            api_version: value.api_version.clone(),
            kind: value.kind.clone(),
            plural: value.plural.clone(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Discovery {
    pub(crate) name_to_resource: HashMap<String, Rc<DiscoveredAPIResource>>,
}

impl Discovery {
    pub(crate) async fn discover(client: &Client) -> DynResult<Self> {
        // https://github.com/kube-rs/kube/blob/d28a7152538c2560f7af9b7339c090c7ccba9fb6/kube-client/src/discovery/mod.rs#L111-L130
        let mut name_to_resource: HashMap<String, Rc<DiscoveredAPIResource>> = HashMap::new();

        // Discover non-core first so that names for core resources override these names.
        let api_groups = client.list_api_groups().await?;
        for g in api_groups.groups {
            let ver = g
                .preferred_version
                .as_ref()
                .or_else(|| g.versions.first())
                .expect("preferred or versions exists");
            let apis = client.list_api_group_resources(&ver.group_version).await?;
            for api in apis.resources {
                if !api.verbs.iter().any(|v| v == "list") {
                    continue;
                }
                let discovered =
                    DiscoveredAPIResource::parse_api_resource(&api, &ver.group_version)?;
                let a = Rc::new(discovered);

                if !a.singular.is_empty() {
                    name_to_resource.insert(a.singular.clone(), a.clone());
                }
                if !a.plural.is_empty() {
                    name_to_resource.insert(a.plural.clone(), a.clone());
                }
                for name in a.short_names.as_deref().unwrap_or_default() {
                    name_to_resource.insert(name.clone(), a.clone());
                }
            }
        }

        let core_api_groups = client.list_core_api_versions().await?;
        for v in core_api_groups.versions {
            let apis = client.list_core_api_resources(&v).await?;
            for api in apis.resources {
                if !api.verbs.iter().any(|v| v == "list") {
                    continue;
                }
                let discovered = DiscoveredAPIResource::parse_api_resource(&api, &v)?;
                let a = Rc::new(discovered);

                if !a.singular.is_empty() {
                    name_to_resource.insert(a.singular.clone(), a.clone());
                }
                if !a.plural.is_empty() {
                    name_to_resource.insert(a.plural.clone(), a.clone());
                }
                for name in a.short_names.as_deref().unwrap_or_default() {
                    name_to_resource.insert(name.clone(), a.clone());
                }
            }
        }

        Ok(Self { name_to_resource })
    }

    pub(crate) fn get(&self, name: &str) -> Option<&Rc<DiscoveredAPIResource>> {
        self.name_to_resource.get(name)
    }
}

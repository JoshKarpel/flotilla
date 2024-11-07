use std::{collections::HashMap, rc::Rc};

use k8s_openapi::apimachinery::pkg::apis::meta::v1::APIResource;
use kube::Client;

use crate::DynResult;

pub async fn discover_api_resources() -> DynResult<HashMap<String, Rc<APIResource>>> {
    // https://github.com/kube-rs/kube/blob/d28a7152538c2560f7af9b7339c090c7ccba9fb6/kube-client/src/discovery/mod.rs#L111-L130
    let mut resources: HashMap<String, Rc<APIResource>> = HashMap::new();
    let client = Client::try_default().await?;

    let apigroups = client.list_core_api_versions().await?;
    for v in apigroups.versions {
        let apis = client.list_core_api_resources(&v).await?;
        for api in apis.resources {
            if !api.verbs.iter().any(|v| v == "list") {
                continue;
            }
            let a = Rc::new(api);

            if !a.singular_name.is_empty() {
                resources.insert(a.singular_name.clone(), a.clone());
            }
            if !a.name.is_empty() {
                resources.insert(a.name.clone(), a.clone());
            }
            for name in a.short_names.as_deref().unwrap_or_default() {
                resources.insert(name.clone(), a.clone());
            }
        }
    }

    let apigroups = client.list_api_groups().await?;
    for g in apigroups.groups {
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
            let a = Rc::new(api);

            if !a.singular_name.is_empty() {
                resources.insert(a.singular_name.clone(), a.clone());
            }
            if !a.name.is_empty() {
                resources.insert(a.name.clone(), a.clone());
            }
            for name in a.short_names.as_deref().unwrap_or_default() {
                resources.insert(name.clone(), a.clone());
            }
        }
    }
    Ok(resources)
}

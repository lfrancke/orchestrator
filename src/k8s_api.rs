use k8s_openapi::apimachinery::pkg::apis::meta::v1::{APIGroupList, APIGroup, GroupVersionForDiscovery, APIVersions, ServerAddressByClientCIDR, APIResourceList, APIResource};
use actix_web::{get, HttpResponse, Responder, web};
use crate::storage_sqlite::SqliteStorage;
use crate::storage::Storage;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use crate::helper::get_crd_resource_type;

// TODO: All APIs probably need to return a kind: Status / 404 instead of empty lists when something doesn't exist
// TODO: Do we also need /apis/{group} ? kubectl works without it but it'd be good to have for compatibility anyway



#[get("/api")]
pub async fn get_api_versions() -> impl Responder {
    // TODO: Server Address needs to be passed in via application state
    let api_versions = APIVersions {
        server_address_by_client_cidrs: vec![ServerAddressByClientCIDR { client_cidr: "0.0.0.0/0".to_string(), server_address: "127.0.0.1:8080".to_string() }],
        versions: vec!["v1".to_string()],
    };
    HttpResponse::Ok().json(api_versions)
}


// This lists all available API Groups and their versions
// It aims to be API compatible to Kubernetes.
#[get("/apis")]
pub async fn list_api_groups(
    storage: web::Data<SqliteStorage>,
) -> impl Responder {
    let crds: Vec<CustomResourceDefinition> = storage.list_cluster_resources(&get_crd_resource_type());

    let mut groups = Vec::with_capacity(crds.len());
    // Iterate over each API Group and for each group iterate over its versions to create the final document
    for crd in crds {
        let mut group_versions = Vec::with_capacity(crd.spec.versions.len());
        for version in crd.spec.versions {
            let group_version = format!("{}/{}", crd.spec.group, version.name);

            group_versions.push(GroupVersionForDiscovery {
                group_version,
                version: version.name,
            })
        }

        let group = APIGroup {
            name: crd.spec.group,
            preferred_version: Some(GroupVersionForDiscovery { group_version: group_versions.get(0).unwrap().group_version.clone(), version: group_versions.get(0).unwrap().version.clone() }),
            server_address_by_client_cidrs: None,
            versions: group_versions,
        };

        groups.push(group);
    }

    let api_group_list = APIGroupList {
        groups
    };
    HttpResponse::Ok().json(api_group_list)
}


#[get("/apis/{group}/{version}")]
pub async fn list_resource_types(
    web::Path((group, version)): web::Path<(String, String)>,
    storage: web::Data<SqliteStorage>,
) -> impl Responder {
    let crds: Vec<CustomResourceDefinition> = storage.list_cluster_resources(&get_crd_resource_type());

    let group_version = format!("{}/{}", group, version);
    let api_resources: Vec<APIResource> = crds.iter()
        .filter(|&crd| crd.spec.group == group)
        .filter(|&crd| crd.spec.versions.iter().any(|crd_version| crd_version.name == version))
        .map(|crd| APIResource {
            categories: crd.spec.names.categories.clone(),
            group: None, // Empty implies the group of the containing resource list.
            kind: crd.spec.names.kind.clone(),
            name: crd.spec.names.plural.clone(),
            namespaced: crd.spec.scope == "Namespaced",
            short_names: crd.spec.names.short_names.clone(),
            singular_name: crd.spec.names.singular.clone().unwrap_or("".to_string()),
            storage_version_hash: None,
            verbs: vec![
                "delete".to_string(),
                "deletecollection".to_string(),
                "get".to_string(),
                "list".to_string(),
                "patch".to_string(),
                "create".to_string(),
                "update".to_string(),
                "watch".to_string()
            ],
            version: None, // TODO
        })
        .collect();

    let resource_list = APIResourceList {
        group_version,
        resources: api_resources,
    };
    HttpResponse::Ok().json(resource_list)
}

use crate::models::{GroupResourceTypeResource, GroupResourceType};

const CRD_GROUP: &str = "apiextensions.k8s.io";
const CRD_RESOURCE_TYPE: &str = "customresourcedefinitions";

pub fn get_crd_resource_type() -> GroupResourceType {
    GroupResourceType::new(
        CRD_GROUP.to_string(),
        CRD_RESOURCE_TYPE.to_string(),
    )
}

pub fn get_crd_for_resource(resource: String) -> GroupResourceTypeResource {
    GroupResourceTypeResource::new(
        CRD_GROUP.to_string(),
        CRD_RESOURCE_TYPE.to_string(),
        resource
    )
}

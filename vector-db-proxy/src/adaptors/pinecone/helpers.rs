use crate::vector_databases::error::VectorDatabaseError;
use crate::vector_databases::models::{CollectionCreate, Distance, Region};
use pinecone_sdk::models::IndexModel;
use pinecone_sdk::pinecone::PineconeClient;
use std::sync::Arc;

pub async fn get_namespaces_for_index(
    client: &PineconeClient,
    index: &IndexModel,
) -> Option<Vec<String>> {
    let mut namespaces: Vec<String> = vec![];
    if let Ok(mut index) = client.index(index.name.as_str()).await {
        if let Ok(index_stats) = index.describe_index_stats(None).await {
            namespaces = index_stats
                .namespaces
                .iter()
                .map(|(k, _)| k.clone())
                .collect();
        }
    }
    Some(namespaces)
}

pub async fn get_indexes(client: &PineconeClient) -> Vec<String> {
    let mut list_of_indexes: Vec<String> = vec![];
    if let Ok(index_list) = client.list_indexes().await {
        if let Some(vec_of_indexes) = index_list.clone().indexes {
            list_of_indexes = vec_of_indexes.iter().map(|i| i.clone().name).collect();
        }
    };
    list_of_indexes
}

pub async fn check_index_exists(
    client: &PineconeClient,
    collection_create: CollectionCreate,
) -> Result<IndexModel, VectorDatabaseError> {
    let region = collection_create.clone().region.unwrap_or(Region::US);
    match client.describe_index(Region::to_str(region)).await {
        Ok(index_model) => {
            if (index_model.dimension == collection_create.dimensions as i32)
                && (Distance::from(index_model.clone().metric) == collection_create.distance)
            {
                Ok(index_model)
            } else {
                Err(VectorDatabaseError::NotFound(format!(
                    "Index: {} was not found",
                    collection_create.collection_name
                )))
            }
        }
        Err(e) => Err(VectorDatabaseError::PineconeError(Arc::new(e))),
    }
}

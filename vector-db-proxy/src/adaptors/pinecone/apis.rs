use crate::adaptors::pinecone::helpers::{check_index_exists, get_indexes};
use crate::vector_databases::error::VectorDatabaseError;
use crate::vector_databases::models::{
    CollectionCreate, CollectionMetadata, CollectionsResult, Distance, Point, Region,
    ScrollResults, SearchRequest, SearchResult, StorageSize, VectorDatabaseStatus,
};
use crate::vector_databases::utils::calculate_vector_storage_size;
use crate::vector_databases::vector_database::VectorDatabase;
use async_trait::async_trait;
use pinecone_sdk::models::Cloud as PineconeCloud;
use pinecone_sdk::models::{DeletionProtection, Metric, Namespace, Vector, WaitPolicy};
use pinecone_sdk::pinecone::PineconeClient;
use std::sync::Arc;

#[async_trait]
impl VectorDatabase for PineconeClient {
    async fn get_list_of_collections(&self) -> Result<Vec<String>, VectorDatabaseError> {
        let mut list_of_namespaces: Vec<String> = vec![];
        let list_of_indexes = get_indexes(self).await;
        for index in list_of_indexes {
            let mut index = self.index(index.as_str()).await.unwrap();
            let index_stats = index.describe_index_stats(None).await.unwrap();
            list_of_namespaces.extend(
                index_stats
                    .namespaces
                    .keys()
                    .cloned()
                    .collect::<Vec<String>>(),
            )
        }
        Ok(list_of_namespaces)
    }

    async fn check_collection_exists(
        &self,
        search_request: SearchRequest,
    ) -> Result<CollectionsResult, VectorDatabaseError> {
        let mut collection_results = CollectionsResult {
            status: VectorDatabaseStatus::NotFound,
            collection_name: search_request.collection.clone(),
            collection_metadata: None,
        };
        let region = search_request.clone().region.unwrap_or(Region::US);
        // Need to figure out the the default index name here
        let mut index = self.index(Region::to_str(region)).await.unwrap();
        let index_stats = index.describe_index_stats(None).await.unwrap();
        if index_stats
            .namespaces
            .contains_key(search_request.clone().collection.as_str())
        {
            let indexed_vectors_count = index_stats
                .namespaces
                .get(search_request.collection.as_str())
                .unwrap()
                .vector_count;
            collection_results.status = VectorDatabaseStatus::Ok;
            collection_results.collection_metadata = Some(CollectionMetadata {
                status: VectorDatabaseStatus::Ok,
                collection_vector_count: Some(indexed_vectors_count as u64),
                metric: None,
                dimensions: None,
            })
        }
        Ok(collection_results)
    }

    async fn create_collection(
        &self,
        collection_create: CollectionCreate,
    ) -> Result<VectorDatabaseStatus, VectorDatabaseError> {
        match check_index_exists(&self, collection_create.clone()).await {
            Ok(_) => Ok(VectorDatabaseStatus::Ok),
            Err(e) => match e {
                VectorDatabaseError::NotFound(_) => {
                    match self
                        .create_serverless_index(
                            Region::to_str(collection_create.region.unwrap()),
                            collection_create.dimensions as i32,
                            Metric::from(collection_create.distance),
                            PineconeCloud::from(collection_create.cloud.unwrap()),
                            Region::to_str(collection_create.region.unwrap()),
                            DeletionProtection::Disabled,
                            WaitPolicy::NoWait,
                        )
                        .await
                    {
                        Ok(_) => Ok(VectorDatabaseStatus::Ok),
                        Err(e) => Err(VectorDatabaseError::PineconeError(Arc::new(e))),
                    }
                }
                _ => Err(e),
            },
        }
    }

    async fn delete_collection(
        &self,
        search_request: SearchRequest,
    ) -> Result<VectorDatabaseStatus, VectorDatabaseError> {
        let region = search_request.clone().region.unwrap_or(Region::US);
        // Need to figure out the the default index name here
        let mut index = self.index(Region::to_str(region)).await.unwrap();
        let pinecone_namespace = Namespace::from(search_request.collection.as_str());
        match index.delete_all(&pinecone_namespace).await {
            Ok(_) => Ok(VectorDatabaseStatus::Ok),
            Err(e) => Ok(VectorDatabaseStatus::Error(
                VectorDatabaseError::PineconeError(Arc::new(e)),
            )),
        }
    }

    async fn insert_point(
        &self,
        search_request: SearchRequest,
        point: Point,
    ) -> Result<VectorDatabaseStatus, VectorDatabaseError> {
        let region = search_request.clone().region.unwrap_or(Region::US);
        // Need to figure out the the default index name here
        let mut index = self.index(Region::to_str(region)).await.unwrap();
        let vector = Vector::from(point);
        let namespace = search_request.collection;

        match index.upsert(&[vector], &namespace.into()).await {
            Ok(status) => {
                if status.upserted_count == 1 {
                    Ok(VectorDatabaseStatus::Ok)
                } else {
                    Ok(VectorDatabaseStatus::Failure)
                }
            }
            Err(e) => Err(VectorDatabaseError::PineconeError(Arc::new(e))),
        }
    }

    async fn bulk_insert_points(
        &self,
        search_request: SearchRequest,
        points: Vec<Point>,
    ) -> Result<VectorDatabaseStatus, VectorDatabaseError> {
        let region = search_request.clone().region.unwrap_or(Region::US);
        // Need to figure out the the default index name here
        let mut index = self.index(Region::to_str(region)).await.unwrap();
        let vectors: Vec<Vector> = points.iter().map(|p| Vector::from(p.to_owned())).collect();
        let namespace = search_request.collection;
        match index.upsert(&vectors, &namespace.into()).await {
            Ok(status) => {
                if status.upserted_count == 1 {
                    Ok(VectorDatabaseStatus::Ok)
                } else {
                    Ok(VectorDatabaseStatus::Failure)
                }
            }
            Err(e) => Err(VectorDatabaseError::PineconeError(Arc::new(e))),
        }
    }

    async fn get_collection_info(
        &self,
        search_request: SearchRequest,
    ) -> Result<Option<CollectionMetadata>, VectorDatabaseError> {
        let region = search_request.clone().region.unwrap_or(Region::US);
        // Need to figure out the the default index name here
        let mut index = self.index(Region::to_str(region)).await.unwrap();
        let index_model = self.describe_index(Region::to_str(region)).await.unwrap();
        let index_stats = index.describe_index_stats(None).await.unwrap();
        let vector_count: Option<u64> = index_stats
            .namespaces
            .iter()
            .find_map(|(k, v)| (*k == search_request.collection).then(|| v.vector_count as u64));
        Ok(Some(CollectionMetadata {
            status: VectorDatabaseStatus::Ok,
            collection_vector_count: vector_count,
            metric: Some(Distance::from(index_model.metric)),
            dimensions: Some(index_model.dimension as u64),
        }))
    }

    async fn get_storage_size(
        &self,
        search_request: SearchRequest,
        vector_length: usize,
    ) -> Result<Option<StorageSize>, VectorDatabaseError> {
        if let Ok(collection_info) = self.get_collection_info(search_request.clone()).await {
            let vector_count = collection_info.unwrap().collection_vector_count.unwrap();
            let vector_storage_size =
                calculate_vector_storage_size(vector_count as usize, vector_length);
            return Ok(Some(StorageSize {
                status: VectorDatabaseStatus::Ok,
                collection_name: search_request.collection,
                size: Some(vector_storage_size),
                points_count: None,
            }));
        }
        Ok(None)
    }

    async fn scroll_points(
        &self,
        _search_request: SearchRequest,
    ) -> Result<Vec<ScrollResults>, VectorDatabaseError> {
        todo!()
    }

    async fn similarity_search(
        &self,
        search_request: SearchRequest,
    ) -> Result<Vec<SearchResult>, VectorDatabaseError> {
        let region = search_request.clone().region.unwrap_or(Region::US);
        // Need to figure out the the default index name here
        let mut index = self.index(Region::to_str(region)).await.unwrap();
        let namespace = Namespace::from(search_request.collection.as_str());
        let _return_params = search_request.search_response_params.unwrap();
        if let Some(vector) = search_request.vector {
            if let Ok(results) = index
                .query_by_value(
                    vector,
                    None,
                    search_request.top_k.unwrap(),
                    &namespace,
                    None,
                    Some(false),
                    Some(true),
                )
                .await
            {
                let matched_vectors: Vec<SearchResult> = results
                    .matches
                    .iter()
                    .map(|res| {
                        let point = Point::from(res.clone().metadata.unwrap());
                        SearchResult {
                            id: res.clone().id,
                            score: Some(res.score),
                            payload: point.payload,
                            vector: Some(res.clone().values),
                        }
                    })
                    .collect();
                Ok(matched_vectors)
            } else {
                Err(VectorDatabaseError::Other(
                    "Something bad happened!".to_string(),
                ))
            }
        } else {
            Err(VectorDatabaseError::Other(
                "Something bad happened!".to_string(),
            ))
        }
    }
}

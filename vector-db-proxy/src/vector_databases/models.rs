use crate::vector_databases::error::VectorDatabaseError;
use crate::vector_databases::models::Cloud::GCP;
use pinecone_sdk::models::{Metric, Vector};
use prost_types::Struct as Metadata;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Serialize)]
pub enum VectorDatabaseStatus {
    Ok,
    Failure,
    NotFound,
    Error(VectorDatabaseError),
}
pub enum CreateDisposition {
    CreateIfNeeded,
    CreateNever,
}
#[derive(Clone, Deserialize)]
pub struct Point {
    pub index: Option<String>,
    pub vector: Vec<f32>,
    pub payload: Option<HashMap<String, String>>,
}

impl Point {
    pub fn new(
        index: Option<String>,
        vector: Vec<f32>,
        payload: Option<HashMap<String, String>>,
    ) -> Self {
        Point {
            index,
            vector,
            payload,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    pub score: Option<f32>,
    pub payload: Option<HashMap<String, String>>,
    pub vector: Option<Vec<f32>>,
}

#[derive(Debug)]
pub struct CollectionsResult {
    pub status: VectorDatabaseStatus,
    pub collection_name: String,
    pub collection_metadata: Option<CollectionMetadata>,
}

#[derive(Debug, Serialize)]
pub struct CollectionMetadata {
    pub status: VectorDatabaseStatus,
    pub collection_vector_count: Option<u64>,
    pub metric: Option<Distance>,
    pub dimensions: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ScrollResults {
    pub status: VectorDatabaseStatus,
    pub id: String,
    pub payload: HashMap<String, String>,
    pub vector: Vec<f32>,
}

#[derive(Serialize, Clone, Debug, Deserialize)]
pub struct FilterConditions {
    pub must: Option<Vec<HashMap<String, String>>>,
    pub must_not: Option<Vec<HashMap<String, String>>>,
    pub should: Option<Vec<HashMap<String, String>>>,
}

impl Default for FilterConditions {
    fn default() -> Self {
        FilterConditions {
            must: None,
            must_not: None,
            should: None,
        }
    }
}
// This will dictate what is included in the response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResponseParams {
    pub include_vectors: Option<bool>,
    pub include_payload: Option<bool>,
    pub get_all_pages: Option<bool>,
    pub limit: Option<u32>,
}

// This will dictate the type of search that is conducted
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SearchType {
    Collection,
    Point,
    Similarity,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub search_type: SearchType,
    pub collection: String,
    pub id: Option<String>,
    pub vector: Option<Vec<f32>>,
    pub filters: Option<FilterConditions>,
    pub search_response_params: Option<SearchResponseParams>,
    pub region: Option<Region>,
    pub cloud: Option<Cloud>,
}

impl SearchRequest {
    pub fn new(search_type: SearchType, collection: String) -> Self {
        Self {
            search_type,
            collection,
            id: None,
            vector: None,
            filters: None,
            search_response_params: None,
            region: Some(Region::US),
            cloud: Some(GCP),
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Region {
    US,
    EU,
    AU,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Cloud {
    GCP,
    AWS,
    AZURE,
}

#[derive(Debug, Clone, Serialize)]
pub struct StorageSize {
    pub status: VectorDatabaseStatus,
    pub collection_name: String,
    pub size: Option<f64>,
    pub points_count: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Distance {
    UnknownDistance = 0,
    Cosine = 1,
    Euclid = 2,
    Dot = 3,
    Manhattan = 4,
}

impl From<Metric> for Distance {
    fn from(value: Metric) -> Self {
        match value {
            Metric::Cosine => Distance::Cosine,
            Metric::Dotproduct => Distance::Dot,
            Metric::Euclidean => Distance::Euclid,
        }
    }
}
#[derive(Serialize, Debug, Clone)]
pub struct CollectionCreate {
    pub collection_name: String,
    pub size: u64,
    pub namespace: Option<String>,
    pub distance: Distance,
    pub vector_name: Option<String>,
}

impl From<bool> for VectorDatabaseStatus {
    fn from(value: bool) -> Self {
        match value {
            true => VectorDatabaseStatus::Ok,
            false => VectorDatabaseStatus::Failure,
        }
    }
}
impl From<qdrant_client::qdrant::CollectionInfo> for VectorDatabaseStatus {
    fn from(value: qdrant_client::qdrant::CollectionInfo) -> Self {
        match value.status {
            1 => VectorDatabaseStatus::Ok,
            2 => VectorDatabaseStatus::Failure,
            _ => VectorDatabaseStatus::Error(VectorDatabaseError::Other(String::from(
                "An error \
            occurred",
            ))),
        }
    }
}

impl From<Point> for BTreeMap<String, String> {
    fn from(value: Point) -> Self {
        BTreeMap::from_iter(value.payload.unwrap_or(HashMap::new()))
    }
}

impl From<Point> for Metadata {
    fn from(value: Point) -> Self {
        let mut btree_map = BTreeMap::new();
        for (k, v) in value.payload.unwrap() {
            btree_map.insert(
                k,
                prost_types::Value {
                    kind: Some(prost_types::value::Kind::StringValue(v)),
                },
            );
        }

        Self { fields: btree_map }
    }
}

impl From<Point> for Vector {
    fn from(value: Point) -> Self {
        let metadata = Some(Metadata::from(value.clone()));
        Self {
            id: value.index.unwrap(),
            values: value.vector,
            sparse_values: None,
            metadata,
        }
    }
}
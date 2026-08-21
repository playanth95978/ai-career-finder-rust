use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Custom query extractor that uses serde_qs to properly handle repeated query parameters
/// like `sort=name,asc&sort=id,desc`
#[derive(Debug, Clone)]
pub struct QsQuery<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for QsQuery<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = QsQueryRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let query = parts.uri.query().unwrap_or_default();
        // Pre-process query string to convert duplicate keys to indexed array format
        // e.g., "sort=a&sort=b" becomes "sort[0]=a&sort[1]=b"
        let processed_query = preprocess_duplicate_keys(query);
        let config = serde_qs::Config::new(5, false);
        let value = config.deserialize_str(&processed_query).map_err(|e| QsQueryRejection(e.to_string()))?;
        Ok(QsQuery(value))
    }
}

/// Pre-process query string to convert duplicate keys to indexed array format
/// This allows serde_qs to properly deserialize duplicate keys like "sort=a&sort=b"
/// into a Vec by converting them to "sort[0]=a&sort[1]=b"
fn preprocess_duplicate_keys(query: &str) -> String {
    use std::collections::HashMap;

    let mut key_counts: HashMap<&str, usize> = HashMap::new();
    let mut result_pairs: Vec<String> = Vec::new();

    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }

        if let Some((key, value)) = pair.split_once('=') {
            let count = key_counts.entry(key).or_insert(0);
            if *count == 0 {
                // First occurrence - check if there are more of this key
                let occurrences = query.split('&')
                    .filter(|p| p.starts_with(&format!("{}=", key)))
                    .count();
                if occurrences > 1 {
                    // Multiple occurrences - use indexed format
                    result_pairs.push(format!("{}[{}]={}", key, count, value));
                } else {
                    // Single occurrence - keep as-is
                    result_pairs.push(pair.to_string());
                }
            } else {
                // Subsequent occurrence - always use indexed format
                result_pairs.push(format!("{}[{}]={}", key, count, value));
            }
            *count += 1;
        } else {
            // No '=' in pair, keep as-is
            result_pairs.push(pair.to_string());
        }
    }

    result_pairs.join("&")
}

/// Rejection type for QsQuery extractor
#[derive(Debug)]
pub struct QsQueryRejection(String);

impl IntoResponse for QsQueryRejection {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to deserialize query string: {}", self.0),
        )
            .into_response()
    }
}

/// Deserialize sort parameter that can be either a single string or an array of strings
fn deserialize_sort<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct SortVisitor;

    impl<'de> Visitor<'de> for SortVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or array of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![value.to_string()])
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![value])
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(value) = seq.next_element::<String>()? {
                vec.push(value);
            }
            Ok(vec)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Vec::new())
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Vec::new())
        }
    }

    deserializer.deserialize_any(SortVisitor)
}

/// Pagination request parameters
#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema)]
pub struct PageRequest {
    /// Page number (0-indexed)
    pub page: Option<i64>,
    /// Number of items per page (max 100)
    pub size: Option<i64>,
    /// Sort parameters - can be a single value (sort=id,asc) or multiple (sort=name,asc&sort=id,desc)
    /// Each sort value is in format "field,direction" where direction is "asc" or "desc"
    #[serde(default, deserialize_with = "deserialize_sort")]
    #[param(value_type = Option<String>)]
    pub sort: Vec<String>,
}

impl PageRequest {
    pub fn offset(&self) -> i64 {
        let page = self.page.unwrap_or(0);
        let size = self.size.unwrap_or(20);
        page * size
    }

    pub fn limit(&self) -> i64 {
        self.size.unwrap_or(20).min(100)
    }

    /// Get the primary sort field and direction (first sort parameter)
    pub fn primary_sort(&self) -> Option<(&str, &str)> {
        self.sort.first().and_then(|s| {
            let parts: Vec<&str> = s.split(',').collect();
            if parts.len() >= 2 {
                Some((parts[0], parts[1]))
            } else if parts.len() == 1 {
                Some((parts[0], "asc"))
            } else {
                None
            }
        })
    }

    /// Get all sort parameters as (field, direction) tuples
    pub fn sort_params(&self) -> Vec<(&str, &str)> {
        self.sort
            .iter()
            .filter_map(|s| {
                let parts: Vec<&str> = s.split(',').collect();
                if parts.len() >= 2 {
                    Some((parts[0], parts[1]))
                } else if parts.len() == 1 {
                    Some((parts[0], "asc"))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            page: Some(0),
            size: Some(20),
            sort: Vec::new(),
        }
    }
}

/// Paginated response
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PageResponse<T> {
    /// List of items on this page
    pub content: Vec<T>,
    /// Total number of items across all pages
    pub total_elements: i64,
    /// Current page number (0-indexed)
    pub page: i64,
    /// Number of items per page
    pub size: i64,
}

impl<T> PageResponse<T> {
    pub fn new(content: Vec<T>, total_elements: i64, page: i64, size: i64) -> Self {
        Self {
            content,
            total_elements,
            page,
            size,
        }
    }

    pub fn total_pages(&self) -> i64 {
        if self.size == 0 {
            0
        } else {
            (self.total_elements + self.size - 1) / self.size
        }
    }

    pub fn has_next(&self) -> bool {
        self.page < self.total_pages() - 1
    }

    pub fn has_previous(&self) -> bool {
        self.page > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod page_request_tests {
        use super::*;

        #[test]
        fn test_default_page_request() {
            let request = PageRequest::default();
            assert_eq!(request.page, Some(0));
            assert_eq!(request.size, Some(20));
            assert!(request.sort.is_empty());
        }

        #[test]
        fn test_offset_calculation() {
            let request = PageRequest {
                page: Some(2),
                size: Some(10),
                sort: Vec::new(),
            };
            assert_eq!(request.offset(), 20);
        }

        #[test]
        fn test_offset_with_defaults() {
            let request = PageRequest {
                page: None,
                size: None,
                sort: Vec::new(),
            };
            assert_eq!(request.offset(), 0);
        }

        #[test]
        fn test_limit_respects_max() {
            let request = PageRequest {
                page: Some(0),
                size: Some(200),
                sort: Vec::new(),
            };
            assert_eq!(request.limit(), 100);
        }

        #[test]
        fn test_limit_with_default() {
            let request = PageRequest {
                page: Some(0),
                size: None,
                sort: Vec::new(),
            };
            assert_eq!(request.limit(), 20);
        }

        #[test]
        fn test_primary_sort_with_direction() {
            let request = PageRequest {
                page: Some(0),
                size: Some(20),
                sort: vec!["name,asc".to_string(), "id,desc".to_string()],
            };
            assert_eq!(request.primary_sort(), Some(("name", "asc")));
        }

        #[test]
        fn test_primary_sort_without_direction() {
            let request = PageRequest {
                page: Some(0),
                size: Some(20),
                sort: vec!["name".to_string()],
            };
            assert_eq!(request.primary_sort(), Some(("name", "asc")));
        }

        #[test]
        fn test_primary_sort_empty() {
            let request = PageRequest::default();
            assert_eq!(request.primary_sort(), None);
        }

        #[test]
        fn test_sort_params_multiple() {
            let request = PageRequest {
                page: Some(0),
                size: Some(20),
                sort: vec!["email,asc".to_string(), "id,desc".to_string()],
            };
            let params = request.sort_params();
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], ("email", "asc"));
            assert_eq!(params[1], ("id", "desc"));
        }
    }

    mod page_response_tests {
        use super::*;

        #[test]
        fn test_new_page_response() {
            let response: PageResponse<i32> = PageResponse::new(vec![1, 2, 3], 100, 0, 20);
            assert_eq!(response.content.len(), 3);
            assert_eq!(response.total_elements, 100);
            assert_eq!(response.page, 0);
            assert_eq!(response.size, 20);
        }

        #[test]
        fn test_total_pages_calculation() {
            let response: PageResponse<i32> = PageResponse::new(vec![], 100, 0, 20);
            assert_eq!(response.total_pages(), 5);
        }

        #[test]
        fn test_total_pages_with_remainder() {
            let response: PageResponse<i32> = PageResponse::new(vec![], 101, 0, 20);
            assert_eq!(response.total_pages(), 6);
        }

        #[test]
        fn test_total_pages_zero_size() {
            let response: PageResponse<i32> = PageResponse::new(vec![], 100, 0, 0);
            assert_eq!(response.total_pages(), 0);
        }

        #[test]
        fn test_has_next_true() {
            let response: PageResponse<i32> = PageResponse::new(vec![], 100, 0, 20);
            assert!(response.has_next());
        }

        #[test]
        fn test_has_next_false_on_last_page() {
            let response: PageResponse<i32> = PageResponse::new(vec![], 100, 4, 20);
            assert!(!response.has_next());
        }

        #[test]
        fn test_has_previous_false_on_first_page() {
            let response: PageResponse<i32> = PageResponse::new(vec![], 100, 0, 20);
            assert!(!response.has_previous());
        }

        #[test]
        fn test_has_previous_true() {
            let response: PageResponse<i32> = PageResponse::new(vec![], 100, 2, 20);
            assert!(response.has_previous());
        }
    }

    // Track 1 Phase 1a (2026-05-11): coverage push for the remaining ~30 uncovered
    // lines — preprocess_duplicate_keys, QsQuery extractor, QsQueryRejection
    // IntoResponse, and the SortVisitor's visit_* methods.
    mod preprocess_duplicate_keys_tests {
        use super::super::preprocess_duplicate_keys;

        #[test]
        fn test_empty_query_returns_empty_string() {
            assert_eq!(preprocess_duplicate_keys(""), "");
        }

        #[test]
        fn test_single_pair_passes_through() {
            // Single occurrence keeps the original key=value shape (no indexing).
            assert_eq!(preprocess_duplicate_keys("sort=name,asc"), "sort=name,asc");
        }

        #[test]
        fn test_duplicate_keys_become_indexed() {
            // The whole point of this helper: serde_qs needs sort[0]=..&sort[1]=..
            // to deserialize repeated keys into a Vec.
            let out = preprocess_duplicate_keys("sort=name,asc&sort=id,desc");
            assert_eq!(out, "sort[0]=name,asc&sort[1]=id,desc");
        }

        #[test]
        fn test_mixed_single_and_duplicate_keys() {
            let out = preprocess_duplicate_keys("page=0&sort=name,asc&sort=id,desc&size=20");
            assert_eq!(out, "page=0&sort[0]=name,asc&sort[1]=id,desc&size=20");
        }

        #[test]
        fn test_skips_empty_fragments() {
            // Trailing/leading & creates empty fragments that must be skipped.
            assert_eq!(preprocess_duplicate_keys("&page=0&"), "page=0");
        }

        #[test]
        fn test_fragment_without_equals_passes_through() {
            // Bare key (no =) is preserved verbatim — serde_qs may treat as flag.
            assert_eq!(preprocess_duplicate_keys("flag&page=0"), "flag&page=0");
        }

        #[test]
        fn test_three_duplicates_all_get_indexed() {
            let out = preprocess_duplicate_keys("sort=a&sort=b&sort=c");
            assert_eq!(out, "sort[0]=a&sort[1]=b&sort[2]=c");
        }
    }

    mod qs_query_extractor_tests {
        use super::super::{QsQuery, PageRequest};
        use axum::extract::FromRequestParts;
        use axum::http::Request;

        async fn extract(uri: &str) -> Result<QsQuery<PageRequest>, super::super::QsQueryRejection> {
            let req = Request::builder().uri(uri).body(()).unwrap();
            let (mut parts, _) = req.into_parts();
            QsQuery::<PageRequest>::from_request_parts(&mut parts, &()).await
        }

        #[tokio::test]
        async fn test_extracts_single_sort_param() {
            let result = extract("/?sort=name,asc").await.unwrap();
            assert_eq!(result.0.sort, vec!["name,asc".to_string()]);
        }

        #[tokio::test]
        async fn test_extracts_duplicate_sort_params_as_vec() {
            // Verifies the duplicate-key preprocessing flows through to PageRequest.sort.
            let result = extract("/?sort=name,asc&sort=id,desc").await.unwrap();
            assert_eq!(result.0.sort, vec!["name,asc".to_string(), "id,desc".to_string()]);
        }

        #[tokio::test]
        async fn test_extracts_page_and_size() {
            let result = extract("/?page=3&size=50").await.unwrap();
            assert_eq!(result.0.page, Some(3));
            assert_eq!(result.0.size, Some(50));
        }

        #[tokio::test]
        async fn test_empty_query_extracts_defaults() {
            let result = extract("/").await.unwrap();
            assert_eq!(result.0.page, None);
            assert_eq!(result.0.size, None);
            assert!(result.0.sort.is_empty());
        }

        #[tokio::test]
        async fn test_invalid_query_returns_rejection() {
            // `page` is i64; non-numeric must fail deserialization.
            let result = extract("/?page=not-a-number").await;
            assert!(result.is_err());
        }
    }

    mod qs_query_rejection_tests {
        use super::super::QsQueryRejection;
        use axum::http::StatusCode;
        use axum::response::IntoResponse;

        #[test]
        fn test_into_response_returns_bad_request_status() {
            let rejection = QsQueryRejection("test failure".to_string());
            let response = rejection.into_response();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }

        #[tokio::test]
        async fn test_into_response_includes_error_message_in_body() {
            use axum::body::to_bytes;
            let rejection = QsQueryRejection("sentinel-error-text".to_string());
            let response = rejection.into_response();
            let bytes = to_bytes(response.into_body(), 1024).await.unwrap();
            let body = String::from_utf8(bytes.to_vec()).unwrap();
            assert!(body.contains("sentinel-error-text"), "got: {}", body);
            assert!(body.contains("Failed to deserialize"), "got: {}", body);
        }
    }

    mod deserialize_sort_tests {
        use super::super::PageRequest;

        // deserialize_sort is private; exercise it through PageRequest's
        // serde_json derive (which invokes the same Visitor).
        #[test]
        fn test_sort_as_single_string() {
            // visit_str / visit_string path.
            let req: PageRequest = serde_json::from_str(r#"{"sort": "name,asc"}"#).unwrap();
            assert_eq!(req.sort, vec!["name,asc".to_string()]);
        }

        #[test]
        fn test_sort_as_array() {
            // visit_seq path.
            let req: PageRequest = serde_json::from_str(r#"{"sort": ["name,asc", "id,desc"]}"#).unwrap();
            assert_eq!(req.sort, vec!["name,asc".to_string(), "id,desc".to_string()]);
        }

        #[test]
        fn test_sort_as_null_returns_empty_vec() {
            // visit_none path.
            let req: PageRequest = serde_json::from_str(r#"{"sort": null}"#).unwrap();
            assert!(req.sort.is_empty());
        }

        #[test]
        fn test_sort_missing_returns_empty_vec() {
            // serde default kicks in — exercises the `#[serde(default, ...)]` path.
            let req: PageRequest = serde_json::from_str("{}").unwrap();
            assert!(req.sort.is_empty());
        }

        #[test]
        fn test_sort_as_empty_array() {
            let req: PageRequest = serde_json::from_str(r#"{"sort": []}"#).unwrap();
            assert!(req.sort.is_empty());
        }
    }

    mod page_request_edge_cases {
        use super::super::PageRequest;

        #[test]
        fn test_primary_sort_with_empty_string_returns_some_with_asc_default() {
            // An empty `sort` entry splits to [""], a 1-element vec — the
            // 1-element branch in primary_sort defaults direction to "asc".
            let req = PageRequest {
                page: None,
                size: None,
                sort: vec!["".to_string()],
            };
            assert_eq!(req.primary_sort(), Some(("", "asc")));
        }

        #[test]
        fn test_sort_params_skips_no_parts_entries() {
            // sort_params filter-maps; entries that yield 0 parts are dropped.
            // (split(',') on non-empty string always yields ≥1 part, so this
            // documents the 1-part default-direction path through sort_params.)
            let req = PageRequest {
                page: None,
                size: None,
                sort: vec!["only-field".to_string()],
            };
            let params = req.sort_params();
            assert_eq!(params, vec![("only-field", "asc")]);
        }
    }
}

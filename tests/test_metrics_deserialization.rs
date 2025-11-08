
#[test]
fn test_real_api_response_deserialization() {
    // Real response from Datadog API (success case)
    let json_success = r#"{
  "status": "ok",
  "res_type": "time_series",
  "resp_version": 1,
  "query": "avg:postgresql.queries.count{*}",
  "from_date": 1762572843000,
  "to_date": 1762576443000,
  "series": [{
    "unit": [{"family": "db", "id": 37, "name": "query", "short_name": null, "plural": "queries", "scale_factor": 1.0}, null],
    "query_index": 0,
    "aggr": "avg",
    "metric": "postgresql.queries.count",
    "tag_set": [],
    "expression": "avg:postgresql.queries.count{*}",
    "scope": "*",
    "interval": 20,
    "length": 179,
    "start": 1762572860000,
    "end": 1762576439000,
    "pointlist": [[1762572860000.0, 4.198947524470634]],
    "display_name": "postgresql.queries.count",
    "attributes": {}
  }],
  "values": [],
  "times": [],
  "message": "",
  "group_by": []
}"#;

    // Real error response
    let json_error = r#"{
  "status": "error",
  "res_type": "time_series",
  "resp_version": 1,
  "query": "invalid query",
  "from_date": 1762572828000,
  "to_date": 1762576428000,
  "series": [],
  "values": [],
  "times": [],
  "error": "Error parsing query",
  "message": null,
  "group_by": []
}"#;

    // Test success case
    let result_success: Result<mcp_datadog::datadog::models::MetricsResponse, _> = 
        serde_json::from_str(json_success);
    
    assert!(result_success.is_ok(), "Failed to deserialize success response: {:?}", result_success.err());
    
    let response = result_success.unwrap();
    assert_eq!(response.status, "ok");
    assert_eq!(response.resp_version, Some(1));
    assert_eq!(response.series.len(), 1);
    assert_eq!(response.message, Some("".to_string()));
    assert_eq!(response.group_by, Some(vec![]));
    
    // Test error case
    let result_error: Result<mcp_datadog::datadog::models::MetricsResponse, _> = 
        serde_json::from_str(json_error);
    
    assert!(result_error.is_ok(), "Failed to deserialize error response: {:?}", result_error.err());
    
    let error_response = result_error.unwrap();
    assert_eq!(error_response.status, "error");
    assert!(error_response.error.is_some());
}

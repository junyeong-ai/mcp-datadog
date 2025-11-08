use serde_json::json;

#[tokio::test]
async fn test_optimized_metrics_response() {
    use std::sync::Arc;
    use mcp_datadog::datadog::DatadogClient;
    use mcp_datadog::handlers::metrics::MetricsHandler;
    
    // Skip if no API keys
    let api_key = match std::env::var("DD_API_KEY") {
        Ok(k) => k,
        Err(_) => {
            println!("Skipping test - no DD_API_KEY");
            return;
        }
    };
    let app_key = match std::env::var("DD_APP_KEY") {
        Ok(k) => k,
        Err(_) => {
            println!("Skipping test - no DD_APP_KEY");
            return;
        }
    };
    
    let client = Arc::new(DatadogClient::new(api_key, app_key, None).unwrap());
    
    let params = json!({
        "query": "avg:system.cpu.user{*}",
        "from": "1 hour ago",
        "to": "now"
    });
    
    let result = MetricsHandler::query(client, &params).await;
    
    assert!(result.is_ok(), "Query failed: {:?}", result.err());
    
    let response = result.unwrap();
    
    // Check structure
    assert!(response.get("data").is_some());
    assert!(response.get("meta").is_some());
    
    let meta = response["meta"].as_object().unwrap();
    
    // Essential fields must be present
    assert!(meta.contains_key("query"));
    assert!(meta.contains_key("status"));
    assert!(meta.contains_key("from"));
    assert!(meta.contains_key("to"));
    
    // Removed fields should not be present
    assert!(!meta.contains_key("res_type"), "res_type should be removed");
    assert!(!meta.contains_key("resp_version"), "resp_version should be removed");
    assert!(!meta.contains_key("values"), "values should be removed");
    assert!(!meta.contains_key("times"), "times should be removed");
    
    // Conditional fields - only present if non-empty
    // error, message, group_by should not be present in success case with no grouping
    
    let data = response["data"].as_array().unwrap();
    if !data.is_empty() {
        let first_series = &data[0];
        
        // Essential fields
        assert!(first_series.get("metric").is_some());
        assert!(first_series.get("scope").is_some());
        assert!(first_series.get("points").is_some());
        
        // Removed fields
        assert!(!first_series.as_object().unwrap().contains_key("display_name"), "display_name should be removed");
        assert!(!first_series.as_object().unwrap().contains_key("expression"), "expression should be removed");
        assert!(!first_series.as_object().unwrap().contains_key("query_index"), "query_index should be removed");
        assert!(!first_series.as_object().unwrap().contains_key("length"), "length should be removed");
        assert!(!first_series.as_object().unwrap().contains_key("start"), "start should be removed");
        assert!(!first_series.as_object().unwrap().contains_key("end"), "end should be removed");
        
        // Unit should be simplified
        if let Some(unit) = first_series.get("unit") {
            let unit_obj = unit.as_object().unwrap();
            assert!(unit_obj.contains_key("name"));
            assert!(unit_obj.contains_key("family"));
            // short_name is optional
        }
    }
    
    println!("✅ Optimized response structure validated!");
    println!("Meta keys: {:?}", meta.keys().collect::<Vec<_>>());
    if !data.is_empty() {
        println!("Series keys: {:?}", data[0].as_object().unwrap().keys().collect::<Vec<_>>());
    }
}

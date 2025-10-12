use serde_json::{Value, json};
use std::sync::Arc;

use crate::cache::DataCache;
use crate::datadog::DatadogClient;
use crate::error::Result;
use crate::handlers::common::{Paginator, ResponseFormatter};

pub struct DashboardsHandler;

impl Paginator for DashboardsHandler {}
impl ResponseFormatter for DashboardsHandler {}

impl DashboardsHandler {
    // Recursively collect widget types from widgets (including nested groups)
    fn collect_widget_types(widgets: &[crate::datadog::models::Widget]) -> Vec<String> {
        let mut types = std::collections::HashSet::new();

        fn collect_recursive(widget: &crate::datadog::models::Widget, types: &mut std::collections::HashSet<String>) {
            types.insert(widget.definition.widget_type.clone());

            // If it's a group widget, check for nested widgets in extra field
            if widget.definition.widget_type == "group" {
                if let Some(widgets_value) = widget.definition.extra.get("widgets") {
                    if let Some(nested_array) = widgets_value.as_array() {
                        for nested_value in nested_array {
                            // Try to deserialize each nested widget
                            if let Ok(nested_widget) = serde_json::from_value::<crate::datadog::models::Widget>(nested_value.clone()) {
                                collect_recursive(&nested_widget, types);
                            }
                        }
                    }
                }
            }
        }

        for widget in widgets {
            collect_recursive(widget, &mut types);
        }

        let mut types_vec: Vec<String> = types.into_iter().collect();
        types_vec.sort();
        types_vec
    }

    pub async fn list(
        client: Arc<DatadogClient>,
        cache: Arc<DataCache>,
        params: &Value,
    ) -> Result<Value> {
        let handler = DashboardsHandler;
        let (page, page_size) = handler.parse_pagination(params);

        let cache_key = crate::cache::create_cache_key("dashboards", &json!({}));

        let all_dashboards = if page == 0 {
            let response = client.list_dashboards().await?;
            let dashboards = response.dashboards.clone();
            cache.set_dashboards(cache_key, dashboards.clone()).await;
            dashboards
        } else {
            cache
                .get_or_fetch_dashboards(&cache_key, || async {
                    let response = client.list_dashboards().await?;
                    Ok(response.dashboards)
                })
                .await?
        };

        let total_count = all_dashboards.len();
        let start = page * page_size;
        let end = std::cmp::min(start + page_size, total_count);

        if start >= total_count {
            let data = json!([]);
            let pagination = handler.format_pagination(page, page_size, total_count);
            return Ok(handler.format_list(data, Some(pagination), None));
        }

        let paginated_dashboards = &all_dashboards[start..end];
        let data = json!(paginated_dashboards);

        let pagination = handler.format_pagination(page, page_size, total_count);

        Ok(handler.format_list(data, Some(pagination), None))
    }

    pub async fn get(client: Arc<DatadogClient>, params: &Value) -> Result<Value> {
        let handler = DashboardsHandler;
        let dashboard_id = params["dashboard_id"].as_str().ok_or_else(|| {
            crate::error::DatadogError::InvalidInput("Missing 'dashboard_id' parameter".to_string())
        })?;

        let response = client.get_dashboard(dashboard_id).await?;

        let data = json!({
            "id": response.id,
            "title": response.title,
            "description": response.description,
            "url": response.url,
            "layout_type": response.layout_type,
            "is_read_only": response.is_read_only.unwrap_or(false),
            "created_at": response.created_at,
            "modified_at": response.modified_at,
            "tags": response.tags.as_ref().unwrap_or(&Vec::new()),
            "author": response.author_info.as_ref().map(|author| json!({
                "name": author.name,
                "handle": author.handle,
                "email": author.email
            })),
            "template_variables": response.template_variables.as_ref().map(|vars| {
                vars.iter().map(|var| json!({
                    "name": var.name,
                    "default": var.default_value,
                    "prefix": var.prefix,
                    "available_values": var.available_values
                })).collect::<Vec<_>>()
            }).unwrap_or_default(),
            "widgets_summary": json!({
                "total_widgets": response.widgets.len(),
                "widget_types": Self::collect_widget_types(&response.widgets),
                "widgets": response.widgets.iter().map(|widget| json!({
                    "id": widget.id,
                    "type": widget.definition.widget_type,
                    "title": widget.definition.title,
                    "layout": widget.layout.as_ref().map(|l| json!({
                        "x": l.x,
                        "y": l.y,
                        "width": l.width,
                        "height": l.height
                    }))
                })).collect::<Vec<_>>()
            })
        });

        Ok(handler.format_detail(data))
    }
}

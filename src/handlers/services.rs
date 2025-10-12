use serde_json::{Value, json};
use std::sync::Arc;

use crate::datadog::DatadogClient;
use crate::error::Result;
use crate::handlers::common::{Paginator, ResponseFormatter};

pub struct ServicesHandler;

impl Paginator for ServicesHandler {}
impl ResponseFormatter for ServicesHandler {}

impl ServicesHandler {
    pub async fn list(client: Arc<DatadogClient>, params: &Value) -> Result<Value> {
        let handler = ServicesHandler;
        let (page, page_size) = handler.parse_pagination(params);

        let page_size_param = Some(page_size as i32);
        let page_number_param = Some(page as i32);
        let filter_env = params["env"].as_str().map(|s| s.to_string());

        let response = client
            .get_service_catalog(page_size_param, page_number_param, filter_env.clone())
            .await?;

        let services_count = response.data.len();

        let data = json!(
            response
                .data
                .iter()
                .map(|service| {
                    let mut formatted_service = json!({
                        "id": service.id,
                        "type": service.service_type,
                    });

                    if let Some(attributes) = &service.attributes {
                        formatted_service["schema_version"] = json!(attributes.schema_version);
                        formatted_service["dd_service"] = json!(attributes.dd_service);
                        formatted_service["dd_team"] = json!(attributes.dd_team);
                        formatted_service["application"] = json!(attributes.application);
                        formatted_service["tier"] = json!(attributes.tier);
                        formatted_service["lifecycle"] = json!(attributes.lifecycle);
                        formatted_service["type_of_service"] = json!(attributes.type_of_service);
                        formatted_service["languages"] = json!(attributes.languages);
                        formatted_service["tags"] = json!(attributes.tags);

                        if let Some(contacts) = &attributes.contacts {
                            formatted_service["contacts"] = json!(
                                contacts
                                    .iter()
                                    .map(|c| json!({
                                        "name": c.name,
                                        "email": c.email,
                                        "type": c.contact_type
                                    }))
                                    .collect::<Vec<_>>()
                            );
                        }

                        if let Some(links) = &attributes.links {
                            formatted_service["links"] = json!(
                                links
                                    .iter()
                                    .map(|l| json!({
                                        "name": l.name,
                                        "url": l.url,
                                        "type": l.link_type
                                    }))
                                    .collect::<Vec<_>>()
                            );
                        }

                        if let Some(repos) = &attributes.repos {
                            formatted_service["repos"] = json!(
                                repos
                                    .iter()
                                    .map(|r| json!({
                                        "name": r.name,
                                        "url": r.url,
                                        "provider": r.provider
                                    }))
                                    .collect::<Vec<_>>()
                            );
                        }

                        if let Some(docs) = &attributes.docs {
                            formatted_service["docs"] = json!(
                                docs.iter()
                                    .map(|d| json!({
                                        "name": d.name,
                                        "url": d.url,
                                        "provider": d.provider
                                    }))
                                    .collect::<Vec<_>>()
                            );
                        }

                        if let Some(integrations) = &attributes.integrations {
                            let mut integrations_json = json!({});

                            if let Some(pagerduty) = &integrations.pagerduty {
                                integrations_json["pagerduty"] = pagerduty.clone();
                            }

                            if let Some(slack) = &integrations.slack {
                                integrations_json["slack"] = slack.clone();
                            }

                            for (key, value) in &integrations.others {
                                integrations_json[key] = value.clone();
                            }

                            formatted_service["integrations"] = integrations_json;
                        }

                        // Include any extra attributes
                        for (key, value) in &attributes.extra {
                            if !formatted_service.as_object().unwrap().contains_key(key) {
                                formatted_service[key] = value.clone();
                            }
                        }
                    }

                    formatted_service
                })
                .collect::<Vec<_>>()
        );

        let pagination = handler.format_pagination(page, page_size, services_count);

        let meta = json!({
            "filter_env": filter_env,
            "warnings": response.meta.as_ref().and_then(|m| m.warnings.clone()).unwrap_or_default(),
            "next": response.links.as_ref().and_then(|l| l.next.clone())
        });

        Ok(handler.format_list(data, Some(pagination), Some(meta)))
    }
}

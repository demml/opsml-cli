/// Copyright (c) Shipt, Inc.
/// This source code is licensed under the MIT license found in the
/// LICENSE file in the root directory of this source tree.
use crate::api::route_helper::RouteHelper;
use crate::api::types;
use crate::api::utils;
use anyhow::Context;
use tabled::settings::style::Style;
use tabled::{settings::Alignment, Table};

struct MetricGetter {}

impl MetricGetter {
    /// Parse metric response
    ///
    /// # Arguments
    ///
    /// * `response` - Response from server
    ///
    /// # Returns
    ///  String - Table of metrics
    ///
    fn parse_metric_response(&self, response: &str) -> Result<String, anyhow::Error> {
        // Parses response and creates a table

        let metrics: types::ListMetricResponse =
            serde_json::from_str(response).expect("Failed to load response to MetricResponse JSON");

        let mut metric_table: Vec<types::MetricTable> = Vec::new();

        for metric in metrics.metric.iter() {
            let step = if metric.step.is_some() {
                metric
                    .step
                    .as_ref()
                    .with_context(|| "Failed to parse metric step")?
                    .to_string()
            } else {
                "None".to_string()
            };

            let timestamp = if metric.timestamp.is_some() {
                metric.timestamp.as_ref().unwrap().to_string()
            } else {
                "None".to_string()
            };

            metric_table.push(types::MetricTable {
                metric: metric.name.clone(),
                value: metric.value.clone(),
                step,
                timestamp,
            });
        }

        let metric_table = Table::new(metric_table)
            .with(Alignment::center())
            .with(Style::sharp())
            .to_string();

        Ok(metric_table)
    }

    /// Get model metrics
    pub async fn get_model_metrics(&self, uid: &str) -> Result<(), anyhow::Error> {
        // if name and version then get most recent uid

        let params = [("run_uid", uid)];
        let response =
            RouteHelper::make_get_request(&utils::OpsmlPaths::Metric.as_str(), Some(&params))
                .await?;

        if response.status().is_success() {
            let metric_table = self
                .parse_metric_response(&response.text().await?)
                .with_context(|| "Failed to parse metrics")?;

            println!("\nModel Metrics");
            println!("{}", metric_table);
            Ok(())
        } else {
            Err(anyhow::Error::msg(format!(
                "Request failed {:?}",
                response.error_for_status_ref()
            )))
        }
    }
}

/// List all metrics for a model
///
/// # Arguments
///
/// * `name` - Name of the model
/// * `version` - Version of the model
/// * `uid` - Unique identifier of the model
/// * `url` - URL of the OpsML server
pub async fn get_model_metrics(uid: &str) -> Result<(), anyhow::Error> {
    let metric_getter = MetricGetter {};
    metric_getter.get_model_metrics(uid).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tokio;

    #[tokio::test]
    async fn test_get_metrics() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();
        let path = "./src/api/test_utils/list_metric.json";
        let metric_data = fs::read_to_string(path).expect("Unable to read file");

        env::set_var("OPSML_TRACKING_URI", url);

        let mut vec = Vec::new();
        let metric1 = types::Metric {
            run_uid: "test".to_string(),
            name: "mae".to_string(),
            value: 5.into(),
            step: None,
            timestamp: None,
        };
        vec.push(metric1);

        let metric2 = types::Metric {
            run_uid: "test".to_string(),
            name: "mape".to_string(),
            value: 10.0.into(),
            step: None,
            timestamp: None,
        };
        vec.push(metric2);

        let metric_getter = MetricGetter {};

        // Create a mock server
        let mock_get_metrics = server
            .mock("GET", "/opsml/metrics?run_uid=fake")
            .with_status(201)
            .with_body(metric_data)
            .create();

        metric_getter.get_model_metrics("fake").await.unwrap();

        let mock_response = types::ListMetricResponse { metric: vec };
        let string_response = serde_json::to_string(&mock_response).unwrap();

        let metric_table = metric_getter
            .parse_metric_response(&string_response)
            .unwrap();

        assert_eq!(
            metric_table,
            concat!(
                "┌────────┬───────┬──────┬───────────┐\n",
                "│ metric │ value │ step │ timestamp │\n",
                "├────────┼───────┼──────┼───────────┤\n",
                "│  mae   │   5   │ None │   None    │\n",
                "│  mape  │ 10.0  │ None │   None    │\n",
                "└────────┴───────┴──────┴───────────┘",
            )
        );

        mock_get_metrics.assert();
    }
}

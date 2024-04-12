/// Copyright (c) Shipt, Inc.
/// This source code is licensed under the MIT license found in the
/// LICENSE file in the root directory of this source tree.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tabled::Tabled;

#[derive(Debug, Serialize)]
pub struct ListTableRequest<'a> {
    pub registry_type: &'a str,
    pub name: Option<&'a str>,
    pub repository: Option<&'a str>,
    pub version: Option<&'a str>,
    pub uid: Option<&'a str>,
    pub limit: Option<&'a i16>,
    pub tags: &'a HashMap<String, String>,
    pub max_date: Option<&'a str>,
    pub ignore_release_candidates: &'a bool,
}

#[derive(Debug, Serialize)]
pub struct CardRequest<'a> {
    pub name: Option<&'a str>,
    pub version: Option<&'a str>,
    pub uid: Option<&'a str>,
}

#[derive(Serialize)]
pub struct ModelMetadataRequest<'a> {
    pub name: Option<&'a str>,
    pub version: Option<&'a str>,
    pub repository: Option<&'a str>,
    pub uid: Option<&'a str>,
    pub ignore_release_candidates: &'a bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    pub name: String,
    pub repository: String,
    pub date: Option<String>,
    pub contact: String,
    pub version: String,
    pub uid: String,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListFileResponse {
    pub files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListMetricResponse {
    pub metric: Vec<Metric>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metric {
    pub run_uid: String,
    pub name: String,
    pub value: Value,
    pub step: Option<Value>,
    pub timestamp: Option<Value>,
}

#[derive(Tabled)]
pub struct MetricTable {
    pub metric: String,
    pub value: Value,
    pub step: String,
    pub timestamp: String,
}

#[derive(Tabled)]
pub struct CompareMetricTable {
    pub champion_name: String,
    pub champion_version: Value,
    pub metric: String,
    pub champion_value: Value,
    pub challenger_value: Value,
    pub challenger_win: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListCardResponse {
    pub cards: Vec<Card>,
}

#[derive(Tabled)]
pub struct CardTable {
    pub name: String,
    pub repository: String,
    pub date: String,
    pub contact: String,
    pub version: String,
    pub uid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Feature {
    feature_type: String,
    shape: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSchema {
    data_type: Option<String>,
    input_features: Option<HashMap<String, Feature>>,
    output_features: Option<HashMap<String, Feature>>,
    onnx_input_features: Option<HashMap<String, Feature>>,
    onnx_output_features: Option<HashMap<String, Feature>>,
    onnx_data_type: Option<String>,
    onnx_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_name: String,
    pub model_class: String,
    pub model_type: String,
    pub model_interface: String,
    pub onnx_uri: Option<String>,
    pub onnx_version: Option<String>,
    pub model_uri: String,
    pub model_version: String,
    pub model_repository: String,
    pub sample_data_uri: String,
    pub data_schema: DataSchema,
    pub preprocessor_uri: Option<String>,
    pub preprocessor_name: Option<String>,
    pub tokenizer_uri: Option<String>,
    pub tokenizer_name: Option<String>,
    pub feature_extractor_uri: Option<String>,
    pub feature_extractor_name: Option<String>,
    pub quantized_model_uri: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CompareMetricRequest<'a> {
    pub metric_name: &'a Vec<String>,
    pub lower_is_better: &'a Vec<bool>,
    pub challenger_uid: &'a str,
    pub champion_uid: &'a Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BattleReport {
    pub champion_name: String,
    pub champion_version: String,
    pub champion_metric: Option<Metric>,
    pub challenger_metric: Option<Metric>,
    pub challenger_win: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompareMetricResponse {
    pub challenger_name: String,
    pub challenger_version: String,
    pub report: HashMap<String, Vec<BattleReport>>,
}

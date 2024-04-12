/// Copyright (c) Shipt, Inc.
/// This source code is licensed under the MIT license found in the
/// LICENSE file in the root directory of this source tree.
use crate::api::route_helper::RouteHelper;
use crate::api::types;
use crate::api::utils;
use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use reqwest::{self, Response};
use serde_json;
use std::collections::HashMap;
use tabled::settings::style::Style;
use tabled::{settings::Alignment, Table};

struct CardLister<'a> {
    pub registry_type: &'a str,
    pub name: Option<&'a str>,
    pub repository: Option<&'a str>,
    pub version: Option<&'a str>,
    pub uid: Option<&'a str>,
    pub limit: Option<&'a i16>,
    pub tags: HashMap<String, String>,
    pub max_date: Option<&'a str>,
    pub ignore_release_candidates: &'a bool,
}
impl CardLister<'_> {
    /// Checks if registry is valid
    ///
    /// # Arguments
    ///
    /// * `registry` - Registry to check
    ///
    fn validate_registry(&self) -> Result<(), anyhow::Error> {
        // Determines correct  registry to use

        let registries = ["data", "model", "run", "pipeline", "audit", "project"];

        if registries.contains(&self.registry_type) {
            Ok(())
        } else {
            Err(anyhow::Error::msg(format!(
                "Invalid registry: {}. Valid registries are: data, model, run, pipeline, audit, project",
                self.registry_type
            )))
        }
    }

    /// Parse card list response
    ///
    /// # Arguments
    ///
    /// * `response` - Response from server
    ///
    /// # Returns
    ///  String - Table of cards
    ///
    fn parse_list_response(&self, response: &str) -> Result<String, anyhow::Error> {
        // Parses response and creates a table

        let cards: types::ListCardResponse = serde_json::from_str(response)
            .with_context(|| "Failed to load response to ListCardResponse JSON")
            .unwrap();

        let mut card_table: Vec<types::CardTable> = Vec::new();

        for card in cards.cards.iter() {
            card_table.push(types::CardTable {
                name: card.name.clone(),
                repository: card.repository.clone(),
                date: card.date.clone().unwrap_or("".to_string()),
                contact: card.contact.clone(),
                version: card.version.clone(),
                uid: card.uid.clone(),
            });
        }

        let list_table = Table::new(card_table)
            .with(Alignment::center())
            .with(Style::sharp())
            .to_string();

        Ok(list_table)
    }

    /// Constructs tags hashmap from supplied value key pairs
    ///
    /// # Arguments
    ///
    /// * `tag_name` - Tag name
    /// * `tag_value` - Tag value
    ///
    /// # Returns
    /// HashMap<String, String> - Tags hashmap
    ///
    fn construct_tags(&mut self, tag_name: Option<Vec<String>>, tag_value: Option<Vec<String>>) {
        let mut tags: HashMap<String, String> = HashMap::new();
        if tag_name.is_some() && tag_value.is_some() {
            tags = tag_name
                .unwrap()
                .iter()
                .zip(tag_value.unwrap().iter())
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
        };

        self.tags = tags
    }

    /// Makes card request
    ///
    /// # Arguments
    ///
    /// * `registry` - Registry to list cards from
    /// * `name` - Name of card
    /// * `repository` - repository name
    ///
    async fn make_card_request(&self) -> Result<Response, anyhow::Error> {
        let list_table_request = types::ListTableRequest {
            registry_type: self.registry_type,
            name: self.name,
            repository: self.repository,
            version: self.version,
            limit: self.limit,
            uid: self.uid,
            tags: &self.tags,
            max_date: self.max_date,
            ignore_release_candidates: self.ignore_release_candidates,
        };

        let response = RouteHelper::make_post_request(
            &utils::OpsmlPaths::ListCard.as_str(),
            &list_table_request,
        )
        .await
        .unwrap();

        Ok(response)
    }

    #[allow(clippy::too_many_arguments)]
    async fn get_cards(
        registry: &str,
        name: Option<&str>,
        repository: Option<&str>,
        version: Option<&str>,
        uid: Option<&str>,
        limit: Option<i16>,
        tag_name: Option<Vec<String>>,
        tag_value: Option<Vec<String>>,
        max_date: Option<&str>,
        ignore_release_candidates: bool,
    ) -> Result<(), anyhow::Error> {
        let tags: HashMap<String, String> = HashMap::new();
        let mut card_lister = CardLister {
            registry_type: registry,
            name,
            repository,
            version,
            uid,
            limit: limit.as_ref(),
            tags,
            max_date,
            ignore_release_candidates: &ignore_release_candidates,
        };

        card_lister.validate_registry()?;
        card_lister.construct_tags(tag_name, tag_value);
        let response = card_lister.make_card_request().await?;

        if response.status().is_success() {
            let card_table = card_lister.parse_list_response(&response.text().await.unwrap());

            println!(
                "\nListing cards from {} registry",
                registry.to_string().bold().green()
            );
            println!("{}", card_table?);
            Ok(())
        } else {
            Err(anyhow::Error::msg(format!(
                "Failed to make call to list cards: {}",
                response.text().await.unwrap()
            )))
        }
    }
}

/// List cards
///     
/// # Arguments
///
/// * `registry` - Registry to list cards from
/// * `name` - Name of card
/// * `repository` - repository name
/// * `version` - Card version
/// * `uid` - Card uid
/// * `limit` - Limit number of cards returned
/// * `url` - OpsML url
/// * `tag_name` - Tag name
/// * `tag_value` - Tag value
/// * `max_date` - Max date
///
#[allow(clippy::too_many_arguments)]
pub async fn list_cards(
    registry: &str,
    name: Option<&str>,
    repository: Option<&str>,
    version: Option<&str>,
    uid: Option<&str>,
    limit: Option<i16>,
    tag_name: Option<Vec<String>>,
    tag_value: Option<Vec<String>>,
    max_date: Option<&str>,
    ignore_release_candidates: bool,
) -> Result<(), anyhow::Error> {
    CardLister::get_cards(
        registry,
        name,
        repository,
        version,
        uid,
        limit,
        tag_name,
        tag_value,
        max_date,
        ignore_release_candidates,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tokio;

    #[test]
    fn test_parse_response() {
        let mut vec = Vec::new();
        let card = types::Card {
            name: "test".to_string(),
            repository: "test".to_string(),
            date: Some("test".to_string()),
            contact: "fake_email".to_string(),
            version: "1.0.0".to_string(),
            uid: "uid".to_string(),
            tags: HashMap::new(),
        };
        vec.push(card);
        let mock_response = types::ListCardResponse { cards: vec };
        let string_response = serde_json::to_string(&mock_response).unwrap();

        let card_lister = CardLister {
            registry_type: "test",
            name: None,
            repository: None,
            version: None,
            uid: None,
            limit: None,
            tags: HashMap::new(),
            max_date: None,
            ignore_release_candidates: &false,
        };

        let card_table = card_lister.parse_list_response(&string_response);
        assert_eq!(
            card_table.unwrap(),
            concat!(
                "┌──────┬────────────┬──────┬────────────┬─────────┬─────┐\n",
                "│ name │ repository │ date │  contact   │ version │ uid │\n",
                "├──────┼────────────┼──────┼────────────┼─────────┼─────┤\n",
                "│ test │    test    │ test │ fake_email │  1.0.0  │ uid │\n",
                "└──────┴────────────┴──────┴────────────┴─────────┴─────┘",
            )
        );
    }

    #[tokio::test]
    async fn test_list_cards() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        env::set_var("OPSML_TRACKING_URI", url);

        let path = "./src/api/test_utils/list_cards.json";
        let data = fs::read_to_string(path).expect("Unable to read file");

        // Create a mock server
        let mock = server
            .mock("POST", "/opsml/cards/list")
            .with_status(201)
            .with_body(data)
            .create();

        CardLister::get_cards(
            "model", None, None, None, None, None, None, None, None, false,
        )
        .await
        .unwrap();

        mock.assert();
    }
}

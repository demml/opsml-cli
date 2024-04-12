use api::cards::list_cards;
use api::metrics::get_model_metrics;
/// Copyright (c) Shipt, Inc.
/// This source code is licensed under the MIT license found in the
/// LICENSE file in the root directory of this source tree.
use api::model::download_model;
use api::model::download_model_metadata;
mod api;
use anyhow::{Context, Result};
use api::cli::{Cli, Commands, LOGO_TEXT};
use clap::Parser;
use owo_colors::OwoColorize;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        // subcommand for list cards
        Some(Commands::ListCards(args)) => {
            list_cards(
                args.registry.as_str(),
                args.name.as_deref(),
                args.repository.as_deref(),
                args.version.as_deref(),
                args.uid.as_deref(),
                args.limit,
                args.tag_name.clone(),
                args.tag_value.clone(),
                args.max_date.as_deref(),
                args.ignore_release_candidates,
            )
            .await
            .with_context(|| format!("{}", "Failed to list cards".bold().red()))?;
            Ok(())
        }

        // subcommand for downloading model metadata
        Some(Commands::DownloadModelMetadata(args)) => {
            download_model_metadata(
                args.name.as_deref(),
                args.version.as_deref(),
                args.repository.as_deref(),
                args.uid.as_deref(),
                &args.write_dir,
                &args.ignore_release_candidates,
            )
            .await
            .with_context(|| {
                format!(
                    "Failed to download model metadata for {:?}",
                    args.name.clone().bold().red()
                )
            })?;

            Ok(())
        }
        // subcommand for downloading a model
        Some(Commands::DownloadModel(args)) => {
            download_model(
                args.name.as_deref(),
                args.version.as_deref(),
                args.repository.as_deref(),
                args.uid.as_deref(),
                &args.write_dir,
                &args.onnx,
                &args.quantize,
                &args.preprocessor,
                &args.ignore_release_candidates,
            )
            .await
            .with_context(|| {
                format!(
                    "Failed to download model for {:?}",
                    args.name.clone().bold().red()
                )
            })?;
            Ok(())
        }
        // subcommand for getting model metrics
        Some(Commands::GetModelMetrics(args)) => {
            get_model_metrics(args.uid.as_str())
                .await
                .with_context(|| {
                    format!(
                        "Failed to get model metrics for {:?}",
                        args.uid.clone().bold().red()
                    )
                })?;

            Ok(())
        }

        // subcommand for listing opsml-cli version
        Some(Commands::Version) => {
            println!(
                "opsml-cli version {}",
                env!("CARGO_PKG_VERSION").bold().green()
            );
            Ok(())
        }

        // subcommand for listing opsml-cli info
        Some(Commands::Info) => {
            println!(
                "\n{}\nopsml-cli version {}\n2023 Shipt, Inc.\n",
                LOGO_TEXT.green(),
                env!("CARGO_PKG_VERSION").bold().purple(),
            );

            Ok(())
        }

        None => Ok(()),
    }
}

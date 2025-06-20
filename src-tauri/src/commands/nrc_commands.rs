use crate::error::{AppError, CommandError};
use crate::minecraft::api::norisk_api::CrashlogDto;
use crate::minecraft::api::norisk_api::NoRiskApi;
// use crate::minecraft::api::wordpress_api::{BlogPost, WordPressApi}; // entfernt, weil wir BlogPost jetzt selbst definieren
use crate::minecraft::auth::minecraft_auth::Credentials;
use crate::state::state_manager::State;
use chrono::{Duration as ChronoDuration, Utc};
use log::info;
use log::{debug, error};
use std::sync::Arc;
use tauri::{AppHandle, Manager, Url, UserAttentionType, WebviewUrl, WebviewWindowBuilder};
use serde::Deserialize;

/// Eigene BlogPost-Datenstruktur – passt zu deiner JSON-Datei
#[derive(Debug, Deserialize, serde::Serialize)]
pub struct BlogPost {
    pub id: i32,
    pub yoast_head_json: YoastHeadJson,
}

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct YoastHeadJson {
    pub title: Option<String>,
    pub og_image: Option<Vec<OgImage>>,
    pub og_url: Option<String>,
}

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct OgImage {
    pub url: String,
}

/// Holt die News von deiner eigenen JSON-Datei
#[tauri::command]
pub async fn get_news_and_changelogs_command() -> Result<Vec<BlogPost>, CommandError> {
    info!("Executing get_news_and_changelogs_command");

    let url = "https://blublauncher.grueneeule.de/assets/json/launcher/news.json";

    let response = reqwest::get(url)
        .await
        .map_err(|e| CommandError::from(AppError::Other(format!(
            "Fehler beim Abrufen der News: {}",
            e
        ))))?;

    let posts: Vec<BlogPost> = response
        .json()
        .await
        .map_err(|e| CommandError::from(AppError::Other(format!(
            "Fehler beim Parsen der News: {}",
            e
        ))))?;

    Ok(posts)
}

#[tauri::command]
pub async fn discord_auth_link(app: AppHandle) -> Result<(), CommandError> {
    debug!("Executing discord_auth_link command");
    let state = State::get().await?;

    let is_experimental = state.config_manager.is_experimental_mode().await;

    let selected_account_arc = state
        .minecraft_account_manager_v2
        .get_active_account()
        .await?
        .ok_or(AppError::AccountError(
            "No active account found for Discord link.".to_string(),
        ))?;

    let norisk_creds = &selected_account_arc.norisk_credentials;
    let token = norisk_creds.get_token_for_mode(is_experimental)?;

    let url_string = format!(
        "https://api{}.norisk.gg/api/v1/core/oauth/discord?token={}",
        if is_experimental { "-staging" } else { "" },
        token
    );
    debug!("Generated Discord auth URL string: {}", url_string);

    let external_url = Url::parse(&url_string).map_err(|e| {
        CommandError::from(AppError::Other(format!(
            "Invalid URL format for Discord auth: {}",
            e
        )))
    })?;

    if let Some(window) = app.get_webview_window("discord-signin") {
        debug!("Closing existing discord-signin window.");
        if let Err(e) = window.close().map_err(|e_close| {
            CommandError::from(AppError::Other(format!(
                "Failed to close existing Discord window: {}",
                e_close
            )))
        }) {
            debug!("Error closing existing discord-signin window: {:?}", e);
        }
    }

    let start_time = Utc::now();

    let window =
        WebviewWindowBuilder::new(&app, "discord-signin", WebviewUrl::External(external_url))
            .title("Discord X NoRiskClient")
            .always_on_top(true)
            .center()
            .max_inner_size(1250.0, 1000.0)
            .build()
            .map_err(|e| {
                CommandError::from(AppError::Other(format!(
                    "Failed to build Discord window: {}",
                    e
                )))
            })?;

    window
        .request_user_attention(Some(UserAttentionType::Critical))
        .map_err(|e| {
            CommandError::from(AppError::Other(format!(
                "Failed to request user attention for Discord window: {}",
                e
            )))
        })?;
    debug!("Discord sign-in window opened.");

    while (Utc::now() - start_time) < ChronoDuration::minutes(10) {
        match window.url().map_err(|e| {
            CommandError::from(AppError::Other(format!(
                "Failed to get Discord window URL: {}",
                e
            )))
        }) {
            Ok(current_url) => {
                let current_url_str = current_url.as_str();
                if current_url_str
                    .starts_with("https://api.norisk.gg/api/v1/core/oauth/discord/complete")
                    || current_url_str.starts_with(
                        "https://api-staging.norisk.gg/api/v1/core/oauth/discord/complete",
                    )
                {
                    debug!("Discord authentication successful, closing window.");
                    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                    window.close().map_err(|e_close| {
                        CommandError::from(AppError::Other(format!(
                            "Failed to close Discord window after auth: {}",
                            e_close
                        )))
                    })?;
                    return Ok(());
                }
            }
            Err(e) => {
                debug!(
                    "Error getting window URL (assuming closed by user): {:?}",
                    e
                );
                return Ok(());
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    debug!("Discord auth timed out after 10 minutes.");
    window.close().map_err(|e_close| {
        CommandError::from(AppError::Other(format!(
            "Failed to close Discord window after timeout: {}",
            e_close
        )))
    })?;
    Ok(())
}

#[tauri::command]
pub async fn discord_auth_status() -> Result<bool, CommandError> {
    debug!("Executing discord_auth_status command");
    let state = State::get().await?;
    let is_experimental = state.config_manager.is_experimental_mode().await;

    let selected_account_arc = state
        .minecraft_account_manager_v2
        .get_active_account()
        .await?
        .ok_or(AppError::AccountError(
            "No active account found for Discord status check.".to_string(),
        ))?;

    let account_id_str = selected_account_arc.id.to_string();
    let norisk_creds = &selected_account_arc.norisk_credentials;
    let token = norisk_creds.get_token_for_mode(is_experimental)?;

    debug!(
        "Checking Discord link status for account {} (experimental: {})",
        account_id_str, is_experimental
    );

    Ok(NoRiskApi::discord_link_status(&token, &account_id_str, is_experimental).await?)
}

#[tauri::command]
pub async fn discord_auth_unlink() -> Result<(), CommandError> {
    debug!("Executing discord_auth_unlink command");
    let state = State::get().await?;
    let is_experimental = state.config_manager.is_experimental_mode().await;

    let selected_account_arc = state
        .minecraft_account_manager_v2
        .get_active_account()
        .await?
        .ok_or(AppError::AccountError(
            "No active account found for Discord unlink.".to_string(),
        ))?;

    let account_id_str = selected_account_arc.id.to_string();
    let norisk_creds = &selected_account_arc.norisk_credentials;
    let token = norisk_creds.get_token_for_mode(is_experimental)?;

    debug!(
        "Unlinking Discord for account {} (experimental: {})",
        account_id_str, is_experimental
    );

    NoRiskApi::unlink_discord(&token, &account_id_str, is_experimental).await?;
    Ok(())
}

#[tauri::command]
pub async fn submit_crash_log_command(payload: CrashlogDto) -> Result<(), CommandError> {
    debug!(
        "Executing submit_crash_log_command with payload: {:?}",
        payload
    );
    let state = State::get().await?;
    let is_experimental = state.config_manager.is_experimental_mode().await;

    let selected_account_arc = state
        .minecraft_account_manager_v2
        .get_active_account()
        .await?
        .ok_or(AppError::AccountError(
            "No active account found for submitting crash log.".to_string(),
        ))?;

    let norisk_creds = &selected_account_arc.norisk_credentials;
    let token = norisk_creds.get_token_for_mode(is_experimental)?;

    debug!(
        "Submitting crash log for account {} (experimental: {}).",
        selected_account_arc.id, is_experimental
    );

    NoRiskApi::submit_crash_log(
        &token,
        &payload,
        &selected_account_arc.id.to_string(),
        is_experimental,
    )
    .await?;
    Ok(())
}

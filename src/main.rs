use clap::Parser;
use reqwest::Url;
use std::{collections::HashMap, fs, str::FromStr};
use univeme::{
    connectors::{
        pprefox::Pprefox,
        windows::{CursorScheme, Windows},
        wpeng::Wpeng,
        Connector,
    },
    toml::Config,
};

pub enum ConnectorConfig {
    Pprefox(Pprefox),
    Windows(Windows),
    Wpeng(Wpeng),
}

/// the universal theme tool
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to config toml
    config: String,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let args = Args::parse();
    let toml_contents = fs::read_to_string(args.config).expect("Config path does not exist.");
    let config: Config = toml::from_str(&toml_contents).unwrap();
    // We will loop through and apply each one soon
    let mut connectors: Vec<ConnectorConfig> = vec![];
    for pprefox in config.pprefox.unwrap_or_default() {
        // Parse endpoint to URL here so we can use Try
        let endpoint = match pprefox.endpoint {
            None => None,
            Some(endpoint) => Some(Url::from_str(&endpoint)?),
        };
        let mut connector = Pprefox {
            endpoint,
            theme_id: None,
        };
        let themes = connector.get_available_themes().await?;
        match themes.get(&pprefox.theme_name) {
            None => panic!("Firefox theme not found: {}", pprefox.theme_name),
            Some(id) => {
                connector.theme_id = Some(id.to_string());
            }
        }
        connectors.push(ConnectorConfig::Pprefox(connector));
    }

    if let Some(windows) = config.windows {
        let mut available_cursors = HashMap::new();
        for system_cursor in Windows::get_system_cursor_schemes().unwrap_or_default() {
            available_cursors.insert(
                system_cursor.clone(),
                CursorScheme::SystemScheme(system_cursor),
            );
        }
        for user_cursor in Windows::get_user_cursor_schemes().unwrap_or_default() {
            available_cursors.insert(user_cursor.clone(), CursorScheme::UserScheme(user_cursor));
        }
        for windows in windows {
            let cursor_scheme = match windows.cursor_scheme {
                None => None,
                Some(cursor_scheme) => match available_cursors.get(&cursor_scheme) {
                    None => todo!(),
                    Some(scheme_with_type) => Some(scheme_with_type.clone()),
                },
            };
            let mut connector = Windows::new()?;
            connector.cursor_scheme = cursor_scheme;
            connectors.push(ConnectorConfig::Windows(connector));
        }
    }
    match config.wpeng {
        None => (),
        Some(wallpapers) => {
            // default desktop_id to 0
            let wallpapers = wallpapers
                .into_iter()
                .map(|wpeng| (wpeng.desktop_id.unwrap_or(0), wpeng.name))
                .collect::<HashMap<u8, Option<String>>>();
            let mut connector = Wpeng::new()?;
            connector.wallpapers = wallpapers;
            connectors.push(ConnectorConfig::Wpeng(connector));
        }
    }
    for connector in connectors {
        match connector {
            ConnectorConfig::Pprefox(pprefox) => {
                pprefox.apply().await?;
            }
            ConnectorConfig::Windows(windows) => {
                windows.apply().await?;
            }
            ConnectorConfig::Wpeng(wpeng) => {
                wpeng.apply().await?;
            }
        }
    }
    Ok(())
}

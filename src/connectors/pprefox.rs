/*
  pprefox-rs connector
  Name: pprefox-rs
  Controls: browser theme

  Config options:
  - Endpoint: Option<String> - The URL where pprefox-rs is listening.
  - Theme ID: Option<String> - The ID of the theme to set. None to leave unchanged.
*/

use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};

use super::Connector;

custom_error::custom_error! {pub PprefoxError
  MissingEndpoint = "No pprefox endpoint selected",
  ExtensionFailure = "Extension request failed"
}

pub struct Pprefox {
    pub endpoint: Option<Url>,
    pub theme_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Theme {
    pub name: String,
    pub id: String,
}

impl Pprefox {
    fn get_endpoint(&self) -> Result<Url, Box<dyn Error + 'static>> {
        match &self.endpoint {
            Some(endpoint) => Ok(endpoint.clone()),
            None => Err(PprefoxError::MissingEndpoint.into()),
        }
    }
    pub async fn get_available_themes(
        &self,
    ) -> Result<HashMap<String, String>, Box<dyn Error + 'static>> {
        let url = self.get_endpoint()?.join("/get_themes")?;
        let resp = reqwest::get(url).await?.json::<Vec<Theme>>().await;
        // Use name as the hash and ID as the value, so we can index by name
        match resp {
            Ok(resp) => Ok(resp
                .iter()
                .map(|f| (f.clone().name, f.clone().id))
                .collect::<HashMap<String, String>>()),
            Err(_) => Err(PprefoxError::ExtensionFailure.into()),
        }
    }
}

#[async_trait::async_trait]
impl Connector for Pprefox {
    fn new() -> Result<Self, Box<dyn Error + 'static>> {
        Ok(Self {
            endpoint: None,
            theme_id: None,
        })
    }
    fn verify(&self) -> Result<(), Box<dyn Error + 'static>> {
        if self.endpoint.is_some() {
            Ok(())
        } else {
            Err(PprefoxError::MissingEndpoint.into())
        }
    }
    async fn apply(&self) -> Result<(), Box<dyn Error + 'static>> {
        match &self.theme_id {
            Some(theme) => {
                let mut url = self.get_endpoint()?.join("/set_theme")?;
                url.set_query(Some(&("id=".to_owned() + theme)));
                reqwest::get(url).await?;
                Ok(())
            }
            None => Ok(()),
        }
    }
}

/*
  ledfx connector
  Name: ledfx
  Controls: LEDs

  Config options:
  - Endpoint: Option<String> - The URL where pprefox-rs is listening.
  - Scene ID: Option<String> - The ID of the scene to enable. None to disable all scenes.
*/

use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};

use super::Connector;

custom_error::custom_error! {pub LedfxError
  MissingEndpoint = "No ledfx endpoint selected",
  ServerFailure = "LEDFX request failed"
}

pub struct Ledfx {
    pub endpoint: Option<Url>,
    pub scene_id: Option<String>,
}

#[derive(Deserialize)]
struct LedfxScene {
    name: String,
}

#[derive(Deserialize)]
struct LedfxResponse {
    #[serde(rename = "status")]
    _status: Option<String>,
    scenes: Option<HashMap<String, LedfxScene>>,
}

#[derive(Serialize)]
struct LedfxRequest {
    pub action: String,
    pub id: String,
}

impl Ledfx {
    fn get_endpoint(&self) -> Result<Url, Box<dyn Error + 'static>> {
        match &self.endpoint {
            Some(endpoint) => Ok(endpoint.clone()),
            None => Err(LedfxError::MissingEndpoint.into()),
        }
    }
    pub async fn get_available_scenes(&self) -> Result<HashMap<String, String>, Box<dyn Error + 'static>> {
        let url = self.get_endpoint()?.join("/api/scenes")?;
        let resp = reqwest::get(url)
            .await?
            .json::<LedfxResponse>()
            .await?
            .scenes
            .ok_or(LedfxError::ServerFailure)?;
        Ok(resp.iter().map(|f| (f.1.name.clone(), f.0.to_string())).collect::<HashMap<String, String>>())
    }
}

#[async_trait::async_trait]
impl Connector for Ledfx {
    fn new() -> Result<Self, Box<dyn Error + 'static>> {
        Ok(Self {
            endpoint: None,
            scene_id: None,
        })
    }
    fn verify(&self) -> Result<(), Box<dyn Error + 'static>> {
        if self.endpoint.is_some() {
            Ok(())
        } else {
            Err(LedfxError::MissingEndpoint.into())
        }
    }
    async fn apply(&self) -> Result<(), Box<dyn Error + 'static>> {
        let url = self.get_endpoint()?.join("/api/scenes")?;
        match &self.scene_id {
            Some(scene) => {
                let request_json = LedfxRequest {
                    action: "activate".to_string(),
                    id: scene.to_string(),
                };
                reqwest::Client::builder()
                    .build()?
                    .put(url)
                    .json(&request_json)
                    .send()
                    .await?;
                Ok(())
            }
            None => {
                // disable all scenes
                let scenes = self.get_available_scenes().await?;
                for (_scene_name, scene_id) in scenes {
                    let request_json = LedfxRequest {
                        action: "deactivate".to_string(),
                        id: scene_id.to_string(),
                    };
                    reqwest::Client::builder()
                        .build()?
                        .put(url.clone())
                        .json(&request_json)
                        .send()
                        .await?;
                }
                Ok(())
            }
        }
    }
}

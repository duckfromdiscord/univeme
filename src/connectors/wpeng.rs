/*
  wpeng-rs connector
  Name: wpeng-rs
  Controls: active wallpaper

  Config options:
  - Wallpapers: Map<u8, Option<String>> - Screen number, name of wallpaper or None for no wallpaper.
*/

const BITS: AutodetectType = AutodetectType::Default64BitMachine;

use std::{collections::HashMap, error::Error, path::PathBuf};

use wpeng_rs::autodetect::{
    autodetect_wallpaper_engine_config, autodetect_wallpaper_engine_exe_from_config,
    autodetect_wallpaper_engine_wallpapers, AutodetectType,
};

use super::Connector;

pub struct Wpeng {
    pub wallpapers: HashMap<u8, Option<String>>,
    pub exe: PathBuf,
}

custom_error::custom_error! {pub WpengError
    ErrorDetectingConfig = "Error detecting wallpaper engine config",
    ErrorDetectingExe = "Error detecting wallpaper engine exe",
    ErrorDetectingWallpapers = "Error detecting wallpaper engine wallpapers",
    ErrorLaunchingProcess = "Error launching wallpaper engine control process",
    MissingWallpaper = "Selected a missing wallpaper"
}

impl Wpeng {
    fn wallpapers_hashmap() -> Result<HashMap<String, String>, WpengError> {
        Ok(autodetect_wallpaper_engine_wallpapers(BITS)
            .ok_or(WpengError::ErrorDetectingWallpapers)?
            .iter()
            .map(|item| (item.clone().name, item.path.to_str().unwrap().to_string()))
            .collect::<HashMap<_, _>>())
    }

    pub fn set_wallpaper(&mut self, screen: u8, wallpaper: Option<String>) {
        match self.wallpapers.get_mut(&screen) {
            Some(item) => {
                *item = wallpaper;
            }
            None => {
                self.wallpapers.insert(screen, wallpaper);
            }
        }
    }
    pub fn get_wallpaper(self, screen: u8) -> Option<Option<String>> {
        self.wallpapers.get(&screen).cloned()
    }
    pub fn get_wallpaper_options(self) -> Result<HashMap<String, String>, WpengError> {
        Wpeng::wallpapers_hashmap()
    }
}

#[async_trait::async_trait]
impl Connector for Wpeng {
    fn new() -> Result<Self, Box<dyn Error + 'static>> {
        let config =
            autodetect_wallpaper_engine_config(BITS).ok_or(WpengError::ErrorDetectingConfig)?;
        let exe = autodetect_wallpaper_engine_exe_from_config(config.clone(), BITS)
            .ok_or(WpengError::ErrorDetectingExe)?;
        Ok(Self {
            wallpapers: HashMap::new(),
            exe,
        })
    }
    fn verify(&self) -> Result<(), Box<dyn Error + 'static>> {
        let _ = autodetect_wallpaper_engine_config(BITS).ok_or(WpengError::ErrorDetectingConfig)?;
        Ok(())
    }
    async fn apply(&self) -> Result<(), Box<dyn Error + 'static>> {
        let available_wallpapers = Wpeng::wallpapers_hashmap()?;
        for (key, value) in self.wallpapers.clone().into_iter() {
            if let Some(name) = value {
                match available_wallpapers.get(&name) {
                    Some(path) => {
                        wpeng_rs::open_wallpaper(
                            self.exe.clone(),
                            path.to_string(),
                            None,
                            Some(key),
                            None,
                            None,
                        )
                        .map_err(|_| WpengError::ErrorLaunchingProcess)?;
                    }
                    None => {
                        return Err(WpengError::MissingWallpaper.into());
                    }
                }
            } else {
                wpeng_rs::remove_wallpaper(self.exe.clone(), None, Some(key))
                    .map_err(|_| WpengError::ErrorLaunchingProcess)?;
            }
        }
        Ok(())
    }
}

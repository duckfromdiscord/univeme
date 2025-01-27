use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub name: Option<String>,
    pub author: Option<String>,
    pub pprefox: Option<Vec<Pprefox>>,
    pub windows: Option<Vec<Windows>>,
    pub wpeng: Option<Vec<Wpeng>>,
    pub ledfx: Option<Vec<Ledfx>>,
}

#[derive(Deserialize, Debug)]
pub struct Pprefox {
    pub comment: Option<String>,
    pub endpoint: Option<String>,
    // Since we do not have a real default in Firefox, there will be no `None` for resetting
    pub theme_name: String,
}

#[derive(Deserialize, Debug)]
pub struct Windows {
    pub comment: Option<String>,
    pub cursor_scheme: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Wpeng {
    pub comment: Option<String>,
    pub name: Option<String>,
    pub desktop_id: Option<u8>,
}

#[derive(Deserialize, Debug)]
pub struct Ledfx {
    pub comment: Option<String>,
    pub endpoint: Option<String>,
    // `None` to reset
    pub scene_name: Option<String>,
}

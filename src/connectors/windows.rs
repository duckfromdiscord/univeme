/*
  Windows connector
  Name: windows
  Controls: OS, window themes

  Config options:
  - Theme path: Option<PathBuf> - Changes the Windows 11 theme. Same as "Settings - Personalization - Themes." None = do not change
  NOTE: User defined themes end up in C:\Users\<UserName>\AppData\Local\Microsoft\Windows\Themes.
  NOTE 2: Whatever path place here is RUN by this program. Only .theme files are allowed for this reason.
  - Enable color prevalance: Option<bool> - Whether the taskbar, start, etc. should show the accent color. None = do not change
  - Light mode: Option<bool> - Whether Windows 11 apps should use their built in light modes. None = do not change
  NOTE: This will work best if you set all of your applications and websites to "System" theme (which copies the system's theme, set here).
  - Cursor scheme: Option<CursorScheme> - Cursor scheme name/type.
*/

use std::{error::Error, path::PathBuf, process::Command};

use registry::*;
use utfx::U16CString;
use winsafe::co::{SPI, SPIF};

use super::Connector;

const CURSOR_ORDER: [&str; 15] = [
    "Arrow",
    "Help",
    "AppStarting",
    "Wait",
    "Crosshair",
    "IBeam",
    "NWPen",
    "No",
    "SizeNS",
    "SizeWE",
    "SizeNWSE",
    "SizeNESW",
    "SizeAll",
    "UpArrow",
    "Hand",
];

pub struct Windows {
    pub theme_path: Option<PathBuf>,
    pub enable_color_prevalence: Option<bool>,
    pub light_mode: Option<bool>,
    pub cursor_scheme: Option<CursorScheme>,
}

#[derive(Debug, Clone)]
pub enum CursorScheme {
    WindowsDefault,
    UserScheme(String),
    SystemScheme(String),
}

custom_error::custom_error! {pub WindowsError
    SchemeNotFound = "cursor scheme not found"
}

impl Windows {
    fn bool_to_data(input: bool) -> Data {
        if input {
            Data::U32(0x00000001)
        } else {
            Data::U32(0x00000000)
        }
    }
    fn get_system_scheme_key() -> Result<RegKey, Box<dyn Error + 'static>> {
        Ok(Hive::LocalMachine.open(
            r"Software\\Microsoft\\Windows\\CurrentVersion\\Control Panel\\Cursors\\Schemes",
            Security::Read,
        )?)
    }
    fn get_user_scheme_key() -> Result<RegKey, Box<dyn Error + 'static>> {
        Ok(Hive::CurrentUser.open(r"Control Panel\\Cursors\\Schemes", Security::Read)?)
    }
    pub fn get_system_cursor_schemes() -> Result<Vec<String>, Box<dyn Error + 'static>> {
        let system_scheme_key = Windows::get_system_scheme_key()?;
        let system_schemes = system_scheme_key
            .values()
            .map(|value_option| value_option.unwrap().into_name().to_string().unwrap())
            .collect::<Vec<_>>();
        Ok(system_schemes)
    }
    pub fn get_user_cursor_schemes() -> Result<Vec<String>, Box<dyn Error + 'static>> {
        let user_scheme_key = Windows::get_user_scheme_key()?;
        let user_schemes = user_scheme_key
            .values()
            .map(|value_option| value_option.unwrap().into_name().to_string().unwrap())
            .collect::<Vec<_>>();
        Ok(user_schemes)
    }
}

#[async_trait::async_trait]
impl Connector for Windows {
    fn new() -> Result<Self, Box<dyn Error + 'static>> {
        Ok(Self {
            theme_path: None,
            enable_color_prevalence: None,
            light_mode: None,
            cursor_scheme: None,
        })
    }
    fn verify(&self) -> Result<(), Box<dyn Error + 'static>> {
        // Check if we have access to registry
        let _ = Hive::CurrentUser.open(
            r"Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
            Security::Read | Security::Write,
        )?;
        Ok(())
    }
    async fn apply(&self) -> Result<(), Box<dyn Error + 'static>> {
        let personalize = Hive::CurrentUser.open(
            r"Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
            Security::Read | Security::Write,
        )?;
        if let Some(light_mode) = self.light_mode {
            personalize.set_value("AppsUseLightTheme", &Windows::bool_to_data(light_mode))?;
            personalize.set_value("SystemUsesLightTheme", &Windows::bool_to_data(light_mode))?;
        }
        if let Some(enable_color_prevalance) = self.enable_color_prevalence {
            personalize.set_value(
                "ColorPrevalence",
                &Windows::bool_to_data(enable_color_prevalance),
            )?;
        }
        //let accent = Hive::CurrentUser.open(r"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Accent", Security::Read | Security::Write)?;
        if let Some(theme_path) = &self.theme_path {
            if theme_path
                .extension()
                .map(|ext| ext.to_str().unwrap_or(""))
                .unwrap_or("")
                == "theme"
            {
                // Execute the theme file
                Command::new("cmd")
                    .arg("/C")
                    .arg(theme_path.as_os_str())
                    .output()?;
            }
        }
        if let Some(cursor_scheme) = &self.cursor_scheme {
            let cursor_key = Hive::CurrentUser
                .open(r"Control Panel\\Cursors", Security::Read | Security::Write)?;
            // Scheme source info from https://thebitguru.com/articles/programmatically-changing-windows-mouse-cursors/3
            let cursors: Option<Vec<String>>;
            match cursor_scheme {
                CursorScheme::WindowsDefault => {
                    // Scheme Source varies depending on where the cursor scheme comes from
                    cursor_key.set_value("Scheme Source", &Data::U32(0))?;
                    // Windows Default has nothing/empty REG_SZ in (Default)
                    cursor_key.set_value("", &Data::String(U16CString::from_str("")?))?;
                    cursors = None;
                }
                CursorScheme::UserScheme(name) => {
                    cursor_key.set_value("Scheme Source", &Data::U32(1))?;
                    cursor_key.set_value("", &Data::String(U16CString::from_str(name)?))?;
                    match Windows::get_user_scheme_key()?.value(U16CString::from_str(name)?)? {
                        Data::String(data) => {
                            cursors = Some(
                                data.to_string()
                                    .unwrap()
                                    .split(",")
                                    .map(|f| f.to_string())
                                    .collect::<Vec<_>>(),
                            );
                        }
                        _ => return Err(WindowsError::SchemeNotFound.into()),
                    }
                }
                CursorScheme::SystemScheme(name) => {
                    cursor_key.set_value("Scheme Source", &Data::U32(2))?;
                    cursor_key.set_value("", &Data::String(U16CString::from_str(name)?))?;
                    match Windows::get_system_scheme_key()?.value(U16CString::from_str(name)?)? {
                        Data::String(data) => {
                            cursors = Some(
                                data.to_string()
                                    .unwrap()
                                    .split(",")
                                    .map(|f| f.to_string())
                                    .collect::<Vec<_>>(),
                            );
                        }
                        _ => return Err(WindowsError::SchemeNotFound.into()),
                    }
                }
            };
            let cursors = cursors.unwrap_or_default();
            for (i, item) in CURSOR_ORDER.iter().enumerate() {
                cursor_key.set_value(
                    *item,
                    // if the cursor is not specified, use default of blank which will use system default
                    &Data::ExpandString(U16CString::from_str(cursors.get(i).map_or("", |v| v))?),
                )?;
            }
            unsafe {
                winsafe::SystemParametersInfo(SPI::SETCURSORS, 0, &mut 0, SPIF::NoValue)?;
            }
        }
        Ok(())
    }
}

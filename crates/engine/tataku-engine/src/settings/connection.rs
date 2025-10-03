use crate::*;
use common::reflect::*;
use tataku_client_proc_macros::Settings;

#[derive(Reflect, Settings)]
#[reflect(display="display")]
#[allow(clippy::manual_non_exhaustive)]
#[derive(Serialize, DeserializeSettings)]
#[derive(Clone, Debug2, Default2, PartialEq)]
#[serde(default)]
pub struct ConnectionSettingsProfile {
    #[default("Default".into())]
    #[setting(text="Connection Profile")]
    pub profile_name: String,

    #[default("Guest".into())]
    #[serde(alias="username")]
    #[setting(text="Tataku Username")]
    pub tataku_username: String,

    #[serde(alias="password")]
    #[setting(text="Tataku Password", password=true)]
    pub tataku_password: String,

    #[default("wss://server.tataku.ca".into())]
    #[setting(text="Tataku Server Url")]
    pub server_url: String,

    #[default("https://scores.tataku.ca".into())]
    #[setting(text="Tataku Score Url")]
    pub score_url: String,
}
impl ConnectionSettingsProfile {
    pub fn changed(&self, other: &Self) -> bool {
        self.score_url != other.score_url 
        || self.server_url != other.server_url
        || self.tataku_username != other.tataku_username
        || self.tataku_password != other.tataku_password
    }
}
impl std::fmt::Display for ConnectionSettingsProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.profile_name.fmt(f)
    }
}

#[derive(Reflect)]
#[allow(clippy::manual_non_exhaustive)]
#[derive(Serialize, DeserializeSettings)]
#[derive(Clone, Debug2, Default2, PartialEq)]
#[serde(default)]
pub struct ConnectionSettings {
    #[default(vec![ConnectionSettingsProfile::default()])]
    pub profiles: Vec<ConnectionSettingsProfile>,
    pub current: ConnectionSettingsProfile,
}

impl settings::MakeSettingsMenu for ConnectionSettings {
    fn create_provider(
        &self, 
        prefix: String,
        builder: &mut settings::SettingsBuilder,
    ) {
        use settings::buildable_settings_provider::*;
        builder.add_item(BuildableSetting {
            name: "Connection Profile".into(),
            path: prefix.clone() + ".current",
            tooltip: None,
            enabled_if: None,
            visible_if: None,
            setting_type: BuildableSettingType::Dropdown { 
                options: BuildableSettingDropdownOptions::Variable {
                    var: prefix.clone() + ".profiles",
                }
            },
        });

        self.current.create_provider(prefix.clone() + ".current", builder);

        let save_action = actions::game::GameAction::UpdateSettings(
            Arc::new(|settings| {
                let conn_settings = &mut settings.connection_settings;

                let current = conn_settings.current.clone();
                for i in conn_settings.profiles.iter_mut() {
                    if i.profile_name == current.profile_name {
                        *i = current;
                        return
                    }
                }

                conn_settings.profiles.push(current);
            })
        );
        builder.add_item(BuildableSetting {
            name: "Save".into(),
            path: prefix.clone() + "",
            tooltip: None,
            enabled_if: None,
            visible_if: None,
            setting_type: BuildableSettingType::Button {
                action: BuildableSettingsAction {
                    inner: Arc::new(actions::Action::from(save_action))
                }
            },
        });

    }
}

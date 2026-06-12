#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueSource {
    Default,
    Environment,
    EnvFile,
    TomlFile,
    IniFile,
    JsonFile,
    YamlFile,
}

impl std::fmt::Display for ValueSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueSource::Default => write!(f, "default"),
            ValueSource::Environment => write!(f, "environment"),
            ValueSource::EnvFile => write!(f, ".env file"),
            ValueSource::TomlFile => write!(f, ".toml file"),
            ValueSource::IniFile => write!(f, ".ini file"),
            ValueSource::JsonFile => write!(f, ".json file"),
            ValueSource::YamlFile => write!(f, ".yaml file"),
        }
    }
}

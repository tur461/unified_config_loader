#[derive(Debug, Clone, Copy)]
pub enum ValueSource {
    Default,
    Environment,
    File,
}

impl std::fmt::Display for ValueSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueSource::Default => write!(f, "default"),
            ValueSource::Environment => write!(f, "environment"),
            ValueSource::File => write!(f, "file"),
        }
    }
}

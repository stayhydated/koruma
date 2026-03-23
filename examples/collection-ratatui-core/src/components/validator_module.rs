use koruma::showcase::ValidatorShowcase;

/// Validator module enumeration representing different categories of validators
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidatorModule {
    String,
    Format,
    Numeric,
    Collection,
    General,
}

impl ValidatorModule {
    /// All available validator modules
    pub const ALL: [Self; 5] = [
        Self::String,
        Self::Format,
        Self::Numeric,
        Self::Collection,
        Self::General,
    ];

    /// Get available modules based on registered validators
    pub fn available_modules(all_validators: &[&'static ValidatorShowcase]) -> Vec<Self> {
        Self::ALL
            .iter()
            .filter(|&&m| all_validators.iter().any(|&v| m.contains_validator(v)))
            .copied()
            .collect()
    }

    /// Get the display name of the module
    pub fn name(&self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Format => "Format",
            Self::Numeric => "Numeric",
            Self::Collection => "Collection",
            Self::General => "General",
        }
    }

    /// Check if the module contains a specific validator
    pub fn contains_validator(&self, showcase: &ValidatorShowcase) -> bool {
        match self {
            Self::String => showcase.module == "string",
            Self::Format => showcase.module == "format",
            Self::Numeric => showcase.module == "numeric",
            Self::Collection => showcase.module == "collection",
            Self::General => showcase.module == "general",
        }
    }
}

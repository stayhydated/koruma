use std::sync::OnceLock;

use es_fluent::EsFluent;
use es_fluent_manager_embedded::EmbeddedI18n;
use unic_langid::{LanguageIdentifier, langid};

es_fluent_manager_dioxus::define_i18n_module!();

static KORUMA_LOCALIZER: OnceLock<Result<EmbeddedI18n, String>> = OnceLock::new();

/// The former demo locale used SP for Serbia. Accept it only at the app boundary.
pub(crate) fn normalize_demo_language(language: &LanguageIdentifier) -> LanguageIdentifier {
    if *language == langid!("sr-SP") {
        langid!("sr-RS")
    } else {
        language.clone()
    }
}

#[cfg(test)]
pub(crate) static KORUMA_LOCALE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn koruma_localizer_for(language: &LanguageIdentifier) -> Option<&'static EmbeddedI18n> {
    let localizer = KORUMA_LOCALIZER
        .get_or_init(|| EmbeddedI18n::try_new().map_err(|error| error.to_string()))
        .as_ref()
        .ok()?;

    localizer
        .select_language(normalize_demo_language(language))
        .ok()?;
    Some(localizer)
}

#[iso_types::iso_locale]
pub(crate) enum DemoLanguage {}

#[derive(Clone, Copy, Debug, EsFluent)]
pub(crate) enum DioxusShowcaseMessage {
    ShowcasePanelLabel,
    ShowcaseIntroTitle,
    ShowcaseIntroBody,
    ModuleString,
    ModuleFormat,
    ModuleNumeric,
    ModuleCollection,
    ModuleGeneral,
    ValidationPlaceholder,
    MessageHeadingResult,
    MessageHeadingError,
    ErrorPrefix,
    ValidationPassed,
    LanguageLabel,
    SearchLanguagesPlaceholder,
    NoMatchingLanguages,
}

#[derive(Clone, Copy, Debug, EsFluent)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum SalesFormMessage {
    PanelLabel,
    IntroTitle,
    IntroBody,
    CompanyLabel,
    CompanyPlaceholder,
    ContactNameLabel,
    ContactNamePlaceholder,
    EmailLabel,
    EmailPlaceholder,
    PhoneLabel,
    PhonePlaceholder,
    PhoneHint,
    DealValueLabel,
    DealValuePlaceholder,
    StageLabel,
    StagePlaceholder,
    SourceUrlLabel,
    SourceUrlPlaceholder,
    SourceUrlHint,
    NextStepLabel,
    NextStepPlaceholder,
    ValidSampleAction,
    InvalidSampleAction,
    ClearAction,
    SubmitAction,
    SubmittedAction,
    SummarySubmittedTitle,
    SummarySubmittedBody,
    StageDiscovery,
    StageQualified,
    StageProposal,
    StageProcurement,
    FieldStatusValid,
    FieldStatusInvalid,
    FieldStatusOptional,
    SummaryTitle,
    SummaryValidTitle,
    SummaryInvalidTitle,
    SummaryValidBody,
    SummaryInvalidBody,
    SummaryProgressLabel,
    RulesTitle,
    RuleRequired,
    RuleOptional,
    FieldCompany,
    FieldContactName,
    FieldEmail,
    FieldPhone,
    FieldDealValue,
    FieldStage,
    FieldSourceUrl,
    FieldNextStep,
}

#[derive(Clone, Copy, Debug, EsFluent)]
pub(crate) enum ShowcaseValidatorMessage {
    AlphanumericName,
    AlphanumericDescription,
    AsciiName,
    AsciiDescription,
    CanonicalFormName,
    CanonicalFormDescription,
    ContainsName,
    ContainsDescription,
    MatchesName,
    MatchesDescription,
    PatternName,
    PatternDescription,
    PrefixName,
    PrefixDescription,
    SuffixName,
    SuffixDescription,
    CreditCardName,
    CreditCardDescription,
    EmailName,
    EmailDescription,
    IpName,
    IpDescription,
    PhoneName,
    PhoneDescription,
    UrlName,
    UrlDescription,
    NegativeName,
    NegativeDescription,
    NonNegativeName,
    NonNegativeDescription,
    NonPositiveName,
    NonPositiveDescription,
    PositiveName,
    PositiveDescription,
    RangeName,
    RangeDescription,
    LengthName,
    LengthDescription,
    NonEmptyName,
    NonEmptyDescription,
}

#[cfg(test)]
mod tests {
    use es_fluent::FluentLocalizerExt as _;
    use koruma_collection::collection::NonEmptyValidation;
    use unic_langid::langid;

    use super::{DemoLanguage, koruma_localizer_for, normalize_demo_language};

    #[test]
    fn koruma_localizer_tracks_demo_language() {
        let _locale_lock = super::KORUMA_LOCALE_TEST_LOCK.lock().unwrap();
        let validator = NonEmptyValidation::<String>::with_value(String::new()).build();

        let en = koruma_localizer_for(&langid!("en-US")).expect("en-US should be supported");
        assert_eq!(
            en.try_localize_message(&validator).as_deref(),
            Some("Must not be empty.")
        );

        let fr = koruma_localizer_for(&langid!("fr-FR")).expect("fr-FR should be supported");
        assert_eq!(
            fr.try_localize_message(&validator).as_deref(),
            Some("Ne doit pas être vide.")
        );

        let zh = koruma_localizer_for(&langid!("zh-CN")).expect("zh-CN should be supported");
        assert_eq!(
            zh.try_localize_message(&validator).as_deref(),
            Some("\u{4e0d}\u{80fd}\u{4e3a}\u{7a7a}\u{3002}")
        );
    }
    #[test]
    fn legacy_serbian_maps_to_a_supported_demo_locale() {
        let language = normalize_demo_language(&langid!("sr-SP"));
        assert_eq!(language, langid!("sr-RS"));
        assert_eq!(DemoLanguage::try_from(&language).unwrap().code(), "sr-RS");
    }
}

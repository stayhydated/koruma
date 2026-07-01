use std::sync::OnceLock;

use es_fluent::EsFluent;
use es_fluent_lang::es_fluent_language;
use es_fluent_manager_embedded::EmbeddedI18n;
use strum::EnumIter;
use unic_langid::LanguageIdentifier;

es_fluent_manager_dioxus::define_i18n_module!();

static KORUMA_LOCALIZER: OnceLock<Result<EmbeddedI18n, String>> = OnceLock::new();

pub(crate) fn koruma_localizer_for(language: &LanguageIdentifier) -> Option<&'static EmbeddedI18n> {
    let localizer = KORUMA_LOCALIZER
        .get_or_init(|| EmbeddedI18n::try_new().map_err(|error| error.to_string()))
        .as_ref()
        .ok()?;

    localizer.select_language(language.clone()).ok()?;
    Some(localizer)
}

#[es_fluent_language]
#[derive(Clone, Copy, Debug, EnumIter, Eq, EsFluent, PartialEq)]
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

#[cfg(test)]
mod tests {
    use es_fluent::FluentLocalizerExt as _;
    use koruma_collection::collection::NonEmptyValidation;
    use unic_langid::langid;

    use super::koruma_localizer_for;

    #[test]
    fn koruma_localizer_tracks_demo_language() {
        let validator = NonEmptyValidation::<String>::with_value(String::new()).build();

        let en = koruma_localizer_for(&langid!("en-US")).expect("en-US should be supported");
        assert_eq!(
            en.try_localize_message(&validator).as_deref(),
            Some("Must not be empty.")
        );

        let fr = koruma_localizer_for(&langid!("fr-FR")).expect("fr-FR should be supported");
        assert_eq!(
            fr.try_localize_message(&validator).as_deref(),
            Some("Ne doit pas \u{00ea}tre vide.")
        );

        let zh = koruma_localizer_for(&langid!("zh-CN")).expect("zh-CN should be supported");
        assert_eq!(
            zh.try_localize_message(&validator).as_deref(),
            Some("\u{4e0d}\u{80fd}\u{4e3a}\u{7a7a}\u{3002}")
        );
    }
}

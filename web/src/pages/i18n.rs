use es_fluent::EsFluent;
use es_fluent_lang::es_fluent_language;
use strum::EnumIter;

es_fluent_manager_dioxus::define_i18n_module!();

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

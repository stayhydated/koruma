use es_fluent::EsFluent;
use es_fluent_lang::es_fluent_language;
use strum::EnumIter;

es_fluent_manager_dioxus::define_i18n_module!();

#[es_fluent_language]
#[derive(Clone, Copy, Debug, EnumIter, Eq, EsFluent, PartialEq)]
pub(crate) enum SiteLanguage {}

#[derive(Clone, Copy, Debug, EsFluent)]
pub(crate) enum HomeMessage {
    HeroEyebrow,
    HeroTitle,
    HeroBody,
    HeroPrimaryAction,
    HeroSecondaryAction,
    WorkflowPanelLabel,
    WorkflowStepDefineTitle,
    WorkflowStepDefineBody,
    WorkflowStepAttachTitle,
    WorkflowStepAttachBody,
    WorkflowStepInspectTitle,
    WorkflowStepInspectBody,
    SurfacePanelLabel,
    SurfaceTitle,
    SurfaceDescribeTitle,
    SurfaceDescribeBody,
    SurfaceReusableTitle,
    SurfaceReusableBody,
    SurfaceI18nTitle,
    SurfaceI18nBody,
}

#[derive(Clone, Copy, Debug, EsFluent)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum DemosPageMessage {
    DioxusLabel,
    DioxusTitle,
    DioxusBody,
    DioxusAction,
    SalesLabel,
    SalesTitle,
    SalesBody,
    SalesAction,
}

#[derive(Clone, Copy, Debug, EsFluent)]
pub(crate) enum ContributeMessage {
    Label,
    Headline,
    BodyPrefix,
    BodyProjectFluent,
    BodyCrowdin,
    BodyGithub,
    FooterDot,
}

#[derive(Clone, Copy, Debug, EsFluent)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum SiteFooterMessage {
    CratesLabel,
    CratesTextPrefix,
    CratesTextMiddle,
    CratesTextSuffix,
}

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

#[derive(Clone, Copy, Debug, EsFluent)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum PageMetadataMessage {
    HomeTitle,
    HomeDescription,
    DemosTitle,
    DemosDescription,
    DioxusDemoTitle,
    DioxusDemoDescription,
    SalesFormDemoTitle,
    SalesFormDemoDescription,
}

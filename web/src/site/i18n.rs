use es_fluent::EsFluent;
use es_fluent_lang::{LanguageIdentifier, es_fluent_language};
use strum::{EnumIter, IntoEnumIterator as _};

es_fluent_manager_dioxus::define_i18n_module!();

#[es_fluent_language]
#[derive(Clone, Copy, Debug, EnumIter, Eq, EsFluent, PartialEq)]
pub(crate) enum SiteLanguage {}

impl SiteLanguage {
    pub(crate) fn all() -> impl Iterator<Item = Self> {
        Self::iter()
    }

    pub(crate) fn lang(self) -> LanguageIdentifier {
        self.into()
    }
}

#[derive(Clone, Copy, Debug, EsFluent)]
pub(crate) enum SiteChromeMessage {
    BrandKicker,
    SiteName,
    NavHome,
    NavDemos,
    NavDocs,
    NavSource,
    LocaleLabel,
}

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
pub(crate) enum PageMetadataMessage {
    HomeTitle,
    HomeDescription,
    DemosTitle,
    DemosDescription,
    DioxusDemoTitle,
    DioxusDemoDescription,
}

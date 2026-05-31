use std::fmt::Display;

use dioxus::events::FormData;
use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdClipboardCheck, LdRotateCcw, LdSend, LdTriangleAlert},
};
use dioxus_primitives::label::Label;
use es_fluent_manager_dioxus::use_i18n;
use koruma::{Koruma, KorumaAllDisplay};
use koruma_collection::{collection, format, general, numeric};

use crate::components::{FooterPanel, PageHeader};
use crate::site::i18n::SalesFormMessage;
use crate::site::routing::PageKind;

const SALES_STAGES: &[&str] = &["Discovery", "Qualified", "Proposal", "Procurement"];
const SALES_FIELD_COUNT: usize = 8;

#[derive(Clone, Debug, Koruma, KorumaAllDisplay)]
struct SalesLeadForm {
    #[koruma(
        collection::NonEmptyValidation::<_>,
        collection::LenValidation::<_>::min(2).max(80)
    )]
    company: String,

    #[koruma(
        collection::NonEmptyValidation::<_>,
        collection::LenValidation::<_>::min(2).max(80)
    )]
    contact_name: String,

    #[koruma(collection::NonEmptyValidation::<_>, format::EmailValidation::<_>)]
    email: String,

    #[koruma(format::PhoneNumberValidation::<_>)]
    phone: Option<String>,

    #[koruma(
        general::RequiredValidation::<_>,
        numeric::RangeValidation::<_>::min(1_000.0_f64).max(500_000.0_f64)
    )]
    deal_value: Option<f64>,

    #[koruma(collection::NonEmptyValidation::<_>)]
    stage: String,

    #[koruma(format::UrlValidation::<_>)]
    source_url: Option<String>,

    #[koruma(
        collection::NonEmptyValidation::<_>,
        collection::LenValidation::<_>::min(4).max(160)
    )]
    next_step: String,
}

#[derive(bon::Builder, Clone, Debug, Eq, PartialEq)]
#[builder(on(String, into))]
struct SalesLeadDraft {
    company: String,
    contact_name: String,
    email: String,
    phone: String,
    deal_value: String,
    stage: String,
    source_url: String,
    next_step: String,
}

impl SalesLeadDraft {
    fn valid_sample() -> Self {
        Self::builder()
            .company("Northwind Robotics")
            .contact_name("Ava Patel")
            .email("ava.patel@northwind.example")
            .phone("+14155552671")
            .deal_value("82000")
            .stage("Proposal")
            .source_url("https://northwind.example/security-review")
            .next_step("Send procurement package before Friday.")
            .build()
    }

    fn invalid_sample() -> Self {
        Self::builder()
            .company("")
            .contact_name("A")
            .email("vp@")
            .phone("555")
            .deal_value("250")
            .stage("")
            .source_url("northwind")
            .next_step("")
            .build()
    }

    fn empty() -> Self {
        Self::builder()
            .company("")
            .contact_name("")
            .email("")
            .phone("")
            .deal_value("")
            .stage("")
            .source_url("")
            .next_step("")
            .build()
    }
}

impl Default for SalesLeadDraft {
    fn default() -> Self {
        Self::invalid_sample()
    }
}

impl From<&SalesLeadDraft> for SalesLeadForm {
    fn from(draft: &SalesLeadDraft) -> Self {
        Self {
            company: draft.company.trim().to_string(),
            contact_name: draft.contact_name.trim().to_string(),
            email: draft.email.trim().to_string(),
            phone: optional_trimmed(&draft.phone),
            deal_value: parse_deal_value(&draft.deal_value),
            stage: draft.stage.trim().to_string(),
            source_url: optional_trimmed(&draft.source_url),
            next_step: draft.next_step.trim().to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct SalesFeedback {
    company: Vec<String>,
    contact_name: Vec<String>,
    email: Vec<String>,
    phone: Vec<String>,
    deal_value: Vec<String>,
    stage: Vec<String>,
    source_url: Vec<String>,
    next_step: Vec<String>,
}

impl SalesFeedback {
    fn is_valid(&self) -> bool {
        self.invalid_count() == 0
    }

    fn invalid_count(&self) -> usize {
        [
            &self.company,
            &self.contact_name,
            &self.email,
            &self.phone,
            &self.deal_value,
            &self.stage,
            &self.source_url,
            &self.next_step,
        ]
        .into_iter()
        .filter(|errors| !errors.is_empty())
        .count()
    }

    fn valid_count(&self) -> usize {
        SALES_FIELD_COUNT.saturating_sub(self.invalid_count())
    }

    fn issue_groups(&self, labels: &SalesFieldLabels) -> Vec<(String, Vec<String>)> {
        [
            (labels.company.clone(), self.company.clone()),
            (labels.contact_name.clone(), self.contact_name.clone()),
            (labels.email.clone(), self.email.clone()),
            (labels.phone.clone(), self.phone.clone()),
            (labels.deal_value.clone(), self.deal_value.clone()),
            (labels.stage.clone(), self.stage.clone()),
            (labels.source_url.clone(), self.source_url.clone()),
            (labels.next_step.clone(), self.next_step.clone()),
        ]
        .into_iter()
        .filter(|(_, errors)| !errors.is_empty())
        .collect()
    }
}

#[derive(bon::Builder, Clone)]
#[builder(on(String, into))]
struct SalesFieldLabels {
    company: String,
    contact_name: String,
    email: String,
    phone: String,
    deal_value: String,
    stage: String,
    source_url: String,
    next_step: String,
}

#[derive(Clone, Copy)]
enum FieldStatus {
    Valid,
    Invalid,
    Optional,
}

impl FieldStatus {
    fn class(self) -> &'static str {
        match self {
            Self::Valid => "is-valid",
            Self::Invalid => "is-invalid",
            Self::Optional => "is-optional",
        }
    }
}

#[component]
pub(crate) fn SalesFormPage() -> Element {
    let i18n = match use_i18n() {
        Ok(i18n) => i18n,
        Err(error) => {
            return rsx! {
                div { class: "page-shell",
                    PageHeader { current_page: PageKind::SalesForm }
                    main { class: "stack",
                        section { class: "page-title-band",
                            span { class: "panel-label", "i18n load failure" }
                            h1 { "Sales form demo" }
                            p { "Failed to initialize i18n: {error}" }
                        }
                    }
                    FooterPanel {}
                }
            };
        },
    };

    let mut draft = use_signal(SalesLeadDraft::default);
    let current = draft.read().clone();
    let feedback = validate_sales_draft(&current);
    let field_labels = SalesFieldLabels::builder()
        .company(i18n.localize_message(&SalesFormMessage::FieldCompany))
        .contact_name(i18n.localize_message(&SalesFormMessage::FieldContactName))
        .email(i18n.localize_message(&SalesFormMessage::FieldEmail))
        .phone(i18n.localize_message(&SalesFormMessage::FieldPhone))
        .deal_value(i18n.localize_message(&SalesFormMessage::FieldDealValue))
        .stage(i18n.localize_message(&SalesFormMessage::FieldStage))
        .source_url(i18n.localize_message(&SalesFormMessage::FieldSourceUrl))
        .next_step(i18n.localize_message(&SalesFormMessage::FieldNextStep))
        .build();
    let issue_groups = feedback.issue_groups(&field_labels);
    let valid_fields = feedback.valid_count();
    let is_valid = feedback.is_valid();

    let status_valid = i18n.localize_message(&SalesFormMessage::FieldStatusValid);
    let status_invalid = i18n.localize_message(&SalesFormMessage::FieldStatusInvalid);
    let status_optional = i18n.localize_message(&SalesFormMessage::FieldStatusOptional);
    let title_style = crate::components::use_reveal_style(0, 24.0);
    let demo_style = crate::components::use_reveal_style(90, 18.0);

    rsx! {
        div { class: "page-shell",
            PageHeader { current_page: PageKind::SalesForm }
            main { class: "stack",
                section { class: "page-title-band motion-reveal",
                    style: title_style,
                    span { class: "panel-label",
                        "{i18n.localize_message(&SalesFormMessage::PanelLabel)}"
                    }
                    h1 { "{i18n.localize_message(&SalesFormMessage::IntroTitle)}" }
                    p { "{i18n.localize_message(&SalesFormMessage::IntroBody)}" }
                }
                section { class: "sales-demo-shell motion-reveal",
                    style: demo_style,
                    div { class: "sales-form-panel",
                        div { class: "sales-action-row",
                            button {
                                class: "sales-sample-button",
                                r#type: "button",
                                onclick: move |_| draft.set(SalesLeadDraft::valid_sample()),
                                Icon {
                                    class: "sales-button-icon".to_string(),
                                    width: 17,
                                    height: 17,
                                    icon: LdClipboardCheck,
                                }
                                "{i18n.localize_message(&SalesFormMessage::ValidSampleAction)}"
                            }
                            button {
                                class: "sales-sample-button",
                                r#type: "button",
                                onclick: move |_| draft.set(SalesLeadDraft::invalid_sample()),
                                Icon {
                                    class: "sales-button-icon".to_string(),
                                    width: 17,
                                    height: 17,
                                    icon: LdTriangleAlert,
                                }
                                "{i18n.localize_message(&SalesFormMessage::InvalidSampleAction)}"
                            }
                            button {
                                class: "sales-sample-button",
                                r#type: "button",
                                onclick: move |_| draft.set(SalesLeadDraft::empty()),
                                Icon {
                                    class: "sales-button-icon".to_string(),
                                    width: 17,
                                    height: 17,
                                    icon: LdRotateCcw,
                                }
                                "{i18n.localize_message(&SalesFormMessage::ClearAction)}"
                            }
                        }
                        div { class: "sales-form-grid",
                            {sales_text_field(
                                "sales-company",
                                i18n.localize_message(&SalesFormMessage::CompanyLabel),
                                current.company.clone(),
                                i18n.localize_message(&SalesFormMessage::CompanyPlaceholder),
                                "text",
                                field_status(&feedback.company, &current.company, false),
                                status_text(field_status(&feedback.company, &current.company, false), &status_valid, &status_invalid, &status_optional),
                                feedback.company.clone(),
                                None,
                                EventHandler::new(move |event: Event<FormData>| {
                                    draft.write().company = event.value();
                                }),
                            )}
                            {sales_text_field(
                                "sales-contact",
                                i18n.localize_message(&SalesFormMessage::ContactNameLabel),
                                current.contact_name.clone(),
                                i18n.localize_message(&SalesFormMessage::ContactNamePlaceholder),
                                "text",
                                field_status(&feedback.contact_name, &current.contact_name, false),
                                status_text(field_status(&feedback.contact_name, &current.contact_name, false), &status_valid, &status_invalid, &status_optional),
                                feedback.contact_name.clone(),
                                None,
                                EventHandler::new(move |event: Event<FormData>| {
                                    draft.write().contact_name = event.value();
                                }),
                            )}
                            {sales_text_field(
                                "sales-email",
                                i18n.localize_message(&SalesFormMessage::EmailLabel),
                                current.email.clone(),
                                i18n.localize_message(&SalesFormMessage::EmailPlaceholder),
                                "email",
                                field_status(&feedback.email, &current.email, false),
                                status_text(field_status(&feedback.email, &current.email, false), &status_valid, &status_invalid, &status_optional),
                                feedback.email.clone(),
                                None,
                                EventHandler::new(move |event: Event<FormData>| {
                                    draft.write().email = event.value();
                                }),
                            )}
                            {sales_text_field(
                                "sales-phone",
                                i18n.localize_message(&SalesFormMessage::PhoneLabel),
                                current.phone.clone(),
                                i18n.localize_message(&SalesFormMessage::PhonePlaceholder),
                                "tel",
                                field_status(&feedback.phone, &current.phone, true),
                                status_text(field_status(&feedback.phone, &current.phone, true), &status_valid, &status_invalid, &status_optional),
                                feedback.phone.clone(),
                                Some(i18n.localize_message(&SalesFormMessage::PhoneHint)),
                                EventHandler::new(move |event: Event<FormData>| {
                                    draft.write().phone = event.value();
                                }),
                            )}
                            {sales_text_field(
                                "sales-deal-value",
                                i18n.localize_message(&SalesFormMessage::DealValueLabel),
                                current.deal_value.clone(),
                                i18n.localize_message(&SalesFormMessage::DealValuePlaceholder),
                                "number",
                                field_status(&feedback.deal_value, &current.deal_value, false),
                                status_text(field_status(&feedback.deal_value, &current.deal_value, false), &status_valid, &status_invalid, &status_optional),
                                feedback.deal_value.clone(),
                                None,
                                EventHandler::new(move |event: Event<FormData>| {
                                    draft.write().deal_value = event.value();
                                }),
                            )}
                            {sales_select_field(
                                "sales-stage",
                                i18n.localize_message(&SalesFormMessage::StageLabel),
                                current.stage.clone(),
                                i18n.localize_message(&SalesFormMessage::StagePlaceholder),
                                field_status(&feedback.stage, &current.stage, false),
                                status_text(field_status(&feedback.stage, &current.stage, false), &status_valid, &status_invalid, &status_optional),
                                feedback.stage.clone(),
                                EventHandler::new(move |event: Event<FormData>| {
                                    draft.write().stage = event.value();
                                }),
                            )}
                            {sales_text_field(
                                "sales-source-url",
                                i18n.localize_message(&SalesFormMessage::SourceUrlLabel),
                                current.source_url.clone(),
                                i18n.localize_message(&SalesFormMessage::SourceUrlPlaceholder),
                                "url",
                                field_status(&feedback.source_url, &current.source_url, true),
                                status_text(field_status(&feedback.source_url, &current.source_url, true), &status_valid, &status_invalid, &status_optional),
                                feedback.source_url.clone(),
                                Some(i18n.localize_message(&SalesFormMessage::SourceUrlHint)),
                                EventHandler::new(move |event: Event<FormData>| {
                                    draft.write().source_url = event.value();
                                }),
                            )}
                            {sales_text_area(
                                "sales-next-step",
                                i18n.localize_message(&SalesFormMessage::NextStepLabel),
                                current.next_step.clone(),
                                i18n.localize_message(&SalesFormMessage::NextStepPlaceholder),
                                field_status(&feedback.next_step, &current.next_step, false),
                                status_text(field_status(&feedback.next_step, &current.next_step, false), &status_valid, &status_invalid, &status_optional),
                                feedback.next_step.clone(),
                                EventHandler::new(move |event: Event<FormData>| {
                                    draft.write().next_step = event.value();
                                }),
                            )}
                        }
                    }
                    aside { class: "sales-summary-panel",
                        div { class: if is_valid { "sales-summary-state is-valid" } else { "sales-summary-state is-invalid" },
                            span { class: "panel-label",
                                "{i18n.localize_message(&SalesFormMessage::SummaryTitle)}"
                            }
                            h2 {
                                if is_valid {
                                    "{i18n.localize_message(&SalesFormMessage::SummaryValidTitle)}"
                                } else {
                                    "{i18n.localize_message(&SalesFormMessage::SummaryInvalidTitle)}"
                                }
                            }
                            p {
                                if is_valid {
                                    "{i18n.localize_message(&SalesFormMessage::SummaryValidBody)}"
                                } else {
                                    "{i18n.localize_message(&SalesFormMessage::SummaryInvalidBody)}"
                                }
                            }
                            p { class: "sales-progress",
                                "{valid_fields} / {SALES_FIELD_COUNT} {i18n.localize_message(&SalesFormMessage::SummaryProgressLabel)}"
                            }
                        }
                        if !issue_groups.is_empty() {
                            div { class: "sales-issue-list",
                                for (field, messages) in issue_groups {
                                    div { class: "sales-issue-group",
                                        h3 { "{field}" }
                                        ul {
                                            for message in messages {
                                                li { "{message}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "sales-rules",
                            h3 { "{i18n.localize_message(&SalesFormMessage::RulesTitle)}" }
                            ul {
                                li { "{i18n.localize_message(&SalesFormMessage::RuleRequired)}" }
                                li { "{i18n.localize_message(&SalesFormMessage::RuleOptional)}" }
                            }
                        }
                        button {
                            class: "sales-submit-button",
                            r#type: "button",
                            disabled: !is_valid,
                            Icon {
                                class: "sales-button-icon".to_string(),
                                width: 17,
                                height: 17,
                                icon: LdSend,
                            }
                            "{i18n.localize_message(&SalesFormMessage::SubmitAction)}"
                        }
                    }
                }
            }
            FooterPanel {}
        }
    }
}

fn sales_text_field(
    input_id: &'static str,
    label: String,
    value: String,
    placeholder: String,
    input_type: &'static str,
    status: FieldStatus,
    status_label: String,
    errors: Vec<String>,
    hint: Option<String>,
    on_input: EventHandler<Event<FormData>>,
) -> Element {
    let control_class = format!("sales-control {}", status.class());
    let badge_class = format!("sales-field-status {}", status.class());

    rsx! {
        div { class: "sales-field",
            div { class: "sales-label-row",
                Label {
                    html_for: input_id.to_string(),
                    class: "sales-label".to_string(),
                    "{label}"
                }
                span { class: "{badge_class}", "{status_label}" }
            }
            input {
                id: input_id,
                class: "{control_class}",
                r#type: input_type,
                value,
                placeholder,
                oninput: on_input,
            }
            {field_hint_and_errors(hint, errors)}
        }
    }
}

fn sales_select_field(
    input_id: &'static str,
    label: String,
    value: String,
    placeholder: String,
    status: FieldStatus,
    status_label: String,
    errors: Vec<String>,
    on_change: EventHandler<Event<FormData>>,
) -> Element {
    let control_class = format!("sales-control {}", status.class());
    let badge_class = format!("sales-field-status {}", status.class());

    rsx! {
        div { class: "sales-field",
            div { class: "sales-label-row",
                Label {
                    html_for: input_id.to_string(),
                    class: "sales-label".to_string(),
                    "{label}"
                }
                span { class: "{badge_class}", "{status_label}" }
            }
            select {
                id: input_id,
                class: "{control_class}",
                value,
                onchange: on_change,
                option { value: "", "{placeholder}" }
                for stage in SALES_STAGES {
                    option { value: "{stage}", "{stage}" }
                }
            }
            {field_hint_and_errors(None, errors)}
        }
    }
}

fn sales_text_area(
    input_id: &'static str,
    label: String,
    value: String,
    placeholder: String,
    status: FieldStatus,
    status_label: String,
    errors: Vec<String>,
    on_input: EventHandler<Event<FormData>>,
) -> Element {
    let control_class = format!("sales-control {}", status.class());
    let badge_class = format!("sales-field-status {}", status.class());

    rsx! {
        div { class: "sales-field sales-field-full",
            div { class: "sales-label-row",
                Label {
                    html_for: input_id.to_string(),
                    class: "sales-label".to_string(),
                    "{label}"
                }
                span { class: "{badge_class}", "{status_label}" }
            }
            textarea {
                id: input_id,
                class: "{control_class}",
                value,
                placeholder,
                rows: "4",
                oninput: on_input,
            }
            {field_hint_and_errors(None, errors)}
        }
    }
}

fn field_hint_and_errors(hint: Option<String>, errors: Vec<String>) -> Element {
    rsx! {
        if let Some(hint) = hint {
            p { class: "sales-field-hint", "{hint}" }
        }
        if !errors.is_empty() {
            ul { class: "sales-field-errors",
                for error in errors {
                    li { "{error}" }
                }
            }
        }
    }
}

fn validate_sales_draft(draft: &SalesLeadDraft) -> SalesFeedback {
    let form = SalesLeadForm::from(draft);

    match form.validate() {
        Ok(()) => SalesFeedback::default(),
        Err(errors) => SalesFeedback {
            company: format_errors(errors.company().all()),
            contact_name: format_errors(errors.contact_name().all()),
            email: format_errors(errors.email().all()),
            phone: format_errors(errors.phone().all()),
            deal_value: format_errors(errors.deal_value().all()),
            stage: format_errors(errors.stage().all()),
            source_url: format_errors(errors.source_url().all()),
            next_step: format_errors(errors.next_step().all()),
        },
    }
}

fn format_errors<T: Display>(errors: impl IntoIterator<Item = T>) -> Vec<String> {
    errors.into_iter().map(|error| error.to_string()).collect()
}

fn field_status(errors: &[String], value: &str, optional: bool) -> FieldStatus {
    if !errors.is_empty() {
        FieldStatus::Invalid
    } else if optional && value.trim().is_empty() {
        FieldStatus::Optional
    } else {
        FieldStatus::Valid
    }
}

fn status_text(
    status: FieldStatus,
    valid_label: &str,
    invalid_label: &str,
    optional_label: &str,
) -> String {
    match status {
        FieldStatus::Valid => valid_label.to_string(),
        FieldStatus::Invalid => invalid_label.to_string(),
        FieldStatus::Optional => optional_label.to_string(),
    }
}

fn optional_trimmed(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn parse_deal_value(value: &str) -> Option<f64> {
    let value = value.trim().replace(',', "");
    if value.is_empty() {
        None
    } else {
        value.parse().ok()
    }
}

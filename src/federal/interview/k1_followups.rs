macro_rules! common {
    ($w:expr, $answer:expr, $invalid:expr, $suffix:expr) => {{
        let invalid = $invalid;
        let suffix = $suffix;
        let w = $w;
        match $answer {
            A::K1RentalParticipation { value } => w.rentals.participation = value,
            A::K1RentalRecords { carryover_active, disposed } => {
                w.rentals.carryover_active = carryover_active;
                w.rentals.disposed = disposed;
            }
            A::K1Section179Records { carryover, sold } => {
                w.more.section179_carryover = carryover;
                w.more.section179_carryover_sold = sold;
            }
            A::K1Section179Amt { value } => w.more.amt_section179_income_carry = value,
            A::K1CreditConfirmations { housing } => {
                let credits =
                    w.more.credits.iter().map(crate::K1CreditCallBox::from).collect::<Vec<_>>();
                let housing_allowed = crate::K1PartnershipCalls::from_credits(&credits).housing;
                if housing != w.more.housing_confirmed
                    && housing.is_some()
                    && housing_allowed.is_empty()
                {
                    return Err(invalid());
                }
                w.more.housing_confirmed = housing;
            }
            A::K1InvestmentRecords { properties, investment, .. } => {
                patch_investment_properties(&mut w.more.investment_property, properties)?;
                if investment != w.more.investment_confirmed
                    && investment.is_some()
                    && crate::k1_page::investment_choices(&w.more.investment_property).is_empty()
                {
                    return Err(invalid());
                }
                w.more.investment_confirmed = investment;
            }
            A::K1CreditOrigins { carbon_oxide_origin, renewable_facility_origin, .. } => {
                let credit = w
                    .more
                    .credits
                    .iter_mut()
                    .find(|credit| credit_key(credit) == suffix)
                    .ok_or_else(invalid)?;
                if (carbon_oxide_origin.is_some()
                    && crate::business_credit_page::k1_carbon_oxide_origin_choices(
                        credit.code.clone(),
                    )
                    .is_empty())
                    || (renewable_facility_origin.is_some()
                        && crate::business_credit_page::k1_renewable_facility_origin_choices(
                            credit.code.clone(),
                        )
                        .is_empty())
                {
                    return Err(invalid());
                }
                credit.carbon_oxide_origin = carbon_oxide_origin;
                credit.renewable_facility_origin = renewable_facility_origin;
            }
            _ => return Err(invalid()),
        }
    }};
}

use super::*;
use crate::{DocumentAnswer as A, K1Document as K};
use alloc::string::String;

type Question = (String, &'static str, A, Vec<i32>);

pub(super) fn catalog(
    draft: &FederalDraft,
    source: &IncomeSource,
    field: &str,
) -> crate::K1QuestionChoices {
    let mut choices = crate::K1QuestionChoices::default();
    let IncomeSource::K1 { value } = source else {
        return choices;
    };
    if let K::K1EstateTrust(source) = value {
        return super::estate_followups::catalog(draft, &source.source().page());
    }
    if field == "records" && matches!(value, K::K1Partnership(_)) {
        let needed = draft.people.as_ref().is_some_and(|people| {
            crate::federal_rules::young_8615(joint(draft), people)
                || crate::federal_rules::dependent_standard(people)
        });
        choices.source_choices = crate::K1IncomeSource::allowed(needed);
    }
    let partner_page = match value {
        K::K1Partnership(w) => Some(w.calculation_page()),
        _ => None,
    };
    let scorp_page = match value {
        K::K1Scorp(w) => Some(w.source().page()),
        _ => None,
    };
    let credits: &[crate::K1CreditPage] = match value {
        K::K1Partnership(_) => {
            partner_page.as_ref().map_or(&[], |page| page.more.credits.as_slice())
        }
        K::K1Scorp(_) => scorp_page.as_ref().map_or(&[], |page| page.more.credits.as_slice()),
        K::K1PartnershipActivities(w) => &w.entity.credits,
        K::K1ScorpActivities(w) => &w.entity.credits,
        _ => return choices,
    };
    if field == "credit_confirmations" || field == "investment" {
        let boxes = credits.iter().map(crate::K1CreditCallBox::from).collect::<Vec<_>>();
        choices.housing_choices = crate::K1PartnershipCalls::from_credits(&boxes).housing;
    }
    if let Some(credit) = credits.iter().find(|credit| credit_key(credit) == field) {
        choices.carbon_choices =
            crate::business_credit_page::k1_carbon_oxide_origin_choices(credit.code.clone());
        choices.renewable_choices =
            crate::business_credit_page::k1_renewable_facility_origin_choices(credit.code.clone());
    }
    choices
}

pub(super) fn income_source_issue(
    draft: &FederalDraft,
    source: &IncomeSource,
    require_answers: bool,
) -> Option<ValidationIssue> {
    let IncomeSource::K1 { value: K::K1Partnership(w) } = source else {
        return None;
    };
    let w = w.calculation_page();
    let needed = draft.people.as_ref().is_some_and(|people| {
        crate::federal_rules::young_8615(joint(draft), people)
            || crate::federal_rules::dependent_standard(people)
    });
    if !require_answers && needed && matches!(w.income_source, crate::K1IncomeSource::NotNeeded) {
        return None;
    }
    (!crate::K1IncomeSource::allowed(needed).contains(&w.income_source.index())).then(|| {
        ValidationIssue::new(if needed { PARTNER_SOURCE_NEEDED } else { SOURCE_NOT_NEEDED })
            .at_field("income_source")
    })
}

pub(super) fn issue(source: &IncomeSource, require_answers: bool) -> Option<ValidationIssue> {
    if let Some(key) = crate::state_return::document_rule(source) {
        return Some(ValidationIssue::new(key).at_record());
    }
    match source {
        IncomeSource::K1 { value: K::K1Partnership(w) } => w.issue_with_mode(require_answers),
        IncomeSource::K1 { value: K::K1Scorp(w) } => w.issue_with_mode(require_answers),
        IncomeSource::K1 { value: K::K1PartnershipActivities(w) } => {
            w.record().issue_with_mode(require_answers).or_else(|| {
                require_answers
                    .then(|| {
                        credit_confirmation_issue(
                            &w.entity.credits,
                            &w.entity.investment_property,
                            w.entity.housing_confirmed.is_some(),
                            w.entity.investment_confirmed.is_some(),
                        )
                    })
                    .flatten()
            })
        }
        IncomeSource::K1 { value: K::K1ScorpActivities(w) } => {
            w.record().issue_with_mode(require_answers).or_else(|| {
                require_answers
                    .then(|| {
                        credit_confirmation_issue(
                            &w.entity.credits,
                            &w.entity.investment_property,
                            w.entity.housing_confirmed.is_some(),
                            w.entity.investment_confirmed.is_some(),
                        )
                    })
                    .flatten()
            })
        }
        IncomeSource::K1 { value: K::K1EstateTrust(w) } => w.issue_with_mode(require_answers),
        source => document_issue(source),
    }
}
pub(super) fn topic_issue(source: &IncomeSource, require_answers: bool) -> Option<ValidationIssue> {
    match source {
        IncomeSource::K1 { value: K::K1EstateTrust(_) } => None,
        IncomeSource::K1 { value: K::K1Partnership(value) } => value.topic_issue(require_answers),
        IncomeSource::K1 { value: K::K1Scorp(_) } => None,
        IncomeSource::K1 { value: K::K1PartnershipActivities(value) } => {
            value.record().topic_issue(require_answers).or_else(|| {
                require_answers
                    .then(|| {
                        credit_confirmation_issue(
                            &value.entity.credits,
                            &value.entity.investment_property,
                            value.entity.housing_confirmed.is_some(),
                            value.entity.investment_confirmed.is_some(),
                        )
                    })
                    .flatten()
            })
        }
        IncomeSource::K1 { value: K::K1ScorpActivities(value) } => {
            value.record().topic_issue(require_answers).or_else(|| {
                require_answers
                    .then(|| {
                        credit_confirmation_issue(
                            &value.entity.credits,
                            &value.entity.investment_property,
                            value.entity.housing_confirmed.is_some(),
                            value.entity.investment_confirmed.is_some(),
                        )
                    })
                    .flatten()
            })
        }
        _ => None,
    }
}

fn credit_confirmation_issue(
    credits: &[crate::K1CreditPage],
    properties: &[crate::K1InvestmentProperty],
    housing: bool,
    investment: bool,
) -> Option<ValidationIssue> {
    if properties.iter().any(|property| property.own_form().is_none()) && !investment {
        Some(
            ValidationIssue::new("K1_INVESTMENT_NEEDS_CONFIRMATION")
                .at_field("investment_confirmed")
                .in_field("entity"),
        )
    } else if credits.iter().any(|credit| credit.code.housing()) && !housing {
        Some(
            ValidationIssue::new("K1_HOUSING_NEEDS_CONFIRMATION")
                .at_field("housing_confirmed")
                .in_field("entity"),
        )
    } else {
        None
    }
}

pub(super) fn answer_kind(answer: &A) -> Option<i32> {
    use libertas_macros::variant_index;
    Some(match answer {
        A::EstateActivityRecords { .. } => variant_index!(A::EstateActivityRecords) as i32,
        A::EstateQbiHistory { .. } => variant_index!(A::EstateQbiHistory) as i32,
        A::EstateAmtTreatment { .. } => variant_index!(A::EstateAmtTreatment) as i32,
        A::EstateInvestmentRecords { .. } => variant_index!(A::EstateInvestmentRecords) as i32,
        A::EstateRoyaltyCharacter { .. } => variant_index!(A::EstateRoyaltyCharacter) as i32,
        A::EstateAccounting { .. } => variant_index!(A::EstateAccounting) as i32,
        A::EstateFinalTaxCharacter { .. } => variant_index!(A::EstateFinalTaxCharacter) as i32,
        A::EstateNolRecords { .. } => variant_index!(A::EstateNolRecords) as i32,
        A::EstateNiitAllocation { .. } => variant_index!(A::EstateNiitAllocation) as i32,
        A::EstateResearchCreditLine { .. } => variant_index!(A::EstateResearchCreditLine) as i32,
        A::EstateOtherSource { .. } => variant_index!(A::EstateOtherSource) as i32,
        A::EstateFinalRecords { .. } => variant_index!(A::EstateFinalRecords) as i32,
        A::EstateFinalSuccessor { .. } => variant_index!(A::EstateFinalSuccessor) as i32,
        A::EstateReitEligibility { .. } => variant_index!(A::EstateReitEligibility) as i32,
        A::K1PartnerRecords { .. } => variant_index!(A::K1PartnerRecords) as i32,
        A::K1PartnerActivityRecords { .. } => variant_index!(A::K1PartnerActivityRecords) as i32,
        A::K1ScorpActivityRecords { .. } => variant_index!(A::K1ScorpActivityRecords) as i32,
        A::K1ScorpRecords { .. } => variant_index!(A::K1ScorpRecords) as i32,
        A::K1EntityDisposition { .. } => variant_index!(A::K1EntityDisposition) as i32,
        A::K1RentalRecords { .. } => variant_index!(A::K1RentalRecords) as i32,
        A::K1RentalParticipation { .. } => variant_index!(A::K1RentalParticipation) as i32,
        A::K1PartnerBasisStart { .. } => variant_index!(A::K1PartnerBasisStart) as i32,
        A::K1ScorpBasisStart { .. } => variant_index!(A::K1ScorpBasisStart) as i32,
        A::K1PartnerParticipation { .. } => variant_index!(A::K1PartnerParticipation) as i32,
        A::K1LimitedForSe { .. } => variant_index!(A::K1LimitedForSe) as i32,
        A::K1PartnerAmt { .. } => variant_index!(A::K1PartnerAmt) as i32,
        A::K1PartnerDisposition { .. } => variant_index!(A::K1PartnerDisposition) as i32,
        A::K1ScorpParticipation { .. } => variant_index!(A::K1ScorpParticipation) as i32,
        A::K1ScorpAmt { .. } => variant_index!(A::K1ScorpAmt) as i32,
        A::K1ScorpDisposition { .. } => variant_index!(A::K1ScorpDisposition) as i32,
        A::K1K3Withholding { .. } => variant_index!(A::K1K3Withholding) as i32,
        A::K1ResearchCreditLine { .. } => variant_index!(A::K1ResearchCreditLine) as i32,
        A::K1Section179Records { .. } => variant_index!(A::K1Section179Records) as i32,
        A::K1Section179Amt { .. } => variant_index!(A::K1Section179Amt) as i32,
        A::K1DepletionSources { .. } => variant_index!(A::K1DepletionSources) as i32,
        A::K1CreditConfirmations { .. } => variant_index!(A::K1CreditConfirmations) as i32,
        A::K1InvestmentRecords { .. } => variant_index!(A::K1InvestmentRecords) as i32,
        A::K1CreditOrigins { .. } => variant_index!(A::K1CreditOrigins) as i32,
        _ => return None,
    })
}

fn credit_key(credit: &crate::K1CreditPage) -> String {
    let stored: crate::K1Credit = credit.clone().into();
    // Native admission guarantees one retained credit per (code, line).
    alloc::format!("credit/{:?}/{:?}/origins", stored.code, stored.line)
}

pub(super) fn own_form_mut(
    property: &mut crate::K1InvestmentProperty,
) -> &mut Option<crate::OwnInvestmentCredit> {
    property.own_form_mut()
}

pub(super) fn patch_investment_properties(
    properties: &mut [crate::K1InvestmentProperty],
    supplied: Vec<crate::K1InvestmentAnswer>,
) -> Result<(), ValidationIssue> {
    if properties.len() != supplied.len() {
        return Err(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("properties"));
    }
    let mut seen = alloc::collections::BTreeSet::new();
    for (index, answer) in supplied.into_iter().enumerate() {
        let Some(property) = properties.iter_mut().find(|property| property.id() == answer.id)
        else {
            return Err(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                .at_field("id")
                .in_list("properties", index));
        };
        if !seen.insert(answer.id) {
            return Err(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                .at_field("id")
                .in_list("properties", index));
        }
        *own_form_mut(property) = answer.own_form;
        if !property.valid() {
            return Err(ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                .at_field("own_form")
                .in_list("properties", index));
        }
    }
    Ok(())
}

pub(super) fn proposed_investment_choices(
    draft: &FederalDraft,
    key: &str,
    supplied: Vec<crate::K1InvestmentAnswer>,
) -> Result<Vec<i32>, ValidationIssue> {
    let entry = draft
        .income
        .iter()
        .find(|entry| key == alloc::format!("income/{}/investment", entry.id))
        .ok_or_else(|| ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("key"))?;
    let item = document_item(entry, &[], crate::federal_rules::separate_return(draft))
        .ok_or_else(|| ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("key"))?;
    let IncomeSource::K1 { value } = item.document else {
        return Err(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("key"));
    };
    let mut properties = match value {
        K::K1Partnership(w) => w.calculation_page().more.investment_property,
        K::K1Scorp(_) => match &entry.income {
            FederalIncome::Scorp { data } => crate::ScorpK1::from(data).investment_property,
            _ => return Err(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("key")),
        },
        K::K1PartnershipActivities(w) => w.entity.investment_property,
        K::K1ScorpActivities(w) => w.entity.investment_property,
        K::K1EstateTrust(w) => {
            let mut properties: Vec<_> = w
                .source()
                .statements
                .investment_property
                .into_iter()
                .map(|row| row.property)
                .collect();
            patch_investment_properties(&mut properties, supplied)?;
            return Ok(Vec::new());
        }
    };
    if properties.is_empty() {
        return Err(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("key"));
    }
    patch_investment_properties(&mut properties, supplied)?;
    Ok(crate::k1_page::investment_choices(&properties))
}

struct Common<'a> {
    rentals: &'a crate::K1RentalBoxes,
    // The stored record has rental real estate (box 2, its carryover, or a
    // rental real estate credit or property): participation is asked.
    rental_needed: bool,
    credits: &'a [crate::K1CreditPage],
    properties: &'a [crate::K1InvestmentProperty],
    carryover: i64,
    sold: i64,
    amt_history: &'a Option<crate::Section179IncomeCarryHistory>,
    // A section 179 deduction (box 12) or regular carryover needs the
    // independent AMT carryover history; it is asked, never left absent.
    section179_amt_needed: bool,
    housing: Option<crate::K1HousingAcquisition>,
    investment: Option<crate::K1InvestmentConfirmation>,
    housing_choices: Vec<i32>,
}

fn common_questions(rows: &mut Vec<Question>, common: Common<'_>) {
    if common.rental_needed {
        rows.push((
            "rental_participation".into(),
            "DOCUMENT_K1_RENTAL_PARTICIPATION",
            A::K1RentalParticipation { value: common.rentals.participation },
            Vec::new(),
        ));
    }
    rows.push((
        "rentals".into(),
        "DOCUMENT_K1_RENTAL_RECORDS",
        A::K1RentalRecords {
            carryover_active: common.rentals.carryover_active,
            disposed: common.rentals.disposed.clone(),
        },
        Vec::new(),
    ));
    rows.push((
        "section179".into(),
        "DOCUMENT_K1_SECTION179_RECORDS",
        A::K1Section179Records { carryover: common.carryover, sold: common.sold },
        Vec::new(),
    ));
    if common.section179_amt_needed {
        rows.push((
            "section179_amt".into(),
            "DOCUMENT_K1_SECTION179_AMT",
            A::K1Section179Amt { value: common.amt_history.clone() },
            Vec::new(),
        ));
    }
    credit_questions(
        rows,
        common.credits,
        common.properties,
        common.housing,
        common.investment,
        common.housing_choices,
    );
}

fn credit_questions(
    rows: &mut Vec<Question>,
    credits: &[crate::K1CreditPage],
    properties: &[crate::K1InvestmentProperty],
    housing: Option<crate::K1HousingAcquisition>,
    investment: Option<crate::K1InvestmentConfirmation>,
    housing_choices: Vec<i32>,
) {
    if !housing_choices.is_empty() {
        rows.push((
            "credit_confirmations".into(),
            "DOCUMENT_K1_CREDIT_CONFIRMATIONS",
            A::K1CreditConfirmations { housing },
            housing_choices,
        ));
    }
    if !properties.is_empty() {
        let descriptions =
            crate::K1ActivityChoiceContext::from_source_properties(None, properties, &[])
                .properties;
        rows.push((
            "investment".into(),
            "DOCUMENT_K1_OWN_INVESTMENT_CREDIT",
            A::K1InvestmentRecords {
                allowed_choices: crate::k1_page::investment_choices(properties),
                properties: properties
                    .iter()
                    .map(|property| crate::K1InvestmentAnswer {
                        id: property.id(),
                        description: descriptions
                            .iter()
                            .find(|choice| choice.id == property.id())
                            .map(|choice| choice.label.clone())
                            .unwrap_or_default(),
                        own_form: property.own_form().cloned(),
                    })
                    .collect(),
                investment,
            },
            Vec::new(),
        ));
    }
    for credit in credits {
        let carbon_choices =
            crate::business_credit_page::k1_carbon_oxide_origin_choices(credit.code.clone());
        let renewable_choices =
            crate::business_credit_page::k1_renewable_facility_origin_choices(credit.code.clone());
        if !carbon_choices.is_empty() || !renewable_choices.is_empty() {
            rows.push((
                credit_key(credit),
                "DOCUMENT_K1_CREDIT_ORIGINS",
                A::K1CreditOrigins {
                    carbon_oxide_origin: credit.carbon_oxide_origin.clone(),
                    renewable_facility_origin: credit.renewable_facility_origin.clone(),
                },
                Vec::new(),
            ));
        }
    }
}

pub(super) fn questions(_draft: &FederalDraft, source: &K) -> Vec<Question> {
    let mut rows = Vec::new();
    let depletion = match source {
        K::K1PartnershipActivities(w) => w.record().depletion_page(),
        K::K1ScorpActivities(w) => w.record().depletion_page(),
        _ => None,
    };
    if let Some(value) = depletion {
        rows.push((
            "depletion_sources".into(),
            "DOCUMENT_K1_DEPLETION_SOURCES",
            A::K1DepletionSources { value },
            Vec::new(),
        ));
    }
    match source {
        K::K1EstateTrust(w) => return estate_questions(&w.source()),
        K::K1Partnership(w) => {
            if w.research_credit_needed() {
                rows.push((
                    "research_credit_line".into(),
                    "DOCUMENT_K1_RESEARCH_CREDIT_LINE",
                    A::K1ResearchCreditLine {
                        value: w.personal.as_ref().and_then(|records| records.research_credit_line),
                    },
                    crate::business_credit_page::research_credit_lines(),
                ));
            }
            if let Some(k3) = &w.statements.schedule_k3
                && k3
                    .native(&[])
                    .countries
                    .iter()
                    .any(|country| country.category == crate::K3Category::Passive)
            {
                rows.push((
                    "k3_withholding".into(),
                    "DOCUMENT_K1_K3_WITHHOLDING",
                    A::K1K3Withholding {
                        value: w
                            .personal
                            .as_ref()
                            .map_or_else(Vec::new, |records| records.k3_withholding.clone()),
                    },
                    Vec::new(),
                ));
            }
            let rental_needed = w.record().rental_participation_needed();
            let w = w.calculation_page();
            rows.push((
                "basis_start".into(),
                "DOCUMENT_K1_OPENING_BASIS",
                A::K1PartnerBasisStart { value: Some(w.basis.start.clone()) },
                crate::ScorpRiskStart::continuing_choices(),
            ));
            rows.push((
                "participation".into(),
                "DOCUMENT_K1_PARTICIPATION",
                A::K1PartnerParticipation { value: w.participation.clone() },
                Vec::new(),
            ));
            rows.push((
                "limited_for_se".into(),
                "DOCUMENT_K1_LIMITED_FOR_SE",
                A::K1LimitedForSe { value: w.limited_for_se },
                Vec::new(),
            ));
            rows.push((
                "amt".into(),
                "DOCUMENT_K1_AMT_RECORDS",
                A::K1PartnerAmt { value: w.amt.clone() },
                Vec::new(),
            ));
            rows.push((
                "disposition".into(),
                "DOCUMENT_K1_DISPOSITION",
                A::K1PartnerDisposition { value: w.disposition.clone() },
                Vec::new(),
            ));
            rows.push((
                "records".into(),
                "DOCUMENT_K1_PARTNER_RECORDS",
                A::K1PartnerRecords {
                    qbi_records: w.qbi_records_record(),
                    basis: crate::PartnerBasisRecords::from_stored(&w.basis),
                    income_source: w.income_source.clone(),
                    special: w.special.clone(),
                    qbi_group: w.qbi_group.clone(),
                    fuel_credit: w.more.fuel_credit.clone(),
                    prior_fuel_credit: w.more.prior_fuel_credit.clone(),
                    term: w.more.term,
                    trading: w.more.trading,
                },
                Vec::new(),
            ));
            let credits =
                w.more.credits.iter().map(crate::K1CreditCallBox::from).collect::<Vec<_>>();
            common_questions(
                &mut rows,
                Common {
                    rental_needed,
                    rentals: &w.rentals,
                    credits: &w.more.credits,
                    properties: &w.more.investment_property,
                    carryover: w.more.section179_carryover,
                    sold: w.more.section179_carryover_sold,
                    amt_history: &w.more.amt_section179_income_carry,
                    section179_amt_needed: w.more.section179 != 0
                        || w.more.section179_carryover != 0,
                    housing: w.more.housing_confirmed,
                    investment: w.more.investment_confirmed,
                    housing_choices: crate::K1PartnershipCalls::from_credits(&credits).housing,
                },
            );
        }
        K::K1Scorp(_) => {}
        K::K1PartnershipActivities(w) => {
            rows.push((
                "activity_records".into(),
                "DOCUMENT_K1_PARTNER_RECORDS",
                A::K1PartnerActivityRecords { value: w.as_ref().into() },
                Vec::new(),
            ));
            rows.push((
                "disposition".into(),
                "DOCUMENT_K1_DISPOSITION",
                A::K1EntityDisposition { value: w.entity.disposition.clone() },
                Vec::new(),
            ));
            let credits =
                w.entity.credits.iter().map(crate::K1CreditCallBox::from).collect::<Vec<_>>();
            credit_questions(
                &mut rows,
                &w.entity.credits,
                &w.entity.investment_property,
                w.entity.housing_confirmed.map(Into::into),
                w.entity.investment_confirmed,
                crate::K1PartnershipCalls::from_credits(&credits).housing,
            );
        }
        K::K1ScorpActivities(w) => {
            rows.push((
                "activity_records".into(),
                "DOCUMENT_K1_SCORP_RECORDS",
                A::K1ScorpActivityRecords { value: w.into() },
                Vec::new(),
            ));
            rows.push((
                "disposition".into(),
                "DOCUMENT_K1_DISPOSITION",
                A::K1EntityDisposition { value: w.entity.disposition.clone() },
                Vec::new(),
            ));
            let credits =
                w.entity.credits.iter().map(crate::K1CreditCallBox::from).collect::<Vec<_>>();
            credit_questions(
                &mut rows,
                &w.entity.credits,
                &w.entity.investment_property,
                w.entity.housing_confirmed.map(Into::into),
                w.entity.investment_confirmed,
                crate::K1ScorpCalls::from_credits(&credits).housing,
            );
        }
    }
    rows
}

pub(super) fn scorp_questions(w: &crate::ScorpSource) -> Vec<Question> {
    let mut rows = Vec::new();
    if crate::K1ScorpDocument::from_source(w).research_credit_needed() {
        rows.push((
            "research_credit_line".into(),
            "DOCUMENT_K1_RESEARCH_CREDIT_LINE",
            A::K1ResearchCreditLine { value: w.personal.research_credit_line },
            crate::business_credit_page::research_credit_lines(),
        ));
    }
    if let Some(k3) = &w.statements.schedule_k3
        && k3
            .native(&[])
            .countries
            .iter()
            .any(|country| country.category == crate::K3Category::Passive)
    {
        rows.push((
            "k3_withholding".into(),
            "DOCUMENT_K1_K3_WITHHOLDING",
            A::K1K3Withholding { value: w.personal.k3_withholding.clone() },
            Vec::new(),
        ));
    }
    let rental_needed = crate::ScorpK1::from(w).rental_participation_needed();
    let w = w.page();
    rows.push((
        "basis_start".into(),
        "DOCUMENT_K1_OPENING_BASIS",
        A::K1ScorpBasisStart { value: Some(w.basis.start.clone()) },
        crate::ScorpRiskStart::continuing_choices(),
    ));
    rows.push((
        "participation".into(),
        "DOCUMENT_K1_PARTICIPATION",
        A::K1ScorpParticipation { value: w.participation.clone() },
        Vec::new(),
    ));
    rows.push((
        "amt".into(),
        "DOCUMENT_K1_AMT_RECORDS",
        A::K1ScorpAmt { value: w.amt.clone() },
        Vec::new(),
    ));
    rows.push((
        "disposition".into(),
        "DOCUMENT_K1_DISPOSITION",
        A::K1ScorpDisposition { value: w.disposition.clone() },
        Vec::new(),
    ));
    rows.push((
        "records".into(),
        "DOCUMENT_K1_SCORP_RECORDS",
        A::K1ScorpRecords {
            qbi_records: w.qbi_records_record(),
            basis: crate::K1ScorpBasisRecords::from_stored(&w.basis),
            special: w.special.clone(),
            qbi_group: w.qbi_group.clone(),
            stock_term: w.more.stock_term,
            trading: w.more.trading,
        },
        Vec::new(),
    ));
    let credits = w.more.credits.iter().map(crate::K1CreditCallBox::from).collect::<Vec<_>>();
    common_questions(
        &mut rows,
        Common {
            rental_needed,
            rentals: &w.rentals,
            credits: &w.more.credits,
            properties: &w.more.investment_property,
            carryover: w.more.section179_carryover,
            sold: w.more.section179_carryover_sold,
            amt_history: &w.more.amt_section179_income_carry,
            section179_amt_needed: w.more.section179 != 0 || w.more.section179_carryover != 0,
            housing: w.more.housing_confirmed,
            investment: w.more.investment_confirmed,
            housing_choices: crate::K1ScorpCalls::from_credits(&credits).housing,
        },
    );
    rows
}

pub(super) fn apply(
    source: &mut IncomeSource,
    accepted_key: &str,
    answer: A,
) -> Result<(), ValidationIssue> {
    let invalid = || ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("value");
    let IncomeSource::K1 { value: source } = source else {
        return Err(invalid());
    };
    let suffix = accepted_key.splitn(3, '/').nth(2).ok_or_else(invalid)?;
    if let A::K1DepletionSources { value } = &answer {
        if suffix != "depletion_sources" {
            return Err(invalid());
        }
        let current = match &*source {
            K::K1PartnershipActivities(w) => w.record().depletion_page(),
            K::K1ScorpActivities(w) => w.record().depletion_page(),
            _ => None,
        }
        .ok_or_else(invalid)?;
        value.validate_for(&current).map_err(|issue| issue.in_field("value"))?;
        let allocations = value.record();
        match source {
            K::K1PartnershipActivities(w) => w.depletion_sources = allocations,
            K::K1ScorpActivities(w) => w.depletion_sources = allocations,
            _ => return Err(invalid()),
        }
        return Ok(());
    }
    if suffix == "activity_records" {
        return super::activity_followups::apply(source, answer);
    }
    if matches!(source, K::K1EstateTrust(_)) {
        return Err(invalid());
    }
    if let K::K1Partnership(w) = &mut *source
        && let A::K1ResearchCreditLine { value } = &answer
    {
        if value.is_some_and(|line| !crate::K1CreditCode::Research.line_allows(line)) {
            return Err(invalid());
        }
        let records = w.personal.get_or_insert_with(crate::PartnerPersonalRecords::default);
        records.research_credit_line = *value;
        return Ok(());
    }
    if let K::K1Partnership(w) = &mut *source
        && let A::K1K3Withholding { value } = &answer
    {
        let records = w.personal.get_or_insert_with(crate::PartnerPersonalRecords::default);
        records.k3_withholding = value.clone();
        return Ok(());
    }
    match (source, answer) {
        (
            K::K1Partnership(w),
            A::K1PartnerRecords {
                qbi_records,
                basis,
                income_source,
                special,
                qbi_group,
                fuel_credit,
                prior_fuel_credit,
                term,
                trading,
                ..
            },
        ) => {
            let mut page = w.calculation_page();
            page.qbi_records = Some(qbi_records);
            page.basis = basis.stored(page.basis.start.clone());
            page.income_source = income_source;
            page.special = special;
            page.qbi_group = qbi_group;
            page.more.fuel_credit = fuel_credit;
            page.more.prior_fuel_credit = prior_fuel_credit;
            page.more.term = term;
            page.more.trading = trading;
            w.replace_personal_page(&page);
        }
        (
            K::K1Partnership(w),
            answer @ (A::K1PartnerParticipation { .. }
            | A::K1PartnerBasisStart { .. }
            | A::K1LimitedForSe { .. }
            | A::K1PartnerAmt { .. }
            | A::K1PartnerDisposition { .. }),
        ) => {
            let mut page = w.calculation_page();
            match answer {
                A::K1PartnerParticipation { value } => page.participation = value,
                A::K1PartnerBasisStart { value: Some(value) } => page.basis.start = value,
                A::K1PartnerBasisStart { value: None } => {}
                A::K1LimitedForSe { value } => page.limited_for_se = value,
                A::K1PartnerAmt { value } => page.amt = value,
                A::K1PartnerDisposition { value } => page.disposition = value,
                _ => unreachable!(),
            }
            w.replace_personal_page(&page);
        }
        (K::K1PartnershipActivities(w), A::K1EntityDisposition { value }) => {
            w.entity.disposition = value
        }
        (K::K1ScorpActivities(w), A::K1EntityDisposition { value }) => w.entity.disposition = value,
        (K::K1Partnership(w), answer) => {
            let mut page = w.calculation_page();
            common!(&mut page, answer, invalid, suffix);
            w.replace_personal_page(&page);
        }
        (K::K1PartnershipActivities(w), answer) => apply_entity_credit(
            &mut w.entity.credits,
            &mut w.entity.investment_property,
            &mut w.entity.housing_confirmed,
            &mut w.entity.investment_confirmed,
            suffix,
            answer,
        )?,
        (K::K1ScorpActivities(w), answer) => apply_entity_credit(
            &mut w.entity.credits,
            &mut w.entity.investment_property,
            &mut w.entity.housing_confirmed,
            &mut w.entity.investment_confirmed,
            suffix,
            answer,
        )?,
        _ => return Err(invalid()),
    }
    Ok(())
}
fn apply_entity_credit(
    credits: &mut [crate::K1CreditPage],
    properties: &mut [crate::K1InvestmentProperty],
    housing: &mut Option<bool>,
    investment: &mut Option<crate::K1InvestmentConfirmation>,
    suffix: &str,
    answer: A,
) -> Result<(), ValidationIssue> {
    let invalid = || ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("value");
    match answer {
        A::K1CreditConfirmations { housing: value } => {
            let calls = credits.iter().map(crate::K1CreditCallBox::from).collect::<Vec<_>>();
            if value.map(|value| matches!(value, crate::K1HousingAcquisition::After1989))
                != *housing
                && value.is_some()
                && crate::K1PartnershipCalls::from_credits(&calls).housing.is_empty()
            {
                return Err(invalid());
            }
            *housing = value.map(|value| matches!(value, crate::K1HousingAcquisition::After1989));
        }
        A::K1InvestmentRecords { properties: supplied, investment: choice, .. } => {
            patch_investment_properties(properties, supplied)?;
            if choice != *investment
                && choice.is_some()
                && crate::k1_page::investment_choices(properties).is_empty()
            {
                return Err(invalid());
            }
            *investment = choice;
        }
        A::K1CreditOrigins { carbon_oxide_origin, renewable_facility_origin, .. } => {
            let credit = credits
                .iter_mut()
                .find(|credit| credit_key(credit) == suffix)
                .ok_or_else(invalid)?;
            if (carbon_oxide_origin.is_some()
                && crate::business_credit_page::k1_carbon_oxide_origin_choices(credit.code.clone())
                    .is_empty())
                || (renewable_facility_origin.is_some()
                    && crate::business_credit_page::k1_renewable_facility_origin_choices(
                        credit.code.clone(),
                    )
                    .is_empty())
            {
                return Err(invalid());
            }
            credit.carbon_oxide_origin = carbon_oxide_origin;
            credit.renewable_facility_origin = renewable_facility_origin;
        }
        _ => return Err(invalid()),
    }
    Ok(())
}
fn retain_children(
    properties: &mut [crate::K1InvestmentProperty],
    previous_properties: &[crate::K1InvestmentProperty],
    credits: &mut [crate::K1CreditPage],
    previous_credits: &[crate::K1CreditPage],
) {
    for property in properties {
        let previous =
            previous_properties.iter().find(|old| old.id() == property.id() && old.id() > 0);
        *own_form_mut(property) = previous.and_then(|old| old.own_form().cloned());
    }
    for credit in credits {
        let previous = previous_credits.iter().find(|old| credit_key(old) == credit_key(credit));
        credit.carbon_oxide_origin = previous.and_then(|old| old.carbon_oxide_origin.clone());
        credit.renewable_facility_origin =
            previous.and_then(|old| old.renewable_facility_origin.clone());
    }
}

// Apply exactly the existing page defaults to a new source. Accepted personal
// records are read from native storage, never from the source's hidden echoes.
pub(super) fn retain_source_answers(source: &mut IncomeSource, accepted: Option<&IncomeSource>) {
    if let IncomeSource::K1 { value } = source {
        super::activity_followups::retain(
            value,
            match accepted {
                Some(IncomeSource::K1 { value }) => Some(value),
                _ => None,
            },
        );
    }
    if matches!(source, IncomeSource::K1 { value: K::K1EstateTrust(_) }) {
        return;
    }

    match source {
        IncomeSource::K1 { value: K::K1Partnership(w) } => {
            w.personal = match accepted {
                Some(IncomeSource::K1 { value: K::K1Partnership(old) }) => old.personal.clone(),
                _ => None,
            };
        }
        IncomeSource::K1 { value: K::K1PartnershipActivities(w) } => {
            w.depletion_sources = match accepted {
                Some(IncomeSource::K1 { value: K::K1PartnershipActivities(old) })
                    if crate::k1_depletion_sources::has_depletion(&w.amt) =>
                {
                    let sources = w.record().depletion_choices();
                    old.depletion_sources.as_ref().map(|rows| {
                        rows.iter()
                            .filter(|row| sources.iter().any(|source| source.id == row.source))
                            .cloned()
                            .collect()
                    })
                }
                _ => None,
            };
            let old = match accepted {
                Some(IncomeSource::K1 { value: K::K1PartnershipActivities(old) }) => {
                    Some(&old.entity)
                }
                _ => None,
            };
            w.entity.disposition = old.and_then(|old| old.disposition.clone());
            w.entity.housing_confirmed = old.and_then(|old| old.housing_confirmed);
            w.entity.investment_confirmed = old.and_then(|old| old.investment_confirmed);
            retain_children(
                &mut w.entity.investment_property,
                old.map_or(&[][..], |old| old.investment_property.as_slice()),
                &mut w.entity.credits,
                old.map_or(&[][..], |old| old.credits.as_slice()),
            );
        }
        IncomeSource::K1 { value: K::K1ScorpActivities(w) } => {
            w.depletion_sources = match accepted {
                Some(IncomeSource::K1 { value: K::K1ScorpActivities(old) })
                    if crate::k1_depletion_sources::has_depletion(&w.amt) =>
                {
                    let sources = w.record().depletion_choices();
                    old.depletion_sources.as_ref().map(|rows| {
                        rows.iter()
                            .filter(|row| sources.iter().any(|source| source.id == row.source))
                            .cloned()
                            .collect()
                    })
                }
                _ => None,
            };
            let old = match accepted {
                Some(IncomeSource::K1 { value: K::K1ScorpActivities(old) }) => Some(&old.entity),
                _ => None,
            };
            w.entity.disposition = old.and_then(|old| old.disposition.clone());
            w.entity.housing_confirmed = old.and_then(|old| old.housing_confirmed);
            w.entity.investment_confirmed = old.and_then(|old| old.investment_confirmed);
            retain_children(
                &mut w.entity.investment_property,
                old.map_or(&[][..], |old| old.investment_property.as_slice()),
                &mut w.entity.credits,
                old.map_or(&[][..], |old| old.credits.as_slice()),
            );
        }
        _ => {}
    }
}

// Locate an existing native refusal on its editable question, retaining the
// owner's key and the remaining typed path after the moved field.
pub(super) fn bind_issue(
    source: &IncomeSource,
    issue: ValidationIssue,
    submitted: &[(i64, String, usize)],
    source_id: i64,
) -> Option<SaveFailure> {
    let IncomeSource::K1 { value: source } = source else {
        return None;
    };
    if let K::K1EstateTrust(w) = source {
        return super::estate_followups::bind_issue(w, issue, submitted, source_id);
    }
    if issue.has_field_prefix(&["depletion_sources"]) {
        let key = alloc::format!("income/{source_id}/depletion_sources");
        let (_, _, index) =
            submitted.iter().find(|(id, found, _)| *id == source_id && *found == key)?;
        return Some(
            issue
                .map_field_prefix(&["depletion_sources"], &["value", "allocations"])
                .in_field("answer")
                .in_list("followups", *index)
                .into(),
        );
    }
    if matches!(source, K::K1PartnershipActivities(_) | K::K1ScorpActivities(_))
        && let Some(failure) =
            super::activity_followups::bind_issue(source, issue.clone(), submitted, source_id)
    {
        return Some(failure);
    }
    if matches!(source, K::K1Partnership(_) | K::K1Scorp(_))
        && (issue.has_field_prefix(&["schedule_k3", "withholding"])
            || issue.key == "K3_WITHHOLDING_NEEDS_EACH_PASSIVE_COUNTRY")
    {
        let key = alloc::format!("income/{source_id}/k3_withholding");
        let (_, _, index) =
            submitted.iter().find(|(id, found, _)| *id == source_id && *found == key)?;
        let issue = if issue.key == "K3_WITHHOLDING_NEEDS_EACH_PASSIVE_COUNTRY" {
            ValidationIssue::new(issue.key).at_field("value")
        } else {
            issue.map_field_prefix(&["schedule_k3", "withholding"], &["value"])
        };
        return Some(issue.in_field("answer").in_list("followups", *index).into());
    }
    let partner_page = match source {
        K::K1Partnership(w) => Some(w.calculation_page()),
        _ => None,
    };
    let scorp_page = match source {
        K::K1Scorp(w) => Some(w.source().page()),
        _ => None,
    };
    let (credits, _properties, container) = match source {
        K::K1Partnership(_) => {
            let w = partner_page.as_ref()?;
            (&w.more.credits, &w.more.investment_property, "more")
        }
        K::K1Scorp(_) => {
            let w = scorp_page.as_ref()?;
            (&w.more.credits, &w.more.investment_property, "more")
        }
        K::K1PartnershipActivities(w) => {
            (&w.entity.credits, &w.entity.investment_property, "entity")
        }
        K::K1ScorpActivities(w) => (&w.entity.credits, &w.entity.investment_property, "entity"),
        _ => return None,
    };
    let row_index = |field: &str| {
        let key = alloc::format!("income/{source_id}/{field}");
        submitted
            .iter()
            .find(|(id, found, _)| *id == source_id && *found == key)
            .map(|(_, _, index)| *index)
    };
    let target = |field: &str| {
        row_index(field).and_then(|index| {
            ValidationIssue::new(issue.key)
                .at_record()
                .in_field("answer")
                .in_list("followups", index)
                .field
        })
    };
    if issue.has_field_prefix(&[container, "credits"]) {
        let targets = credits.iter().map(|credit| target(&credit_key(credit))).collect::<Vec<_>>();
        return Some(
            SaveFailure::from(issue).map_submitted_items(&[container, "credits"], &targets),
        );
    }
    if issue.has_field_prefix(&[container, "investment_property"]) {
        let index = row_index("investment")?;
        return Some(
            issue
                .map_field_prefix(&[container, "investment_property"], &["properties"])
                .in_field("answer")
                .in_list("followups", index)
                .into(),
        );
    }
    if container == "entity" {
        if issue.has_field_prefix(&["entity", "disposition"]) {
            return Some(
                issue
                    .map_field_prefix(&["entity", "disposition"], &["value"])
                    .in_field("answer")
                    .in_list("followups", row_index("disposition")?)
                    .into(),
            );
        }
        let field = if issue.has_field_prefix(&["entity", "housing_confirmed"]) {
            "housing"
        } else if issue.has_field_prefix(&["entity", "investment_confirmed"]) {
            "investment"
        } else {
            return None;
        };
        let index =
            row_index(if field == "housing" { "credit_confirmations" } else { "investment" })?;
        return Some(
            issue
                .map_field_prefix(
                    &[
                        "entity",
                        if field == "housing" {
                            "housing_confirmed"
                        } else {
                            "investment_confirmed"
                        },
                    ],
                    &[field],
                )
                .in_field("answer")
                .in_list("followups", index)
                .into(),
        );
    }
    let fields: &[(&str, &[&'static str], &[&'static str])] = &[
        ("section179", &["more", "section179_carryover"], &["carryover"]),
        ("section179", &["more", "section179_carryover_sold"], &["sold"]),
        ("section179_amt", &["more", "amt_section179_income_carry"], &["value"]),
        ("credit_confirmations", &["more", "housing_confirmed"], &["housing"]),
        ("investment", &["more", "investment_confirmed"], &["investment"]),
        ("basis_start", &["basis", "start"], &["value"]),
        ("participation", &["participation"], &["value"]),
        ("limited_for_se", &["limited_for_se"], &["value"]),
        ("amt", &["amt"], &["value"]),
        ("disposition", &["disposition"], &["value"]),
        ("rental_participation", &["rentals", "participation"], &["value"]),
        ("rentals", &["rentals"], &[]),
        ("records", &["more"], &[]),
        ("records", &[], &[]),
    ];
    let (field, from, to) = fields.iter().find(|(_, from, _)| issue.has_field_prefix(from))?;
    let index = row_index(field)?;
    Some(issue.map_field_prefix(from, to).in_field("answer").in_list("followups", index).into())
}

pub(super) fn scorp_topic_issue(income: &FederalIncome, required: bool) -> Option<ValidationIssue> {
    let FederalIncome::Scorp { data } = income else { return None };
    data.topic_issue(required)
}

pub(super) fn apply_scorp(
    source: &mut crate::ScorpSource,
    accepted_key: &str,
    answer: A,
) -> Result<(), ValidationIssue> {
    let invalid = || ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("value");
    let suffix = accepted_key.splitn(3, '/').nth(2).ok_or_else(invalid)?;
    match answer {
        A::K1ResearchCreditLine { value } => {
            if value.is_some_and(|line| !crate::K1CreditCode::Research.line_allows(line)) {
                return Err(invalid());
            }
            source.personal.research_credit_line = value;
        }
        A::K1K3Withholding { value } => source.personal.k3_withholding = value,
        answer => {
            let mut page = source.page();
            match answer {
                A::K1ScorpParticipation { value } => page.participation = value,
                A::K1ScorpBasisStart { value: Some(value) } => page.basis.start = value,
                A::K1ScorpBasisStart { value: None } => {}
                A::K1ScorpAmt { value } => page.amt = value,
                A::K1ScorpDisposition { value } => page.disposition = value,
                A::K1ScorpRecords {
                    qbi_records,
                    basis,
                    special,
                    qbi_group,
                    stock_term,
                    trading,
                } => {
                    page.qbi_records = Some(qbi_records);
                    page.basis = basis.stored(page.basis.start.clone());
                    page.special = special;
                    page.qbi_group = qbi_group;
                    page.more.stock_term = stock_term;
                    page.more.trading = trading;
                }
                answer => common!(&mut page, answer, invalid, suffix),
            }
            source.replace_personal_page(&page);
        }
    }
    Ok(())
}

pub(super) fn estate_questions(source: &crate::EstateTrustSource) -> Vec<Question> {
    let mut rows = super::estate_followups::questions(&source.page());
    rows.extend(source.moved_questions());
    rows
}
pub(super) fn estate_topic_issue(
    income: &FederalIncome,
    required: bool,
) -> Option<ValidationIssue> {
    let FederalIncome::EstateTrustPortfolio { data } = income else { return None };
    if let Some(issue) = data.admission_issue() {
        return Some(issue);
    }
    if required
        && data
            .statements
            .reported_credits
            .iter()
            .any(|credit| credit.code_name() == "I" && credit.research_line.is_none())
    {
        return Some(
            ValidationIssue::new("DOCUMENT_K1_RESEARCH_CREDIT_LINE").at_field("research_line"),
        );
    }
    data.page().issue_with_mode(required).or_else(|| {
        super::estate_followups::answer_issue(
            &crate::FederalEstateTrustPortfolioK1::from(data),
            required,
        )
    })
}
pub(super) fn apply_estate(
    source: &mut crate::EstateTrustSource,
    key: &str,
    answer: A,
) -> Result<(), ValidationIssue> {
    if let Some(issue) = source.admission_issue() {
        return Err(issue);
    }
    let suffix = key
        .splitn(3, '/')
        .nth(2)
        .ok_or_else(|| ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("value"))?;
    if matches!(
        answer,
        A::EstateRoyaltyCharacter { .. }
            | A::EstateAccounting { .. }
            | A::EstateFinalTaxCharacter { .. }
            | A::EstateNolRecords { .. }
            | A::EstateNiitAllocation { .. }
            | A::EstateOtherSource { .. }
            | A::EstateResearchCreditLine { .. }
    ) {
        return source.apply_moved_answer(suffix, answer);
    }
    let mut page = source.page();
    super::estate_followups::apply(&mut page, suffix, answer)?;
    source.replace_personal_page(&page);
    Ok(())
}

// Participation is asked only with rental real estate: once every row of a
// save is applied, a K-1 that no longer has it drops the unasked answer.
pub(super) fn prune_rental_participation(source: &mut IncomeSource) {
    if let IncomeSource::K1 { value: K::K1Partnership(w) } = source
        && !w.record().rental_participation_needed()
    {
        let mut page = w.calculation_page();
        if page.rentals.participation.take().is_some() {
            w.replace_personal_page(&page);
        }
    }
}

pub(super) fn prune_scorp_rental_participation(source: &mut crate::ScorpSource) {
    if !crate::ScorpK1::from(&*source).rental_participation_needed() {
        let mut page = source.page();
        if page.rentals.participation.take().is_some() {
            source.replace_personal_page(&page);
        }
    }
}

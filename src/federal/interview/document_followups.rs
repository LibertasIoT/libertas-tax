use super::*;
use crate::{DocumentAnswer as A, DocumentQuestion};

// A printed document may belong to one category while its personal answers
// belong to another topic. The paper and its accepted source keep their owner.
pub(crate) fn answer_topic(
    draft: &FederalDraft,
    entry: &FederalIncomeEntry,
    answer: &A,
) -> Option<TaxSection> {
    if matches!(answer, A::Management { .. }) {
        return Some(TaxSection::InjuredSpouse);
    }
    match entry.income {
        FederalIncome::EducationDistribution { .. } => Some(TaxSection::Education),
        FederalIncome::LtcBenefits { .. } | FederalIncome::LtcInsured { .. } => {
            Some(TaxSection::LongTermCareBenefits)
        }
        FederalIncome::MedicalDistribution { .. } => Some(TaxSection::Hsa),
        _ => topic_of(draft, entry),
    }
}

pub(crate) fn followup_topic(
    draft: &FederalDraft,
    entry: &FederalIncomeEntry,
) -> Option<TaxSection> {
    answer_topic(draft, entry, &A::Community { value: CommunityShare::NotNeeded })
}

// A Form 1098 named once, as every document is: its form, lender and the borrower it was
// printed for; the property label only when no name is printed. Never an empty name.
pub(super) fn mortgage_document_name(source: &crate::Mortgage1098Source) -> Vec<u8> {
    let kind = || MessageArg::Encoded(status("FORM_FORM1098"));
    let lender = source.form.lender.name.trim();
    let borrower = source.form.borrower.name.trim();
    let property = source.property.trim();
    match (lender.is_empty(), borrower.is_empty()) {
        (false, false) => Message::new(
            "DOCUMENT_CHOICE",
            alloc::vec![kind(), MessageArg::Text(lender.into()), MessageArg::Text(borrower.into())],
        ),
        (false, true) => {
            Message::new("DOCUMENT_FROM", alloc::vec![kind(), MessageArg::Text(lender.into())])
        }
        (true, false) => Message::new(
            "DOCUMENT_PRINTED_FOR",
            alloc::vec![kind(), MessageArg::Text(borrower.into())],
        ),
        (true, true) if !property.is_empty() => {
            Message::new("TEXT", alloc::vec![MessageArg::Text(property.into())])
        }
        (true, true) => Message::new("TEXT", alloc::vec![kind()]),
    }
    .encode()
}

fn followup_prompt(question: &str, document: Vec<u8>) -> Vec<u8> {
    Message::new(
        "DOCUMENT_TOPIC_SOURCE_ITEM",
        alloc::vec![MessageArg::Resource(question.into()), MessageArg::Encoded(document)],
    )
    .encode()
}

pub(super) fn followup_document_name(draft: &FederalDraft, entry: &FederalIncomeEntry) -> Vec<u8> {
    super::document_owners::document_name(draft, entry.id).unwrap_or_else(|| {
        let kind = document_kind_keys(entry).0;
        if kind.starts_with("SAVING_KIND_") {
            status(&alloc::format!("{kind}_ONE"))
        } else {
            status(kind)
        }
    })
}

fn followup_question_document(
    draft: &FederalDraft,
    entry: &FederalIncomeEntry,
    field: &str,
) -> Vec<u8> {
    let document = followup_document_name(draft, entry);
    let detail = match &entry.income {
        FederalIncome::Cooperative { data } => {
            cooperative_followups::description(&crate::Patr1099::from(data), field)
        }
        _ => None,
    };
    detail.map_or_else(
        || document.clone(),
        |item| {
            Message::new(
                "DOCUMENT_TOPIC_SOURCE_ITEM",
                alloc::vec![MessageArg::Encoded(document.clone()), MessageArg::Encoded(item)],
            )
            .encode()
        },
    )
}

// Locate a changed actual rollover fact without reimplementing its owner's
// relationship checks. Unchanged or structurally replaced records use the
// complete payment control rather than a guessed child.
fn changed_medical_payment_issue(
    key: &'static str,
    before: &FederalIncome,
    after: &FederalIncome,
) -> ValidationIssue {
    let issue = ValidationIssue::new(key);
    if let (
        FederalIncome::MedicalDistribution { data: before },
        FederalIncome::MedicalDistribution { data: after },
    ) = (before, after)
        && let (
            Some(crate::MedicalSaPayment::Own { rollover: Some(before), .. }),
            Some(crate::MedicalSaPayment::Own { rollover: Some(after), .. }),
        ) = (&before.payment, &after.payment)
    {
        for (i, (before, after)) in before.receipts.iter().zip(&after.receipts).enumerate() {
            let at_receipt = |issue: ValidationIssue| {
                issue.in_list("receipts", i).in_field("rollover").in_field("value")
            };
            if before.received != after.received {
                return at_receipt(issue.at_field("received"));
            }
            for (j, (before, after)) in before.deposits.iter().zip(&after.deposits).enumerate() {
                let field = if before.amount != after.amount {
                    "amount"
                } else if before.deposited != after.deposited {
                    "deposited"
                } else if before.destination != after.destination {
                    "destination"
                } else if before.character != after.character {
                    "character"
                } else {
                    continue;
                };
                return at_receipt(issue.at_field(field).in_list("deposits", j));
            }
        }
    }
    issue.at_field("value")
}

fn answer_kind(answer: &A) -> i32 {
    if let Some(kind) = k1_followups::answer_kind(answer) {
        return kind;
    }
    use libertas_macros::variant_index;
    match answer {
        A::BondRedemptionForm8815 { .. } => variant_index!(A::BondRedemptionForm8815) as i32,
        A::SavingsBondsRecords { .. } => variant_index!(A::SavingsBondsRecords) as i32,
        A::EsppAcquisition { .. } => variant_index!(A::EsppAcquisition) as i32,
        A::EsppFixedGrantPrice { .. } => variant_index!(A::EsppFixedGrantPrice) as i32,
        A::EsppAdjustments { .. } => variant_index!(A::EsppAdjustments) as i32,
        A::EsppWageInclusion { .. } => variant_index!(A::EsppWageInclusion) as i32,
        A::IsoAdjustments { .. } => variant_index!(A::IsoAdjustments) as i32,
        A::IsoWageInclusion { .. } => variant_index!(A::IsoWageInclusion) as i32,
        A::NuaAdjustments { .. } => variant_index!(A::NuaAdjustments) as i32,
        A::NuaOwnership { .. } => variant_index!(A::NuaOwnership) as i32,
        A::SocialPremiums { .. } => variant_index!(A::SocialPremiums) as i32,
        A::SocialReimbursedPremiums { .. } => variant_index!(A::SocialReimbursedPremiums) as i32,
        A::SocialSharedPremiums { .. } => variant_index!(A::SocialSharedPremiums) as i32,
        A::DebtTreatment { .. } => variant_index!(A::DebtTreatment) as i32,
        A::DebtProperty { .. } => variant_index!(A::DebtProperty) as i32,
        A::DebtAcquisition { .. } => variant_index!(A::DebtAcquisition) as i32,
        A::LtcInterest { .. } => variant_index!(A::LtcInterest) as i32,
        A::LtcBenefitOrigin { .. } => variant_index!(A::LtcBenefitOrigin) as i32,
        A::LtcAcceleratedOrigin { .. } => variant_index!(A::LtcAcceleratedOrigin) as i32,
        A::LtcContract { .. } => variant_index!(A::LtcContract) as i32,
        A::LtcInsured { .. } => variant_index!(A::LtcInsured) as i32,
        A::RailroadRecipient { .. } => variant_index!(A::RailroadRecipient) as i32,
        A::RailroadRepayments { .. } => variant_index!(A::RailroadRepayments) as i32,
        A::Wage { .. } => variant_index!(A::Wage) as i32,
        A::RetirementTreatment { .. } => variant_index!(A::RetirementTreatment) as i32,
        A::RetirementMethod { .. } => variant_index!(A::RetirementMethod) as i32,
        A::RetirementBasis { .. } => variant_index!(A::RetirementBasis) as i32,
        A::RetirementDisability { .. } => variant_index!(A::RetirementDisability) as i32,
        A::Community { .. } => variant_index!(A::Community) as i32,
        A::Management { .. } => variant_index!(A::Management) as i32,
        A::RetirementDisaster { .. } => variant_index!(A::RetirementDisaster) as i32,
        A::RetirementPublicSafety { .. } => variant_index!(A::RetirementPublicSafety) as i32,
        A::RetirementAfterDeath { .. } => variant_index!(A::RetirementAfterDeath) as i32,
        A::InterestNominee { .. } => variant_index!(A::InterestNominee) as i32,
        A::InterestAfterDeath { .. } => variant_index!(A::InterestAfterDeath) as i32,
        A::InterestAdjustment { .. } => variant_index!(A::InterestAdjustment) as i32,
        A::InterestAccrued { .. } => variant_index!(A::InterestAccrued) as i32,
        A::InterestTreatment { .. } => variant_index!(A::InterestTreatment) as i32,
        A::OidNominee { .. } => variant_index!(A::OidNominee) as i32,
        A::OidPremium { .. } => variant_index!(A::OidPremium) as i32,
        A::OidPrivateActivity { .. } => variant_index!(A::OidPrivateActivity) as i32,
        A::OidTreatment { .. } => variant_index!(A::OidTreatment) as i32,
        A::RentalSection179Business { .. } => variant_index!(A::RentalSection179Business) as i32,
        A::RentalHomeCarries { .. } => variant_index!(A::RentalHomeCarries) as i32,
        A::DividendNominee { .. } => variant_index!(A::DividendNominee) as i32,
        A::DividendEligibility { .. } => variant_index!(A::DividendEligibility) as i32,
        A::DividendAfterDeath { .. } => variant_index!(A::DividendAfterDeath) as i32,
        A::DividendFundExclusion { .. } => variant_index!(A::DividendFundExclusion) as i32,
        A::DividendCapitalReturns { .. } => variant_index!(A::DividendCapitalReturns) as i32,
        A::DividendTreatment { .. } => variant_index!(A::DividendTreatment) as i32,
        A::BrokerSaleRecords { .. } => variant_index!(A::BrokerSaleRecords) as i32,
        A::BrokerContractsTreatment { .. } => variant_index!(A::BrokerContractsTreatment) as i32,
        A::SaleAdjustment { .. } => variant_index!(A::SaleAdjustment) as i32,
        A::SaleTreatment { .. } => variant_index!(A::SaleTreatment) as i32,
        A::UndistributedGainSpecial { .. } => variant_index!(A::UndistributedGainSpecial) as i32,
        A::ContractsSpecial { .. } => variant_index!(A::ContractsSpecial) as i32,
        A::IsoExerciseFacts { .. } => variant_index!(A::IsoExerciseFacts) as i32,
        A::OpportunityFundEvent { .. } => variant_index!(A::OpportunityFundEvent) as i32,
        A::OpportunityFundSpecial { .. } => variant_index!(A::OpportunityFundSpecial) as i32,
        A::GovernmentGrantUse { .. } => variant_index!(A::GovernmentGrantUse) as i32,
        A::GovernmentMedicalLeave { .. } => variant_index!(A::GovernmentMedicalLeave) as i32,
        A::GovernmentRepayments { .. } => variant_index!(A::GovernmentRepayments) as i32,
        A::GovernmentSpecial { .. } => variant_index!(A::GovernmentSpecial) as i32,
        A::GamblingSpecial { .. } => variant_index!(A::GamblingSpecial) as i32,
        A::RefundDeduction { .. } => variant_index!(A::RefundDeduction) as i32,
        A::HomeSale { .. } => variant_index!(A::HomeSale) as i32,
        A::CooperativeAllocations { .. } => variant_index!(A::CooperativeAllocations) as i32,
        A::CooperativeCreditRecords { .. } => variant_index!(A::CooperativeCreditRecords) as i32,
        A::CooperativeOtherRecords { .. } => variant_index!(A::CooperativeOtherRecords) as i32,
        A::NonemployeeUse { .. } => variant_index!(A::NonemployeeUse) as i32,
        A::MiscellaneousUse { .. } => variant_index!(A::MiscellaneousUse) as i32,
        A::AttorneyClientFunds { .. } => variant_index!(A::AttorneyClientFunds) as i32,
        A::PaymentParts(..) => variant_index!(A::PaymentParts) as i32,
        A::AbleTreatment { .. } => variant_index!(A::AbleTreatment) as i32,
        A::EducationBeneficiary { .. } => variant_index!(A::EducationBeneficiary) as i32,
        A::CoverdellRecords { .. } => variant_index!(A::CoverdellRecords) as i32,
        A::EducationRedeposited { .. } => variant_index!(A::EducationRedeposited) as i32,
        A::EducationTaxableTransfer { .. } => variant_index!(A::EducationTaxableTransfer) as i32,
        A::EducationRothRecords { .. } => variant_index!(A::EducationRothRecords) as i32,
        A::MedicalPayment { .. } => variant_index!(A::MedicalPayment) as i32,
        A::MedicareSpousal { .. } => variant_index!(A::MedicareSpousal) as i32,
        A::MedicarePrior { .. } => variant_index!(A::MedicarePrior) as i32,
        A::QsbsBoolean { .. } => variant_index!(A::QsbsBoolean) as i32,
        A::QsbsPriorGain { .. } => variant_index!(A::QsbsPriorGain) as i32,
        A::QsbsZone { .. } => variant_index!(A::QsbsZone) as i32,
        A::QsbsZoneGain { .. } => variant_index!(A::QsbsZoneGain) as i32,
        _ => unreachable!("every K-1 answer has a native kind"),
    }
}

// Source edits retain personal answers from the accepted native record. Hidden
// source echoes cannot supply or clear answers owned by the topic page.
pub(super) fn retain_source_answers(draft: &FederalDraft, items: &mut [IncomeDocumentItem]) {
    for item in items {
        let accepted = draft
            .income
            .iter()
            .find(|entry| entry.id == item.id)
            .and_then(|entry| document_item(entry, &[], false));
        item.community = accepted.as_ref().map_or_else(
            || {
                if document_community_answers(draft, &item.document).len() == 1 {
                    fixed_community(draft)
                } else {
                    CommunityShare::NotNeeded
                }
            },
            |old| old.community,
        );
        item.refund_management = accepted.as_ref().and_then(|old| old.refund_management);
        let old = accepted.as_ref().map(|old| &old.document);
        k1_followups::retain_source_answers(&mut item.document, old);
        {}
    }
}

// Bind native validation paths to the one editable question. The original
// validators remain authoritative; this only locates their field on the topic.
type QuestionField = (&'static str, &'static [&'static str], &'static [&'static str]);

pub(super) fn question_field(issue: &ValidationIssue) -> Option<QuestionField> {
    let fields: &[QuestionField] = &[
        ("ltc_benefit_origin", &["benefit_origin"], &["value"]),
        ("ltc_accelerated_origin", &["accelerated_origin"], &["value"]),
        ("ltc_contract", &["contract"], &["value"]),
        ("ltc_insured", &["insured"], &["value"]),
        ("railroad_recipient", &["recipient"], &["value"]),
        ("railroad_repayments", &["repayments"], &["value"]),
        ("reit", &["answers", "reit_eligible"], &["value"]),
        ("after_death", &["answers", "after_death"], &["value"]),
        ("wage", &["answers"], &["answers"]),
        ("community", &["community"], &["value"]),
        ("basis", &["more", "box_10_basis"], &["basis"]),
        ("method", &["more", "taxable_method"], &["method"]),
        ("disability", &["more", "disability"], &["disability"]),
        ("treatment", &["more", "box_3"], &["code"]),
        ("treatment", &["more", "box_6"], &["code"]),
        ("treatment", &["more", "box_8a"], &["code"]),
        ("treatment", &["more", "box_8b_percent"], &["code"]),
        ("treatment", &["more", "box_9a_percent"], &["code"]),
        ("treatment", &["more", "box_9b"], &["code"]),
        ("treatment", &["more", "box_10"], &["code"]),
        // The 1099-R source keeps these boxes and answers outside `more`.
        ("basis", &["box_10_basis"], &["basis"]),
        ("method", &["taxable_method"], &["method"]),
        ("disability", &["disability"], &["disability"]),
        ("treatment", &["box_3"], &["code"]),
        ("treatment", &["box_6"], &["code"]),
        ("treatment", &["box_8a"], &["code"]),
        ("treatment", &["box_8b_percent"], &["code"]),
        ("treatment", &["box_9a_percent"], &["code"]),
        ("treatment", &["box_9b"], &["code"]),
        ("treatment", &["box_10"], &["code"]),
        ("treatment", &["code"], &["code"]),
        ("treatment", &["treatment"], &["value"]),
        ("beneficiary", &["beneficiary"], &["value"]),
        ("coverdell", &["coverdell_basis_start"], &["opening"]),
        ("coverdell", &["coverdell_contributions"], &["contributions"]),
        ("coverdell", &["late_rollovers"], &["late_rollovers"]),
        ("redeposited", &["redeposited"], &["value"]),
        ("taxable_transfer", &["taxable_transfer_part"], &["value"]),
        ("roth", &["roth_rollover"], &["value"]),
        ("payment", &["payment"], &["value"]),
        ("medicare_spousal", &["medicare_from_spouse"], &["value"]),
        ("medicare_prior", &["medicare_prior"], &["value"]),
        ("nominee", &["nominee"], &["value"]),
        ("capital_returns", &["capital_returns"], &["value"]),
        ("event", &["event"], &["value"]),
        ("after_death", &["after_death"], &["value"]),
        ("adjustments", &["adjustments"], &["value"]),
        ("accrued_interest", &["more", "accrued_interest"], &["value"]),
        ("special", &["more", "special"], &["value"]),
        ("premium", &["more", "tax_exempt_bond_premium"], &["value"]),
        ("premium", &["premium"], &["value"]),
        ("private_activity", &["private_activity"], &["value"]),
        ("private_activity", &["more", "private_activity"], &["value"]),
        ("exclusion", &["more", "section_1202_exclusion"], &["value"]),
        ("exclusion", &["section_1202_exclusion"], &["value"]),
        ("adjustment", &["adjustment"], &["value"]),
        ("special", &["special"], &["value"]),
        ("grant", &["more", "grant_use"], &["value"]),
        ("medical", &["more", "family_leave_medical"], &["value"]),
        ("repayments", &["more", "repayments"], &["value"]),
        ("allocations", &["more", "allocations"], &["value"]),
        ("use", &["belongs"], &["value"]),
        ("use", &["other_use"], &["value"]),
        ("client_funds", &["client_funds"], &["value"]),
        ("parts", &["parts"], &["value"]),
        ("disaster", &["disaster"], &["value"]),
        ("public_safety", &["public_safety"], &["value"]),
    ];
    fields.iter().copied().find(|(_, path, _)| issue.has_field_prefix(path))
}

fn investment_answer_conflict(
    document: &IncomeSource,
    income: Option<&FederalIncome>,
) -> Option<(&'static str, ValidationIssue)> {
    // A 1099-INT, -DIV or -OID's answers are those of its accepted source, not of its paper.
    let printed;
    let dividend = match (document, income) {
        (_, Some(FederalIncome::Dividend { data })) => Some(data),
        (
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Div1099(form),
            },
            _,
        ) => {
            printed = crate::DividendSource::printed(form.clone());
            Some(&printed)
        }
        _ => None,
    };
    let printed_interest;
    let interest = match (document, income) {
        (_, Some(FederalIncome::Interest { data: crate::InterestSource::Int1099(data) })) => {
            Some(data)
        }
        (
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(form),
            },
            _,
        ) => {
            printed_interest = crate::Form1099IntSource::printed(form.clone());
            Some(&printed_interest)
        }
        _ => None,
    };
    let printed_oid;
    let oid = match (document, income) {
        (_, Some(FederalIncome::Oid { data })) => Some(data),
        (
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Oid1099(form),
            },
            _,
        ) => {
            printed_oid = crate::Form1099OidSource::printed(form.clone());
            Some(&printed_oid)
        }
        _ => None,
    };
    let issue = match (document, dividend) {
        (
            IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Div1099(_) },
            Some(source),
        ) => dividend_source_issue(source, true)?,
        (
            IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Int1099(_) },
            _,
        ) => int_source_issue(interest?, true)?,
        (
            IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Oid1099(_) },
            _,
        ) => oid_source_issue(oid?, true)?,
        _ => investment_statement_issue(document)?,
    };
    let field = match document {
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Int1099(_) } => {
            let value = interest?;
            if value
                .adjustments
                .as_ref()
                .is_some_and(|adjustments| adjustments.instrument_records.is_some())
                || issue.has_field_prefix(&["adjustments"])
            {
                "adjustments"
            } else if value.nominee.is_some() {
                "nominee"
            } else {
                "after_death"
            }
        }
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Div1099(_) } => {
            let value = dividend?;
            if issue.key == "DIV_CAPITAL_RETURNS_RECONCILE"
                || issue.has_field_prefix(&["capital_returns"])
            {
                "capital_returns"
            } else if value.nominee.is_some() {
                "nominee"
            } else {
                "after_death"
            }
        }
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Oid1099(_) } => {
            let value = oid?;
            if value
                .adjustments
                .as_ref()
                .is_some_and(|adjustments| adjustments.instrument_records.is_some())
            {
                "adjustments"
            } else if issue.key == "OID_PRIVATE_ACTIVITY_ABOVE_EXEMPT" {
                "private_activity"
            } else if issue.key == "EXEMPT_REDUCTION_ABOVE_AMOUNT"
                && value.tax_exempt_bond_premium > value.form.tax_exempt_oid
            {
                "premium"
            } else {
                "nominee"
            }
        }
        _ => return None,
    };
    Some((field, if issue.has_field_prefix(&[field]) { issue } else { issue.at_field(field) }))
}

// The answers of a broker form collected in one topic save start from its accepted source.
fn broker_source<'a>(
    work: &'a mut alloc::collections::BTreeMap<i64, crate::BrokerDocument>,
    draft: &FederalDraft,
    id: i64,
) -> Option<&'a mut crate::BrokerDocument> {
    if let alloc::collections::btree_map::Entry::Vacant(slot) = work.entry(id) {
        let FederalIncome::Broker { data } =
            &draft.income.iter().find(|entry| entry.id == id)?.income
        else {
            return None;
        };
        slot.insert(data.clone());
    }
    work.get_mut(&id)
}

// Filer-weight estimates for matching question cohorts order their rows.
// An unweighted question remains unknown and keeps its existing relative order.
fn followup_filer_weight(answer: &A) -> Option<u64> {
    match answer {
        A::Wage { .. } => Some(415_779),
        A::DividendTreatment { .. } => Some(369_435),
        A::InterestNominee { .. } | A::OidNominee { .. } | A::DividendNominee { .. } => {
            Some(207_154)
        }
        A::CooperativeCreditRecords { .. } | A::CooperativeOtherRecords { .. } => Some(182_568),
        A::SavingsBondsRecords { .. } | A::InterestAdjustment { .. } => Some(155_790),
        A::LtcContract { .. } => Some(33_609),
        _ => None,
    }
}

impl Interview {
    pub(super) fn document_followups(
        &self,
        topic: TaxSection,
        result: &FederalResult,
    ) -> Vec<DocumentQuestion> {
        Self::document_followup_rows(&self.draft, Some(topic), Some(result))
            .into_iter()
            .map(|(_, row)| row)
            .collect()
    }

    pub(super) fn wages_followups(&self) -> Vec<crate::document_center::WagesDocumentQuestion> {
        self.document_followups(TaxSection::Wages, &self.income_result())
            .into_iter()
            .filter(|row| !matches!(row.answer, A::Wage { .. }))
            .map(|row| {
                crate::document_center::WagesDocumentQuestion::from_canonical(row)
                    .expect("the additional wages questions contain community answers")
            })
            .collect()
    }

    pub(super) fn education_followups(
        &self,
    ) -> Vec<crate::document_center::EducationDocumentQuestion> {
        self.document_followups(TaxSection::Education, &self.income_result())
            .into_iter()
            .map(|row| {
                crate::document_center::EducationDocumentQuestion::from_canonical(row)
                    .expect("the education walk emits education-account answers")
            })
            .collect()
    }

    pub(super) fn education_followup_candidate(
        &self,
        supplied: &[crate::document_center::EducationDocumentQuestion],
        require_answers: bool,
    ) -> Result<Self, SaveFailure> {
        let expected = Self::document_followup_rows(
            &self.draft,
            Some(TaxSection::Education),
            Some(&self.income_result()),
        );
        let rows = supplied
            .iter()
            .map(|row| {
                let expected = expected.iter().find(|(_, expected)| expected.key == row.key);
                expected
                    .and_then(|(_, expected)| row.clone().into_canonical(expected.clone()))
                    .ok_or_else(|| {
                        SaveFailure::from(
                            ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                                .at_field("key")
                                .in_key("followups", row.key.as_str()),
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.followup_candidate_with_mode(TaxSection::Education, rows, require_answers, &[])
    }

    pub(super) fn hsa_followups(&self) -> Vec<crate::document_center::HsaDocumentQuestion> {
        self.document_followups(TaxSection::Hsa, &self.income_result())
            .into_iter()
            .map(|row| {
                crate::document_center::HsaDocumentQuestion::from_canonical(row)
                    .expect("the medical-account walk emits medical-account answers")
            })
            .collect()
    }

    pub(super) fn hsa_followup_candidate(
        &self,
        supplied: &[crate::document_center::HsaDocumentQuestion],
        require_answers: bool,
    ) -> Result<Self, SaveFailure> {
        let expected = Self::document_followup_rows(
            &self.draft,
            Some(TaxSection::Hsa),
            Some(&self.income_result()),
        );
        let rows = supplied
            .iter()
            .map(|row| {
                let expected = expected.iter().find(|(_, expected)| expected.key == row.key);
                expected
                    .and_then(|(_, expected)| row.clone().into_canonical(expected.clone()))
                    .ok_or_else(|| {
                        SaveFailure::from(
                            ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                                .at_field("key")
                                .in_key("followups", row.key.as_str()),
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.followup_candidate_with_mode(TaxSection::Hsa, rows, require_answers, &[])
    }

    pub(super) fn injured_spouse_followups_for(
        draft: &FederalDraft,
        result: &FederalResult,
    ) -> Vec<crate::document_center::InjuredSpouseDocumentQuestion> {
        Self::document_followup_rows(draft, Some(TaxSection::InjuredSpouse), Some(result))
            .into_iter()
            .map(|(_, row)| {
                crate::document_center::InjuredSpouseDocumentQuestion::from_canonical(row)
                    .expect("the injured-spouse walk emits injured-spouse answers")
            })
            .collect()
    }

    pub(super) fn injured_spouse_followup_candidate(
        &self,
        supplied: &[crate::document_center::InjuredSpouseDocumentQuestion],
        require_answers: bool,
    ) -> Result<Self, SaveFailure> {
        let expected = Self::document_followup_rows(
            &self.draft,
            Some(TaxSection::InjuredSpouse),
            Some(&self.income_result()),
        );
        let rows = supplied
            .iter()
            .map(|row| {
                let expected = expected.iter().find(|(_, expected)| expected.key == row.key);
                expected
                    .and_then(|(_, expected)| row.clone().into_canonical(expected.clone()))
                    .ok_or_else(|| {
                        SaveFailure::from(
                            ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                                .at_field("key")
                                .in_key("followups", row.key.as_str()),
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.followup_candidate_with_mode(TaxSection::InjuredSpouse, rows, require_answers, &[])
    }

    pub(super) fn document_followup_rows(
        draft: &FederalDraft,
        topic: Option<TaxSection>,
        result: Option<&FederalResult>,
    ) -> Vec<(i64, DocumentQuestion)> {
        let mut rows = Vec::new();
        // Reg. 1.179-2(c): a rental counts toward the business income limit
        // only when actively conducted as a trade or business; asked only
        // while the return has a section 179 deduction.
        let rental_section179 =
            draft.income.iter().any(|entry| matches!(entry.income, FederalIncome::Rental { .. }))
                && crate::section179::rental_business_asked(draft);
        for entry in &draft.income {
            // Management is the only answer that crosses the source's topic.
            // Other editors need not construct unrelated answer catalogs.
            if topic.is_some_and(|topic| {
                topic != TaxSection::InjuredSpouse && followup_topic(draft, entry) != Some(topic)
            }) {
                continue;
            }
            let Some(item) =
                document_item(entry, &[], crate::federal_rules::separate_return(draft))
            else {
                continue;
            };
            // A retained native capital distribution reopens through the same
            // paper mapping as its next save, including its exact accepted lot.
            let projected;
            let entry = if matches!(entry.income, FederalIncome::ReturnOfCapital { .. }) {
                projected = document_entry(item.clone(), core::slice::from_ref(entry));
                &projected
            } else {
                entry
            };
            let k1_source =
                matches!(item.document, IncomeSource::K1 { .. }).then(|| item.document.clone());
            let mut add = |field: &str, question: &str, answer: A, allowed_values: Vec<i32>| {
                if topic.is_some_and(|topic| answer_topic(draft, entry, &answer) != Some(topic)) {
                    return;
                }
                rows.push((
                    entry.id,
                    DocumentQuestion {
                        key: alloc::format!("income/{}/{field}", entry.id),
                        prompt: followup_prompt(
                            question,
                            followup_question_document(draft, entry, field),
                        ),
                        allowed_answers: alloc::vec![answer_kind(&answer)],
                        allowed_values,
                        k1: k1_source
                            .as_ref()
                            .map_or_else(crate::K1QuestionChoices::default, |source| {
                                k1_followups::catalog(draft, source, field)
                            }),
                        espp_acquisitions: crate::espp_return::choices(draft, entry.owner.owned()),
                        savings_bond_exclusions: savings_bond_exclusions(draft, entry.owner),
                        health_plans: crate::social_documents::plans(draft)
                            .map(|plan| crate::RecordChoice {
                                id: plan.id,
                                label: income_display_label(plan, draft.dependents()).encode(),
                            })
                            .collect(),
                        answer,
                    },
                ));
            };
            if let FederalIncome::ChildElection { data: child } = &entry.income {
                for statement in &child.statements {
                    let crate::ChildIncome::Dividend { statement_id, data } = statement else {
                        continue;
                    };
                    if data.form.nondividend_distributions.unwrap_or(0) != 0
                        || data.form.cash_liquidation.unwrap_or(0) != 0
                        || data.form.noncash_liquidation.unwrap_or(0) != 0
                        || !data.capital_returns.is_empty()
                    {
                        add(
                            &alloc::format!("statement/{statement_id}/capital_returns"),
                            "DOCUMENT_ANSWER_DIVIDENDCAPITALRETURNS",
                            A::DividendCapitalReturns { value: data.capital_returns.clone() },
                            Vec::new(),
                        );
                    }
                    add(
                        &alloc::format!("statement/{statement_id}/special"),
                        "DOCUMENT_ANSWER_DIVIDENDTREATMENT",
                        A::DividendTreatment { value: data.special },
                        Vec::new(),
                    );
                }
                continue;
            }
            if let FederalIncome::Rental { data } = &entry.income
                && rental_section179
            {
                add(
                    "section179_business",
                    "DOCUMENT_ANSWER_RENTALSECTION179BUSINESS",
                    A::RentalSection179Business { value: data.section179_business },
                    Vec::new(),
                );
            }
            // Last year's home-use worksheet carries apply with or without
            // personal use this year; asked while the checklist names them.
            if let FederalIncome::Rental { data } = &entry.income
                && draft.checklist.carryovers & crate::CarryoverTopics::HOME_USE_RENTAL.0 != 0
            {
                add(
                    "home_carries",
                    "DOCUMENT_ANSWER_RENTALHOMECARRIES",
                    A::RentalHomeCarries { value: data.prior_home_carry.clone() },
                    Vec::new(),
                );
            }
            if matches!(item.document, IncomeSource::Record { .. }) {
                continue;
            }
            if result.is_some_and(|result| {
                document_refund_management_needed(draft, result, &item) == Some(true)
            }) {
                add(
                    "management",
                    "INJURED_SPOUSE_WITHHOLDING_MANAGEMENT_NEEDED",
                    A::Management { value: item.refund_management },
                    crate::injured_spouse_payments::allowed_document_management(
                        crate::injured_spouse::texas(draft),
                    ),
                );
            }
            if topic == Some(TaxSection::InjuredSpouse) {
                continue;
            }
            if let FederalIncome::QualifiedSmallBusinessSale { data } = &entry.income {
                for (field, prompt, answer) in crate::qsbs_followups::questions(data) {
                    add(&field, prompt, answer, Vec::new());
                }
            }
            if let FederalIncome::EsppSaleSource { data: paper } = &entry.income {
                let answer = crate::espp_return::answers(draft, entry.id)
                    .cloned()
                    .unwrap_or_else(|| crate::EsppSaleAnswers::unanswered(entry.id));
                if !answer.more.special {
                    // A printed plan record supplies the acquisition facts.
                    if paper.plan_record.is_none() {
                        add(
                            "espp_acquisition",
                            "ESPP_ACQUISITION_NEEDED",
                            A::EsppAcquisition {
                                value: answer.acquisition.clone().map(Into::into),
                            },
                            Vec::new(),
                        );
                    }
                    if crate::espp_return::fixed_price_needed(draft, entry) {
                        add(
                            "espp_fixed_grant_price",
                            if paper.plan_record.is_some() {
                                "ESPP_RECORD_GRANT_PRICE_NEEDED"
                            } else {
                                "ESPP_FIXED_GRANT_PRICE_NEEDED"
                            },
                            A::EsppFixedGrantPrice {
                                value: crate::espp_return::fixed_price(draft, entry),
                            },
                            Vec::new(),
                        );
                    }
                }
                add(
                    "espp_adjustments",
                    "ESPP_SALE_ADJUSTMENTS",
                    A::EsppAdjustments { value: answer.more.clone() },
                    Vec::new(),
                );
                if crate::espp_return::wage_needed(draft, entry) {
                    add(
                        "espp_wage_inclusion",
                        "ESPP_WAGE_INCLUSION_NEEDED",
                        A::EsppWageInclusion { value: answer.wage_inclusion },
                        Vec::new(),
                    );
                }
            }
            if matches!(entry.income, FederalIncome::IsoSaleSource { .. }) {
                let answer = crate::iso_return::answers(draft, entry.id);
                add(
                    "iso_adjustments",
                    "ISO_SALE_ADJUSTMENTS_NEEDED",
                    A::IsoAdjustments { value: answer.and_then(|a| a.more.clone()) },
                    Vec::new(),
                );
                if crate::iso_return::wage_needed(draft, entry) {
                    add(
                        "iso_wage_inclusion",
                        "ISO_WAGE_INCLUSION_NEEDED",
                        A::IsoWageInclusion { value: answer.and_then(|a| a.wage_inclusion) },
                        Vec::new(),
                    );
                }
            }
            if matches!(entry.income, FederalIncome::NuaSaleSource { .. }) {
                let answer = crate::nua_sale::answers(draft, entry.id);
                add(
                    "nua_adjustments",
                    "NUA_SALE_ADJUSTMENTS_NEEDED",
                    A::NuaAdjustments { value: answer.and_then(|a| a.more.clone()) },
                    Vec::new(),
                );
                if crate::nua_sale::ownership_needed(draft, entry.id) {
                    add(
                        "nua_ownership",
                        "NUA_OWNERSHIP_NEEDED",
                        A::NuaOwnership { value: answer.and_then(|a| a.ownership.clone()) },
                        Vec::new(),
                    );
                }
            }
            if crate::social_documents::printed_premiums(entry) > 0 {
                let plan = crate::social_documents::plans(draft).next().is_some();
                let hsa = crate::medical_account_return::qualified_hsa_spending(draft) > 0;
                let value =
                    crate::social_documents::allocation(draft, entry.id).cloned().map(Into::into);
                let reimbursed = crate::social_documents::reimbursement(draft, entry.id);
                let answer = match (plan, hsa) {
                    (true, true) => Some(A::SocialSharedPremiums { value, reimbursed }),
                    (true, false) => Some(A::SocialPremiums { value }),
                    (false, true) => Some(A::SocialReimbursedPremiums { reimbursed }),
                    (false, false) => None,
                };
                if let Some(answer) = answer {
                    add(
                        "social_premiums",
                        if hsa {
                            "SOCIAL_PREMIUM_REIMBURSEMENT_NEEDED"
                        } else {
                            "SOCIAL_PREMIUM_USE_NEEDED"
                        },
                        answer,
                        Vec::new(),
                    );
                }
            }
            let community = document_community_answers(draft, &item.document);
            if community.len() > 1 || !community.contains(&crate::community::index(item.community))
            {
                add(
                    "community",
                    "ISSUE_COMMUNITY_ANSWER_NEEDED",
                    A::Community { value: item.community },
                    community,
                );
            }
            match item.document {
                IncomeSource::CanceledDebt {
                    value: crate::CanceledDebtDocument::Cancel1099C(_),
                } => {
                    let FederalIncome::PersonalDebt { data } = &entry.income else {
                        unreachable!("a Form 1099-C row reopens a canceled-debt source")
                    };
                    add(
                        "debt_treatment",
                        "DEBT_TREATMENT_NEEDED",
                        A::DebtTreatment { value: data.treatment.clone() },
                        Vec::new(),
                    );
                    // A retained property answer stays on the topic after box 7 is
                    // cleared, so its save asks for box 7 again.
                    if data.form.property_value.is_some() || data.property.is_some() {
                        add(
                            "debt_property",
                            "DEBT_PROPERTY_NEEDED",
                            A::DebtProperty { value: data.property.clone() },
                            Vec::new(),
                        );
                    }
                }
                IncomeSource::CanceledDebt {
                    value: crate::CanceledDebtDocument::Acquisition1099A(_),
                } => {
                    let FederalIncome::PropertyAcquisition { data } = &entry.income else {
                        unreachable!("a Form 1099-A row reopens an acquisition source")
                    };
                    add(
                        "debt_acquisition",
                        "DEBT_ACQUISITION_PROPERTY_NEEDED",
                        A::DebtAcquisition { value: data.property.as_ref().map(Into::into) },
                        Vec::new(),
                    );
                }
                IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Ltc1099(_) } => {
                    let FederalIncome::LtcBenefits { data: w } = &entry.income else {
                        unreachable!("a Form 1099-LTC row reopens a long-term care source")
                    };
                    if w.form.benefits.unwrap_or(0) > 0 || w.benefit_origin.is_some() {
                        add(
                            "ltc_benefit_origin",
                            "DOCUMENT_LTC_BENEFIT_ORIGIN",
                            A::LtcBenefitOrigin { value: w.benefit_origin.clone() },
                            Vec::new(),
                        );
                    }
                    if w.form.accelerated.unwrap_or(0) > 0 || w.accelerated_origin.is_some() {
                        add(
                            "ltc_accelerated_origin",
                            "DOCUMENT_LTC_ACCELERATED_ORIGIN",
                            A::LtcAcceleratedOrigin { value: w.accelerated_origin.clone() },
                            Vec::new(),
                        );
                    }
                    // Not a box of Form 1099-LTC: the payer's statement reports it, so it
                    // is asked beside the paper and kept on its wrapper.
                    add(
                        "ltc_interest",
                        "DOCUMENT_LTC_INTEREST",
                        A::LtcInterest { value: w.interest.unwrap_or(0) },
                        Vec::new(),
                    );
                    if w.form.benefits.unwrap_or(0) > 0 && !w.form.box4_qualified {
                        add(
                            "ltc_contract",
                            "DOCUMENT_LTC_CONTRACT",
                            A::LtcContract { value: (&w.contract).into() },
                            Vec::new(),
                        );
                    }
                    if (w.form.paid != crate::LtcPaidBasis::Reimbursed
                        && ((w.form.benefits.unwrap_or(0) > 0
                            && (w.form.box4_qualified
                                || !matches!(
                                    w.contract,
                                    crate::LtcContract::Nonqualified { .. }
                                )))
                            || (w.form.accelerated.unwrap_or(0) > 0
                                && w.form.illness != Some(crate::LtcIllness::Terminal))))
                        || w.insured.is_some()
                    {
                        add(
                            "ltc_insured",
                            "DOCUMENT_LTC_INSURED",
                            A::LtcInsured { value: w.insured },
                            Vec::new(),
                        );
                    }
                }
                IncomeSource::Retirement { value: crate::RetirementDocument::Rrb1099R(_) } => {
                    let FederalIncome::Retirement { data: crate::RetirementSource::Rrb1099R(w) } =
                        &entry.income
                    else {
                        continue;
                    };
                    add(
                        "railroad_recipient",
                        "DOCUMENT_RAILROAD_RECIPIENT",
                        A::RailroadRecipient { value: w.recipient_kind.clone() },
                        Vec::new(),
                    );
                    if w.box_8.unwrap_or(0) != 0 || !w.repayments.is_empty() {
                        add(
                            "railroad_repayments",
                            "DOCUMENT_RAILROAD_REPAYMENTS",
                            A::RailroadRepayments { value: w.repayments.clone() },
                            Vec::new(),
                        );
                    }
                }
                IncomeSource::Wage { value: crate::WageDocument::W2(_) } => {
                    let FederalIncome::Wage { data } = &entry.income else { unreachable!() };
                    let mut answers = crate::W2AnswersView::from(data.answers().unwrap());
                    let crate::WageSource::W2(source) = data else { unreachable!() };
                    answers.calls = w2_calls(draft, &(&source.native()).into());
                    answers.box14 = source.box14_asks();
                    add("wage", "DOCUMENT_WAGE_ANSWERS", A::Wage { answers }, Vec::new());
                }
                IncomeSource::Retirement { value: crate::RetirementDocument::R1099(_) } => {
                    let Some(source) = entry.income.r1099_source() else {
                        continue;
                    };
                    let r = retirement_answered(draft, entry.id, entry.owner.owned(), source);
                    if r.trump && r.box7 == crate::DistributionCodes::CODE_8.0 {
                        continue;
                    }
                    add(
                        "treatment",
                        "DOCUMENT_RETIREMENT_TREATMENT",
                        A::RetirementTreatment { code: r.code.clone().map(Into::into) },
                        crate::DistributionCodes::treatments(r.box7, r.ira, r.trump),
                    );
                    let methods = crate::DistributionCodes::methods(&r);
                    if methods != [0] {
                        add(
                            "method",
                            "DOCUMENT_RETIREMENT_METHOD",
                            A::RetirementMethod { method: r.taxable_method.clone() },
                            methods,
                        );
                    }
                    let calls =
                        crate::RCalls::for_boxes(r.box7, r.ira, r.trump, r.box_10.unwrap_or(0));
                    if !calls.box_10_basis.is_empty() || r.box_10_basis.is_some() {
                        add(
                            "basis",
                            "ISSUE_BOX_10_BASIS_UNKNOWN",
                            A::RetirementBasis { basis: r.box_10_basis },
                            Vec::new(),
                        );
                    }
                    if !calls.disability.is_empty() || r.disability.is_some() {
                        add(
                            "disability",
                            "DOCUMENT_RETIREMENT_DISABILITY",
                            A::RetirementDisability { disability: r.disability.clone() },
                            Vec::new(),
                        );
                    }
                    if !calls.disaster.is_empty() || r.disaster.is_some() {
                        add(
                            "disaster",
                            "DOCUMENT_ANSWER_RETIREMENTDISASTER",
                            A::RetirementDisaster { value: r.disaster.clone() },
                            Vec::new(),
                        );
                    }
                    let public_safety = crate::PublicSafetyAnswer::of(r.public_safety().to_vec());
                    if !calls.public_safety.is_empty() || public_safety.is_some() {
                        add(
                            "public_safety",
                            "DOCUMENT_ANSWER_RETIREMENTPUBLICSAFETY",
                            A::RetirementPublicSafety { value: public_safety },
                            Vec::new(),
                        );
                    }
                    add(
                        "after_death",
                        "DOCUMENT_ANSWER_RETIREMENTAFTERDEATH",
                        A::RetirementAfterDeath { value: r.after_death.clone() },
                        Vec::new(),
                    );
                }
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Form8818(_),
                } => {
                    let FederalIncome::BondRedemption { data } = &entry.income else {
                        unreachable!()
                    };
                    if !savings_bond_exclusions(draft, entry.owner).is_empty()
                        || data.form_8815.is_some()
                    {
                        add(
                            "form_8815",
                            "DOCUMENT_BOND_REDEMPTION_FORM_8815",
                            A::BondRedemptionForm8815 {
                                value: data
                                    .form_8815
                                    .map(|form_8815| crate::BondRedemptionLink { form_8815 }),
                            },
                            Vec::new(),
                        );
                    }
                }
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::SavingsBonds(_),
                } => {
                    let FederalIncome::SavingsBonds { data } = &entry.income else {
                        unreachable!()
                    };
                    add(
                        "savings_bonds_records",
                        "DOCUMENT_SAVINGS_BONDS_RECORDS",
                        A::SavingsBondsRecords { value: data.answers.as_ref().map(Into::into) },
                        Vec::new(),
                    );
                }
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(_),
                } => {
                    let w = match &entry.income {
                        FederalIncome::Interest { data: crate::InterestSource::Int1099(data) } => {
                            data.clone()
                        }
                        FederalIncome::Interest { data: crate::InterestSource::Record(data) } => {
                            interest_page(data, &Default::default())
                        }
                        _ => unreachable!("a Form 1099-INT row reopens an interest source"),
                    };
                    let calls = int_calls(investment_final_return(draft), &(&w).into());
                    if !calls.nominee.is_empty() || w.nominee.is_some() {
                        add(
                            "nominee",
                            "DOCUMENT_ANSWER_INTERESTNOMINEE",
                            A::InterestNominee { value: w.nominee },
                            calls.nominee,
                        );
                    }
                    if !calls.after_death.is_empty() || w.after_death.is_some() {
                        add(
                            "after_death",
                            "DOCUMENT_ANSWER_INTERESTAFTERDEATH",
                            A::InterestAfterDeath { value: w.after_death },
                            calls.after_death,
                        );
                    }
                    add(
                        "adjustments",
                        "DOCUMENT_ANSWER_INTERESTADJUSTMENT",
                        A::InterestAdjustment { value: w.adjustments },
                        Vec::new(),
                    );
                    add(
                        "accrued_interest",
                        "DOCUMENT_ANSWER_INTERESTACCRUED",
                        A::InterestAccrued { value: w.accrued_interest },
                        Vec::new(),
                    );
                    add(
                        "special",
                        "DOCUMENT_ANSWER_INTERESTTREATMENT",
                        A::InterestTreatment { value: w.special },
                        Vec::new(),
                    );
                }
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(_),
                } => {
                    let FederalIncome::Oid { data: w } = &entry.income else {
                        unreachable!("a Form 1099-OID row reopens an OID source")
                    };
                    let w = w.clone();
                    let calls = oid_calls(&(&w).into());
                    if !calls.nominee.is_empty() || w.nominee.is_some() {
                        add(
                            "nominee",
                            "DOCUMENT_ANSWER_OIDNOMINEE",
                            A::OidNominee { value: w.nominee },
                            calls.nominee,
                        );
                    }
                    add(
                        "adjustments",
                        "DOCUMENT_ANSWER_INTERESTADJUSTMENT",
                        A::InterestAdjustment { value: w.adjustments },
                        Vec::new(),
                    );
                    if w.form.tax_exempt_oid > 0
                        || w.tax_exempt_bond_premium > 0
                        || w.private_activity > 0
                    {
                        add(
                            "premium",
                            "DOCUMENT_ANSWER_OIDPREMIUM",
                            A::OidPremium { value: w.tax_exempt_bond_premium },
                            Vec::new(),
                        );
                        add(
                            "private_activity",
                            "DOCUMENT_ANSWER_OIDPRIVATEACTIVITY",
                            A::OidPrivateActivity { value: w.private_activity },
                            Vec::new(),
                        );
                    }
                    // The treatment is the person's topic answer, never the paper's:
                    // always shown, answered once it is saved.
                    add(
                        "special",
                        "DOCUMENT_ANSWER_OIDTREATMENT",
                        A::OidTreatment { value: w.special },
                        Vec::new(),
                    );
                }
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(w),
                } => {
                    let FederalIncome::Dividend { data } = &entry.income else {
                        unreachable!("a Form 1099-DIV row reopens a dividend source")
                    };
                    let calls = div_calls(investment_final_return(draft), &(&w).into());
                    if w.nondividend_distributions.unwrap_or(0) > 0
                        || w.cash_liquidation.unwrap_or(0) > 0
                        || w.noncash_liquidation.unwrap_or(0) > 0
                        || !data.capital_returns.is_empty()
                    {
                        add(
                            "capital_returns",
                            "DOCUMENT_ANSWER_DIVIDENDCAPITALRETURNS",
                            A::DividendCapitalReturns { value: data.capital_returns.clone() },
                            Vec::new(),
                        );
                    }
                    if !calls.nominee.is_empty() || data.nominee.is_some() {
                        add(
                            "nominee",
                            "DOCUMENT_ANSWER_DIVIDENDNOMINEE",
                            A::DividendNominee { value: data.nominee.clone() },
                            calls.nominee,
                        );
                    }
                    if !calls.after_death.is_empty() || data.after_death.is_some() {
                        add(
                            "after_death",
                            "DOCUMENT_ANSWER_DIVIDENDAFTERDEATH",
                            A::DividendAfterDeath { value: data.after_death.clone() },
                            calls.after_death,
                        );
                    }
                    if !calls.reit_eligible.is_empty() {
                        add(
                            "reit",
                            "DOCUMENT_ANSWER_DIVIDENDELIGIBILITY",
                            A::DividendEligibility { value: data.reit_eligible },
                            calls.reit_eligible,
                        );
                    }
                    if !calls.section_1202_exclusion.is_empty() {
                        add(
                            "exclusion",
                            "DOCUMENT_ANSWER_DIVIDENDFUNDEXCLUSION",
                            A::DividendFundExclusion { value: data.section_1202_exclusion },
                            calls.section_1202_exclusion,
                        );
                    }
                    add(
                        "special",
                        "DOCUMENT_ANSWER_DIVIDENDTREATMENT",
                        A::DividendTreatment { value: data.special },
                        Vec::new(),
                    );
                }
                IncomeSource::InvestmentSale {
                    value:
                        crate::InvestmentSaleDocument::Form1099B(_)
                        | crate::InvestmentSaleDocument::Form1099DA(_),
                } => {
                    let FederalIncome::Broker { data: source } = &entry.income else {
                        unreachable!("a broker paper reopens its accepted source")
                    };
                    if source.sale_needs_answers() {
                        add(
                            "broker_sale",
                            "DOCUMENT_ANSWER_BROKERSALE",
                            A::BrokerSaleRecords { value: source.sale_answers() },
                            Vec::new(),
                        );
                    }
                    if source.contracts_needs_answers() {
                        add(
                            "broker_contracts",
                            "DOCUMENT_ANSWER_BROKERCONTRACTS",
                            A::BrokerContractsTreatment {
                                value: source.personal.contracts.special_treatment,
                            },
                            Vec::new(),
                        );
                    }
                }
                IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Sale1099B(_),
                } => {
                    let FederalIncome::Sale { data } = &entry.income else {
                        unreachable!("a 1099-B row reopens a sale source")
                    };
                    add(
                        "adjustment",
                        "DOCUMENT_ANSWER_SALEADJUSTMENT",
                        A::SaleAdjustment { value: data.adjustment.clone() },
                        Vec::new(),
                    );
                    add(
                        "special",
                        "DOCUMENT_ANSWER_SALETREATMENT",
                        A::SaleTreatment { value: data.special },
                        Vec::new(),
                    );
                }
                IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::OpportunityFund(_),
                } => {
                    let FederalIncome::OpportunityFund { data } = &entry.income else {
                        unreachable!("a Form 8997 row reopens an opportunity fund source")
                    };
                    if !data.gain_code.unsupported() {
                        add(
                            "special",
                            "DOCUMENT_ANSWER_OPPORTUNITYFUNDSPECIAL",
                            A::OpportunityFundSpecial { value: data.special },
                            Vec::new(),
                        );
                    }
                    if !data.left_out() {
                        add(
                            "event",
                            "DOCUMENT_ANSWER_OPPORTUNITYFUNDEVENT",
                            A::OpportunityFundEvent { value: data.event.clone() },
                            Vec::new(),
                        );
                    }
                }
                IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Contracts1099B(_),
                } => {
                    let FederalIncome::Section1256 { data } = &entry.income else {
                        unreachable!("a 1099-B contracts row reopens a contracts source")
                    };
                    add(
                        "special",
                        "DOCUMENT_ANSWER_CONTRACTSSPECIAL",
                        A::ContractsSpecial { value: data.special },
                        Vec::new(),
                    );
                }
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Form2439(w),
                } => {
                    let FederalIncome::UndistributedGain { data } = &entry.income else {
                        unreachable!("a Form 2439 row reopens an undistributed gain source")
                    };
                    if crate::Form2439::from(&w).small_business > 0
                        || data.section_1202_exclusion.is_some()
                    {
                        add(
                            "exclusion",
                            "DOCUMENT_ANSWER_DIVIDENDFUNDEXCLUSION",
                            A::DividendFundExclusion { value: data.section_1202_exclusion },
                            alloc::vec![0, 1, 2, 3],
                        );
                    }
                    add(
                        "special",
                        "DOCUMENT_ANSWER_UNDISTRIBUTEDGAINSPECIAL",
                        A::UndistributedGainSpecial { value: data.special },
                        Vec::new(),
                    );
                }
                IncomeSource::GovernmentPayment { .. } => {
                    let FederalIncome::GovernmentPayment { data } = &entry.income else {
                        unreachable!("a Form 1099-G row reopens its government payment source")
                    };
                    let boxes = crate::FederalUnemployment::from(data);
                    if boxes.taxable_grants > 0 {
                        add(
                            "grant",
                            "DOCUMENT_ANSWER_GOVERNMENTGRANTUSE",
                            A::GovernmentGrantUse { value: data.grant_use.clone() },
                            Vec::new(),
                        );
                    }
                    if boxes.family_leave > 0 {
                        add(
                            "medical",
                            "DOCUMENT_ANSWER_GOVERNMENTMEDICALLEAVE",
                            A::GovernmentMedicalLeave { value: data.family_leave_medical },
                            Vec::new(),
                        );
                    }
                    if boxes.amount > 0 || !data.repayments.is_empty() {
                        add(
                            "repayments",
                            "DOCUMENT_ANSWER_GOVERNMENTREPAYMENTS",
                            A::GovernmentRepayments { value: data.repayments.clone() },
                            Vec::new(),
                        );
                    }
                    if data.form.has_payments() || !data.repayments.is_empty() {
                        add(
                            "special",
                            "DOCUMENT_ANSWER_GOVERNMENTSPECIAL",
                            A::GovernmentSpecial { value: data.special },
                            Vec::new(),
                        );
                    }
                    if data.form.state_or_local_income_tax_refunds != 0 {
                        add(
                            "deduction",
                            "DOCUMENT_ANSWER_REFUNDDEDUCTION",
                            A::RefundDeduction { value: refund_topic_deduction(data) },
                            Vec::new(),
                        );
                    }
                }
                IncomeSource::Gambling(_) => {
                    let FederalIncome::Gambling { data: crate::GamblingSource::W2g(data) } =
                        &entry.income
                    else {
                        unreachable!("a Form W-2G row reopens a W-2G source")
                    };
                    add(
                        "special",
                        "DOCUMENT_ANSWER_GAMBLINGSPECIAL",
                        A::GamblingSpecial { value: data.special },
                        Vec::new(),
                    );
                }
                IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Form1099S(_),
                } => {
                    let FederalIncome::HomeSale { data } = &entry.income else {
                        unreachable!("a Form 1099-S row reopens a home sale source")
                    };
                    add(
                        "home",
                        "DOCUMENT_ANSWER_HOMESALE",
                        A::HomeSale { value: data.sale_use() },
                        Vec::new(),
                    );
                }
                IncomeSource::SelfEmployment {
                    value: crate::SelfEmploymentDocument::Form1099Nec(_),
                } => {
                    let FederalIncome::Nec1099 { data } = &entry.income else {
                        unreachable!("a Form 1099-NEC row reopens its accepted source")
                    };
                    let boxes = crate::FederalForm1099Nec::from(data);
                    if boxes.compensation > 0 || data.belongs.is_some() {
                        add(
                            "use",
                            "DOCUMENT_ANSWER_NONEMPLOYEEUSE",
                            A::NonemployeeUse { value: data.belongs.clone() },
                            crate::IncomeUse::nonemployee_routes(),
                        );
                    }
                }
                IncomeSource::SelfEmployment {
                    value: crate::SelfEmploymentDocument::Form1099K(_),
                } => {
                    let FederalIncome::K1099 { data: w } = &entry.income else { unreachable!() };
                    add(
                        "parts",
                        "DOCUMENT_ANSWER_PAYMENTPARTS",
                        A::PaymentParts(crate::PaymentPartsAnswer { value: w.parts.clone() }),
                        Vec::new(),
                    );
                }
                IncomeSource::OtherIncome {
                    value: crate::OtherIncomeDocument::Form1099Misc(_),
                } => {
                    let FederalIncome::Misc1099 { data: w } = &entry.income else { unreachable!() };
                    let w = crate::Form1099Misc::from(w);
                    if w.other > 0 || w.other_use.is_some() {
                        add(
                            "use",
                            "DOCUMENT_ANSWER_MISCELLANEOUSUSE",
                            A::MiscellaneousUse { value: w.other_use },
                            crate::IncomeUse::miscellaneous_routes(),
                        );
                    }
                    if w.attorney > 0 || w.client_funds > 0 {
                        add(
                            "client_funds",
                            "DOCUMENT_ANSWER_ATTORNEYCLIENTFUNDS",
                            A::AttorneyClientFunds { value: w.client_funds },
                            Vec::new(),
                        );
                    }
                }
                IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Qa1099(_) } => {
                    let FederalIncome::Able { data } = &entry.income else {
                        unreachable!("a Form 1099-QA row reopens an ABLE source")
                    };
                    if !crate::AbleDistribution::from(data).transfer_only() {
                        add(
                            "treatment",
                            "DOCUMENT_ANSWER_ABLETREATMENT",
                            A::AbleTreatment { value: data.treatment.clone() },
                            Vec::new(),
                        );
                    }
                }
                IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Q1099(_) } => {
                    let FederalIncome::EducationDistribution { data: w } = &entry.income else {
                        unreachable!("a Form 1099-Q row reopens an education distribution source")
                    };
                    if w.form.recipient_not_beneficiary
                        || !matches!(
                            w.beneficiary,
                            None | Some(crate::FederalEducationBeneficiary::Recipient)
                        )
                    {
                        let choices = if w.form.recipient_not_beneficiary {
                            alloc::vec![
                                libertas_macros::variant_index!(crate::Q1099Beneficiary::Student)
                                    as i32,
                                libertas_macros::variant_index!(
                                    crate::Q1099Beneficiary::OtherStudent
                                ) as i32
                            ]
                        } else {
                            alloc::vec![libertas_macros::variant_index!(
                                crate::Q1099Beneficiary::Recipient
                            ) as i32]
                        };
                        add(
                            "beneficiary",
                            "DOCUMENT_ANSWER_EDUCATIONBENEFICIARY",
                            A::EducationBeneficiary {
                                value: w.beneficiary.as_ref().map(Into::into),
                            },
                            choices,
                        );
                    }
                    let form = crate::Q1099::from(&w.form);
                    if matches!(
                        form.account,
                        crate::EducationAccount::Coverdell {
                            amounts: crate::CoverdellAmounts::ValueOnly { .. }
                        }
                    ) {
                        add(
                            "coverdell",
                            "DOCUMENT_ANSWER_COVERDELLRECORDS",
                            A::CoverdellRecords {
                                opening: w.coverdell_basis_start,
                                contributions: w.coverdell_contributions,
                                late_rollovers: w.late_rollovers,
                            },
                            Vec::new(),
                        );
                    }
                    match form.transfer {
                        crate::Q1099Transfer::Withdrawal { .. } => add(
                            "redeposited",
                            "DOCUMENT_ANSWER_EDUCATIONREDEPOSITED",
                            A::EducationRedeposited { value: w.redeposited },
                            Vec::new(),
                        ),
                        crate::Q1099Transfer::TrusteeTransfer => add(
                            "taxable_transfer",
                            "DOCUMENT_ANSWER_EDUCATIONTAXABLETRANSFER",
                            A::EducationTaxableTransfer { value: w.taxable_transfer_part },
                            Vec::new(),
                        ),
                        crate::Q1099Transfer::RothIra => add(
                            "roth",
                            "DOCUMENT_ANSWER_EDUCATIONROTHRECORDS",
                            A::EducationRothRecords { value: w.roth_rollover.clone() },
                            Vec::new(),
                        ),
                        _ => {}
                    }
                }
                IncomeSource::OtherIncome {
                    value:
                        crate::OtherIncomeDocument::Sa1099(_)
                        | crate::OtherIncomeDocument::HsaTrusteeStatement(_),
                } => {
                    use crate::MedicalSaPayment as M;
                    use libertas_macros::variant_index;
                    let FederalIncome::MedicalDistribution { data: w } = &entry.income else {
                        unreachable!("a Form 1099-SA row reopens a medical distribution source")
                    };
                    let printed = w.internal();
                    let excess_from_print = printed.origin
                        == crate::HealthDocumentOrigin::Form1099Sa
                        && printed.box3_code == Some(2);
                    if !excess_from_print || !matches!(w.payment, None | Some(M::ExcessReturned)) {
                        let choices = match (printed.origin, printed.box3_code) {
                            (crate::HealthDocumentOrigin::TrusteeStatement, _) => {
                                alloc::vec![variant_index!(M::Unreported) as i32]
                            }
                            (_, Some(1 | 3)) => alloc::vec![variant_index!(M::Own) as i32],
                            (_, Some(2)) => alloc::vec![variant_index!(M::ExcessReturned) as i32],
                            (_, Some(4 | 6)) => alloc::vec![variant_index!(M::Inherited) as i32],
                            (_, Some(5)) => alloc::vec![variant_index!(M::Prohibited) as i32],
                            _ => Vec::new(),
                        };
                        add(
                            "payment",
                            "DOCUMENT_ANSWER_MEDICALPAYMENT",
                            A::MedicalPayment { value: w.payment.clone() },
                            choices,
                        );
                    }
                    if matches!(w.account(), crate::MedicalSaAccount::MedicareAdvantage) {
                        add(
                            "medicare_spousal",
                            "DOCUMENT_ANSWER_MEDICARESPOUSAL",
                            A::MedicareSpousal { value: w.medicare_from_spouse },
                            Vec::new(),
                        );
                        if w.medicare_from_spouse != Some(true)
                            && matches!(printed.box3_code, Some(1 | 3 | 5))
                        {
                            add(
                                "medicare_prior",
                                "DOCUMENT_ANSWER_MEDICAREPRIOR",
                                A::MedicarePrior { value: w.medicare_prior.clone() },
                                Vec::new(),
                            );
                        }
                    }
                }
                IncomeSource::Farming(_) => {
                    let FederalIncome::Cooperative { data } = &entry.income else { unreachable!() };
                    let w = crate::Patr1099::from(data);
                    add(
                        "allocations",
                        "DOCUMENT_ANSWER_COOPERATIVEALLOCATIONS",
                        A::CooperativeAllocations { value: w.more.allocations.clone() },
                        Vec::new(),
                    );
                    for (field, question, answer, allowed) in cooperative_followups::questions(&w) {
                        add(&field, question, answer, allowed);
                    }
                }
                IncomeSource::K1 { value } => {
                    let questions = match &entry.income {
                        FederalIncome::Scorp { data } => k1_followups::scorp_questions(data),
                        FederalIncome::EstateTrustPortfolio { data } => {
                            k1_followups::estate_questions(data)
                        }
                        _ => k1_followups::questions(draft, &value),
                    };
                    for (field, question, answer, allowed) in questions {
                        add(&field, question, answer, allowed);
                    }
                }
                _ => {}
            }
        }
        // A Form 3921 is a screening source, not an income entry: its answers
        // are asked with the Investment sales topic.
        if topic.is_none_or(|topic| topic == TaxSection::InvestmentSales) {
            for source in draft.screening.iter().flat_map(|screening| &screening.iso_exercises) {
                let answer = A::IsoExerciseFacts { value: source.more.clone() };
                rows.push((
                    source.id,
                    DocumentQuestion {
                        key: alloc::format!("income/{}/iso", source.id),
                        prompt: followup_prompt(
                            "DOCUMENT_ANSWER_ISOEXERCISEFACTS",
                            super::document_owners::document_name(draft, source.id)
                                .unwrap_or_else(|| status("SAVING_KIND_FORM_3921_ONE")),
                        ),
                        allowed_answers: alloc::vec![answer_kind(&answer)],
                        allowed_values: Vec::new(),
                        k1: Default::default(),
                        espp_acquisitions: Vec::new(),
                        savings_bond_exclusions: Vec::new(),
                        health_plans: Vec::new(),
                        answer,
                    },
                ));
            }
        }
        rows.sort_by(|(_, left), (_, right)| {
            match (followup_filer_weight(&left.answer), followup_filer_weight(&right.answer)) {
                (Some(left), Some(right)) => right.cmp(&left),
                (Some(_), None) => core::cmp::Ordering::Less,
                (None, Some(_)) => core::cmp::Ordering::Greater,
                (None, None) => core::cmp::Ordering::Equal,
            }
        });
        rows
    }

    pub(super) fn followup_candidate(
        &self,
        topic: TaxSection,
        supplied: Vec<DocumentQuestion>,
        records: &[crate::IncomeRecordItem],
    ) -> Result<Self, SaveFailure> {
        self.followup_candidate_with_mode(topic, supplied, true, records)
    }
    pub(super) fn followup_candidate_with_mode(
        &self,
        topic: TaxSection,
        supplied: Vec<DocumentQuestion>,
        require_answers: bool,
        records: &[crate::IncomeRecordItem],
    ) -> Result<Self, SaveFailure> {
        let keys: Vec<_> = supplied.iter().map(|row| row.key.clone()).collect();
        self.followup_candidate_indexed(topic, supplied, require_answers, records)
            .map_err(|failure| keyed_followup_failure(failure, &keys))
    }

    // Source owners bind their diagnostics before the request's keys are applied.
    pub(super) fn followup_candidate_indexed(
        &self,
        topic: TaxSection,
        supplied: Vec<DocumentQuestion>,
        require_answers: bool,
        records: &[crate::IncomeRecordItem],
    ) -> Result<Self, SaveFailure> {
        let expected =
            Self::document_followup_rows(&self.draft, Some(topic), Some(&self.income_result()));
        let failure = |index, field| {
            ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                .at_field(field)
                .in_list("followups", index)
        };
        if supplied.len() != expected.len() {
            return Err(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                .at_field("followups")
                .into());
        }
        let mut seen = alloc::collections::BTreeSet::new();
        let mut draft = self.draft.clone();
        if topic == TaxSection::Retirement {
            let forms: Vec<_> = records
                .iter()
                .filter(|record| matches!(record.document, crate::IncomeRecord::Form8606(_)))
                .cloned()
                .map(crate::IncomeRecordItem::into_income)
                .collect();
            let repayments = draft
                .adjustments
                .iter()
                .flat_map(|a| &a.ira_basis)
                .flat_map(|basis| {
                    basis.special_repayments.iter().map(move |record| (basis.owner, record.clone()))
                })
                .collect();
            let basis = ira_basis_records(&forms, draft.adjustments.as_ref(), repayments);
            if let Some(adjustments) = draft.adjustments.as_mut() {
                adjustments.ira_basis = basis;
            }
        }
        let mut items: Vec<(i64, IncomeDocumentItem)> = Vec::new();
        // A 1099-R's answers belong to its accepted source, not to its paper.
        let mut retirement: Vec<(i64, crate::R1099Answered)> = Vec::new();
        let mut railroad: Vec<(i64, crate::RailroadSource)> = Vec::new();
        for (id, row) in &expected {
            if matches!(row.answer, A::Management { .. }) {
                continue;
            }
            if items.iter().any(|(found, _)| found == id) {
                continue;
            }
            // A Form 3921 has no income entry; its answers are applied below.
            let Some(entry) = draft.income.iter().find(|entry| entry.id == *id) else {
                continue;
            };
            let item = document_item(entry, &[], crate::federal_rules::separate_return(&draft))
                .ok_or_else(|| {
                    SaveFailure::from(
                        ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("followups"),
                    )
                })?;
            if let Some(source) = entry.income.r1099_source() {
                retirement.push((
                    *id,
                    retirement_answered(&self.draft, *id, entry.owner.owned(), source),
                ));
            }
            if let FederalIncome::Retirement { data: crate::RetirementSource::Rrb1099R(source) } =
                &entry.income
            {
                railroad.push((*id, source.clone()));
            }
            items.push((*id, item));
        }
        let mut submitted = Vec::new();
        let mut debt_answers = Vec::new();
        let mut refund_answers = Vec::new();
        let mut rental_section179_answers = Vec::new();
        let mut rental_home_answers = Vec::new();
        let mut wage_answers = Vec::new();
        let mut scorp_answers = Vec::new();
        let mut estate_answers = Vec::new();
        let mut social_premium_answers = Vec::new();
        let mut espp_answers = Vec::new();
        let mut plan_sale_answers = Vec::new();
        let mut government_answers = Vec::new();
        let mut gambling_answers = Vec::new();
        let mut nonemployee_answers = Vec::new();
        let mut misc_answers = Vec::new();
        let mut medical_answers = Vec::new();
        let mut able_answers = Vec::new();
        let mut fund_answers = Vec::new();
        let mut education_answers = Vec::new();
        let mut ltc_answers = Vec::new();
        let mut cooperative_answers = Vec::new();
        let mut sale_answers = Vec::new();
        let mut opportunity_fund_answers = Vec::new();
        let mut dividend_answers = Vec::new();
        let mut child_dividend_answers = Vec::new();
        let mut interest_answers = Vec::new();
        let mut oid_answers = Vec::new();
        let mut iso_answers = Vec::new();
        let mut home_answers = Vec::new();
        let mut qsbs_answers = Vec::new();
        let mut broker_sources = alloc::collections::BTreeMap::new();
        let mut savings_bond_answers = Vec::new();
        let mut redemption_answers = Vec::new();
        for (index, row) in supplied.into_iter().enumerate() {
            let (id, accepted) = expected
                .iter()
                .find(|(_, accepted)| accepted.key == row.key)
                .ok_or_else(|| SaveFailure::from(failure(index, "key")))?;
            submitted.push((*id, row.key.clone(), index));
            if !seen.insert(row.key) || answer_kind(&row.answer) != answer_kind(&accepted.answer) {
                return Err(failure(index, "answer").into());
            }
            if accepted.key.split('/').nth(2) == Some("statement") {
                let statement_id = accepted
                    .key
                    .split('/')
                    .nth(3)
                    .and_then(|value| value.parse::<i64>().ok())
                    .ok_or_else(|| SaveFailure::from(failure(index, "key")))?;
                child_dividend_answers.push((*id, statement_id, index, row.answer));
                continue;
            }
            if matches!(
                row.answer,
                A::QsbsBoolean { .. }
                    | A::QsbsPriorGain { .. }
                    | A::QsbsZone { .. }
                    | A::QsbsZoneGain { .. }
            ) {
                let field = String::from(accepted.key.splitn(3, '/').nth(2).unwrap());
                qsbs_answers.push((*id, index, field, row.answer));
                continue;
            }
            if let A::IsoExerciseFacts { value } = &row.answer {
                iso_answers.push((*id, index, value.clone()));
                continue;
            }
            let invalid = || {
                ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                    .at_field("value")
                    .in_field("answer")
                    .in_list("followups", index)
            };
            if let A::Management { value } = &row.answer {
                if require_answers && value.is_none() {
                    return Err(ValidationIssue::new(
                        "INJURED_SPOUSE_WITHHOLDING_MANAGEMENT_NEEDED",
                    )
                    .at_field("value")
                    .in_field("answer")
                    .in_list("followups", index)
                    .into());
                }
                if value.is_some_and(|value| {
                    !accepted
                        .allowed_values
                        .contains(&crate::injured_spouse_payments::management_index(value))
                }) {
                    return Err(invalid().into());
                }
                // Management is independent of this paper's other topic answers.
                draft.income.iter_mut().find(|entry| entry.id == *id).unwrap().refund_management =
                    *value;
                continue;
            }
            let item = &mut items.iter_mut().find(|(found, _)| found == id).unwrap().1;
            macro_rules! railroad_edit {
                (|$answered:ident| $change:expr) => {
                    if let Some((_, $answered)) = railroad.iter_mut().find(|(found, _)| found == id)
                    {
                        $change;
                    } else {
                        return Err(failure(index, "answer").into());
                    }
                };
            }
            macro_rules! retirement_edit {
                (|$answered:ident| $change:expr) => {
                    if let Some((_, $answered)) =
                        retirement.iter_mut().find(|(found, _)| found == id)
                    {
                        $change;
                    } else {
                        return Err(failure(index, "answer").into());
                    }
                };
            }
            match row.answer {
                A::BondRedemptionForm8815 { value } => redemption_answers.push((*id, index, value)),
                A::SavingsBondsRecords { value } => savings_bond_answers.push((*id, index, value)),
                value @ (A::EsppAcquisition { .. }
                | A::EsppFixedGrantPrice { .. }
                | A::EsppAdjustments { .. }
                | A::EsppWageInclusion { .. }) => espp_answers.push((*id, index, value)),
                value @ (A::IsoAdjustments { .. }
                | A::IsoWageInclusion { .. }
                | A::NuaAdjustments { .. }
                | A::NuaOwnership { .. }) => plan_sale_answers.push((*id, index, value)),
                A::SocialPremiums { value } => {
                    social_premium_answers.push((*id, value.map(Into::into), None))
                }
                A::SocialReimbursedPremiums { reimbursed } => social_premium_answers.push((
                    *id,
                    Some(crate::SocialPremiumUse::Medical),
                    reimbursed,
                )),
                A::SocialSharedPremiums { value, reimbursed } => {
                    social_premium_answers.push((*id, value.map(Into::into), reimbursed))
                }
                answer @ (A::DebtTreatment { .. }
                | A::DebtProperty { .. }
                | A::DebtAcquisition { .. }) => {
                    debt_answers.push((*id, answer));
                }
                A::LtcInsured { value } => {
                    if value.is_some_and(|id| {
                        !draft.income.iter().any(|entry| {
                            entry.id == id
                                && matches!(entry.income, FederalIncome::LtcInsured { .. })
                                && (self.joint() || entry.owner == item.owner)
                        })
                    }) {
                        return Err(invalid().into());
                    }
                    ltc_answers.push((*id, index, A::LtcInsured { value }));
                }
                answer @ (A::LtcInterest { .. }
                | A::LtcBenefitOrigin { .. }
                | A::LtcAcceleratedOrigin { .. }
                | A::LtcContract { .. }) => ltc_answers.push((*id, index, answer)),
                A::RailroadRecipient { value } => {
                    railroad_edit!(|w| w.recipient_kind = value)
                }
                A::RailroadRepayments { value } => {
                    railroad_edit!(|w| w.repayments = value)
                }
                A::Community { value } => {
                    if !accepted.allowed_values.contains(&crate::community::index(value)) {
                        return Err(ValidationIssue::new("COMMUNITY_ANSWER_NOT_ALLOWED")
                            .at_field("value")
                            .in_field("answer")
                            .in_list("followups", index)
                            .into());
                    }
                    item.community = value;
                }
                A::Management { .. } => unreachable!("management is applied independently"),
                A::Wage { mut answers } => {
                    if !matches!(
                        item.document,
                        IncomeSource::Wage { value: crate::WageDocument::W2(_) }
                    ) {
                        return Err(failure(index, "answer").into());
                    }
                    answers.calls = Default::default();
                    wage_answers.push((*id, index, answers));
                }
                answer @ (A::InterestNominee { .. }
                | A::InterestAfterDeath { .. }
                | A::InterestAccrued { .. }
                | A::InterestTreatment { .. }) => interest_answers.push((*id, index, answer)),
                answer @ (A::OidNominee { .. }
                | A::OidPremium { .. }
                | A::OidPrivateActivity { .. }
                | A::OidTreatment { .. }) => oid_answers.push((*id, index, answer)),
                answer @ A::DividendFundExclusion { .. }
                    if matches!(
                        item.document,
                        IncomeSource::InterestDividend {
                            value: crate::InterestDividendDocument::Form2439(_)
                        }
                    ) =>
                {
                    fund_answers.push((*id, index, answer))
                }
                answer @ (A::DividendNominee { .. }
                | A::DividendEligibility { .. }
                | A::DividendAfterDeath { .. }
                | A::DividendFundExclusion { .. }
                | A::DividendCapitalReturns { .. }
                | A::DividendTreatment { .. }) => dividend_answers.push((*id, index, answer)),
                answer @ (A::SaleAdjustment { .. }
                | A::SaleTreatment { .. }
                | A::ContractsSpecial { .. }) => sale_answers.push((*id, index, answer)),
                answer @ (A::OpportunityFundEvent { .. } | A::OpportunityFundSpecial { .. }) => {
                    opportunity_fund_answers.push((*id, index, answer));
                }
                answer @ A::UndistributedGainSpecial { .. } => {
                    fund_answers.push((*id, index, answer))
                }
                A::BrokerSaleRecords { value } => {
                    let source =
                        broker_source(&mut broker_sources, &self.draft, *id).ok_or_else(invalid)?;
                    if let Some(issue) = source.sale_answers_issue(&value)
                        && (require_answers
                            || !matches!(
                                issue.key,
                                "BROKER_SALE_USE_NEEDED"
                                    | "SALE_TERM_NEEDED"
                                    | "BROKER_SALE_BASIS_NEEDED"
                            ))
                    {
                        return Err(issue
                            .in_field("value")
                            .in_field("answer")
                            .in_list("followups", index)
                            .into());
                    }
                    source.apply_sale_answers(&value);
                }
                A::BrokerContractsTreatment { value } => {
                    if require_answers && value.is_none() {
                        return Err(ValidationIssue::new("BROKER_CONTRACTS_TREATMENT_NEEDED")
                            .at_field("value")
                            .in_field("answer")
                            .in_list("followups", index)
                            .into());
                    }
                    let source =
                        broker_source(&mut broker_sources, &self.draft, *id).ok_or_else(invalid)?;
                    source.personal.contracts.special_treatment = value;
                }
                answer @ (A::GovernmentGrantUse { .. }
                | A::GovernmentMedicalLeave { .. }
                | A::GovernmentRepayments { .. }
                | A::GovernmentSpecial { .. }) => government_answers.push((*id, index, answer)),
                A::GamblingSpecial { value } => gambling_answers.push((*id, index, value)),
                A::RefundDeduction { value } => refund_answers.push((*id, index, value)),
                A::RentalSection179Business { value } => {
                    rental_section179_answers.push((*id, index, value))
                }
                A::RentalHomeCarries { value } => rental_home_answers.push((*id, index, value)),
                A::HomeSale { value } => home_answers.push((*id, index, value)),
                A::RetirementDisaster { value } => {
                    retirement_edit!(|answered| answered.source_mut().disaster = value)
                }
                A::RetirementPublicSafety { value } => {
                    retirement_edit!(|answered| answered
                        .set_public_safety(crate::PublicSafetyAnswer::rows(&value).to_vec()))
                }
                A::RetirementAfterDeath { value } => {
                    retirement_edit!(|answered| answered.source_mut().after_death = value)
                }
                A::InterestAdjustment { value } => match &item.document {
                    IncomeSource::InterestDividend {
                        value: crate::InterestDividendDocument::Int1099(_),
                    } => interest_answers.push((*id, index, A::InterestAdjustment { value })),
                    IncomeSource::InterestDividend {
                        value: crate::InterestDividendDocument::Oid1099(_),
                    } => oid_answers.push((*id, index, A::InterestAdjustment { value })),
                    _ => return Err(failure(index, "answer").into()),
                },
                answer @ (A::CooperativeAllocations { .. }
                | A::CooperativeCreditRecords { .. }
                | A::CooperativeOtherRecords { .. }) => {
                    let field = String::from(accepted.key.splitn(3, '/').nth(2).unwrap());
                    cooperative_answers.push((*id, index, field, answer));
                }
                A::NonemployeeUse { value } => {
                    if value.as_ref().is_some_and(|route| !route.offered(&accepted.allowed_values))
                    {
                        return Err(invalid().into());
                    }
                    nonemployee_answers.push((*id, index, value));
                }
                answer @ (A::MiscellaneousUse { .. } | A::AttorneyClientFunds { .. }) => {
                    if let A::MiscellaneousUse { value: Some(route) } = &answer
                        && !route.offered(&accepted.allowed_values)
                    {
                        return Err(invalid().into());
                    }
                    misc_answers.push((*id, index, answer));
                }
                // These displayed source answers are service-written. Only the
                // source-only linked composer may accept changed payment parts.
                A::PaymentParts(..) => {}
                answer @ A::AbleTreatment { .. } => able_answers.push((*id, index, answer)),
                answer @ (A::EducationBeneficiary { .. }
                | A::CoverdellRecords { .. }
                | A::EducationRedeposited { .. }
                | A::EducationTaxableTransfer { .. }
                | A::EducationRothRecords { .. }) => education_answers.push((*id, index, answer)),
                answer @ (A::MedicalPayment { .. }
                | A::MedicareSpousal { .. }
                | A::MedicarePrior { .. }) => medical_answers.push((*id, index, answer)),
                answer if k1_followups::answer_kind(&answer).is_some() => {
                    if matches!(
                        &item.document,
                        IncomeSource::K1 { value: crate::K1Document::K1Scorp(_) }
                    ) {
                        scorp_answers.push((*id, accepted.key.clone(), index, answer));
                        continue;
                    }
                    if matches!(
                        &item.document,
                        IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(_) }
                    ) {
                        estate_answers.push((*id, accepted.key.clone(), index, answer));
                        continue;
                    }
                    k1_followups::apply(&mut item.document, &accepted.key, answer).map_err(
                        |issue| {
                            SaveFailure::from(issue.in_field("answer").in_list("followups", index))
                        },
                    )?;
                }
                answer => {
                    retirement_edit!(|answered| {
                        let r = answered.source_mut();
                        match answer {
                            A::RetirementTreatment { code } => r.code = code.map(Into::into),
                            A::RetirementMethod { method } => r.taxable_method = method,
                            A::RetirementBasis { basis } => r.box_10_basis = basis,
                            A::RetirementDisability { disability } => r.disability = disability,
                            _ => unreachable!(),
                        }
                    })
                }
            }
        }
        for (_, item) in &mut items {
            k1_followups::prune_rental_participation(&mut item.document);
        }
        for (id, index, more) in iso_answers {
            let answer_issue = |issue: ValidationIssue| {
                SaveFailure::from(
                    issue.in_field("value").in_field("answer").in_list("followups", index),
                )
            };
            let Some(source) = draft
                .screening
                .as_mut()
                .and_then(|s| s.iso_exercises.iter_mut().find(|row| row.id == id))
            else {
                return Err(failure(index, "answer").into());
            };
            let Some(more) = more else {
                source.more = None;
                continue;
            };
            if more.sold > source.form.shares {
                return Err(answer_issue(
                    ValidationIssue::new("ISO_SOLD_ABOVE_TRANSFERRED").at_field("sold"),
                ));
            }
            if let crate::IsoRecognition::Vesting { date, .. } = &more.recognition
                && *date < source.form.exercised
            {
                return Err(answer_issue(
                    ValidationIssue::new("ISO_VESTED_BEFORE_EXERCISED")
                        .at_field("date")
                        .in_field("recognition"),
                ));
            }
            source.more = Some(more);
            if !crate::IsoExercise::try_from(&*source).is_ok_and(|value| value.valid_timing()) {
                return Err(crate::interview::status(
                    "MESSAGE_FORM_3921_THESE_SHARES_ENTER_ALTERNATIVE_MINIMUM_TAXABLE_INCOME_FORM_6251_LI",
                )
                .into());
            }
            if !crate::extended_rules::valid_exercises(core::slice::from_ref(&*source)) {
                return Err(crate::interview::status(
                    "MESSAGE_FORM_3921_CHECK_THE_SHARE_COUNTS_UP_TO_SIX_DECIMAL_PLACES_AND_THE_NAMES",
                )
                .into());
            }
        }
        if items.is_empty() {
            return Ok(self.candidate_editor(draft));
        }
        let sources = items.iter().map(|(id, _)| *id).collect();
        let (premiums, repayments) =
            retirement_personal_records(retirement.iter().map(|(id, answered)| {
                let owner = items.iter().find(|(found, _)| found == id).unwrap().1.owner.owned();
                ((*id, owner), retirement_personal_choices(answered))
            }));
        let mut entries: Vec<_> = items
            .iter()
            .map(|(id, item)| {
                let mut entry = document_entry(item.clone(), &self.draft.income);
                // Topic answers cannot rename their independently accepted paper source.
                entry.label =
                    self.draft.income.iter().find(|old| old.id == *id).unwrap().label.clone();
                if let Some((_, answered)) = retirement.iter().find(|(found, _)| found == id) {
                    entry.income = answered.clone().into_source().without_repayment().income();
                }
                if let Some((_, source)) = railroad.iter().find(|(found, _)| found == id) {
                    entry.income = FederalIncome::Retirement {
                        data: crate::RetirementSource::Rrb1099R(source.clone()),
                    };
                }
                entry
            })
            .collect();
        for (id, index, field, answer) in cooperative_answers {
            let Some(FederalIncome::Cooperative { data }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            match answer {
                A::CooperativeAllocations { value } => {
                    data.more.allocations = value.into_iter().map(Into::into).collect();
                }
                answer => cooperative_followups::apply(data, &field, answer).map_err(|issue| {
                    SaveFailure::from(issue.in_field("answer").in_list("followups", index))
                })?,
            }
        }
        for (id, index, answers) in wage_answers {
            let Some(FederalIncomeEntry { income: FederalIncome::Wage { data }, .. }) =
                entries.iter_mut().find(|entry| entry.id == id)
            else {
                return Err(failure(index, "answer").into());
            };
            let Some(accepted) = data.answers_mut() else {
                return Err(failure(index, "answer").into());
            };
            let recorded = core::mem::take(&mut accepted.box14);
            *accepted = (&answers).into();
            // Only an open question takes an answer; a recorded meaning stays as it is.
            accepted.box14 = recorded
                .into_iter()
                .map(|meaning| {
                    accepted
                        .box14
                        .iter()
                        .find(|row| row.item == meaning.item && meaning.asks())
                        .cloned()
                        .unwrap_or(meaning)
                })
                .collect();
        }
        for (id, key, index, answer) in estate_answers {
            let entry = entries.iter_mut().find(|entry| entry.id == id).unwrap();
            let FederalIncome::EstateTrustPortfolio { data } = &mut entry.income else {
                unreachable!()
            };
            k1_followups::apply_estate(data, &key, answer).map_err(|issue| {
                SaveFailure::from(issue.in_field("answer").in_list("followups", index))
            })?;
        }
        let scorps: Vec<i64> = scorp_answers.iter().map(|(id, ..)| *id).collect();
        for (id, key, index, answer) in scorp_answers {
            let entry = entries.iter_mut().find(|entry| entry.id == id).unwrap();
            let FederalIncome::Scorp { data } = &mut entry.income else { unreachable!() };
            k1_followups::apply_scorp(data, &key, answer).map_err(|issue| {
                SaveFailure::from(issue.in_field("answer").in_list("followups", index))
            })?;
        }
        for entry in entries.iter_mut().filter(|entry| scorps.contains(&entry.id)) {
            if let FederalIncome::Scorp { data } = &mut entry.income {
                k1_followups::prune_scorp_rental_participation(data);
            }
        }
        // A 1099-LTC's topic answers belong to its accepted source, not to its paper.
        for (id, index, answer) in ltc_answers {
            let Some(FederalIncome::LtcBenefits { data }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            match answer {
                // An unanswered amount stays unanswered when saved as zero.
                A::LtcInterest { value } => {
                    data.interest = (value != 0 || data.interest.is_some()).then_some(value)
                }
                A::LtcBenefitOrigin { value } => data.benefit_origin = value,
                A::LtcAcceleratedOrigin { value } => data.accelerated_origin = value,
                A::LtcContract { value } => data.contract = (&value).into(),
                A::LtcInsured { value } => data.insured = value,
                _ => unreachable!(),
            }
        }
        for (id, index, answer) in &opportunity_fund_answers {
            let Some(FederalIncome::OpportunityFund { data }) =
                entries.iter_mut().find(|entry| entry.id == *id).map(|entry| &mut entry.income)
            else {
                return Err(failure(*index, "answer").into());
            };
            match answer {
                A::OpportunityFundEvent { value } => data.event = value.clone(),
                A::OpportunityFundSpecial { value } => data.special = *value,
                _ => unreachable!(),
            }
        }
        for (id, index, answer) in &opportunity_fund_answers {
            let FederalIncome::OpportunityFund { data } =
                &entries.iter().find(|entry| entry.id == *id).unwrap().income
            else {
                unreachable!()
            };
            if require_answers
                && !data.left_out()
                && data.event.is_none()
                && matches!(answer, A::OpportunityFundEvent { .. })
            {
                return Err(ValidationIssue::new("QOF_EVENT_NEEDED")
                    .at_field("value")
                    .in_field("answer")
                    .in_list("followups", *index)
                    .into());
            }
        }
        // A sale's topic answers belong to its accepted source, not to its paper.
        for (id, index, answer) in sale_answers {
            let Some(income) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            match (income, answer) {
                (FederalIncome::Sale { data }, A::SaleAdjustment { value }) => {
                    data.adjustment = value
                }
                (FederalIncome::Sale { data }, A::SaleTreatment { value }) => data.special = value,
                (FederalIncome::Section1256 { data }, A::ContractsSpecial { value }) => {
                    data.special = value
                }
                _ => return Err(failure(index, "answer").into()),
            }
        }
        for (id, index, answer) in misc_answers {
            let Some(FederalIncome::Misc1099 { data }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            match answer {
                A::MiscellaneousUse { value } => data.other_use = value,
                A::AttorneyClientFunds { value } => data.client_funds = value,
                _ => unreachable!(),
            }
        }
        // Preserve the accepted paper-entry corrections together with the independent answers.
        for (id, source) in broker_sources {
            if let Some(FederalIncome::Broker { data }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            {
                *data = source;
            }
        }
        for (id, index, value) in nonemployee_answers {
            let Some(FederalIncome::Nec1099 { data }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            data.belongs = value;
        }
        // A 1099-G's topic answers belong to its accepted source, not to its paper.
        for (id, index, answer) in government_answers {
            let Some(FederalIncome::GovernmentPayment { data }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            match answer {
                A::GovernmentGrantUse { value } => data.grant_use = value,
                A::GovernmentMedicalLeave { value } => data.family_leave_medical = value,
                A::GovernmentRepayments { value } => data.repayments = value,
                A::GovernmentSpecial { value } => data.special = value,
                _ => unreachable!(),
            }
        }
        // A W-2G's topic answer belongs to its accepted source, not to its paper.
        for (id, index, value) in gambling_answers {
            let Some(FederalIncome::Gambling { data: crate::GamblingSource::W2g(data) }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            data.special = value;
        }
        // A 1099-SA's topic answers belong to its accepted source, not to its paper.
        for (id, index, answer) in medical_answers {
            let Some(FederalIncome::MedicalDistribution { data }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            match answer {
                A::MedicalPayment { value } => data.payment = value,
                A::MedicareSpousal { value } => data.medicare_from_spouse = value,
                A::MedicarePrior { value } => data.medicare_prior = value,
                _ => unreachable!(),
            }
        }
        // A 1099-QA's treatment belongs to its accepted source, not to its paper.
        for (id, index, answer) in able_answers {
            let Some(FederalIncome::Able { data }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            if let A::AbleTreatment { value } = answer {
                data.treatment = value;
            }
        }
        // A Form 2439's topic answers belong to its accepted source, not to its paper.
        for (id, index, answer) in fund_answers {
            let Some(FederalIncome::UndistributedGain { data }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            match answer {
                A::DividendFundExclusion { value } => data.section_1202_exclusion = value,
                A::UndistributedGainSpecial { value } => data.special = value,
                _ => unreachable!(),
            }
        }
        // A 1099-Q's topic answers belong to its accepted source, not to its paper.
        for (id, index, answer) in education_answers {
            let Some(FederalIncome::EducationDistribution { data }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            match answer {
                A::EducationBeneficiary { value } => {
                    data.beneficiary = value.as_ref().map(Into::into);
                }
                A::CoverdellRecords { opening, contributions, late_rollovers } => {
                    data.coverdell_basis_start = opening;
                    data.coverdell_contributions = contributions;
                    data.late_rollovers = late_rollovers;
                }
                A::EducationRedeposited { value } => data.redeposited = value,
                A::EducationTaxableTransfer { value } => data.taxable_transfer_part = value,
                A::EducationRothRecords { value } => data.roth_rollover = value,
                _ => unreachable!(),
            }
        }
        // The rental's section 179 answer belongs to its accepted record.
        for (id, index, value) in rental_section179_answers {
            let Some(FederalIncome::Rental { data }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            if require_answers && value.is_none() {
                return Err(ValidationIssue::new("RENTAL_SECTION179_BUSINESS_NEEDED")
                    .at_field("value")
                    .in_field("answer")
                    .in_list("followups", index)
                    .into());
            }
            if value.is_some() {
                data.section179_business = value;
            }
        }
        // The rental's prior home-use carries belong to its accepted record.
        for (id, index, value) in rental_home_answers {
            let Some(FederalIncome::Rental { data }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            if require_answers && value.is_none() {
                return Err(ValidationIssue::new("RENTAL_HOME_CARRIES_NEEDED")
                    .at_field("value")
                    .in_field("answer")
                    .in_list("followups", index)
                    .into());
            }
            if value.is_some() {
                data.prior_home_carry = value.clone();
            }
        }
        // A refund's deduction belongs to its accepted source, not to its paper.
        for (id, index, value) in refund_answers {
            let Some(FederalIncome::GovernmentPayment { data }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            let Some(value) = value else {
                if require_answers {
                    return Err(ValidationIssue::new("REFUND_DEDUCTION_NEEDED")
                        .at_field("value")
                        .in_field("answer")
                        .in_list("followups", index)
                        .into());
                }
                continue;
            };
            if let Some(issue) = refund_deduction_issue(&value) {
                return Err(issue
                    .in_field("value")
                    .in_field("answer")
                    .in_list("followups", index)
                    .into());
            }
            apply_refund_deduction(data, &value);
        }
        // A 1099-DIV's topic answers belong to its accepted source, not to its paper.
        for (id, index, answer) in dividend_answers {
            let Some(FederalIncome::Dividend { data }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            match answer {
                A::DividendNominee { value } => data.nominee = value,
                A::DividendEligibility { value } => data.reit_eligible = value,
                A::DividendAfterDeath { value } => data.after_death = value,
                A::DividendFundExclusion { value } => data.section_1202_exclusion = value,
                A::DividendCapitalReturns { value } => {
                    data.capital_returns = value;
                    data.normalize_capital_return_kinds();
                    if require_answers
                        && let Some(lot) = data
                            .capital_returns
                            .iter()
                            .position(crate::CapitalReturn1099Div::needs_liquidation_status)
                    {
                        return Err(ValidationIssue::new("DIVIDEND_LIQUIDATION_STAGE_NEEDED")
                            .at_field("kind")
                            .in_list("value", lot)
                            .in_field("answer")
                            .in_list("followups", index)
                            .into());
                    }
                }
                A::DividendTreatment { value } => data.special = value,
                _ => unreachable!(),
            }
        }
        // A 1099-INT's topic answers belong to its accepted source, not to its paper.
        for (id, index, answer) in interest_answers {
            let Some(FederalIncome::Interest { data: crate::InterestSource::Int1099(data) }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            match answer {
                A::InterestNominee { value } => data.nominee = value,
                A::InterestAfterDeath { value } => data.after_death = value,
                A::InterestAccrued { value } => data.accrued_interest = value,
                A::InterestTreatment { value } => data.special = value,
                A::InterestAdjustment { value } => data.adjustments = value,
                _ => unreachable!(),
            }
        }
        // A 1099-OID's topic answers belong to its accepted source, not to its paper.
        for (id, index, answer) in oid_answers {
            let Some(FederalIncome::Oid { data }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            match answer {
                A::OidNominee { value } => data.nominee = value,
                A::OidPremium { value } => data.tax_exempt_bond_premium = value,
                A::OidPrivateActivity { value } => data.private_activity = value,
                A::OidTreatment { value } => data.special = value,
                A::InterestAdjustment { value } => data.adjustments = value,
                _ => unreachable!(),
            }
        }
        for (id, index, value) in &redemption_answers {
            let Some(entry) = entries.iter_mut().find(|entry| entry.id == *id) else {
                return Err(failure(*index, "answer").into());
            };
            let FederalIncome::BondRedemption { data } = &mut entry.income else {
                return Err(failure(*index, "answer").into());
            };
            data.form_8815 = value.as_ref().map(|link| link.form_8815);
        }
        if let Some((id, issue)) = crate::savings_bonds::redemption_link_issue(&entries)
            && let Some((_, index, _)) =
                redemption_answers.iter().find(|(source, _, _)| *source == id)
        {
            return Err(issue
                .in_field("value")
                .in_field("answer")
                .in_list("followups", *index)
                .into());
        }
        for (id, index, value) in savings_bond_answers {
            let mut resolved = crate::savings_bonds::resolved_source(&entries, id);
            let Some(entry) = entries.iter_mut().find(|entry| entry.id == id) else {
                return Err(failure(index, "answer").into());
            };
            let FederalIncome::SavingsBonds { data } = &mut entry.income else {
                return Err(failure(index, "answer").into());
            };
            data.answers = value.map(Into::into);
            if let Some(resolved) = &mut resolved {
                resolved.answers = data.answers.clone();
            }
            if let Some(answers) = &data.answers
                && let Some(natural) = resolved.as_ref().unwrap_or(data).natural()
            {
                let issue = savings_bonds_issue(&natural).or_else(|| {
                    (!crate::savings_bonds::sources(&self.draft.income).iter().any(|source| {
                        source.id == answers.interest_statement
                            && source.owner == entry.owner.owned()
                            && natural.redemption_interest <= source.gross
                    }))
                    .then(|| {
                        ValidationIssue::new("SAVINGS_BONDS_SOURCE_INVALID")
                            .at_field("interest_statement")
                    })
                });
                if let Some(issue) = issue {
                    return Err(issue
                        .in_field("value")
                        .in_field("answer")
                        .in_list("followups", index)
                        .into());
                }
            }
        }
        // Stock-sale answers update their accepted source after paper reconciliation.
        for (id, index, field, answer) in qsbs_answers {
            let Some(FederalIncome::QualifiedSmallBusinessSale { data }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            crate::qsbs_followups::apply(data, &field, answer, require_answers)
                .map_err(|issue| issue.in_field("answer").in_list("followups", index))?;
        }
        // A home sale's facts belong to its accepted source, not to its paper.
        for (id, index, value) in home_answers {
            let Some(FederalIncome::HomeSale { data }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            data.set_use(value);
            if let Some(issue) = data.answers_issue(self.joint()) {
                return Err(issue
                    .in_field("MainHome")
                    .in_field("value")
                    .in_field("answer")
                    .in_list("followups", index)
                    .into());
            }
        }
        for (id, answer) in debt_answers {
            let entry = entries.iter_mut().find(|entry| entry.id == id).unwrap();
            debt_followups::apply(entry, answer).map_err(|issue| {
                debt_followups::bind_issue(
                    id,
                    issue,
                    &submitted,
                    matches!(entry.income, FederalIncome::PropertyAcquisition { .. }),
                )
                .unwrap_or_else(|| {
                    ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("followups").into()
                })
            })?;
        }
        for entry in &entries {
            if let Some(issue) = debt_followups::answer_issue(
                entry,
                u32::try_from(draft.tax_year).unwrap_or(0),
                require_answers,
            ) && let Some(failure) = debt_followups::bind_issue(
                entry.id,
                issue,
                &submitted,
                matches!(entry.income, FederalIncome::PropertyAcquisition { .. }),
            ) {
                return Err(failure);
            }
        }
        // A cooperative's credit items named on its answers take their identity here.
        for entry in &mut entries {
            if !matches!(entry.income, FederalIncome::Cooperative { .. }) {
                continue;
            }
            let accepted = self
                .draft
                .income
                .iter()
                .find(|old| old.id == entry.id)
                .map_or_else(Vec::new, |old| crate::cooperative::other_ids(&old.income));
            assign_record_ids(
                crate::cooperative::other_ids_mut(&mut entry.income).into_iter(),
                accepted.into_iter(),
                &mut draft.next_id,
            )
            .map_err(|(key, _, child)| {
                SaveFailure::from(
                    ValidationIssue::new(key)
                        .at_field("id")
                        .in_list("other", child)
                        .in_field("credit_facts")
                        .in_field("more"),
                )
            })?;
        }
        related_forms::prepare_component_ids(&mut entries, &self.draft.income, &mut draft.next_id)
            .map_err(|(id, issue)| {
                let key = alloc::format!("income/{id}/parts");
                let index = submitted
                    .iter()
                    .find(|(_, found, _)| *found == key)
                    .or_else(|| submitted.iter().find(|(found, _, _)| *found == id))
                    .unwrap()
                    .2;
                issue
                    .map_field_prefix(&["parts"], &["value"])
                    .in_field("answer")
                    .in_list("followups", index)
            })?;
        related_forms::prepare_annual_ids(&mut entries, &self.draft.income, &mut draft.next_id)
            .map_err(|(id, issue)| {
                let index = submitted
                    .iter()
                    .find(|(source, key, _)| {
                        *source == id && *key == alloc::format!("income/{id}/adjustments")
                    })
                    .unwrap()
                    .2;
                issue
                    .map_field_prefix(&["adjustments"], &["value"])
                    .in_field("answer")
                    .in_list("followups", index)
            })?;
        related_forms::prepare_activity_ids(&mut entries, &self.draft.income, &mut draft.next_id)
            .map_err(|(id, issue)| {
            let entry = entries.iter().find(|entry| entry.id == id).unwrap();
            let source =
                document_item(entry, &[], crate::federal_rules::separate_return(&draft)).unwrap();
            k1_followups::bind_issue(&source.document, issue, &submitted, id).unwrap_or_else(|| {
                ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("followups").into()
            })
        })?;
        // Apply personal answers only after accepted statement identities and
        // printed corrections have been reconciled in the candidate.
        for (id, statement_id, index, answer) in child_dividend_answers {
            let Some(FederalIncome::ChildElection { data: child }) =
                entries.iter_mut().find(|entry| entry.id == id).map(|entry| &mut entry.income)
            else {
                return Err(failure(index, "answer").into());
            };
            let Some(data) = child.statements.iter_mut().find_map(|statement| match statement {
                crate::ChildIncome::Dividend { statement_id: current, data }
                    if *current == statement_id =>
                {
                    Some(data)
                }
                _ => None,
            }) else {
                return Err(failure(index, "answer").into());
            };
            match answer {
                A::DividendCapitalReturns { value } => {
                    data.capital_returns = value;
                    data.normalize_capital_return_kinds();
                    if let Some(issue) =
                        child_dividend_answer_issue(&crate::FederalDividend::from(data.as_ref()))
                        && (require_answers || issue.key != "DIVIDEND_LIQUIDATION_STAGE_NEEDED")
                    {
                        return Err(issue.in_field("answer").in_list("followups", index).into());
                    }
                }
                A::DividendTreatment { value } => data.special = value,
                _ => return Err(failure(index, "answer").into()),
            }
        }
        for entry in entries {
            let position = draft.income.iter().position(|old| old.id == entry.id).unwrap();
            draft.income[position] = entry;
        }
        apply_retirement_personal_records(&mut draft, &sources, premiums, repayments);
        if !espp_answers.is_empty() {
            let switched: Vec<i64> = espp_answers
                .iter()
                .filter_map(|(id, _, answer)| {
                    let A::EsppAcquisition { value } = answer else {
                        return None;
                    };
                    let before = crate::espp_return::answers(&self.draft, *id)
                        .and_then(|row| row.acquisition.as_ref());
                    let unchanged = match (before, value.as_ref()) {
                        (
                            Some(crate::EsppAcquisitionChoice::Printed { document_id: before }),
                            Some(crate::EsppAcquisitionPage::Printed { document_id: after }),
                        ) => before == after,
                        (
                            Some(crate::EsppAcquisitionChoice::Records { .. }),
                            Some(crate::EsppAcquisitionPage::Records { .. }),
                        )
                        | (None, None) => true,
                        _ => false,
                    };
                    (!unchanged).then_some(*id)
                })
                .collect();
            let recorded: Vec<i64> = draft
                .income
                .iter()
                .filter(|entry| {
                    matches!(&entry.income, FederalIncome::EsppSaleSource { data } if data.plan_record.is_some())
                })
                .map(|entry| entry.id)
                .collect();
            let screening = draft.screening.as_mut().unwrap();
            for (id, _, answer) in &espp_answers {
                if !screening.espp_sale_answers.iter().any(|row| row.document_id == *id) {
                    screening.espp_sale_answers.push(crate::EsppSaleAnswers::unanswered(*id));
                }
                let row = screening
                    .espp_sale_answers
                    .iter_mut()
                    .find(|row| row.document_id == *id)
                    .unwrap();
                match answer {
                    A::EsppAcquisition { value } => row.acquisition = value.clone().map(Into::into),
                    A::EsppFixedGrantPrice { value } if recorded.contains(id) => {
                        row.fixed_grant_price = value.clone()
                    }
                    A::EsppFixedGrantPrice { value } if !switched.contains(id) => {
                        if let Some(crate::EsppAcquisitionChoice::Printed { document_id }) =
                            row.acquisition.as_ref()
                            && let Some(source) = screening
                                .espp_acquisitions
                                .iter_mut()
                                .find(|source| source.id == *document_id)
                        {
                            source.fixed_grant_price = value.clone();
                        }
                    }
                    A::EsppFixedGrantPrice { .. } => {}
                    A::EsppAdjustments { value } => row.more = value.clone(),
                    A::EsppWageInclusion { value } => row.wage_inclusion = *value,
                    _ => unreachable!(),
                }
            }
            for (id, index, _) in &espp_answers {
                let entry = draft.income.iter().find(|entry| entry.id == *id).unwrap();
                if let Err(issue) = crate::espp_return::resolve(&draft, entry) {
                    if issue.key == "WARN_ESPP_SALE_LEFT_OUT"
                        || issue.key == "ESPP_FIXED_GRANT_PRICE_NEEDED" && switched.contains(id)
                    {
                        continue;
                    }
                    let field = if issue.key == "ESPP_WAGE_INCLUSION_NEEDED" {
                        "espp_wage_inclusion"
                    } else if matches!(
                        issue.key,
                        "ESPP_FIXED_GRANT_PRICE_NEEDED" | "ESPP_RECORD_GRANT_PRICE_NEEDED"
                    ) {
                        "espp_fixed_grant_price"
                    } else if issue.key == "SALE_WASH_SALE_ABOVE_LOSS" {
                        "espp_adjustments"
                    } else {
                        "espp_acquisition"
                    };
                    if let Some((_, _, submitted_index)) =
                        submitted.iter().find(|(source, key, _)| {
                            *source == *id && *key == alloc::format!("income/{id}/{field}")
                        })
                    {
                        let missing = matches!(
                            issue.key,
                            "ESPP_ACQUISITION_NEEDED"
                                | "ESPP_FIXED_GRANT_PRICE_NEEDED"
                                | "ESPP_RECORDS_GRANT_PRICE_NEEDED"
                                | "ESPP_RECORD_GRANT_PRICE_NEEDED"
                                | "ESPP_WAGE_INCLUSION_NEEDED"
                        );
                        if !missing || require_answers {
                            let issue = match field {
                                "espp_adjustments" => issue.map_field_prefix(&["more"], &["value"]),
                                "espp_fixed_grant_price" => {
                                    issue.map_field_prefix(&["fixed_grant_price"], &["value"])
                                }
                                "espp_wage_inclusion" => {
                                    issue.map_field_prefix(&["wage_inclusion"], &["value"])
                                }
                                _ => issue.map_field_prefix(&["acquisition"], &["value"]),
                            };
                            return Err(issue
                                .in_field("answer")
                                .in_list("followups", *submitted_index)
                                .into());
                        }
                    } else {
                        let _ = index;
                    }
                }
            }
        }

        if !plan_sale_answers.is_empty() {
            fill_unasked(&mut draft);
            let screening = draft.screening.as_mut().unwrap();
            for (id, _, answer) in &plan_sale_answers {
                match answer {
                    A::IsoAdjustments { .. } | A::IsoWageInclusion { .. } => {
                        if !screening.iso_sale_answers.iter().any(|row| row.document_id == *id) {
                            screening.iso_sale_answers.push(crate::IsoSaleAnswers {
                                document_id: *id,
                                more: None,
                                wage_inclusion: None,
                            });
                        }
                        let row = screening
                            .iso_sale_answers
                            .iter_mut()
                            .find(|row| row.document_id == *id)
                            .unwrap();
                        match answer {
                            A::IsoAdjustments { value } => row.more = value.clone(),
                            A::IsoWageInclusion { value } => row.wage_inclusion = *value,
                            _ => unreachable!(),
                        }
                    }
                    _ => {
                        if !screening.nua_sale_answers.iter().any(|row| row.document_id == *id) {
                            screening.nua_sale_answers.push(crate::NuaSaleAnswers {
                                document_id: *id,
                                more: None,
                                ownership: None,
                            });
                        }
                        let row = screening
                            .nua_sale_answers
                            .iter_mut()
                            .find(|row| row.document_id == *id)
                            .unwrap();
                        match answer {
                            A::NuaAdjustments { value } => row.more = value.clone(),
                            A::NuaOwnership { value } => row.ownership = value.clone(),
                            _ => unreachable!(),
                        }
                    }
                }
            }
            // The ISO W-2 question goes once the sale has no compensation.
            crate::iso_return::reconcile(&mut draft);
            for (id, _, _) in &plan_sale_answers {
                let entry = draft.income.iter().find(|entry| entry.id == *id).unwrap();
                let (resolved, missing_keys): (Result<(), _>, &[&str]) = match entry.income {
                    FederalIncome::IsoSaleSource { .. } => (
                        crate::iso_return::resolve(&draft, entry).map(|_| ()),
                        &["ISO_SALE_ADJUSTMENTS_NEEDED", "ISO_WAGE_INCLUSION_NEEDED"],
                    ),
                    _ => (
                        crate::nua_sale::resolve(&draft, entry).map(|_| ()),
                        &["NUA_SALE_ADJUSTMENTS_NEEDED", "NUA_OWNERSHIP_NEEDED"],
                    ),
                };
                let Err(issue) = resolved else {
                    continue;
                };
                // The answer row that owns the issue: its first path field.
                let iso = matches!(entry.income, FederalIncome::IsoSaleSource { .. });
                let (field, prefix) = match issue.key {
                    "ISO_WAGE_INCLUSION_NEEDED" => ("iso_wage_inclusion", "wage_inclusion"),
                    "NUA_OWNERSHIP_NEEDED"
                    | "NUA_DIED_BEFORE_DISTRIBUTED"
                    | "NUA_SOLD_BEFORE_DIED" => ("nua_ownership", "ownership"),
                    // Every other refusal belongs to the adjustments answer.
                    _ if iso => ("iso_adjustments", "more"),
                    _ => ("nua_adjustments", "more"),
                };
                if missing_keys.contains(&issue.key) && !require_answers {
                    continue;
                }
                if let Some((_, _, submitted_index)) = submitted.iter().find(|(source, key, _)| {
                    *source == *id && *key == alloc::format!("income/{id}/{field}")
                }) {
                    return Err(issue
                        .map_field_prefix(&[prefix], &["value"])
                        .in_field("answer")
                        .in_list("followups", *submitted_index)
                        .into());
                }
            }
        }

        if !social_premium_answers.is_empty() {
            fill_unasked(&mut draft);
            let allocations = &mut draft.adjustments.as_mut().unwrap().social_premiums;
            for (id, value, reimbursed) in social_premium_answers {
                allocations.retain(|row| row.document_id != id);
                if let Some(usage) = value {
                    allocations.push(crate::SocialPremiumAllocation {
                        document_id: id,
                        usage,
                        reimbursed,
                    });
                }
            }
            if let Some((id, issue)) = crate::social_documents::issue(&draft, require_answers)
                && let Some((_, _, index)) = submitted.iter().find(|(source, key, _)| {
                    *source == id && *key == alloc::format!("income/{id}/social_premiums")
                })
            {
                return Err(issue.in_field("answer").in_list("followups", *index).into());
            }
        }
        let validate_items = |draft: &FederalDraft, required: bool| -> Result<(), SaveFailure> {
            for (id, item) in &items {
                if let Some(issue) = qbi_group_input_issue(item, false)
                    .or_else(|| {
                        investment_answer_conflict(
                            &item.document,
                            draft.income.iter().find(|e| e.id == *id).map(|e| &e.income),
                        )
                        .filter(|(_, issue)| {
                            // Unanswered lots are asked on the topic page, as these answers are.
                            let lots_unanswered = || {
                                draft.income.iter().any(|e| {
                                    e.id == *id
                                        && matches!(&e.income, FederalIncome::Dividend { data }
                                            if data.capital_returns.is_empty())
                                })
                            };
                            required
                                || !(matches!(
                                    issue.key,
                                    "DIV_REIT_ELIGIBILITY_NEEDED"
                                        | "DIV_SECTION_1202_NEEDS_EXCLUSION"
                                ) || issue.key == "DIV_CAPITAL_RETURNS_RECONCILE"
                                    && lots_unanswered())
                        })
                        .map(|(_, issue)| issue)
                    })
                    .or_else(|| {
                        draft.income.iter().find(|entry| entry.id == *id).and_then(|entry| {
                            k1_followups::scorp_topic_issue(&entry.income, required).or_else(|| {
                                k1_followups::estate_topic_issue(&entry.income, required)
                            })
                        })
                    })
                    .or_else(|| {
                        draft.income.iter().find(|entry| entry.id == *id).and_then(|entry| {
                            match &entry.income {
                                FederalIncome::SavingsBonds { data } => {
                                    let resolved =
                                        crate::savings_bonds::resolved_source(&draft.income, *id);
                                    savings_bonds_source_issue(
                                        resolved.as_ref().unwrap_or(data),
                                        required,
                                    )
                                }
                                _ => None,
                            }
                        })
                    })
                    .or_else(|| k1_followups::topic_issue(&item.document, required))
                    .or_else(|| k1_followups::income_source_issue(draft, &item.document, required))
                    .or_else(|| {
                        draft
                            .income
                            .iter()
                            .find(|entry| entry.id == *id)
                            .and_then(wage_answer_issue)
                            .filter(|issue| {
                                required
                                    || !matches!(
                                        issue.key,
                                        "W2_TP_OCCUPATION_NEEDED"
                                            | "W2_TT_FLSA_NEEDED"
                                            | "W2_BOX14_KIND_NEEDED"
                                            | "W2_BOX14_STATE_NEEDED"
                                    )
                            })
                    })
                    .or_else(|| match &item.document {
                        IncomeSource::Farming(_) => {
                            draft.income.iter().find(|entry| entry.id == *id).and_then(|entry| {
                                let FederalIncome::Cooperative { data } = &entry.income else {
                                    return None;
                                };
                                patr_input_issue(&crate::Patr1099::from(data), true)
                            })
                        }
                        IncomeSource::InvestmentSale {
                            value: crate::InvestmentSaleDocument::Sale1099B(value),
                        } => sale_issue(value).or_else(|| {
                            // The sale's answers join its paper for the answer-dependent guards.
                            draft.income.iter().find(|entry| entry.id == *id).and_then(|entry| {
                                let FederalIncome::Sale { data } = &entry.income else {
                                    return None;
                                };
                                crate::interview::native_sale_issue(&crate::FederalSale::from(data))
                            })
                        }),
                        IncomeSource::OtherIncome {
                            value: crate::OtherIncomeDocument::Form1099Misc(_),
                        } => draft.income.iter().find(|entry| entry.id == *id).and_then(|entry| {
                            let FederalIncome::Misc1099 { data } = &entry.income else {
                                return None;
                            };
                            crate::information_returns::misc_input_issue(
                                &crate::Form1099Misc::from(data),
                                required,
                            )
                        }),
                        IncomeSource::SelfEmployment {
                            value: crate::SelfEmploymentDocument::Form1099Nec(value),
                        } => value.issue(),
                        IncomeSource::SelfEmployment {
                            value: crate::SelfEmploymentDocument::Form1099K(_),
                        } => draft.income.iter().find(|entry| entry.id == *id).and_then(|entry| {
                            let FederalIncome::K1099 { data } = &entry.income else {
                                return None;
                            };
                            crate::information_returns::k_input_issue(
                                &crate::Form1099K::from(data),
                                required || !data.parts.is_empty(),
                            )
                        }),
                        _ => None,
                    })
                    .or_else(|| {
                        draft.income.iter().find(|entry| entry.id == *id).and_then(|entry| {
                            match &entry.income {
                                FederalIncome::Nec1099 { data } => {
                                    crate::information_returns::nec_issue(
                                        &crate::Form1099Nec::from(data),
                                    )
                                }
                                _ => None,
                            }
                        })
                    })
                    .or_else(|| wage_reference_issue(draft, item))
                    .or_else(|| {
                        submitted
                            .iter()
                            .any(|(source, key, _)| {
                                *source == *id && *key == alloc::format!("income/{id}/community")
                            })
                            .then(|| document_community_issue(draft, item))
                            .flatten()
                    })
                    .or_else(|| {
                        let (_, repaid) = retirement
                            .iter()
                            .find(|(found, _)| found == id)
                            .map_or((Vec::new(), None), |(_, answered)| {
                                retirement_personal_choices(answered)
                            });
                        (repaid.is_some()
                            && !draft
                                .adjustments
                                .iter()
                                .flat_map(|a| &a.ira_basis)
                                .any(|basis| basis.owner == item.owner.owned()))
                        .then(|| {
                            ValidationIssue::new("R_REPAYMENT_NEEDS_FORM_8606")
                                .at_field("repaid")
                                .in_field("exception")
                                .in_field("code")
                        })
                    })
                    .or_else(|| match &item.document {
                        IncomeSource::OtherIncome {
                            value: crate::OtherIncomeDocument::Ltc1099(_),
                        } => draft.income.iter().find(|entry| entry.id == *id).and_then(|entry| {
                            match &entry.income {
                                FederalIncome::LtcBenefits { data } => {
                                    data.record().topic_issue(draft, item.owner.owned(), required)
                                }
                                _ => None,
                            }
                        }),
                        IncomeSource::InvestmentSale {
                            value: crate::InvestmentSaleDocument::OpportunityFund(_),
                        } => draft.income.iter().find(|entry| entry.id == *id).and_then(|entry| {
                            match &entry.income {
                                FederalIncome::OpportunityFund { data } => data.issue(),
                                _ => None,
                            }
                        }),
                        IncomeSource::Retirement {
                            value: crate::RetirementDocument::Rrb1099R(_),
                        } => {
                            railroad.iter().find(|(found, _)| found == id).and_then(|(_, value)| {
                                rrb_topic_issue(value, required).or_else(|| {
                                    rrb_birth_issue(value, owner_birth(draft, item.owner))
                                        .filter(|_| required)
                                })
                            })
                        }
                        IncomeSource::OtherIncome {
                            value: crate::OtherIncomeDocument::Qa1099(_),
                        } => match draft.income.iter().find(|entry| entry.id == *id) {
                            Some(crate::FederalIncomeEntry {
                                income: FederalIncome::Able { data },
                                ..
                            }) => data.topic_issue(required),
                            _ => None,
                        },
                        IncomeSource::OtherIncome {
                            value: crate::OtherIncomeDocument::Q1099(value),
                        } => q1099_input_issue(value).or_else(|| {
                            match draft.income.iter().find(|entry| entry.id == *id) {
                                Some(crate::FederalIncomeEntry {
                                    income: FederalIncome::EducationDistribution { data },
                                    ..
                                }) => crate::FederalEducationDistribution::from(data)
                                    .answer_issue_with_required(required),
                                _ => None,
                            }
                        }),
                        IncomeSource::OtherIncome {
                            value:
                                crate::OtherIncomeDocument::Sa1099(_)
                                | crate::OtherIncomeDocument::HsaTrusteeStatement(_),
                        } => draft.income.iter().find(|entry| entry.id == *id).and_then(|entry| {
                            match &entry.income {
                                FederalIncome::MedicalDistribution { data } => {
                                    data.topic_issue(required)
                                }
                                _ => None,
                            }
                        }),
                        IncomeSource::Retirement { value: crate::RetirementDocument::R1099(_) } => {
                            retirement
                                .iter()
                                .find(|(found, _)| found == id)
                                .and_then(|(_, answered)| {
                                    crate::retirement::input_issue(answered, required).or_else(
                                        || {
                                            r1099_birth_issue(
                                                answered,
                                                owner_birth(draft, item.owner),
                                            )
                                            .filter(|_| required)
                                        },
                                    )
                                })
                                .map(|issue| {
                                    issue.map_field_prefix(
                                        &["public_safety"],
                                        &["public_safety", "rows"],
                                    )
                                })
                        }
                        _ => None,
                    })
                {
                    if let Some(FederalIncome::Cooperative { data }) =
                        draft.income.iter().find(|entry| entry.id == *id).map(|entry| &entry.income)
                        && let Some(failure) = cooperative_followups::bind_issue(
                            &crate::Patr1099::from(data),
                            issue.clone(),
                            &submitted,
                            *id,
                        )
                    {
                        return Err(failure);
                    }
                    if let Some(failure) =
                        k1_followups::bind_issue(&item.document, issue.clone(), &submitted, *id)
                    {
                        return Err(failure);
                    }
                    let binding = question_field(&issue);
                    let field = binding.map_or("", |(field, _, _)| field);
                    let key = alloc::format!("income/{id}/{field}");
                    let index = submitted
                        .iter()
                        .find(|(_, found, _)| *found == key)
                        .or_else(|| submitted.iter().find(|(found, _, _)| found == id))
                        .unwrap()
                        .2;
                    let issue = if let Some((_, path, target)) = binding {
                        issue.map_field_prefix(path, target)
                    } else {
                        issue
                    };
                    return Err(issue.in_field("answer").in_list("followups", index).into());
                }
            }
            Ok(())
        };
        validate_items(&draft, false)?;
        // The original multi-form disaster owner reads the same complete set
        // after personal answers have moved out of the printed source editor.
        if topic == TaxSection::Retirement {
            let physical: Vec<_> = items.iter().map(|(_, item)| item.clone()).collect();
            let targets: Vec<_> = items
                .iter()
                .map(|(id, _)| {
                    submitted
                        .iter()
                        .find(|(source, key, _)| {
                            source == id && *key == alloc::format!("income/{id}/disaster")
                        })
                        .and_then(|(_, _, index)| {
                            ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                                .at_field("value")
                                .in_field("answer")
                                .in_list("followups", *index)
                                .field
                        })
                })
                .collect();
            let designations: Vec<_> = items
                .iter()
                .enumerate()
                .filter_map(|(index, (id, item))| {
                    let (_, answered) = retirement.iter().find(|(found, _)| found == id)?;
                    Some((index, item.owner.owned(), answered.disaster.as_ref()?))
                })
                .collect();
            disaster_records(self, &physical, &designations).map_err(|failure| {
                failure
                    .map_list_item_field_prefix(
                        &["documents", "documents"],
                        &["document", "disaster"],
                        &[],
                        &alloc::vec![true; physical.len()],
                    )
                    .map_submitted_items(&["documents", "documents"], &targets)
            })?;
        }
        prepare_cooperative_source_ids(&mut draft, &self.draft).map_err(|(id, issue)| {
            let index = submitted
                .iter()
                .find(|(source, key, _)| {
                    *source == id && *key == alloc::format!("income/{id}/allocations")
                })
                .unwrap()
                .2;
            issue
                .map_field_prefix(&["allocations"], &["value"])
                .in_field("answer")
                .in_list("followups", index)
        })?;
        if let Some((id, issue)) = crate::cooperative::graph_issue_for_sources(&draft, &sources) {
            if let Some(FederalIncome::Cooperative { data }) =
                draft.income.iter().find(|entry| entry.id == id).map(|entry| &entry.income)
                && let Some(failure) = cooperative_followups::bind_issue(
                    &crate::Patr1099::from(data),
                    issue.clone(),
                    &submitted,
                    id,
                )
            {
                return Err(failure);
            }
            let issue = project_cooperative_issue(issue, false);
            let binding = question_field(&issue);
            let key = alloc::format!(
                "income/{id}/{}",
                binding.map_or("allocations", |(field, _, _)| field)
            );
            if let Some((_, _, index)) = submitted.iter().find(|(_, found, _)| *found == key) {
                let issue =
                    binding.map_or(issue.clone(), |(_, from, to)| issue.map_field_prefix(from, to));
                return Err(issue.in_field("answer").in_list("followups", *index).into());
            }
        }
        if let Some((id, issue)) = crate::estate_trust::graph_issue(&draft)
            && let Some(entry) = draft.income.iter().find(|entry| entry.id == id)
            && let Some(item) =
                document_item(entry, &[], crate::federal_rules::separate_return(&draft))
            && let Some(failure) = k1_followups::bind_issue(&item.document, issue, &submitted, id)
        {
            return Err(failure);
        }

        if let Some((Some(id), issue)) = health_correction_source_issue(&draft)
            && let Some((_, _, index)) = submitted.iter().find(|(source, key, _)| {
                *source == id && *key == alloc::format!("income/{id}/treatment")
            })
            && let Some(before) = self
                .draft
                .income
                .iter()
                .find(|entry| entry.id == id)
                .and_then(|entry| entry.income.roth())
            && let Some(after) = draft
                .income
                .iter()
                .find(|entry| entry.id == id)
                .and_then(|entry| entry.income.roth())
            && before.distributed != after.distributed
        {
            return Err(ValidationIssue::new(issue.key)
                .at_field("distributed")
                .in_field("roth")
                .in_field("code")
                .in_field("answer")
                .in_list("followups", *index)
                .into());
        }
        if let Some((Some(id), issue)) = health_correction_source_issue(&draft)
            && let Some((_, _, index)) = submitted.iter().find(|(source, key, _)| {
                *source == id && *key == alloc::format!("income/{id}/payment")
            })
        {
            let before = &self.draft.income.iter().find(|entry| entry.id == id).unwrap().income;
            let after = &draft.income.iter().find(|entry| entry.id == id).unwrap().income;
            return Err(changed_medical_payment_issue(issue.key, before, after)
                .in_field("answer")
                .in_list("followups", *index)
                .into());
        }
        if let Some((id, issue)) = crate::care_benefit_return::reference_issue(&draft)
            && let Some((_, _, index)) = submitted.iter().find(|(source, _, _)| *source == id)
        {
            let binding = question_field(&issue);
            let field = binding.map_or("ltc_contract", |(field, _, _)| field);
            let key = alloc::format!("income/{id}/{field}");
            let index = submitted
                .iter()
                .find(|(_, found, _)| *found == key)
                .map_or(*index, |(_, _, index)| *index);
            let issue = if let Some((_, path, target)) = binding {
                issue.map_field_prefix(path, target)
            } else {
                issue
            };
            return Err(issue.in_field("answer").in_list("followups", index).into());
        }
        if require_answers {
            validate_items(&draft, true)?;
        }
        for entry in &draft.income {
            if let Some(issue) = recorded_income_owner_issue(&draft, entry)
                && let Some((suffix, from, to)) = question_field(&issue)
                && let Some((_, _, index)) = submitted.iter().find(|(id, key, _)| {
                    *id == entry.id && *key == alloc::format!("income/{id}/{suffix}")
                })
            {
                return Err(issue
                    .map_field_prefix(from, to)
                    .in_field("answer")
                    .in_list("followups", *index)
                    .into());
            }
        }
        if let Some((owner, issue)) = crate::foreign_housing::source_pay_issue(&draft) {
            let changed: Vec<_> = draft.income.iter().filter(|entry| entry.owner == crate::DocumentOwner::from(owner))
                .filter(|entry| matches!(&entry.income, FederalIncome::Wage { data } if self.draft.income.iter().find(|old| old.id == entry.id).is_some_and(|old| matches!(&old.income, FederalIncome::Wage { data: before } if crate::FederalWage::from(data).abroad != crate::FederalWage::from(before).abroad))))
                .filter_map(|entry| submitted.iter().find(|(id, key, _)| *id == entry.id && *key == alloc::format!("income/{}/wage", entry.id)))
                .collect();
            if let [(_, _, index)] = changed.as_slice() {
                return Err(ValidationIssue::new(issue.key)
                    .at_field("abroad")
                    .in_field("answers")
                    .in_field("answer")
                    .in_list("followups", *index)
                    .into());
            }
        }
        for (id, item) in &items {
            let entry = draft.income.iter().find(|entry| entry.id == *id).unwrap();
            if let Some(issue) = crate::k1_activity::income_issue(&entry.income)
                && let Some(failure) =
                    k1_followups::bind_issue(&item.document, issue, &submitted, *id)
            {
                return Err(failure);
            }
        }
        if !crate::k1_activity::section179_amounts_in_range(&draft.income) {
            // The aggregate owner decides once. A separately failing source can
            // name its actual editable container; cross-parent failures stay global.
            for (id, item) in &items {
                let entry = draft.income.iter().find(|entry| entry.id == *id).unwrap();
                if !crate::k1_activity::section179_amounts_in_range(core::slice::from_ref(entry))
                    && let Some(failure) = k1_followups::bind_issue(
                        &item.document,
                        ValidationIssue::new(crate::k1_activity::ordinary::RANGE_KEY).at_record(),
                        &submitted,
                        *id,
                    )
                {
                    return Err(failure);
                }
            }
            return Err(ValidationIssue::new(crate::k1_activity::ordinary::RANGE_KEY)
                .at_field("followups")
                .into());
        }
        Ok(self.candidate_editor(draft))
    }
}

fn child_dividend_answer_issue(data: &crate::FederalDividend) -> Option<ValidationIssue> {
    crate::investment_statements::div_issue(data)
        .filter(|issue| {
            [
                "capital_returns",
                "nondividend_distributions",
                "cash_liquidation",
                "noncash_liquidation",
            ]
            .iter()
            .any(|field| issue.has_field_prefix(&[field]))
        })
        .map(|issue| {
            if issue.has_field_prefix(&["capital_returns"]) {
                issue.map_field_prefix(&["capital_returns"], &["value"])
            } else {
                ValidationIssue::new(issue.key).at_field("value")
            }
        })
        .or_else(|| {
            data.capital_returns
                .iter()
                .position(crate::CapitalReturn1099Div::needs_liquidation_status)
                .map(|index| {
                    ValidationIssue::new("DIVIDEND_LIQUIDATION_STAGE_NEEDED")
                        .at_field("kind")
                        .in_list("value", index)
                })
        })
}

// Preserve the owner's message and semantic suffix while selecting the actual
// submitted question, including keys containing schema-path punctuation.
fn keyed_followup_failure(failure: SaveFailure, keys: &[String]) -> SaveFailure {
    let targets: Vec<_> = keys
        .iter()
        .map(|key| {
            ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                .at_record()
                .in_key("followups", key.as_str())
                .field
        })
        .collect();
    failure.map_submitted_items(&["followups"], &targets)
}

// Shared by the pending list and automatic topic walk: optional questions do
// not create work merely because a document offers them.
fn unanswered_question_keys(
    draft: &FederalDraft,
    entry: &FederalIncomeEntry,
    row: &DocumentQuestion,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if let FederalIncome::ChildElection { data: child } = &entry.income
        && matches!(row.answer, A::DividendCapitalReturns { .. })
        && let Some(id) = row.key.split('/').nth(3).and_then(|value| value.parse::<i64>().ok())
        && let Some(data) = child.statements.iter().find_map(|statement| match statement {
            crate::ChildIncome::Dividend { statement_id, data, .. } if *statement_id == id => {
                Some(data)
            }
            _ => None,
        })
    {
        if let Some(issue) =
            child_dividend_answer_issue(&crate::FederalDividend::from(data.as_ref()))
        {
            missing.push(issue.key);
        }
        return missing;
    }
    // An owned 1099-DIV's lots answer boxes 3, 9 and 10 by the same rule as a
    // child's statement: lots that do not reconcile are this row's work.
    if let FederalIncome::Dividend { data } = &entry.income
        && matches!(row.answer, A::DividendCapitalReturns { .. })
    {
        if let Some(issue) = child_dividend_answer_issue(&crate::FederalDividend::from(data)) {
            missing.push(issue.key);
        }
        return missing;
    }
    if crate::qsbs_followups::missing(&row.answer) {
        missing.push("QSBS_ANSWER_NEEDED");
        return missing;
    }
    match &row.answer {
        A::DividendCapitalReturns { value }
            if value.iter().any(crate::CapitalReturn1099Div::needs_liquidation_status) =>
        {
            missing.push("DIVIDEND_LIQUIDATION_STAGE_NEEDED");
        }
        A::SavingsBondsRecords { value }
            if value.is_none()
                || matches!(&entry.income,
                FederalIncome::SavingsBonds { data } if crate::savings_bonds::resolved_source(&draft.income, entry.id)
                    .as_ref().unwrap_or(data).natural().is_none()) =>
        {
            missing.push("DOCUMENT_SAVINGS_BONDS_RECORDS");
        }
        A::RefundDeduction { .. }
            if matches!(&entry.income, FederalIncome::GovernmentPayment { data }
                if matches!(data.history, crate::TaxRefundHistory::Unanswered)) =>
        {
            missing.push("REFUND_DEDUCTION_NEEDED");
        }
        A::OpportunityFundEvent { value: None } if matches!(&entry.income, FederalIncome::OpportunityFund { data } if !data.left_out()) =>
        {
            missing.push("QOF_EVENT_NEEDED");
        }
        A::SocialPremiums { value: None } => missing.push("SOCIAL_PREMIUM_USE_NEEDED"),
        A::RentalSection179Business { value: None } => {
            missing.push("RENTAL_SECTION179_BUSINESS_NEEDED")
        }
        A::RentalHomeCarries { value: None } => missing.push("RENTAL_HOME_CARRIES_NEEDED"),
        A::SocialReimbursedPremiums { reimbursed: None } => {
            missing.push("SOCIAL_PREMIUM_REIMBURSEMENT_NEEDED")
        }
        A::SocialSharedPremiums { value, reimbursed } => {
            if value.is_none() {
                missing.push("SOCIAL_PREMIUM_USE_NEEDED");
            }
            if reimbursed.is_none() {
                missing.push("SOCIAL_PREMIUM_REIMBURSEMENT_NEEDED");
            }
        }
        A::DebtTreatment { value: None } => missing.push("DEBT_TREATMENT_NEEDED"),
        A::DebtProperty { value: None } => missing.push("DEBT_PROPERTY_NEEDED"),
        A::DebtAcquisition { value: None } => missing.push("DEBT_ACQUISITION_PROPERTY_NEEDED"),
        A::HomeSale { value: None } => missing.push("HOME_SALE_ANSWERS_NEEDED"),
        A::EsppAcquisition { value: None } => missing.push("ESPP_ACQUISITION_NEEDED"),
        A::EsppAcquisition { value: Some(crate::EsppAcquisitionPage::Records { facts }) }
            if facts.grant_price == crate::SourceCents::Unavailable =>
        {
            missing.push("ESPP_RECORDS_GRANT_PRICE_NEEDED")
        }
        A::EsppFixedGrantPrice { value: crate::SourceCents::Unavailable } => {
            missing.extend(["ESPP_FIXED_GRANT_PRICE_NEEDED", "ESPP_RECORD_GRANT_PRICE_NEEDED"])
        }
        A::EsppWageInclusion { value: None } => missing.push("ESPP_WAGE_INCLUSION_NEEDED"),
        A::MedicalPayment { value: None } => missing.push("SA1099_PAYMENT_RECORDS_NEEDED"),
        A::OidTreatment { .. } if matches!(&entry.income, FederalIncome::Oid { data } if data.special.is_none()) => {
            missing.push("DOCUMENT_ANSWER_OIDTREATMENT")
        }
        A::MedicareSpousal { value: None } => {
            missing.push("SA1099_MEDICARE_SPOUSAL_RECORDS_NEEDED")
        }
        A::MedicarePrior { value: None } => missing.push("SA1099_MEDICARE_PRIOR_RECORDS_NEEDED"),
        A::EducationBeneficiary { value: None } => missing.push("Q1099_BENEFICIARY_NEEDED"),
        A::CoverdellRecords { opening, late_rollovers, .. } => {
            if opening.is_none() {
                missing.push("Q1099_COVERDELL_BASIS_NEEDED");
            }
            if late_rollovers.is_none() {
                missing.push("Q1099_LATE_ROLLOVERS_NEEDED");
            }
        }
        A::EducationRothRecords { value: None } => missing.push("Q1099_ROTH_RECORDS_NEEDED"),
        A::Wage { answers } => {
            if !answers.calls.occupation.is_empty() && answers.tips_listed_occupation.is_none() {
                missing.push("W2_TP_OCCUPATION_NEEDED");
            }
            if !answers.calls.overtime.is_empty() && answers.overtime_flsa_required.is_none() {
                missing.push("W2_TT_FLSA_NEEDED");
            }
            if answers.box14.iter().any(|row| row.kind.is_none()) {
                missing.push("W2_BOX14_KIND_NEEDED");
            }
            if answers
                .box14
                .iter()
                .any(|row| row.kind.is_some_and(|k| k.needs_state()) && row.state.is_none())
            {
                missing.push("W2_BOX14_STATE_NEEDED");
            }
        }
        A::RetirementTreatment {
            code: Some(crate::RetirementTreatmentView::TrumpDeath { basis: None, .. }),
        } => missing.push("R_TRUMP_BASIS_NEEDED"),
        A::RetirementTreatment { code: None } => missing.push("R_TREATMENT_NEEDED"),
        A::RetirementMethod { method: None } => missing.push("R_TAXABLE_METHOD_NEEDED"),
        A::RetirementBasis { basis: None } => missing.push("ISSUE_BOX_10_BASIS_UNKNOWN"),
        A::EstateResearchCreditLine { value: None } => {
            missing.push("DOCUMENT_K1_RESEARCH_CREDIT_LINE")
        }
        A::EstateNiitAllocation { value: None } => missing.push("DOCUMENT_ESTATE_NIIT_ALLOCATION"),
        A::EstateNolRecords { value: None } => missing.push("DOCUMENT_ESTATE_NOL_RECORDS"),
        A::K1ResearchCreditLine { value: None } => missing.push("DOCUMENT_K1_RESEARCH_CREDIT_LINE"),
        A::K1K3Withholding { value } if value.is_empty() => {
            missing.push("K3_WITHHOLDING_NEEDS_EACH_PASSIVE_COUNTRY")
        }
        A::K1PartnerRecords { income_source, .. }
            if !row.k1.source_choices.contains(&income_source.index()) =>
        {
            missing.push(if row.k1.source_choices.contains(&0) {
                SOURCE_NOT_NEEDED
            } else {
                PARTNER_SOURCE_NEEDED
            });
        }
        A::K1PartnerParticipation { value: None } | A::K1ScorpParticipation { value: None } => {
            missing.push("DOCUMENT_K1_PARTICIPATION")
        }
        A::K1LimitedForSe { value: None } => missing.push("DOCUMENT_K1_PARTNER_RECORDS"),
        A::K1PartnerDisposition { value: None } | A::K1ScorpDisposition { value: None } => {
            missing.push("DOCUMENT_K1_DISPOSITION")
        }
        A::K1PartnerAmt { value: None } | A::K1ScorpAmt { value: None } => {
            missing.push("DOCUMENT_K1_AMT_RECORDS")
        }
        A::MiscellaneousUse { value: None } => missing.push("DOCUMENT_ANSWER_MISCELLANEOUSUSE"),
        A::K1Section179Amt { value: None } => missing.push("DOCUMENT_K1_SECTION179_AMT"),
        A::K1DepletionSources { value }
            if value.allocations.iter().any(|row| row.adjustment.is_none()) =>
        {
            missing.push("K1_DEPLETION_SOURCE_NEEDED");
        }
        A::K1EntityDisposition { value: None } => missing.push("DOCUMENT_K1_DISPOSITION"),
        A::EstateFinalSuccessor { value: None } => missing.push("ESTATE_SUCCESSOR_NEEDED"),
        A::K1PartnerBasisStart { value }
            if value.as_ref().and_then(crate::StrictPartnerStart::from_stored).is_none() =>
        {
            missing.push("DOCUMENT_K1_OPENING_BASIS")
        }
        A::K1ScorpBasisStart { value }
            if value.as_ref().and_then(crate::StrictScorpStart::from_stored).is_none() =>
        {
            missing.push("DOCUMENT_K1_OPENING_BASIS")
        }
        A::K1RentalParticipation { value: None } => missing.push("DOCUMENT_K1_RENTAL_RECORDS"),
        A::K1CreditConfirmations { housing: None } if !row.k1.housing_choices.is_empty() => {
            missing.push("K1_HOUSING_NEEDS_CONFIRMATION");
        }
        A::K1InvestmentRecords { investment: None, allowed_choices, .. }
            if !allowed_choices.is_empty() =>
        {
            missing.push("K1_INVESTMENT_NEEDS_CONFIRMATION");
        }
        A::Community { value: CommunityShare::NotNeeded } => {
            missing.push("ISSUE_COMMUNITY_ANSWER_NEEDED")
        }
        A::Management { value: None } => {
            missing.push("INJURED_SPOUSE_WITHHOLDING_MANAGEMENT_NEEDED")
        }
        _ => {}
    }
    missing
}

fn required_k1_question_issue(
    draft: &FederalDraft,
    entry: &FederalIncomeEntry,
) -> Option<ValidationIssue> {
    if matches!(&entry.income, FederalIncome::EstateTrustPortfolio { .. }) {
        return k1_followups::estate_topic_issue(&entry.income, true);
    }
    if !matches!(
        entry.income,
        FederalIncome::Partnership { .. }
            | FederalIncome::Scorp { .. }
            | FederalIncome::PartnershipActivities { .. }
            | FederalIncome::ScorpActivities { .. }
    ) {
        return None;
    }
    if let FederalIncome::Scorp { data } = &entry.income {
        return data.topic_issue(true);
    }
    let item = document_item(entry, &[], crate::federal_rules::separate_return(draft))?;
    k1_followups::topic_issue(&item.document, true)
}

// Whether a rental still lacks its section 179 trade-or-business answer.
pub(super) fn rental_section179_open(draft: &FederalDraft) -> bool {
    Interview::document_followup_rows(draft, Some(TaxSection::Rentals), None)
        .into_iter()
        .any(|(_, row)| matches!(row.answer, A::RentalSection179Business { value: None }))
}

pub(super) fn topic_followups_needed(draft: &FederalDraft, topic: TaxSection) -> bool {
    let rows = Interview::document_followup_rows(draft, Some(topic), None);
    super::followup_page::questions_open(draft, &rows)
        || rows.into_iter().any(|(id, row)| {
            draft.income.iter().find(|entry| entry.id == id).is_some_and(|entry| {
                !unanswered_question_keys(draft, entry, &row).is_empty()
                    || (!matches!(row.answer, A::Management { .. })
                        && required_k1_question_issue(draft, entry).is_some())
            })
        })
}

fn pending_document_name(name: &str) -> Vec<u8> {
    if name.trim().is_empty() {
        Vec::new()
    } else {
        Message::new("TEXT", alloc::vec![MessageArg::Text(name.into())]).encode()
    }
}

pub(super) fn pending_questions(
    draft: &FederalDraft,
    result: &FederalResult,
) -> Vec<(Option<i64>, &'static str, ReviewItem)> {
    let mut pending = Vec::new();
    // A pending row names its answer on that question's own page, also from Review.
    let questions = Interview::document_followup_rows(draft, None, Some(result));
    let mut add = |source: Option<i64>,
                   key: &'static str,
                   section: TaxSection,
                   document: Vec<u8>| {
        let field = source.and_then(|id| match section {
            TaxSection::Wages => {
                // The W-2's own questions are answered on the page's `questions`.
                let part: alloc::string::String = match key {
                    "W2_TP_OCCUPATION_NEEDED" => "occupation".into(),
                    "W2_TT_FLSA_NEEDED" => "overtime".into(),
                    "W2_BOX14_KIND_NEEDED" | "W2_BOX14_STATE_NEEDED" => {
                        let FederalIncome::Wage { data: crate::WageSource::W2(source) } =
                            &draft.income.iter().find(|entry| entry.id == id)?.income
                        else {
                            return None;
                        };
                        alloc::format!("box14/{}", source.box14_asks().first()?.item)
                    }
                    _ => return None,
                };
                ValidationIssue::new(key)
                    .at_field("answer")
                    .in_key("questions", alloc::format!("income/{id}/wage/{part}").as_str())
                    .field
                    .map(|field| field.square())
            }
            TaxSection::Education => education_answer_path(draft, id, key),
            TaxSection::Mortgage => mortgage_answer_path(draft, id, key),
            TaxSection::PassThrough if key == "ESTATE_QBI_APPORTIONMENT_NEEDED" => {
                estate_followups::apportionment_path(draft, id)
            }
            _ => questions
                .iter()
                .find(|(source, row)| {
                    *source == id
                        && draft.income.iter().find(|entry| entry.id == id).is_some_and(|entry| {
                            unanswered_question_keys(draft, entry, row).contains(&key)
                        })
                })
                .and_then(|(_, row)| {
                    let issue = if key == "K1_DEPLETION_SOURCE_NEEDED" {
                        let A::K1DepletionSources { value } = &row.answer else { return None };
                        let missing =
                            value.allocations.iter().find(|row| row.adjustment.is_none())?;
                        ValidationIssue::new(key)
                            .at_field("adjustment")
                            .in_key("allocations", missing.source)
                            .in_field("value")
                            .in_field("answer")
                    } else if key == "QOF_EVENT_NEEDED" {
                        ValidationIssue::new(key).at_field("value").in_field("answer")
                    } else if key == "ESPP_RECORDS_GRANT_PRICE_NEEDED" {
                        ValidationIssue::new(key)
                            .at_field("grant_price")
                            .in_field("facts")
                            .in_field("Records")
                            .in_field("value")
                            .in_field("answer")
                    } else {
                        ValidationIssue::new(key).at_field("answer")
                    };
                    // The person answers regrouped questions in their own page fields.
                    let list = match super::followup_page::kind(row) {
                        super::followup_page::Kind::Strict(_) => "questions",
                        super::followup_page::Kind::Rare => {
                            return Some(libertas::SchemaPath::root().field("situations").into());
                        }
                        _ => "followups",
                    };
                    issue.in_key(list, row.key.as_str()).field.map(|field| field.square())
                }),
        });
        let row = ReviewItem {
            field,
            item: if document.is_empty() {
                status(key)
            } else {
                Message::new(
                    "DOCUMENT_PENDING_ANSWER",
                    alloc::vec![MessageArg::Resource(key.into()), MessageArg::Encoded(document)],
                )
                .encode()
            },
            status: status(if coming_up(draft, section) {
                "REVIEW_COMING_UP"
            } else {
                "DOCUMENT_PENDING_FIX"
            }),
            section,
        };
        if !pending.iter().any(|(old_source, old_key, _)| *old_source == source && *old_key == key)
        {
            pending.push((source, key, row));
        }
    };
    for entry in
        draft.income.iter().filter(|entry| crate::trump_account::printed_excess(entry).is_some())
    {
        let facts = draft
            .screening
            .as_ref()
            .into_iter()
            .flat_map(|s| &s.trump_accounts)
            .filter(|account| account.owner == entry.owner.owned())
            .flat_map(|account| &account.returns)
            .find(|row| row.document_id == Some(entry.id));
        if facts.is_none() {
            add(
                Some(entry.id),
                "DOCUMENT_TRUMP_ACCOUNT_FACTS",
                TaxSection::OtherTaxes,
                followup_document_name(draft, entry),
            );
        }
    }
    for entry in &draft.income {
        let pending = match entry.income {
            FederalIncome::IsoSaleSource { .. } => crate::iso_return::resolve(draft, entry).err(),
            FederalIncome::NuaSaleSource { .. } => crate::nua_sale::resolve(draft, entry).err(),
            _ => None,
        };
        if let Some(issue) = pending {
            add(
                Some(entry.id),
                issue.key,
                TaxSection::InvestmentSales,
                followup_document_name(draft, entry),
            );
        }
    }
    for entry in &draft.income {
        if matches!(entry.income, FederalIncome::EsppSaleSource { .. })
            && let Err(issue) = crate::espp_return::resolve(draft, entry)
            && issue.key != "WARN_ESPP_SALE_LEFT_OUT"
            && (issue.key != "ESPP_FIXED_GRANT_PRICE_NEEDED"
                || crate::espp_return::fixed_price_needed(draft, entry))
        {
            add(
                Some(entry.id),
                issue.key,
                TaxSection::InvestmentSales,
                followup_document_name(draft, entry),
            );
        }
    }
    if let Some((id, issue)) = crate::social_documents::issue(draft, true)
        && let Some(entry) = draft.income.iter().find(|entry| entry.id == id)
    {
        add(Some(id), issue.key, TaxSection::Retirement, followup_document_name(draft, entry));
    }
    if matches!(
        draft.adjustments.as_ref().map(|a| &a.social_election),
        Some(crate::SocialElection::Compare { complete: true, .. })
    ) && let Some(id) = crate::social_documents::unallocated_railroad_years(draft)
    {
        let entry = draft.income.iter().find(|entry| entry.id == id).unwrap();
        add(
            Some(id),
            "RRB_EARLIER_YEAR_ALLOCATION_NEEDED",
            TaxSection::Retirement,
            followup_document_name(draft, entry),
        );
    }
    if draft.adjustments.as_ref().is_some_and(|a| !a.social_election.printed_years_complete(draft))
        && crate::social_documents::unallocated_railroad_years(draft).is_none()
    {
        add(
            None,
            "DOCUMENT_SOCIAL_PRIOR_YEAR_FACTS",
            TaxSection::Retirement,
            status("SECTION_RETIREMENT"),
        );
    }
    if let Some((id, issue)) = retirement_aggregate_issue(draft) {
        let document =
            id.and_then(|id| draft.income.iter().find(|entry| entry.id == id)).map_or_else(
                || status("SECTION_RETIREMENT"),
                |entry| followup_document_name(draft, entry),
            );
        add(id, issue.key, TaxSection::Retirement, document);
    }
    if let Some((id, issue)) = estate_answer_issue(draft)
        && let Some(entry) = draft.income.iter().find(|entry| entry.id == id)
    {
        add(Some(id), issue.key, TaxSection::PassThrough, followup_document_name(draft, entry));
    }
    if let Some((id, issue)) = crate::cooperative::graph_issue(draft)
        && let Some(entry) = draft.income.iter().find(|entry| entry.id == id)
    {
        add(Some(id), issue.key, TaxSection::Farming, followup_document_name(draft, entry));
    }
    let mut checked_k1 = alloc::collections::BTreeSet::new();
    for entry in &draft.income {
        let Some(item) = document_item(entry, &[], crate::federal_rules::separate_return(draft))
        else {
            continue;
        };
        if let Some(issue) = recorded_income_owner_issue(draft, entry) {
            add(
                Some(entry.id),
                issue.key,
                topic_of(draft, entry).unwrap(),
                followup_document_name(draft, entry),
            );
        }
        if let Some(issue) =
            debt_followups::answer_issue(entry, u32::try_from(draft.tax_year).unwrap_or(0), true)
        {
            add(
                Some(entry.id),
                issue.key,
                TaxSection::CanceledDebt,
                followup_document_name(draft, entry),
            );
        }
        if let Some(issue) = special_retirement_answer_issue(&entry.income) {
            add(
                Some(entry.id),
                issue.key,
                TaxSection::Retirement,
                followup_document_name(draft, entry),
            );
        }
        if let FederalIncome::Retirement { data } = &entry.income
            && let Some(issue) = retirement_answer_issue(data)
        {
            add(
                Some(entry.id),
                issue.key,
                TaxSection::Retirement,
                followup_document_name(draft, entry),
            );
        }
        if let FederalIncome::Retirement { data: crate::RetirementSource::Rrb1099R(value) } =
            &entry.income
            && let Some(issue) = rrb_issue(value)
        {
            add(
                Some(entry.id),
                issue.key,
                TaxSection::Retirement,
                followup_document_name(draft, entry),
            );
        }
        if let FederalIncome::Cooperative { data } = &entry.income
            && let Some(issue) = crate::cooperative::input_issue(&crate::Patr1099::from(data))
        {
            add(
                Some(entry.id),
                issue.key,
                TaxSection::Farming,
                followup_document_name(draft, entry),
            );
        }
        if let FederalIncome::UndistributedGain { data } = &entry.income
            && crate::UndistributedGain::from(data).small_business != 0
            && data.section_1202_exclusion.is_none()
        {
            add(
                Some(entry.id),
                "FORM_2439_SECTION_1202_NEEDS_EXCLUSION",
                TaxSection::InterestDividends,
                followup_document_name(draft, entry),
            );
        }
        if let Some((_, issue)) = investment_answer_conflict(&item.document, Some(&entry.income)) {
            add(
                Some(entry.id),
                issue.key,
                TaxSection::InterestDividends,
                followup_document_name(draft, entry),
            );
        }
        if let FederalIncome::LtcBenefits { data } = &entry.income
            && let Some(issue) = data.record().character_issue(draft, entry.owner.owned())
        {
            add(
                Some(entry.id),
                issue.key,
                followup_topic(draft, entry).unwrap(),
                followup_document_name(draft, entry),
            );
        }
        if let FederalIncome::Able { data } = &entry.income
            && let Some(issue) = crate::AbleDistribution::from(data).answer_issue()
        {
            add(
                Some(entry.id),
                issue.key,
                TaxSection::OtherIncome,
                followup_document_name(draft, entry),
            );
        }
        if let FederalIncome::EducationDistribution { data } = &entry.income
            && let Some(issue) = crate::FederalEducationDistribution::from(data).answer_issue()
        {
            add(
                Some(entry.id),
                issue.key,
                followup_topic(draft, entry).unwrap(),
                followup_document_name(draft, entry),
            );
        }
        if let FederalIncome::MedicalDistribution { data } = &entry.income
            && let Some(issue) = crate::FederalMedicalDistribution::from(data).answer_issue()
        {
            add(
                Some(entry.id),
                issue.key,
                followup_topic(draft, entry).unwrap(),
                followup_document_name(draft, entry),
            );
        }
        if let FederalIncome::K1099 { data } = &entry.income
            && let Some(issue) = crate::information_returns::k_issue(&crate::Form1099K::from(data))
        {
            add(
                Some(entry.id),
                issue.key,
                TaxSection::SelfEmployment,
                followup_document_name(draft, entry),
            );
        }
        if let FederalIncome::Misc1099 { data } = &entry.income
            && let Some(issue) =
                crate::information_returns::misc_issue(&crate::Form1099Misc::from(data))
        {
            add(
                Some(entry.id),
                issue.key,
                topic_of(draft, entry).unwrap(),
                followup_document_name(draft, entry),
            );
        }
        if let Some(issue) = wage_answer_issue(entry) {
            add(Some(entry.id), issue.key, TaxSection::Wages, followup_document_name(draft, entry));
        }
    }
    for (id, row) in Interview::document_followup_rows(draft, None, Some(result)) {
        if matches!(&row.answer, A::IsoExerciseFacts { value: None }) {
            add(
                Some(id),
                "DOCUMENT_ANSWER_ISOEXERCISEFACTS",
                TaxSection::InvestmentSales,
                super::document_owners::document_name(draft, id)
                    .unwrap_or_else(|| status("SAVING_KIND_FORM_3921_ONE")),
            );
            continue;
        }
        let Some(entry) = draft.income.iter().find(|entry| entry.id == id) else {
            continue;
        };
        let Some(section) = answer_topic(draft, entry, &row.answer) else {
            continue;
        };
        let mut missing = Vec::new();
        let mut paper_fact = None;
        if !matches!(row.answer, A::Management { .. })
            && checked_k1.insert(id)
            && let Some(issue) = required_k1_question_issue(draft, entry)
        {
            // A printed code the App does not calculate is a paper fact: the
            // topic shows the K-1 read-only, so the item opens the Document Center row.
            if issue.key == "PARTNERSHIP_SOURCE_CODE_NOT_CALCULATED" {
                paper_fact = Some(issue.key);
            } else {
                missing.push(issue.key);
            }
        }
        missing.extend(unanswered_question_keys(draft, entry, &row));
        let prefix = alloc::format!("income/{id}/");
        let document =
            followup_question_document(draft, entry, row.key.strip_prefix(&prefix).unwrap_or(""));
        if let Some(key) = paper_fact {
            add(Some(id), key, TaxSection::Documents, document.clone());
        }
        for key in missing {
            add(Some(id), key, section, document.clone());
        }
    }
    if let Some(credits) = &draft.credits {
        let allocation_issue =
            result.issues.iter().any(|issue| issue.display.key == "ISSUE_BOND_ALLOCATION_SUMMARY");
        let source_issue = crate::tax_credit_bonds::source_document_issue(draft);
        for bond in &credits.tax_credit_bonds {
            let key = if bond.resolved_answers().is_none() {
                Some("DOCUMENT_TAX_CREDIT_BOND_ANSWERS")
            } else if let Some(issue) = bond.issue() {
                Some(issue.key)
            } else if bond
                .resolved_answers()
                .is_some_and(|answers| !answers.kind.issue_date_allowed(answers.issued_on))
            {
                Some(
                    "ISSUE_FORM_8912_REQUIRES_A_PRE_2018_QUALIFIED_TAX_CREDIT_OR_HOLDER_CREDIT_57C707E1",
                )
            } else if let Some((source, issue)) = &source_issue
                && source.is_none_or(|id| id == bond.id)
            {
                Some(issue.key)
            } else if allocation_issue {
                Some("DOCUMENT_TAX_CREDIT_BOND_ANSWERS")
            } else {
                None
            };
            if let Some(key) = key {
                add(
                    Some(bond.id),
                    key,
                    TaxSection::MoreCredits,
                    Message::new(
                        "DOCUMENT_FROM",
                        alloc::vec![
                            MessageArg::Encoded(status("FORM_FORM1097BTC")),
                            MessageArg::Text(bond.form.issuer.name.clone()),
                        ],
                    )
                    .encode(),
                );
            }
        }
        if let Some(source) = &credits.bba_audit_interest
            && let Err((key, _)) = source.refigure()
        {
            add(
                Some(source.id),
                key,
                // The furnishing date is printed (Part II G): corrected on the paper.
                if source.document.issue().is_some() || key == "ISSUE_BBA_AUDIT_FURNISHED_YEAR" {
                    TaxSection::Documents
                } else {
                    TaxSection::MoreCredits
                },
                Message::new(
                    "DOCUMENT_FROM",
                    alloc::vec![
                        MessageArg::Encoded(status("FORM_FORM8986")),
                        MessageArg::Text(source.partnership.name.clone()),
                    ],
                )
                .encode(),
            );
        }
        if let Some(facts) = crate::marketplace::facts(&credits.marketplace) {
            for policy in &facts.forms {
                if crate::marketplace::policy_answer_issue(facts, draft, policy).is_some() {
                    add(
                        Some(policy.id),
                        "DOCUMENT_MARKETPLACE_ANSWERS",
                        TaxSection::Marketplace,
                        pending_document_name(&policy.form.issuer),
                    );
                }
            }
        }
        let people = education_people(draft);
        for (student, key) in credits.education_pending() {
            let document = people
                .iter()
                .find(|person| person.id == student)
                .map(|person| person.label.clone())
                .unwrap_or_else(|| {
                    Message::new("DOCUMENT_SOURCE_NUMBER", alloc::vec![MessageArg::Number(student)])
                        .encode()
                });
            add(Some(student), key, TaxSection::Education, document);
        }
    }
    if let Some(adjustments) = &draft.adjustments
        && adjustments.student_loan_employer_paid() > adjustments.student_loan_interest()
    {
        add(
            None,
            "STUDENT_LOAN_EMPLOYER_ABOVE_INTEREST",
            TaxSection::StudentLoan,
            section_message(TaxSection::StudentLoan).encode(),
        );
    }
    if let Some(deductions) = &draft.deductions {
        for loan in &deductions.vehicle_loans {
            let document = pending_document_name(&loan.model().lender.name);
            if loan.business > loan.model().interest {
                add(
                    Some(loan.id),
                    "VEHICLE_BUSINESS_ABOVE_INTEREST",
                    TaxSection::Deductions,
                    document.clone(),
                );
            }
            if loan.model().refund > 0 {
                let origin = deductions
                    .vehicle_recoveries
                    .iter()
                    .find(|row| row.id == loan.id)
                    .map(|row| &row.origin);
                if origin.is_none()
                    || matches!(origin, Some(crate::VehicleRefundOrigin::Deducted { prior }) if !prior.ordinary)
                {
                    add(
                        Some(loan.id),
                        "DOCUMENT_VEHICLE_REFUND_BENEFIT",
                        TaxSection::Deductions,
                        document,
                    );
                }
            }
        }
        for source in &deductions.mortgage {
            let document = mortgage_document_name(source);
            if let Some(issue) = source.conversion_issue() {
                add(Some(source.id), issue.key, TaxSection::Mortgage, document.clone());
            }
            if source.form.box_4 > 0 && source.refund_origin.is_none() {
                add(
                    Some(source.id),
                    "DOCUMENT_MORTGAGE_REFUND_ORIGIN",
                    TaxSection::Mortgage,
                    document,
                );
            }
        }
    }
    // A document no saved answer has covered asks its page's situations again.
    for topic in INCOME_TOPICS {
        if super::followup_page::situations_open(draft, topic) {
            let row = ReviewItem {
                field: Some(libertas::SchemaPath::root().field("situations").into()),
                item: Message::new("DOCUMENT_SITUATIONS_NEEDED", alloc::vec![]).encode(),
                status: status(if coming_up(draft, topic) {
                    "REVIEW_COMING_UP"
                } else {
                    "DOCUMENT_PENDING_FIX"
                }),
                section: topic,
            };
            pending.push((None, "DOCUMENT_SITUATIONS_NEEDED", row));
        }
    }
    pending
}

#[cfg(test)]
pub(crate) fn fixture_issue_path(
    source: &IncomeSource,
    accepted_source: &IncomeSource,
    source_id: i64,
    key: &str,
    local: &str,
    rows: &[DocumentQuestion],
) -> Option<String> {
    let keys: Vec<_> = rows.iter().map(|row| row.key.clone()).collect();
    let locate = |failure| keyed_followup_failure(failure, &keys).field.map(|field| field.square());
    let mut issue = ValidationIssue::new(alloc::boxed::Box::leak(key.into())).at_record();
    let mut index = None;
    for segment in local.trim_matches('/').split('/').rev().filter(|part| !part.is_empty()) {
        if let Ok(value) = segment.parse::<usize>() {
            index = Some(value);
        } else {
            let field = alloc::boxed::Box::leak(segment.into());
            issue = if let Some(index) = index.take() {
                issue.in_list(field, index)
            } else {
                issue.in_field(field)
            };
        }
    }
    let submitted: Vec<_> = rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            Some((row.key.split('/').nth(1)?.parse::<i64>().ok()?, row.key.clone(), index))
        })
        .collect::<Option<_>>()?;
    if let IncomeSource::Farming(source) = accepted_source {
        let mut native = crate::Patr1099::from(source);
        if issue.has_field_prefix(&["more", "credit_facts", "other"])
            || issue.has_field_prefix(&["credit_facts", "other"])
        {
            let key = alloc::format!("income/{source_id}/credit_records");
            let row = rows.iter().find(|row| row.key == key)?;
            let crate::DocumentAnswer::CooperativeCreditRecords { other, .. } = &row.answer else {
                return None;
            };
            native.more.credit_facts.other.clone_from(other);
        }
        if let Some(failure) =
            cooperative_followups::bind_issue(&native, issue.clone(), &submitted, source_id)
        {
            return locate(failure);
        }
    }
    if matches!(source, IncomeSource::CanceledDebt { .. }) {
        let issue = if issue.has_field_prefix(&["property", "property"]) {
            issue.map_field_prefix(&["property", "property"], &["property"])
        } else {
            issue
        };
        return debt_followups::bind_issue(
            source_id,
            issue,
            &submitted,
            matches!(
                source,
                IncomeSource::CanceledDebt {
                    value: crate::CanceledDebtDocument::Acquisition1099A(_)
                }
            ),
        )
        .and_then(locate);
    }
    if key == "K1_LIQUIDATION_NEEDS_FINAL" {
        issue = ValidationIssue::new("K1_LIQUIDATION_NEEDS_FINAL").at_field("disposition");
    }
    if let Some(failure) =
        k1_followups::bind_issue(accepted_source, issue.clone(), &submitted, source_id)
    {
        return locate(failure);
    }
    // The 1099-INT, -DIV and -OID answers sit beside the paper, in this
    // source's submitted questions; the conflict reads them from there.
    let answered = fixture_investment_income(source, source_id, rows);
    if let Some((_, conflict)) = investment_answer_conflict(source, answered.as_ref()) {
        issue = conflict;
    }
    let (suffix, from, to) = question_field(&issue)?;
    let target = alloc::format!("income/{source_id}/{suffix}");
    let index = rows.iter().position(|row| row.key == target)?;
    locate(issue.map_field_prefix(from, to).in_field("answer").in_list("followups", index).into())
}

// The accepted source a fixture's paper and its submitted answers make.
#[cfg(test)]
fn fixture_investment_income(
    paper: &IncomeSource,
    source_id: i64,
    rows: &[DocumentQuestion],
) -> Option<FederalIncome> {
    let prefix = alloc::format!("income/{source_id}/");
    let answers = rows.iter().filter(|row| row.key.starts_with(&prefix)).map(|row| &row.answer);
    let IncomeSource::InterestDividend { value } = paper else { return None };
    Some(match value {
        crate::InterestDividendDocument::Int1099(form) => {
            let mut data = crate::Form1099IntSource::printed(form.clone());
            for answer in answers {
                match answer.clone() {
                    A::InterestNominee { value } => data.nominee = value,
                    A::InterestAfterDeath { value } => data.after_death = value,
                    A::InterestAccrued { value } => data.accrued_interest = value,
                    A::InterestTreatment { value } => data.special = value,
                    A::InterestAdjustment { value } => data.adjustments = value,
                    _ => {}
                }
            }
            FederalIncome::Interest { data: crate::InterestSource::Int1099(data) }
        }
        crate::InterestDividendDocument::Div1099(form) => {
            let mut data = crate::DividendSource::printed(form.clone());
            for answer in answers {
                match answer.clone() {
                    A::DividendNominee { value } => data.nominee = value,
                    A::DividendEligibility { value } => data.reit_eligible = value,
                    A::DividendAfterDeath { value } => data.after_death = value,
                    A::DividendFundExclusion { value } => data.section_1202_exclusion = value,
                    A::DividendCapitalReturns { value } => data.capital_returns = value,
                    A::DividendTreatment { value } => data.special = value,
                    _ => {}
                }
            }
            FederalIncome::Dividend { data }
        }
        crate::InterestDividendDocument::Oid1099(form) => {
            let mut data = crate::Form1099OidSource::printed(form.clone());
            for answer in answers {
                match answer.clone() {
                    A::OidNominee { value } => data.nominee = value,
                    A::OidPremium { value } => data.tax_exempt_bond_premium = value,
                    A::OidPrivateActivity { value } => data.private_activity = value,
                    A::OidTreatment { value } => data.special = value,
                    A::InterestAdjustment { value } => data.adjustments = value,
                    _ => {}
                }
            }
            FederalIncome::Oid { data }
        }
        _ => return None,
    })
}

// The date of birth of the person a document belongs to, when the people page holds it.
fn owner_birth(draft: &FederalDraft, owner: crate::DocumentOwner) -> Option<u32> {
    draft.people.as_ref()?.person(owner.tax_owner()?).map(|person| person.birth_date)
}

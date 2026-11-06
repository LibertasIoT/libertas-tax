use super::*;
use alloc::{string::ToString, vec};

pub(crate) fn center_document_kinds() -> Vec<i32> {
    vec![
        libertas_macros::variant_index!(crate::IncomeSourceKind::W2) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Form8919) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Int1099) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Div1099) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Oid1099) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Form2439) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::SavingsBonds) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Form8818) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Form1099B) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Form1099DA) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Sale1099B) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Contracts1099B) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Form1099S) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Form3921) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::IsoShareSale) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::EmployeeShareSale) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::EsppShareSale) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::NuaShareSale) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::QsbsSale) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::OpportunityFund) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::R1099) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Rrb1099R) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Ssa1099) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Rrb1099) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Form3922) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Form1099Nec) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Form1099K) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Form1099G) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Form1099Misc) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Q1099) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Sa1099) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::HsaTrusteeStatement) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Ltc1099) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Qa1099) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::WildfireRelief) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::DisasterRelief) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::K1Partnership) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::K1PartnershipActivities) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::K1Scorp) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::K1ScorpActivities) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::K1EstateTrust) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Cancel1099C) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Acquisition1099A) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::Patr1099) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::W2g) as i32,
    ]
}

fn vehicle_loan_documents_issue(documents: &[crate::Form1098VliDocument]) -> Option<SaveFailure> {
    documents
        .iter()
        .any(|v| v.form.lender.name.trim().is_empty())
        .then(|| status("MESSAGE_NAME_EACH_VEHICLE_LOAN_S_LENDER_UP_TO_80_CHARACTERS").into())
}

// A paper still in ordinary initialization has nothing printed entered: the shape an unfilled
// hand entry (such as the one after a failed import) takes. Every initialized field encodes as
// a zero byte (zero, false, empty text or list, no value, the first choice), so one check serves
// every paper kind: past the variant tag, the encoding is all zero bytes. A paper whose
// initialization is not zero is never mistaken for blank. The Document Center refuses to save a
// blank paper until it is filled or deleted.
fn nothing_entered(paper: &impl libertas::AvroEncode, tagged: bool) -> bool {
    let mut bytes = Vec::new();
    paper.avro_encode(&mut bytes);
    bytes.iter().skip(usize::from(tagged)).all(|byte| *byte == 0)
}

// The refusal for a blank paper, anchored to its item; the caller adds the group path.
fn blank_paper_issue(
    paper: &impl libertas::AvroEncode,
    tagged: bool,
    field: &'static str,
    index: usize,
    list: &'static str,
) -> Option<ValidationIssue> {
    nothing_entered(paper, tagged).then(|| {
        ValidationIssue::new("ISSUE_PAPER_NOTHING_ENTERED")
            .at_record()
            .in_field(field)
            .in_list(list, index)
    })
}

fn credit_document_scope_issue(documents: &[crate::CreditDocument]) -> Option<SaveFailure> {
    documents
        .iter()
        .enumerate()
        .filter(|(_, document)| matches!(document, crate::CreditDocument::PartnershipAudit(_)))
        .nth(1)
        .map(|(index, _)| {
            ValidationIssue::new("ISSUE_BBA_AUDIT_SCOPE")
                .at_record()
                .in_list("documents", index)
                .in_field("credits")
                .in_field("other")
                .in_field("documents")
                .into()
        })
}

impl Interview {
    fn document_center_value(
        &self,
        rows: Vec<IncomeDocumentItem>,
    ) -> Option<crate::DocumentCenter> {
        let d = &self.draft;
        let people = education_people(d);
        let mut value = crate::DocumentCenter {
            common: crate::DocumentsYouMayHave {
                health_accounts: crate::HealthContributionDocuments {
                    documents: crate::contribution_documents::stored(d).to_vec(),
                },
                wages: crate::WageDocuments { documents: Vec::new() },
                interest_dividends: crate::InterestDividendDocuments { documents: Vec::new() },
                investment_sales: crate::InvestmentSaleDocuments { documents: Vec::new() },
                retirement: crate::RetirementDocuments { documents: Vec::new() },
                self_employment: crate::SelfEmploymentDocuments { documents: Vec::new() },
                government_payments: crate::GovernmentPaymentDocuments { documents: Vec::new() },
                gambling: crate::GamblingDocuments { documents: Vec::new() },
                k1: crate::K1Documents { documents: Vec::new() },
                mortgages: crate::MortgageDocuments {
                    documents: d
                        .deductions
                        .as_ref()?
                        .mortgage
                        .iter()
                        .map(|source| crate::Form1098Document {
                            id: source.id,
                            form: source.form.clone(),
                            lender: source.lender.clone(),
                        })
                        .collect(),
                },
                student_loans: crate::StudentLoanDocuments {
                    documents: d
                        .adjustments
                        .as_ref()?
                        .student_loan
                        .forms
                        .iter()
                        .map(|source| crate::StudentLoanDocument {
                            id: source.id,
                            form: source.document.clone(),
                        })
                        .collect(),
                },
                education: EducationCredits {
                    students: d
                        .credits
                        .as_ref()?
                        .education
                        .iter()
                        .map(student_credit_value)
                        .collect(),
                    people,
                },
                marketplace: crate::MarketplaceDocuments {
                    documents: marketplace_value(d)
                        .forms
                        .into_iter()
                        .map(|view| crate::Form1095ADocument { id: view.id, value: view.value })
                        .collect(),
                },
            },
            other: crate::OtherDocumentTypes {
                farming: crate::FarmingDocuments { documents: Vec::new() },
                other_income: crate::OtherIncomeDocuments { documents: Vec::new() },
                canceled_debt: crate::CanceledDebtDocuments { documents: Vec::new() },
                vehicle_loans: crate::VehicleLoanDocuments {
                    documents: d
                        .deductions
                        .as_ref()?
                        .vehicle_loans
                        .iter()
                        .map(|source| crate::Form1098VliDocument {
                            id: source.id,
                            form: source.document.clone(),
                        })
                        .collect(),
                },
                vehicle_gifts: crate::VehicleGiftDocuments {
                    documents: d.deductions.as_ref()?.vehicle_gift_documents.clone(),
                },
                credits: crate::CreditDocuments {
                    documents: d
                        .credits
                        .as_ref()?
                        .tax_credit_bonds
                        .iter()
                        .cloned()
                        .map(|v| {
                            crate::CreditDocument::CreditBond(alloc::boxed::Box::new(
                                crate::TaxCreditBondDocument::from(v),
                            ))
                        })
                        .chain(d.credits.as_ref()?.bba_audit_interest.iter().cloned().map(
                            |source| {
                                crate::CreditDocument::PartnershipAudit(alloc::boxed::Box::new(
                                    crate::PartnershipAuditRow {
                                        id: source.id,
                                        document: source.document,
                                    },
                                ))
                            },
                        ))
                        .collect(),
                },
            },
        };
        for row in rows {
            let IncomeDocumentItem {
                id,
                owner,
                document,
                community,
                refund_management,
                allowed_community,
                preview,
                investment_caps,
                refund_management_needed,
                ..
            } = row;
            match document {
                IncomeSource::Wage { value: document } => {
                    value.common.wages.documents.push(crate::WageDocumentRow {
                        id,
                        owner,
                        document,
                        community,
                        refund_management,
                        allowed_community,
                        preview,
                        investment_caps,
                        refund_management_needed,
                    })
                }
                IncomeSource::InterestDividend { value: document } => {
                    value.common.interest_dividends.documents.push(
                        crate::InterestDividendDocumentRow {
                            id,
                            owner,
                            document,
                            community,
                            refund_management,
                            allowed_community,
                            preview,
                            investment_caps,
                            refund_management_needed,
                        },
                    )
                }
                IncomeSource::InvestmentSale { value: document } => {
                    value.common.investment_sales.documents.push(crate::InvestmentSaleDocumentRow {
                        id,
                        owner,
                        document,
                        community,
                        refund_management,
                        allowed_community,
                        preview,
                        investment_caps,
                        refund_management_needed,
                    })
                }
                IncomeSource::Retirement { value: document } => {
                    value.common.retirement.documents.push(crate::RetirementDocumentRow {
                        id,
                        owner,
                        document,
                        community,
                        refund_management,
                        allowed_community,
                        preview,
                        investment_caps,
                        refund_management_needed,
                    })
                }
                IncomeSource::SelfEmployment { value: document } => {
                    value.common.self_employment.documents.push(crate::SelfEmploymentDocumentRow {
                        id,
                        owner,
                        document,
                        community,
                        refund_management,
                        allowed_community,
                        preview,
                        investment_caps,
                        refund_management_needed,
                    })
                }
                IncomeSource::GovernmentPayment {
                    value: document,
                    refund_state,
                    refund_locality,
                } => value.common.government_payments.documents.push(
                    crate::GovernmentPaymentDocumentRow {
                        id,
                        owner,
                        document,
                        refund_state,
                        refund_locality,
                        community,
                        refund_management,
                        allowed_community,
                        preview,
                        investment_caps,
                        refund_management_needed,
                    },
                ),
                IncomeSource::Gambling(document) => {
                    value.common.gambling.documents.push(crate::GamblingDocumentRow {
                        id,
                        owner,
                        document,
                        community,
                        refund_management,
                        allowed_community,
                        preview,
                        investment_caps,
                        refund_management_needed,
                    })
                }
                IncomeSource::K1 { value: document } => {
                    value.common.k1.documents.push(crate::K1DocumentRow {
                        id,
                        owner,
                        document,
                        community,
                        refund_management,
                        allowed_community,
                        preview,
                        investment_caps,
                        refund_management_needed,
                    })
                }
                IncomeSource::Farming(document) => {
                    value.other.farming.documents.push(crate::FarmingDocumentRow {
                        id,
                        owner,
                        document,
                        community,
                        refund_management,
                        allowed_community,
                        preview,
                        investment_caps,
                        refund_management_needed,
                    })
                }
                IncomeSource::OtherIncome { value: document } => {
                    value.other.other_income.documents.push(crate::OtherIncomeDocumentRow {
                        id,
                        owner,
                        document,
                        community,
                        refund_management,
                        allowed_community,
                        preview,
                        investment_caps,
                        refund_management_needed,
                    })
                }
                IncomeSource::CanceledDebt { value: document } => {
                    value.other.canceled_debt.documents.push(crate::CanceledDebtDocumentRow {
                        id,
                        owner,
                        document,
                        community,
                        refund_management,
                        allowed_community,
                        preview,
                        investment_caps,
                        refund_management_needed,
                    })
                }
                IncomeSource::Record { .. } => return None,
            }
        }
        Some(value)
    }
    pub(super) fn credit_document_views(&self) -> Vec<crate::CreditDocumentView> {
        self.draft
            .credits
            .iter()
            .flat_map(|credits| {
                credits
                    .tax_credit_bonds
                    .iter()
                    .map(|bond| crate::CreditDocumentView::CreditBond {
                        value: crate::TaxCreditBondPage::from(bond),
                    })
                    .chain(credits.bba_audit_interest.iter().map(|source| {
                        crate::CreditDocumentView::PartnershipAudit {
                            id: source.id,
                            document: source.document.clone(),
                            answers: source.answers.clone(),
                        }
                    }))
            })
            .collect()
    }
    pub(super) fn credit_document_answers_candidate(
        &self,
        mut draft: FederalDraft,
        documents: &[crate::CreditDocumentView],
    ) -> Result<FederalDraft, SaveFailure> {
        let credits = draft
            .credits
            .as_mut()
            .ok_or_else(|| SaveFailure::from(status("PROBLEM_SECTION_UNAVAILABLE_YET")))?;
        let mut seen = alloc::collections::BTreeSet::new();
        for (index, document) in documents.iter().enumerate() {
            let changed = || {
                SaveFailure::from(
                    ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                        .at_record()
                        .in_list("documents", index),
                )
            };
            match document {
                crate::CreditDocumentView::CreditBond { value } => {
                    if !seen.insert(value.id) {
                        return Err(changed());
                    }
                    let source = credits
                        .tax_credit_bonds
                        .iter_mut()
                        .find(|source| source.id == value.id)
                        .ok_or_else(changed)?;
                    // The answers are the questions page's (credit_questions);
                    // this page shows them read-only.
                    if let Some(issue) = source.issue() {
                        return Err(issue.in_field("value").in_list("documents", index).into());
                    }
                }
                crate::CreditDocumentView::PartnershipAudit { id, .. } => {
                    if !seen.insert(*id) {
                        return Err(changed());
                    }
                    credits
                        .bba_audit_interest
                        .as_ref()
                        .filter(|source| source.id == *id)
                        .ok_or_else(changed)?;
                }
            }
        }
        if seen.len() != credits.tax_credit_bonds.len() + credits.bba_audit_interest.iter().count()
        {
            return Err(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                .at_field("documents")
                .into());
        }
        if let Some(issue) = crate::tax_credit_bonds::source_issue(&draft) {
            let paths: Vec<_> = draft
                .credits
                .as_ref()
                .unwrap()
                .tax_credit_bonds
                .iter()
                .map(|source| {
                    documents.iter().enumerate().find_map(|(index, document)| {
                        matches!(document, crate::CreditDocumentView::CreditBond { value } if value.id == source.id)
                            .then(|| ValidationIssue::new("")
                                .at_field("value")
                                .in_list("documents", index)
                                .field
                                .unwrap())
                    })
                })
                .collect();
            return Err(SaveFailure::from(issue)
                .map_submitted_items(&["tax_credit_bonds"], &paths)
                .map_field_prefix(&["tax_credit_bonds"], &["documents"]));
        }
        Ok(draft)
    }
    fn credit_document_sources_candidate(
        &self,
        documents: &[crate::CreditDocument],
    ) -> Result<FederalDraft, SaveFailure> {
        if let Some(failure) = credit_document_scope_issue(documents) {
            return Err(failure);
        }
        let mut draft = self.draft.clone();
        let mut bonds = Vec::new();
        let mut audit = None;
        for document in documents {
            match document {
                crate::CreditDocument::CreditBond(value) => {
                    let accepted = self
                        .draft
                        .credits
                        .iter()
                        .flat_map(|credits| &credits.tax_credit_bonds)
                        .find(|bond| value.id != 0 && bond.id == value.id);
                    bonds.push(value.record(
                        accepted,
                        accepted.map_or(crate::Owner::Taxpayer, |bond| bond.owner),
                    ));
                }
                crate::CreditDocument::PartnershipAudit(value) => {
                    // Validation runs before Save (credit_document_scope_issue and the raw
                    // document validators); here the accepted answers stay with the same paper.
                    let accepted = self
                        .draft
                        .credits
                        .iter()
                        .flat_map(|credits| credits.bba_audit_interest.iter())
                        .find(|source| value.id != 0 && source.id == value.id);
                    let mut source = accepted
                        .cloned()
                        .unwrap_or_else(|| crate::BbaAuditSource::from(value.document.clone()));
                    source.id = value.id;
                    source.document = value.document.clone();
                    audit = Some(source);
                }
            }
        }
        let credits = draft
            .credits
            .as_mut()
            .ok_or_else(|| SaveFailure::from(status("PROBLEM_SECTION_UNAVAILABLE_YET")))?;
        credits.tax_credit_bonds = bonds;
        credits.bba_audit_interest = audit;
        Ok(draft)
    }
    pub(super) fn topic_document_views(
        &self,
        topic: TaxSection,
    ) -> Option<crate::TopicDocumentViews> {
        let value = self.document_center_value(self.center_source_rows()?)?;
        Some(match topic {
            TaxSection::Wages => crate::TopicDocumentViews::Wages {
                documents: value
                    .common
                    .wages
                    .documents
                    .into_iter()
                    .map(|value| crate::WageDocumentView { value })
                    .collect(),
            },
            TaxSection::InterestDividends => crate::TopicDocumentViews::InterestDividends {
                documents: value
                    .common
                    .interest_dividends
                    .documents
                    .into_iter()
                    .map(|value| crate::InterestDividendDocumentView { value })
                    .collect(),
            },
            TaxSection::InvestmentSales => crate::TopicDocumentViews::InvestmentSales {
                documents: value
                    .common
                    .investment_sales
                    .documents
                    .into_iter()
                    .map(|value| crate::InvestmentSaleDocumentView { value })
                    .collect(),
            },
            TaxSection::Retirement => crate::TopicDocumentViews::Retirement {
                documents: value
                    .common
                    .retirement
                    .documents
                    .into_iter()
                    .map(|value| crate::RetirementDocumentView { value })
                    .collect(),
            },
            TaxSection::SelfEmployment => crate::TopicDocumentViews::SelfEmployment {
                documents: value
                    .common
                    .self_employment
                    .documents
                    .into_iter()
                    .map(|value| crate::SelfEmploymentDocumentView { value })
                    .collect(),
            },
            TaxSection::GovernmentBenefits => crate::TopicDocumentViews::GovernmentPayments {
                documents: value
                    .common
                    .government_payments
                    .documents
                    .into_iter()
                    .map(|value| crate::GovernmentPaymentDocumentView { value })
                    .collect(),
            },
            TaxSection::Gambling => crate::TopicDocumentViews::Gambling {
                documents: value
                    .common
                    .gambling
                    .documents
                    .into_iter()
                    .map(|value| crate::GamblingDocumentView { value })
                    .collect(),
            },
            TaxSection::PassThrough => crate::TopicDocumentViews::K1 {
                documents: value
                    .common
                    .k1
                    .documents
                    .into_iter()
                    .map(|value| crate::K1DocumentView { value })
                    .collect(),
            },
            TaxSection::Farming => crate::TopicDocumentViews::Farming {
                documents: value
                    .other
                    .farming
                    .documents
                    .into_iter()
                    .map(|value| crate::FarmingDocumentView { value })
                    .collect(),
            },
            TaxSection::OtherIncome => crate::TopicDocumentViews::OtherIncome {
                documents: value
                    .other
                    .other_income
                    .documents
                    .into_iter()
                    .map(|value| crate::OtherIncomeDocumentView { value })
                    .collect(),
            },
            TaxSection::CanceledDebt => crate::TopicDocumentViews::CanceledDebt {
                documents: value
                    .other
                    .canceled_debt
                    .documents
                    .into_iter()
                    .map(|value| crate::CanceledDebtDocumentView { value })
                    .collect(),
            },
            _ => return None,
        })
    }
    fn center_source_rows(&self) -> Option<Vec<IncomeDocumentItem>> {
        let mut rows = self
            .draft
            .income
            .iter()
            .filter(|entry| source_in_editor(&self.draft, entry, TaxSection::Documents))
            .map(|entry| {
                document_item(entry, &[], crate::federal_rules::separate_return(&self.draft))
            })
            .collect::<Option<Vec<_>>>()?;
        for item in &mut rows {
            if let IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(k1) } =
                &mut item.document
            {
                k1.estate_mortgages = self
                    .mortgage_choices()
                    .into_iter()
                    .map(|choice| crate::EstateSourceChoice { id: choice.id, label: choice.label })
                    .collect();
            }
        }

        Some(rows)
    }
    pub(super) fn document_center_page(&self, review: bool) -> Reply {
        let Reply::Edit(P::SaveIncomes {
            initial_interest_limit_choices: _,
            initial_intangible_predecessors: _,
            initial_intangible_choices: _,
            source_choices,
            initial_rental_recovery_choices,
            initial_asset_amt_carry_choices,
            qbi_sources,
            rental_energy_buildings,
            casualty_properties,
            empty_k1_activity_choices,
            allowed_injured_source_facts,
            allowed_injured_retained_owners,
            ltc_payee_sources,
            wage_scorporations,
            allowed_wage_scorporations,
            initial_k1_investment_choices,
            research_businesses,
            retained_research_choices,
            k1_credit_lines,
            research_credit_lines,
            renewable_credit_lines,
            transferable_credit_lines,
            user,
            allowed_continuing_risk,
            cookie,
            revision,
            page: _,
            previous: _,
            review,
            estimate,
            warnings,
            progress: _,
            topic,
            allowed_documents: _,
            allowed_owners,
            allowed_community,
            allowed_refund_management,
            foreign_countries: _,
            interest_statements: _,
            people,
            farms,
            cooperative_activities,
            cooperative_properties,
            businesses,
            qbi_businesses,
            rentals,
            mortgage_loans,
            royalty_properties,
            equipment_rentals,
            hobbies,
            ccc_lots,
            farm_codes: _,
            farm_limitations: _,
            farm_sources: _,
            basis_assets: _,
            farm_method_changes: _,
            inherited_iras,
            inherited_roth_iras,
            ltc_insured,
            annuity_sources,
            recovery_sources,
            allowed_business_source,
            allowed_farm_source,
            allowed_partner_source,
            allowed_home_spouse,
            allowed_other_parent,
            children,
            document_facts,
            documents: _,
            source_documents: _,
            initial_carbon_oxide_origins,
            initial_renewable_facility_origins,
            followups: _,
            questions: _,
            situations: _,
            allowed_situations: _,
            situation_answers: _,
            situation_documents: _,
            disaster_choices: _,
        }) = self.topic_page(TaxSection::Documents, review)
        else {
            return problem_key("PROBLEM_UNDISPLAYABLE_INCOME_RECORD");
        };
        let Some(mut source_rows) = self.center_source_rows() else {
            return problem_key("PROBLEM_UNDISPLAYABLE_INCOME_RECORD");
        };
        if let Some(rows) = data_answer_items(&self.draft, TaxSection::InvestmentSales) {
            source_rows.extend(
                rows.into_iter().filter(|row| !matches!(row.document, IncomeSource::Record { .. })),
            );
        }
        for row in &mut source_rows {
            row.allowed_community = document_community_answers(&self.draft, &row.document);
            let current = crate::community::index(row.community);
            if !row.allowed_community.contains(&current) {
                row.allowed_community.push(current);
            }
        }
        let Some(documents) = self.document_center_value(source_rows) else {
            return problem_key("PROBLEM_UNDISPLAYABLE_INCOME_RECORD");
        };
        let catalog = marketplace_catalog(&self.draft);
        let mut document_owners = self.document_owners(TaxSection::Documents);
        attach_document_edit_targets(&documents, &mut document_owners);
        Reply::Edit(P::SaveDocuments {
            document_owners,
            allowed_document_ownership: alloc::vec![libertas_macros::variant_index!(
                crate::AddedOwnership::Yours
            ) as i32],
            credit_income_sources: crate::tax_credit_bonds::income_sources(&self.draft),
            allowed_credit_owners: self.allowed_owners(),
            source_choices,
            initial_rental_recovery_choices,
            initial_asset_amt_carry_choices,
            qbi_sources,
            rental_energy_buildings,
            casualty_properties,
            empty_k1_activity_choices,
            allowed_injured_source_facts,
            allowed_injured_retained_owners,
            ltc_payee_sources,
            wage_scorporations,
            allowed_wage_scorporations,
            initial_k1_investment_choices,
            research_businesses,
            retained_research_choices,
            k1_credit_lines,
            research_credit_lines,
            renewable_credit_lines,
            transferable_credit_lines,
            user,
            allowed_continuing_risk,
            cookie,
            revision,
            page: DOCUMENTS_PAGE,
            previous: CHILDREN_PAGE,
            review,
            estimate,
            warnings,
            progress: Some(progress_text(DOCUMENTS_PAGE, &self.draft, None)),
            topic,
            allowed_documents: center_document_kinds(),
            allowed_owners,
            allowed_community,
            allowed_refund_management,
            foreign_countries: crate::foreign_accounts::countries(),
            interest_statements: crate::savings_bonds::source_choices(&self.draft.income),
            people,
            farms,
            cooperative_activities,
            cooperative_properties,
            businesses,
            qbi_businesses,
            rentals,
            mortgage_loans,
            royalty_properties,
            equipment_rentals,
            hobbies,
            ccc_lots,
            farm_codes: crate::farm_specials::code_choices(),
            farm_limitations: crate::farm_specials::limitation_choices(),
            farm_sources: crate::farm_specials::source_choices(&self.draft),
            basis_assets: crate::farm_specials::basis_choices(&self.draft),
            farm_method_changes: crate::farm_specials::method_choices(&self.draft),
            inherited_iras,
            inherited_roth_iras,
            ltc_insured,
            annuity_sources,
            recovery_sources,
            allowed_business_source,
            allowed_farm_source,
            allowed_partner_source,
            allowed_home_spouse,
            allowed_other_parent,
            children,
            document_facts,
            documents,
            initial_carbon_oxide_origins,
            initial_renewable_facility_origins,
            ptc_people: catalog.ptc_people,
            ptc_families: catalog.ptc_families,
            ptc_policies: catalog.ptc_policies,
            ptc_w2: catalog.ptc_w2,
            ptc_employees: catalog.ptc_employees,
            ptc_nonreturn_people: catalog.ptc_nonreturn_people,
            ptc_claimed_dependents: catalog.ptc_claimed_dependents,
            allowed_marriage: catalog.allowed_marriage,
            allowed_separate: allowed_separate(filing_status(&self.draft)),
            rental_mortgage_loans: crate::extended_rules::rental_mortgage_roots(&self.draft),
        })
    }
}

// Submitted rows use positions, including new documents without an accepted id.
// Review destinations share the same row shape in the editable projection.
fn center_row_path(
    group: &'static str,
    category: &'static str,
    list: &'static str,
    index: usize,
) -> crate::validation::FieldPath {
    ValidationIssue::new("")
        .at_list_item(list, index)
        .in_field(category)
        .in_field(group)
        .in_field("documents")
        .field
        .unwrap()
}

pub(super) fn accepted_document_row_path(d: &FederalDraft, id: i64) -> Option<String> {
    if id <= 0 {
        return None;
    }
    let positioned =
        |group, category, list, index| Some(center_row_path(group, category, list, index).square());
    let mut counts = alloc::collections::BTreeMap::new();
    for entry in d.income.iter().filter(|entry| source_in_editor(d, entry, TaxSection::Documents)) {
        let row = document_item(entry, &[], crate::federal_rules::separate_return(d))?;
        let (group, category) = match row.document {
            IncomeSource::Wage { .. } => ("common", "wages"),
            IncomeSource::InterestDividend { .. } => ("common", "interest_dividends"),
            IncomeSource::InvestmentSale { .. } => ("common", "investment_sales"),
            IncomeSource::Retirement { .. } => ("common", "retirement"),
            IncomeSource::SelfEmployment { .. } => ("common", "self_employment"),
            IncomeSource::GovernmentPayment { .. } => ("common", "government_payments"),
            IncomeSource::Gambling(_) => ("common", "gambling"),
            IncomeSource::K1 { .. } => ("common", "k1"),
            IncomeSource::Farming(_) => ("other", "farming"),
            IncomeSource::OtherIncome { .. } => ("other", "other_income"),
            IncomeSource::CanceledDebt { .. } => ("other", "canceled_debt"),
            IncomeSource::Record { .. } => continue,
        };
        let index = counts.entry((group, category)).or_insert(0);
        if entry.id == id {
            return positioned(group, category, "documents", *index);
        }
        *index += 1;
    }
    macro_rules! find {
        ($sources:expr, $group:literal, $category:literal, $list:literal) => {
            if let Some(index) = $sources.iter().position(|row| row.id == id) {
                return positioned($group, $category, $list, index);
            }
        };
    }
    if let Some(value) = &d.adjustments {
        find!(value.health_statements, "common", "health_accounts", "documents");
        find!(value.student_loan.forms, "common", "student_loans", "documents");
    }
    if let Some(value) = &d.deductions {
        find!(value.mortgage, "common", "mortgages", "documents");
        find!(value.vehicle_loans, "other", "vehicle_loans", "documents");
        find!(value.vehicle_gift_documents, "other", "vehicle_gifts", "documents");
    }
    if let Some(value) = &d.credits {
        find!(value.education, "common", "education", "students");
        if let Some(facts) = crate::marketplace::facts(&value.marketplace) {
            find!(facts.forms, "common", "marketplace", "documents");
        }
        // CreditDocument is an unkeyed Union list; its editable projection puts
        // Forms 1097 before Forms 8986, matching document_center_value.
        let index = value.tax_credit_bonds.iter().position(|row| row.id == id).or_else(|| {
            value
                .bba_audit_interest
                .iter()
                .position(|row| row.id == id)
                .map(|index| value.tax_credit_bonds.len() + index)
        });
        if let Some(index) = index {
            return Some(center_row_path("other", "credits", "documents", index).square());
        }
    }
    if let Some(value) = &d.screening {
        let mut extra = value
            .espp_acquisitions
            .iter()
            .map(|row| row.id)
            .chain(value.iso_exercises.iter().map(|row| row.id));
        if let Some(index) = extra.position(|source| source == id) {
            let before = counts.get(&("common", "investment_sales")).copied().unwrap_or(0);
            return positioned("common", "investment_sales", "documents", before + index);
        }
    }
    None
}

type CenterCandidate =
    (FederalDraft, Vec<(i64, usize, bool)>, Vec<Option<crate::validation::FieldPath>>);

pub(crate) struct PreparedDocumentCommit {
    pruned: Pruned,
    changed: bool,
    income_finished: bool,
}

impl Interview {
    pub(crate) fn candidate_editor(&self, draft: FederalDraft) -> Self {
        Self {
            user: self.user,
            draft,
            dirty: false,
            accepted_save: false,
            blocked: self.blocked,
            viewing: self.viewing,
        }
    }

    pub(super) fn document_center_admission(
        &self,
        value: &crate::DocumentCenter,
    ) -> Result<(), SaveFailure> {
        macro_rules! blank_papers {
            ($docs:expr, $field:ident, $list:literal, $group:literal, $category:literal, $tagged:expr) => {
                for (index, document) in $docs.iter().enumerate() {
                    if let Some(issue) = blank_paper_issue(
                        &document.$field,
                        $tagged,
                        stringify!($field),
                        index,
                        $list,
                    ) {
                        return Err(issue
                            .in_field($group)
                            .in_field($category)
                            .in_field("documents")
                            .into());
                    }
                }
            };
        }
        blank_papers!(
            value.common.mortgages.documents,
            form,
            "documents",
            "mortgages",
            "common",
            false
        );
        blank_papers!(
            value.common.student_loans.documents,
            form,
            "documents",
            "student_loans",
            "common",
            false
        );
        blank_papers!(
            value.common.education.students,
            form,
            "students",
            "education",
            "common",
            false
        );
        blank_papers!(
            value.common.marketplace.documents,
            value,
            "documents",
            "marketplace",
            "common",
            false
        );
        blank_papers!(
            value.other.vehicle_loans.documents,
            form,
            "documents",
            "vehicle_loans",
            "other",
            false
        );
        blank_papers!(
            value.other.vehicle_gifts.documents,
            form,
            "documents",
            "vehicle_gifts",
            "other",
            false
        );
        for (index, document) in value.other.credits.documents.iter().enumerate() {
            let blank = match document {
                crate::CreditDocument::CreditBond(row) => {
                    blank_paper_issue(&row.form, false, "form", index, "documents")
                }
                crate::CreditDocument::PartnershipAudit(row) => {
                    blank_paper_issue(&row.document, false, "document", index, "documents")
                }
            };
            if let Some(issue) = blank {
                return Err(issue
                    .in_field("credits")
                    .in_field("other")
                    .in_field("documents")
                    .into());
            }
        }
        for (index, document) in value.common.marketplace.documents.iter().enumerate() {
            if let Some(issue) = crate::marketplace_sources::paper_issue(&document.value) {
                return Err(issue
                    .in_field("value")
                    .in_list("documents", index)
                    .in_field("marketplace")
                    .in_field("common")
                    .in_field("documents")
                    .into());
            }
        }
        if let Some(failure) = vehicle_loan_documents_issue(&value.other.vehicle_loans.documents) {
            return Err(failure);
        }
        for (index, document) in value.other.vehicle_gifts.documents.iter().enumerate() {
            if let Some(issue) = document.form.entry_issue() {
                return Err(issue
                    .in_field("form")
                    .in_list("documents", index)
                    .in_field("vehicle_gifts")
                    .in_field("other")
                    .in_field("documents")
                    .into());
            }
        }
        if let Some(failure) = credit_document_scope_issue(&value.other.credits.documents) {
            return Err(failure);
        }
        for (index, document) in value.other.credits.documents.iter().enumerate() {
            if let crate::CreditDocument::PartnershipAudit(row) = document
                && let Some((key, _)) = row.document.issue()
            {
                return Err(ValidationIssue::new(key)
                    .at_record()
                    .in_list("documents", index)
                    .in_field("credits")
                    .in_field("other")
                    .in_field("documents")
                    .into());
            }
        }
        macro_rules! admit_documents {
            ($group:ident, $category:ident, $tagged:literal, $check:expr) => {
                for (index, row) in value.$group.$category.documents.iter().enumerate() {
                    let issue = if nothing_entered(&row.document, $tagged) {
                        Some(ValidationIssue::new("ISSUE_PAPER_NOTHING_ENTERED").at_record())
                    } else {
                        ($check)(&row.document)
                    };
                    if let Some(issue) = issue {
                        return Err(issue
                            .in_field("document")
                            .in_list("documents", index)
                            .in_field(stringify!($category))
                            .in_field(stringify!($group))
                            .in_field("documents")
                            .into());
                    }
                }
            };
        }
        admit_documents!(common, wages, true, |document: &crate::WageDocument| match document {
            crate::WageDocument::W2(form) => w2_input_issue(form),
            crate::WageDocument::Form8919(form) => form.issue(),
        });
        admit_documents!(
            common,
            interest_dividends,
            true,
            |document: &crate::InterestDividendDocument| match document {
                crate::InterestDividendDocument::Int1099(form) =>
                    crate::investment_statements::int_printed_issue(form)
                        .map(project_interest_issue),
                crate::InterestDividendDocument::Div1099(form) =>
                    crate::investment_statements::div_printed_issue(form)
                        .map(project_dividend_issue),
                crate::InterestDividendDocument::Form2439(form) => form.issue(),
                crate::InterestDividendDocument::SavingsBonds(form) =>
                    savings_bonds_entry_issue(form),
                crate::InterestDividendDocument::Form8818(form) =>
                    bond_redemption_paper_issue(form),
                _ => None,
            }
        );
        admit_documents!(
            common,
            investment_sales,
            true,
            |document: &crate::InvestmentSaleDocument| {
                match document {
                    crate::InvestmentSaleDocument::Form1099B(form) => {
                        crate::broker_document::printed_year_issue(
                            form.year,
                            i64::from(self.draft.tax_year),
                        )
                        .or_else(|| form.admission_issue())
                    }
                    crate::InvestmentSaleDocument::Form1099DA(form) => {
                        crate::broker_document::printed_year_issue(
                            form.year,
                            i64::from(self.draft.tax_year),
                        )
                        .or_else(|| form.admission_issue())
                    }
                    crate::InvestmentSaleDocument::Form1099S(form) => form.issue(),
                    crate::InvestmentSaleDocument::Form3921(form) => {
                        crate::extended_rules::form3921_paper_issue(form)
                    }
                    crate::InvestmentSaleDocument::Form3922(form) => form.issue(),
                    // Keep sale-year checks with their acquisition relationships so
                    // the sale validator preserves its first failing relationship.
                    crate::InvestmentSaleDocument::IsoShareSale(_)
                    | crate::InvestmentSaleDocument::EmployeeShareSale(_)
                    | crate::InvestmentSaleDocument::NuaShareSale(_) => None,
                    crate::InvestmentSaleDocument::EsppShareSale(form) => form.issue(),
                    crate::InvestmentSaleDocument::OpportunityFund(form) => form.printed_issue(),
                    _ => None,
                }
            }
        );
        admit_documents!(common, retirement, true, retirement_paper_issue);
        admit_documents!(
            common,
            self_employment,
            true,
            |document: &crate::SelfEmploymentDocument| {
                match document {
                    crate::SelfEmploymentDocument::Form1099Nec(form) => form.issue(),
                    crate::SelfEmploymentDocument::Form1099K(form) => form.issue(),
                }
            }
        );
        admit_documents!(common, government_payments, false, |form: &crate::Form1099GPaper| {
            if form.has_payments() {
                form.payments_admission_issue()
            } else {
                form.refund_admission_issue()
            }
        });
        admit_documents!(common, gambling, false, gambling_document_issue);
        admit_documents!(common, k1, true, |document: &crate::K1Document| match document {
            crate::K1Document::K1Partnership(form) => form.printed_admission_issue(),
            crate::K1Document::K1Scorp(form) => form.printed_admission_issue(),
            crate::K1Document::K1EstateTrust(form) => form.printed_admission_issue(),
            _ => None,
        });
        admit_documents!(other, farming, false, patr_paper_input_issue);
        admit_documents!(other, canceled_debt, true, |_: &crate::CanceledDebtDocument| None);
        admit_documents!(other, other_income, true, |document: &crate::OtherIncomeDocument| {
            match document {
                crate::OtherIncomeDocument::Form1099Misc(form) => form.admission_issue(),
                crate::OtherIncomeDocument::Q1099(form) => q1099_input_issue(form),
                crate::OtherIncomeDocument::Sa1099(form) => form.input_issue(),
                crate::OtherIncomeDocument::HsaTrusteeStatement(form) => form.input_issue(),
                crate::OtherIncomeDocument::Ltc1099(form) => form.intrinsic_issue(),
                crate::OtherIncomeDocument::Qa1099(form) => form.boxes_issue(),
                crate::OtherIncomeDocument::WildfireRelief(form) => form.printed_issue(),
                crate::OtherIncomeDocument::DisasterRelief(form) => form.printed_issue(),
            }
        });
        Ok(())
    }

    pub(super) fn save_document_center(
        &mut self,
        value: crate::DocumentCenter,
        owners: &[crate::PersonDocuments],
        review: bool,
    ) -> Reply {
        let prepared = self.prepare_document_center(value, owners);
        self.finish_document_center_save(prepared, review)
    }

    pub(crate) fn prepare_document_center(
        &mut self,
        value: crate::DocumentCenter,
        owners: &[crate::PersonDocuments],
    ) -> Result<PreparedDocumentCommit, SaveFailure> {
        let mut owned = self.draft.clone();
        document_owners::apply_section_view(
            &mut owned,
            TaxSection::Documents,
            owners,
            &self.income_owners(),
        )?;
        let candidate = self.candidate_editor(owned);
        let (mut draft, origins, paths) =
            candidate.document_center_candidate_with_origins(value)?;
        // Only an accepted Save makes a new paper's situations pending; the
        // running estimate of an unsaved Document center does not.
        super::followup_page::note_new_documents(&mut draft, &self.draft);
        let (draft, pruned) = self
            .income_draft_for_commit(draft, &origins, Some(TaxSection::Documents))
            .map_err(|failure| failure.map_submitted_items(&["documents", "documents"], &paths))?;
        let changed = draft.is_some();
        let income_finished =
            draft.as_ref().unwrap_or(&self.draft).finished_sections.contains(&TaxSection::Income);
        if let Some(draft) = draft {
            self.draft = draft;
            self.dirty = true;
        }
        self.accepted_save = true;
        Ok(PreparedDocumentCommit { pruned, changed, income_finished })
    }

    pub(crate) fn finish_document_center_save(
        &self,
        prepared: Result<PreparedDocumentCommit, SaveFailure>,
        review: bool,
    ) -> Reply {
        let prepared = match prepared {
            Ok(prepared) => prepared,
            Err(failure) => return refused(failure),
        };
        let mut calculation = self.draft.clone();
        finish_section(&mut calculation, TaxSection::Income);
        let result = alloc::boxed::Box::new(estimate(&calculation));
        self.income_save_reply(
            PreparedIncomeCommit {
                pruned: prepared.pruned,
                changed: prepared.changed,
                income_finished: prepared.income_finished,
                result,
            },
            DOCUMENTS_PAGE,
            review,
            Some(TaxSection::Documents),
        )
    }
    pub(super) fn document_center_candidate_with_origins(
        &self,
        mut value: crate::DocumentCenter,
    ) -> Result<CenterCandidate, SaveFailure> {
        let before = self
            .document_center_value(Vec::new())
            .ok_or_else(|| SaveFailure::from(status("PROBLEM_UNDISPLAYABLE_INCOME_RECORD")))?;
        for source in &mut value.common.health_accounts.documents {
            let accepted = crate::contribution_documents::stored(&self.draft)
                .iter()
                .find(|old| old.id != 0 && old.id == source.id);
            source.retain_assignment(accepted);
        }
        let marks = self.draft.finished_sections.clone();
        let mut candidate = self.candidate_editor(self.draft.clone());
        if value.common.health_accounts != before.common.health_accounts
            && let Some(adjustments) = candidate.draft.adjustments.as_mut()
        {
            adjustments.health_statements = value.common.health_accounts.documents.clone();
        }
        if value.common.mortgages != before.common.mortgages {
            let accepted =
                self.draft.deductions.as_ref().map(|d| d.mortgage.as_slice()).unwrap_or(&[]);
            let sources: Vec<crate::Mortgage1098Source> = value
                .common
                .mortgages
                .documents
                .iter()
                .map(|document| {
                    let mut source = crate::Mortgage1098Source::printed(document.form.clone());
                    source.id = document.id;
                    if let Some(old) =
                        accepted.iter().find(|old| document.id != 0 && old.id == document.id)
                    {
                        source.keep_answers(old);
                        if source.form.box_4 == 0 {
                            source.refund_origin = None;
                        }
                    }
                    source.lender = document.lender.clone();
                    source
                })
                .collect();
            candidate.draft = candidate.mortgage_sources_candidate(&sources).map_err(|e| {
                e.map_current_field_prefix(
                    &["value", "loans"],
                    &["documents", "common", "mortgages", "documents"],
                )
            })?;
        }
        if value.common.student_loans != before.common.student_loans {
            let accepted = &candidate.draft.adjustments.as_ref().unwrap().student_loan;
            candidate.draft = candidate
                .student_loan_sources_candidate(
                    &value.common.student_loans.documents,
                    accepted.student_loan_interest,
                    accepted.employer_paid_interest,
                )
                .map_err(|e| {
                    e.map_current_field_prefix(
                        &["value", "forms"],
                        &["documents", "common", "student_loans", "documents"],
                    )
                })?;
        }
        if value.common.education != before.common.education {
            candidate.draft =
                candidate.education_sources_candidate(&value.common.education).map_err(|e| {
                    e.map_current_field_prefix(&["value"], &["documents", "common", "education"])
                })?;
        }
        if value.common.marketplace != before.common.marketplace {
            let page = marketplace_value(&candidate.draft);
            candidate.draft = candidate
                .marketplace_sources_candidate(&page, &value.common.marketplace.documents)
                .map_err(|e| {
                    e.map_current_field_prefix(
                        &["value", "forms"],
                        &["documents", "common", "marketplace", "documents"],
                    )
                })?;
        }
        if value.other.vehicle_loans != before.other.vehicle_loans {
            if let Some(failure) =
                vehicle_loan_documents_issue(&value.other.vehicle_loans.documents)
            {
                return Err(failure);
            }
            if let Some(deductions) = candidate.draft.deductions.as_mut() {
                // The topic answer stays with the accepted source.
                let accepted = core::mem::take(&mut deductions.vehicle_loans);
                deductions.vehicle_loans = value
                    .other
                    .vehicle_loans
                    .documents
                    .iter()
                    .map(|document| {
                        let mut source =
                            crate::VehicleLoanSource::printed(document.id, document.form.clone());
                        if let Some(old) =
                            accepted.iter().find(|old| document.id != 0 && old.id == document.id)
                        {
                            source.business = old.business;
                        }
                        source
                    })
                    .collect();
            }
        }
        if value.other.vehicle_gifts != before.other.vehicle_gifts {
            for (index, document) in value.other.vehicle_gifts.documents.iter().enumerate() {
                if let Some(issue) = document.form.entry_issue() {
                    return Err(issue
                        .in_field("form")
                        .in_list("documents", index)
                        .in_field("vehicle_gifts")
                        .in_field("other")
                        .in_field("documents")
                        .into());
                }
            }
            if let Some(deductions) = candidate.draft.deductions.as_mut() {
                deductions.vehicle_gift_documents = value.other.vehicle_gifts.documents.clone();
                deductions.vehicle_gifts.retain(|gift| {
                    deductions
                        .vehicle_gift_documents
                        .iter()
                        .any(|document| document.id != 0 && document.id == gift.id)
                });
            }
        }
        if value.other.credits != before.other.credits {
            candidate.draft =
                candidate.credit_document_sources_candidate(&value.other.credits.documents)?;
        }
        let mut rows: Vec<(IncomeDocumentItem, crate::validation::FieldPath)> = Vec::new();
        rows.extend(value.common.wages.documents.into_iter().enumerate().map(|(index, row)| {
            (row.into_income(), center_row_path("common", "wages", "documents", index))
        }));
        rows.extend(value.common.interest_dividends.documents.into_iter().enumerate().map(
            |(index, row)| {
                (
                    row.into_income(),
                    center_row_path("common", "interest_dividends", "documents", index),
                )
            },
        ));
        rows.extend(value.common.investment_sales.documents.into_iter().enumerate().map(
            |(index, row)| {
                (
                    row.into_income(),
                    center_row_path("common", "investment_sales", "documents", index),
                )
            },
        ));
        rows.extend(value.common.retirement.documents.into_iter().enumerate().map(
            |(index, row)| {
                (row.into_income(), center_row_path("common", "retirement", "documents", index))
            },
        ));
        rows.extend(value.common.self_employment.documents.into_iter().enumerate().map(
            |(index, row)| {
                (
                    row.into_income(),
                    center_row_path("common", "self_employment", "documents", index),
                )
            },
        ));
        rows.extend(value.common.government_payments.documents.into_iter().enumerate().map(
            |(index, row)| {
                (
                    row.into_income(),
                    center_row_path("common", "government_payments", "documents", index),
                )
            },
        ));
        rows.extend(value.common.gambling.documents.into_iter().enumerate().map(|(index, row)| {
            (row.into_income(), center_row_path("common", "gambling", "documents", index))
        }));
        rows.extend(value.common.k1.documents.into_iter().enumerate().map(|(index, row)| {
            (row.into_income(), center_row_path("common", "k1", "documents", index))
        }));
        rows.extend(value.other.farming.documents.into_iter().enumerate().map(|(index, row)| {
            (row.into_income(), center_row_path("other", "farming", "documents", index))
        }));
        rows.extend(value.other.other_income.documents.into_iter().enumerate().map(
            |(index, row)| {
                (row.into_income(), center_row_path("other", "other_income", "documents", index))
            },
        ));
        rows.extend(value.other.canceled_debt.documents.into_iter().enumerate().map(
            |(index, row)| {
                (row.into_income(), center_row_path("other", "canceled_debt", "documents", index))
            },
        ));
        // The owner is the App's assignment, never an echoed value.
        for (row, _) in &mut rows {
            let names = row.document.printed_names().unwrap_or_default();
            let accepted =
                self.draft.income.iter().find(|source| row.id != 0 && source.id == row.id);
            row.owner = document_owners::entry_assignment(
                &self.draft,
                accepted.map(|source| source.printed_names().cloned().unwrap_or_default()).as_ref(),
                accepted.map(|source| source.owner),
                &names,
                row.document.recipient_domain(),
            )
            .owner
            .document_owner();
        }
        // The accepted source sequence owns ordering; category grouping is presentation.
        rows.sort_by_key(|(row, _)| {
            self.draft
                .income
                .iter()
                .position(|entry| row.id != 0 && entry.id == row.id)
                .unwrap_or(self.draft.income.len())
        });
        let (documents, paths): (Vec<_>, Vec<_>) = rows.into_iter().unzip();
        for (index, row) in documents.iter().enumerate() {
            if let IncomeSource::K1 { value: crate::K1Document::K1Partnership(document) } =
                &row.document
                && let Some(issue) = document.source().admission_issue()
            {
                return Err(issue.in_field("document").in_list("documents", index).into());
            }
        }
        let retirement = |income: &FederalIncome| match income {
            FederalIncome::Partnership { data } => Some(data.retirement.clone()),
            FederalIncome::PartnershipActivities { data } => data.retirement.clone(),
            _ => None,
        };
        if self.draft.income.iter().any(|entry| {
            if !source_in_editor(&self.draft, entry, TaxSection::Documents) {
                return false;
            }
            let health = crate::health::source_health_plan(&entry.income)
                .filter(|plan| matches!(plan, crate::BusinessHealthInsurance::Plan { .. }));
            let payment = crate::k1_activity::managed_payment(&entry.income).filter(|v| *v != 0);
            let plan = retirement(&entry.income).filter(|v| *v != crate::PartnerRetirement::None);
            if health.is_none() && payment.is_none() && plan.is_none() {
                return false;
            }
            let Some(row) = documents.iter().find(|row| row.id == entry.id) else {
                return true;
            };
            let proposed = document_entry(row.clone(), &self.draft.income);
            health.is_some_and(|value| {
                crate::health::source_health_plan(&proposed.income) != Some(value)
            }) || payment.is_some_and(|value| {
                crate::k1_activity::managed_payment(&proposed.income) != Some(value)
            }) || plan.is_some_and(|value| retirement(&proposed.income) != Some(value))
        }) {
            return Err(
                status("ISSUE_INTERVIEW_REVIEW_THE_ACCEPTED_PAGE_VALUES_BEFORE_65B6D7B7").into()
            );
        }
        let paths: Vec<_> = paths.into_iter().map(Some).collect();
        let supplied = IncomeEdits {
            qbi_aggregations: crate::qbi_aggregation::project_pages(
                candidate.draft.screening.as_ref().map_or(&[], |v| v.qbi_aggregations.as_slice()),
                &crate::qbi_aggregation::source_views(&candidate.draft, &candidate.draft),
            ),
            documents,
        };
        let espp = form3922_records(&supplied.documents, &self.draft)
            .map_err(|e| e.map_submitted_items(&["documents", "documents"], &paths))?;
        let iso = form3921_records(&candidate, &supplied.documents)
            .map_err(|e| e.map_submitted_items(&["documents", "documents"], &paths))?;
        if let Some(screening) = candidate.draft.screening.as_mut() {
            screening.iso_exercises = iso;
            screening.espp_acquisitions = espp;
        }
        let selected = vec![true; paths.len()];
        let mut origins = Vec::new();
        let mut draft = candidate
            .documents_candidate_inner_with_origins(
                TaxSection::Documents,
                supplied,
                false,
                Some(&mut origins),
                None,
            )
            .map_err(|e| {
                e.map_list_item_field_prefix(
                    &["documents", "documents"],
                    &["document", "value"],
                    &["document"],
                    &selected,
                )
                .map_submitted_items(&["documents", "documents"], &paths)
                .map_field_prefix(&["documents", "documents"], &["documents"])
            })?;
        document_owners::assign_native_document_ids(&mut draft, &self.draft)?;
        document_owners::match_documents(&mut draft, &self.draft);
        draft.finished_sections = marks;
        finish_section(&mut draft, TaxSection::Documents);
        Ok((draft, origins, paths))
    }
}

// Bind navigation to the actual editable projection, rather than to a stored
// source category: this is the value the current Request's editor holds.
fn attach_document_edit_targets(documents: &crate::DocumentCenter, people: &mut [PersonDocuments]) {
    let mut paths = alloc::collections::BTreeMap::new();
    macro_rules! rows {
        ($group:ident, $category:ident, $list:ident) => {
            for (index, row) in documents.$group.$category.$list.iter().enumerate() {
                paths.insert(
                    row.id,
                    libertas::SchemaPath::root()
                        .field("documents")
                        .field(stringify!($group))
                        .field(stringify!($category))
                        .field(stringify!($list))
                        .index(index)
                        .to_string(),
                );
            }
        };
    }
    rows!(common, health_accounts, documents);
    rows!(common, wages, documents);
    rows!(common, interest_dividends, documents);
    rows!(common, investment_sales, documents);
    rows!(common, retirement, documents);
    rows!(common, self_employment, documents);
    rows!(common, government_payments, documents);
    rows!(common, gambling, documents);
    rows!(common, k1, documents);
    rows!(common, mortgages, documents);
    rows!(common, student_loans, documents);
    rows!(common, education, students);
    rows!(common, marketplace, documents);
    rows!(other, farming, documents);
    rows!(other, other_income, documents);
    rows!(other, canceled_debt, documents);
    rows!(other, vehicle_loans, documents);
    rows!(other, vehicle_gifts, documents);
    for (index, row) in documents.other.credits.documents.iter().enumerate() {
        let id = match row {
            crate::CreditDocument::CreditBond(row) => row.id,
            crate::CreditDocument::PartnershipAudit(row) => row.id,
        };
        paths.insert(
            id,
            libertas::SchemaPath::root()
                .field("documents")
                .field("other")
                .field("credits")
                .field("documents")
                .index(index)
                .to_string(),
        );
    }
    for person in people {
        for row in &mut person.documents {
            if let Some(field) = paths.get(&row.document) {
                row.edit_document.push(crate::DocumentEditTarget {
                    action: status("DOCUMENT_CENTER_EDIT_DOCUMENT"),
                    field: Some(field.clone()),
                });
            }
        }
    }
}

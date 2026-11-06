use crate::tests::cross::interview::PAGE_INCOME;
use crate::tests::*;
use alloc::string::{String, ToString};

pub(in crate::tests) fn topic_for(document: &IncomeSource) -> TaxSection {
    match document {
        IncomeSource::Record { value: crate::IncomeRecord::Clergy(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::CashTipsWithoutW2(_) } => {
            TaxSection::Wages
        }
        IncomeSource::Record { value: crate::IncomeRecord::NonfarmOptionalMethod(_) } => {
            TaxSection::SelfEmployment
        }
        IncomeSource::Wage { value: crate::WageDocument::W2(_) }
        | IncomeSource::Wage { value: crate::WageDocument::Form8919(_) } => TaxSection::Wages,
        IncomeSource::Record { value: crate::IncomeRecord::PrivateLoanInterest(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::SellerMortgageInterest(_) }
        | IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Int1099(_) }
        | IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Div1099(_) }
        | IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Oid1099(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::Form8814(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::ForeignAccountOrTrust(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::TaxableBondPremium(_) }
        | IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Form2439(_) }
        | IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Form8818(_) }
        | IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::SavingsBonds(_),
        } => TaxSection::InterestDividends,
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Sale1099B(_) }
        | IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099B(_) }
        | IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099DA(_) }
        | IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::OpportunityFund(_),
        }
        | IncomeSource::Record { value: crate::IncomeRecord::OpportunityFundDeferral(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::CapitalLossCarryover(_) }
        | IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form3921(_) }
        | IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form3922(_) }
        | IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::IsoShareSale(_) }
        | IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::EmployeeShareSale(_),
        }
        | IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::EsppShareSale(_) }
        | IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::NuaShareSale(_) }
        | IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::QsbsSale(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::SmallBusinessStockLoss(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::WorthlessSecurity(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::BadDebt(_) }
        | IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099S(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::MainHomeSale { .. } }
        | IncomeSource::Record { value: crate::IncomeRecord::InstallmentPayments(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::InstallmentSale(_) }
        | IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::Contracts1099B(_),
        }
        | IncomeSource::Record { value: crate::IncomeRecord::Section1256Carryback(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::RelatedExchange(_) } => {
            TaxSection::InvestmentSales
        }
        IncomeSource::Retirement { value: crate::RetirementDocument::R1099(_) }
        | IncomeSource::Retirement { value: crate::RetirementDocument::Rrb1099R(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::InheritedIra(_) }
        | IncomeSource::Retirement { value: crate::RetirementDocument::Ssa1099(_) }
        | IncomeSource::Retirement { value: crate::RetirementDocument::Rrb1099(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::Form8606(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::Qcd(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::Form8915(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::SocialLumpSums(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::MissedRmd(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::RothHistory(_) } => {
            TaxSection::Retirement
        }
        IncomeSource::Record { value: crate::IncomeRecord::Business(_) }
        | IncomeSource::SelfEmployment { value: crate::SelfEmploymentDocument::Form1099Nec(_) }
        | IncomeSource::SelfEmployment { value: crate::SelfEmploymentDocument::Form1099K(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::QbiLossCarryforward(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::SpouseSection179(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::NetOperatingLoss(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::BusinessPropertyLosses(_) } => {
            TaxSection::SelfEmployment
        }
        IncomeSource::Record { value: crate::IncomeRecord::Farm(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::FarmRental(_) }
        | IncomeSource::Farming(_)
        | IncomeSource::Record { value: crate::IncomeRecord::FarmAveraging(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::FarmLoss(_) } => TaxSection::Farming,
        IncomeSource::Record { value: crate::IncomeRecord::Rental(_) } => TaxSection::Rentals,
        IncomeSource::GovernmentPayment { .. } => TaxSection::GovernmentBenefits,
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Form1099Misc(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::OtherIncome(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::Royalties(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::EquipmentRental(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::AlimonyReceived(_) }
        | IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Q1099(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::EducationAccountExpenses(_) }
        | IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Sa1099(_) }
        | IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::HsaTrusteeStatement(_) }
        | IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Ltc1099(_) }
        | IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Qa1099(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::AbleContributions(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::Scholarship(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::JuryPay(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::HobbySales(_) }
        | IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::WildfireRelief(_) }
        | IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::DisasterRelief(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::CasualtyLivingInsurance(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::DisasterMitigation(_) } => {
            TaxSection::OtherIncome
        }
        IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(_) } => {
            TaxSection::LongTermCareBenefits
        }
        IncomeSource::Gambling(_)
        | IncomeSource::Record { value: crate::IncomeRecord::GamblingLosses(_) } => {
            TaxSection::Gambling
        }
        IncomeSource::K1 { value: crate::K1Document::K1Partnership(_) }
        | IncomeSource::K1 { value: crate::K1Document::K1PartnershipActivities(_) }
        | IncomeSource::K1 { value: crate::K1Document::K1ScorpActivities(_) }
        | IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(_) }
        | IncomeSource::K1 { value: crate::K1Document::K1Scorp(_) } => TaxSection::PassThrough,
        IncomeSource::CanceledDebt { value: crate::CanceledDebtDocument::Cancel1099C(_) }
        | IncomeSource::CanceledDebt { value: crate::CanceledDebtDocument::Acquisition1099A(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::HealthLoanRepayment(_) } => {
            TaxSection::CanceledDebt
        }
        IncomeSource::Record { value: crate::IncomeRecord::Form2555(_) }
        | IncomeSource::Record { value: crate::IncomeRecord::ForeignPay(_) } => {
            TaxSection::WorkAbroad
        }
    }
}
// The document kinds an income topic's page offers, by variant name.
pub(in crate::tests) fn topic_kinds(topic: TaxSection) -> Vec<i32> {
    match topic {
        TaxSection::Wages => vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::CashTipsWithoutW2) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Clergy) as i32,
        ],
        TaxSection::InterestDividends => vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::PrivateLoanInterest) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::SellerMortgageInterest) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::ForeignAccountOrTrust) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::TaxableBondPremium) as i32,
        ],
        TaxSection::InvestmentSales => vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::MainHomeSale) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::CapitalLossCarryover) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Section1256Carryback) as i32,
        ],
        TaxSection::Retirement => vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form8606) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Qcd) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form8915) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::SocialLumpSums) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::MissedRmd) as i32,
        ],
        TaxSection::SelfEmployment => vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::NonfarmOptionalMethod) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Business) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::QbiLossCarryforward) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::NetOperatingLoss) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::BusinessPropertyLosses) as i32,
        ],
        TaxSection::Farming => vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::Farm) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::FarmRental) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::FarmAveraging) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::FarmLoss) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::QbiLossCarryforward) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::NetOperatingLoss) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::BusinessPropertyLosses) as i32,
        ],
        TaxSection::Rentals => vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::Royalties) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Rental) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::BusinessPropertyLosses) as i32,
        ],
        TaxSection::GovernmentBenefits => vec![],
        TaxSection::OtherIncome => vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::OtherIncome) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Royalties) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::EquipmentRental) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::AlimonyReceived) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::EducationAccountExpenses)
                as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::AbleContributions) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Scholarship) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::JuryPay) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::HobbySales) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::CasualtyLivingInsurance)
                as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::DisasterMitigation) as i32,
        ],
        TaxSection::LongTermCareBenefits => {
            vec![libertas_macros::variant_index!(crate::IncomeSourceKind::LtcInsured) as i32]
        }
        TaxSection::Gambling => {
            vec![libertas_macros::variant_index!(crate::IncomeSourceKind::GamblingLosses) as i32]
        }
        TaxSection::PassThrough => vec![],
        TaxSection::CanceledDebt => {
            vec![libertas_macros::variant_index!(crate::IncomeSourceKind::HealthLoanRepayment)
                as i32]
        }
        TaxSection::WorkAbroad => vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::Form2555) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::ForeignPay) as i32,
        ],
        _ => vec![],
    }
}
// Construct current record input from the actual topic editor. Native fixture rows
// are adapted only after the source/record owner is known; no retired wire type is kept.
pub(in crate::tests) fn save_topic(
    session: &mut Interview,
    topic: TaxSection,
    rows: Vec<IncomeDocumentItem>,
) -> P {
    let mut request = open_topic(session, topic);
    if let P::SaveWagesAnswers { records, .. } = &mut request {
        records.documents = rows
            .into_iter()
            .map(|row| {
                crate::IncomeRecordItem::from_income(row)
                    .and_then(crate::WagesRecordItem::from_income)
                    .expect("wage records use their topic; papers use the Document center")
            })
            .collect();
        return request;
    }
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    documents.documents = rows
        .into_iter()
        .map(|row| {
            crate::IncomeRecordItem::from_income(row)
                .expect("records use their topic; standalone documents use the Document center")
        })
        .collect();
    // These complete fixtures explicitly supply their election answer. Build
    // its presentation choices from the supplied property, as an opened page does.
    for item in &mut documents.documents {
        let retained = session
            .draft
            .income
            .iter()
            .find(|entry| entry.id == item.id)
            .is_some_and(|entry| crate::interest_limit::source(&entry.income).is_some());
        crate::interest_limit::refresh(&mut item.document, retained);
        // Complete source fixtures echo the questions an opened property row offers;
        // explicit unknown answers stay unknown and cannot admit passive gain.
        crate::farm_rental::refresh_property_questions(&mut item.document);
    }
    request
}
// Construct a source edit from the actual Document center. Categories represented
// by the supplied native fixtures are replaced; every other category stays accepted.
pub(in crate::tests) fn save_documents(
    session: &mut Interview,
    rows: Vec<IncomeDocumentItem>,
) -> P {
    let mut request = open_document_center(session);
    let P::SaveDocuments { documents, allowed_community: page_community, .. } = &mut request else {
        unreachable!()
    };
    if rows.is_empty() {
        documents.common.wages.documents.clear();
    }
    let mut cleared = [false; 11];
    for row in rows {
        let IncomeDocumentItem {
            id,
            owner,
            document,
            community,
            preview,
            refund_management,
            refund_management_needed,
            investment_caps,
            mut allowed_community,
        } = row;
        if allowed_community.is_empty() {
            allowed_community = page_community.clone();
        }
        match document {
            IncomeSource::Wage { value: document } => {
                if !cleared[0] {
                    documents.common.wages.documents.clear();
                    cleared[0] = true;
                }
                documents.common.wages.documents.push(crate::WageDocumentRow {
                    id,
                    owner,
                    document,
                    community,
                    preview,
                    refund_management,
                    refund_management_needed,
                    investment_caps,
                    allowed_community,
                });
            }
            IncomeSource::InterestDividend { value: document } => {
                if !cleared[1] {
                    documents.common.interest_dividends.documents.clear();
                    cleared[1] = true;
                }
                documents.common.interest_dividends.documents.push(
                    crate::InterestDividendDocumentRow {
                        id,
                        owner,
                        document,
                        community,
                        preview,
                        refund_management,
                        refund_management_needed,
                        investment_caps,
                        allowed_community,
                    },
                );
            }
            IncomeSource::InvestmentSale { value: document } => {
                if !cleared[2] {
                    documents.common.investment_sales.documents.clear();
                    cleared[2] = true;
                }
                documents.common.investment_sales.documents.push(
                    crate::InvestmentSaleDocumentRow {
                        id,
                        owner,
                        document,
                        community,
                        preview,
                        refund_management,
                        refund_management_needed,
                        investment_caps,
                        allowed_community,
                    },
                );
            }
            IncomeSource::Retirement { value: document } => {
                if !cleared[3] {
                    documents.common.retirement.documents.clear();
                    cleared[3] = true;
                }
                documents.common.retirement.documents.push(crate::RetirementDocumentRow {
                    id,
                    owner,
                    document,
                    community,
                    preview,
                    refund_management,
                    refund_management_needed,
                    investment_caps,
                    allowed_community,
                });
            }
            IncomeSource::SelfEmployment { value: document } => {
                if !cleared[4] {
                    documents.common.self_employment.documents.clear();
                    cleared[4] = true;
                }
                documents.common.self_employment.documents.push(crate::SelfEmploymentDocumentRow {
                    id,
                    owner,
                    document,
                    community,
                    preview,
                    refund_management,
                    refund_management_needed,
                    investment_caps,
                    allowed_community,
                });
            }
            IncomeSource::GovernmentPayment { value: document, refund_state, refund_locality } => {
                if !cleared[5] {
                    documents.common.government_payments.documents.clear();
                    cleared[5] = true;
                }
                documents.common.government_payments.documents.push(
                    crate::GovernmentPaymentDocumentRow {
                        id,
                        owner,
                        document,
                        refund_state,
                        refund_locality,
                        community,
                        preview,
                        refund_management,
                        refund_management_needed,
                        investment_caps,
                        allowed_community,
                    },
                );
            }
            IncomeSource::Gambling(document) => {
                if !cleared[6] {
                    documents.common.gambling.documents.clear();
                    cleared[6] = true;
                }
                documents.common.gambling.documents.push(crate::GamblingDocumentRow {
                    id,
                    owner,
                    document,
                    community,
                    preview,
                    refund_management,
                    refund_management_needed,
                    investment_caps,
                    allowed_community,
                });
            }
            IncomeSource::K1 { value: document } => {
                if !cleared[7] {
                    documents.common.k1.documents.clear();
                    cleared[7] = true;
                }
                documents.common.k1.documents.push(crate::K1DocumentRow {
                    id,
                    owner,
                    document,
                    community,
                    preview,
                    refund_management,
                    refund_management_needed,
                    investment_caps,
                    allowed_community,
                });
            }
            IncomeSource::Farming(document) => {
                if !cleared[8] {
                    documents.other.farming.documents.clear();
                    cleared[8] = true;
                }
                documents.other.farming.documents.push(crate::FarmingDocumentRow {
                    id,
                    owner,
                    document,
                    community,
                    preview,
                    refund_management,
                    refund_management_needed,
                    investment_caps,
                    allowed_community,
                });
            }
            IncomeSource::OtherIncome { value: document } => {
                if !cleared[9] {
                    documents.other.other_income.documents.clear();
                    cleared[9] = true;
                }
                documents.other.other_income.documents.push(crate::OtherIncomeDocumentRow {
                    id,
                    owner,
                    document,
                    community,
                    preview,
                    refund_management,
                    refund_management_needed,
                    investment_caps,
                    allowed_community,
                });
            }
            IncomeSource::CanceledDebt { value: document } => {
                if !cleared[10] {
                    documents.other.canceled_debt.documents.clear();
                    cleared[10] = true;
                }
                documents.other.canceled_debt.documents.push(crate::CanceledDebtDocumentRow {
                    id,
                    owner,
                    document,
                    community,
                    preview,
                    refund_management,
                    refund_management_needed,
                    investment_caps,
                    allowed_community,
                });
            }
            IncomeSource::Record { .. } => {
                panic!("topic records use save_topic or save_each_topic")
            }
        }
    }
    request = fixture_blank_owner_inputs(&session.draft, request);
    request
}

pub(in crate::tests) fn k1_fixture_answer_request(
    session: &mut Interview,
    rows: &[IncomeDocumentItem],
) -> Option<P> {
    fn name(value: &crate::K1Document) -> &str {
        use crate::K1Document::*;
        match value {
            K1Partnership(w) => &w.form.partnership.name,
            K1Scorp(w) => &w.form.corporation.name,
            K1PartnershipActivities(w) => &w.partnership.name,
            K1ScorpActivities(w) => &w.corporation.name,
            K1EstateTrust(w) => &w.form.estate_trust.name,
        }
    }
    let accepted_rows = page_documents(&open_documents(session));
    let mut used = Vec::new();
    let mut sources = Vec::new();
    for accepted in &accepted_rows {
        let IncomeSource::K1 { value: accepted_source } = &accepted.document else {
            continue;
        };
        let supplied = rows.iter().enumerate().find(|(index, supplied)| {
            if used.contains(index) {
                return false;
            }
            let IncomeSource::K1 { value } = &supplied.document else {
                return false;
            };
            if supplied.id > 0 {
                supplied.id == accepted.id
            } else {
                core::mem::discriminant(value) == core::mem::discriminant(accepted_source)
                    && name(value) == name(accepted_source)
            }
        });
        if let Some((index, supplied)) = supplied {
            used.push(index);
            let IncomeSource::K1 { value } = &supplied.document else { unreachable!() };
            sources.push((accepted.id, accepted_source, value, supplied));
        }
    }
    if sources.is_empty() {
        return None;
    }
    let mut request = open_topic(session, TaxSection::PassThrough);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    for row in followups {
        let id = row.key.split('/').nth(1).unwrap().parse::<i64>().unwrap();
        let Some((_, accepted, desired, supplied)) =
            sources.iter().find(|(source, _, _, _)| *source == id)
        else {
            continue;
        };
        match &mut row.answer {
            crate::DocumentAnswer::Community { value } => *value = supplied.community,
            crate::DocumentAnswer::Management { value } => *value = supplied.refund_management,
            _ => {}
        }
        fixture_k1_answer(session, row, accepted, desired);
    }
    refresh_k1_fixture_choices(session, &mut request);
    let calculated = send(session, super::estimate_documents_from_save(&request));
    if let Reply::Response(P::IncomeRunningEstimate { groups, sources, .. }) = calculated {
        let P::SaveIncomes { documents, qbi_sources, .. } = &mut request else { unreachable!() };
        for (row, derived) in documents.qbi_aggregations.iter_mut().zip(groups) {
            row.members = derived.members;
            row.disclosure_sources = derived.disclosure_sources;
            row.businesses = derived.businesses;
        }
        *qbi_sources = sources;
    }
    Some(request)
}

// A moved personal guard is exercised on its actual topic after the printed
// candidate is admitted in an independent session. Intrinsic guards stay on
// the original source request; no normal Send path supplies hidden answers.
extern crate std;
std::thread_local! {
    static K1_PAGE_LOCALS: core::cell::RefCell<Vec<(String, String)>> =
        const { core::cell::RefCell::new(Vec::new()) };
}

pub(in crate::tests) fn topic_fixture_control(
    session: &Interview,
    request: &P,
    key: &str,
    path: &str,
) -> Option<(Interview, P, String)> {
    if !matches!(request, P::SaveDocuments { .. }) {
        return None;
    }
    fn category(source: &IncomeSource) -> &str {
        match source {
            IncomeSource::Wage { .. } => "common/wages",
            IncomeSource::InterestDividend { .. } => "common/interest_dividends",
            IncomeSource::InvestmentSale { .. } => "common/investment_sales",
            IncomeSource::Retirement { .. } => "common/retirement",
            IncomeSource::SelfEmployment { .. } => "common/self_employment",
            IncomeSource::GovernmentPayment { .. } => "common/government_payments",
            IncomeSource::Gambling(_) => "common/gambling",
            IncomeSource::K1 { .. } => "common/k1",
            IncomeSource::Farming(_) => "other/farming",
            IncomeSource::OtherIncome { .. } => "other/other_income",
            IncomeSource::CanceledDebt { .. } => "other/canceled_debt",
            IncomeSource::Record { .. } => "",
        }
    }
    let source_path = path.trim_matches(['[', ']']).strip_prefix("documents/")?;
    let (group, tail) = source_path.rsplit_once("/documents/")?;
    let (index, local) = tail.split_once('/')?;
    let index = index.parse::<usize>().ok()?;
    let local = local.strip_prefix("document/").unwrap_or(local);
    let page_local = k1_page_local(path);
    let local = page_local.as_deref().unwrap_or(local);
    let desired = page_documents(request);
    let original = desired.iter().filter(|row| category(&row.document) == group).nth(index)?;
    let mut candidate = restart(session.draft.clone());
    let mut paper = request.clone();
    paper = fixture_blank_owner_inputs(&candidate.draft, paper);
    let P::SaveDocuments { cookie, revision, .. } = &mut paper else { unreachable!() };
    *cookie = candidate.draft.cookie.clone();
    *revision = candidate.draft.revision;
    if matches!(send(&mut candidate, paper), Reply::Response(P::Problem { .. })) {
        return None;
    }
    let accepted = page_documents(&open_document_center(&mut candidate));
    let id = accepted.iter().filter(|row| category(&row.document) == group).nth(index)?.id;
    let entry = candidate.draft.income.iter().find(|entry| entry.id == id)?;
    let topic = crate::interview::topic_of(&candidate.draft, entry)?;
    let request = if matches!(original.document, IncomeSource::K1 { .. }) {
        k1_fixture_answer_request(&mut candidate, &desired)?
    } else {
        information_return_answers_for_topic(&mut candidate, &desired, topic)?
    };
    if let P::SaveWagesAnswers { wages, followups, .. } = &request {
        let (root, local) = if local == "community" {
            let row = followups.iter().find(|row| {
                row.key == alloc::format!("income/{id}/community")
                    && matches!(row.answer, crate::WagesDocumentAnswer::Community { .. })
            })?;
            (
                libertas::SchemaPath::root()
                    .field("followups")
                    .key(row.key.as_str())
                    .field("answer"),
                "value",
            )
        } else {
            let answer = wages.iter().find(|row| row.key == alloc::format!("income/{id}/wage"))?;
            (
                libertas::SchemaPath::root()
                    .field("wages")
                    .key(answer.key.as_str())
                    .field("answers"),
                local.strip_prefix("answers/")?,
            )
        };
        let mapped = local
            .split('/')
            .fold(root, |path, segment| match segment.parse::<usize>() {
                Ok(index) => path.index(index),
                Err(_) => path.field(segment),
            })
            .to_string();
        return Some((candidate, request, mapped));
    }
    if matches!(request, P::SavePaymentParts { .. }) {
        let local = local.strip_prefix("parts")?;
        return Some((candidate, request, alloc::format!("[value{local}]")));
    }
    let P::SaveIncomes { followups, .. } = &request else { unreachable!() };
    let actual = accepted.iter().find(|row| row.id == id).unwrap();
    let mapped = crate::interview::fixture_issue_path(
        &original.document,
        &actual.document,
        id,
        key,
        local,
        followups,
    )?;
    Some((candidate, request, mapped))
}

fn topics_contains_interest(rows: &[IncomeDocumentItem]) -> bool {
    rows.iter().any(|row| matches!(row.document, IncomeSource::InterestDividend { .. }))
}

// The topic that owns an activity kind refuses its save until the paper has a
// receiving activity. A fixture that supplies the paper alone leaves that
// activity to its own later save, and the Review reports the gap meanwhile.
pub(in crate::tests) fn awaits_activity(reply: &Reply) -> bool {
    matches!(reply, Reply::Response(P::Problem { message, .. })
        if matches!(crate::Message::decode(message).key.as_str(),
            "ISSUE_RELATED_FORM_PAGE_RENTALS"
                | "ISSUE_RELATED_FORM_PAGE_FARMING"
                | "ISSUE_RELATED_FORM_PAGE_BUSINESS"))
}

pub(in crate::tests) fn save_source_fixture_request(
    session: &mut Interview,
    mut request: P,
) -> Reply {
    request = fixture_blank_owner_inputs(&session.draft, request);
    assert!(matches!(request, P::SaveDocuments { .. }));
    let rows = pending_fixture_answer_rows(session, &page_documents(&request));
    fn attempt(session: &mut Interview, request: P, rows: &[IncomeDocumentItem]) -> Reply {
        let loans = match &request {
            P::SaveDocuments { documents, .. } => documents
                .other
                .vehicle_loans
                .documents
                .iter()
                .filter(|row| row.id == 0)
                .cloned()
                .collect::<Vec<_>>(),
            _ => unreachable!(),
        };
        let mut reply = send(session, request);
        if matches!(reply, Reply::Response(P::Problem { .. })) {
            return reply;
        }
        if let Some(answer_reply) = save_k1_fixture_answers(session, rows) {
            reply = answer_reply;
            if matches!(reply, Reply::Response(P::Problem { .. })) {
                return reply;
            }
        }
        let matched = information_return_sources(session, rows);
        let mut topics = Vec::new();
        for (_, topic, _) in &matched {
            if !topics.contains(topic) {
                topics.push(*topic);
            }
        }
        for topic in topics {
            let request = information_return_answers_for_topic(session, rows, topic).unwrap();
            let next = send_fixture_topic_answers(session, rows, topic, request);
            if matches!(next, Reply::Response(P::Problem { .. })) {
                if !awaits_activity(&next) {
                    return next;
                }
                continue;
            }
            reply = next;
            if cooperative_child_answers(&matched, topic) {
                let request = information_return_answers_for_topic(session, rows, topic).unwrap();
                let next = send(session, request);
                if matches!(next, Reply::Response(P::Problem { .. })) && !awaits_activity(&next) {
                    return next;
                }
                if !matches!(next, Reply::Response(P::Problem { .. })) {
                    reply = next;
                }
            }
        }
        if topics_contains_interest(rows)
            && let Some(request) =
                information_return_answers_for_topic(session, rows, TaxSection::InterestDividends)
        {
            reply = send(session, request);
            if matches!(reply, Reply::Response(P::Problem { .. })) {
                return reply;
            }
        }
        if !loans.is_empty() {
            let Reply::Edit(mut request @ P::SaveDeductions { .. }) =
                crate::tests::cross::interview::from_review(session, TaxSection::Deductions)
            else {
                panic!("vehicle answers use the actual Deductions editor")
            };
            let P::SaveDeductions { value, .. } = &mut request else { unreachable!() };
            let mut used = Vec::new();
            for answer in &mut value.car_loans {
                if let Some((index, supplied)) =
                    loans.iter().enumerate().find(|(index, supplied)| {
                        !used.contains(index)
                            && (supplied.id == answer.id
                                || (supplied.id == 0
                                    && supplied.form.lender.name == answer.value.lender.name))
                    })
                {
                    used.push(index);
                    if let Some(business) = fixture_vehicle_business(supplied) {
                        answer.business = business;
                    }
                }
            }
            reply = send(session, request);
            if matches!(reply, Reply::Response(P::Problem { .. })) {
                return reply;
            }
        }
        finish_qsbs_fixture_answers(session, rows).unwrap_or(reply)
    }
    let mut candidate = restart(session.draft.clone());
    let reply = attempt(&mut candidate, request.clone(), &rows);
    if matches!(reply, Reply::Response(P::Problem { .. })) {
        return reply;
    }
    let reply = attempt(session, request, &rows);
    if !matches!(reply, Reply::Response(P::Problem { .. })) {
        mark_fixture_answers_applied(session, &rows);
    }
    reply
}

// Convenience fixtures provide printed facts and separately supplied personal
// answers. Exercise both real editors; keep rejected candidates atomic.
pub(in crate::tests) fn save_source_fixtures(
    session: &mut Interview,
    rows: Vec<IncomeDocumentItem>,
) -> Reply {
    if rows.iter().all(|row| matches!(row.document, IncomeSource::K1 { .. })) {
        return save_k1_sources_and_answers(session, rows);
    }
    let pending = pending_fixture_answer_rows(session, &rows);
    // Sources whose fixtures use their own answer adapter keep that adapter's
    // existing save behavior; registration tracking owns only these caches.
    let desired: Vec<_> = rows
        .iter()
        .filter(|row| {
            row.id == 0
                || pending.iter().any(|given| given.id == row.id)
                || !fixture_source_has_answers(&row.document)
        })
        .cloned()
        .collect();
    fn attempt(
        session: &mut Interview,
        rows: &[IncomeDocumentItem],
        desired: &[IncomeDocumentItem],
    ) -> Reply {
        let request = save_documents(session, rows.to_vec());
        let mut reply = send(session, request);
        if matches!(reply, Reply::Response(P::Problem { .. })) {
            return reply;
        }
        if let Some(answer_reply) = save_k1_fixture_answers(session, rows) {
            reply = answer_reply;
            if matches!(reply, Reply::Response(P::Problem { .. })) {
                return reply;
            }
        }
        let matched = information_return_sources(session, desired);
        let mut topics = Vec::new();
        for (_, topic, _) in &matched {
            if !topics.contains(topic) {
                topics.push(*topic);
            }
        }
        for topic in topics {
            let request = information_return_answers_for_topic(session, desired, topic).unwrap();
            reply = send_fixture_topic_answers(session, desired, topic, request);
            if matches!(reply, Reply::Response(P::Problem { .. })) {
                return reply;
            }
            if topic == TaxSection::InterestDividends
                || (cooperative_child_answers(&matched, topic))
            {
                let request =
                    information_return_answers_for_topic(session, desired, topic).unwrap();
                reply = send(session, request);
                if matches!(reply, Reply::Response(P::Problem { .. })) {
                    return reply;
                }
            }
        }
        finish_qsbs_fixture_answers(session, desired).unwrap_or(reply)
    }
    let mut candidate = restart(session.draft.clone());
    let reply = attempt(&mut candidate, &rows, &desired);
    if matches!(reply, Reply::Response(P::Problem { .. })) {
        return reply;
    }
    let reply = attempt(session, &rows, &desired);
    if !matches!(reply, Reply::Response(P::Problem { .. })) {
        mark_fixture_answers_applied(session, &desired);
    }
    reply
}

pub(in crate::tests) fn save_each_topic(session: &mut Interview, rows: Vec<IncomeDocumentItem>) {
    let (records, sources): (Vec<_>, Vec<_>) =
        rows.into_iter().partition(|row| matches!(row.document, IncomeSource::Record { .. }));
    let desired = pending_fixture_answer_rows(session, &sources);
    let deferred_answers = sources
        .iter()
        .any(|row| {
            matches!(
                row.document,
                IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Ltc1099(_) }
            )
        })
        .then(|| desired.clone());
    if !sources.is_empty() {
        let reply = if sources.iter().any(|row| {
            matches!(
                row.document,
                IncomeSource::K1 {
                    value: crate::K1Document::K1Partnership(_)
                        | crate::K1Document::K1Scorp(_)
                        | crate::K1Document::K1PartnershipActivities(_)
                        | crate::K1Document::K1ScorpActivities(_)
                }
            )
        }) {
            save_k1_sources_and_answers(session, sources)
        } else {
            let request = save_documents(session, sources);
            send(session, request)
        };
        crate::tests::cross::interview::accepted(&reply);
        if deferred_answers.is_none() {
            save_information_return_answers(session, &desired);
        }
    }
    if deferred_answers.is_some()
        && let Some(request) =
            information_return_answers_for_topic(session, &desired, TaxSection::SelfEmployment)
    {
        // Other personal answers still wait for their records; source-only
        // split acceptance cannot depend on the pending business save.
        let _ =
            accept_fixture_payment_parts(session, &desired, TaxSection::SelfEmployment, request)
                .expect("complete split fixtures are accepted before activity links");
    }
    let mut topics = Vec::new();
    for row in &records {
        let topic = topic_for(&row.document);
        if !topics.contains(&topic) {
            topics.push(topic);
        }
    }
    for topic in topics {
        let mine =
            records.iter().filter(|row| topic_for(&row.document) == topic).cloned().collect();
        let matched =
            deferred_answers.as_ref().map(|desired| information_return_sources(session, desired));
        let mut request = save_topic(session, topic, mine);
        if let Some(matched) = &matched {
            fill_information_return_answers(session, &mut request, matched);
        }
        crate::tests::cross::interview::accepted(&send(session, request));
    }
    if let Some(desired) = deferred_answers {
        save_information_return_answers(session, &desired);
    }
    mark_fixture_answers_applied(session, &desired);
}
// Opens an income topic's page from its hub, the way the client receives it.
pub(in crate::tests) fn open_topic(session: &mut Interview, topic: TaxSection) -> P {
    if matches!(topic, TaxSection::Education | TaxSection::Hsa | TaxSection::InjuredSpouse) {
        let reply = crate::tests::cross::interview::from_review(session, topic);
        let reply = if topic == TaxSection::InjuredSpouse {
            super::injured::native_editor(reply, &[])
        } else {
            reply
        };
        return match reply {
            Reply::Edit(request) => request,
            other => panic!("expected the {topic:?} page, got {other:?}"),
        };
    }
    let request = P::Navigate {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision: session.draft.revision,
        from: PAGE_INCOME,
        section: (topic).into(),
        navigation: NavigationAvailability::default(),
    };
    match send(session, request) {
        Reply::Edit(request @ (P::SaveIncomes { .. } | P::SaveWagesAnswers { .. })) => request,
        other => panic!("expected the {topic:?} page, got {other:?}"),
    }
}
// Open the source editor through its normal navigation.
pub(in crate::tests) fn open_documents(session: &mut Interview) -> P {
    open_document_center(session)
}
pub(in crate::tests) fn all_documents(session: &mut Interview) -> Vec<IncomeDocumentItem> {
    page_documents(&open_document_center(session))
}
// Observe current typed editor values as native fixture rows, without retaining
// any former client request or decoder.
pub(in crate::tests) fn page_documents(page: &P) -> Vec<IncomeDocumentItem> {
    match page {
        P::SaveWagesAnswers { records, .. } => records
            .documents
            .iter()
            .cloned()
            .map(crate::WagesRecordItem::into_income)
            .map(crate::IncomeRecordItem::into_income)
            .collect(),
        P::SaveIncomes { documents, .. } => {
            documents.documents.iter().cloned().map(crate::IncomeRecordItem::into_income).collect()
        }
        P::SaveDocuments { documents, .. } => {
            let mut rows = Vec::new();
            rows.extend(
                documents
                    .common
                    .wages
                    .documents
                    .iter()
                    .cloned()
                    .map(crate::WageDocumentRow::into_income),
            );
            rows.extend(
                documents
                    .common
                    .interest_dividends
                    .documents
                    .iter()
                    .cloned()
                    .map(crate::InterestDividendDocumentRow::into_income),
            );
            rows.extend(
                documents
                    .common
                    .investment_sales
                    .documents
                    .iter()
                    .cloned()
                    .map(crate::InvestmentSaleDocumentRow::into_income),
            );
            rows.extend(
                documents
                    .common
                    .retirement
                    .documents
                    .iter()
                    .cloned()
                    .map(crate::RetirementDocumentRow::into_income),
            );
            rows.extend(
                documents
                    .common
                    .self_employment
                    .documents
                    .iter()
                    .cloned()
                    .map(crate::SelfEmploymentDocumentRow::into_income),
            );
            rows.extend(
                documents
                    .common
                    .government_payments
                    .documents
                    .iter()
                    .cloned()
                    .map(crate::GovernmentPaymentDocumentRow::into_income),
            );
            rows.extend(
                documents
                    .common
                    .gambling
                    .documents
                    .iter()
                    .cloned()
                    .map(crate::GamblingDocumentRow::into_income),
            );
            rows.extend(
                documents
                    .common
                    .k1
                    .documents
                    .iter()
                    .cloned()
                    .map(crate::K1DocumentRow::into_income),
            );
            rows.extend(
                documents
                    .other
                    .farming
                    .documents
                    .iter()
                    .cloned()
                    .map(crate::FarmingDocumentRow::into_income),
            );
            rows.extend(
                documents
                    .other
                    .other_income
                    .documents
                    .iter()
                    .cloned()
                    .map(crate::OtherIncomeDocumentRow::into_income),
            );
            rows.extend(
                documents
                    .other
                    .canceled_debt
                    .documents
                    .iter()
                    .cloned()
                    .map(crate::CanceledDebtDocumentRow::into_income),
            );
            rows
        }
        other => panic!("expected a current income editor, got {other:?}"),
    }
}

// Native preview catalogs used by the ordinary income interview producers.
pub(in crate::tests) fn catalog_preview_document(document: IncomeSource) -> P {
    let mut request = super::survey::survey_preview_document(document);
    let P::PreviewDocument {
        k1_credit_lines,
        research_credit_lines,
        renewable_credit_lines,
        transferable_credit_lines,
        foreign_countries,
        ..
    } = &mut request
    else {
        return request;
    };
    *k1_credit_lines = crate::business_credit_page::k1_credit_lines();
    *research_credit_lines = crate::business_credit_page::research_credit_lines();
    *renewable_credit_lines = crate::business_credit_page::renewable_credit_lines();
    *transferable_credit_lines = crate::business_credit_page::transferable_credit_lines();
    *foreign_countries = crate::foreign_accounts::countries();
    request
}

// Static native catalogs for pure calculation tests. Editor-derived dynamic
// catalogs continue through estimate_documents_from_save.
pub(in crate::tests) fn catalog_estimate_documents(
    topic: TaxSection,
    documents: crate::IncomeRecordsEstimateInput,
    followups: Vec<crate::DocumentQuestion>,
) -> P {
    P::EstimateDocuments {
        cookie: alloc::string::String::new(),
        revision: 0,
        followups,
        questions: Vec::new(),
        situations: crate::DocumentSituations::NoneOfThese,
        allowed_situations: Vec::new(),
        situation_answers: Vec::new(),
        situation_documents: Vec::new(),
        disaster_choices: Vec::new(),
        initial_asset_amt_carry_choices: Vec::new(),
        initial_interest_limit_choices: Vec::new(),
        initial_rental_recovery_choices: Vec::new(),
        rental_energy_buildings: Vec::new(),
        casualty_properties: Vec::new(),
        empty_k1_activity_choices: crate::tests::support::builders::empty_k1_activity_choices(),
        allowed_continuing_risk: ScorpRiskStart::continuing_choices(),
        allowed_injured_source_facts: vec![],
        allowed_injured_retained_owners: vec![],
        ltc_payee_sources: vec![],
        wage_scorporations: Vec::new(),
        allowed_wage_scorporations: Vec::new(),
        initial_k1_investment_choices: alloc::vec![0, 1],
        research_businesses: Vec::new(),
        retained_research_choices: Vec::new(),
        k1_credit_lines: crate::business_credit_page::k1_credit_lines(),
        research_credit_lines: crate::business_credit_page::research_credit_lines(),
        renewable_credit_lines: crate::business_credit_page::renewable_credit_lines(),
        transferable_credit_lines: crate::business_credit_page::transferable_credit_lines(),
        mortgage_loans: Vec::new(),
        allowed_refund_management: vec![],
        user: TEST_USER,
        topic,
        source_choices: crate::IncomeSourceChoices::from_kinds(&topic_kinds(topic)),
        allowed_documents: topic_kinds(topic),
        allowed_owners: vec![Owner::Taxpayer as i32],
        allowed_business_source: vec![],
        allowed_farm_source: vec![],
        farms: vec![],
        cooperative_activities: Vec::new(),

        cooperative_properties: Vec::new(),
        businesses: vec![],
        rentals: vec![],
        royalty_properties: vec![],
        equipment_rentals: vec![],
        hobbies: vec![],
        ccc_lots: vec![],
        farm_codes: vec![],
        farm_limitations: vec![],
        farm_sources: vec![],
        basis_assets: vec![],
        farm_method_changes: vec![],
        inherited_iras: vec![],
        inherited_roth_iras: vec![],
        ltc_insured: vec![],
        annuity_sources: vec![],
        recovery_sources: vec![],
        allowed_partner_source: vec![],
        allowed_home_spouse: vec![],
        people: vec![],
        allowed_other_parent: vec![],
        children: vec![],
        documents,
        allowed_community: Vec::new(),
        foreign_countries: crate::foreign_accounts::countries(),
        interest_statements: Vec::new(),
        qbi_businesses: vec![],
    }
}

// Open the standalone-document editor through ordinary offered navigation.
pub(in crate::tests) fn open_document_center(session: &mut Interview) -> P {
    let request = crate::tests::cross::interview::navigate(&session.draft, TaxSection::Documents);
    match send(session, request) {
        Reply::Edit(request @ P::SaveDocuments { .. }) => {
            crate::tests::support::builders::remember_accepted_answers(&session.draft, &request);
            remember_stored_estate_answers(session, &request);
            request
        }
        other => panic!("expected Document center, got {other:?}"),
    }
}

// A reopened estate, trust or S-corporation K-1 carries only its stored paper; the fixture
// answers a later edit starts from are the stored source's own.
fn remember_stored_estate_answers(session: &Interview, request: &P) {
    for row in page_documents(request) {
        let IncomeSource::K1 { value } = row.document else { continue };
        let Some(stored) = session.draft.income.iter().find(|entry| entry.id == row.id) else {
            continue;
        };
        match (value, &stored.income) {
            (
                crate::K1Document::K1EstateTrust(document),
                FederalIncome::EstateTrustPortfolio { data },
            ) => crate::tests::support::builders::remember_reopened_estate_document(
                document,
                data.clone(),
            ),
            (crate::K1Document::K1Scorp(document), FederalIncome::Scorp { data }) => {
                crate::tests::support::builders::remember_reopened_scorp_document(
                    document,
                    data.personal.clone(),
                )
            }
            _ => {}
        }
    }
}

// Exercise candidate validation through the real Save and Review lifecycle in
// a cold session, leaving the caller's accepted return untouched.
pub(in crate::tests) fn candidate_save_review(session: &mut Interview, request: P) -> Reply {
    if !matches!(request, P::SaveDocuments { .. }) {
        return send(session, request);
    }
    let mut candidate = restart(session.draft.clone());
    let reply = send(&mut candidate, request);
    if matches!(reply, Reply::Response(P::Problem { .. })) {
        return reply;
    }
    send(&mut candidate, P::ReviewAccepted { user: TEST_USER })
}

// A document's accepted native amount, after normal source validation in a
// cold candidate session. Record previews keep their ordinary protocol.
pub(in crate::tests) fn saved_document_amount(
    session: &mut Interview,
    source: IncomeSource,
) -> Option<i64> {
    if matches!(source, IncomeSource::Record { .. }) {
        let request = catalog_preview_document(source);
        let Reply::Response(P::IncomePreview { preview, .. }) = send(session, request) else {
            panic!("ordinary record preview")
        };
        return preview;
    }
    let mut candidate = restart(session.draft.clone());
    let original_ids: Vec<_> = candidate.draft.income.iter().map(|entry| entry.id).collect();
    let desired = vec![crate::tests::support::builders::native_income::document(0, source)];
    let request = save_documents(&mut candidate, desired.clone());
    accepted(&send(&mut candidate, request));
    let accepted_id = candidate
        .draft
        .income
        .iter()
        .find(|entry| !original_ids.contains(&entry.id))
        .expect("a newly accepted physical source has its own identity")
        .id;
    if let Some(reply) = save_k1_fixture_answers(&mut candidate, &desired)
        && matches!(reply, Reply::Response(P::Problem { .. }))
    {
        return None;
    }
    let topics = information_return_sources(&mut candidate, &desired);
    let mut handled = Vec::new();
    for (_, topic, _) in topics {
        if handled.contains(&topic) {
            continue;
        }
        handled.push(topic);
        let request =
            information_return_answers_for_topic(&mut candidate, &desired, topic).unwrap();
        if matches!(
            send_fixture_topic_answers(&mut candidate, &desired, topic, request),
            Reply::Response(P::Problem { .. })
        ) {
            return None;
        }
    }
    let mut notes = Vec::new();
    let working = crate::unknowns::supported(&candidate.draft, &mut notes);
    let shown = working.as_ref().unwrap_or(&candidate.draft);
    match shown.income.iter().find(|entry| entry.id == accepted_id) {
        Some(entry) => crate::interview::document_amount(
            &entry.income,
            crate::federal_rules::separate_return(shown),
        ),
        // The supported projection may explicitly omit a source. Its amount
        // on that projection is zero; a missing accepted source is an error.
        None if working.is_some() => Some(0),
        None => None,
    }
}

pub(in crate::tests) fn save_ltc_fixture(
    session: &mut Interview,
    id: i64,
    paper: Ltc1099Paper,
) -> Reply {
    let request = save_documents(
        session,
        vec![crate::tests::support::builders::native_income::document(
            id,
            IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Ltc1099(paper) },
        )],
    );
    send(session, request)
}

pub(in crate::tests) fn reopened_ltc_fixture(session: &mut Interview) -> (i64, Ltc1099Paper) {
    let request = open_document_center(session);
    let row = page_documents(&request)
        .into_iter()
        .find(|row| {
            matches!(
                row.document,
                IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Ltc1099(_) }
            )
        })
        .expect("accepted long-term care source");
    let IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Ltc1099(paper) } =
        row.document
    else {
        unreachable!()
    };
    (row.id, paper)
}

pub(in crate::tests) fn qsbs_bool_fact(session: &mut Interview, field: &str, answer: bool) {
    let mut host = crate::tests::cross::interview::open_topic(session, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut host else { panic!("QSBS topic") };
    let row = followups.iter_mut().find(|row| row.key.ends_with(field)).unwrap();
    let before = session.draft.clone();
    let Reply::Response(P::QsbsBooleanAnswered { value, known }) = send(
        session,
        P::AnswerQsbsBoolean { user: TEST_USER, prompt: row.prompt.clone(), value: answer },
    ) else {
        panic!("strict question answer")
    };
    assert_eq!(session.draft, before, "link response only updates the unsaved host");
    let crate::DocumentAnswer::QsbsBoolean { value: target } = &mut row.answer else { panic!() };
    target.value = Some(value);
    target.known = known;
    crate::tests::cross::interview::accepted(&send(session, host));
}

pub(in crate::tests) fn qsbs_accepted_source(session: &Interview) -> &QsbIssuerSale {
    session
        .draft
        .income
        .iter()
        .find_map(|entry| match &entry.income {
            FederalIncome::QualifiedSmallBusinessSale { data } => Some(data),
            _ => None,
        })
        .unwrap()
}

// Per-form modules split out by tools/split-hubs; the tool regenerates this block.
#[path = "form_1099_b_pages.rs"]
mod form_1099_b_pages;
#[path = "form_1099_c_pages.rs"]
mod form_1099_c_pages;
#[path = "form_1099_div_pages.rs"]
mod form_1099_div_pages;
#[path = "form_1099_int_pages.rs"]
mod form_1099_int_pages;
#[path = "form_1099_oid_pages.rs"]
mod form_1099_oid_pages;
#[path = "information_returns_pages.rs"]
mod information_returns_pages;
#[path = "k1_pages.rs"]
mod k1_pages;
#[path = "w2_pages.rs"]
mod w2_pages;
#[allow(unused_imports)]
pub use form_1099_b_pages::*;
#[allow(unused_imports)]
pub use form_1099_c_pages::*;
#[allow(unused_imports)]
pub use form_1099_div_pages::*;
#[allow(unused_imports)]
pub use form_1099_int_pages::*;
#[allow(unused_imports)]
pub use form_1099_oid_pages::*;
#[allow(unused_imports)]
pub use information_returns_pages::*;
#[allow(unused_imports)]
pub use k1_pages::*;
#[allow(unused_imports)]
pub use w2_pages::*;
#[cfg(test)]
#[allow(dead_code)]
pub(crate) const HUB_SOURCE: &str = concat!(
    include_str!("income_pages.rs"),
    include_str!("form_1099_b_pages.rs"),
    include_str!("form_1099_c_pages.rs"),
    include_str!("form_1099_div_pages.rs"),
    include_str!("form_1099_int_pages.rs"),
    include_str!("form_1099_oid_pages.rs"),
    include_str!("information_returns_pages.rs"),
    include_str!("k1_pages.rs"),
    include_str!("w2_pages.rs"),
);
// End of the tools/split-hubs block.

/// Answers the long-term care interest question that sits beside Form 1099-LTC `id`.
pub(in crate::tests) fn answer_ltc_interest(
    session: &mut Interview,
    id: i64,
    amount: i64,
) -> Reply {
    let mut host =
        crate::tests::cross::interview::open_topic(session, TaxSection::LongTermCareBenefits);
    let P::SaveIncomes { followups, .. } = &mut host else { panic!("long-term care topic") };
    let row = followups
        .iter_mut()
        .find(|row| row.key == alloc::format!("income/{id}/ltc_interest"))
        .expect("long-term care interest question");
    let crate::DocumentAnswer::LtcInterest { value } = &mut row.answer else { unreachable!() };
    *value = amount;
    send(session, host)
}

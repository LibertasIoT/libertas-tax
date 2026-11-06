use crate::document_names::{AcceptedPartySource, Assignment};
use crate::{
    BusinessActivity, FederalDependent, FederalGambling, FederalIncome, FederalIncomeEntry,
    NonbusinessPropertyKind,
};
use crate::{DocumentRecipient, PrintedDocumentNames};

// A document's kind: its summary noun resources (SAVING_KIND_*) and its display
// name resource (LABEL_ENTRY_*, the capitalized noun over the entry identity).
// An entry stores only the name the person typed; an unnamed source is named by
// the second, built in one place (income_context).
pub(crate) fn document_kind_keys(entry: &FederalIncomeEntry) -> (&'static str, &'static str) {
    match &entry.income {
        FederalIncome::Wage { data } if data.is_cash_tips() => {
            ("SAVING_KIND_CASH_TIP_RECORD", "LABEL_ENTRY_CASH_TIP_RECORD")
        }
        FederalIncome::Wage { .. } => ("SAVING_KIND_W2", "LABEL_ENTRY_W2"),
        FederalIncome::Interest { data } if crate::FederalInterest::from(data).private_loan => {
            ("SAVING_KIND_PRIVATE_LOAN_INTEREST", "LABEL_ENTRY_PRIVATE_LOAN_INTEREST")
        }
        FederalIncome::Interest { .. } => ("SAVING_KIND_1099_INT", "LABEL_ENTRY_1099_INT"),
        FederalIncome::Dividend { .. } => ("SAVING_KIND_1099_DIV", "LABEL_ENTRY_1099_DIV"),
        FederalIncome::Oid { .. } => ("SAVING_KIND_1099_OID", "LABEL_ENTRY_1099_OID"),
        FederalIncome::InstallmentSale { .. } => {
            ("SAVING_KIND_CURRENT_INSTALLMENT_SALE", "LABEL_ENTRY_CURRENT_INSTALLMENT_SALE")
        }
        FederalIncome::Broker { data } => match data.paper {
            crate::BrokerPaper::Form1099B(_) => ("SAVING_KIND_FORM1099B", "LABEL_ENTRY_FORM1099B"),
            crate::BrokerPaper::Form1099DA(_) => {
                ("SAVING_KIND_FORM1099DA", "LABEL_ENTRY_FORM1099DA")
            }
        },
        FederalIncome::Sale { .. } => ("SAVING_KIND_1099_B", "LABEL_ENTRY_1099_B"),
        FederalIncome::OpportunityFund { .. } => ("SAVING_KIND_QOF", "LABEL_ENTRY_QOF"),
        FederalIncome::OpportunityFundDeferral { .. } => {
            ("SAVING_KIND_QOF_DEFERRAL", "LABEL_ENTRY_QOF_DEFERRAL")
        }
        FederalIncome::Section1256 { .. } => {
            ("SAVING_KIND_1099_B_CONTRACTS", "LABEL_ENTRY_1099_B_CONTRACTS")
        }
        FederalIncome::IsoSale { .. } | FederalIncome::IsoSaleSource { .. } => {
            ("SAVING_KIND_ISO_SALE", "LABEL_ENTRY_ISO_SALE")
        }
        FederalIncome::EmployeeStockSale { .. } => {
            ("SAVING_KIND_EMPLOYEE_SALE", "LABEL_ENTRY_EMPLOYEE_SALE")
        }
        FederalIncome::EsppSale { .. } | FederalIncome::EsppSaleSource { .. } => {
            ("SAVING_KIND_ESPP_SALE", "LABEL_ENTRY_ESPP_SALE")
        }
        FederalIncome::NuaSale { .. } | FederalIncome::NuaSaleSource { .. } => {
            ("SAVING_KIND_NUA_SALE", "LABEL_ENTRY_NUA_SALE")
        }
        FederalIncome::QualifiedSmallBusinessSale { .. } => {
            ("SAVING_KIND_QSBS_SALE", "LABEL_ENTRY_QSBS_SALE")
        }
        FederalIncome::SmallBusinessStock { .. } => {
            ("SAVING_KIND_SECTION_1244_LOSS", "LABEL_ENTRY_SECTION_1244_LOSS")
        }
        FederalIncome::WorthlessSecurity { .. } => {
            ("SAVING_KIND_WORTHLESS_SECURITY", "LABEL_ENTRY_WORTHLESS_SECURITY")
        }
        FederalIncome::NonbusinessBadDebt { .. } => {
            ("SAVING_KIND_BAD_DEBT", "LABEL_ENTRY_BAD_DEBT")
        }
        FederalIncome::ReturnOfCapital { .. } => {
            ("SAVING_KIND_CAPITAL_RETURN", "LABEL_ENTRY_CAPITAL_RETURN")
        }
        FederalIncome::UndistributedGain { .. } => {
            ("SAVING_KIND_FORM_2439", "LABEL_ENTRY_FORM_2439")
        }
        FederalIncome::HomeSale { data } if data.form.paper().is_some() => {
            ("SAVING_KIND_1099_S", "LABEL_ENTRY_1099_S")
        }
        FederalIncome::HomeSale { .. } => ("SAVING_KIND_HOME_SALE", "LABEL_ENTRY_HOME_SALE"),
        FederalIncome::Retirement { data: crate::RetirementSource::Rrb1099R(_) } => {
            ("SAVING_KIND_RRB_1099_R", "LABEL_ENTRY_RRB_1099_R")
        }
        FederalIncome::Retirement { .. }
        | FederalIncome::Roth { .. }
        | FederalIncome::TrumpDeath { .. }
        | FederalIncome::TrumpAbleRollover { .. } => ("SAVING_KIND_1099_R", "LABEL_ENTRY_1099_R"),
        FederalIncome::InheritedIra { .. } => {
            ("SAVING_KIND_INHERITED_IRA", "LABEL_ENTRY_INHERITED_IRA")
        }
        FederalIncome::SocialSecurity { data: crate::SocialBenefitStatement::Rrb1099(_) } => {
            ("FORM_RRB1099", "LABEL_ENTRY_RRB1099")
        }
        FederalIncome::SocialSecurity { .. } => ("SAVING_KIND_SSA_1099", "LABEL_ENTRY_SSA_1099"),
        FederalIncome::Business { data }
            if matches!(data.activity, BusinessActivity::Farming { .. }) =>
        {
            ("SAVING_KIND_FARM", "LABEL_ENTRY_FARM")
        }
        FederalIncome::Business { .. } => ("SAVING_KIND_BUSINESS", "LABEL_ENTRY_BUSINESS"),
        FederalIncome::Rental { .. } => ("SAVING_KIND_RENTAL", "LABEL_ENTRY_RENTAL"),
        FederalIncome::FarmRental { .. } => ("SAVING_KIND_FARM_RENTAL", "LABEL_ENTRY_FARM_RENTAL"),
        FederalIncome::GovernmentPayment { .. } => ("SAVING_KIND_1099_G", "LABEL_ENTRY_1099_G"),
        FederalIncome::Cooperative { .. } => ("SAVING_KIND_1099_PATR", "LABEL_ENTRY_1099_PATR"),
        FederalIncome::OtherIncome { .. } => {
            ("SAVING_KIND_OTHER_INCOME", "LABEL_ENTRY_OTHER_INCOME")
        }
        FederalIncome::Nec1099 { .. } => ("SAVING_KIND_1099_NEC", "LABEL_ENTRY_1099_NEC"),
        FederalIncome::Misc1099 { .. } => ("SAVING_KIND_1099_MISC", "LABEL_ENTRY_1099_MISC"),
        FederalIncome::K1099 { .. } => ("SAVING_KIND_1099_K", "LABEL_ENTRY_1099_K"),
        FederalIncome::Royalty { .. } => ("SAVING_KIND_ROYALTIES", "LABEL_ENTRY_ROYALTIES"),
        FederalIncome::NonbusinessProperty {
            kind: NonbusinessPropertyKind::PersonalRental { .. },
            ..
        } => ("SAVING_KIND_EQUIPMENT_RENTAL", "LABEL_ENTRY_EQUIPMENT_RENTAL"),
        FederalIncome::Gambling { data } if FederalGambling::from(data).own_records => {
            ("SAVING_KIND_GAMBLING_LOSSES_ENTRY", "LABEL_ENTRY_GAMBLING_LOSSES_ENTRY")
        }
        FederalIncome::Gambling { .. } => ("SAVING_KIND_W2G", "LABEL_ENTRY_W2G"),
        FederalIncome::EstateTrustPortfolio { .. } => {
            ("SAVING_KIND_FORM_1041_K1", "LABEL_ENTRY_FORM_1041_K1")
        }
        FederalIncome::Scorp { .. } | FederalIncome::ScorpActivities { .. } => {
            ("SAVING_KIND_FORM_1120_S_K1", "LABEL_ENTRY_FORM_1120_S_K1")
        }
        FederalIncome::Partnership { .. } | FederalIncome::PartnershipActivities { .. } => {
            ("SAVING_KIND_FORM_1065_K1", "LABEL_ENTRY_FORM_1065_K1")
        }
        FederalIncome::PersonalDebt { .. } => ("SAVING_KIND_1099_C", "LABEL_ENTRY_1099_C"),
        FederalIncome::PropertyAcquisition { .. } => ("SAVING_KIND_1099_A", "LABEL_ENTRY_1099_A"),
        FederalIncome::Alimony { .. } => {
            ("SAVING_KIND_ALIMONY_AGREEMENT", "LABEL_ENTRY_ALIMONY_AGREEMENT")
        }
        FederalIncome::SavingsBonds { .. } => {
            ("SAVING_KIND_SAVINGS_BONDS", "LABEL_ENTRY_SAVINGS_BONDS")
        }
        FederalIncome::BondRedemption { .. } => ("SAVING_KIND_FORM_8818", "LABEL_ENTRY_FORM_8818"),
        FederalIncome::ChildElection { .. } => ("SAVING_KIND_FORM_8814", "LABEL_ENTRY_FORM_8814"),
        FederalIncome::EducationDistribution { .. } => ("SAVING_KIND_1099_Q", "LABEL_ENTRY_1099_Q"),
        FederalIncome::MedicalDistribution { data } if data.trustee_statement_paper().is_some() => {
            ("SAVING_KIND_HSA_TRUSTEE_STATEMENT", "LABEL_ENTRY_HSA_TRUSTEE_STATEMENT")
        }
        FederalIncome::MedicalDistribution { .. } => ("SAVING_KIND_1099_SA", "LABEL_ENTRY_1099_SA"),
        FederalIncome::LtcBenefits { .. } => ("SAVING_KIND_1099_LTC", "LABEL_ENTRY_1099_LTC"),
        FederalIncome::LtcInsured { .. } => ("SAVING_KIND_LTC_INSURED", "LABEL_ENTRY_LTC_INSURED"),
        FederalIncome::Able { .. } => ("SAVING_KIND_1099_QA", "LABEL_ENTRY_1099_QA"),
        FederalIncome::EducationAccountExpenses { .. } => {
            ("SAVING_KIND_EDUCATION_ACCOUNT_EXPENSES", "LABEL_ENTRY_EDUCATION_ACCOUNT_EXPENSES")
        }
        FederalIncome::ForeignEmployment { .. } => {
            ("SAVING_KIND_FOREIGN_PAY", "LABEL_ENTRY_FOREIGN_PAY")
        }
        // The nine relief and other-income kinds.
        FederalIncome::EmployeeWages { .. } => ("SAVING_KIND_FORM_8919", "LABEL_ENTRY_FORM_8919"),
        FederalIncome::HealthLoanAssistance { .. } => {
            ("SAVING_KIND_HEALTH_LOAN_REPAYMENT", "LABEL_ENTRY_HEALTH_LOAN_REPAYMENT")
        }
        FederalIncome::Scholarship { .. } => ("SAVING_KIND_SCHOLARSHIP", "LABEL_ENTRY_SCHOLARSHIP"),
        FederalIncome::JuryDuty { .. } => ("SAVING_KIND_JURY_PAY", "LABEL_ENTRY_JURY_PAY"),
        FederalIncome::HobbySales { .. } => ("SAVING_KIND_HOBBY_SALES", "LABEL_ENTRY_HOBBY_SALES"),
        FederalIncome::WildfireRelief { .. } => {
            ("SAVING_KIND_WILDFIRE_RELIEF", "LABEL_ENTRY_WILDFIRE_RELIEF")
        }
        FederalIncome::DisasterRelief { .. } => {
            ("SAVING_KIND_DISASTER_RELIEF", "LABEL_ENTRY_DISASTER_RELIEF")
        }
        FederalIncome::CasualtyLivingInsurance { .. } => {
            ("SAVING_KIND_CASUALTY_LIVING_INSURANCE", "LABEL_ENTRY_CASUALTY_LIVING_INSURANCE")
        }
        FederalIncome::DisasterMitigation { .. } => {
            ("SAVING_KIND_DISASTER_MITIGATION", "LABEL_ENTRY_DISASTER_MITIGATION")
        }
    }
}

pub(crate) fn record_display_arg(name: &str, id: i64) -> crate::MessageArg {
    if name.trim().is_empty() {
        crate::MessageArg::Number(id)
    } else {
        crate::MessageArg::Text(name.into())
    }
}

pub(crate) fn income_display_arg(entry: &FederalIncomeEntry) -> crate::MessageArg {
    record_display_arg(&entry.label, entry.id)
}

// The one owner of an income entry's display name. A name the person typed
// stays literal. An unnamed entry reads as its kind's display noun over its
// identity (LABEL_ENTRY_*: "Business 2", "W-2 1"), a typed Number argument; a
// Form 8814 election reads as its child's name, looked up on the people page.
pub(crate) fn income_display_label(
    entry: &FederalIncomeEntry,
    dependents: &[FederalDependent],
) -> crate::Message {
    if !entry.label.trim().is_empty() {
        return crate::Message::new("TEXT", alloc::vec![income_display_arg(entry)]);
    }
    if let FederalIncome::ChildElection { data } = &entry.income
        && let Some(child) = dependents.iter().find(|c| c.id == data.child_id)
    {
        return crate::Message::new("TEXT", alloc::vec![crate::MessageArg::Encoded(child.name())]);
    }
    // A printed issuer names the document by its form and issuer ("1099-MISC from Acme");
    // the record number is only the last resort.
    if let Some(issuer) = entry
        .printed
        .as_ref()
        .and_then(|names| names.issuer.as_ref())
        .map(|party| party.name.trim())
        .filter(|name| !name.is_empty())
    {
        let kind = document_kind_keys(entry).0;
        let kind = if kind.starts_with("SAVING_KIND_") {
            alloc::format!("{kind}_ONE")
        } else {
            kind.into()
        };
        return crate::Message::new(
            "DOCUMENT_FROM",
            alloc::vec![crate::MessageArg::Resource(kind), crate::MessageArg::Text(issuer.into())],
        );
    }
    crate::Message::new(
        document_kind_keys(entry).1,
        alloc::vec![crate::MessageArg::Number(entry.id)],
    )
}

impl crate::IncomeSource {
    pub(crate) fn recipient_domain(&self) -> crate::document_name_matcher::RecipientDomain {
        use crate::document_name_matcher::RecipientDomain;
        match self {
            Self::InterestDividend {
                value:
                    crate::InterestDividendDocument::Int1099(_)
                    | crate::InterestDividendDocument::Div1099(_),
            } => RecipientDomain::Account,
            Self::OtherIncome { value: crate::OtherIncomeDocument::Q1099(_) } => {
                RecipientDomain::Student
            }
            _ => RecipientDomain::Earner,
        }
    }
    pub(crate) fn printed_names(&self) -> Option<PrintedDocumentNames> {
        let (recipient, issuer) = match self {
            Self::Wage { value: crate::WageDocument::W2(value) } => {
                (&value.employee, &value.employer)
            }
            Self::Wage { value: crate::WageDocument::Form8919(value) } => {
                (&value.source_recipient, &value.source_issuer)
            }
            Self::InterestDividend { value: crate::InterestDividendDocument::Int1099(value) } => {
                (&value.recipient, &value.payer)
            }
            Self::InterestDividend { value: crate::InterestDividendDocument::Div1099(value) } => {
                (&value.recipient, &value.payer)
            }
            Self::InterestDividend { value: crate::InterestDividendDocument::Oid1099(value) } => {
                (&value.recipient, &value.payer)
            }
            Self::InterestDividend { value: crate::InterestDividendDocument::Form2439(value) } => {
                (&value.shareholder, &value.ric_or_reit)
            }
            Self::InterestDividend { value: crate::InterestDividendDocument::Form8818(value) } => {
                return Some(PrintedDocumentNames {
                    recipient: crate::PrintedParty {
                        name: value.name.clone(),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            }
            Self::InterestDividend {
                value: crate::InterestDividendDocument::SavingsBonds(value),
            } => {
                return Some(PrintedDocumentNames {
                    recipient: value.form.taxpayer.clone(),
                    ..Default::default()
                });
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099B(value) } => {
                (&value.recipient, &value.payer)
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099DA(value) } => {
                (&value.recipient, &value.filer)
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::Sale1099B(value) } => {
                (&value.recipient, &value.payer)
            }
            Self::InvestmentSale {
                value: crate::InvestmentSaleDocument::Contracts1099B(value),
            } => (&value.recipient, &value.payer),
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099S(value) } => {
                (&value.transferor, &value.filer)
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::Form3921(value) } => {
                (&value.employee, &value.transferor)
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::IsoShareSale(value) } => {
                (&value.source_recipient, &value.source_issuer)
            }
            Self::InvestmentSale {
                value: crate::InvestmentSaleDocument::EmployeeShareSale(value),
            } => (&value.source_recipient, &value.source_issuer),
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::EsppShareSale(value) } => {
                (&value.source_recipient, &value.source_issuer)
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::NuaShareSale(value) } => {
                (&value.source_recipient, &value.source_issuer)
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::QsbsSale(value) } => {
                (&value.source_recipient, &value.source_issuer)
            }
            // Form 8997 prints no issuer: its fund, column (c) with the column (a) EIN, names it.
            Self::InvestmentSale {
                value: crate::InvestmentSaleDocument::OpportunityFund(value),
            } => {
                return Some(PrintedDocumentNames {
                    recipient: value.source_recipient.clone(),
                    issuer: Some(crate::PrintedParty {
                        name: value.fund.clone(),
                        federal_id: value.fund_ein.clone(),
                        address: Default::default(),
                    }),
                    ..Default::default()
                });
            }
            Self::Retirement { value: crate::RetirementDocument::R1099(value) } => {
                (&value.recipient, &value.payer)
            }
            Self::Retirement { value: crate::RetirementDocument::Rrb1099R(value) } => {
                (&value.recipient, &value.issuer)
            }
            Self::Retirement { value: crate::RetirementDocument::Ssa1099(value) } => {
                (&value.recipient, &value.issuer)
            }
            Self::Retirement { value: crate::RetirementDocument::Rrb1099(value) } => {
                return Some(PrintedDocumentNames {
                    recipient: crate::PrintedParty {
                        name: value.recipient.name.clone(),
                        federal_id: value.recipient_id.clone(),
                        address: value.recipient.address.clone(),
                    },
                    issuer: Some(value.issuer.clone()),
                    ..Default::default()
                });
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::Form3922(value) } => {
                (&value.employee, &value.corporation)
            }
            Self::SelfEmployment { value: crate::SelfEmploymentDocument::Form1099Nec(value) } => {
                (&value.recipient, &value.payer)
            }
            Self::SelfEmployment { value: crate::SelfEmploymentDocument::Form1099K(value) } => {
                (&value.recipient, &value.payer)
            }
            Self::GovernmentPayment { value, .. } => (&value.recipient, &value.payer),
            Self::OtherIncome { value: crate::OtherIncomeDocument::Form1099Misc(value) } => {
                (&value.recipient, &value.payer)
            }
            Self::OtherIncome { value: crate::OtherIncomeDocument::Q1099(value) } => {
                (&value.recipient, &value.payer_trustee)
            }
            Self::OtherIncome { value: crate::OtherIncomeDocument::Sa1099(value) } => {
                (&value.recipient, &value.trustee_payer)
            }
            Self::OtherIncome { value: crate::OtherIncomeDocument::HsaTrusteeStatement(value) } => {
                (&value.recipient, &value.trustee_payer)
            }
            Self::OtherIncome { value: crate::OtherIncomeDocument::Ltc1099(value) } => {
                (&value.recipient, &value.payer)
            }
            Self::OtherIncome { value: crate::OtherIncomeDocument::Qa1099(value) } => {
                (&value.recipient, &value.payer)
            }
            Self::OtherIncome { value: crate::OtherIncomeDocument::WildfireRelief(value) } => {
                (&value.source_recipient, &value.source_issuer)
            }
            Self::OtherIncome { value: crate::OtherIncomeDocument::DisasterRelief(value) } => {
                (&value.source_recipient, &value.source_issuer)
            }
            Self::K1 { value: crate::K1Document::K1Partnership(value) } => {
                (&value.form.partner, &value.form.partnership)
            }
            Self::K1 { value: crate::K1Document::K1Scorp(value) } => {
                (&value.form.shareholder, &value.form.corporation)
            }
            Self::K1 { value: crate::K1Document::K1ScorpActivities(value) } => {
                (&value.shareholder, &value.corporation)
            }
            Self::K1 { value: crate::K1Document::K1EstateTrust(value) } => {
                (&value.form.beneficiary, &value.form.estate_trust)
            }
            Self::K1 { value: crate::K1Document::K1PartnershipActivities(value) } => {
                (&value.partner, &value.partnership)
            }
            Self::CanceledDebt { value: crate::CanceledDebtDocument::Cancel1099C(value) } => {
                (&value.debtor, &value.creditor)
            }
            Self::CanceledDebt { value: crate::CanceledDebtDocument::Acquisition1099A(value) } => {
                (&value.borrower, &value.lender)
            }
            Self::Farming(value) => (&value.recipient, &value.payer),
            Self::Gambling(value) => (&value.winner, &value.payer),
            Self::Record { .. } => return None,
        };
        let mut names = PrintedDocumentNames {
            recipient: recipient.clone(),
            issuer: Some(issuer.clone()),
            ..Default::default()
        };
        match self {
            Self::OtherIncome { value: crate::OtherIncomeDocument::Ltc1099(value) } => {
                names.insured_party = Some(value.insured_party.clone())
            }
            Self::K1 { value: crate::K1Document::K1EstateTrust(value) } => {
                names.fiduciary = Some(value.form.fiduciary.clone())
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::Form3921(value) } => {
                names.transfer_corporation = Some(value.corporation.clone())
            }
            _ => {}
        }
        Some(names)
    }
    pub(crate) fn set_printed_names(&mut self, names: &PrintedDocumentNames) {
        match self {
            Self::Wage { value: crate::WageDocument::W2(value) } => {
                value.employee = names.recipient.clone();
                value.employer = names.issuer.clone().unwrap_or_default();
            }
            Self::Wage { value: crate::WageDocument::Form8919(value) } => {
                value.source_recipient = names.recipient.clone();
                value.source_issuer = names.issuer.clone().unwrap_or_default();
            }
            Self::InterestDividend { value: crate::InterestDividendDocument::Int1099(value) } => {
                value.recipient = names.recipient.clone();
                value.payer = names.issuer.clone().unwrap_or_default();
            }
            Self::InterestDividend { value: crate::InterestDividendDocument::Div1099(value) } => {
                value.recipient = names.recipient.clone();
                value.payer = names.issuer.clone().unwrap_or_default();
            }
            Self::InterestDividend { value: crate::InterestDividendDocument::Oid1099(value) } => {
                value.recipient = names.recipient.clone();
                value.payer = names.issuer.clone().unwrap_or_default();
            }
            Self::InterestDividend { value: crate::InterestDividendDocument::Form2439(value) } => {
                value.shareholder = names.recipient.clone();
                value.ric_or_reit = names.issuer.clone().unwrap_or_default();
            }
            Self::InterestDividend { value: crate::InterestDividendDocument::Form8818(value) } => {
                value.name = names.recipient.name.clone();
            }
            Self::InterestDividend {
                value: crate::InterestDividendDocument::SavingsBonds(value),
            } => {
                value.form.taxpayer = names.recipient.clone();
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099B(value) } => {
                value.recipient = names.recipient.clone();
                value.payer = names.issuer.clone().unwrap_or_default();
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099DA(value) } => {
                value.recipient = names.recipient.clone();
                value.filer = names.issuer.clone().unwrap_or_default();
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::Sale1099B(value) } => {
                value.recipient = names.recipient.clone();
                value.payer = names.issuer.clone().unwrap_or_default();
            }
            Self::InvestmentSale {
                value: crate::InvestmentSaleDocument::Contracts1099B(value),
            } => {
                value.recipient = names.recipient.clone();
                value.payer = names.issuer.clone().unwrap_or_default();
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099S(value) } => {
                value.transferor = names.recipient.clone();
                value.filer = names.issuer.clone().unwrap_or_default();
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::Form3921(value) } => {
                value.corporation = names.transfer_corporation.clone().unwrap_or_default();
                value.employee = names.recipient.clone();
                value.transferor = names.issuer.clone().unwrap_or_default();
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::IsoShareSale(value) } => {
                value.source_recipient = names.recipient.clone();
                value.source_issuer = names.issuer.clone().unwrap_or_default();
            }
            Self::InvestmentSale {
                value: crate::InvestmentSaleDocument::EmployeeShareSale(value),
            } => {
                value.source_recipient = names.recipient.clone();
                value.source_issuer = names.issuer.clone().unwrap_or_default();
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::EsppShareSale(value) } => {
                value.source_recipient = names.recipient.clone();
                value.source_issuer = names.issuer.clone().unwrap_or_default();
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::NuaShareSale(value) } => {
                value.source_recipient = names.recipient.clone();
                value.source_issuer = names.issuer.clone().unwrap_or_default();
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::QsbsSale(value) } => {
                value.source_recipient = names.recipient.clone();
                value.source_issuer = names.issuer.clone().unwrap_or_default();
            }
            // The fund and its EIN stay as the accepted record holds them.
            Self::InvestmentSale {
                value: crate::InvestmentSaleDocument::OpportunityFund(value),
            } => value.source_recipient = names.recipient.clone(),
            Self::Retirement { value: crate::RetirementDocument::R1099(value) } => {
                value.recipient = names.recipient.clone();
                value.payer = names.issuer.clone().unwrap_or_default();
            }
            Self::Retirement { value: crate::RetirementDocument::Rrb1099R(value) } => {
                value.recipient = names.recipient.clone();
                value.issuer = names.issuer.clone().unwrap_or_default();
            }
            Self::Retirement { value: crate::RetirementDocument::Ssa1099(value) } => {
                value.recipient = names.recipient.clone();
                value.issuer = names.issuer.clone().unwrap_or_default();
            }
            Self::Retirement { value: crate::RetirementDocument::Rrb1099(value) } => {
                value.recipient.name = names.recipient.name.clone();
                value.recipient.address = names.recipient.address.clone();
                value.recipient_id = names.recipient.federal_id.clone();
                value.issuer = names.issuer.clone().unwrap_or_default();
            }
            Self::InvestmentSale { value: crate::InvestmentSaleDocument::Form3922(value) } => {
                value.employee = names.recipient.clone();
                value.corporation = names.issuer.clone().unwrap_or_default();
            }
            Self::SelfEmployment { value: crate::SelfEmploymentDocument::Form1099Nec(value) } => {
                value.recipient = names.recipient.clone();
                value.payer = names.issuer.clone().unwrap_or_default();
            }
            Self::SelfEmployment { value: crate::SelfEmploymentDocument::Form1099K(value) } => {
                value.recipient = names.recipient.clone();
                value.payer = names.issuer.clone().unwrap_or_default();
            }
            Self::GovernmentPayment { value, .. } => {
                value.recipient = names.recipient.clone();
                value.payer = names.issuer.clone().unwrap_or_default();
            }
            Self::OtherIncome { value: crate::OtherIncomeDocument::Form1099Misc(value) } => {
                value.recipient = names.recipient.clone();
                value.payer = names.issuer.clone().unwrap_or_default();
            }
            Self::OtherIncome { value: crate::OtherIncomeDocument::Q1099(value) } => {
                value.recipient = names.recipient.clone();
                value.payer_trustee = names.issuer.clone().unwrap_or_default();
            }
            Self::OtherIncome { value: crate::OtherIncomeDocument::Sa1099(value) } => {
                value.recipient = names.recipient.clone();
                value.trustee_payer = names.issuer.clone().unwrap_or_default();
            }
            Self::OtherIncome { value: crate::OtherIncomeDocument::HsaTrusteeStatement(value) } => {
                value.recipient = names.recipient.clone();
                value.trustee_payer = names.issuer.clone().unwrap_or_default();
            }
            Self::OtherIncome { value: crate::OtherIncomeDocument::Ltc1099(value) } => {
                value.insured_party = names.insured_party.clone().unwrap_or_default();
                value.recipient = names.recipient.clone();
                value.payer = names.issuer.clone().unwrap_or_default();
            }
            Self::OtherIncome { value: crate::OtherIncomeDocument::Qa1099(value) } => {
                value.recipient = names.recipient.clone();
                value.payer = names.issuer.clone().unwrap_or_default();
            }
            Self::OtherIncome { value: crate::OtherIncomeDocument::WildfireRelief(value) } => {
                value.source_recipient = names.recipient.clone();
                value.source_issuer = names.issuer.clone().unwrap_or_default();
            }
            Self::OtherIncome { value: crate::OtherIncomeDocument::DisasterRelief(value) } => {
                value.source_recipient = names.recipient.clone();
                value.source_issuer = names.issuer.clone().unwrap_or_default();
            }
            Self::K1 { value: crate::K1Document::K1Partnership(value) } => {
                value.form.partner = names.recipient.clone();
                value.form.partnership = names.issuer.clone().unwrap_or_default();
            }
            Self::K1 { value: crate::K1Document::K1Scorp(value) } => {
                value.form.shareholder = names.recipient.clone();
                value.form.corporation = names.issuer.clone().unwrap_or_default();
            }
            Self::K1 { value: crate::K1Document::K1ScorpActivities(value) } => {
                value.shareholder = names.recipient.clone();
                value.corporation = names.issuer.clone().unwrap_or_default();
            }
            Self::K1 { value: crate::K1Document::K1EstateTrust(value) } => {
                value.form.fiduciary = names.fiduciary.clone().unwrap_or_default();
                value.form.beneficiary = names.recipient.clone();
                value.form.estate_trust = names.issuer.clone().unwrap_or_default();
            }
            Self::K1 { value: crate::K1Document::K1PartnershipActivities(value) } => {
                value.partner = names.recipient.clone();
                value.partnership = names.issuer.clone().unwrap_or_default();
            }
            Self::CanceledDebt { value: crate::CanceledDebtDocument::Cancel1099C(value) } => {
                value.debtor = names.recipient.clone();
                value.creditor = names.issuer.clone().unwrap_or_default();
            }
            Self::CanceledDebt { value: crate::CanceledDebtDocument::Acquisition1099A(value) } => {
                value.borrower = names.recipient.clone();
                value.lender = names.issuer.clone().unwrap_or_default();
            }
            Self::Farming(value) => {
                value.recipient = names.recipient.clone();
                value.payer = names.issuer.clone().unwrap_or_default();
            }
            Self::Gambling(value) => {
                value.winner = names.recipient.clone();
                value.payer = names.issuer.clone().unwrap_or_default();
            }
            Self::Record { .. } => {}
        }
    }
}

impl AcceptedPartySource for crate::FederalIncomeEntry {
    fn accepted_parties(&self) -> Option<PrintedDocumentNames> {
        if let crate::FederalIncome::Partnership { data } = &self.income
            && let Some(source) = data.source.as_deref()
        {
            return Some(source.printed_names());
        }
        if let crate::FederalIncome::Scorp { data } = &self.income {
            return Some(data.printed_names());
        }
        if let crate::FederalIncome::Broker { data } = &self.income {
            return Some(data.printed_names());
        }
        self.printed.clone()
    }
    fn set_document_recipient(&mut self, recipient: Assignment) -> bool {
        if let crate::FederalIncome::Broker { data } = &mut self.income {
            recipient.apply_to(&mut data.assigned, &mut data.owner_source, &mut data.joint_share);
            self.printed = Some(data.printed_names());
            return true;
        }
        if let crate::FederalIncome::Partnership { data } = &mut self.income
            && let Some(source) = data.source.as_deref_mut()
        {
            recipient.apply_to(
                &mut source.assigned,
                &mut source.owner_source,
                &mut source.joint_share,
            );
            self.printed = Some(source.printed_names());
            return true;
        }
        if let crate::FederalIncome::Scorp { data } = &mut self.income {
            recipient.apply_to(&mut data.assigned, &mut data.owner_source, &mut data.joint_share);
            self.printed = Some(data.printed_names());
            return true;
        }
        let Some(parties) = &mut self.printed else {
            return false;
        };
        parties.set_assignment(recipient);
        true
    }
}

impl crate::FederalIncomeEntry {
    pub(crate) fn printed_names(&self) -> Option<&PrintedDocumentNames> {
        self.printed.as_ref()
    }

    // Read accepted ownership without materializing the source's printed parties.
    pub(crate) fn accepted_assignment(&self) -> Assignment {
        if let crate::FederalIncome::Partnership { data } = &self.income
            && let Some(source) = data.source.as_deref()
        {
            return Assignment {
                owner: source.assigned,
                owner_source: source.owner_source,
                joint_share: source.joint_share,
            };
        }
        let (owner, owner_source, joint_share) = match &self.income {
            crate::FederalIncome::Scorp { data } => {
                (data.assigned, data.owner_source, data.joint_share)
            }
            crate::FederalIncome::Broker { data } => {
                (data.assigned, data.owner_source, data.joint_share)
            }
            _ => {
                return self.printed_names().map(PrintedDocumentNames::assignment).unwrap_or_else(
                    || Assignment {
                        owner: match self.owner.tax_owner() {
                            Some(crate::Owner::Taxpayer) => DocumentRecipient::Taxpayer,
                            Some(crate::Owner::Spouse) => DocumentRecipient::Spouse,
                            None => DocumentRecipient::Unknown,
                        },
                        ..Assignment::default()
                    },
                );
            }
        };
        Assignment { owner, owner_source, joint_share }
    }
}

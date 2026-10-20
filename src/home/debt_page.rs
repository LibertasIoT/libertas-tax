// Coverage: T3-982 Canceled debt topic page types (2026-09-27). The Form 1099-C kind
//   (IncomeDocument::Cancel1099C) maps onto
//   FederalIncome::PersonalDebt (PersonalDebtCancellation) and the Form 1099-A kind
//   (IncomeDocument::Acquisition1099A) onto FederalIncome::PropertyAcquisition, both ways
//   (record / from_record; reopening a saved page gives the same page). The creditor's or
//   lender's name is the entry label. Printed boxes in box order: 1099-C boxes 1, 2, 5 and 7 (the
//   PropertyTaken fold: None exactly when box 7 is blank); 1099-A boxes 1, 2, 4, 5. Filer facts
//   stay apart: what the property was (TransferredProperty) and why the debt is or is not taxable
//   (CanceledDebtChoice). Each CanceledDebtChoice branch is its stored treatment; Something
//   else writes PersonalDebtTreatment::Unsupported with the named situation, refused by name in
//   debt.rs. Rules on both sides (Cancel1099C::rule on save, same keys):
//   DEBT_LIABILITIES_BELOW_CANCELED, DEBT_PRICE_REDUCTION_ABOVE_BASIS,
//   DEBT_PRICE_REDUCTION_ABOVE_LIABILITIES, DEBT_SALE_BEFORE_REDUCTION,
//   DEBT_PRINCIPAL_BELOW_CANCELED, DEBT_ELECTION_ABOVE_CANCELED. Tests: src/tests/home/interview.rs
//   canceled_debt_*.
use crate::{
    DebtInsolvency, DebtProperty, DebtPropertySale, FarmDebtExclusion, PersonalBankruptcy,
    PersonalDebtCancellation, PersonalDebtTreatment, PersonalInsolvency, PropertyAcquisition,
    PropertyTransfer, RealPropertyDebtExclusion, StudentDischargeReason, TransferredProperty,
    UnsupportedDebt,
};
use alloc::{string::String, vec::Vec};
use libertas_macros::*;

/// Form 1099-C
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the 1099-C exactly. Check its tax year against the supplied year; report a mismatch. Return the creditor's name as printed. Boxes 1 and 2 must show values; report a blank one as an error. Return the box 1 date and the box 2 amount as printed. Return true for box 5 only when it is checked. Use None when box 7 is blank; otherwise use Taken with the box 7 fair market value. Report illegible or conflicting values as errors. Do not infer what the property was, whether the canceled debt is income, insolvency, bankruptcy, a shared debt, or other facts absent from the 1099-C. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct Cancel1099C {
    /// Creditor's name
    /// Titles this entry.
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    pub creditor: String,
    /// Box 1 date of identifiable event
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub date: u32,
    /// Box 2 amount of debt discharged
    /// If someone else was also liable and got a form for the whole debt, enter only your share.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amount: i64,
    /// Box 5 you were personally liable for repayment of the debt
    #[libertas_llm_doc_field]
    #[libertas_default(true)]
    pub liable: bool,
    /// Box 7 fair market value of property
    #[libertas_llm_doc_field]
    #[libertas_default(None)]
    pub property: PropertyTaken,
    /// Is the canceled debt income?
    #[libertas_default(Taxable)]
    pub treatment: CanceledDebtChoice,
}

/// Box 7 fair market value of property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum PropertyTaken {
    /// Box 7 is blank
    None,
    /// Box 7 shows a value
    /// The creditor took property that secured the debt.
    Taken {
        /// Box 7 fair market value of property
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        value: i64,
        /// What the property was
        property: TransferredProperty,
    },
}

/// Is the canceled debt income?
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CanceledDebtChoice {
    /// Yes, it is income
    /// A personal debt: you were solvent, not in bankruptcy, and nothing below applies.
    Taxable,
    /// No, you were insolvent
    /// Your debts were more than the value of everything you owned immediately before the cancellation. Choosing this confirms the snapshots and the property list, completed after the rest of your 2026 return.
    Insolvent {
        /// Total liabilities immediately before the cancellation
        /// Include this debt and every other debt you owed, with your share of joint debts.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        liabilities: i64,
        /// Fair market value of everything you owned immediately before
        /// Include cash, retirement accounts, exempt assets and property that secured the debt.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        assets: i64,
        /// Tax basis of your property plus money immediately after
        /// Not its value: what you paid, adjusted for depreciation and improvements.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        basis_and_cash_after: i64,
        /// Alternative minimum tax basis plus money immediately after
        /// Leave empty when it equals the regular basis.
        /// [NullText]
        /// AMT basis plus money equals the regular amount
        /// [SetText]
        /// Different AMT basis plus money is
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default($null)]
        amt_basis_and_cash_after: Option<i64>,
        /// It was a business, farm or rental debt
        /// Any part above your insolvency is then business income, which is not calculated.
        #[libertas_default(false)]
        business_debt: bool,
        /// Property you hold on January 1, 2027
        /// Every asset except money, once each, with its adjusted basis. The excluded debt reduces its basis.
        /// ----
        /// Property
        #[libertas_unordered]
        property: Vec<DebtProperty>,
        /// Amount you elect to apply first to depreciable property
        /// Form 982 line 5. Leave zero unless you make this election.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        depreciable_first: i64,
    },
    /// No, discharged in a chapter 12 or 13 bankruptcy
    /// The court or its approved plan discharged the debt. Choosing this confirms the discharge, the snapshots and the property list.
    Bankruptcy {
        /// Tax basis of your property plus money immediately after
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        basis_and_cash_after: i64,
        /// Alternative minimum tax basis plus money immediately after
        /// Leave empty when it equals the regular basis.
        /// [NullText]
        /// AMT basis plus money equals the regular amount
        /// [SetText]
        /// Different AMT basis plus money is
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default($null)]
        amt_basis_and_cash_after: Option<i64>,
        /// Liabilities remaining immediately after the discharge
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        liabilities_after: i64,
        /// Property you hold on January 1, 2027
        /// Every asset except money, once each, with its adjusted basis.
        /// ----
        /// Property
        #[libertas_unordered]
        property: Vec<DebtProperty>,
        /// Amount you elect to apply first to depreciable property
        /// Form 982 line 5. Leave zero unless you make this election.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        depreciable_first: i64,
    },
    /// No, a qualifying student-loan discharge
    /// Choosing this confirms the servicer's final discharge determination.
    StudentLoan {
        /// Reason in the final discharge determination
        reason: StudentDischargeReason,
    },
    /// No, a gift or inheritance
    /// The creditor forgave the debt as a genuine gift or through an estate, not as a business write-off.
    Gift,
    /// No, the seller reduced the price of property you bought
    /// The original seller of personal-use property you still owned reduced the debt you owed for it, while you were solvent.
    SellerReduction {
        /// Property's adjusted basis immediately before the reduction
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        basis: i64,
        /// Fair market value of everything you owned immediately before
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        assets: i64,
        /// Total liabilities immediately before, including this debt
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        liabilities: i64,
        /// Sold later in 2026
        /// Leave empty if you still owned it at the end of 2026.
        /// [NullText]
        /// Still owned the property at the end of 2026
        /// [SetText]
        /// The later 2026 sale is
        sale: Option<DebtPropertySale>,
    },
    /// No, qualified farm debt
    /// Debt from operating your farm, canceled by a lender in the business of lending or a government agency, with at least half your 2023–2025 gross receipts from farming. Choosing this confirms those facts.
    FarmDebt {
        /// Were you insolvent immediately before?
        #[libertas_default(Solvent)]
        solvency: DebtSolvency,
        /// Property you hold on January 1, 2027
        /// Every asset except money, once each, with its adjusted basis.
        /// ----
        /// Property
        #[libertas_unordered]
        property: Vec<DebtProperty>,
        /// Amount you elect to apply first to depreciable property
        /// Form 982 line 5. Leave zero unless you make this election.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        depreciable_first: i64,
    },
    /// No, qualified real property business debt
    /// A loan to buy, build or improve real property used in your business, secured by it, that you elect to exclude. Choosing this confirms the election.
    RealPropertyBusiness {
        /// Were you insolvent immediately before?
        #[libertas_default(Solvent)]
        solvency: DebtSolvency,
        /// Outstanding principal of this debt immediately before
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        principal: i64,
        /// Fair market value of the property securing it
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        value: i64,
        /// Other qualifying business debt secured by that property
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        other_secured: i64,
        /// Adjusted basis of all your depreciable real property immediately before
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        depreciable_basis: i64,
        /// Property you hold on January 1, 2027
        /// Every asset except money, once each, with its adjusted basis. Mark the building that secured the debt.
        /// ----
        /// Property
        #[libertas_unordered]
        property: Vec<DebtProperty>,
        /// Amount you elect to apply first to depreciable property
        /// Form 982 line 5, for a part excluded because you were insolvent. Leave zero unless you make this election.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        depreciable_first: i64,
    },
    /// Something else applies
    SomethingElse {
        /// Situation
        situation: UnsupportedDebt,
    },
}

/// Were you insolvent immediately before?
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum DebtSolvency {
    /// No
    Solvent,
    /// Yes
    /// The insolvency exclusion applies first.
    Insolvent {
        /// Total liabilities immediately before the cancellation
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        liabilities: i64,
        /// Fair market value of everything you owned immediately before
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        assets: i64,
        /// Tax basis of your property plus money immediately after
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        basis_and_cash_after: i64,
        /// Alternative minimum tax basis plus money immediately after
        /// Leave empty when it equals the regular basis.
        /// [NullText]
        /// AMT basis plus money equals the regular amount
        /// [SetText]
        /// Different AMT basis plus money is
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default($null)]
        amt_basis_and_cash_after: Option<i64>,
    },
}
impl DebtSolvency {
    fn record(&self) -> Option<DebtInsolvency> {
        match self {
            Self::Solvent => None,
            Self::Insolvent {
                liabilities,
                assets,
                basis_and_cash_after,
                amt_basis_and_cash_after,
            } => Some(DebtInsolvency {
                liabilities: *liabilities,
                assets: *assets,
                basis_and_cash_after: *basis_and_cash_after,
                amt_basis_and_cash_after: *amt_basis_and_cash_after,
            }),
        }
    }
    fn from_record(value: &Option<DebtInsolvency>) -> Self {
        match value {
            None => Self::Solvent,
            Some(v) => Self::Insolvent {
                liabilities: v.liabilities,
                assets: v.assets,
                basis_and_cash_after: v.basis_and_cash_after,
                amt_basis_and_cash_after: v.amt_basis_and_cash_after,
            },
        }
    }
    fn liabilities(&self) -> Option<i64> {
        match self {
            Self::Solvent => None,
            Self::Insolvent { liabilities, .. } => Some(*liabilities),
        }
    }
}

impl Cancel1099C {
    // The client rules' server twins, in declaration order.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        let amount = self.amount;
        let t = &self.treatment;
        let solvency = match t {
            CanceledDebtChoice::FarmDebt { solvency, .. }
            | CanceledDebtChoice::RealPropertyBusiness { solvency, .. } => solvency.liabilities(),
            CanceledDebtChoice::Insolvent { liabilities, .. } => Some(*liabilities),
            _ => None,
        };
        if solvency.is_some_and(|v| v < amount) {
            let issue = crate::validation::ValidationIssue::new("DEBT_LIABILITIES_BELOW_CANCELED")
                .at_field("liabilities");
            return Some(match t {
                CanceledDebtChoice::FarmDebt { .. }
                | CanceledDebtChoice::RealPropertyBusiness { .. } => {
                    issue.in_field("solvency").in_field("treatment")
                }
                _ => issue.in_field("treatment"),
            });
        }
        match t {
            CanceledDebtChoice::SellerReduction { basis, .. } if *basis < amount => {
                return Some(
                    crate::validation::ValidationIssue::new("DEBT_PRICE_REDUCTION_ABOVE_BASIS")
                        .at_field("basis")
                        .in_field("treatment"),
                );
            }
            CanceledDebtChoice::SellerReduction { liabilities, .. } if *liabilities < amount => {
                return Some(
                    crate::validation::ValidationIssue::new(
                        "DEBT_PRICE_REDUCTION_ABOVE_LIABILITIES",
                    )
                    .at_field("liabilities")
                    .in_field("treatment"),
                );
            }
            CanceledDebtChoice::SellerReduction { sale: Some(sale), .. }
                if sale.date < self.date =>
            {
                return Some(
                    crate::validation::ValidationIssue::new("DEBT_SALE_BEFORE_REDUCTION")
                        .at_field("date")
                        .in_field("sale")
                        .in_field("treatment"),
                );
            }
            CanceledDebtChoice::RealPropertyBusiness { principal, .. } if *principal < amount => {
                return Some(
                    crate::validation::ValidationIssue::new("DEBT_PRINCIPAL_BELOW_CANCELED")
                        .at_field("principal")
                        .in_field("treatment"),
                );
            }
            _ => (),
        }
        match t {
            CanceledDebtChoice::Insolvent { depreciable_first, .. }
            | CanceledDebtChoice::Bankruptcy { depreciable_first, .. }
            | CanceledDebtChoice::FarmDebt { depreciable_first, .. }
            | CanceledDebtChoice::RealPropertyBusiness { depreciable_first, .. }
                if *depreciable_first > amount =>
            {
                Some(
                    crate::validation::ValidationIssue::new("DEBT_ELECTION_ABOVE_CANCELED")
                        .at_field("depreciable_first")
                        .in_field("treatment"),
                )
            }
            _ => None,
        }
    }
    pub(crate) fn record(&self) -> PersonalDebtCancellation {
        let treatment = match &self.treatment {
            CanceledDebtChoice::Taxable => PersonalDebtTreatment::Taxable,
            CanceledDebtChoice::Insolvent {
                liabilities,
                assets,
                basis_and_cash_after,
                amt_basis_and_cash_after,
                business_debt,
                property,
                depreciable_first,
            } => PersonalDebtTreatment::Insolvent {
                data: PersonalInsolvency {
                    liabilities: *liabilities,
                    assets: *assets,
                    basis_and_cash_after: *basis_and_cash_after,
                    amt_basis_and_cash_after: *amt_basis_and_cash_after,
                    property: property.clone(),
                    depreciable_first: *depreciable_first,
                    business_debt: *business_debt,
                },
            },
            CanceledDebtChoice::Bankruptcy {
                basis_and_cash_after,
                amt_basis_and_cash_after,
                liabilities_after,
                property,
                depreciable_first,
            } => PersonalDebtTreatment::Bankruptcy {
                data: PersonalBankruptcy {
                    basis_and_cash_after: *basis_and_cash_after,
                    amt_basis_and_cash_after: *amt_basis_and_cash_after,
                    liabilities_after: *liabilities_after,
                    property: property.clone(),
                    depreciable_first: *depreciable_first,
                },
            },
            CanceledDebtChoice::StudentLoan { reason } => {
                PersonalDebtTreatment::StudentDischarge { reason: *reason }
            }
            CanceledDebtChoice::Gift => PersonalDebtTreatment::GiftOrBequest,
            CanceledDebtChoice::SellerReduction { basis, assets, liabilities, sale } => {
                PersonalDebtTreatment::PriceReduction {
                    basis: *basis,
                    assets: *assets,
                    liabilities: *liabilities,
                    sale: sale.clone(),
                }
            }
            CanceledDebtChoice::FarmDebt { solvency, property, depreciable_first } => {
                PersonalDebtTreatment::FarmDebt {
                    data: FarmDebtExclusion {
                        insolvency: solvency.record(),
                        property: property.clone(),
                        depreciable_first: *depreciable_first,
                    },
                }
            }
            CanceledDebtChoice::RealPropertyBusiness {
                solvency,
                principal,
                value,
                other_secured,
                depreciable_basis,
                property,
                depreciable_first,
            } => PersonalDebtTreatment::RealPropertyDebt {
                data: RealPropertyDebtExclusion {
                    insolvency: solvency.record(),
                    principal: *principal,
                    value: *value,
                    other_secured: *other_secured,
                    depreciable_basis: *depreciable_basis,
                    property: property.clone(),
                    depreciable_first: *depreciable_first,
                },
            },
            CanceledDebtChoice::SomethingElse { situation } => {
                PersonalDebtTreatment::Unsupported { situation: *situation }
            }
        };
        PersonalDebtCancellation {
            date: self.date,
            amount: self.amount,
            treatment,
            personally_liable: self.liable,
            transfer: match &self.property {
                PropertyTaken::None => None,
                PropertyTaken::Taken { value, property } => {
                    Some(PropertyTransfer { value: *value, property: property.clone() })
                }
            },
        }
    }
    pub(crate) fn from_record(creditor: &str, data: &PersonalDebtCancellation) -> Option<Self> {
        let treatment = match &data.treatment {
            PersonalDebtTreatment::Taxable => CanceledDebtChoice::Taxable,
            PersonalDebtTreatment::Insolvent { data: v } => CanceledDebtChoice::Insolvent {
                liabilities: v.liabilities,
                assets: v.assets,
                basis_and_cash_after: v.basis_and_cash_after,
                amt_basis_and_cash_after: v.amt_basis_and_cash_after,
                business_debt: v.business_debt,
                property: v.property.clone(),
                depreciable_first: v.depreciable_first,
            },
            PersonalDebtTreatment::Bankruptcy { data: v } => CanceledDebtChoice::Bankruptcy {
                basis_and_cash_after: v.basis_and_cash_after,
                amt_basis_and_cash_after: v.amt_basis_and_cash_after,
                liabilities_after: v.liabilities_after,
                property: v.property.clone(),
                depreciable_first: v.depreciable_first,
            },
            PersonalDebtTreatment::StudentDischarge { reason } => {
                CanceledDebtChoice::StudentLoan { reason: *reason }
            }
            PersonalDebtTreatment::GiftOrBequest => CanceledDebtChoice::Gift,
            PersonalDebtTreatment::PriceReduction { basis, assets, liabilities, sale } => {
                CanceledDebtChoice::SellerReduction {
                    basis: *basis,
                    assets: *assets,
                    liabilities: *liabilities,
                    sale: sale.clone(),
                }
            }
            PersonalDebtTreatment::FarmDebt { data: v } => CanceledDebtChoice::FarmDebt {
                solvency: DebtSolvency::from_record(&v.insolvency),
                property: v.property.clone(),
                depreciable_first: v.depreciable_first,
            },
            PersonalDebtTreatment::RealPropertyDebt { data: v } => {
                CanceledDebtChoice::RealPropertyBusiness {
                    solvency: DebtSolvency::from_record(&v.insolvency),
                    principal: v.principal,
                    value: v.value,
                    other_secured: v.other_secured,
                    depreciable_basis: v.depreciable_basis,
                    property: v.property.clone(),
                    depreciable_first: v.depreciable_first,
                }
            }
            PersonalDebtTreatment::Unsupported { situation } => {
                CanceledDebtChoice::SomethingElse { situation: *situation }
            }
        };
        Some(Self {
            creditor: creditor.into(),
            date: data.date,
            amount: data.amount,
            liable: data.personally_liable,
            property: match &data.transfer {
                None => PropertyTaken::None,
                Some(v) => PropertyTaken::Taken { value: v.value, property: v.property.clone() },
            },
            treatment,
        })
    }
}

/// Form 1099-A
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the 1099-A exactly. Check its tax year against the supplied year; report a mismatch. Return the lender's name as printed. Boxes 1, 2 and 4 must show values; report a blank one as an error. Return the box 1 date and the box 2 and box 4 amounts as printed. Return true for box 5 only when it is checked. Report illegible or conflicting values as errors. Do not infer what the property was or other facts absent from the 1099-A. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct Acquisition1099A {
    /// Lender's name
    /// Titles this entry.
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    pub lender: String,
    /// Box 1 date of lender's acquisition or knowledge of abandonment
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub date: u32,
    /// Box 2 balance of principal outstanding
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub balance: i64,
    /// Box 4 fair market value of property
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub value: i64,
    /// Box 5 you were personally liable for repayment of the debt
    #[libertas_llm_doc_field]
    #[libertas_default(true)]
    pub liable: bool,
    /// What the property was
    pub property: AcquiredProperty,
}
impl Acquisition1099A {
    pub(crate) fn record(&self) -> PropertyAcquisition {
        let (property, crop) = match &self.property {
            AcquiredProperty::CccCrop { loan } => (TransferredProperty::Business, Some(*loan)),
            AcquiredProperty::Property { kind } => (kind.clone(), None),
        };
        PropertyAcquisition {
            date: self.date,
            balance: self.balance,
            value: self.value,
            personally_liable: self.liable,
            property,
            crop,
        }
    }
    pub(crate) fn from_record(lender: &str, data: &PropertyAcquisition) -> Self {
        Self {
            lender: lender.into(),
            date: data.date,
            balance: data.balance,
            value: data.value,
            liable: data.personally_liable,
            property: match data.crop {
                Some(loan) => AcquiredProperty::CccCrop { loan },
                None => AcquiredProperty::Property { kind: data.property.clone() },
            },
        }
    }
}

/// What the property was
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum AcquiredProperty {
    /// A crop forfeited to the Commodity Credit Corporation
    /// The crop pledged for one of the CCC loans entered on your farm or farm rental. The loan is counted there, not again here.
    CccCrop {
        /// Forfeited loan
        /// A loan marked forfeited in 2026 whose principal equals box 2. A farm entered on this same page is chosen after you save it.
        #[libertas_enum_source("$.ccc_lots")]
        loan: i64,
    },
    /// Other property
    Property {
        /// What the property was
        kind: TransferredProperty,
    },
}

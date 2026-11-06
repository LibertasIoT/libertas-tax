//! Page vocabulary of the reference cases beyond wages and children: income documents entered
//! through a topic page, loans and a home office. Amounts are whole dollars.
extern crate std;
use super::interview::{document, r1099};
use super::*;
use alloc::string::String;

#[derive(serde::Deserialize)]
#[serde(tag = "doc", rename_all = "snake_case")]
pub(super) enum Doc {
    Interest {
        taxable: i64,
        #[serde(default)]
        early_penalty: i64,
    },
    // Net benefits (box 5); `railroad` is an RRB-1099.
    Ssa {
        benefits: i64,
        #[serde(default)]
        railroad: bool,
    },
    Pension(Pension),
    // An early Roth IRA distribution (box 7a J) with the history it is ordered against
    // (Pub. 590-B: regular contributions, then conversions oldest first, taxable part first).
    RothDistribution(RothDistribution),
}

#[derive(serde::Deserialize)]
pub(super) struct Pension {
    // Box 7a as printed, such as "7" or "1D".
    printed: String,
    gross: i64,
    taxable: i64,
    #[serde(default)]
    withholding: i64,
    // Box 10, the amount allocable to an in-plan Roth rollover.
    #[serde(default)]
    box_10: i64,
    // The part of box 10 that was not taxable when rolled in (the chart's Nontaxable column).
    box_10_basis: Option<i64>,
    #[serde(default)]
    ira: bool,
    #[serde(default)]
    spouse: bool,
    // The date employment with the plan's employer ended, claimed as the exception for a
    // separation in or after the year of age 55 (section 72(t)(2)(A)(v); Pub. 575).
    separated: Option<u32>,
    // Dollars rolled over to an IRA (a kept distribution when absent).
    rolled_over: Option<i64>,
    #[serde(default)]
    method: Method,
}

#[derive(serde::Deserialize)]
pub(super) struct RothDistribution {
    gross: i64,
    // Regular Roth contributions made before 2026, and conversions `(year, taxable, nontaxable)`.
    regular: i64,
    conversions: Vec<(u32, i64, i64)>,
    first_year: u32,
}

#[derive(serde::Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum Method {
    #[default]
    BoxTwoA,
    // Pub. 575 Simplified Method Worksheet A (Tables 1 and 2 give the payment count from the
    // annuitant's age, or the combined ages of the annuitant and the survivor).
    Simplified {
        cost: i64,
        months: u32,
        ages: Vec<u32>,
        started: u32,
        // Monthly exclusion and cost recovered tax free in earlier years (a worksheet carried forward).
        prior: Option<(i64, i64)>,
    },
    // Pub. 575, qualified plan withdrawal before the annuity starting date: the tax-free part
    // is the distribution times the investment in the contract over the account balance.
    PlanWithdrawal {
        value: i64,
        investment: i64,
    },
    // Pub. 575, nonqualified plan withdrawal before the annuity starting date: earnings come
    // first, so the taxable part is the smaller of the payment and cash value less investment.
    AnnuityWithdrawal {
        value: i64,
        investment: i64,
    },
}

pub(super) fn section(name: &str) -> TaxSection {
    match name {
        "wages" => TaxSection::Wages,
        "interest_dividends" => TaxSection::InterestDividends,
        "retirement" => TaxSection::Retirement,
        other => panic!("unknown topic {other}"),
    }
}

fn pension(p: &Pension) -> IncomeDocumentItem {
    let c = |v: i64| v * 100;
    let kept = Disposition::Kept;
    let what_happened = match p.rolled_over {
        Some(amount) => {
            Disposition::RolledOver {
                amount: c(amount),
                // A designated Roth account distribution can be rolled over only to a Roth IRA.
                into: if p.printed.contains('B') {
                    crate::RolloverDestination::Roth
                } else {
                    crate::RolloverDestination::Ira
                },
            }
        }
        None => kept,
    };
    let code = if p.printed.starts_with('1') {
        let exception = match p.separated {
            Some(separated) => EarlyException::Applies {
                distributed: 20260801,
                facts: crate::RetirementException::Separation {
                    separated,
                    work: crate::SeparationWork::Ordinary,
                },
                second: None,
                repaid: None,
            },
            None => EarlyException::None,
        };
        DistributionCode::Early { what_happened, exception }
    } else if p.printed.starts_with('7') {
        DistributionCode::Normal { what_happened }
    } else if p.printed == "G" {
        DistributionCode::DirectRollover { to: RolloverTarget::Roth }
    } else if p.printed == "8" {
        DistributionCode::Corrective { excess_deferral: 0 }
    } else {
        panic!("unsupported box 7a {}", p.printed)
    };
    let mut r = printed_as(r1099(c(p.gross), code, p.ira), &p.printed);
    r.source_mut().form.taxable = Some(c(p.taxable));
    r.source_mut().form.withholding = Some(c(p.withholding));
    r.source_mut().form.box_10 = Some(c(p.box_10));
    r.source_mut().box_10_basis =
        p.box_10_basis.map(|amount| crate::InPlanRothBasis::Known { amount: c(amount) });
    // A source worksheet supplies the taxable amount; the synthetic payer has not determined it.
    r.source_mut().form.taxable_not_determined = !matches!(&p.method, Method::BoxTwoA);
    r.source_mut().taxable_method = Some(match &p.method {
        Method::BoxTwoA => TaxableMethod::BoxTwoA,
        Method::Simplified { cost, months, ages, started, prior } => TaxableMethod::Simplified {
            data: crate::PensionBasis {
                earlier_months: 0,
                changes: vec![],
                share: None,
                started: *started,
                cost: Some(c(*cost)),
                months: *months,
                history: match prior {
                    Some((monthly, recovered)) => crate::PensionHistory::Prior {
                        monthly: c(*monthly),
                        recovered: c(*recovered),
                    },
                    None => crate::PensionHistory::New {
                        duration: match ages[..] {
                            [age] => crate::PensionDuration::Single { age },
                            [age, survivor_age] => {
                                crate::PensionDuration::Joint { age, survivor_age }
                            }
                            _ => panic!("one or two ages"),
                        },
                    },
                },
                eligible: true,
            },
        },
        Method::PlanWithdrawal { value, investment } => TaxableMethod::PlanWithdrawal {
            data: crate::PlanDistribution {
                value: c(*value),
                investment: c(*investment),
                eligible: true,
            },
        },
        Method::AnnuityWithdrawal { value, investment } => TaxableMethod::AnnuityWithdrawal {
            data: crate::AnnuityDistribution {
                loan_pledge: false,
                value: c(*value),
                investment: c(*investment),
                related: vec![],
                eligible: true,
            },
        },
    });
    let mut item = document(
        0,
        IncomeSource::Retirement {
            value: crate::RetirementDocument::R1099(retirement_document(r)),
        },
    );
    if p.spouse {
        item.owner = crate::DocumentOwner::Spouses;
    }
    item
}

fn roth_distribution(r: &RothDistribution) -> Vec<IncomeDocumentItem> {
    let c = |v: i64| v * 100;
    let code = DistributionCode::RothIra {
        sequence: None,
        rolled_over: 0,
        roth: RothIraCode::Early { distributed: 20260710, exception: None, repaid: None },
    };
    let mut row = printed_as(r1099(c(r.gross), code, true), "J");
    row.source_mut().form.taxable = Some(0);
    vec![
        document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(row)),
            },
        ),
        document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::RothHistory(RothIraHistory {
                    first_year: r.first_year,
                    amounts: Some(RothOrdering {
                        regular: c(r.regular),
                        conversions: r
                            .conversions
                            .iter()
                            .map(|&(year, taxable, nontaxable)| RothConversion {
                                year,
                                taxable: c(taxable),
                                nontaxable: c(nontaxable),
                            })
                            .collect(),
                    }),
                }),
            },
        ),
    ]
}

pub(super) fn documents(docs: &[Doc]) -> Vec<IncomeDocumentItem> {
    docs.iter()
        .flat_map(|doc| match doc {
            Doc::RothDistribution(r) => roth_distribution(r),
            doc => vec![single(doc)],
        })
        .collect()
}

fn single(doc: &Doc) -> IncomeDocumentItem {
    let c = |v: i64| v * 100;
    match doc {
        Doc::Interest { taxable, early_penalty } => document(
            0,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(interest_document(Int1099 {
                    payer: crate::PrintedParty::from_name("Synthetic bank".into()),
                    recipient: Default::default(),
                    adjustments: None,
                    nominee: None,
                    taxable: c(*taxable),
                    more: IntMore { early_penalty: c(*early_penalty), ..IntMore::default() },
                    after_death: None,
                    state_details: Default::default(),
                })),
            },
        ),
        Doc::Ssa { benefits, railroad } => document(
            0,
            IncomeSource::Retirement {
                value: if *railroad {
                    crate::RetirementDocument::Rrb1099(crate::rrb_document::Rrb1099Paper {
                        year: 2026,
                        benefits_paid: c(*benefits).max(0),
                        benefits_repaid: c(*benefits).saturating_neg().max(0),
                        benefits: c(*benefits),
                        ..Default::default()
                    })
                } else {
                    crate::RetirementDocument::Ssa1099(ssa_document(ssa_1099_net(c(*benefits), 0)))
                },
            },
        ),
        Doc::Pension(p) => pension(p),
        Doc::RothDistribution(_) => unreachable!(),
    }
}

// One principal-place home office used for the last business entered, by month:
// `(month, days, square feet)` rows; months not listed have no qualifying use.
pub(super) fn office_site(business_id: i64, months: &[(u32, u32, u32)]) -> HomeOfficeSite {
    let month = |m: u32| {
        let (days, area) = months.iter().find(|r| r.0 == m).map_or((0, 0), |r| (r.1, r.2));
        OfficeSimplifiedMonth {
            month: m,
            exclusive_days: days,
            exclusive_area: area,
            shared_days: 0,
            shared_area: 0,
        }
    };
    HomeOfficeSite {
        id: 0,
        address: "Synthetic home".into(),
        home_area: 2_000,
        paid_by: HomeCostOwners::Taxpayer,
        mortgage_loans: vec![],
        tenure: HomeTenure::Rented,
        claims: vec![HomeOfficeClaim {
            owner: Owner::Taxpayer,
            starts: 20260101,
            ends: 20261231,
            use_kind: OfficeQualifiedUse::PrincipalPlace,
            method: OfficeMethod::Simplified { months: (1..=12).map(month).collect() },
            activities: vec![OfficeActivity {
                activity: business_id,
                gross_share: 100.0,
                expense_period_share: 100.0,
                attribution_reason: String::new(),
                deduction_parts: 1,
            }],
            prior_operating: 0,
            prior_depreciation: 0,
            prior_casualty: 0,
        }],
        costs: vec![],
        casualties: vec![],
    }
}

// A Form 1098 loan secured by the main home.
#[derive(serde::Deserialize)]
pub(super) struct Loan {
    interest: i64,
    principal: i64,
    originated: u32,
    #[serde(default)]
    points: LoanPoints,
    // Refinanced earlier debt `(amount, date incurred, dollars used to improve the home)`.
    refinance: Option<(i64, u32, i64)>,
}

#[derive(serde::Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum LoanPoints {
    #[default]
    None,
    Amortized {
        amount: i64,
        term_months: u32,
        ended: bool,
    },
    Improvement {
        amount: i64,
        term_months: u32,
    },
}

impl Loan {
    pub(super) fn record(&self) -> Mortgage1098 {
        let c = |v: i64| v * 100;
        let mut loan = Mortgage1098 {
            lender: Default::default(),
            borrower: Default::default(),
            id: 0,
            interest: c(self.interest),
            principal: c(self.principal),
            originated: self.originated,
            refund: 0,
            insurance: 0,
            points: match self.points {
                LoanPoints::None => 0,
                LoanPoints::Amortized { amount, .. } | LoanPoints::Improvement { amount, .. } => {
                    c(amount)
                }
            },
            more: MortgageMore::default(),
        };
        if let Some((refinanced, incurred, improvement)) = self.refinance {
            loan.more.details = MortgageDetails::FirstLender(MortgageFirstLender {
                purpose: crate::MortgageUse::Refinance(crate::MortgageRefinance {
                    refinanced: c(refinanced),
                    earlier_incurred: incurred,
                    // A 30-year term, the usual one, when the source gives none.
                    earlier_term_ends: Some(incurred + 300_000),
                    improvement: c(improvement),
                    other: 0,
                }),
                ..MortgageFirstLender::default()
            });
        }
        loan
    }
    pub(super) fn points_answer(&self) -> crate::MortgagePointTreatment {
        match self.points {
            LoanPoints::None => crate::MortgagePointTreatment::None,
            LoanPoints::Amortized { term_months, ended, .. } => {
                crate::MortgagePointTreatment::Amortized { term_months, ended }
            }
            LoanPoints::Improvement { term_months, .. } => {
                crate::MortgagePointTreatment::Improvement { term_months }
            }
        }
    }
}

use crate::FederalResult;
use crate::line_emission::line_cents;
use crate::validation::ValidationIssue;
use alloc::vec::Vec;
use libertas_macros::*;

/// Commodity Credit Corporation marketing-assistance loans
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmCcc {
    /// Treatment of all loans received in 2026
    /// Use the same method for all of this taxpayer's farming businesses. An existing section 77 income election continues; changing back to loan treatment requires separate accounting-change treatment not implemented here.
    pub method: CccMethod,
    /// Pledged commodity lots
    /// Enter each source-identified collateral lot or released portion once. A partially redeemed loan uses separate records for each released portion and the remaining pledged collateral; copy each portion's attributable original principal from the loan/release records, never repeat the entire loan principal for each part. Include every portion of a current-year advance. A warehouse release must correspond to the complete warehouse receipt identified by the CCC. Include prior loans redeemed or forfeited now, previously redeemed commodities sold now, and surplus payments received or required deficiency payments made now after an earlier CCC disposition. Do not also enter these sales, market gains or forfeitures as other farm receipts. Separate interest and ordinary fees belong in business expenses, not principal.
    #[libertas_size(min = 1)]
    pub lots: Vec<CccLot>,
    /// Federal income tax withheld in 2026
    /// Actual current-year withholding from payment statements, even when loan principal is not income. Exclude amounts already entered elsewhere.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Producer, retained production cost and original section 77 history
    /// The loan-detail election statement remains attached to the tax return; the App does not file it. Missing commodity basis, an accrual income election or a changed method needs its actual source worksheet, rather than discarding the loan income.
    pub source: crate::CccSourceFacts,
}

/// Current-year CCC loan method
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CccMethod {
    /// Treat advances as loans
    Loan,
    /// Include advances in income under section 77
    Income,
}

/// Pledged crop lot
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CccLot {
    /// Accepted record identity
    /// Names this loan on a Form 1099-A, whatever the order of the loan list; the App assigns it when the farm is saved.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Loan receipt and prior tax treatment
    pub origin: CccOrigin,
    /// Original principal attributable to this lot or collateral portion
    /// Use the loan/release record amount before redemption payments, not the current unpaid balance. Do not invent a proportional allocation when the source records do not establish it.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub principal: i64,
    /// Redemption, forfeiture or outstanding collateral
    pub disposition: CccDisposition,
}

/// Loan origin
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CccOrigin {
    /// Received in 2026
    Current,
    /// Received before 2026 and excluded from income
    PriorExcluded,
    /// Received before 2026 and included in income
    PriorIncluded,
}

/// Commodity disposition
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CccDisposition {
    /// Still pledged at year end
    Outstanding,
    /// Crop disposed of by the CCC
    /// Use the CCC settlement records, not a sale you made after redeeming the crop. No deduction arises merely because the CCC sells for less than the loan and releases you from the shortfall. Enter a required deficiency payment separately; do not also deduct it in ordinary business expenses.
    Forfeited {
        /// Calendar year the crop was forfeited or disposed of by the CCC
        // Tax structure: this source year must be no later than the 2026 return year.
        #[libertas_number(min = 1900, max = 2026)]
        #[libertas_default(2026)]
        year: u32,
        /// Surplus above the loan recognized from the CCC during 2026
        /// Additional CCC proceeds received this year for cash accounting or recognized this year under the accrual all-events rules, even if forfeiture occurred earlier. Exclude original loan proceeds and amounts recognized in another year. Zero means no current surplus income.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        surplus: i64,
        /// Required loan deficiency actually paid during 2026
        /// Use only the principal shortfall you were legally required to pay and paid this year after CCC disposition, for a loan included in income under section 77. Zero if unpaid or released. Exclude interest, fees, redemption payments, voluntary payments and amounts deducted in another year. Loan-method deficiencies still need their own worksheet.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        deficiency: i64,
    },
    /// Redeemed commodity
    Redeemed {
        /// Calendar year redeemed
        // Tax structure: this source year must be no later than the 2026 return year.
        #[libertas_number(min = 1900, max = 2026)]
        #[libertas_default(2026)]
        year: u32,
        /// Principal redemption payment
        /// Exclude interest and fees. A purchaser's principal payment on your behalf also counts. Market gain is original principal minus this amount.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        paid: i64,
        /// 2026 sale proceeds recognized for the entire lot
        /// Cash farms enter proceeds received; accrual farms enter recognized sales, including receivables. Leave absent if still held; zero means a completed sale with no proceeds. Include buyer-paid loan settlement in gross proceeds. No prior-year sale belongs here. A partial sale needs a separate source-identified portion and attributable principal/redemption records; this proceeds amount must cover that entire portion.
        /// [NullText]
        /// The entire crop portion is still held
        /// [SetText]
        /// Recognized sale proceeds for the entire crop portion are
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        sale: Option<i64>,
    },
}
impl CccLot {
    fn included(&self, method: &CccMethod) -> bool {
        self.origin == CccOrigin::PriorIncluded
            || self.origin == CccOrigin::Current && *method == CccMethod::Income
    }
    fn amounts(&self, method: &CccMethod) -> [i64; 9] {
        let included = self.included(method);
        let advance =
            if self.origin == CccOrigin::Current && included { self.principal } else { 0 };
        let mut market = 0;
        let mut taxable_market = 0;
        let mut forfeiture = 0;
        let mut surplus = 0;
        let mut sale_net = 0;
        let mut retained_basis = if included { self.principal } else { 0 };
        match self.disposition {
            CccDisposition::Outstanding => (),
            CccDisposition::Forfeited { year, surplus: paid, .. } => {
                forfeiture = if !included && year == 2026 { self.principal } else { 0 };
                // Section 77 cash receipts follow 26 CFR 1.77-2(a)(1). Loan-method
                // accrual records supply recognized proceeds under their accounting
                // rules; neither route repeats an earlier advance/forfeiture.
                surplus = paid;
                retained_basis = 0;
            }
            CccDisposition::Redeemed { year, paid, sale } => {
                if year == 2026 {
                    market = self.principal - paid;
                    taxable_market = if included { 0 } else { market };
                }
                retained_basis = if included { paid } else { 0 };
                if let Some(proceeds) = sale {
                    sale_net = proceeds - retained_basis;
                    retained_basis = 0;
                }
            }
        }
        [
            advance,
            market,
            taxable_market,
            forfeiture,
            sale_net,
            retained_basis,
            advance + taxable_market + forfeiture + sale_net + surplus,
            surplus,
            if matches!(self.disposition, CccDisposition::Forfeited { year: 2026, .. }) {
                self.principal
            } else {
                0
            },
        ]
    }
}
impl FarmCcc {
    pub(crate) fn source_issue(&self) -> Option<ValidationIssue> {
        use crate::CccElectionHistory::*;
        if !(0..=100_000_000_000).contains(&self.source.unrecovered_production_cost) {
            return Some(
                ValidationIssue::new("CCC_PRODUCTION_BASIS_SOURCE")
                    .at_field("unrecovered_production_cost")
                    .in_field("source"),
            );
        }
        let (year, evidence, field) = match &self.source.history {
            NoIncomeElection => return None,
            IncomeElection { year, statement } => (*year, statement, "statement"),
            ChangedMethod { year, authority } => (*year, authority, "authority"),
        };
        let field = if !(1900..=crate::year_constants::TAX_YEAR).contains(&year) {
            Some("year")
        } else if evidence.trim().is_empty() {
            Some(field)
        } else {
            None
        };
        field.map(|field| {
            ValidationIssue::new("CCC_ELECTION_HISTORY_SOURCE")
                .at_field(field)
                .in_field("history")
                .in_field("source")
        })
    }
    pub(crate) fn source_rule(&self) -> Option<&'static str> {
        self.source_issue().map(|issue| issue.key)
    }
    pub(crate) fn valid(&self) -> bool {
        self.source_rule().is_none()
            && !self.lots.is_empty()
            && self.lots.len() <= 400
            && (0..=100000000000).contains(&self.withholding)
            && self.lots.iter().all(|v| {
                (1..=100000000000).contains(&v.principal)
                    && match v.disposition {
                        CccDisposition::Forfeited { year, surplus, deficiency } => {
                            (1900..=2026).contains(&year)
                                && (0..=100000000000).contains(&surplus)
                                && (0..=100000000000).contains(&deficiency)
                        }
                        CccDisposition::Redeemed { year, paid, sale } => {
                            (1900..=2026).contains(&year)
                                && (0..=100000000000).contains(&paid)
                                && sale.is_none_or(|x| (0..=100000000000).contains(&x))
                        }
                        _ => true,
                    }
            })
    }
    pub(crate) fn deficiency(&self) -> i64 {
        self.lots
            .iter()
            .map(|lot| match lot.disposition {
                CccDisposition::Forfeited { deficiency, .. } => deficiency,
                _ => 0,
            })
            .sum()
    }
    pub(crate) fn income(&self) -> i64 {
        self.lots.iter().map(|v| v.amounts(&self.method)[6]).sum()
    }
    // Whether the lot at this position was forfeited to the CCC in 2026 (the
    // lot a Form 1099-A names): Some(true) when its loan proceeds are
    // reported under section 77 (this year or earlier), Some(false) when
    // they were never income, None when it is not such a lot.
    pub(crate) fn forfeited_current_year(&self, index: usize) -> Option<bool> {
        let lot = self.lots.get(index)?;
        matches!(lot.disposition, CccDisposition::Forfeited { year: 2026, .. })
            .then(|| lot.included(&self.method))
    }
    // The position, identity and principal of each lot forfeited in 2026.
    pub(crate) fn forfeited_lots(&self) -> impl Iterator<Item = (usize, i64, i64)> + '_ {
        self.lots.iter().enumerate().filter_map(|(index, lot)| {
            matches!(lot.disposition, CccDisposition::Forfeited { year: 2026, .. }).then_some((
                index,
                lot.id,
                lot.principal,
            ))
        })
    }
    // The cash-method lot amounts a Form 4835 prints beside the loan lines:
    // (market gain, its taxable part, the net of a redeemed crop's sale,
    // surplus above the loan). Line 1 takes the sale net, line 3 the market
    // gain and line 6 the surplus.
    pub(crate) fn cash_parts(&self) -> (i64, i64, i64, i64) {
        self.lots.iter().fold((0, 0, 0, 0), |t, lot| {
            let amounts = lot.amounts(&self.method);
            (t.0 + amounts[1], t.1 + amounts[2], t.2 + amounts[4], t.3 + amounts[7])
        })
    }
    // Schedule F Part I placement of the cash-method lot amounts (Pub 225
    // chapter 3): [line 4a market gain, its line 4b taxable part, line 1a
    // proceeds and 1b basis of a redeemed commodity whose loan was included in
    // income (basis = the redemption payment), line 2 proceeds of one whose
    // loan was not (a raised product with no basis)].
    pub(crate) fn part_one(&self) -> [i64; 5] {
        let mut out = [0; 5];
        for lot in &self.lots {
            let amounts = lot.amounts(&self.method);
            out[0] += amounts[1];
            out[1] += amounts[2];
            if let CccDisposition::Redeemed { paid, sale: Some(proceeds), .. } = lot.disposition {
                if lot.included(&self.method) {
                    out[2] += proceeds;
                    out[3] += paid;
                } else {
                    out[4] += proceeds;
                }
            }
        }
        out
    }
    pub(crate) fn report(
        &self,
        cash: bool,
        id: i64,
        form: crate::farm::FarmForm,
        r: &mut FederalResult,
    ) {
        let input = form.input("ccc");
        use crate::CccElectionHistory::*;
        crate::require_localized!(
            r,
            self.source.producer_owned_raised && self.source.unrecovered_production_cost == 0,
            &crate::IssueContext::Section(crate::TaxSection::Farming),
            "CCC commodity source",
            "ISSUE_CCC_COMMODITY_BASIS_SOURCE",
            "The CCC commodity needs its actual ownership and unrecovered production-cost basis. The zero-basis raised-crop calculation cannot replace those facts or discard known loan income.",
        );
        crate::require_localized!(
            r,
            match &self.source.history {
                NoIncomeElection => {
                    self.method == CccMethod::Loan
                        && !self.lots.iter().any(|lot| lot.origin == CccOrigin::PriorIncluded)
                }
                IncomeElection { year, .. } => {
                    self.method == CccMethod::Income
                        && (*year < crate::year_constants::TAX_YEAR
                            || !self.lots.iter().any(|lot| lot.origin == CccOrigin::PriorIncluded))
                }
                ChangedMethod { .. } => false,
            },
            &crate::IssueContext::Section(crate::TaxSection::Farming),
            "CCC original method history",
            "ISSUE_CCC_ELECTION_TRANSITION_SOURCE",
            "Original section77 history must agree with the current method and earlier included loans. A documented method change needs its actual transition and commodity-basis owner; changing the method answer alone cannot compute it.",
        );

        crate::require_localized!(
            r,
            cash || self.method == CccMethod::Loan,
            &crate::IssueContext::Section(crate::TaxSection::Farming),
            "CCC accrual inventory",
            "ISSUE_AN_ACCRUAL_SECTION_77_ELECTION_NEEDS_COORDINATION_BETWEEN_THE_FARM_4B04A5BF",
            "An accrual section 77 election needs coordination between the farm's inventory valuation and the loan's income-inclusion and redemption basis. Aggregate opening/closing inventory alone does not establish that adjustment. Keep the recorded election; do not change an established method to bypass this unfinished worksheet.",
        );
        crate::require_localized!(
            r,
            self.method == CccMethod::Income
                || !self.lots.iter().any(|v| v.origin == CccOrigin::PriorIncluded),
            &crate::IssueContext::Section(crate::TaxSection::Farming),
            "CCC loan election",
            "ISSUE_AN_EXISTING_INCOME_ELECTION_CONTINUES_FOR_CURRENT_LOANS_CHANGING_TO_BA0302A2",
            "An existing income election continues for current loans; changing to loan treatment requires an accounting-change worksheet.",
        );
        crate::require_localized!(
            r,
            self.lots.iter().all(|v| match v.disposition {
                CccDisposition::Forfeited { year, .. } => {
                    v.origin != CccOrigin::Current || year == 2026
                }
                CccDisposition::Redeemed { year, paid, .. } => {
                    paid <= v.principal && (v.origin != CccOrigin::Current || year == 2026)
                }
                _ => true,
            }),
            &crate::IssueContext::Section(crate::TaxSection::Farming),
            "CCC loan settlement",
            "ISSUE_REDEMPTION_PRINCIPAL_CANNOT_EXCEED_THE_ORIGINAL_PRINCIPAL_REDEMPTIO_7D5465D5",
            "Redemption principal cannot exceed the original principal; redemption or forfeiture cannot precede a current-year loan. Enter interest and fees separately.",
        );
        crate::require_localized!(
            r,
            self.lots.iter().all(|lot| match lot.disposition {
                CccDisposition::Forfeited { deficiency, .. } => {
                    deficiency <= lot.principal && (deficiency == 0 || lot.included(&self.method))
                }
                _ => true,
            }),
            &crate::IssueContext::Section(crate::TaxSection::Farming),
            "CCC paid deficiency",
            "ISSUE_A_SUPPORTED_DEFICIENCY_MUST_BE_REQUIRED_AND_ACTUALLY_PAID_FOR_A_SEC_708807CC",
            "A supported deficiency must be required and actually paid for a section 77 income-included loan, and cannot exceed principal. Loan-method deficiencies need a separate worksheet; released or unpaid shortfalls are not deductions.",
        );
        line_cents(
            r,
            &alloc::format!("farm_{id}_ccc_paid_deficiency"),
            self.deficiency(),
            &[&input],
            "https://www.govinfo.gov/content/pkg/CFR-2025-title26-vol2/pdf/CFR-2025-title26-vol2-sec1-77-2.pdf",
            msg!("EXPLAIN_FARM_N_CCC_PAID_DEFICIENCY"),
        );
        // The paid deficiency is a Schedule F Part II amount (lines 32a-32f).
        if self.deficiency() != 0 {
            line_cents(
                r,
                &alloc::format!("farm_{id}_expense_line32_ccc_deficiency"),
                self.deficiency(),
                &[&input],
                "https://www.govinfo.gov/content/pkg/CFR-2025-title26-vol2/pdf/CFR-2025-title26-vol2-sec1-77-2.pdf",
                msg!("EXPLAIN_FARM_N_EXPENSE_LINE32_CCC_DEFICIENCY"),
            );
        }
        let mut totals = [0; 9];
        for lot in &self.lots {
            for (total, amount) in totals.iter_mut().zip(lot.amounts(&self.method)) {
                *total += amount;
            }
        }
        for (i, (name, note)) in [
            ("advances", msg!("EXPLAIN_FARM_N_CCC_ADVANCES")),
            ("market_gain", msg!("EXPLAIN_FARM_N_CCC_MARKET_GAIN")),
            ("taxable_market_gain", msg!("EXPLAIN_FARM_N_CCC_TAXABLE_MARKET_GAIN")),
            ("forfeiture", msg!("EXPLAIN_FARM_N_CCC_FORFEITURE")),
            ("sales_net", msg!("EXPLAIN_FARM_N_CCC_SALES_NET")),
            ("retained_basis", msg!("EXPLAIN_FARM_N_CCC_RETAINED_BASIS")),
            ("income", msg!("EXPLAIN_FARM_N_CCC_INCOME")),
            ("surplus", msg!("EXPLAIN_FARM_N_CCC_SURPLUS")),
            ("forfeiture_reported", msg!("EXPLAIN_FARM_N_CCC_FORFEITURE_REPORTED")),
        ]
        .into_iter()
        .enumerate()
        {
            if !cash && name == "retained_basis" {
                continue; // Accrual crop basis belongs to the farm inventory, not the zero cash-basis calculation.
            }
            let note = if !cash && name == "sales_net" {
                msg!("EXPLAIN_FARM_N_CCC_SALES_NET_CASH")
            } else {
                note
            };
            // Schedule F line 5a (Part III 40a) is the advances reported
            // under the section 77 election; 5b (40b) the forfeited
            // principal (T2-45p, question F14). Form 4835 prints them as
            // lines 4a and 4b, with 4c the part of the forfeiture not
            // included in income earlier.
            let form_line = match (form, name) {
                (crate::farm::FarmForm::ScheduleF, "advances") => {
                    Some(if cash { "5a" } else { "40a" })
                }
                (crate::farm::FarmForm::ScheduleF, "forfeiture_reported") => {
                    Some(if cash { "5b" } else { "40b" })
                }
                (crate::farm::FarmForm::Form4835, "advances") => Some("4a"),
                (crate::farm::FarmForm::Form4835, "forfeiture_reported") => Some("4b"),
                (crate::farm::FarmForm::Form4835, "forfeiture") => Some("4c"),
                _ => None,
            }
            .map(|line| crate::FormLine::new(form.tax_form(), line));
            crate::line_emission::push_line(
                r,
                form_line,
                &alloc::format!("farm_{id}_ccc_{name}"),
                totals[i],
                &[&input],
                if name == "surplus" {
                    "https://www.govinfo.gov/content/pkg/CFR-2025-title26-vol2/pdf/CFR-2025-title26-vol2-sec1-77-2.pdf"
                } else {
                    "https://www.irs.gov/publications/p225"
                },
                note,
            );
        }
        line_cents(
            r,
            &alloc::format!("farm_{id}_ccc_withholding"),
            self.withholding,
            &[&input],
            "https://www.irs.gov/publications/p225",
            msg!("EXPLAIN_FARM_N_CCC_WITHHOLDING"),
        );
        if self.method == CccMethod::Income {
            r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] "CCC section 77 treatment: retain the loan-detail election statement and individual commodity basis records. The election applies to all current and later loans unless an authorized method change applies; the App does not file it.", crate::Message::new("NOTE_CCC_SECTION_77_TREATMENT_RETAIN_THE_LOAN_DETAIL_ELECTION_STATEME_BA3452BD", alloc::vec![])));
        }
    }
}

// Community-property projections preserve accepted source identity and facts.
crate::community::divide_record!(FarmCcc {
    amounts: [method, lots, withholding, source],
    gains: [],
    facts: [],
});

crate::community::divide_record!(CccLot {
    amounts: [origin, principal, disposition],
    gains: [],
    facts: [id],
});

crate::community::divide_enum!(CccDisposition {
    records: [
        Forfeited { amounts: [surplus, deficiency], facts: [year] },
        Redeemed { amounts: [paid, sale], facts: [year] }
    ],
    units: [Outstanding],
    tuples: [],
});

crate::community::divide_enum!(CccOrigin {
    records: [],
    units: [Current, PriorExcluded, PriorIncluded],
    tuples: [],
});

crate::community::divide_enum!(CccMethod { records: [], units: [Loan, Income], tuples: [] });

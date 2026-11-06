use crate::kernel::carry::{Amount, Cents};
// Coverage: Schedule K-1 (Form 1065) boxes 11, 13 and 20 and (Form 1120-S) boxes 10, 12 and 17 —
//   Partial (unit B9-1, 2026-10-10; i1065sk1 2025 final (no 2026 revision), i1120ssk 2026 draft of
//   9/18/26, Pub 541, Pub 925, i6198 Nov. 2025, f8582 2026 draft, i8960, i4797, i1040sd,
//   26 U.S.C. 165(d), 170, 469(g), 731, 741, 751, 1256, 1411(c)(4), Reg. 1.199A-3, 1.704-1(d),
//   1.1366-2, 1.469-2T).
//   Lines: one typed record per code (PartnerIncome, ScorpIncome, PartnerDeduction,
//   ScorpDeduction); each code has a K1ItemKind (its treatment) and a K1LossClass (the basis and
//   at-risk class its loss or deduction joins; k1_loss.rs OtherLoss, OtherCharity,
//   InvestmentInterest, OtherDeductions). Destinations after the limits (K1Items::flows):
//   box 11 (10) A Schedule 1 line 8z, Form 8960 line 7, Form 4952 line 4a; C Form 6781 Part I line 1
//   (section1256.rs form_6781) and Schedule D lines 4 and 11; E Schedule 1 line 8c; F and box 13 V
//   the box 1 amount; J Schedule 1 line 8z; K winnings Schedule 1 line 8b and losses Schedule A
//   line 16 at 90% (section 165(d)); Q Schedule D lines 5 and 12; R Form 4797 Part II line 11 and
//   Schedule 1 line 4; S Schedule D lines 5 and 12; box 13 (12) B, C, D, E the Schedule A section
//   170 class worksheet; H Form 4952 line 1; I Schedule E line 19 (portfolio); K the outside basis only; AB Schedule 1 line 18; AE no
//   deduction (section 67(h)); box 20 AB, AC, AD the sale of the interest; AI Schedule 2 line 13h.
//   Section 199A statement: section 199A dividends and qualified PTP items join the REIT/PTP
//   component; the section 751 gain joins the statement's QBI.
//   The sale and liquidation of the whole interest (K1Disposition): the gain is the amount
//   realized (proceeds plus the item K1 liability relief) less the ending basis, section 751
//   ordinary on Form 4797 Part II line 11 and the rest capital on Form 8949 box C or F with the
//   28% and unrecaptured section 1250 worksheets; the gain is Form 6198 activity income, which
//   releases the at-risk carries, and section 469(g)(1)(A) releases the activity's passive losses.
//   A distribution above basis is section 731 gain. Left out with WARN_K1_ITEM_LEFT_OUT (the
//   estimate proceeds; the amounts still reduce basis): the codes the App does not compute, a
//   section 108 exclusion, a section 111 exclusion, unreimbursed partner expenses, the pre-1990
//   housing credit, a carried excess business interest expense and box 20 code AQ.
//   Refused by name: box 11 codes B, D, I, L through P, ZZ and a code A loss or REMIC residual
//   interest, a code C straddle, gambling of a gambling business, box 13 code O, box 20
//   recapture, look-back interest, section 179 dispositions and the other codes of
//   PartnerK1Item::OtherInformation, cooperative section 199A items, a disposition other than a
//   sale or a cash liquidation, the Form 8960 line 5c adjustment, investment interest with an
//   at-risk limit.
//   Evidence: k1_items.rs, partnership.rs, scorp.rs, k1_loss.rs, federal_rules.rs,
//   extended_rules.rs, section1256.rs, partnership_qbi.rs, scorp_qbi.rs, k1_page.rs;
//   tests/passthrough/k1_items.rs.
//   B9-7 R9 source worksheet foundation: K1Section179Sale validates statement
//   dates/basis and computes per-property ordinary/section1231 character with
//   the personal carryover restored to basis (i4797 2025 final; Reg. 1.179-3(f)).
//   tests/k1_items k1_section179_sale_ cover F2 source amounts and boundaries.
//   R9 input adapter checkpoint: the source list is reachable from both K-1
//   pages. Each identified physical source owns a known regular reported/absent
//   view and explicit Reported/NotReported/Unavailable AMT source. Unavailable
//   survives Save and returns ISSUE_K1_SALE_AMT_UNAVAILABLE at AMT admission;
//   no absent or copied amount replaces it. The existing explicit parent
//   SameAsRegular assertion supplies only a known source convention. Each
//   system retains its twenty-statement limit; the disjoint physical union can
//   contain forty sources. Save enforces those counts and same-parent global
//   IDs; no physical correspondence is inferred from names, values or order.
//   AI imports description/dates and the regular monetary record's four printed
//   amounts, excluding accepted IDs, AMT facts and personal sold carryover.
//   Client/server rules and raw record validation cover both tax systems.
//   tests/passthrough/interview.rs k1_sale_inputs_ cover Save/reopen/restart, correction,
//   rejection without draft mutation and independent AMT depreciation.
//   unit17_k1_sale_identity_complete_editor_save_reopen_restart covers accepted
//   same-description sources, one ID across tax views, reorder/rename and
//   atomic foreign/duplicate/guessed-ID rejection; explicit disjoint40 controls
//   retain each original per-view bound and independent worksheet results.
//   tests/passthrough/k1_sale_amt.rs and unit17_k1_sale_amt_unavailable_ cover the named
//   final source boundary and complete Avro Save/reopen/restart/correction.
//   R9 loss-owner checkpoint: each property's positive income and current
//   loss enter the existing basis/risk classes separately; business-property
//   aggregates join one K-1 passive source and the common section 1231 lookback.
//   tests/passthrough/k1_sale_limits.rs cover both entity kinds, gross source allocations,
//   per-property ordinary routes, passive suspension and independent AMT.
//   Core F2/F2b returns, Form4562, QBI, NIIT and independent AMT use these
//   source owners (tests/passthrough/k1_sale_final.rs); named unsupported cases remain.
//   The 2026 draft routes ordinary code R and section 751 amounts to Part II
//   line 11; line 10 is the Part I capital transfer. Code R/section 751 and
//   capital footing: tests/passthrough/k1_sale_footing.rs; f4797 2026 draft (10/1/26).
use crate::k1_loss::{K1LossClass, LossAmounts, LossLedger};
use crate::kernel::phase;
use crate::kernel::ratio::dollars;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// Arithmetic safety: worksheet, K1Items::with_sales and K1ItemFlows retain native i64 arithmetic
// before k1_totals dollar-to-cent transfers.
// Widen only with those actual receivers, including their regular/AMT paths.
const MONEY: core::ops::RangeInclusive<i64> = 0..=100_000_000_000;
const SIGNED: core::ops::RangeInclusive<i64> = -100_000_000_000..=100_000_000_000;

/// Section 179 property the entity sold
/// From the Form 1065 box 20 code L or Form 1120-S box 17 code K statement for a fully taxable sale. Enter your shares, not the entity's totals. Use the AMT depreciation in an AMT record. Casualties, theft, installment sales and like-kind exchanges need separate worksheets.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct K1Section179Sale {
    /// Description of the property
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_size(min = 1)]
    pub description: alloc::string::String,
    /// Date acquired and placed in service
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub acquired: u32,
    /// Date sold
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    // Tax structure: this sale occurs in the current tax year.
    #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
    pub sold: u32,
    /// Regular-tax monetary view
    /// Leave absent only when this property is not reported in the regular-tax sale statement. Its AMT statement can still report the same physical property.
    /// [NullText]
    /// This property is not reported in the regular-tax sale statement
    /// [SetText]
    /// The regular-tax amounts are
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub regular: Option<K1Section179SaleAmounts>,
    /// AMT monetary source for this same property
    /// Unavailable retains an unknown AMT source. Not reported is a known statement absence; only the parent's explicit Same as regular convention supplies equal regular amounts.
    #[libertas_default(Unavailable)]
    pub alternative: K1Section179SaleAmtSource,
    /// Accepted physical sale source
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub id: i64,
}

/// Availability of the same physical property's AMT sale statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum K1Section179SaleAmtSource {
    /// The AMT sale records are unavailable
    Unavailable,
    /// This property is not reported in the separate AMT sale statement
    NotReported,
    /// The AMT sale statement reports these amounts
    Reported { amounts: K1Section179SaleAmounts },
}
impl K1Section179SaleAmtSource {
    pub(crate) fn reported_amounts(&self) -> Option<&K1Section179SaleAmounts> {
        match self {
            Self::Reported { amounts } => Some(amounts),
            _ => None,
        }
    }
    #[cfg(test)]
    pub(crate) fn reported_amounts_mut(&mut self) -> Option<&mut K1Section179SaleAmounts> {
        match self {
            Self::Reported { amounts } => Some(amounts),
            _ => None,
        }
    }
    pub(crate) fn from_known_regular(regular: &Option<K1Section179SaleAmounts>) -> Self {
        match regular {
            Some(amounts) => Self::Reported { amounts: amounts.clone() },
            None => Self::NotReported,
        }
    }
    pub(crate) fn matches_known_regular(&self, regular: &Option<K1Section179SaleAmounts>) -> bool {
        match (self, regular) {
            (Self::Reported { amounts }, Some(regular)) => amounts == regular,
            (Self::NotReported, None) => true,
            _ => false,
        }
    }
}

/// Monetary view of the same section179 sale
/// Physical description, dates and accepted source identity belong to the property row.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1Section179SaleAmounts {
    /// Your share of the gross sales price
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub price: i64,
    /// Your share of the cost or other basis plus expense of sale
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub cost: i64,
    /// Your share of depreciation excluding section179
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub depreciation: i64,
    /// Your share of section179 passed through for this property
    /// The whole amount passed through, including any part you could not deduct.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub section179: i64,
}

// i4797, worksheet for partners and S corporation shareholders; Reg.
// 1.179-3(f)(1). Whole-dollar worksheet inputs are rounded once, like the
// shared direct-disposition and loss-limit owners. These are source amounts,
// before basis, at-risk, passive and return-wide section 1231 limitations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Section179SaleResult {
    pub depreciation: i64,
    pub line4: i64,
    pub gain: i64,
    pub held_long: bool,
    pub recapture: i64,
    pub section1231: i64,
    pub ordinary: i64,
}

// Current business-property sources stay gross through basis and at-risk
// limits. Netting a sale loss against another property's gain here would
// discard both the income that increases basis and the loss to be limited.
// The entered box amount and its QBI statement remain unchanged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct BusinessProperty {
    pub gain: i64,
    pub loss: i64,
    pub unrecaptured: i64,
    pub basis_loss: i64,
    pub risk_loss: i64,
    pub passive_loss: i64,
}

impl BusinessProperty {
    pub(crate) fn of(
        source: Option<&crate::K1Section1231>,
        sales: &[Section179SaleResult],
    ) -> Option<Self> {
        let mut result = source.map_or(Self::default(), |source| Self {
            gain: source.gain(),
            loss: phase::floor(-source.net, 0),
            unrecaptured: source.unrecaptured,
            basis_loss: source.basis_loss,
            risk_loss: source.risk_loss,
            passive_loss: source.passive_loss,
        });
        for sale in sales {
            result.gain += phase::floor(sale.section1231, 0) * 100;
            result.loss += phase::floor(-sale.section1231, 0) * 100;
        }
        (result != Self::default()).then_some(result)
    }

    pub(crate) fn net(&self) -> i64 {
        self.gain - self.loss
    }
}

impl K1Section179Sale {
    // The outer absence is unknown, the inner absence is known not reported.
    // This is the one source query; consumers must retain that distinction.
    pub(crate) fn monetary_view(&self, amt: bool) -> Option<Option<&K1Section179SaleAmounts>> {
        if !amt {
            return Some(self.regular.as_ref());
        }
        match &self.alternative {
            K1Section179SaleAmtSource::Unavailable => None,
            K1Section179SaleAmtSource::NotReported => Some(None),
            K1Section179SaleAmtSource::Reported { amounts } => Some(Some(amounts)),
        }
    }
    pub(crate) fn view(&self, amt: bool) -> Result<Option<Self>, (&'static str, &'static str)> {
        let amounts = self.monetary_view(amt).ok_or((
            "ISSUE_K1_SALE_AMT_UNAVAILABLE",
            crate::fixture_english!("AMT amounts for a section 179 sale are unavailable. Enter the property's AMT statement before its AMT sale worksheet can be calculated."),
        ))?;
        let Some(amounts) = amounts else {
            return Ok(None);
        };
        let mut view = self.clone();
        view.regular = Some(amounts.clone());
        view.alternative = K1Section179SaleAmtSource::NotReported;
        Ok(Some(view))
    }
    pub(crate) fn views(
        sales: &[Self],
        amt: bool,
    ) -> Result<Vec<Self>, (&'static str, &'static str)> {
        sales
            .iter()
            .map(|sale| sale.view(amt))
            .collect::<Result<Vec<_>, _>>()
            .map(|views| views.into_iter().flatten().collect())
    }
    fn amount_views(&self) -> impl Iterator<Item = &K1Section179SaleAmounts> {
        self.regular.iter().chain(self.alternative.reported_amounts())
    }
    pub(crate) fn apply_parent_same_as_regular(&mut self) {
        if matches!(self.alternative, K1Section179SaleAmtSource::NotReported) {
            self.alternative = K1Section179SaleAmtSource::from_known_regular(&self.regular);
        }
    }
    // Relationships that span the statement list and the owner's personal
    // carryover. Both interview pages must declare the same two rules when
    // their adapters are added. A nonzero carryover
    // with multiple properties is structurally valid but its allocation is
    // unresolved, so it is refused separately by `worksheets`.
    pub(crate) fn carryover_rule(
        sales: &[Self],
        carryover: i64,
        sold: i64,
    ) -> Option<&'static str> {
        Self::carryover_issue(sales, carryover, sold).map(|issue| issue.key)
    }

    pub(crate) fn carryover_issue(
        sales: &[Self],
        carryover: i64,
        sold: i64,
    ) -> Option<crate::validation::ValidationIssue> {
        if sold > carryover {
            Some(
                crate::validation::ValidationIssue::new("K1_SALE_CARRYOVER_ABOVE_CARRYOVER")
                    .at_field("sold"),
            )
        } else if sales
            .iter()
            .try_fold(0i64, |sum, sale| {
                sum.checked_add(sale.regular.as_ref().map_or(0, |amounts| amounts.section179))
            })
            .is_none_or(|share| sold > share)
        {
            Some(
                crate::validation::ValidationIssue::new("K1_SALE_CARRYOVER_ABOVE_SHARE")
                    .at_field("sold"),
            )
        } else {
            None
        }
    }

    pub(crate) fn worksheets(
        sales: &[Self],
        carryover: i64,
        sold: i64,
    ) -> Result<Vec<Section179SaleResult>, (&'static str, &'static str)> {
        if !Self::valid_list(sales) || !MONEY.contains(&carryover) || !MONEY.contains(&sold) {
            return Err((
                "ISSUE_K1_SALE_RECORDS",
                crate::fixture_english!(
                    "Enter valid section 179 sale statements and nonnegative carryover amounts within the supported limits."
                ),
            ));
        }
        if let Some(key) = Self::carryover_rule(sales, carryover, sold) {
            return Err((
                key,
                match key {
                    "K1_SALE_CARRYOVER_ABOVE_CARRYOVER" => crate::fixture_english!(
                        "The sold-property part cannot exceed this K-1's section 179 carryover."
                    ),
                    _ => crate::fixture_english!(
                        "The sold-property carryover cannot exceed the section 179 deduction passed through for the properties sold."
                    ),
                },
            ));
        }
        if sales.len() > 1 && sold != 0 {
            return Err((
                "ISSUE_K1_SALE_CARRYOVER_SPLIT",
                crate::fixture_english!(
                    "More than one section 179 property was sold and a carryover belongs to sold property. Each property's carryover must be allocated separately; this K-1 does not calculate that allocation."
                ),
            ));
        }
        Ok(sales.iter().map(|sale| sale.worksheet(sold)).collect())
    }

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let locate = |issue: ValidationIssue, view: usize| {
            if self.regular.is_some() && view == 0 {
                issue.in_field("regular")
            } else {
                issue.in_field("amounts").in_field("alternative")
            }
        };
        if self.sold < self.acquired {
            return Some(ValidationIssue::new("K1_SALE_DATES").at_field("sold"));
        }
        if let Some((view, _)) = self.amount_views().enumerate().find(|(_, amounts)| {
            amounts
                .depreciation
                .checked_add(amounts.section179)
                .is_none_or(|total| total > amounts.cost)
        }) {
            return Some(locate(
                ValidationIssue::new("K1_SALE_BASIS_BELOW_ZERO").at_field("depreciation"),
                view,
            ));
        }
        if self.id < 0 {
            return Some(ValidationIssue::new("ISSUE_K1_SALE_RECORDS").at_field("id"));
        }
        if self.regular.is_none()
            && matches!(self.alternative, K1Section179SaleAmtSource::NotReported)
        {
            return Some(ValidationIssue::new("ISSUE_K1_SALE_RECORDS").at_field("regular"));
        }
        for (view, amounts) in self.amount_views().enumerate() {
            if let Some((field, _)) = [
                ("price", amounts.price),
                ("cost", amounts.cost),
                ("depreciation", amounts.depreciation),
                ("section179", amounts.section179),
            ]
            .into_iter()
            .find(|(_, value)| !MONEY.contains(value))
            {
                return Some(locate(
                    ValidationIssue::new("ISSUE_K1_SALE_RECORDS").at_field(field),
                    view,
                ));
            }
        }
        None
    }

    fn valid_source(&self) -> bool {
        self.source_issue().is_none()
    }

    fn source_issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let field = if self.description.is_empty() {
            Some("description")
        } else if !crate::federal_rules::valid_date(self.acquired) {
            Some("acquired")
        } else if !crate::federal_rules::valid_date(self.sold)
            || !(crate::year_constants::TAX_YEAR_START_DATE
                ..=crate::year_constants::TAX_YEAR_END_DATE)
                .contains(&self.sold)
        {
            Some("sold")
        } else {
            None
        };
        if let Some(field) = field {
            return Some(ValidationIssue::new("ISSUE_K1_SALE_RECORDS").at_field(field));
        }
        self.issue()
    }

    pub(crate) fn valid_source_counts(sales: &[Self]) -> bool {
        // Arithmetic safety: native worksheet/loss allocation and K1Items::with_sales
        // receivers consume the physical union of regular and AMT sale sources.
        sales.len() <= 40
            // Arithmetic safety: the regular worksheet/loss allocation and
            // k1_totals dollar-to-cent receivers retain native i64 totals.
            && sales.iter().filter(|sale| sale.regular.is_some()).count() <= 20
            // Arithmetic safety: the AMT view uses the same native worksheet,
            // loss allocation and dollar-to-cent receivers as the regular view.
            && sales.iter().filter(|sale| sale.alternative.reported_amounts().is_some()).count()
                <= 20
    }

    pub(crate) fn valid_source_list(sales: &[Self]) -> bool {
        Self::source_list_issue(sales).is_none()
    }

    pub(crate) fn source_list_issue(sales: &[Self]) -> Option<crate::validation::ValidationIssue> {
        if !Self::valid_source_counts(sales) {
            return Some(
                crate::validation::ValidationIssue::new("ISSUE_K1_SALE_RECORDS")
                    .at_field("section179_sales"),
            );
        }
        sales.iter().enumerate().find_map(|(index, sale)| {
            sale.source_issue().map(|issue| issue.in_list("section179_sales", index))
        })
    }

    pub(crate) fn valid_list(sales: &[Self]) -> bool {
        // Arithmetic safety: selected worksheet/loss allocations, K1Items::with_sales
        // and k1_totals dollar-to-cent transfers retain native i64 receivers.
        sales.len() <= 20 && sales.iter().all(Self::valid_source)
    }

    pub(crate) fn worksheet(&self, carryover: i64) -> Section179SaleResult {
        let amounts = self.regular.as_ref().expect("selected reported monetary view");
        let depreciation =
            dollars(amounts.depreciation) + dollars(amounts.section179) - dollars(carryover);
        let line4 = dollars(amounts.cost) - depreciation;
        let gain = dollars(amounts.price) - line4;
        // The same strict anniversary test as ordinary property in
        // asset_sale::Livestock::held_long_enough: exactly one year is short.
        let held_long = crate::dates::held_more_than_one_year(self.acquired, self.sold);
        let recapture =
            if held_long { phase::cap(phase::floor(gain, 0), [depreciation]) } else { 0 };
        Section179SaleResult {
            depreciation,
            line4,
            gain,
            held_long,
            recapture,
            section1231: if held_long { gain - recapture } else { 0 },
            ordinary: if held_long { recapture } else { gain },
        }
    }
}

/// Box 11 code
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum PartnerIncomeCode {
    /// Code A other portfolio income
    /// Income only. A portfolio loss or a REMIC residual interest belongs among the other boxes.
    PortfolioIncome,
    /// Code C section 1256 contracts
    /// The net gain or loss. Straddles on the statement belong among the other boxes.
    Section1256,
    /// Code E cancellation of debt
    DebtCancellation,
    /// Code F section 743(b) positive adjustment to the box 1 business
    /// Only when the statement shows it adjusts the box 1 trade or business.
    BasisAdjustmentIncome,
    /// Code J recoveries of tax benefit items
    TaxBenefitRecovery,
    /// Code K gambling winnings
    /// When the partnership isn't in the business of gambling.
    GamblingWinnings,
    /// Code K gambling losses
    /// The losses on the code K statement, as a positive amount.
    GamblingLosses,
    /// Code Q Fannie Mae or Freddie Mac preferred stock, held one year or less
    PreferredStockShort,
    /// Code Q Fannie Mae or Freddie Mac preferred stock, held more than one year
    PreferredStockLong,
    /// Code R specially allocated ordinary gain (loss)
    SpeciallyAllocatedOrdinary,
    /// Code S non-portfolio short-term capital gain (loss)
    NonportfolioShort,
    /// Code S non-portfolio long-term capital gain (loss)
    NonportfolioLong,
}

/// Box 11 amount
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerIncome {
    /// Code
    pub code: PartnerIncomeCode,
    /// Amount
    /// Enter a loss as a negative amount; only codes C, Q, R and S can be a loss.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub amount: i64,
}

/// Calculated box 11 amount
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerIncomeTotal {
    /// Code
    pub code: PartnerIncomeCode,
    /// Amount
    #[libertas_money("USD", 2)]
    pub amount: i64,
}

impl From<&PartnerIncome> for PartnerIncomeTotal {
    fn from(item: &PartnerIncome) -> Self {
        Self { code: item.code, amount: item.amount }
    }
}

/// Box 10 code
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ScorpIncomeCode {
    /// Code A other portfolio income
    /// Income only. A portfolio loss or a REMIC residual interest belongs among the other boxes.
    PortfolioIncome,
    /// Code C section 1256 contracts
    /// The net gain or loss. Straddles on the statement belong among the other boxes.
    Section1256,
    /// Code J recoveries of tax benefit items
    TaxBenefitRecovery,
    /// Code K gambling winnings
    /// When the corporation isn't in the business of gambling.
    GamblingWinnings,
    /// Code K gambling losses
    /// The losses on the code K statement, as a positive amount.
    GamblingLosses,
    /// Code S non-portfolio short-term capital gain (loss)
    NonportfolioShort,
    /// Code S non-portfolio long-term capital gain (loss)
    NonportfolioLong,
}

/// Box 10 amount
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpIncome {
    /// Code
    pub code: ScorpIncomeCode,
    /// Amount
    /// Enter a loss as a negative amount; only codes C and S can be a loss.
    #[libertas_money("USD", 2)]
    pub amount: i64,
}

/// Box 13 code
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum PartnerDeductionCode {
    /// Code B cash contributions (30%)
    CashThirty,
    /// Code C noncash contributions (50%)
    NoncashFifty,
    /// Code D noncash contributions (30%)
    NoncashThirty,
    /// Code E capital gain property to a 50% organization (30%)
    CapitalGainPropertyThirty,
    /// Code F capital gain property (20%)
    CapitalGainPropertyTwenty,
    /// Code G contributions (100%)
    ContributionsHundred,
    /// Code H investment interest expense
    InvestmentInterest,
    /// Code I deductions — royalty income
    RoyaltyDeductions,
    /// Code J section 59(e)(2) expenditures
    Section59e,
    /// Code K excess business interest expense
    ExcessBusinessInterest,
    /// Code L deductions — portfolio income (other)
    PortfolioOther,
    /// Code N educational assistance benefits
    EducationalAssistance,
    /// Code P preproductive period expenses
    Preproductive,
    /// Code S reforestation expense deduction
    Reforestation,
    /// Code V section 743(b) negative adjustment to the box 1 business
    /// Only when the statement shows it adjusts the box 1 trade or business.
    BasisAdjustmentDeduction,
    /// Code W soil and water conservation
    SoilWater,
    /// Code X film, television, theatrical and sound recording production expenses
    Production,
    /// Code Y expenditures for removal of barriers
    Barriers,
    /// Code Z itemized deductions
    Itemized,
    /// Code AA contributions to a capital construction fund
    ConstructionFund,
    /// Code AB penalty on early withdrawal of savings
    EarlyWithdrawalPenalty,
    /// Code AC interest expense allocated to debt-financed distributions
    DistributionInterest,
    /// Code AD interest expense on a working interest in oil or gas
    WorkingInterest,
    /// Code AE deductions — portfolio income
    PortfolioDeductions,
    /// Code ZZ other deductions
    Other,
}

/// Box 13 amount
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerDeduction {
    /// Code
    pub code: PartnerDeductionCode,
    /// Amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
}

/// Box 12 code
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ScorpDeductionCode {
    /// Code B cash contributions (30%)
    CashThirty,
    /// Code C noncash contributions (50%)
    NoncashFifty,
    /// Code D noncash contributions (30%)
    NoncashThirty,
    /// Code E capital gain property to a 50% limit organization (30%)
    CapitalGainPropertyThirty,
    /// Code F capital gain property (20%)
    CapitalGainPropertyTwenty,
    /// Code G contributions (100%)
    ContributionsHundred,
    /// Code H investment interest expense
    InvestmentInterest,
    /// Code I deductions — royalty income
    RoyaltyDeductions,
    /// Code J section 59(e)(2) expenditures
    Section59e,
    /// Code L deductions — portfolio income (other)
    PortfolioOther,
    /// Code M preproductive period expenses
    Preproductive,
    /// Code O reforestation expense deduction
    Reforestation,
    /// Code W soil and water conservation
    SoilWater,
    /// Code X film, television, live theatrical and qualified sound recording production expenditures
    Production,
    /// Code Y expenditures for removal of barriers
    Barriers,
    /// Code Z itemized deductions
    Itemized,
    /// Code AA contributions to a capital construction fund
    ConstructionFund,
    /// Code AB penalty on early withdrawal of savings
    EarlyWithdrawalPenalty,
    /// Code AC interest expense allocated to debt-financed distributions
    DistributionInterest,
    /// Code ZZ other deductions
    Other,
}

/// Box 12 amount
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpDeduction {
    /// Code
    pub code: ScorpDeductionCode,
    /// Amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
}

/// What happened to the interest in 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum K1Disposition {
    /// Still owned in full
    Kept,
    /// Sold or exchanged entirely to an unrelated buyer, all taxed this year
    Sold {
        /// Proceeds
        /// Cash plus the fair market value of property received, less selling costs. Excludes the share of the entity's debts.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        proceeds: i64,
    },
    /// The partnership paid out the whole interest in cash
    LiquidatedForCash,
    /// Another kind of disposition
    Other,
}

impl K1Disposition {
    // The whole interest left the owner's hands this year and every gain and
    // loss is recognized now (section 469(g)(1)(A)).
    pub(crate) fn ended(&self) -> bool {
        matches!(self, Self::Sold { .. } | Self::LiquidatedForCash)
    }
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Sold { proceeds } => MONEY.contains(proceeds),
            _ => true,
        }
    }
}

/// Parts of the gain on the sale of a partnership interest — box 20 codes AB, AC and AD
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct PartnerSaleParts {
    /// Section 751 gain (loss) — code AB
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub ordinary: i64,
    /// Collectibles gain — code AC
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub collectibles: i64,
    /// Unrecaptured section 1250 gain — code AD
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub unrecaptured: i64,
}

impl PartnerSaleParts {
    pub(crate) fn valid(&self) -> bool {
        SIGNED.contains(&self.ordinary)
            && MONEY.contains(&self.collectibles)
            && MONEY.contains(&self.unrecaptured)
    }
}

/// Situation this App does not calculate for a partnership K-1
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum PartnerSpecial {
    /// You contributed property to the partnership
    /// In 2026, or earlier with built-in gain or loss the partnership still tracks (item M answered Yes). This K-1 is left out of the estimate.
    PropertyContributed,
    /// Payments to you as a retiring partner or a deceased partner's successor
    /// Section 736 payments. This K-1 is left out of the estimate.
    RetiringPayments,
    /// Expenses you paid for the partnership without reimbursement
    /// Required by the partnership agreement. The deduction is left out of the estimate; the rest of the K-1 is calculated.
    UnreimbursedExpenses,
    /// Low-income housing credit (codes C or D) for an interest you acquired before 1990
    /// The credit is left out of the estimate; the rest of the K-1 is calculated.
    HousingBefore1990,
    /// Excess business interest expense from this partnership carried from an earlier year
    /// Box 13 code K amounts you could not deduct yet. Any deduction this year is left out of the estimate.
    ExcessInterestCarried,
}

impl PartnerSpecial {
    #[cfg(test)]
    pub(crate) const ALL: [Self; 5] = [
        Self::PropertyContributed,
        Self::RetiringPayments,
        Self::UnreimbursedExpenses,
        Self::HousingBefore1990,
        Self::ExcessInterestCarried,
    ];
    // The situations that leave the whole K-1 out of the estimate (answers
    // and unknowns, decision 2); the others leave only their item out.
    pub(crate) fn whole(self) -> bool {
        matches!(self, Self::PropertyContributed | Self::RetiringPayments)
    }
    // The resource that names an item-only situation in WARN_K1_ITEM_LEFT_OUT.
    pub(crate) fn item(self) -> Option<&'static str> {
        match self {
            Self::UnreimbursedExpenses => Some("K1_ITEM_UNREIMBURSED_EXPENSES"),
            Self::HousingBefore1990 => Some("K1_ITEM_HOUSING_BEFORE_1990"),
            Self::ExcessInterestCarried => Some("K1_ITEM_EXCESS_INTEREST_CARRIED"),
            Self::PropertyContributed | Self::RetiringPayments => None,
        }
    }
    pub(crate) fn valid_list(items: &[Self]) -> bool {
        Self::issue_list(items).is_none()
    }
    pub(crate) fn issue_list(items: &[Self]) -> Option<crate::validation::ValidationIssue> {
        items.iter().enumerate().find(|(index, item)| items[..*index].contains(item)).map(
            |(index, _)| {
                crate::validation::ValidationIssue::new("K1_SITUATION_TWICE")
                    .at_list_item("special", index)
            },
        )
    }
}

/// Situation this App does not calculate for an S corporation K-1
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum ScorpSpecial {
    /// The corporation's business is farming
    /// This K-1 is left out of the estimate.
    Farming,
    /// A debt between you and the corporation was forgiven
    /// This K-1 is left out of the estimate.
    DebtForgiveness,
    /// Low-income housing credit (codes C or D) for shares you acquired before 1990
    /// The credit is left out of the estimate; the rest of the K-1 is calculated.
    HousingBefore1990,
}

impl ScorpSpecial {
    #[cfg(test)]
    pub(crate) const ALL: [Self; 3] =
        [Self::Farming, Self::DebtForgiveness, Self::HousingBefore1990];
    pub(crate) fn whole(self) -> bool {
        matches!(self, Self::Farming | Self::DebtForgiveness)
    }
    pub(crate) fn item(self) -> Option<&'static str> {
        match self {
            Self::HousingBefore1990 => Some("K1_ITEM_HOUSING_BEFORE_1990"),
            Self::Farming | Self::DebtForgiveness => None,
        }
    }
    pub(crate) fn valid_list(items: &[Self]) -> bool {
        Self::issue_list(items).is_none()
    }
    pub(crate) fn issue_list(items: &[Self]) -> Option<crate::validation::ValidationIssue> {
        items.iter().enumerate().find(|(index, item)| items[..*index].contains(item)).map(
            |(index, _)| {
                crate::validation::ValidationIssue::new("K1_SITUATION_TWICE")
                    .at_list_item("special", index)
            },
        )
    }
}

// One distinct treatment of the codes of section 3 of the map. A code that
// the App does not compute is LeftOut: its amount still reduces basis and a
// notice says so (answers and unknowns, decision 1 and the inventor decision
// of 2026-10-10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum K1ItemKind {
    // Box 11 (10) code A: Schedule 1 line 8z, Form 8960 line 7, Form 4952 line 4a.
    Portfolio,
    // Code C: one Form 6781 Part I line 1 row.
    Section1256,
    // Box 11 code E: Schedule 1 line 8c.
    DebtCancellation,
    // Box 11 code F and box 13 code V: part of box 1.
    BoxOneAdjustment,
    // Code J: Schedule 1 line 8z.
    Recovery,
    // Code K winnings (line 8b) and losses (Schedule A line 16).
    GamblingWinnings,
    GamblingLosses,
    // Box 11 code Q: Schedule D lines 5 and 12.
    PreferredShort,
    PreferredLong,
    // Box 11 code R: Form 4797 Part II line 11 (2026).
    OrdinaryActivity,
    // Code S: Schedule D lines 5 and 12 of the K-1's activity.
    NonportfolioShort,
    NonportfolioLong,
    // Box 13 (12) B and D, C, E: the section 170 classes of the worksheet.
    CharityThirtyB,
    CharityFiftyA,
    CharityThirtyC,
    // Code H: Form 4952 line 1.
    InvestmentInterest,
    // Code I: Schedule E line 19; portfolio, outside Form 8582.
    RoyaltyDeductions,
    // Box 13 code K: basis only.
    Ebie,
    // Code AB: Schedule 1 line 18.
    EarlyPenalty,
    // Box 13 code AE: no deduction (section 67(h)); basis only.
    Nondeductible,
    // Everything else: left out with a notice; basis only.
    LeftOut,
}

impl K1ItemKind {
    // The explanation of the code's row, the line it reaches and the sources
    // that it reads. None for the kinds that reach no line of their own (the
    // box 1 adjustments, the basis-only codes and the left-out ones).
    pub(crate) fn row(self, code: &str) -> Option<(crate::TaxForm, &'static str, &'static str)> {
        use crate::TaxForm::*;
        let cash = code.ends_with('B');
        Some(match self {
            Self::Portfolio => (Schedule1, "8z", "EXPLAIN_K1_ITEM_PORTFOLIO"),
            Self::Recovery => (Schedule1, "8z", "EXPLAIN_K1_ITEM_RECOVERY"),
            Self::DebtCancellation => (Schedule1, "8c", "EXPLAIN_K1_ITEM_DEBT"),
            Self::GamblingWinnings => (Schedule1, "8b", "EXPLAIN_K1_ITEM_WINNINGS"),
            Self::GamblingLosses => (ScheduleA, "17a", "EXPLAIN_K1_ITEM_GAMBLING_LOSSES"),
            Self::Section1256 => (Form6781, "1", "EXPLAIN_K1_ITEM_SECTION1256"),
            Self::PreferredShort => (ScheduleD, "5", "EXPLAIN_K1_ITEM_PREFERRED"),
            Self::PreferredLong => (ScheduleD, "12", "EXPLAIN_K1_ITEM_PREFERRED"),
            Self::OrdinaryActivity => (Form4797, "11", "EXPLAIN_K1_ITEM_ORDINARY"),
            Self::NonportfolioShort => (ScheduleD, "5", "EXPLAIN_K1_ITEM_NONPORTFOLIO"),
            Self::NonportfolioLong => (ScheduleD, "12", "EXPLAIN_K1_ITEM_NONPORTFOLIO"),
            Self::CharityThirtyB => {
                (ScheduleA, if cash { "11" } else { "12" }, "EXPLAIN_K1_ITEM_CHARITY_THIRTY")
            }
            Self::CharityFiftyA => (ScheduleA, "12", "EXPLAIN_K1_ITEM_CHARITY_FIFTY"),
            Self::CharityThirtyC => (ScheduleA, "12", "EXPLAIN_K1_ITEM_CHARITY_PROPERTY"),
            Self::InvestmentInterest => (Form4952, "1", "EXPLAIN_K1_ITEM_INVESTMENT_INTEREST"),
            Self::RoyaltyDeductions => (ScheduleE, "19", "EXPLAIN_K1_ROYALTY_DEDUCTIONS"),
            Self::EarlyPenalty => (Schedule1, "18", "EXPLAIN_K1_ITEM_EARLY_PENALTY"),
            Self::BoxOneAdjustment | Self::Ebie | Self::Nondeductible | Self::LeftOut => {
                return None;
            }
        })
    }

    // Income of the K-1's activity or the K-1 itself (positive amounts):
    // basis worksheet line 4 and Form 6198 activity income.
    fn income(self) -> bool {
        matches!(
            self,
            Self::Portfolio
                | Self::Section1256
                | Self::DebtCancellation
                | Self::Recovery
                | Self::GamblingWinnings
                | Self::PreferredShort
                | Self::PreferredLong
                | Self::OrdinaryActivity
                | Self::NonportfolioShort
                | Self::NonportfolioLong
        )
    }
}

macro_rules! item_codes {
    ($code:ident, [$($member:ident => ($name:literal, $row:literal, $kind:ident, $class:expr)),+ $(,)?]) => {
        impl $code {
            // Test catalog for the declared code choices and their text.
            #[cfg(test)]
            pub(crate) const ALL: [Self; [$(Self::$member),+].len()] = [$(Self::$member),+];

            /// The treatment of the code.
            pub(crate) fn kind(self) -> K1ItemKind {
                match self { $(Self::$member => K1ItemKind::$kind),+ }
            }
            /// The class the code's loss or deduction joins in the basis and
            /// at-risk limits; None when it never does.
            pub(crate) fn class(self) -> Option<K1LossClass> {
                match self { $(Self::$member => $class),+ }
            }
            /// The printed box and code, a unique suffix of the code's row.
            pub(crate) fn row(self) -> &'static str {
                match self { $(Self::$member => $row),+ }
            }
            /// The resource that names the code in a notice.
            pub(crate) fn item(self) -> &'static str {
                match self { $(Self::$member => concat!("K1_ITEM_", $name)),+ }
            }
        }
    };
}

item_codes!(PartnerIncomeCode, [
    PortfolioIncome => ("PARTNER_INCOME_PORTFOLIO", "11A", Portfolio, None),
    Section1256 => ("PARTNER_INCOME_SECTION1256", "11C", Section1256, Some(K1LossClass::OtherLoss)),
    DebtCancellation => ("PARTNER_INCOME_DEBT", "11E", DebtCancellation, None),
    BasisAdjustmentIncome => ("PARTNER_INCOME_ADJUSTMENT", "11F", BoxOneAdjustment, None),
    TaxBenefitRecovery => ("PARTNER_INCOME_RECOVERY", "11J", Recovery, None),
    GamblingWinnings => ("PARTNER_INCOME_WINNINGS", "11K-winnings", GamblingWinnings, None),
    GamblingLosses => ("PARTNER_INCOME_LOSSES", "11K-losses", GamblingLosses, Some(K1LossClass::OtherDeductions)),
    PreferredStockShort => ("PARTNER_INCOME_PREFERRED_SHORT", "11Q-short", PreferredShort, Some(K1LossClass::OtherLoss)),
    PreferredStockLong => ("PARTNER_INCOME_PREFERRED_LONG", "11Q-long", PreferredLong, Some(K1LossClass::OtherLoss)),
    SpeciallyAllocatedOrdinary => ("PARTNER_INCOME_ORDINARY", "11R", OrdinaryActivity, Some(K1LossClass::OtherLoss)),
    NonportfolioShort => ("PARTNER_INCOME_NONPORTFOLIO_SHORT", "11S-short", NonportfolioShort, Some(K1LossClass::OtherLoss)),
    NonportfolioLong => ("PARTNER_INCOME_NONPORTFOLIO_LONG", "11S-long", NonportfolioLong, Some(K1LossClass::OtherLoss)),
]);

item_codes!(ScorpIncomeCode, [
    PortfolioIncome => ("SCORP_INCOME_PORTFOLIO", "10A", Portfolio, None),
    Section1256 => ("SCORP_INCOME_SECTION1256", "10C", Section1256, Some(K1LossClass::OtherLoss)),
    TaxBenefitRecovery => ("SCORP_INCOME_RECOVERY", "10J", Recovery, None),
    GamblingWinnings => ("SCORP_INCOME_WINNINGS", "10K-winnings", GamblingWinnings, None),
    GamblingLosses => ("SCORP_INCOME_LOSSES", "10K-losses", GamblingLosses, Some(K1LossClass::OtherDeductions)),
    NonportfolioShort => ("SCORP_INCOME_NONPORTFOLIO_SHORT", "10S-short", NonportfolioShort, Some(K1LossClass::OtherLoss)),
    NonportfolioLong => ("SCORP_INCOME_NONPORTFOLIO_LONG", "10S-long", NonportfolioLong, Some(K1LossClass::OtherLoss)),
]);

item_codes!(PartnerDeductionCode, [
    CashThirty => ("PARTNER_DEDUCTION_CASH_THIRTY", "13B", CharityThirtyB, Some(K1LossClass::OtherCharity)),
    NoncashFifty => ("PARTNER_DEDUCTION_NONCASH_FIFTY", "13C", CharityFiftyA, Some(K1LossClass::OtherCharity)),
    NoncashThirty => ("PARTNER_DEDUCTION_NONCASH_THIRTY", "13D", CharityThirtyB, Some(K1LossClass::OtherCharity)),
    CapitalGainPropertyThirty => ("PARTNER_DEDUCTION_PROPERTY_THIRTY", "13E", CharityThirtyC, Some(K1LossClass::OtherCharity)),
    CapitalGainPropertyTwenty => ("PARTNER_DEDUCTION_PROPERTY_TWENTY", "13F", LeftOut, Some(K1LossClass::OtherCharity)),
    ContributionsHundred => ("PARTNER_DEDUCTION_CONTRIBUTIONS_HUNDRED", "13G", LeftOut, Some(K1LossClass::OtherCharity)),
    InvestmentInterest => ("PARTNER_DEDUCTION_INVESTMENT_INTEREST", "13H", InvestmentInterest, Some(K1LossClass::InvestmentInterest)),
    RoyaltyDeductions => ("PARTNER_DEDUCTION_ROYALTIES", "13I", RoyaltyDeductions, Some(K1LossClass::OtherDeductions)),
    Section59e => ("PARTNER_DEDUCTION_SECTION59E", "13J", LeftOut, Some(K1LossClass::OtherDeductions)),
    ExcessBusinessInterest => ("PARTNER_DEDUCTION_EBIE", "13K", Ebie, None),
    PortfolioOther => ("PARTNER_DEDUCTION_PORTFOLIO_OTHER", "13L", LeftOut, Some(K1LossClass::OtherDeductions)),
    EducationalAssistance => ("PARTNER_DEDUCTION_EDUCATIONAL", "13N", LeftOut, Some(K1LossClass::OtherDeductions)),
    Preproductive => ("PARTNER_DEDUCTION_PREPRODUCTIVE", "13P", LeftOut, Some(K1LossClass::OtherDeductions)),
    Reforestation => ("PARTNER_DEDUCTION_REFORESTATION", "13S", LeftOut, Some(K1LossClass::OtherDeductions)),
    BasisAdjustmentDeduction => ("PARTNER_DEDUCTION_ADJUSTMENT", "13V", BoxOneAdjustment, None),
    SoilWater => ("PARTNER_DEDUCTION_SOIL_WATER", "13W", LeftOut, Some(K1LossClass::OtherDeductions)),
    Production => ("PARTNER_DEDUCTION_PRODUCTION", "13X", LeftOut, Some(K1LossClass::OtherDeductions)),
    Barriers => ("PARTNER_DEDUCTION_BARRIERS", "13Y", LeftOut, Some(K1LossClass::OtherDeductions)),
    Itemized => ("PARTNER_DEDUCTION_ITEMIZED", "13Z", LeftOut, Some(K1LossClass::OtherDeductions)),
    ConstructionFund => ("PARTNER_DEDUCTION_CONSTRUCTION_FUND", "13AA", LeftOut, Some(K1LossClass::OtherDeductions)),
    EarlyWithdrawalPenalty => ("PARTNER_DEDUCTION_EARLY_PENALTY", "13AB", EarlyPenalty, Some(K1LossClass::OtherDeductions)),
    DistributionInterest => ("PARTNER_DEDUCTION_DISTRIBUTION_INTEREST", "13AC", LeftOut, Some(K1LossClass::OtherDeductions)),
    WorkingInterest => ("PARTNER_DEDUCTION_WORKING_INTEREST", "13AD", LeftOut, Some(K1LossClass::OtherDeductions)),
    PortfolioDeductions => ("PARTNER_DEDUCTION_PORTFOLIO_DEDUCTIONS", "13AE", Nondeductible, Some(K1LossClass::OtherDeductions)),
    Other => ("PARTNER_DEDUCTION_OTHER", "13ZZ", LeftOut, Some(K1LossClass::OtherDeductions)),
]);

item_codes!(ScorpDeductionCode, [
    CashThirty => ("SCORP_DEDUCTION_CASH_THIRTY", "12B", CharityThirtyB, Some(K1LossClass::OtherCharity)),
    NoncashFifty => ("SCORP_DEDUCTION_NONCASH_FIFTY", "12C", CharityFiftyA, Some(K1LossClass::OtherCharity)),
    NoncashThirty => ("SCORP_DEDUCTION_NONCASH_THIRTY", "12D", CharityThirtyB, Some(K1LossClass::OtherCharity)),
    CapitalGainPropertyThirty => ("SCORP_DEDUCTION_PROPERTY_THIRTY", "12E", CharityThirtyC, Some(K1LossClass::OtherCharity)),
    CapitalGainPropertyTwenty => ("SCORP_DEDUCTION_PROPERTY_TWENTY", "12F", LeftOut, Some(K1LossClass::OtherCharity)),
    ContributionsHundred => ("SCORP_DEDUCTION_CONTRIBUTIONS_HUNDRED", "12G", LeftOut, Some(K1LossClass::OtherCharity)),
    InvestmentInterest => ("SCORP_DEDUCTION_INVESTMENT_INTEREST", "12H", InvestmentInterest, Some(K1LossClass::InvestmentInterest)),
    RoyaltyDeductions => ("SCORP_DEDUCTION_ROYALTIES", "12I", RoyaltyDeductions, Some(K1LossClass::OtherDeductions)),
    Section59e => ("SCORP_DEDUCTION_SECTION59E", "12J", LeftOut, Some(K1LossClass::OtherDeductions)),
    PortfolioOther => ("SCORP_DEDUCTION_PORTFOLIO_OTHER", "12L", LeftOut, Some(K1LossClass::OtherDeductions)),
    Preproductive => ("SCORP_DEDUCTION_PREPRODUCTIVE", "12M", LeftOut, Some(K1LossClass::OtherDeductions)),
    Reforestation => ("SCORP_DEDUCTION_REFORESTATION", "12O", LeftOut, Some(K1LossClass::OtherDeductions)),
    SoilWater => ("SCORP_DEDUCTION_SOIL_WATER", "12W", LeftOut, Some(K1LossClass::OtherDeductions)),
    Production => ("SCORP_DEDUCTION_PRODUCTION", "12X", LeftOut, Some(K1LossClass::OtherDeductions)),
    Barriers => ("SCORP_DEDUCTION_BARRIERS", "12Y", LeftOut, Some(K1LossClass::OtherDeductions)),
    Itemized => ("SCORP_DEDUCTION_ITEMIZED", "12Z", LeftOut, Some(K1LossClass::OtherDeductions)),
    ConstructionFund => ("SCORP_DEDUCTION_CONSTRUCTION_FUND", "12AA", LeftOut, Some(K1LossClass::OtherDeductions)),
    EarlyWithdrawalPenalty => ("SCORP_DEDUCTION_EARLY_PENALTY", "12AB", EarlyPenalty, Some(K1LossClass::OtherDeductions)),
    DistributionInterest => ("SCORP_DEDUCTION_DISTRIBUTION_INTEREST", "12AC", LeftOut, Some(K1LossClass::OtherDeductions)),
    Other => ("SCORP_DEDUCTION_OTHER", "12ZZ", LeftOut, Some(K1LossClass::OtherDeductions)),
]);

impl PartnerIncomeCode {
    // A loss is a negative amount of these codes only (a sign rule).
    pub(crate) fn may_be_loss(self) -> bool {
        self.class() == Some(K1LossClass::OtherLoss)
    }
}
impl ScorpIncomeCode {
    pub(crate) fn may_be_loss(self) -> bool {
        self.class() == Some(K1LossClass::OtherLoss)
    }
}

impl PartnerIncome {
    // The client rules of the list, by their keys: a code twice, then an
    // amount that is zero or a loss of a code that cannot be one.
    pub(crate) fn rule(items: &[Self]) -> Option<&'static str> {
        Self::issue(items, "items").map(|issue| issue.key)
    }
    pub(crate) fn issue(
        items: &[Self],
        list: &'static str,
    ) -> Option<crate::validation::ValidationIssue> {
        partner_income_issue(items, list, |item| (item.code, item.amount))
    }
    pub(crate) fn valid_list(items: &[Self]) -> bool {
        Self::rule(items).is_none()
    }
}
impl PartnerIncomeTotal {
    pub(crate) fn issue(
        items: &[Self],
        list: &'static str,
    ) -> Option<crate::validation::ValidationIssue> {
        partner_income_issue(items, list, |item| (item.code, item.amount))
    }
    pub(crate) fn rule(items: &[Self]) -> Option<&'static str> {
        Self::issue(items, "items").map(|issue| issue.key)
    }
}

fn partner_income_issue<T>(
    items: &[T],
    list: &'static str,
    value: impl Fn(&T) -> (PartnerIncomeCode, i64),
) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    if let Some((index, _)) = items
        .iter()
        .enumerate()
        .find(|(index, item)| items[..*index].iter().any(|prior| value(prior).0 == value(item).0))
    {
        return Some(
            ValidationIssue::new("K1_INCOME_CODE_TWICE").at_field("code").in_list(list, index),
        );
    }
    items
        .iter()
        .enumerate()
        .find(|(_, item)| {
            let (code, amount) = value(item);
            amount == 0 || (amount < 0 && !code.may_be_loss())
        })
        .map(|(index, _)| {
            ValidationIssue::new("K1_INCOME_CODE_SIGN").at_field("amount").in_list(list, index)
        })
}

impl ScorpIncome {
    pub(crate) fn rule(items: &[Self]) -> Option<&'static str> {
        Self::issue(items, "items").map(|issue| issue.key)
    }
    pub(crate) fn issue(
        items: &[Self],
        list: &'static str,
    ) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if let Some((index, _)) = items
            .iter()
            .enumerate()
            .find(|(index, item)| items[..*index].iter().any(|prior| prior.code == item.code))
        {
            return Some(
                ValidationIssue::new("K1_INCOME_CODE_TWICE").at_field("code").in_list(list, index),
            );
        }
        items
            .iter()
            .enumerate()
            .find(|(_, item)| item.amount == 0 || (item.amount < 0 && !item.code.may_be_loss()))
            .map(|(index, _)| {
                ValidationIssue::new("K1_INCOME_CODE_SIGN").at_field("amount").in_list(list, index)
            })
    }
    pub(crate) fn valid_list(items: &[Self]) -> bool {
        Self::rule(items).is_none() && items.iter().all(|item| SIGNED.contains(&item.amount))
    }
}
impl PartnerDeduction {
    pub(crate) fn rule(items: &[Self]) -> Option<&'static str> {
        Self::issue(items, "items").map(|issue| issue.key)
    }
    pub(crate) fn issue(
        items: &[Self],
        list: &'static str,
    ) -> Option<crate::validation::ValidationIssue> {
        items
            .iter()
            .enumerate()
            .find(|(index, item)| items[..*index].iter().any(|prior| prior.code == item.code))
            .map(|(index, _)| {
                crate::validation::ValidationIssue::new("K1_DEDUCTION_CODE_TWICE")
                    .at_field("code")
                    .in_list(list, index)
            })
    }
    pub(crate) fn valid_list(items: &[Self]) -> bool {
        Self::rule(items).is_none()
            && items.iter().all(|item| (1..=100_000_000_000).contains(&item.amount))
    }
}
impl ScorpDeduction {
    pub(crate) fn rule(items: &[Self]) -> Option<&'static str> {
        Self::issue(items, "items").map(|issue| issue.key)
    }
    pub(crate) fn issue(
        items: &[Self],
        list: &'static str,
    ) -> Option<crate::validation::ValidationIssue> {
        items
            .iter()
            .enumerate()
            .find(|(index, item)| items[..*index].iter().any(|prior| prior.code == item.code))
            .map(|(index, _)| {
                crate::validation::ValidationIssue::new("K1_DEDUCTION_CODE_TWICE")
                    .at_field("code")
                    .in_list(list, index)
            })
    }
    pub(crate) fn valid_list(items: &[Self]) -> bool {
        Self::rule(items).is_none()
            && items.iter().all(|item| (1..=100_000_000_000).contains(&item.amount))
    }
}

// The gain a distribution above the basis and the sale or liquidation of the
// whole interest produce in one tax system, actual source cents (unit B9-1, rules
// R14 and R17). The sale row of Form 8949 is its proceeds, less the section
// 751 ordinary part, and its basis; the ordinary part is Form 4797 Part II
// line 10.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Disposal {
    // Money above the basis before distributions: section 731(a)(1) gain
    // (section 1368(b)(2) for stock), capital by the holding period.
    pub distribution_gain: i64,
    // Whether the whole interest was sold or liquidated (the row exists).
    pub sold: bool,
    // Box 20 code AB, the section 751 gain (loss).
    pub ordinary: i64,
    pub proceeds: i64,
    pub basis: i64,
}

impl Disposal {
    // The capital gain (loss) of the sale or liquidation.
    pub(crate) fn capital(&self) -> i64 {
        if self.sold { self.proceeds - self.basis } else { 0 }
    }
    // Gain of the sale or liquidation altogether.
    pub(crate) fn sale_gain(&self) -> i64 {
        if self.sold { self.ordinary + self.capital() } else { 0 }
    }
    // Income of the K-1's one Form 6198 activity: a gain on disposing of the
    // interest (i6198 lines 2a to 2c); a loss adds none.
    pub(crate) fn activity_gain(&self) -> i64 {
        self.distribution_gain + phase::floor(self.sale_gain(), 0)
    }
}

// One entered code in the shared form the limits and destinations read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Entry {
    pub kind: K1ItemKind,
    pub class: Option<K1LossClass>,
    // The row's printed box and code; the notice resource of a left-out code.
    pub row: &'static str,
    pub item: &'static str,
    // Signed cents for a box 11 (10) code; the positive deduction otherwise.
    pub cents: i64,
    // The box 13 (12) list, whose amounts are deductions.
    pub deduction: bool,
    // One-based property index and whether it was held more than a year.
    // Sale rows have their own Form 4797 route, independent of code R.
    pub sale: Option<(usize, bool)>,
}

impl Entry {
    // The loss or deduction this entry brings to its class, actual source cents.
    fn checked_loss(&self) -> Option<i64> {
        if self.class.is_none() {
            Some(0)
        } else if self.deduction || self.kind == K1ItemKind::GamblingLosses {
            Some(self.cents)
        } else if self.cents < 0 {
            self.cents.checked_neg()
        } else {
            Some(0)
        }
    }
    // The entry's positive income, actual source cents.
    fn income(&self) -> i64 {
        if !self.deduction && self.kind.income() && self.cents > 0 { self.cents } else { 0 }
    }
}

// The codes of one K-1 in one list, and what they bring to every stage.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct K1Items {
    entries: Vec<Entry>,
}

// What the codes amount to after the basis and at-risk limits, in cents, by
// destination (K1Items::flows). Every field is the amount that reaches its
// line for this tax system.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct K1ItemFlows<Number = i64> {
    // Schedule 1 line 8z: code A and code J.
    pub portfolio: Number,
    pub recovery: Number,
    // Schedule 1 line 8c: code E.
    pub debt: Number,
    // Schedule 1 line 8b winnings and Schedule A line 16 losses.
    pub winnings: Number,
    pub gambling_losses: Number,
    // Form 6781 Part I line 1: the K-1's net section 1256 amount.
    pub section1256: Number,
    // Schedule D lines 5 and 12: code Q (portfolio).
    pub preferred: [Number; 2],
    // Form 4797 Part II line 11 (2026): code R.
    pub ordinary: Number,
    // Schedule D lines 5 and 12: code S (the K-1's activity).
    pub nonportfolio: [Number; 2],
    // Schedule A class worksheet: the (B) 30%, (A) 50% and (C) 30% classes.
    pub charity: [Number; 3],
    pub charity_cash30: Number,
    // Form 4952 line 1.
    pub investment_interest: Number,
    // Schedule E line 19, after basis and at-risk limits.
    pub royalty_deductions: Number,
    // Schedule 1 line 18.
    pub early_penalty: Number,
    // Each calculated code's amount for its row: (row, kind, cents).
    pub rows: Vec<(&'static str, K1ItemKind, Number)>,
    // Section 179 sale ordinary amounts after basis and at-risk limits:
    // (one-based property index, held long, cents). Also in `ordinary`.
    pub sales: Vec<(usize, bool, Number)>,
}

impl K1ItemFlows<i128> {
    fn narrow(self) -> Option<K1ItemFlows> {
        Some(K1ItemFlows {
            portfolio: self.portfolio.try_into().ok()?,
            recovery: self.recovery.try_into().ok()?,
            debt: self.debt.try_into().ok()?,
            winnings: self.winnings.try_into().ok()?,
            gambling_losses: self.gambling_losses.try_into().ok()?,
            section1256: self.section1256.try_into().ok()?,
            ordinary: self.ordinary.try_into().ok()?,
            charity_cash30: self.charity_cash30.try_into().ok()?,
            investment_interest: self.investment_interest.try_into().ok()?,
            royalty_deductions: self.royalty_deductions.try_into().ok()?,
            early_penalty: self.early_penalty.try_into().ok()?,
            preferred: [self.preferred[0].try_into().ok()?, self.preferred[1].try_into().ok()?],
            nonportfolio: [
                self.nonportfolio[0].try_into().ok()?,
                self.nonportfolio[1].try_into().ok()?,
            ],
            charity: [
                self.charity[0].try_into().ok()?,
                self.charity[1].try_into().ok()?,
                self.charity[2].try_into().ok()?,
            ],
            rows: self
                .rows
                .into_iter()
                .map(|(row, kind, amount)| Some((row, kind, amount.try_into().ok()?)))
                .collect::<Option<_>>()?,
            sales: self
                .sales
                .into_iter()
                .map(|(index, held_long, amount)| Some((index, held_long, amount.try_into().ok()?)))
                .collect::<Option<_>>()?,
        })
    }
}

impl K1ItemFlows {
    // Form 8582's ordinary-character cut is shared among the individual
    // ordinary loss sources. Positive recapture and sale gains stay whole.
    // Keep cents exactly as the loss-class owner does.
    pub(crate) fn after_passive(&self, cut: i64) -> Option<Self> {
        let weights: Vec<i64> = self
            .rows
            .iter()
            .map(|(_, kind, cents)| {
                if *kind == K1ItemKind::OrdinaryActivity {
                    i64::try_from((-i128::from(*cents)).max(0)).ok()
                } else {
                    Some(0)
                }
            })
            .chain(
                self.sales
                    .iter()
                    .map(|(_, _, cents)| i64::try_from((-i128::from(*cents)).max(0)).ok()),
            )
            .collect::<Option<_>>()?;
        if cut < 0
            || i128::from(cut) > (-i128::from(self.ordinary)).max(0)
            || cut > weights.iter().try_fold(0i64, |total, value| total.checked_add(*value))?
        {
            return None;
        }
        let cuts = crate::kernel::ratio::allocate_remaining(cut, &weights);
        let mut result = self.clone();
        for (index, (_, _, cents)) in result.rows.iter_mut().enumerate() {
            *cents = cents.checked_add(cuts[index])?;
        }
        for (index, (_, _, cents)) in result.sales.iter_mut().enumerate() {
            *cents = cents.checked_add(cuts[result.rows.len() + index])?;
        }
        result.ordinary = result.ordinary.checked_add(cut)?;
        Some(result)
    }

    // The Schedule D lines 5 and 12 amounts of the codes, in cents.
    pub(crate) fn capital(&self) -> [i64; 2] {
        self.checked_capital().expect("bounded K-1 capital amounts")
    }
    pub(crate) fn checked_capital(&self) -> Option<[i64; 2]> {
        Some([
            (i128::from(self.preferred[0]) + i128::from(self.nonportfolio[0])).try_into().ok()?,
            (i128::from(self.preferred[1]) + i128::from(self.nonportfolio[1])).try_into().ok()?,
        ])
    }
    // The activity items (Form 8582), in source cents: debt, ordinary,
    // short-term and long-term.
    pub(crate) fn activity(&self) -> ActivityItems {
        ActivityItems {
            debt: self.debt,
            ordinary: self.ordinary,
            short: self.nonportfolio[0],
            long: self.nonportfolio[1],
        }
    }
}

// The net of the K-1's activity items after the basis and at-risk limits,
// source cents: code E income, code R and code S gains and losses. They join
// the activity's passive result (Reg. 1.469-2T(c)(2)).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ActivityItems {
    pub debt: i64,
    pub ordinary: i64,
    pub short: i64,
    pub long: i64,
}

impl ActivityItems {
    pub(crate) fn net(&self) -> i64 {
        self.checked_net().expect("bounded K-1 activity net")
    }
    pub(crate) fn checked_net(&self) -> Option<i64> {
        self.checked_net_with(0)
    }
    pub(crate) fn checked_net_with(&self, ordinary: i64) -> Option<i64> {
        [ordinary, self.debt, self.ordinary, self.short, self.long]
            .into_iter()
            .map(i128::from)
            .sum::<i128>()
            .try_into()
            .ok()
    }
    pub(crate) fn any(&self) -> bool {
        *self != Self::default()
    }
    // What a limit of the combined activity holds back of each loss item:
    // [Schedule E ordinary part, code R, code S short, code S long], shared
    // among the characters that have a net loss in proportion to it (Form
    // 8582 Parts VII to IX). `ordinary` is the Schedule E part of the net,
    // its prior unallowed loss included.
    pub(crate) fn cuts(&self, ordinary: i64, held: i64) -> Option<[i64; 4]> {
        Self::cuts_in_units::<crate::kernel::carry::Cents>(
            [ordinary, self.ordinary, self.short, self.long, self.debt],
            held,
        )
    }

    // Source-precision Part IX nets in the same order as cuts(): Schedule E,
    // ordinary property, short capital, long capital, debt. The caller supplies
    // all five in Unit; no annual-dollar ActivityItems field is relabeled as
    // unrounded source cents, and no rounding precedes the character cut.
    pub(crate) fn cuts_in_units<Unit>(nets: [i64; 5], held: i64) -> Option<[i64; 4]> {
        let nets = nets.into_iter().enumerate().collect::<alloc::vec::Vec<_>>();
        let crate::kernel::source_fact::SourceFact::Reported(split) =
            crate::kernel::loss_limits::passive_characters(
                crate::kernel::source_fact::SourceFact::Reported(
                    crate::kernel::loss_limits::CharacterInput {
                        nets: &nets,
                        suspended: crate::kernel::carry::Amount::<Unit>::new(held).ok()?,
                    },
                ),
            )
            .ok()?
        else {
            unreachable!("reported characters remain reported")
        };
        Some([split[0].held(), split[1].held(), split[2].held(), split[3].held()])
    }
}

impl K1Items {
    pub(crate) fn partner(income: &[PartnerIncomeTotal], deductions: &[PartnerDeduction]) -> Self {
        let mut entries = Vec::new();
        for item in income {
            entries.push(Entry {
                kind: item.code.kind(),
                class: item.code.class(),
                row: item.code.row(),
                item: item.code.item(),
                cents: item.amount,
                deduction: false,
                sale: None,
            });
        }
        for item in deductions {
            entries.push(Entry {
                kind: item.code.kind(),
                class: item.code.class(),
                row: item.code.row(),
                item: item.code.item(),
                cents: item.amount,
                deduction: true,
                sale: None,
            });
        }
        Self { entries }
    }

    pub(crate) fn scorp(income: &[ScorpIncome], deductions: &[ScorpDeduction]) -> Self {
        let mut entries = Vec::new();
        for item in income {
            entries.push(Entry {
                kind: item.code.kind(),
                class: item.code.class(),
                row: item.code.row(),
                item: item.code.item(),
                cents: item.amount,
                deduction: false,
                sale: None,
            });
        }
        for item in deductions {
            entries.push(Entry {
                kind: item.code.kind(),
                class: item.code.class(),
                row: item.code.row(),
                item: item.code.item(),
                cents: item.amount,
                deduction: true,
                sale: None,
            });
        }
        Self { entries }
    }

    // Ordinary recapture and short-held dispositions join OtherLoss, but
    // each property's gain and loss must enter the limit separately.
    pub(crate) fn with_sales(self, sales: &[Section179SaleResult]) -> Self {
        self.checked_with_sales(sales).expect("bounded K-1 sale cents")
    }
    pub(crate) fn checked_with_sales(mut self, sales: &[Section179SaleResult]) -> Option<Self> {
        for (index, sale) in sales.iter().enumerate() {
            if sale.ordinary != 0 {
                self.entries.push(Entry {
                    kind: K1ItemKind::OrdinaryActivity,
                    class: Some(K1LossClass::OtherLoss),
                    row: "",
                    item: "",
                    cents: sale.ordinary.checked_mul(100)?,
                    deduction: false,
                    sale: Some((index + 1, sale.held_long)),
                });
            }
        }
        Some(self)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    // This year's loss or deduction of every class the codes join, whole
    // dollars (the current share of the class input).
    pub(crate) fn current(&self) -> LossAmounts {
        self.checked_current().expect("bounded K-1 class cents")
    }
    pub(crate) fn checked_current(&self) -> Option<LossAmounts> {
        let mut amounts = LossAmounts::default();
        for entry in &self.entries {
            if let Some(class) = entry.class {
                amounts[class] =
                    Amount::<Cents>::new(amounts[class].get().checked_add(entry.checked_loss()?)?)
                        .ok()?;
            }
        }
        Some(amounts)
    }

    // The positive income of the codes, actual source cents: partner basis worksheet
    // line 4, Form 7203 line 3 and the K-1 activity's Form 6198 income.
    pub(crate) fn income(&self) -> i64 {
        self.checked_income().expect("bounded K-1 income cents")
    }
    pub(crate) fn checked_income(&self) -> Option<i64> {
        self.entries.iter().map(Entry::income).map(i128::from).sum::<i128>().try_into().ok()
    }

    // Code F less code V, cents: part of box 1 for every box 1 purpose.
    pub(crate) fn box_one(&self) -> i64 {
        self.checked_box_one().expect("bounded K-1 box-one adjustment")
    }
    pub(crate) fn checked_box_one(&self) -> Option<i64> {
        self.entries
            .iter()
            .filter(|entry| entry.kind == K1ItemKind::BoxOneAdjustment)
            .map(
                |entry| {
                    if entry.deduction { -i128::from(entry.cents) } else { i128::from(entry.cents) }
                },
            )
            .sum::<i128>()
            .try_into()
            .ok()
    }

    // Box 13 code K, actual source cents: it reduces basis whether or not it is
    // deductible (the outside basis worksheet line 15n).
    pub(crate) fn ebie(&self) -> i64 {
        self.checked_ebie().expect("bounded K-1 excess business interest")
    }
    pub(crate) fn checked_ebie(&self) -> Option<i64> {
        self.entries
            .iter()
            .filter(|entry| entry.kind == K1ItemKind::Ebie)
            .map(|entry| i128::from(entry.cents))
            .sum::<i128>()
            .try_into()
            .ok()
    }

    // Whether any code is investment interest (box 13 (12) code H).
    pub(crate) fn investment_interest(&self) -> i64 {
        self.checked_investment_interest().expect("bounded K-1 investment interest")
    }
    pub(crate) fn checked_investment_interest(&self) -> Option<i64> {
        Some(self.checked_current()?[K1LossClass::InvestmentInterest].get())
    }

    // The resources of the left-out codes in a notice, once each.
    pub(crate) fn left_out(&self) -> Vec<&'static str> {
        let mut items: Vec<&'static str> = Vec::new();
        for entry in &self.entries {
            if entry.kind == K1ItemKind::LeftOut && !items.contains(&entry.item) {
                items.push(entry.item);
            }
        }
        items
    }

    // Whether a code reports a portfolio kind of income that is investment
    // income of Form 4952 (line 4a) and net investment income (line 7).
    pub(crate) fn has(&self, kind: K1ItemKind) -> bool {
        self.entries.iter().any(|entry| entry.kind == kind)
    }

    // The amounts of every code when no limit holds anything back.
    pub(crate) fn flows_whole(&self) -> K1ItemFlows {
        self.checked_flows_whole().expect("bounded whole K-1 flows")
    }
    pub(crate) fn checked_flows_whole(&self) -> Option<K1ItemFlows> {
        let ledger = LossLedger { risk_allowed: self.checked_current()?, ..LossLedger::default() };
        self.checked_flows(&ledger)
    }

    // The amounts of every code after the limits. A class's allowed amount is
    // shared by its codes in proportion to this year's loss of each (the
    // item-level pro rata of Reg. 1.704-1(d)(2) and 1.1366-2(a)(5), which is
    // linear); a gain is never limited.
    pub(crate) fn flows(&self, ledger: &LossLedger) -> K1ItemFlows {
        self.checked_flows(ledger).expect("bounded K-1 flows")
    }
    pub(crate) fn checked_flows(&self, ledger: &LossLedger) -> Option<K1ItemFlows> {
        let mut flows = K1ItemFlows::<i128>::default();
        let losses: Vec<i64> =
            self.entries.iter().map(Entry::checked_loss).collect::<Option<_>>()?;
        let mut shares = alloc::vec![0i64; self.entries.len()];
        for class in K1LossClass::ALL {
            let members: Vec<usize> = self
                .entries
                .iter()
                .enumerate()
                .filter(|(index, entry)| entry.class == Some(class) && losses[*index] > 0)
                .map(|(index, _)| index)
                .collect();
            if members.is_empty() {
                continue;
            }
            let weights: Vec<i64> = members.iter().map(|i| losses[*i]).collect();
            let total = weights.iter().try_fold(0i64, |total, value| total.checked_add(*value))?;
            let allowed = phase::bound(ledger.risk_allowed[class].get(), 0, total);
            let parts = crate::kernel::ratio::allocate_remaining(allowed, &weights);
            for (index, part) in members.iter().zip(parts) {
                shares[*index] = part;
            }
        }
        for (entry, share) in self.entries.iter().zip(shares) {
            // What reaches the entry's line, cents: a gain whole, a loss
            // less what the limits held back.
            let amount = if entry.deduction || entry.kind == K1ItemKind::GamblingLosses {
                share
            } else if entry.cents < 0 {
                -share
            } else {
                entry.cents
            };
            let amount = i128::from(amount);
            match entry.kind {
                K1ItemKind::Portfolio => flows.portfolio += amount,
                K1ItemKind::Recovery => flows.recovery += amount,
                K1ItemKind::DebtCancellation => flows.debt += amount,
                K1ItemKind::GamblingWinnings => flows.winnings += amount,
                K1ItemKind::GamblingLosses => flows.gambling_losses += amount,
                K1ItemKind::Section1256 => flows.section1256 += amount,
                K1ItemKind::PreferredShort => flows.preferred[0] += amount,
                K1ItemKind::PreferredLong => flows.preferred[1] += amount,
                K1ItemKind::OrdinaryActivity => flows.ordinary += amount,
                K1ItemKind::NonportfolioShort => flows.nonportfolio[0] += amount,
                K1ItemKind::NonportfolioLong => flows.nonportfolio[1] += amount,
                K1ItemKind::CharityThirtyB => {
                    flows.charity[0] += amount;
                    if entry.row.ends_with('B') {
                        flows.charity_cash30 += amount;
                    }
                }
                K1ItemKind::CharityFiftyA => flows.charity[1] += amount,
                K1ItemKind::CharityThirtyC => flows.charity[2] += amount,
                K1ItemKind::InvestmentInterest => flows.investment_interest += amount,
                K1ItemKind::RoyaltyDeductions => flows.royalty_deductions += amount,
                K1ItemKind::EarlyPenalty => flows.early_penalty += amount,
                K1ItemKind::BoxOneAdjustment
                | K1ItemKind::Ebie
                | K1ItemKind::Nondeductible
                | K1ItemKind::LeftOut => continue,
            }
            if let Some((index, held_long)) = entry.sale {
                flows.sales.push((index, held_long, amount));
            } else {
                flows.rows.push((entry.row, entry.kind, amount));
            }
        }
        flows.narrow()
    }
}

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(K1Section179Sale {
    amounts: [regular, alternative],
    gains: [],
    facts: [description, acquired, sold, id],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(PartnerIncome { amounts: [amount], gains: [], facts: [code] });

// Allocate calculated amounts with their retained code character.
crate::community::divide_record!(PartnerIncomeTotal {
    amounts: [amount],
    gains: [],
    facts: [code]
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(PartnerDeduction { amounts: [amount], gains: [], facts: [code] });

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(ScorpIncome { amounts: [amount], gains: [], facts: [code] });

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(ScorpDeduction { amounts: [amount], gains: [], facts: [code] });

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(PartnerSaleParts {
    amounts: [ordinary, collectibles, unrecaptured],
    gains: [],
    facts: [],
});

crate::community::divide_enum!(K1Disposition {
    records: [Sold { amounts: [proceeds], facts: [] }],
    units: [Kept, LiquidatedForCash, Other],
    tuples: [],
});

crate::community::divide_enum!(K1Section179SaleAmtSource {
    records: [Reported { amounts: [amounts], facts: [] }],
    units: [Unavailable, NotReported],
    tuples: [],
});

crate::community::divide_record!(K1Section179SaleAmounts {
    amounts: [price, cost, depreciation, section179],
    gains: [],
    facts: [],
});

// Physical identity traversal visits the canonical source once. AMT monetary
// projections preserve that same ID and never allocate another object.
pub(crate) fn k1_sale_ids(income: &crate::FederalIncome) -> Vec<i64> {
    match income {
        crate::FederalIncome::Partnership { data } => {
            data.section179_sales.iter().map(|sale| sale.id).collect()
        }
        crate::FederalIncome::Scorp { data } => {
            data.statements.section179_sales.iter().map(|sale| sale.id).collect()
        }
        crate::FederalIncome::PartnershipActivities { data } => {
            data.entity.section179_sales.iter().map(|sale| sale.id).collect()
        }
        crate::FederalIncome::ScorpActivities { data } => {
            data.entity.section179_sales.iter().map(|sale| sale.id).collect()
        }
        _ => Vec::new(),
    }
}
pub(crate) fn k1_sale_ids_mut(income: &mut crate::FederalIncome) -> Vec<&mut i64> {
    match income {
        crate::FederalIncome::Partnership { data } => {
            data.section179_sales.iter_mut().map(|sale| &mut sale.id).collect()
        }
        crate::FederalIncome::Scorp { data } => {
            data.statements.section179_sales.iter_mut().map(|sale| &mut sale.id).collect()
        }
        crate::FederalIncome::PartnershipActivities { data } => {
            data.entity.section179_sales.iter_mut().map(|sale| &mut sale.id).collect()
        }
        crate::FederalIncome::ScorpActivities { data } => {
            data.entity.section179_sales.iter_mut().map(|sale| &mut sale.id).collect()
        }
        _ => Vec::new(),
    }
}

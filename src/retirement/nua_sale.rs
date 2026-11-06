// Coverage: NUA sales (B7a, 2026-10-09). NuaSale (employer stock a qualifying plan distribution sent
//   out with net unrealized appreciation, sold in 2026: the deferred NUA is long-term gain up to its
//   amount whatever the holding period, the rest is long or short by the holding period from the
//   distribution; Pub. 575, section 402(e)(4)(B); inherited shares keep the estate value as basis and
//   are long-term) gains its page, NuaShareSale, on Investment sales, with the Form 1099-B boxes 1a,
//   1c, 1d and 4 imported and the distribution facts (Form 1099-R box 6 of the earlier year) entered.
//   Each of the two parts is reported on Form 8949 with code B (box A/D; the broker's basis is the
//   plan cost without the NUA), so they total on Schedule D line 1b or 8b. A sale with a special
//   treatment is left out with WARN_NUA_SALE_LEFT_OUT. Forms: 8949, Schedule D, 1099-R box 6. Tests:
//   tests/investment/investment_kinds.rs nua_share_sale_*.
use crate::kernel::phase;
use crate::{FederalSale, GainTerm, SaleAdjustment, SaleUse};
use alloc::string::String;
use libertas_macros::*;

/// Sale of employer stock with deferred net unrealized appreciation
/// One sold lot from a qualifying retirement-plan distribution. Enter the original distribution separately using its Form 1099-R; do not repeat this sale as another capital sale.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct NuaSale {
    /// Date the plan distributed these shares
    #[libertas_date_only]
    pub distributed: u32,
    /// Sale date
    #[libertas_date_only]
    pub sold: u32,
    /// Fair market value of this sold lot at distribution
    /// Use only the shares sold, including fractional shares, not all shares on the original statement.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub distribution_value: i64,
    /// Deferred NUA allocated to this sold lot
    /// From Form 1099-R box 6 and the plan's lot allocation. This amount was excluded at distribution and is not included in stock basis. Do not include appreciation after distribution.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub deferred_nua: i64,
    /// Inherited shares already distributed by the plan before death
    /// Leave absent for your own directly held shares. This is not a beneficiary distribution made by the retirement plan after death.
    pub inheritance: Option<NuaInheritance>,
    /// Net sale proceeds after fees
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub proceeds: i64,
    /// Disallowed wash-sale loss from replacement-share records
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub wash_sale: i64,
    /// Federal income tax withheld on this sale, not on the original distribution
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
    /// Basis the Form 1099-B printed in box 1e, when it shows one
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub reported_basis: Option<i64>,
    /// Whether the broker reported the basis to the IRS (box 12)
    pub reporting: crate::BasisReporting,
    /// A gift, a rollover, an election to tax the NUA when distributed, or another special treatment
    /// Yes leaves this sale out of the estimate with a notice; that treatment is not supported.
    #[libertas_default(false)]
    pub special: bool,
}
/// Inherited employer stock with deferred NUA
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct NuaInheritance {
    /// Previous owner died on
    #[libertas_date_only]
    pub died: u32,
    /// Estate valuation of only the shares sold
    /// Use this lot's reconciled date-of-death value, or the applicable section 2032 value from the executor's valid alternate-valuation election. An estate distribution, sale or other disposition within six months can set an earlier valuation date; do not automatically use the six-month value. Do not use the whole holding's value. Deferred NUA is excluded from the inherited basis adjustment. Values below deferred NUA require further allocation support. The App does not prepare the estate return or make its election.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub value: i64,
    /// Federal estate or generation-skipping tax attributable to this inherited income
    /// Ask the executor whether the estate paid federal estate or generation-skipping tax. Yes leaves out the section 691(c) deduction, which needs a capital-gain estate-tax allocation the App does not calculate.
    pub estate_tax: bool,
}
impl NuaSale {
    pub(crate) fn confirmed(&self) -> bool {
        !self.special && self.inheritance.as_ref().is_none_or(|v| v.value >= self.deferred_nua)
    }

    pub(crate) fn basis(&self) -> i64 {
        self.inheritance.as_ref().map_or(self.distribution_value, |v| v.value) - self.deferred_nua
    }
    pub(crate) fn realized_nua(&self) -> i64 {
        phase::cap(phase::floor(self.proceeds - self.basis(), 0), [self.deferred_nua])
    }
    pub(crate) fn parts(&self) -> [FederalSale; 2] {
        let nua = self.realized_nua();
        let base = FederalSale {
            digital_asset: false,
            market_discount: 0,
            several_sales: false,
            term: GainTerm::Long,
            collectible: false,
            proceeds: nua,
            basis: Some(0),
            amt: crate::SaleAmt::SameAsRegular,
            withholding: 0,
            use_kind: SaleUse::Investment { wash_sale: 0 },
            special: crate::SaleSpecial::NoneOfThese,
            // Form 8949 code B: the NUA part has no basis of its own (a
            // basis not reported to the IRS, or no Form 1099-B, needs none).
            adjustment: if self.reporting == crate::BasisReporting::ReportedToIrs {
                SaleAdjustment::BasisCorrection { reported_basis: 0 }
            } else {
                SaleAdjustment::None
            },
            basis_reporting: self.reporting,
        };
        let remainder = FederalSale {
            digital_asset: false,
            market_discount: 0,
            several_sales: false,
            term: if self.inheritance.is_some()
                || self.sold > self.distributed.saturating_add(10000)
            {
                GainTerm::Long
            } else {
                GainTerm::Short
            },
            proceeds: self.proceeds,
            basis: Some(self.basis() + nua),
            withholding: self.withholding,
            use_kind: SaleUse::Investment { wash_sale: self.wash_sale },
            // Code B: the broker shows the plan's cost, without the NUA, as
            // box 1e prints it (F3, 2026-10-09) or as the model has it.
            adjustment: crate::employee_stock::sale_adjustment(
                self.reporting,
                self.reported_basis,
                self.basis(),
                self.basis() + nua,
            ),
            ..base.clone()
        };
        [base, remainder]
    }
    pub(crate) fn valid(&self) -> bool {
        crate::federal_rules::valid_birth(self.distributed)
            && crate::federal_rules::valid_birth(self.sold)
            && self.distributed <= self.sold
            && self.sold / 10000 == 2026
            && [
                self.distribution_value,
                self.deferred_nua,
                self.proceeds,
                self.wash_sale,
                self.withholding,
            ]
            .iter()
            .all(|v| (0..=100000000000).contains(v))
            && self.inheritance.as_ref().is_none_or(|v| {
                crate::federal_rules::valid_birth(v.died)
                    && self.distributed <= v.died
                    && v.died <= self.sold
                    && (0..=100000000000).contains(&v.value)
            })
            && self.deferred_nua <= self.distribution_value
            && self.wash_sale <= (self.basis() - self.proceeds).max(0)
    }
}

/// Employer stock sold after a plan distribution (NUA)
/// Employer shares a retirement plan distributed to you with net unrealized appreciation (Form 1099-R box 6), sold in 2026. Enter the sale here with its 1099-B, not again as a 1099-B sale; enter the distribution itself on its 1099-R only in the year you received it.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into source_issuer, source_recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read the 1099-B sale of these employer shares exactly. Check its tax year against the supplied year; report a mismatch. The statement must show exactly one sale of these shares; if it shows several sales, report an error asking for only this sale. Return the box 1a description, or an empty string when it is blank. The box 1c date sold and box 1d proceeds must show values; report a blank one as an error. Return zero for a clearly blank box 4. Report illegible or conflicting values as errors. Return box 1e as reported_basis exactly as printed, or leave it empty when the box is blank, and return box 12 as reporting. Read distributed, distribution_value and deferred_nua from the plan's record supplied with the 1099-B for these shares: the date the plan distributed them, the value of the sold shares on that date and the net unrealized appreciation attributable to them. When no such record is supplied, report an error asking for the plan's record of these shares. Since Form 1099-R box 6 covers the whole distribution, do not return Form 1099-R values even when a Form 1099-R is supplied; do not infer a wash sale, an inheritance or other facts absent from these pages. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form. Read the printed calendar year as an integer such as 2026, or zero when blank. Read the account number exactly as printed, or empty text when blank. Read box 1b date acquired as the printed date, VARIOUS, INHERITED or another notation, or null when blank. Read the Form 8949 box letter printed on the statement, or null when none is printed, and box 2 short-term, long-term and ordinary as printed checkboxes (true only when checked).",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.source_issuer.federal_id,@.source_recipient.federal_id,@.description,@.sold,@.distributed,@.proceeds,@.reported_basis,@.withholding,@.distribution_value,@.deferred_nua",
    form_date_order = "MonthDayYear"
)]
pub struct NuaShareSale {
    /// Calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub calendar_year: u32,
    /// Source document issuer as printed
    #[libertas_llm_doc_field]
    pub source_issuer: crate::PrintedParty,
    /// Source document recipient as printed
    #[libertas_llm_doc_field]
    pub source_recipient: crate::PrintedParty,
    /// Account number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// 1099-B box 1a description
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_size(min = 0)]
    #[libertas_default("")]
    pub description: String,
    /// 1b — Date acquired
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed date or notation is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    pub box_1b_acquired: Option<crate::BrokerAcquisitionDate>,
    /// 1099-B box 1c date sold
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub sold: u32,
    /// 1099-B box 1d proceeds
    /// Net of selling fees, as the 1099-B reports them.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub proceeds: i64,
    /// 1099-B box 1e cost or other basis
    /// As printed, before any correction: the broker's figure leaves out the compensation or net unrealized appreciation, which the App adds with Form 8949 code B. Leave empty when the box is blank.
    /// [NullText]
    /// No cost basis is printed in box 1e
    /// [SetText]
    /// The cost basis printed in box 1e is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub reported_basis: Option<i64>,
    /// 1099-B box 12 basis reported to the IRS
    /// Box A or D when the broker reported the basis to the IRS, B or E when it did not, C or F when no Form 1099-B was received.
    #[libertas_llm_doc_field]
    #[libertas_default(ReportedToIrs)]
    pub reporting: crate::BasisReporting,
    /// Applicable checkbox on Form 8949
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed code is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    pub form_8949_checkbox: Option<crate::Form1099B8949Checkbox>,
    /// 2 — Short-term gain or loss
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_2_short_term: bool,
    /// 2 — Long-term gain or loss
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_2_long_term: bool,
    /// 2 — Ordinary
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_2_ordinary: bool,
    /// 1099-B box 4 federal income tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Date the plan distributed these shares
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub distributed: u32,
    /// Value of the shares sold on that date
    /// For only the shares sold, from the plan's distribution statement.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub distribution_value: i64,
    /// Their net unrealized appreciation
    /// The part of Form 1099-R box 6 for the shares sold. It is not in your basis; do not include growth after the distribution.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub deferred_nua: i64,
}

// The personal facts of one printed NUA sale, asked on Investment sales (the
// printed sale itself is the Document center's).
/// Wash sale and special treatment for this sale
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct NuaShareSaleMore {
    /// Wash sale loss disallowed
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub wash_sale: i64,
    /// A gift, a rollover, an election to tax the NUA when distributed, or another special treatment
    /// Yes leaves this sale out of the estimate with a notice; that treatment is not supported.
    pub special: bool,
}

/// Whose shares these were
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum NuaOwnership {
    /// My own shares
    /// The plan distributed these shares to you.
    Own,
    /// Inherited shares
    /// The plan distributed these shares to someone who died before the sale.
    Inherited {
        /// The inheritance details
        inheritance: NuaInheritance,
    },
}

/// Answers for one printed NUA sale
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct NuaSaleAnswers {
    /// Accepted printed sale identity
    pub document_id: i64,
    /// Wash sale and special treatment, absent until answered
    /// [NullText]
    /// Not answered yet
    pub more: Option<NuaShareSaleMore>,
    /// Whose shares these were, absent until answered
    /// [NullText]
    /// Not answered yet
    pub ownership: Option<NuaOwnership>,
}

impl NuaShareSale {
    // The first relationship between the printed values that fails, by its
    // rule's key, in the order of the client rules.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if self.distributed > self.sold {
            return Some(
                crate::validation::ValidationIssue::new("NUA_SOLD_BEFORE_DISTRIBUTED")
                    .at_field("sold"),
            );
        }
        if !(20260101..=20261231).contains(&self.sold) {
            return Some(
                crate::validation::ValidationIssue::new("NUA_SALE_NOT_IN_2026").at_field("sold"),
            );
        }
        if self.deferred_nua > self.distribution_value {
            return Some(
                crate::validation::ValidationIssue::new("NUA_ABOVE_DISTRIBUTION_VALUE")
                    .at_field("deferred_nua"),
            );
        }
        None
    }
    // The complete sale from the printed sale and its answers.
    pub(crate) fn lot(
        &self,
        more: &NuaShareSaleMore,
        inheritance: Option<NuaInheritance>,
    ) -> NuaSale {
        NuaSale {
            distributed: self.distributed,
            sold: self.sold,
            distribution_value: self.distribution_value,
            deferred_nua: self.deferred_nua,
            inheritance,
            proceeds: self.proceeds,
            wash_sale: more.wash_sale,
            withholding: self.withholding,
            reported_basis: self.reported_basis,
            reporting: self.reporting,
            special: more.special,
        }
    }
    // The printed sale of a complete one (its answers are not printed).
    pub(crate) fn from_record(label: &str, v: &NuaSale) -> Self {
        Self {
            calendar_year: Default::default(),
            account_number: Default::default(),
            box_1b_acquired: Default::default(),
            form_8949_checkbox: Default::default(),
            box_2_short_term: Default::default(),
            box_2_long_term: Default::default(),
            box_2_ordinary: Default::default(),
            source_issuer: Default::default(),
            source_recipient: Default::default(),
            description: label.into(),
            sold: v.sold,
            proceeds: v.proceeds,
            withholding: v.withholding,
            reported_basis: v.reported_basis,
            reporting: v.reporting,
            distributed: v.distributed,
            distribution_value: v.distribution_value,
            deferred_nua: v.deferred_nua,
        }
    }
    // The entry label: the box 1a description as typed.
    pub(crate) fn label(&self) -> String {
        self.description.clone()
    }
}

// A printed NUA sale (FederalIncome::NuaSaleSource) and its answers on
// Investment sales; absent answers are not answered yet.
pub(crate) fn answers(draft: &crate::FederalDraft, id: i64) -> Option<&NuaSaleAnswers> {
    draft.screening.as_ref()?.nua_sale_answers.iter().find(|row| row.document_id == id)
}

// The ownership question is asked unless the sale is left out.
pub(crate) fn ownership_needed(draft: &crate::FederalDraft, id: i64) -> bool {
    answers(draft, id).and_then(|answer| answer.more.as_ref()).is_none_or(|more| !more.special)
}

// The complete sale, or the first answer it still needs or refuses.
pub(crate) fn resolve(
    draft: &crate::FederalDraft,
    entry: &crate::FederalIncomeEntry,
) -> Result<NuaSale, crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue as Issue;
    let crate::FederalIncome::NuaSaleSource { data: paper } = &entry.income else {
        unreachable!("a printed NUA sale")
    };
    let answer = answers(draft, entry.id);
    let more = answer
        .and_then(|answer| answer.more.as_ref())
        .ok_or_else(|| Issue::new("NUA_SALE_ADJUSTMENTS_NEEDED").at_field("more"))?;
    let inheritance = if more.special {
        None
    } else {
        match answer.and_then(|answer| answer.ownership.as_ref()) {
            None => return Err(Issue::new("NUA_OWNERSHIP_NEEDED").at_field("ownership")),
            Some(NuaOwnership::Own) => None,
            Some(NuaOwnership::Inherited { inheritance }) => {
                let at = |issue: Issue, field: &'static str| {
                    issue
                        .at_field(field)
                        .in_field("inheritance")
                        .in_field("Inherited")
                        .in_field("ownership")
                };
                if inheritance.died < paper.distributed {
                    return Err(at(Issue::new("NUA_DIED_BEFORE_DISTRIBUTED"), "died"));
                }
                if inheritance.died > paper.sold {
                    return Err(at(Issue::new("NUA_SOLD_BEFORE_DIED"), "died"));
                }
                Some(inheritance.clone())
            }
        }
    };
    let sale = paper.lot(more, inheritance);
    if i128::from(sale.wash_sale) > (i128::from(sale.basis()) - i128::from(sale.proceeds)).max(0) {
        return Err(Issue::new("SALE_WASH_SALE_ABOVE_LOSS").at_field("wash_sale").in_field("more"));
    }
    if !sale.valid() {
        return Err(Issue::new(
            "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
        )
        .at_field("more"));
    }
    Ok(sale)
}

// Answers follow their printed sales: a removed sale's answers go.
pub(crate) fn reconcile(draft: &mut crate::FederalDraft) {
    let keep: alloc::vec::Vec<i64> = draft
        .income
        .iter()
        .filter(|entry| matches!(entry.income, crate::FederalIncome::NuaSaleSource { .. }))
        .map(|entry| entry.id)
        .collect();
    if let Some(screening) = &mut draft.screening {
        screening.nua_sale_answers.retain(|row| keep.contains(&row.document_id));
    }
}

// A complete sale saved before its answers moved to Investment sales becomes
// its printed sale on the next Document center save: its own answers come
// with it, so nothing it already said is asked again.
pub(crate) fn carry_lot_answers(before: &crate::FederalDraft, draft: &mut crate::FederalDraft) {
    let carried: alloc::vec::Vec<NuaSaleAnswers> = draft
        .income
        .iter()
        .filter(|entry| matches!(entry.income, crate::FederalIncome::NuaSaleSource { .. }))
        .filter(|entry| answers(draft, entry.id).is_none())
        .filter_map(|entry| {
            let crate::FederalIncome::NuaSale { data } =
                &before.income.iter().find(|old| old.id == entry.id)?.income
            else {
                return None;
            };
            Some(NuaSaleAnswers {
                document_id: entry.id,
                more: Some(NuaShareSaleMore { wash_sale: data.wash_sale, special: data.special }),
                ownership: Some(match &data.inheritance {
                    None => NuaOwnership::Own,
                    Some(inheritance) => {
                        NuaOwnership::Inherited { inheritance: inheritance.clone() }
                    }
                }),
            })
        })
        .collect();
    if !carried.is_empty() {
        crate::interview::fill_unasked_screening(draft);
        draft.screening.as_mut().unwrap().nua_sale_answers.extend(carried);
    }
}

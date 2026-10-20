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
    #[libertas_number(min = 0)]
    pub distribution_value: i64,
    /// Deferred NUA allocated to this sold lot
    /// From Form 1099-R box 6 and the plan's lot allocation. This amount was excluded at distribution and is not included in stock basis. Do not include appreciation after distribution.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub deferred_nua: i64,
    /// Inherited shares already distributed by the plan before death
    /// Leave absent for your own directly held shares. This is not a beneficiary distribution made by the retirement plan after death.
    pub inheritance: Option<NuaInheritance>,
    /// Net sale proceeds after fees
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub proceeds: i64,
    /// Disallowed wash-sale loss from replacement-share records
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub wash_sale: i64,
    /// Federal income tax withheld on this sale, not on the original distribution
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub withholding: i64,
    /// Basis the Form 1099-B printed in box 1e, when it shows one
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
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
    instruction = "Read the 1099-B sale of these employer shares exactly. Check its tax year against the supplied year; report a mismatch. The statement must show exactly one sale of these shares; if it shows several sales, report an error asking for only this sale. Return the box 1a description, or an empty string when it is blank. The box 1c date sold and box 1d proceeds must show values; report a blank one as an error. Return zero for a clearly blank box 4. Report illegible or conflicting values as errors. Return box 1e as reported_basis exactly as printed, or leave it empty when the box is blank, and return box 12 as reporting; do not return box 1b, and do not return Form 1099-R values even when a Form 1099-R is supplied; do not infer the distribution date, the value at distribution, the net unrealized appreciation, a wash sale, an inheritance or other facts absent from the 1099-B. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct NuaShareSale {
    /// 1099-B box 1a description
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_size(min = 0)]
    #[libertas_default("")]
    pub description: String,
    /// 1099-B box 1c date sold
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub sold: u32,
    /// 1099-B box 1d proceeds
    /// Net of selling fees, as the 1099-B reports them.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub proceeds: i64,
    /// 1099-B box 1e cost or other basis
    /// As printed, before any correction: the broker's figure leaves out the compensation or net unrealized appreciation, which the App adds with Form 8949 code B. Leave empty when the box is blank.
    /// [NullText]
    /// No cost basis is printed in box 1e
    /// [SetText]
    /// The cost basis printed in box 1e is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub reported_basis: Option<i64>,
    /// 1099-B box 12 basis reported to the IRS
    /// Box A or D when the broker reported the basis to the IRS, B or E when it did not, C or F when no Form 1099-B was received.
    #[libertas_llm_doc_field]
    #[libertas_default(ReportedToIrs)]
    pub reporting: crate::BasisReporting,
    /// 1099-B box 4 federal income tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Date the plan distributed these shares
    #[libertas_date_only]
    pub distributed: u32,
    /// Value of the shares sold on that date
    /// For only the shares sold, from the plan's distribution statement.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub distribution_value: i64,
    /// Their net unrealized appreciation
    /// The part of Form 1099-R box 6 for the shares sold. It is not in your basis; do not include growth after the distribution.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub deferred_nua: i64,
    /// About this sale
    pub more: NuaShareSaleMore,
}

/// About this sale
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct NuaShareSaleMore {
    /// Wash sale loss disallowed
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub wash_sale: i64,
    /// Inherited shares
    /// Only for shares the plan distributed to someone who died before the sale. Leave empty for your own shares.
    /// [NullText]
    /// These are my own shares
    /// [SetText]
    /// The inheritance details are
    pub inherited: Option<NuaInheritance>,
    /// A gift, a rollover, an election to tax the NUA when distributed, or another special treatment
    /// Yes leaves this sale out of the estimate with a notice; that treatment is not supported.
    #[libertas_default(false)]
    pub special: bool,
}

impl NuaShareSale {
    // The first relationship between this page's fields that fails, by its
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
        if let Some(v) = &self.more.inherited {
            if v.died < self.distributed {
                return Some(
                    crate::validation::ValidationIssue::new("NUA_DIED_BEFORE_DISTRIBUTED")
                        .at_field("died")
                        .in_field("inherited")
                        .in_field("more"),
                );
            }
            if v.died > self.sold {
                return Some(
                    crate::validation::ValidationIssue::new("NUA_SOLD_BEFORE_DIED")
                        .at_field("sold"),
                );
            }
        }
        let record = self.record();
        if i128::from(record.wash_sale)
            > (i128::from(record.basis()) - i128::from(record.proceeds)).max(0)
        {
            return Some(
                crate::validation::ValidationIssue::new("SALE_WASH_SALE_ABOVE_LOSS")
                    .at_field("wash_sale")
                    .in_field("more"),
            );
        }
        None
    }
    // The stored sale this page writes.
    pub(crate) fn record(&self) -> NuaSale {
        NuaSale {
            distributed: self.distributed,
            sold: self.sold,
            distribution_value: self.distribution_value,
            deferred_nua: self.deferred_nua,
            inheritance: self.more.inherited.clone(),
            proceeds: self.proceeds,
            wash_sale: self.more.wash_sale,
            withholding: self.withholding,
            reported_basis: self.reported_basis,
            reporting: self.reporting,
            special: self.more.special,
        }
    }
    // The reverse of record.
    pub(crate) fn from_record(label: &str, v: &NuaSale) -> Self {
        Self {
            description: label.into(),
            sold: v.sold,
            proceeds: v.proceeds,
            withholding: v.withholding,
            reported_basis: v.reported_basis,
            reporting: v.reporting,
            distributed: v.distributed,
            distribution_value: v.distribution_value,
            deferred_nua: v.deferred_nua,
            more: NuaShareSaleMore {
                wash_sale: v.wash_sale,
                inherited: v.inheritance.clone(),
                special: v.special,
            },
        }
    }
    // The entry label: the box 1a description as typed.
    pub(crate) fn label(&self) -> String {
        self.description.clone()
    }
}

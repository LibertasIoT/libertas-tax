// Coverage: Schedule K-1 (Form 1041) beneficiary source adapter — implementation in progress.
//   Source item A EIN retains leading zeroes; ASCII-digit validation on both
//   normal Save and restored records. No IRS issuance check.
//   Activity, AMT, credit and final-character declarations below are source
//   fitting in progress, not a claim that every tax owner is connected.
//   Known componentized community sources reuse Share's distinct income and
//   property-gain axes before beneficiary limits. Existing ratio Prefix owns
//   cent rounding in fixed28%-rate/1250/other and qualified/REIT/other source
//   categories; editable row order never owns a category tie. Regular/AMT
//   differences and source equalities are rebuilt from projected children.
//   Preferential codeE retains its projected monetary character. Raw-source
//   preflight retains mandatory boundaries for differential-axis unclassified
//   A/J/H; optional4952/QBI/NIIT source gaps retain their existing notices.
//   This independent correction preserves accepted source bytes and does not
//   close final11B investment interest, beneficiary at-risk, inheritedNOL/year,
//   home-office tax, cooperative QBI or text-printing owner dependencies.
//   Synthetic controls in tests/passthrough/unit27_estate_community.rs (not IRS tax totals):
//   unit27_estate_differential_share_reuses_actual_regular_and_amt_components;
//   unit27_estate_odd_cent_special_categories_conserve_parent_and_counterpart;
//   unit27_estate_signed_amt_underlying_loss_is_not_floored_to_zero;
//   unit27_estate_source_preflight_retains_unclassified_amt_and_niit_boundaries;
//   unit27_estate_optional_mismatched_investment_character_does_not_change_income;
//   unit27_estate_normal_save_reopen_restart_preserves_whole_source_and_correction_id;
//   unit27_estate_final_capital_parts_use_gain_axis_and_preserve_odd_cent_caps;
//   unit27_estate_shared_qbi_history_uses_existing_name_prefix_before_limits;
//   unit27_estate_optional_qbi_odd_cent_boundary_preserves_actual_source_components;
//   unit27_estate_raw_nonzero_amt_confirmations_remain_valid_when_projection_is_zero;
//   unit27_estate_zero_projected_qbi_vintage_is_omitted_without_changing_retained_source.
//   A <=6-day change FROM an elected52/53 year is retained with the following
//   ordinary taxable period, normal month-first beginning and actual dates.
//   It counts once and uses ordinary actual-start law applicability; no
//   elected-year nearest-month convention is applied to an ordinary year.
//   IRS TD8996 pp30-31, Reg1.441-2(b)(2)(ii); public source-chain fixture.
//   Final capital NIIT excluded-loss balances use the existing net-gains
//   worksheet once; missing balance omits only that investment-tax benefit.
//   Independent regular/AMT investment and business ST/LT carry portions feed
//   the existing Form4952 and current-NOL owners. Neither axis is inferred
//   from NIIT difference or the other axis. Missing classification omits only
//   its dependent benefit, retaining ordinary ScheduleD/AMT carry totals.
//   Fixtures: tests/passthrough/interview.rs k1_1041_mixed_capital_*; 2026 draft4952
//   printedp3 line4d, finalDec2024 Form172 lines2-22, Reg1.642(h)-1(b).
//   Current entity periods retain actual dates/election: a normal December-
//   reference52/53 closing in earlyJanuary2027 belongs to beneficiary2026,
//   Reg1.441-2(e). Ordinary/short sources follow1.652(c)/1.662(c). Shortened
//   elected years with differing actual/reference years need source-year
//   verification; final-NOL termination/history remains a separate owner.
//   Supported activity and nonbusiness royalty amounts retain Schedule E
//   numeric columns 33(c)-(f), totals 34a/34b and footing 35-37. Text
//   name/EIN printing depends on the separately owned text-row interface.
//   Unentered identified box14 B/C/D/E/G/J/K/L/M benefits carry named
//   notices without suppressing known ordinary income; missing codeE also
//   omits dependent Form4952 deduction. Unclassified/AMT/recapture sources
//   retain explicit complete-estimate limitations. Public-page lifecycle
//   fixtures: tests/passthrough/interview.rs k1_1041_unentered_*.
//   Sources: 2026 drafts f1041sk1/i1041sk1 and i1040se; TD9918; TD9847.
use crate::kernel::phase;
use crate::validation::ValidationIssue;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

pub(crate) fn ein_valid(value: &str) -> bool {
    // Legal EIN format: nine ASCII digits, stored without the printed hyphen.
    value.len() == 9 && value.bytes().all(|byte| byte.is_ascii_digit())
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateActivities {
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub box6: i64,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub box7: i64,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub box8: i64,
    pub activities: Vec<EstateActivity>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EstateActivityKind {
    /// Trade or business — box 6
    Business,
    /// Rental real estate — box 7
    RentalRealEstate,
    /// Other rental — box 8
    OtherRental,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateActivity {
    /// Activity reference
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    #[libertas_size(min = 1)]
    pub name: String,
    pub kind: EstateActivityKind,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub income: i64,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub depreciation: i64,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub depletion: i64,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub amortization: i64,
    /// Your participation in this activity
    pub participation: EstateParticipation,
    /// Prior suspended passive losses
    /// Copy the regular-tax and AMT balances for this activity from your prior worksheets. Do not include current-year amounts.
    pub carried: EstatePassiveCarry,
    /// Signed difference to activity net from fiduciary's AMT activity statement
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub amt_difference: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EstateParticipation {
    /// Passive activity
    Passive,
    /// You materially participated in the activity
    Material(crate::ScorpParticipation),
    /// Rental real estate treated as nonpassive
    /// Your real-estate-professional determination and participation records establish this treatment.
    Professional(crate::ScorpParticipation),
    /// Participation treatment is not determined
    /// Known income remains included; an unresolved loss benefit is left out with a notice.
    Unresolved,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstatePassiveCarry {
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub regular: i64,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub alternative: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateFinalCharacter {
    /// The estate or trust terminated
    pub entity_terminated: bool,
    /// Part of box 11 code A deductible against net investment income
    /// Copy the allocable share from the fiduciary statement. It cannot exceed the box 11 code A deduction.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section67e_nii: i64,
    /// The fiduciary included this deductible share in its box 14 code H amount
    pub section67e_in_h: bool,
    /// Other box 11 code B deductions
    /// Identify each deduction from the final fiduciary statement. State and local taxes are entered in the final-year boxes.
    pub additional: Vec<EstateFinalDeduction>,
    /// Capital loss carryover details
    /// Copy separate AMT balances and special long-term portions from the fiduciary statement.
    pub capital: EstateFinalCapital,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EstateFinalDeduction {
    /// Investment interest
    /// Copy the fiduciary statement amount. It is figured on Form 4952 with your own investment interest. It is allowed only this year, so your net investment income must absorb all of it.
    InvestmentInterest {
        #[libertas_size(min = 1)]
        description: String,
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
    /// Another deduction
    /// Describe the fiduciary statement item. This deduction is not calculated yet and is left out with a notice.
    Other {
        #[libertas_size(min = 1)]
        description: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateFinalCapital {
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    #[libertas_llm_doc_field]
    pub alternative_short: i64,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    #[libertas_llm_doc_field]
    pub alternative_long: i64,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    #[libertas_llm_doc_field]
    pub rate28: i64,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    #[libertas_llm_doc_field]
    pub unrecaptured: i64,
    /// Transferred capital-loss carryover excluded from net investment income
    /// Copy the positive excluded-loss difference from the fiduciary's retained Form 8960 loss worksheet, including confirmed zero. This is part of the regular short- and long-term carryovers, not another loss. Leave absent when unavailable; only the investment-tax loss benefit is then left out. Do not repeat a current box 14 H adjustment here.
    /// [NullText]
    /// The excluded-loss difference is not available
    /// [SetText]
    /// The documented excluded-loss difference is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub niit_difference: Option<i64>,
    /// Regular capital losses from property held for investment
    /// Copy the fiduciary's actual short/long portions for Form 4952, including confirmed zero. This classification is independent of net investment tax and business-loss character.
    /// [NullText]
    /// The regular-tax investment loss portions are not established
    /// [SetText]
    /// The documented regular-tax investment loss portions are
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub regular_investment: Option<EstateCapitalLossParts>,
    /// AMT capital losses from property held for investment
    /// Copy the separate AMT portions; do not assume the regular amounts apply.
    /// [NullText]
    /// The AMT investment loss portions are not established
    /// [SetText]
    /// The documented AMT investment loss portions are
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub alternative_investment: Option<EstateCapitalLossParts>,
    /// Regular business capital-loss portions
    /// Copy the actual business portions for Form 172. The remainder is nonbusiness. Leave absent when the fiduciary records do not establish character.
    /// [NullText]
    /// The regular-tax business loss portions are not established
    /// [SetText]
    /// The documented regular-tax business loss portions are
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub regular_business: Option<EstateCapitalLossParts>,
    /// AMT business capital-loss portions
    /// Copy the separate AMT business portions for the alternative NOL worksheet.
    /// [NullText]
    /// The AMT business loss portions are not established
    /// [SetText]
    /// The documented AMT business loss portions are
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub alternative_business: Option<EstateCapitalLossParts>,
}

/// Classified portions of an inherited capital-loss carryover
/// These are parts of the existing carryovers, never additional losses.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateCapitalLossParts {
    /// Short-term portion
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub short: i64,
    /// Long-term portion
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub long: i64,
}
impl EstateCapitalLossParts {
    fn invalid_field(&self, total: [i64; 2]) -> Option<&'static str> {
        if !MONEY.contains(&self.short) {
            Some("short")
        } else if !MONEY.contains(&self.long) {
            Some("long")
        } else if self.short > total[0] {
            Some("short")
        } else if self.long > total[1] {
            Some("long")
        } else {
            None
        }
    }
}
#[derive(Clone, Copy)]
pub(crate) enum CapitalLossUse {
    Investment,
    Business,
}

pub(crate) fn capital_parts(
    data: &crate::FederalEstateTrustPortfolioK1,
    alternative: bool,
    purpose: CapitalLossUse,
) -> Option<[i64; 2]> {
    let total = if alternative {
        data.final_character.as_ref().map_or([data.final_short_carry, data.final_long_carry], |f| {
            [f.capital.alternative_short, f.capital.alternative_long]
        })
    } else {
        [data.final_short_carry, data.final_long_carry]
    };
    if total == [0, 0] {
        return Some([0, 0]);
    }
    let c = &data.final_character.as_ref()?.capital;
    let p = match (alternative, purpose) {
        (false, CapitalLossUse::Investment) => &c.regular_investment,
        (true, CapitalLossUse::Investment) => &c.alternative_investment,
        (false, CapitalLossUse::Business) => &c.regular_business,
        (true, CapitalLossUse::Business) => &c.alternative_business,
    };
    p.as_ref().map(|p| [p.short, p.long])
}

// Projection only: sum actual retained source cents, then let the consuming
// existing tax owner round once with its other source totals.
pub(crate) fn capital_character(
    draft: &crate::FederalDraft,
    alternative: bool,
    purpose: CapitalLossUse,
) -> Option<[i64; 2]> {
    crate::community::entries(draft).iter().try_fold([0; 2], |mut sum, entry| {
        if let crate::FederalIncome::EstateTrustPortfolio { data } = &entry.income {
            let data = &crate::FederalEstateTrustPortfolioK1::from(data);
            let parts = capital_parts(data, alternative, purpose)?;
            sum[0] += parts[0];
            sum[1] += parts[1];
        }
        Some(sum)
    })
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateInvestmentIncome {
    /// Box 14 code E net investment income for regular tax
    /// Copy the statement amount, including its sign. This is not additional income on top of the income boxes.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub regular: i64,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub alternative: i64,
    /// Investment income character
    #[libertas_llm_doc_field]
    pub character: EstateInvestmentCharacter,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EstateInvestmentCharacter {
    Ordinary,
    /// Statement identifies qualified dividends or capital gain
    /// Copy their separate regular-tax and AMT portions.
    Preferential {
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_llm_doc_field(allow_empty = false)]
        qualified: i64,
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_llm_doc_field(allow_empty = false)]
        gains: i64,
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_llm_doc_field(allow_empty = false)]
        amt_qualified: i64,
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_llm_doc_field(allow_empty = false)]
        amt_gains: i64,
    },
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateQbiStatements {
    pub statements: Vec<EstateQbiStatement>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateQbiStatement {
    /// Accepted source-statement identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    #[libertas_size(min = 1)]
    pub name: String,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub income: i64,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub wages: i64,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub property: i64,
    /// The statement identifies a specified service trade or business
    pub service: bool,
    pub kind: EstateQbiKind,
    /// Activities identified by this statement
    /// Select only activities the fiduciary statement links to this QBI. An independent DNI allocation may have none.
    #[libertas_unordered]
    pub activities: Vec<i64>,
    /// Prior suspended qualified business losses
    /// Copy original-year amounts and eligibility from your retained QBI worksheets, not the current K-1.
    pub passive_history: Vec<crate::ScorpLossYear>,
    /// Fiduciary statement QBI apportioned to its linked activities
    /// Required when limits or retained losses affect several activities, or several statements share an activity. Copy signed source QBI and your retained activity history; their current amounts must sum to this statement. Do not derive the allocation from distributions.
    pub allocations: Vec<EstateQbiAllocation>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EstateQbiKind {
    Business,
    PubliclyTradedPartnership,
    Cooperative,
}

/// One activity's share of a fiduciary QBI statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateQbiAllocation {
    /// Activity identified in this statement
    #[libertas_number(min = 1)]
    pub activity: i64,
    /// Signed current QBI apportioned to this activity
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub income: i64,
    /// This statement's qualified share of the activity's prior loss vintages
    /// Retain the activity's full original and remaining tax losses, with only this statement's qualified portion. Other statements may share the tax loss, but cannot repeat its qualified portion.
    #[libertas_hidden]
    pub passive_history: Vec<crate::ScorpLossYear>,
}

/// One activity's share of a fiduciary QBI statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateQbiAllocationPage {
    /// Activity identified in this statement
    #[libertas_number(min = 1)]
    #[libertas_enum_source("^<K1EstateTrust>.estate_activities")]
    pub activity: i64,
    /// Signed current QBI apportioned to this activity
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub income: i64,
    /// This statement's qualified share of the activity's prior loss vintages
    /// Retain the activity's full original and remaining tax losses, with only this statement's qualified portion. Other statements may share the tax loss, but cannot repeat its qualified portion.
    pub passive_history: Vec<crate::ScorpLossYear>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateAmtReconciliation {
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub other_in_a: i64,
    pub adjustments: Vec<EstateAmtAdjustment>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateAmtAdjustment {
    /// Accepted source-statement identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    pub code: EstateAmtCode,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub amount: i64,
    pub treatment: EstateAmtTreatment,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EstateAmtCode {
    DepreciationG,
    DepletionH,
    AmortizationI,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EstateAmtTreatment {
    /// Already included in box 12 code A
    IncludedInA,
    Activity {
        /// This amount is already part of box 12 code A
        included_in_a: bool,
        #[libertas_number(min = 1)]
        activity: i64,
    },
    /// Separate Form 6251 adjustment
    /// Use the line identified by the fiduciary statement and the applicable Form 6251 instructions.
    Independent {
        line: EstateAmtLine,
        /// This amount is already part of box 12 code A
        included_in_a: bool,
    },
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EstateAmtLine {
    /// Depletion — Form 6251 line 2d
    Depletion2d,
    /// Depreciation — Form 6251 line 2l
    Depreciation2l,
    /// Pre-1987 depreciation — Form 6251 line 3
    Pre1987Depreciation3,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateReportedCredit {
    pub code: EstateCreditCode,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    #[libertas_unordered]
    pub activities: Vec<i64>,
}

/// Box 13 credit code
/// The Form 1041 letters, which differ from the partnership and S corporation codes.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EstateCreditCode {
    /// Code C — low-income housing credit
    #[libertas_form_code("C")]
    C,
    /// Code D — advanced manufacturing production credit
    #[libertas_form_code("D")]
    D,
    /// Code E — clean electricity production credit
    #[libertas_form_code("E")]
    E,
    /// Code F — work opportunity credit
    #[libertas_form_code("F")]
    F,
    /// Code G — credit for small employer health insurance premiums
    #[libertas_form_code("G")]
    G,
    /// Code H — biofuel producer credit
    #[libertas_form_code("H")]
    H,
    /// Code I — credit for increasing research activities
    #[libertas_form_code("I")]
    I {
        /// Form 3800 line
        /// Choose line 4i only when both you and the estate or trust meet the eligible-small-business test (average annual gross receipts of $50 million or less for the prior three years); otherwise line 1c. This is your own answer, not a printed K-1 fact.
        #[libertas_constrained_by("$.research_credit_lines")]
        line: crate::BusinessCredit,
    },
    /// Code J — renewable electricity production credit
    /// The fiduciary's statement splits the credit between production in a facility's first four years and later production. Enter each part in its own row.
    #[libertas_form_code("J")]
    J {
        /// Form 3800 line
        /// Use line 4e for the part the statement attributes to the facility's first four years of service; otherwise line 1f.
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_constrained_by("$.renewable_credit_lines")]
        line: crate::BusinessCredit,
    },
    /// Code K — empowerment zone employment credit
    #[libertas_form_code("K")]
    K,
    /// Code L — clean fuel production credit
    #[libertas_form_code("L")]
    L,
    /// Code M — orphan drug credit
    #[libertas_form_code("M")]
    M,
    /// Code N — credit for employer-provided childcare facilities and services
    #[libertas_form_code("N")]
    N,
    /// Code O — biodiesel and renewable diesel fuels credit
    #[libertas_form_code("O")]
    O,
    /// Code P — credit to holders of tax credit bonds
    #[libertas_form_code("P")]
    P,
    /// Code Q — credit for employer differential wage payments
    #[libertas_form_code("Q")]
    Q,
    /// Code S — credit for production from advanced nuclear power facilities
    #[libertas_form_code("S")]
    S,
    /// Code T — zero-emission nuclear power production credit
    #[libertas_form_code("T")]
    T,
    /// Code ZZ — other credits
    #[libertas_form_code("ZZ")]
    ZZ {
        /// Form 3800 line
        /// Use the Form 3800 line the fiduciary's credit statement names.
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_constrained_by("$.k1_credit_lines")]
        line: crate::BusinessCredit,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateForeignStatement {
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub tax: i64,
    /// Foreign-tax statement treatment
    #[libertas_llm_doc_field]
    pub treatment: EstateForeignTreatment,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EstateForeignTreatment {
    QualifiedPassivePayeeStatement,
    RequiresAllocation,
    /// Complete fiduciary allocation of passive interest and dividends
    /// Other income categories, capital gains and activity allocations need their own source worksheets.
    PassivePortfolio {
        /// The fiduciary's actual method for these foreign taxes
        /// Copy paid or accrued from the statement; do not infer it from your own election.
        #[libertas_llm_doc_field]
        basis: crate::ForeignTaxBasis,
        /// Country and payment-class allocations of the entire box 14 code B amount
        #[libertas_llm_doc_field]
        allocations: Vec<crate::EstateForeignPortfolioAllocation>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateSourceChoice {
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    #[libertas_read_only]
    pub id: i64,
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateFinalCharacterPage {
    /// The estate or trust terminated
    #[libertas_llm_doc_field]
    pub entity_terminated: bool,
    /// Part of box 11 code A deductible against net investment income
    /// Copy the allocable share from the fiduciary statement. It cannot exceed the box 11 code A deduction.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    #[libertas_llm_doc_field]
    pub section67e_nii: i64,
    /// The fiduciary included this deductible share in its box 14 code H amount
    #[libertas_llm_doc_field]
    pub section67e_in_h: bool,
    /// Other box 11 code B deductions
    /// Identify each deduction from the final fiduciary statement. State and local taxes are entered in the final-year boxes.
    #[libertas_llm_doc_field]
    pub additional: Vec<EstateFinalDeductionPage>,
    /// Capital loss carryover details
    /// Copy separate AMT balances and special long-term portions from the fiduciary statement.
    #[libertas_llm_doc_field]
    pub capital: EstateFinalCapital,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EstateFinalDeductionPage {
    /// Investment interest
    /// Copy the fiduciary statement amount. It is figured on Form 4952 with your own investment interest. It is allowed only this year, so your net investment income must absorb all of it.
    InvestmentInterest {
        #[libertas_size(min = 1)]
        #[libertas_llm_doc_field]
        description: String,
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_llm_doc_field(allow_empty = false)]
        amount: i64,
    },
    /// Another deduction
    /// Describe the fiduciary statement item. This deduction is not calculated yet and is left out with a notice.
    Other {
        #[libertas_size(min = 1)]
        #[libertas_llm_doc_field]
        description: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateQbiStatementsPage {
    pub statements: Vec<EstateQbiStatementPage>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the invoked beneficiary section 199A statement attached to Schedule K-1 (Form 1041), with the fiduciary apportioned signed QBI, W-2 wages, UBIA, SSTB and business/PTP/cooperative character. Use beneficiary allocations, never total trust amounts. Duplicate copies count once. Do not infer source links or prior-loss histories; preserve them on a corrected statement. Missing or illegible source amounts are errors.",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true,
    form_date_order = "MonthDayYear"
)]
pub struct EstateQbiStatementPage {
    /// Accepted source-statement identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_size(min = 1)]
    pub name: String,
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub income: i64,
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub wages: i64,
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub property: i64,
    /// The statement identifies a specified service trade or business
    #[libertas_llm_doc_field(allow_empty = false)]
    pub service: bool,
    #[libertas_llm_doc_field(allow_empty = false)]
    pub kind: EstateQbiKind,
    /// Activities identified by this statement
    /// Select only activities the fiduciary statement links to this QBI. An independent DNI allocation may have none.
    #[libertas_enum_source("^<K1EstateTrust>.estate_activities")]
    #[libertas_unordered]
    pub activities: Vec<i64>,
    /// Prior suspended qualified business losses
    /// Copy original-year amounts and eligibility from your retained QBI worksheets, not the current K-1.
    #[libertas_hidden]
    pub passive_history: Vec<crate::ScorpLossYear>,
    /// Fiduciary statement QBI apportioned to its linked activities
    /// Required when limits or retained losses affect several activities, or several statements share an activity. Copy signed source QBI and your retained activity history; their current amounts must sum to this statement. Do not derive the allocation from distributions.
    pub allocations: Vec<EstateQbiAllocationPage>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateAmtReconciliationPage {
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub other_in_a: i64,
    pub adjustments: Vec<EstateAmtAdjustmentPage>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read only this invoked beneficiary AMT source statement attached to Schedule K-1 (Form 1041). Copy its actual box 12 code G, H or I and signed allocated amount. Do not use trust-wide totals or infer a Form 6251 line, activity link, or whether the amount is already in code A. Preserve all unmarked treatment facts. Duplicate copies count once; missing, illegible or conflicting source amounts are errors.",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true,
    form_date_order = "MonthDayYear"
)]
pub struct EstateAmtAdjustmentPage {
    /// Accepted source-statement identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    #[libertas_llm_doc_field(allow_empty = false)]
    pub code: EstateAmtCode,
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub amount: i64,
    #[libertas_hidden]
    pub treatment: EstateAmtTreatmentPage,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EstateAmtTreatmentPage {
    /// Already included in box 12 code A
    IncludedInA,
    Activity {
        /// This amount is already part of box 12 code A
        included_in_a: bool,
        #[libertas_number(min = 1)]
        #[libertas_enum_source("^<K1EstateTrust>.estate_activities")]
        activity: i64,
    },
    /// Separate Form 6251 adjustment
    /// Use the line identified by the fiduciary statement and the applicable Form 6251 instructions.
    Independent {
        line: EstateAmtLine,
        /// This amount is already part of box 12 code A
        included_in_a: bool,
    },
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read only this invoked box 13 credit source statement attached to Schedule K-1 (Form 1041). Copy its actual Form 1041 code and allocated amount, not trust-wide totals. Preserve unmarked activity links; do not infer them. Duplicate copies count once. Missing or conflicting source data are errors.",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true,
    form_date_order = "MonthDayYear"
)]
pub struct EstateReportedCreditPage {
    #[libertas_llm_doc_field(allow_empty = false)]
    pub code: EstateCreditCode,
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    #[libertas_enum_source("^<K1EstateTrust>.estate_activities")]
    #[libertas_unordered]
    pub activities: Vec<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read printed Schedule K-1 (Form 1041) boxes 6, 7 and 8 exactly with their signs. A clearly blank box is zero; illegible or conflicting source data are errors. Preserve the unmarked separate activity records, identities, participation and historical balances; each activity row has its own current-source import action.",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true,
    form_date_order = "MonthDayYear"
)]
pub struct EstateActivitiesPage {
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub box6: i64,
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub box7: i64,
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub box8: i64,
    pub activities: Vec<EstateActivityPage>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read only the invoked beneficiary activity from the fiduciary attached Schedule K-1 (Form 1041) schedule. Read its source name, business/rental character, signed allocated income, separate codes 9A depreciation, 9B depletion, 9C amortization and signed AMT net difference. Clearly blank amounts are zero; missing required statements or illegible values are errors. Duplicate copies of the same activity count once. Do not supply participation, prior passive balances or identity. A corrected statement replaces current source facts in this row and retains personal records.",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true,
    form_date_order = "MonthDayYear"
)]
pub struct EstateActivityPage {
    /// Activity reference
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_size(min = 1)]
    pub name: String,
    #[libertas_llm_doc_field(allow_empty = false)]
    pub kind: EstateActivityKind,
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub income: i64,
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub depreciation: i64,
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub depletion: i64,
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub amortization: i64,
    /// Your participation in this activity
    #[libertas_hidden]
    pub participation: EstateParticipation,
    /// Prior suspended passive losses
    /// Copy the regular-tax and AMT balances for this activity from your prior worksheets. Do not include current-year amounts.
    #[libertas_hidden]
    pub carried: EstatePassiveCarry,
    /// Signed difference to activity net from fiduciary's AMT activity statement
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub amt_difference: i64,
}

// Arithmetic safety: activity net, qbi_projection and community_projection
// still combine native i64 source cents; capital/AMT receivers also add and
// subtract component amounts before their final reported-dollar transfer.
const MONEY: core::ops::RangeInclusive<i64> = 0..=100_000_000_000;
const SIGNED: core::ops::RangeInclusive<i64> = -100_000_000_000..=100_000_000_000;

impl EstateActivities {
    pub(crate) fn empty() -> Self {
        Self { box6: 0, box7: 0, box8: 0, activities: Vec::new() }
    }
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        self.issue_with_mode(true)
    }
    pub(crate) fn issue_with_mode(&self, require_answers: bool) -> Option<ValidationIssue> {
        // Arithmetic safety: box reconciliation and community_projection sum
        // signed native activity cents; extended_rules activity/history prefix
        // pools and qbi_projection also retain native i64 receivers.
        if self.activities.len() > 100
            || ![self.box6, self.box7, self.box8].iter().all(|a| SIGNED.contains(a))
        {
            return Some(
                ValidationIssue::new("ESTATE_ACTIVITY_AMOUNT_INVALID").at_field("activities"),
            );
        }
        for (i, a) in self.activities.iter().enumerate() {
            if let Some(issue) = if require_answers { a.issue() } else { a.issue_with_mode(false) }
            {
                return Some(issue.in_list("activities", i));
            }
            if a.id != 0 && self.activities[..i].iter().any(|old| old.id == a.id) {
                return Some(
                    ValidationIssue::new("ESTATE_ACTIVITY_ID_TWICE")
                        .at_field("id")
                        .in_list("activities", i),
                );
            }
            if self.activities[..i].iter().any(|old| old.name == a.name) {
                return Some(
                    ValidationIssue::new("ESTATE_ACTIVITY_NAME_TWICE")
                        .at_field("name")
                        .in_list("activities", i),
                );
            }
        }
        for (kind, total, key, field) in [
            (EstateActivityKind::Business, self.box6, "ESTATE_BOX6_RECONCILE", "box6"),
            (EstateActivityKind::RentalRealEstate, self.box7, "ESTATE_BOX7_RECONCILE", "box7"),
            (EstateActivityKind::OtherRental, self.box8, "ESTATE_BOX8_RECONCILE", "box8"),
        ] {
            if self.activities.iter().filter(|a| a.kind == kind).map(|a| a.income).sum::<i64>()
                != total
            {
                return Some(ValidationIssue::new(key).at_field(field));
            }
        }
        None
    }
}
impl EstateActivity {
    pub(crate) fn net(&self, alternative: bool) -> i64 {
        self.income - self.depreciation - self.depletion - self.amortization
            + if alternative { self.amt_difference } else { 0 }
    }
    pub(crate) fn prior(&self, alternative: bool) -> i64 {
        if alternative { self.carried.alternative } else { self.carried.regular }
    }
    pub(crate) fn material(&self) -> bool {
        match &self.participation {
            EstateParticipation::Material(p) | EstateParticipation::Professional(p) => {
                !p.left_out() && p.material() == crate::participation::Participation::Yes
            }
            _ => false,
        }
    }
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        self.issue_with_mode(true)
    }
    pub(crate) fn issue_with_mode(&self, require_answers: bool) -> Option<ValidationIssue> {
        if self.id < 0
            || self.name.is_empty()
            || !SIGNED.contains(&self.income)
            || !SIGNED.contains(&self.amt_difference)
            || ![
                self.depreciation,
                self.depletion,
                self.amortization,
                self.carried.regular,
                self.carried.alternative,
            ]
            .iter()
            .all(|a| MONEY.contains(a))
        {
            return Some(ValidationIssue::new("ESTATE_ACTIVITY_AMOUNT_INVALID").at_field("income"));
        }
        match &self.participation {
            EstateParticipation::Material(p)
                if (require_answers && self.kind != EstateActivityKind::Business) || !p.valid() =>
            {
                Some(
                    ValidationIssue::new("ESTATE_ACTIVITY_PARTICIPATION").at_field("participation"),
                )
            }
            EstateParticipation::Professional(p)
                if (require_answers && self.kind != EstateActivityKind::RentalRealEstate)
                    || !p.valid() =>
            {
                Some(
                    ValidationIssue::new("ESTATE_ACTIVITY_PARTICIPATION").at_field("participation"),
                )
            }
            _ => None,
        }
    }
}
impl EstateQbiStatements {
    /// Statements apportioned to one activity agree on their printed service and kind:
    /// the first disagreeing statement's index and that field.
    pub(crate) fn kind_conflict(&self, activity: i64) -> Option<(usize, &'static str)> {
        let owners: Vec<_> = self
            .statements
            .iter()
            .enumerate()
            .filter(|(_, s)| s.allocations.iter().any(|a| a.activity == activity))
            .collect();
        owners.iter().find_map(|(_, s)| {
            owners.iter().find(|(_, other)| s.service != other.service || s.kind != other.kind).map(
                |(index, other)| {
                    (*index, if s.service != other.service { "service" } else { "kind" })
                },
            )
        })
    }
    /// The first statement whose activity apportionment is required and absent:
    /// it shares a linked activity with another statement, or links several
    /// activities one of which carries a prior passive loss. Without it the
    /// statement's QBI is never calculable (qbi_projection leaves it out).
    pub(crate) fn apportionment_needed(&self, activities: &EstateActivities) -> Option<usize> {
        self.statements.iter().position(|s| {
            s.allocations.is_empty()
                && (s.activities.iter().any(|id| {
                    self.statements.iter().filter(|other| other.activities.contains(id)).count() > 1
                }) || (s.activities.len() > 1
                    && activities
                        .activities
                        .iter()
                        .any(|a| s.activities.contains(&a.id) && a.carried.regular > 0)))
        })
    }
    pub(crate) fn rule(&self, activities: &EstateActivities) -> Option<&'static str> {
        self.issue(activities).map(|issue| issue.key)
    }
    pub(crate) fn issue(&self, activities: &EstateActivities) -> Option<ValidationIssue> {
        self.issue_with_mode(activities, true)
    }
    pub(crate) fn issue_with_mode(
        &self,
        activities: &EstateActivities,
        require_answers: bool,
    ) -> Option<ValidationIssue> {
        // Arithmetic safety: qbi_projection and community_projection retain
        // native current/history accumulation across statements; extended_rules
        // activity/history prefix pools also retain native i64 receivers.
        if self.statements.len() > 100 {
            return Some(ValidationIssue::new("ESTATE_QBI_AMOUNT_INVALID").at_field("statements"));
        }
        for (i, s) in self.statements.iter().enumerate() {
            if s.id < 0
                || s.name.is_empty()
                || !SIGNED.contains(&s.income)
                || !MONEY.contains(&s.wages)
                || !MONEY.contains(&s.property)
                || !crate::scorp_qbi::valid_history(&s.passive_history, None)
            {
                return Some(
                    ValidationIssue::new("ESTATE_QBI_AMOUNT_INVALID")
                        .at_field("income")
                        .in_list("statements", i),
                );
            }
            if self.statements[..i].iter().any(|old| old.name == s.name) {
                return Some(
                    ValidationIssue::new("ESTATE_QBI_SOURCE_TWICE")
                        .at_field("name")
                        .in_list("statements", i),
                );
            }
            if require_answers
                && !s.passive_history.is_empty()
                && s.activities.len() == 1
                && !crate::scorp_qbi::valid_history(
                    &s.passive_history,
                    activities
                        .activities
                        .iter()
                        .find(|a| a.id == s.activities[0])
                        .map(|a| a.carried.regular),
                )
            {
                return Some(
                    ValidationIssue::new("ESTATE_QBI_HISTORY_RECONCILE")
                        .at_field("passive_history")
                        .in_list("statements", i),
                );
            }
            if !s.allocations.is_empty() {
                let field = if s.allocations.len() != s.activities.len() {
                    Some("allocations")
                } else if require_answers && !s.passive_history.is_empty() {
                    Some("passive_history")
                } else if s.allocations.iter().map(|a| i128::from(a.income)).sum::<i128>()
                    != i128::from(s.income)
                {
                    Some("income")
                } else {
                    None
                };
                if let Some(field) = field {
                    return Some(
                        ValidationIssue::new("ESTATE_QBI_ALLOCATION_RECONCILE")
                            .at_field(field)
                            .in_list("statements", i),
                    );
                }
                for (position, part) in s.allocations.iter().enumerate() {
                    if !SIGNED.contains(&part.income)
                        || !s.activities.contains(&part.activity)
                        || s.allocations[..position].iter().any(|a| a.activity == part.activity)
                    {
                        return Some(
                            ValidationIssue::new("ESTATE_QBI_ALLOCATION_RECONCILE")
                                .at_field("activity")
                                .in_list("allocations", position)
                                .in_list("statements", i),
                        );
                    }
                    let Some(activity) =
                        activities.activities.iter().find(|a| a.id == part.activity)
                    else {
                        return Some(
                            ValidationIssue::new("ESTATE_ACTIVITY_LINK_INVALID")
                                .at_field("activity")
                                .in_list("allocations", position)
                                .in_list("statements", i),
                        );
                    };
                    if !crate::scorp_qbi::valid_history(&part.passive_history, None)
                        || (require_answers
                            && !part.passive_history.is_empty()
                            && !crate::scorp_qbi::valid_history(
                                &part.passive_history,
                                Some(activity.carried.regular),
                            ))
                    {
                        return Some(
                            ValidationIssue::new("ESTATE_QBI_HISTORY_RECONCILE")
                                .at_field("passive_history")
                                .in_list("allocations", position)
                                .in_list("statements", i),
                        );
                    }
                }
            }
            for (j, id) in s.activities.iter().enumerate() {
                if s.activities[..j].contains(id) {
                    return Some(
                        ValidationIssue::new("ESTATE_SELECTED_ACTIVITY_TWICE")
                            .at_list_item("activities", j)
                            .in_list("statements", i),
                    );
                }
                if *id <= 0 || !activities.activities.iter().any(|a| a.id == *id) {
                    return Some(
                        ValidationIssue::new("ESTATE_ACTIVITY_LINK_INVALID")
                            .at_list_item("activities", j)
                            .in_list("statements", i),
                    );
                }
            }
        }
        for activity in &activities.activities {
            if let Some((index, field)) = self.kind_conflict(activity.id) {
                return Some(
                    ValidationIssue::new("ESTATE_QBI_ALLOCATION_RECONCILE")
                        .at_field(field)
                        .in_list("statements", index),
                );
            }
            let parts: Vec<_> = self
                .statements
                .iter()
                .enumerate()
                .flat_map(|(s, statement)| {
                    statement.allocations.iter().enumerate().map(move |(p, part)| (s, p, part))
                })
                .filter(|(_, _, part)| part.activity == activity.id)
                .collect();
            if parts.iter().map(|(_, _, a)| i128::from((-a.income).max(0))).sum::<i128>()
                > i128::from((-activity.net(false)).max(0))
            {
                let (s, p, _) = parts
                    .iter()
                    .find(|(_, _, part)| part.income < 0)
                    .expect("positive allocated loss has a negative part");
                return Some(
                    ValidationIssue::new("ESTATE_QBI_ALLOCATION_RECONCILE")
                        .at_field("income")
                        .in_list("allocations", *p)
                        .in_list("statements", *s),
                );
            }
            if require_answers {
                for (_, _, part) in &parts {
                    for year in &part.passive_history {
                        let histories: Vec<_> = parts
                            .iter()
                            .flat_map(|(s, p, a)| {
                                a.passive_history
                                    .iter()
                                    .enumerate()
                                    .map(move |(y, v)| (*s, *p, y, v))
                            })
                            .filter(|(_, _, _, v)| v.year == year.year)
                            .collect();
                        let failure = if let Some((s, p, y, v)) =
                            histories.iter().find(|(_, _, _, v)| {
                                v.original != year.original || v.remaining != year.remaining
                            }) {
                            Some((
                                *s,
                                *p,
                                *y,
                                if v.original != year.original { "original" } else { "remaining" },
                            ))
                        } else if histories
                            .iter()
                            .map(|(_, _, _, v)| i128::from(v.qualified))
                            .sum::<i128>()
                            > i128::from(year.original)
                        {
                            histories
                                .iter()
                                .rev()
                                .find(|(_, _, _, v)| v.qualified > 0)
                                .map(|(s, p, y, _)| (*s, *p, *y, "qualified"))
                        } else if histories
                            .iter()
                            .map(|(_, _, _, v)| i128::from(v.remaining_qualified))
                            .sum::<i128>()
                            > i128::from(year.remaining)
                        {
                            histories
                                .iter()
                                .rev()
                                .find(|(_, _, _, v)| v.remaining_qualified > 0)
                                .map(|(s, p, y, _)| (*s, *p, *y, "remaining_qualified"))
                        } else {
                            None
                        };
                        if let Some((s, p, y, field)) = failure {
                            return Some(
                                ValidationIssue::new("ESTATE_QBI_HISTORY_RECONCILE")
                                    .at_field(field)
                                    .in_list("passive_history", y)
                                    .in_list("allocations", p)
                                    .in_list("statements", s),
                            );
                        }
                    }
                }
            }
        }
        None
    }
}
impl EstateActivitiesPage {
    pub(crate) fn record(&self) -> EstateActivities {
        EstateActivities {
            box6: self.box6,
            box7: self.box7,
            box8: self.box8,
            activities: self.activities.iter().map(EstateActivityPage::record).collect(),
        }
    }
    pub(crate) fn from_record(r: &EstateActivities) -> Self {
        Self {
            box6: r.box6,
            box7: r.box7,
            box8: r.box8,
            activities: r.activities.iter().map(EstateActivityPage::from_record).collect(),
        }
    }
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        self.record().issue()
    }
}
impl EstateActivityPage {
    pub(crate) fn record(&self) -> EstateActivity {
        EstateActivity {
            id: self.id,
            name: self.name.clone(),
            kind: self.kind.clone(),
            income: self.income,
            depreciation: self.depreciation,
            depletion: self.depletion,
            amortization: self.amortization,
            participation: self.participation.clone(),
            carried: self.carried.clone(),
            amt_difference: self.amt_difference,
        }
    }
    pub(crate) fn from_record(r: &EstateActivity) -> Self {
        Self {
            id: r.id,
            name: r.name.clone(),
            kind: r.kind.clone(),
            income: r.income,
            depreciation: r.depreciation,
            depletion: r.depletion,
            amortization: r.amortization,
            participation: r.participation.clone(),
            carried: r.carried.clone(),
            amt_difference: r.amt_difference,
        }
    }
}
impl EstateQbiStatementsPage {
    pub(crate) fn issue(&self, activities: &EstateActivities) -> Option<ValidationIssue> {
        self.record().issue(activities)
    }

    pub(crate) fn record(&self) -> EstateQbiStatements {
        EstateQbiStatements {
            statements: self.statements.iter().map(EstateQbiStatementPage::record).collect(),
        }
    }
    pub(crate) fn from_record(r: &EstateQbiStatements) -> Self {
        Self { statements: r.statements.iter().map(EstateQbiStatementPage::from_record).collect() }
    }
}
impl EstateQbiStatementPage {
    pub(crate) fn record(&self) -> EstateQbiStatement {
        EstateQbiStatement {
            id: self.id,
            name: self.name.clone(),
            income: self.income,
            wages: self.wages,
            property: self.property,
            service: self.service,
            kind: self.kind.clone(),
            activities: self.activities.clone(),
            passive_history: self.passive_history.clone(),
            allocations: self
                .allocations
                .iter()
                .map(|a| EstateQbiAllocation {
                    activity: a.activity,
                    income: a.income,
                    passive_history: a.passive_history.clone(),
                })
                .collect(),
        }
    }
    pub(crate) fn from_record(r: &EstateQbiStatement) -> Self {
        Self {
            id: r.id,
            name: r.name.clone(),
            income: r.income,
            wages: r.wages,
            property: r.property,
            service: r.service,
            kind: r.kind.clone(),
            activities: r.activities.clone(),
            passive_history: r.passive_history.clone(),
            allocations: r
                .allocations
                .iter()
                .map(|a| EstateQbiAllocationPage {
                    activity: a.activity,
                    income: a.income,
                    passive_history: a.passive_history.clone(),
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EstateActivityResult {
    pub source: i64,
    pub activity: i64,
    pub current: i64,
    pub prior: i64,
    pub passive: i64,
    pub nonpassive: i64,
    pub suspended: i64,
    pub prior_released: i64,
}

pub(crate) fn activity_ids(income: &crate::FederalIncome) -> impl Iterator<Item = i64> + '_ {
    match income {
        crate::FederalIncome::EstateTrustPortfolio { data } => {
            Some(data.statements.activities.iter())
        }
        _ => None,
    }
    .into_iter()
    .flatten()
    .map(|a| a.id)
}
pub(crate) fn activity_ids_mut(
    income: &mut crate::FederalIncome,
) -> impl Iterator<Item = &mut i64> {
    match income {
        crate::FederalIncome::EstateTrustPortfolio { data } => {
            Some(data.statements.activities.iter_mut())
        }
        _ => None,
    }
    .into_iter()
    .flatten()
    .map(|a| &mut a.id)
}
pub(crate) fn activity_identity_issue(
    entries: &[crate::FederalIncomeEntry],
    accepted: &[crate::FederalIncomeEntry],
) -> Option<(usize, ValidationIssue)> {
    for (entry_index, entry) in entries.iter().enumerate() {
        let crate::FederalIncome::EstateTrustPortfolio { data } = &entry.income else {
            continue;
        };
        let data = &crate::FederalEstateTrustPortfolioK1::from(data);
        let prior = accepted.iter().find(|old| old.id == entry.id);
        for (index, activity) in data.activities.activities.iter().enumerate() {
            if activity.id != 0
                && !prior.is_some_and(|old| activity_ids(&old.income).any(|id| id == activity.id))
            {
                return Some((
                    entry_index,
                    ValidationIssue::new("ESTATE_ACTIVITY_LINK_INVALID")
                        .at_list_item("activities", index)
                        .in_field("statements"),
                ));
            }
        }
    }
    None
}

// The source ID belongs to FederalIncomeEntry, not a submitted list position.
// Local accepted paths map activities -> Page business and final_character ->
// Page final_character; the Save owner performs that explicit projection.
pub(crate) fn graph_issue(d: &crate::FederalDraft) -> Option<(i64, ValidationIssue)> {
    graph_issue_with_mode(d, true)
}
pub(crate) fn graph_issue_with_mode(
    d: &crate::FederalDraft,
    require_answers: bool,
) -> Option<(i64, ValidationIssue)> {
    if require_answers && let Some(issue) = mortgage_graph_issue(d) {
        return Some(issue);
    }
    for entry in &d.income {
        let crate::FederalIncome::EstateTrustPortfolio { data } = &entry.income else {
            continue;
        };
        let data = &crate::FederalEstateTrustPortfolioK1::from(data);
        for (index, activity) in data.activities.activities.iter().enumerate() {
            if activity.id <= 0 {
                return Some((
                    entry.id,
                    ValidationIssue::new("ESTATE_ACTIVITY_LINK_INVALID")
                        .at_field("id")
                        .in_list("activities", index)
                        .in_field("activities"),
                ));
            }
            if require_answers
                && matches!(activity.participation, EstateParticipation::Professional(_))
                && d.professional(crate::community::professional_owner(
                    d.setup.as_ref(),
                    entry.owner.owned(),
                ))
                .is_none()
            {
                return Some((
                    entry.id,
                    ValidationIssue::new("RENTAL_PROFESSIONAL_NEEDS_STATUS")
                        .at_field("participation")
                        .in_list("activities", index)
                        .in_field("activities"),
                ));
            }
        }
    }
    None
}

pub(crate) fn activity_choices(activities: &EstateActivities) -> Vec<EstateSourceChoice> {
    activities
        .activities
        .iter()
        .filter(|a| a.id > 0)
        .map(|a| EstateSourceChoice {
            id: a.id,
            label: crate::Message::new(
                "LABEL_ESTATE_ACTIVITY",
                alloc::vec![crate::MessageArg::Text(a.name.clone())],
            )
            .encode(),
        })
        .collect()
}

impl EstateFinalCharacterPage {
    pub(crate) fn record(&self) -> EstateFinalCharacter {
        EstateFinalCharacter {
            entity_terminated: self.entity_terminated,
            section67e_nii: self.section67e_nii,
            section67e_in_h: self.section67e_in_h,
            capital: self.capital.clone(),
            additional: self
                .additional
                .iter()
                .map(|item| match item {
                    EstateFinalDeductionPage::InvestmentInterest { description, amount } => {
                        EstateFinalDeduction::InvestmentInterest {
                            description: description.clone(),
                            amount: *amount,
                        }
                    }
                    EstateFinalDeductionPage::Other { description } => {
                        EstateFinalDeduction::Other { description: description.clone() }
                    }
                })
                .collect(),
        }
    }
    pub(crate) fn from_record(r: &EstateFinalCharacter) -> Self {
        Self {
            entity_terminated: r.entity_terminated,
            section67e_nii: r.section67e_nii,
            section67e_in_h: r.section67e_in_h,
            capital: r.capital.clone(),
            additional: r
                .additional
                .iter()
                .map(|item| match item {
                    EstateFinalDeduction::InvestmentInterest { description, amount } => {
                        EstateFinalDeductionPage::InvestmentInterest {
                            description: description.clone(),
                            amount: *amount,
                        }
                    }
                    EstateFinalDeduction::Other { description } => {
                        EstateFinalDeductionPage::Other { description: description.clone() }
                    }
                })
                .collect(),
        }
    }
}
impl EstateAmtTreatmentPage {
    pub(crate) fn from_record(value: &EstateAmtTreatment) -> Self {
        match value {
            EstateAmtTreatment::IncludedInA => Self::IncludedInA,
            EstateAmtTreatment::Activity { activity, included_in_a } => {
                Self::Activity { activity: *activity, included_in_a: *included_in_a }
            }
            EstateAmtTreatment::Independent { line, included_in_a } => {
                Self::Independent { line: line.clone(), included_in_a: *included_in_a }
            }
            EstateAmtTreatment::Unavailable => Self::Unavailable,
        }
    }
}
impl EstateAmtReconciliationPage {
    pub(crate) fn record(&self) -> EstateAmtReconciliation {
        EstateAmtReconciliation {
            other_in_a: self.other_in_a,
            adjustments: self
                .adjustments
                .iter()
                .map(|a| EstateAmtAdjustment {
                    id: a.id,
                    code: a.code.clone(),
                    amount: a.amount,
                    treatment: match &a.treatment {
                        EstateAmtTreatmentPage::IncludedInA => EstateAmtTreatment::IncludedInA,
                        EstateAmtTreatmentPage::Activity { activity, included_in_a } => {
                            EstateAmtTreatment::Activity {
                                activity: *activity,
                                included_in_a: *included_in_a,
                            }
                        }
                        EstateAmtTreatmentPage::Independent { line, included_in_a } => {
                            EstateAmtTreatment::Independent {
                                line: line.clone(),
                                included_in_a: *included_in_a,
                            }
                        }
                        EstateAmtTreatmentPage::Unavailable => EstateAmtTreatment::Unavailable,
                    },
                })
                .collect(),
        }
    }
    pub(crate) fn from_record(r: &EstateAmtReconciliation) -> Self {
        Self {
            other_in_a: r.other_in_a,
            adjustments: r
                .adjustments
                .iter()
                .map(|a| EstateAmtAdjustmentPage {
                    id: a.id,
                    code: a.code.clone(),
                    amount: a.amount,
                    treatment: EstateAmtTreatmentPage::from_record(&a.treatment),
                })
                .collect(),
        }
    }
}
impl EstateReportedCreditPage {
    pub(crate) fn record(&self) -> EstateReportedCredit {
        EstateReportedCredit {
            code: self.code.clone(),
            amount: self.amount,
            activities: self.activities.clone(),
        }
    }
    pub(crate) fn from_record(r: &EstateReportedCredit) -> Self {
        Self { code: r.code.clone(), amount: r.amount, activities: r.activities.clone() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateInvestmentProperty {
    pub property: crate::K1InvestmentProperty,
    pub activities: Vec<i64>,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateInvestmentPropertyPage {
    pub property: crate::K1InvestmentProperty,
    #[libertas_enum_source("^<K1EstateTrust>.estate_activities")]
    pub activities: Vec<i64>,
}
impl EstateInvestmentPropertyPage {
    pub(crate) fn record(&self) -> EstateInvestmentProperty {
        EstateInvestmentProperty {
            property: self.property.clone(),
            activities: self.activities.clone(),
        }
    }
    pub(crate) fn from_record(r: &EstateInvestmentProperty) -> Self {
        Self { property: r.property.clone(), activities: r.activities.clone() }
    }
}

pub(crate) fn source_rule(r: &crate::FederalEstateTrustPortfolioK1) -> Option<&'static str> {
    source_issue(r).map(|issue| issue.key)
}

pub(crate) fn source_issue(r: &crate::FederalEstateTrustPortfolioK1) -> Option<ValidationIssue> {
    source_issue_with_mode(r, true)
}
pub(crate) fn source_issue_with_mode(
    r: &crate::FederalEstateTrustPortfolioK1,
    require_answers: bool,
) -> Option<ValidationIssue> {
    if r.minimum_tax_exclusion.is_some_and(|amount| !SIGNED.contains(&amount)) {
        return Some(
            ValidationIssue::new("ESTATE_MINIMUM_TAX_EXCLUSION_INVALID")
                .at_field("minimum_tax_exclusion"),
        );
    }
    if let Some(p) = &r.entity_period {
        let period_issue = if require_answers {
            p.issue()
        } else {
            p.period.issue().map(|issue| issue.in_field("period"))
        };
        if let Some(mut issue) = period_issue {
            issue.key = "ESTATE_SOURCE_PERIOD_INVALID";
            return Some(issue.in_field("entity_period"));
        }
        if require_answers {
            let year = match &p.accounting {
                crate::FinalEstateTrustAccountingYear::CalendarFiscalOrShort
                | crate::FinalEstateTrustAccountingYear::CalendarFiscalAfterWeeks52Or53 {
                    ..
                } => p.period.ended / 10000,
                crate::FinalEstateTrustAccountingYear::Weeks52Or53 {
                    annual_ended,
                    reference_ended,
                    ..
                } if reference_ended / 100 % 100 == 12 => {
                    // Reg1.441-2(e) covers the normal same-reference-month
                    // closing. A shortened year with a different actual year
                    // needs its own source attribution; never guess that year.
                    if p.period.ended != *annual_ended
                        && p.period.ended / 10000 != reference_ended / 10000
                    {
                        return Some(
                            ValidationIssue::new("ESTATE_SOURCE_PERIOD_INVALID")
                                .at_field("ended")
                                .in_field("period")
                                .in_field("entity_period"),
                        );
                    }
                    reference_ended / 10000
                }
                crate::FinalEstateTrustAccountingYear::Weeks52Or53 { .. } => p.period.ended / 10000,
            };
            if year != 2026 {
                return Some(
                    ValidationIssue::new("ESTATE_SOURCE_PERIOD_INVALID")
                        .at_field("ended")
                        .in_field("period")
                        .in_field("entity_period"),
                );
            }
        }
    }
    for (index, loan) in r.final_mortgages.iter().enumerate() {
        if *loan <= 0 || r.final_mortgages[..index].contains(loan) {
            return Some(
                ValidationIssue::new("ESTATE_FINAL_CHARACTER_INVALID")
                    .at_list_item("final_mortgages", index),
            );
        }
    }
    if let Some(nol) = &r.final_nol
        && let Some(issue) = nol.issue()
    {
        return Some(issue.in_field("final_nol"));
    }

    if let Some(finals) = &r.final_character {
        let key = "ESTATE_FINAL_CHARACTER_INVALID";
        if !MONEY.contains(&finals.section67e_nii) || finals.section67e_nii > r.final_67e_deduction
        {
            return Some(
                ValidationIssue::new(key).at_field("section67e_nii").in_field("final_character"),
            );
        }
        for (field, amount) in [
            ("alternative_short", finals.capital.alternative_short),
            ("alternative_long", finals.capital.alternative_long),
            ("rate28", finals.capital.rate28),
            ("unrecaptured", finals.capital.unrecaptured),
        ] {
            if !MONEY.contains(&amount) {
                return Some(
                    ValidationIssue::new(key)
                        .at_field(field)
                        .in_field("capital")
                        .in_field("final_character"),
                );
            }
        }
        if finals.capital.rate28 + finals.capital.unrecaptured > r.final_long_carry {
            return Some(
                ValidationIssue::new(key)
                    .at_field("rate28")
                    .in_field("capital")
                    .in_field("final_character"),
            );
        }
        for (field, parts, total) in [
            (
                "regular_investment",
                &finals.capital.regular_investment,
                [r.final_short_carry, r.final_long_carry],
            ),
            (
                "regular_business",
                &finals.capital.regular_business,
                [r.final_short_carry, r.final_long_carry],
            ),
            (
                "alternative_investment",
                &finals.capital.alternative_investment,
                [finals.capital.alternative_short, finals.capital.alternative_long],
            ),
            (
                "alternative_business",
                &finals.capital.alternative_business,
                [finals.capital.alternative_short, finals.capital.alternative_long],
            ),
        ] {
            if let Some(parts) = parts
                && let Some(leaf) = parts.invalid_field(total)
            {
                return Some(
                    ValidationIssue::new(key)
                        .at_field(leaf)
                        .in_field(field)
                        .in_field("capital")
                        .in_field("final_character"),
                );
            }
        }
        if finals.capital.niit_difference.is_some_and(|amount| {
            !MONEY.contains(&amount) || amount > r.final_short_carry + r.final_long_carry
        }) {
            return Some(
                ValidationIssue::new(key)
                    .at_field("niit_difference")
                    .in_field("capital")
                    .in_field("final_character"),
            );
        }
        if finals.section67e_in_h && r.niit_adjustment.is_none() && finals.section67e_nii > 0 {
            return Some(
                ValidationIssue::new(key).at_field("section67e_in_h").in_field("final_character"),
            );
        }
        for (index, item) in finals.additional.iter().enumerate() {
            let field = match item {
                EstateFinalDeduction::InvestmentInterest { description, amount } => {
                    if description.is_empty() {
                        Some("description")
                    } else if !MONEY.contains(amount) {
                        Some("amount")
                    } else {
                        None
                    }
                }
                EstateFinalDeduction::Other { description } if description.is_empty() => {
                    Some("description")
                }
                _ => None,
            };
            if let Some(field) = field {
                return Some(
                    ValidationIssue::new(key)
                        .at_field(field)
                        .in_list("additional", index)
                        .in_field("final_character"),
                );
            }
        }
    }
    // Arithmetic safety: federal_rules creates a CreditEntry per property;
    // general_business_credit entry/category/carry totals retain native i64
    // receivers after investment_credit computes each property credit.
    if r.investment_property.len() > 10
        || r.investment_property.iter().any(|item| !item.property.valid())
    {
        return Some(
            ValidationIssue::new("ESTATE_CREDIT_AMOUNT_INVALID").at_field("investment_property"),
        );
    }
    for (property_index, item) in r.investment_property.iter().enumerate() {
        for (i, id) in item.activities.iter().enumerate() {
            if item.activities[..i].contains(id) {
                return Some(
                    ValidationIssue::new("ESTATE_SELECTED_ACTIVITY_TWICE")
                        .at_list_item("activities", i)
                        .in_list("investment_property", property_index),
                );
            }
            if *id <= 0 || !r.activities.activities.iter().any(|a| a.id == *id) {
                return Some(
                    ValidationIssue::new("ESTATE_ACTIVITY_LINK_INVALID")
                        .at_list_item("activities", i)
                        .in_list("investment_property", property_index),
                );
            }
        }
    }
    for (index, credit) in r.reported_credits.iter().enumerate() {
        if !(1..=100_000_000_000).contains(&credit.amount) {
            return Some(
                ValidationIssue::new("ESTATE_CREDIT_AMOUNT_INVALID")
                    .at_field("amount")
                    .in_list("reported_credits", index),
            );
        }
        if !credit_line_allowed(&credit.code) {
            return Some(
                ValidationIssue::new("ESTATE_CREDIT_LINE_INVALID")
                    .at_field("line")
                    .in_field("code")
                    .in_list("reported_credits", index),
            );
        }
        if r.reported_credits[..index].iter().any(|old| old.code == credit.code) {
            return Some(
                ValidationIssue::new("ESTATE_CREDIT_CODE_TWICE")
                    .at_field("code")
                    .in_list("reported_credits", index),
            );
        }
        for (i, id) in credit.activities.iter().enumerate() {
            if credit.activities[..i].contains(id) {
                return Some(
                    ValidationIssue::new("ESTATE_SELECTED_ACTIVITY_TWICE")
                        .at_list_item("activities", i)
                        .in_list("reported_credits", index),
                );
            }
            if *id <= 0 || !r.activities.activities.iter().any(|a| a.id == *id) {
                return Some(
                    ValidationIssue::new("ESTATE_ACTIVITY_LINK_INVALID")
                        .at_list_item("activities", i)
                        .in_list("reported_credits", index),
                );
            }
        }
    }
    if let Some(foreign) = &r.foreign_statement
        && !MONEY.contains(&foreign.tax)
    {
        return Some(
            ValidationIssue::new("ESTATE_FOREIGN_AMOUNT_INVALID").at_field("foreign_statement"),
        );
    }
    if let Some(issue) = crate::estate_foreign::source_issue(r) {
        return Some(issue);
    }
    if let Some(investment) = &r.investment_income {
        if !SIGNED.contains(&investment.regular) || !SIGNED.contains(&investment.alternative) {
            return Some(
                ValidationIssue::new("ESTATE_INVESTMENT_AMOUNT_INVALID")
                    .at_field("investment_income"),
            );
        }
        if let EstateInvestmentCharacter::Preferential {
            qualified,
            gains,
            amt_qualified,
            amt_gains,
        } = investment.character
            && (![qualified, gains, amt_qualified, amt_gains].iter().all(|a| MONEY.contains(a))
                || qualified + gains > phase::floor(investment.regular, 0)
                || amt_qualified + amt_gains > phase::floor(investment.alternative, 0))
        {
            return Some(
                ValidationIssue::new("ESTATE_INVESTMENT_AMOUNT_INVALID")
                    .at_field("investment_income"),
            );
        }
    }
    if let Some(amt) = &r.amt_reconciliation {
        if !SIGNED.contains(&amt.other_in_a) {
            return Some(
                ValidationIssue::new("ESTATE_AMT_RECONCILE")
                    .at_field("other_in_a")
                    .in_field("amt_reconciliation"),
            );
        }
        // Arithmetic safety: activity reconciliation and community_projection
        // still sum signed native AMT adjustment cents. federal_rules Totals
        // estate_amt_reallocated/independent also use signed i64 additions
        // before their final Form 6251 dollar receivers.
        if amt.adjustments.len() > 100 {
            return Some(
                ValidationIssue::new("ESTATE_AMT_RECONCILE")
                    .at_field("adjustments")
                    .in_field("amt_reconciliation"),
            );
        }
        if let Some(index) = amt.adjustments.iter().position(|a| !SIGNED.contains(&a.amount)) {
            return Some(
                ValidationIssue::new("ESTATE_AMT_RECONCILE")
                    .at_field("amount")
                    .in_list("adjustments", index)
                    .in_field("amt_reconciliation"),
            );
        }
        if r.amt_box12a != r.amt_box12b + r.amt_box12c + r.amt_box12d + amt.other_in_a {
            return Some(
                ValidationIssue::new("ESTATE_AMT_RECONCILE")
                    .at_field("other_in_a")
                    .in_field("amt_reconciliation"),
            );
        }
        if r.rate_28_gain + r.amt_box12f < 0 {
            return Some(ValidationIssue::new("ESTATE_AMT_RECONCILE").at_field("rate_28_gain"));
        }
        if r.unrecaptured_gain + r.amt_box12e < 0 {
            return Some(
                ValidationIssue::new("ESTATE_AMT_RECONCILE").at_field("unrecaptured_gain"),
            );
        }
        if r.rate_28_gain + r.amt_box12f + r.unrecaptured_gain + r.amt_box12e
            > phase::floor(r.long_gain + r.amt_box12d, 0)
        {
            return Some(ValidationIssue::new("ESTATE_AMT_RECONCILE").at_field("long_gain"));
        }
        if require_answers {
            for (adjustment_index, adjustment) in amt.adjustments.iter().enumerate() {
                if let EstateAmtTreatment::Independent { line, .. } = &adjustment.treatment
                    && !matches!(
                        (&adjustment.code, line),
                        (EstateAmtCode::DepletionH, EstateAmtLine::Depletion2d)
                            | (
                                EstateAmtCode::DepreciationG,
                                EstateAmtLine::Depreciation2l | EstateAmtLine::Pre1987Depreciation3
                            )
                    )
                {
                    return Some(
                        ValidationIssue::new("ESTATE_AMT_LINE_INVALID")
                            .at_field("line")
                            .in_field("treatment")
                            .in_list("adjustments", adjustment_index)
                            .in_field("amt_reconciliation"),
                    );
                }
                if let EstateAmtTreatment::Activity { activity, .. } = adjustment.treatment
                    && !r.activities.activities.iter().any(|a| a.id == activity)
                {
                    return Some(
                        ValidationIssue::new("ESTATE_ACTIVITY_LINK_INVALID")
                            .at_field("activity")
                            .in_field("treatment")
                            .in_list("adjustments", adjustment_index)
                            .in_field("amt_reconciliation"),
                    );
                }
            }
            for (activity_index, activity) in r.activities.activities.iter().enumerate() {
                let allocated: i64 = amt.adjustments.iter().filter(|a| matches!(a.treatment, EstateAmtTreatment::Activity { activity: id, .. } if id == activity.id)).map(|a| a.amount).sum();
                if allocated != activity.amt_difference {
                    return Some(
                        ValidationIssue::new("ESTATE_AMT_ACTIVITY_RECONCILE")
                            .at_field("amt_difference")
                            .in_list("activities", activity_index)
                            .in_field("activities"),
                    );
                }
            }
        }
    } else if require_answers
        && let Some(activity_index) =
            r.activities.activities.iter().position(|a| a.amt_difference != 0)
    {
        return Some(
            ValidationIssue::new("ESTATE_AMT_ACTIVITY_RECONCILE")
                .at_field("amt_difference")
                .in_list("activities", activity_index)
                .in_field("activities"),
        );
    }
    None
}

// The Form 3800 Part III credit each 2026 Schedule K-1 (Form 1041) box 13 code
// reports (2026 Instructions for Schedule K-1 (Form 1041), box 13; Form 3800
// instructions, Part III). When the estate or trust is the only source of the
// credit, the beneficiary reports it directly on Form 3800. Code H goes on
// line 4z, as those instructions direct, because the section 40 credit ended
// for fuel sold after 2024. Code P is the Form 8912 bond credit, which is not
// a general business credit and is not figured for a pass-through holder.
pub(crate) fn credit_identity(code: &EstateCreditCode) -> Option<crate::BusinessCredit> {
    use crate::BusinessCredit as B;
    Some(match code {
        EstateCreditCode::C => B::LowIncomeHousing,
        EstateCreditCode::D => B::AdvancedManufacturingProduction,
        EstateCreditCode::E => B::CleanElectricityProduction,
        EstateCreditCode::F => B::WorkOpportunity,
        EstateCreditCode::G => B::SmallEmployerHealth,
        EstateCreditCode::H => B::OtherSpecified,
        EstateCreditCode::I { line }
        | EstateCreditCode::J { line }
        | EstateCreditCode::ZZ { line } => *line,
        EstateCreditCode::K => B::EmpowermentZone,
        EstateCreditCode::L => B::CleanFuel,
        EstateCreditCode::M => B::OrphanDrug,
        EstateCreditCode::N => B::EmployerChildcare,
        EstateCreditCode::O => B::Biodiesel,
        EstateCreditCode::P => return None,
        EstateCreditCode::Q => B::DifferentialWage,
        EstateCreditCode::S => B::AdvancedNuclear,
        EstateCreditCode::T => B::ZeroEmissionNuclear,
    })
}
// The Form 3800 lines a statement-supplied line may name, the same lists the
// partnership codes M (research), AB (renewable electricity) and ZZ allow.
fn credit_line_allowed(code: &EstateCreditCode) -> bool {
    match code {
        EstateCreditCode::I { line } => crate::K1CreditCode::Research.line_allows(*line),
        EstateCreditCode::J { line } => {
            crate::K1CreditCode::RenewableElectricity.line_allows(*line)
        }
        EstateCreditCode::ZZ { line } => crate::K1CreditCode::Other.line_allows(*line),
        _ => true,
    }
}
pub(crate) fn credit_letters(code: &EstateCreditCode) -> &'static str {
    match code {
        EstateCreditCode::C => "C",
        EstateCreditCode::D => "D",
        EstateCreditCode::E => "E",
        EstateCreditCode::F => "F",
        EstateCreditCode::G => "G",
        EstateCreditCode::H => "H",
        EstateCreditCode::I { .. } => "I",
        EstateCreditCode::J { .. } => "J",
        EstateCreditCode::K => "K",
        EstateCreditCode::L => "L",
        EstateCreditCode::M => "M",
        EstateCreditCode::N => "N",
        EstateCreditCode::O => "O",
        EstateCreditCode::P => "P",
        EstateCreditCode::Q => "Q",
        EstateCreditCode::S => "S",
        EstateCreditCode::T => "T",
        EstateCreditCode::ZZ { .. } => "ZZ",
    }
}
pub(crate) fn credit_passive(activities: &[EstateActivity], ids: &[i64]) -> Option<bool> {
    let mut passive = None;
    for id in ids {
        let activity = activities.iter().find(|a| a.id == *id)?;
        if matches!(activity.participation, EstateParticipation::Unresolved)
            || matches!(&activity.participation, EstateParticipation::Material(p) | EstateParticipation::Professional(p) if p.left_out())
        {
            return None;
        }
        let value = !activity.material();
        if passive.is_some_and(|old| old != value) {
            return None;
        }
        passive = Some(value);
    }
    passive
}

// One statement's current qualified source and the actually released historical
// loss, projected after the existing passive owner. Never apply PAL percentages
// to unrelated DNI, PTP pools or a second statement sharing the same loss.
pub(crate) struct QbiProjection {
    pub current: crate::business_rules::QbiActivity,
    pub historical: i64,
    pub remaining: Vec<crate::scorp_basis::LossVintage<crate::kernel::carry::Dollars>>,
    pub history_missing: bool,
}
pub(crate) fn qbi_projection(
    source: i64,
    data: &crate::FederalEstateTrustPortfolioK1,
    statement: &EstateQbiStatement,
    results: &[EstateActivityResult],
    taxable: i64,
    status: crate::FederalStatus,
) -> Result<QbiProjection, &'static str> {
    // Independent source-component rounding can leave an optional qualified
    // portion unsupported by this return's actual tax loss. Keep the known
    // income and the existing omission boundary rather than alter source gross.
    if data.qbi.rule(&data.activities).is_some() {
        return Err("WARN_ESTATE_QBI_SOURCE_LEFT_OUT");
    }
    if !statement.allocations.is_empty() {
        let mut combined: Option<QbiProjection> = None;
        for part in &statement.allocations {
            // Prefix differences preserve one rounding result for a shared
            // activity's qualified portions. Names provide a stable source
            // order; the existing vintage owner still releases every loss.
            let project = |inclusive: bool| {
                let mut piece = statement.clone();
                piece.income = 0;
                piece.activities = alloc::vec![part.activity];
                piece.passive_history.clear();
                piece.allocations.clear();
                for owner in data.qbi.statements.iter().filter(|owner| {
                    owner.name < statement.name || (inclusive && owner.name == statement.name)
                }) {
                    for allocation in
                        owner.allocations.iter().filter(|a| a.activity == part.activity)
                    {
                        piece.income += allocation.income;
                        for year in &allocation.passive_history {
                            if let Some(existing) =
                                piece.passive_history.iter_mut().find(|y| y.year == year.year)
                            {
                                existing.qualified += year.qualified;
                                existing.remaining_qualified += year.remaining_qualified;
                            } else {
                                piece.passive_history.push(year.clone());
                            }
                        }
                    }
                }
                let mut source_data = data.clone();
                source_data.qbi.statements = alloc::vec![piece.clone()];
                qbi_projection(source, &source_data, &piece, results, taxable, status)
            };
            let mut projected = project(true)?;
            let previous = project(false)?;
            projected.current.income -= previous.current.income;
            projected.historical -= previous.historical;
            for year in &mut projected.remaining {
                if let Some(previous) = previous.remaining.iter().find(|v| v.year == year.year) {
                    year.original_qbi = crate::kernel::carry::closing_values(
                        year.original_qbi.get(),
                        previous.original_qbi.get(),
                    )
                    .map_err(|_| "WARN_ESTATE_QBI_SOURCE_LEFT_OUT")?;
                    year.remaining_qbi = crate::kernel::carry::closing_values(
                        year.remaining_qbi.get(),
                        previous.remaining_qbi.get(),
                    )
                    .map_err(|_| "WARN_ESTATE_QBI_SOURCE_LEFT_OUT")?;
                }
            }
            projected.history_missing = data.activities.activities.iter().any(|a| {
                a.id == part.activity && a.carried.regular > 0 && part.passive_history.is_empty()
            });
            if let Some(total) = &mut combined {
                total.current.income += projected.current.income;
                total.current.active &= projected.current.active;
                total.historical += projected.historical;
                total.remaining.extend(projected.remaining);
                total.history_missing |= projected.history_missing;
            } else {
                combined = Some(projected);
            }
        }
        return combined.ok_or("WARN_ESTATE_QBI_SOURCE_LEFT_OUT");
    }
    let mut current = crate::business_rules::eligible_qbi_activity(
        &crate::business_rules::QbiActivity {
            source_wages_cents: None,
            source_property_cents: None,
            source_income_cents: None,
            owner: None,
            income: crate::kernel::ratio::dollars(statement.income),
            wages: crate::kernel::ratio::dollars(statement.wages),
            property: crate::kernel::ratio::dollars(statement.property),
            active: !statement.activities.is_empty()
                && statement.activities.iter().all(|id| {
                    data.activities.activities.iter().any(|a| a.id == *id && a.material())
                }),
            service: Some(statement.service),
            group: None,
        },
        taxable,
        status,
    )
    .map_err(|_| "WARN_ESTATE_QBI_SOURCE_LEFT_OUT")?;
    if statement.activities.is_empty() {
        if !statement.passive_history.is_empty() {
            return Err("WARN_ESTATE_QBI_SOURCE_LEFT_OUT");
        }
        return Ok(QbiProjection {
            current,
            historical: 0,
            remaining: Vec::new(),
            history_missing: false,
        });
    }
    // A fiduciary statement may identify several fully allowed material
    // activities. Their reported QBI is already one source amount: no PAL
    // fraction or historical loss allocation is needed at this endpoint.
    if statement.activities.len() > 1
        && statement.passive_history.is_empty()
        && statement.activities.iter().all(|id| {
            data.qbi.statements.iter().filter(|s| s.activities.contains(id)).count() == 1
                && data.activities.activities.iter().any(|a| {
                    a.id == *id
                        && a.material()
                        && a.carried.regular == 0
                        && results.iter().any(|r| {
                            r.source == source
                                && r.activity == *id
                                && r.current == crate::kernel::ratio::dollars(a.net(false))
                                && r.nonpassive == r.current
                                && r.prior == 0
                                && r.suspended == 0
                                && r.prior_released == 0
                        })
                })
        })
    {
        return Ok(QbiProjection {
            current,
            historical: 0,
            remaining: Vec::new(),
            history_missing: false,
        });
    }
    if statement.activities.len() != 1
        || data
            .qbi
            .statements
            .iter()
            .filter(|s| s.activities.contains(&statement.activities[0]))
            .count()
            != 1
    {
        return Err("WARN_ESTATE_QBI_SOURCE_LEFT_OUT");
    }
    let activity = data
        .activities
        .activities
        .iter()
        .find(|a| a.id == statement.activities[0])
        .ok_or("WARN_ESTATE_QBI_SOURCE_LEFT_OUT")?;
    if matches!(activity.participation, EstateParticipation::Unresolved)
        || matches!(&activity.participation, EstateParticipation::Material(p) | EstateParticipation::Professional(p) if p.left_out())
    {
        return Err("WARN_ESTATE_QBI_SOURCE_LEFT_OUT");
    }
    let result = results
        .iter()
        .find(|r| r.source == source && r.activity == activity.id)
        .ok_or("WARN_ESTATE_QBI_SOURCE_LEFT_OUT")?;
    let missing = result.prior > 0 && statement.passive_history.is_empty();
    if !missing
        && !crate::scorp_qbi::valid_history(
            &statement.passive_history,
            Some(activity.carried.regular),
        )
    {
        return Err("WARN_ESTATE_QBI_SOURCE_LEFT_OUT");
    }
    // Estate amounts have already crossed their owning annual report boundary.
    // Keep that dollar quantum in the same generic vintage allocator.
    let report_amount = |dollars: i64| {
        crate::kernel::carry::Amount::<crate::kernel::carry::Dollars>::new(dollars)
            .map_err(|_| "WARN_ESTATE_QBI_SOURCE_LEFT_OUT")
    };
    let history = if missing {
        Vec::new()
    } else {
        statement
            .passive_history
            .iter()
            .filter(|year| crate::kernel::ratio::dollars(year.original) != 0)
            .map(|year| {
                Ok(crate::scorp_basis::LossVintage {
                    year: year.year,
                    original: report_amount(crate::kernel::ratio::dollars(year.original))?,
                    original_qbi: report_amount(crate::kernel::ratio::dollars(year.qualified))?,
                    remaining: report_amount(crate::kernel::ratio::dollars(year.remaining))?,
                    remaining_qbi: report_amount(crate::kernel::ratio::dollars(
                        year.remaining_qualified,
                    ))?,
                })
            })
            .collect::<Result<Vec<_>, &'static str>>()?
    };
    let allowed_loss = phase::floor(
        phase::cap(result.current, [0]).saturating_neg() + result.prior - result.suspended,
        0,
    );
    let prior_allowed = if activity.material() {
        phase::floor(result.prior - result.suspended, 0)
    } else {
        phase::cap(allowed_loss, [result.prior])
    };
    let prior = crate::scorp_basis::release_losses(
        &history,
        report_amount(if missing { 0 } else { prior_allowed })?,
    )
    .map_err(|_| "WARN_ESTATE_QBI_SOURCE_LEFT_OUT")?;
    let mut remaining = prior.remaining;
    if result.current < 0 {
        if current.income > 0 || -current.income > -result.current {
            return Err("WARN_ESTATE_QBI_SOURCE_LEFT_OUT");
        }
        let current_loss = -result.current;
        let current_allowed = if activity.material() {
            current_loss
        } else {
            phase::cap(allowed_loss - prior_allowed, [current_loss])
        };
        let release = crate::scorp_basis::release_losses(
            &[crate::scorp_basis::LossVintage {
                year: 2026,
                original: report_amount(current_loss)?,
                original_qbi: report_amount(-current.income)?,
                remaining: report_amount(current_loss)?,
                remaining_qbi: report_amount(-current.income)?,
            }],
            report_amount(current_allowed)?,
        )
        .map_err(|_| "WARN_ESTATE_QBI_SOURCE_LEFT_OUT")?;
        current.income = -release.qbi.get();
        remaining.extend(crate::scorp_qbi::finalize_current_vintage(release).remaining);
    }
    Ok(QbiProjection { current, historical: prior.qbi.get(), remaining, history_missing: missing })
}

pub(crate) fn mortgage_graph_issue(d: &crate::FederalDraft) -> Option<(i64, ValidationIssue)> {
    let mut used = Vec::new();
    for e in &d.income {
        let crate::FederalIncome::EstateTrustPortfolio { data } = &e.income else {
            continue;
        };
        let data = &crate::FederalEstateTrustPortfolioK1::from(data);
        if data.final_character.is_none() {
            continue;
        }
        for (index, loan) in data.final_mortgages.iter().enumerate() {
            if !d.deductions.as_ref().is_some_and(|v| {
                v.mortgage_loans().unwrap_or_default().iter().any(|l| {
                    l.id == *loan && matches!(l.details, crate::MortgageLoanDetails::FirstLender(_))
                })
            }) {
                return Some((
                    e.id,
                    ValidationIssue::new("ESTATE_MORTGAGE_LINK_INVALID")
                        .at_list_item("final_mortgages", index),
                ));
            }
            if used.contains(loan) {
                return Some((
                    e.id,
                    ValidationIssue::new("ESTATE_MORTGAGE_LINK_TWICE")
                        .at_list_item("final_mortgages", index),
                ));
            }
            used.push(*loan);
        }
    }
    None
}

// Project actual fiduciary investment-income character into Form 4952;
// its existing gain and qualified-dividend election retains preferential amounts.
pub(crate) fn investment_adjustment(
    data: &crate::FederalEstateTrustPortfolioK1,
) -> Option<[i64; 2]> {
    let Some(source) = &data.investment_income else {
        return Some([0; 2]);
    };
    // Unknown-benefit preflight can inspect restored data before its normal
    // validity gate. Check only this adapter's arithmetic; unrelated optional
    // source classification does not decide an investment-interest benefit.
    let regular_gains = data.short_gain.checked_add(data.long_gain)?;
    let alternative_qualified = portfolio_qualified(data)?;
    let alternative_gains =
        regular_gains.checked_add(data.amt_box12c)?.checked_add(data.amt_box12d)?;
    let preferential = match source.character {
        EstateInvestmentCharacter::Ordinary => {
            if data.qualified_dividends != 0
                || data.short_gain != 0
                || data.long_gain != 0
                || data.amt_box12b != 0
                || data.amt_box12c != 0
                || data.amt_box12d != 0
            {
                return None;
            }
            [0; 2]
        }
        EstateInvestmentCharacter::Preferential { qualified, gains, amt_qualified, amt_gains }
            if qualified == data.qualified_dividends
                && gains == phase::floor(regular_gains, 0)
                && amt_qualified == phase::floor(alternative_qualified, 0)
                && amt_gains == phase::floor(alternative_gains, 0) =>
        {
            [qualified.checked_add(gains)?, amt_qualified.checked_add(amt_gains)?]
        }
        _ => return None,
    };
    // This source replaces its ordinary investment subtotal. The existing
    // disposition pool and single global election retain the matching gain
    // and qualified-dividend portions, independently for regular tax and AMT.
    let subtotal = data
        .interest
        .checked_add(data.ordinary_dividends)?
        .checked_sub(data.qualified_dividends)?
        .checked_add(data.royalty_income)?
        .checked_sub(data.box_9_royalty_depreciation)?
        .checked_sub(data.box_9_royalty_depletion)?
        .checked_sub(data.box_9_royalty_amortization)?;
    Some([
        source.regular.checked_sub(preferential[0])?.checked_sub(subtotal)?,
        source.alternative.checked_sub(preferential[1])?.checked_sub(subtotal)?,
    ])
}

// Runtime projection into the existing regular/AMT loss owners. Accepted
// fiduciary origins remain separate; compatible same-original-year balances
// are pooled once, without assigning source priority or changing original years.
#[derive(Default)]
pub(crate) struct LossProjection {
    pub vintages: Vec<LossVintage>,
    pub single_estate: Option<i64>,
    pub has_estate: bool,
}
#[derive(Clone)]
pub(crate) struct LossVintage {
    pub origin: Option<i64>,
    pub year: u32,
    pub post_2017: bool,
    pub balance: crate::kernel::ratio::exact::Exact,
    pub special: bool,
    pub expiration: Option<u32>,
    pub period: Option<(u32, u32)>,
}
impl LossVintage {
    pub(crate) fn eligible(&self) -> bool {
        self.post_2017
            || self
                .expiration
                .is_some_and(|year| year >= crate::year_constants::TAX_YEAR_END_DATE / 10000)
    }
    pub(crate) fn expires(&self) -> bool {
        !self.post_2017
            && self
                .expiration
                .is_some_and(|year| year <= crate::year_constants::TAX_YEAR_END_DATE / 10000)
    }
    pub(crate) fn expires_now(&self) -> bool {
        !self.post_2017 && self.expiration == Some(crate::year_constants::TAX_YEAR_END_DATE / 10000)
    }
    fn chronology(&self) -> (u32, u32) {
        self.period.unwrap_or((self.year * 10000 + 101, self.year * 10000 + 1231))
    }
    fn key(&self) -> (i64, u32, u32, u32) {
        let (began, ended) = self.period.unwrap_or((0, 0));
        (self.origin.unwrap_or(0), self.year, began, ended)
    }
}
pub(crate) fn loss_vintages(d: &crate::FederalDraft, alternative: bool) -> Vec<LossVintage> {
    let mut sources = Vec::new();
    if let Some(screen) = &d.screening {
        if alternative {
            if let crate::AtnolHistory::Records { years, confirmed: true } = &screen.atnol {
                for y in years.iter().filter(|y| y.balance > 0) {
                    sources.push(LossVintage {
                        origin: None,
                        year: y.year,
                        post_2017: y.year >= 2018,
                        balance: crate::kernel::ratio::exact::whole(y.balance),
                        special: y.special_exception,
                        expiration: (y.year < 2018).then_some(y.year + 20),
                        period: Some((y.year * 10000 + 101, y.year * 10000 + 1231)),
                    });
                }
            }
        } else if let crate::NolHistory::Records { years, confirmed: true } = &screen.nol {
            for y in years.iter().filter(|y| y.balance > 0) {
                sources.push(LossVintage {
                    origin: None,
                    year: y.year,
                    post_2017: y.year >= 2018,
                    balance: crate::kernel::ratio::exact::whole(y.balance),
                    special: false,
                    expiration: (y.year < 2018).then_some(y.year + 20),
                    period: Some((y.year * 10000 + 101, y.year * 10000 + 1231)),
                });
            }
        }
    }
    let current = crate::year_constants::TAX_YEAR_END_DATE / 10000;
    for entry in &crate::community::entries(d) {
        let crate::FederalIncome::EstateTrustPortfolio { data } = &entry.income else {
            continue;
        };
        let data = &crate::FederalEstateTrustPortfolioK1::from(data);
        let Some(nol) = &data.final_nol else { continue };
        if !nol.valid() {
            continue;
        }
        for y in &nol.years {
            let balance = if alternative { y.alternative_balance } else { y.regular_balance };
            if balance > 0 {
                let Some(post_2017) =
                    crate::FinalEstateTrustNol::post_2017(y, &nol.taxable_periods)
                else {
                    continue;
                };
                sources.push(LossVintage {
                    origin: Some(entry.id),
                    year: y.year,
                    post_2017,
                    balance: crate::kernel::ratio::exact::whole(balance),
                    special: false,
                    expiration: y.remaining_years.map(|n| current + n - 1),
                    period: y.original_period.as_ref().map(|p| (p.began, p.ended)),
                });
            }
        }
    }
    sources
}

// No primary rule establishing a priority between distinct succeeded origins
// with incompatible finite periods or AMT characters is inferred. The affected
// inherited benefit is omitted by name before the shared owner sees the pool.
pub(crate) fn loss_conflicts(
    d: &crate::FederalDraft,
) -> alloc::collections::BTreeMap<(i64, u32, u32, u32), (bool, bool)> {
    let mut conflicts = alloc::collections::BTreeMap::new();
    for alternative in [false, true] {
        let sources = loss_vintages(d, alternative);
        for source in &sources {
            if source.origin.is_none() {
                continue;
            }
            if sources.iter().any(|other| {
                other.year == source.year
                    && (other.expiration != source.expiration || other.special != source.special)
                    && !matches!((source.period, other.period), (Some((a, b)), Some((c, e))) if b < c || e < a)
            }) {
                let bits = conflicts.entry(source.key()).or_insert((false, false));
                if alternative {
                    bits.1 = true;
                } else {
                    bits.0 = true;
                }
            }
        }
    }
    conflicts
}

pub(crate) fn losses(d: &crate::FederalDraft, alternative: bool) -> LossProjection {
    pooled_loss_vintages(loss_vintages(d, alternative))
}

pub(crate) fn pooled_loss_vintages(sources: Vec<LossVintage>) -> LossProjection {
    let ids: alloc::collections::BTreeSet<_> = sources.iter().filter_map(|s| s.origin).collect();
    let own = sources.iter().any(|s| s.origin.is_none());
    let mut result = LossProjection {
        single_estate: (!own && ids.len() == 1).then(|| *ids.first().unwrap()),
        has_estate: !ids.is_empty(),
        ..Default::default()
    };
    // Pool only true equal original periods with equal character and expiry.
    // Original dates order distinct short years; no source-ID or expiry priority.
    let mut cohorts = alloc::collections::BTreeMap::<
        (u32, Option<(u32, u32)>, Option<u32>, bool, bool),
        LossVintage,
    >::new();
    for source in sources {
        let key = (source.year, source.period, source.expiration, source.special, source.post_2017);
        cohorts.entry(key).and_modify(|v| v.balance += &source.balance).or_insert(source);
    }
    result.vintages = cohorts.into_values().collect();
    result.vintages.sort_by_key(LossVintage::chronology);
    result
}

fn included_in_a(treatment: &EstateAmtTreatment) -> bool {
    matches!(
        treatment,
        EstateAmtTreatment::IncludedInA
            | EstateAmtTreatment::Activity { included_in_a: true, .. }
            | EstateAmtTreatment::Independent { included_in_a: true, .. }
    )
}

fn classified_other_in_a(data: &crate::FederalEstateTrustPortfolioK1) -> Option<i64> {
    let other = data.amt_reconciliation.as_ref().map_or(0, |source| source.other_in_a);
    if data.amt_box12a_standalone && data.amt_box12a != 0
        || data.amt_box12a
            != data
                .amt_box12b
                .checked_add(data.amt_box12c)?
                .checked_add(data.amt_box12d)?
                .checked_add(other)?
    {
        return None;
    }
    let identified = data
        .amt_reconciliation
        .as_ref()
        .map(|source| {
            source
                .adjustments
                .iter()
                .filter(|item| included_in_a(&item.treatment))
                .try_fold(0i64, |sum, item| sum.checked_add(item.amount))
        })
        .unwrap_or(Some(0))?;
    (identified == other).then_some(other)
}

/// Inspect the accepted source before allocation; aggregate adjustments cannot
/// establish their own income/property-gain axis by their projected amount.
pub(crate) fn community_source_rule(
    draft: &crate::FederalDraft,
    entry: &crate::FederalIncomeEntry,
) -> Option<&'static str> {
    let crate::FederalIncome::EstateTrustPortfolio { data } = &entry.income else {
        return None;
    };
    let data = &crate::FederalEstateTrustPortfolioK1::from(data);
    let share = crate::community::share(
        draft.setup.as_ref()?,
        entry.owner.owned(),
        entry.community,
        &entry.income,
    )?;
    if share == share.for_gains() {
        return None;
    }
    if data.minimum_tax_exclusion.is_some_and(|amount| amount != 0)
        || classified_other_in_a(data).is_none()
    {
        return Some("ISSUE_ESTATE_AMT_SOURCE_UNAVAILABLE");
    }
    if data
        .niit_adjustment
        .as_ref()
        .is_some_and(|source| source.line_7 != 0 || source.modified_agi != 0)
    {
        return Some("ISSUE_ESTATE_COMMUNITY_NIIT_SOURCE_UNAVAILABLE");
    }
    None
}

// Fixed source categories, not editable statement order, own cent ties. Prefix
// uses Share's existing ratio projection once through each cumulative amount;
// the counterpart therefore retains exactly original minus owner's cents.
fn capital_categories(
    total: i64,
    rate28: i64,
    unrecaptured: i64,
    share: crate::community::Share,
) -> [i64; 3] {
    let mut prefix = crate::kernel::ratio::Prefix::default();
    [rate28, unrecaptured, total - rate28 - unrecaptured]
        .map(|amount| prefix.next(amount, |through| share.gain(through)))
}

pub(crate) fn reconcile_community_source(
    source: &crate::FederalEstateTrustPortfolioK1,
    projected: &mut crate::FederalEstateTrustPortfolioK1,
    share: crate::community::Share,
) {
    let regular =
        capital_categories(source.long_gain, source.rate_28_gain, source.unrecaptured_gain, share);
    projected.rate_28_gain = regular[0];
    projected.unrecaptured_gain = regular[1];
    projected.long_gain = regular.iter().sum();
    let alternative = capital_categories(
        source.long_gain + source.amt_box12d,
        source.rate_28_gain + source.amt_box12f,
        source.unrecaptured_gain + source.amt_box12e,
        share,
    );
    projected.amt_box12d = alternative.iter().sum::<i64>() - projected.long_gain;
    projected.amt_box12e = alternative[1] - projected.unrecaptured_gain;
    projected.amt_box12f = alternative[0] - projected.rate_28_gain;
    projected.amt_box12c = share.gain(source.short_gain + source.amt_box12c) - projected.short_gain;

    let mut dividends = crate::kernel::ratio::Prefix::default();
    projected.qualified_dividends =
        dividends.next(source.qualified_dividends, |through| share.filer(through));
    let reit =
        dividends.next(source.section199a_dividends.amount(), |through| share.filer(through));
    if let crate::ReitDividends::Reported { amount, .. } = &mut projected.section199a_dividends {
        *amount = reit;
    }
    projected.ordinary_dividends = projected.qualified_dividends
        + reit
        + dividends.next(
            source.ordinary_dividends
                - source.qualified_dividends
                - source.section199a_dividends.amount(),
            |through| share.filer(through),
        );
    projected.amt_box12b =
        share.filer(source.qualified_dividends + source.amt_box12b) - projected.qualified_dividends;
    crate::estate_foreign::project(source, projected, share);

    if let (Some(original), Some(finals)) =
        (&source.final_character, &mut projected.final_character)
    {
        let parts = capital_categories(
            source.final_long_carry,
            original.capital.rate28,
            original.capital.unrecaptured,
            share,
        );
        finals.capital.rate28 = parts[0];
        finals.capital.unrecaptured = parts[1];
        projected.final_long_carry = parts.iter().sum();
        if original.capital.niit_difference
            == Some(source.final_short_carry + source.final_long_carry)
        {
            finals.capital.niit_difference =
                Some(projected.final_short_carry + projected.final_long_carry);
        } else if finals
            .capital
            .niit_difference
            .is_some_and(|amount| amount > projected.final_short_carry + projected.final_long_carry)
        {
            // A partial excluded-loss aggregate has no short/long attribution.
            // Absence preserves the existing optional NIIT benefit boundary.
            finals.capital.niit_difference = None;
        }
    }
    // Box totals and linked AMT differences are copies of these children,
    // never independently rounded additional source amounts.
    for (kind, total) in [
        (EstateActivityKind::Business, &mut projected.activities.box6),
        (EstateActivityKind::RentalRealEstate, &mut projected.activities.box7),
        (EstateActivityKind::OtherRental, &mut projected.activities.box8),
    ] {
        *total = projected
            .activities
            .activities
            .iter()
            .filter(|item| item.kind == kind)
            .map(|item| item.income)
            .sum();
    }
    if let Some(reconciliation) = &mut projected.amt_reconciliation {
        for activity in &mut projected.activities.activities {
            activity.amt_difference = reconciliation.adjustments.iter().filter(|item| {
                matches!(item.treatment, EstateAmtTreatment::Activity { activity: id, .. } if id == activity.id)
            }).map(|item| item.amount).sum();
        }
        if classified_other_in_a(source).is_some() {
            reconciliation.other_in_a = reconciliation
                .adjustments
                .iter()
                .filter(|item| included_in_a(&item.treatment))
                .map(|item| item.amount)
                .sum();
        }
    }
    if classified_other_in_a(source).is_some() {
        projected.amt_box12a = projected.amt_box12b
            + projected.amt_box12c
            + projected.amt_box12d
            + projected.amt_reconciliation.as_ref().map_or(0, |item| item.other_in_a);
    }

    for (statement, original) in projected.qbi.statements.iter_mut().zip(&source.qbi.statements) {
        for (part, original_part) in statement.allocations.iter_mut().zip(&original.allocations) {
            for (year, original_year) in
                part.passive_history.iter_mut().zip(&original_part.passive_history)
            {
                // qbi_projection already uses unique source names as its stable
                // prefix order. Use that same comparator before limit engines.
                let preceding = |remaining: bool| {
                    source
                        .qbi
                        .statements
                        .iter()
                        .filter(|other| other.name < original.name)
                        .flat_map(|other| &other.allocations)
                        .filter(|allocation| allocation.activity == original_part.activity)
                        .flat_map(|allocation| &allocation.passive_history)
                        .filter(|history| history.year == original_year.year)
                        .map(|history| {
                            if remaining { history.remaining_qualified } else { history.qualified }
                        })
                        .sum::<i64>()
                };
                let before = preceding(false);
                year.qualified =
                    share.filer(before + original_year.qualified) - share.filer(before);
                let before = preceding(true);
                year.remaining_qualified =
                    share.filer(before + original_year.remaining_qualified) - share.filer(before);
            }
            part.passive_history.retain(nonzero_qbi_history);
        }
        statement.passive_history.retain(nonzero_qbi_history);
        if !statement.allocations.is_empty() {
            statement.income = statement.allocations.iter().map(|part| part.income).sum();
        }
    }

    if let (Some(original), Some(investment)) =
        (&source.investment_income, &mut projected.investment_income)
    {
        if let Some(adjustment) = investment_adjustment(source) {
            let regular_net = projected.short_gain + projected.long_gain;
            let alternative_net = regular_net + projected.amt_box12c + projected.amt_box12d;
            let original_regular_net = source.short_gain + source.long_gain;
            let original_alternative_net =
                original_regular_net + source.amt_box12c + source.amt_box12d;
            // A signed net-loss floor is not a monetary component. If cent
            // projection crosses that floor, retain the existing benefit boundary.
            let conserved = |whole: i64, part: i64| {
                phase::floor(part, 0) + phase::floor(whole - part, 0) == phase::floor(whole, 0)
            };
            if conserved(original_regular_net, regular_net)
                && conserved(original_alternative_net, alternative_net)
            {
                let preferential = match original.character {
                    EstateInvestmentCharacter::Preferential { .. } => {
                        investment.character = EstateInvestmentCharacter::Preferential {
                            qualified: projected.qualified_dividends,
                            gains: phase::floor(regular_net, 0),
                            amt_qualified: phase::floor(
                                projected.qualified_dividends + projected.amt_box12b,
                                0,
                            ),
                            amt_gains: phase::floor(alternative_net, 0),
                        };
                        [
                            projected.qualified_dividends + phase::floor(regular_net, 0),
                            phase::floor(projected.qualified_dividends + projected.amt_box12b, 0)
                                + phase::floor(alternative_net, 0),
                        ]
                    }
                    _ => [0; 2],
                };
                let ordinary = projected.interest + projected.ordinary_dividends
                    - projected.qualified_dividends
                    + projected.royalty_income
                    - projected.box_9_royalty_depreciation
                    - projected.box_9_royalty_depletion
                    - projected.box_9_royalty_amortization;
                investment.regular = ordinary + share.filer(adjustment[0]) + preferential[0];
                investment.alternative = ordinary + share.filer(adjustment[1]) + preferential[1];
            } else {
                investment.character = EstateInvestmentCharacter::Unavailable;
            }
        } else {
            investment.character = EstateInvestmentCharacter::Unavailable;
        }
    }
}

fn nonzero_qbi_history(history: &crate::ScorpLossYear) -> bool {
    history.original != 0
        || history.qualified != 0
        || history.remaining != 0
        || history.remaining_qualified != 0
}

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(EstateActivities {
    amounts: [box6, box7, box8, activities],
    gains: [],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(EstateActivity {
    amounts: [income, depreciation, depletion, amortization, carried, amt_difference],
    gains: [],
    facts: [id, name, kind, participation],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(EstatePassiveCarry {
    amounts: [regular, alternative],
    gains: [],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(EstateFinalCharacter {
    amounts: [section67e_nii, additional, capital],
    gains: [],
    facts: [entity_terminated, section67e_in_h],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(EstateFinalCapital {
    amounts: [],
    gains: [
        alternative_short,
        alternative_long,
        rate28,
        unrecaptured,
        niit_difference,
        regular_investment,
        alternative_investment,
        regular_business,
        alternative_business
    ],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(EstateCapitalLossParts {
    amounts: [],
    gains: [short, long],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
impl crate::community::Divide for EstateInvestmentIncome {
    fn divided(&self, share: crate::community::Share) -> Self {
        match self.character {
            EstateInvestmentCharacter::Preferential {
                qualified,
                gains,
                amt_qualified,
                amt_gains,
            } => {
                let projected_qualified = share.filer(qualified);
                let projected_gains = share.gain(gains);
                let projected_amt_qualified = share.filer(amt_qualified);
                let projected_amt_gains = share.gain(amt_gains);
                Self {
                    regular: share.filer(self.regular - qualified - gains)
                        + projected_qualified
                        + projected_gains,
                    alternative: share.filer(self.alternative - amt_qualified - amt_gains)
                        + projected_amt_qualified
                        + projected_amt_gains,
                    character: EstateInvestmentCharacter::Preferential {
                        qualified: projected_qualified,
                        gains: projected_gains,
                        amt_qualified: projected_amt_qualified,
                        amt_gains: projected_amt_gains,
                    },
                }
            }
            _ => Self {
                regular: share.filer(self.regular),
                alternative: share.filer(self.alternative),
                character: self.character.clone(),
            },
        }
    }
}

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(EstateQbiStatements {
    amounts: [statements],
    gains: [],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(EstateQbiStatement {
    amounts: [income, wages, property, passive_history, allocations],
    gains: [],
    facts: [id, name, service, kind, activities],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(EstateQbiAllocation {
    amounts: [income, passive_history],
    gains: [],
    facts: [activity],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(EstateAmtReconciliation {
    amounts: [other_in_a, adjustments],
    gains: [],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(EstateAmtAdjustment {
    amounts: [amount],
    gains: [],
    facts: [id, code, treatment],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(EstateReportedCredit {
    amounts: [amount],
    gains: [],
    facts: [code, activities],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(EstateForeignStatement {
    amounts: [tax, treatment],
    gains: [],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(EstateInvestmentProperty {
    amounts: [property],
    gains: [],
    facts: [activities],
});

// Allocate amounts; legal character, dates and retained facts are unchanged.
crate::community::divide_enum!(EstateFinalDeduction {
    records: [
        InvestmentInterest { amounts: [amount], facts: [description] },
        Other { amounts: [], facts: [description] },
    ],
    units: [],
    tuples: [],
});

// Allocate only source amounts; the source's country, method and payment class remain facts.
crate::community::divide_enum!(EstateForeignTreatment {
    records: [PassivePortfolio { amounts: [allocations], facts: [basis] },],
    units: [QualifiedPassivePayeeStatement, RequiresAllocation],
    tuples: [],
});
crate::community::divide_record!(crate::EstateForeignPortfolioAllocation {
    amounts: [tax, regular, alternative],
    gains: [],
    facts: [country, withholding],
});
crate::community::divide_record!(crate::EstateForeignPortfolioAmounts {
    amounts: [interest, dividends, qualified, direct_expenses, interest_expense],
    gains: [],
    facts: [],
});

// The complete fiduciary reconciliation identifies whether another adjustment
// could alter the interest/dividend axis. An aggregate code A cannot do so.
pub(crate) fn foreign_portfolio_capacities(
    data: &crate::FederalEstateTrustPortfolioK1,
) -> Option<[[i64; 3]; 2]> {
    data.amt_reconciliation.as_ref()?;
    if classified_other_in_a(data)? != 0 {
        return None;
    }
    let regular = regular_foreign_portfolio_capacity(data)?;
    let amt_qualified = portfolio_qualified(data)?;
    Some([regular, [data.interest, regular[1].checked_add(data.amt_box12b)?, amt_qualified]])
}
fn portfolio_qualified(data: &crate::FederalEstateTrustPortfolioK1) -> Option<i64> {
    data.qualified_dividends.checked_add(data.amt_box12b)
}

pub(crate) fn regular_foreign_portfolio_capacity(
    data: &crate::FederalEstateTrustPortfolioK1,
) -> Option<[i64; 3]> {
    Some([
        data.interest,
        data.ordinary_dividends.checked_sub(data.section199a_dividends.amount())?,
        data.qualified_dividends,
    ])
}

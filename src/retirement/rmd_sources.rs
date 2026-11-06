use crate::common::Owner;
use crate::dates::{source_valid_date as valid_date, valid_birth};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Missed required minimum distribution
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MissedDistribution {
    /// Plan or IRA group
    /// A short name, such as "Traditional IRAs" or "Employer 401(k)". Do not enter an account number. Own traditional IRAs may be grouped; inherited IRAs only with the same deceased owner; each employer plan is separate.
    #[libertas_size(min = 1)]
    pub label: String,
    /// Required minimum for 2026
    pub required: crate::RmdMinimum,
    /// Taken by the deadline
    /// Include qualified charitable distributions. Leave out payments made after the deadline and rollovers.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub timely: i64,
    /// Correction
    pub correction: RmdCorrection,
    /// Waiver requested for reasonable cause
    /// Leave empty for no waiver request. The IRS decides; the estimate removes the amount pending its review. IRS approval is not automatic: retain and file the explanation with Form 5329.
    /// [NullText]
    /// Do not request a reasonable-cause waiver
    /// [SetText]
    /// I request a reasonable-cause waiver for
    #[libertas_default($null)]
    pub waiver: Option<RmdWaiver>,
}
impl MissedDistribution {
    pub(crate) fn record(&self, owner: Owner) -> RequiredDistribution {
        RequiredDistribution {
            owner,
            label: self.label.clone(),
            required: self.required.clone(),
            timely: self.timely,
            correction: self.correction.clone(),
            waiver: self.waiver.clone(),
        }
    }
    pub(crate) fn from_record(v: &RequiredDistribution) -> Self {
        Self {
            label: v.label.clone(),
            required: v.required.clone(),
            timely: v.timely,
            correction: v.correction.clone(),
            waiver: v.waiver.clone(),
        }
    }
}

/// Required-distribution account group
/// One entry per plan or legally aggregatable group. Never offset one group's shortfall with another group's excess. Choose an administrator's required amount or calculate from own IRA balances and beneficiary facts.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RequiredDistribution {
    /// Person required to withdraw
    pub owner: Owner,
    /// Plan or inherited-owner label
    /// Use a descriptive synthetic label, not an account number or tax ID.
    pub label: String,
    /// Required-minimum source
    pub required: RmdMinimum,
    /// Amount satisfying these requirements by their deadlines
    /// Include eligible qualified charitable distributions. Exclude corrective payments after the deadline, rollovers and unrelated accounts. If two deadlines fall in 2026, do not count a payment made after the first deadline against its missed requirement or count a payment twice.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub timely: i64,
    /// Later correction and additional-tax return
    pub correction: RmdCorrection,
    /// Reasonable-cause waiver request, if any
    /// IRS approval is not automatic. The requested amount is removed from the reported shortfall pending IRS review; retain and file the explanation with Form 5329.
    pub waiver: Option<RmdWaiver>,
}

/// Correction records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RmdCorrection {
    /// Full correction or a dated filing commitment is missing
    Uncorrected,
    /// Full correction received; return filed or planned
    Corrected {
        /// Date the full shortfall was withdrawn from the responsible plan or eligible account group
        /// Partial correction does not qualify this group for the reduced rate. Do not split an uncorrected plan to obtain 10% treatment on only its corrected portion.
        #[libertas_date_only]
        received: u32,
        /// Return reflecting this additional tax
        filed: RmdFiling,
        /// Deficiency-notice mailing date, if any
        /// [NullText]
        /// No deficiency notice has been mailed for this tax
        /// [SetText]
        /// The deficiency notice was mailed on
        #[libertas_date_only]
        notice: Option<u32>,
        /// Assessment date, if any
        /// [NullText]
        /// This additional tax has not been assessed
        /// [SetText]
        /// This additional tax was assessed on
        #[libertas_date_only]
        assessed: Option<u32>,
    },
}

/// Additional-tax return submission
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RmdFiling {
    /// Already submitted
    Submitted {
        /// Actual submission date
        #[libertas_date_only]
        date: u32,
    },
    /// Prepare an estimate for a planned submission
    Planned {
        /// Intended submission date
        /// The reduced-rate estimate depends on actually submitting the return reflecting this tax by the correction-window deadline. This prototype does not file it. Update the calculation if filing is delayed or an IRS notice or assessment arrives first.
        #[libertas_date_only]
        date: u32,
    },
}
impl RmdFiling {
    pub(crate) fn date(&self) -> u32 {
        match self {
            Self::Submitted { date } | Self::Planned { date } => *date,
        }
    }
}

/// Requested reasonable-cause relief
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RmdWaiver {
    /// Shortfall amount for which relief is requested
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// Explanation of reasonable error and corrective steps
    pub explanation: String,
}
impl RequiredDistribution {
    pub(crate) fn valid(&self) -> bool {
        self.required.valid()
            && (0..=100000000000).contains(&self.timely)
            && self.waiver.as_ref().is_none_or(|v| (1..=100000000000).contains(&v.amount))
            && match &self.correction {
                RmdCorrection::Uncorrected => true,
                RmdCorrection::Corrected { received, filed, notice, assessed } => {
                    [Some(*received), Some(filed.date()), *notice, *assessed]
                        .into_iter()
                        .flatten()
                        .all(|v| valid_date(v) && v >= 20260101)
                }
            }
    }
}
/// Source of the required minimum
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RmdMinimum {
    /// Calculate an original beneficiary’s inherited IRA minimum
    Inherited { data: InheritedRmd },
    /// Administrator's required amount
    Administrator {
        /// Minimum required by deadlines in 2026
        /// Retain the administrator's account-balance and beneficiary calculation. Include first payments deferred from 2025 to April 2026; exclude payments lawfully deferred until 2027.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
    /// Calculate from own traditional IRA account balances
    OwnIras {
        /// Distribution year and first-year deadline choice
        period: RmdPeriod,
        /// Separately valued accounts with their beneficiary designations
        /// Compute each IRA separately before adding the requirements. A withdrawal from one own traditional IRA may satisfy this group's total; never combine inherited or Roth IRAs here. Own traditional, SEP and SIMPLE IRAs, each account once: no inherited account, first-year spouse ownership election after death, annuity purchase or valuation election or outstanding transfer not entered. Working past age 73 does not defer an IRA's requirement. The December 31 custodian statements (Form 5498 box 5) show the balances.
        #[libertas_size(min = 1)]
        accounts: Vec<RmdIra>,
    },
}

/// IRA distribution year
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum RmdPeriod {
    /// 2026 requirement
    /// Due by December 31, 2026. In the year you turn 73 the first requirement is due April 1, 2027 instead: the calculated amount is shown, no shortfall tax is figured on this return, and the next requirement is also due in 2027, by December 31. Waiting can concentrate taxable income in 2027.
    Current,
    /// First 2025 requirement deferred to April 1, 2026
    /// Only for an owner who turned 73 in 2025. Use December 31, 2024 balances and 2025 beneficiary facts. Include all payments satisfying this first requirement by April 1, including any made in 2025, in the timely-payment field. Report the separate 2026 requirement too; never count a payment twice.
    FirstPriorYear,
}

/// IRA balance and beneficiary
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RmdIra {
    /// December 31 balance before the distribution year
    /// Use December 31, 2025 for 2026; December 31, 2024 for the deferred first 2025 requirement. Do not add later contributions designated for that earlier year or subtract later withdrawals.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub balance: i64,
    /// Outstanding incoming rollover absent from all year-end accounts
    /// Use its value on receipt and add it only to the receiving IRA. Enter zero when none. Do not enter a rollover already included in the year-end balance or another account here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub rollover: i64,
    /// Spouse qualifying as sole beneficiary for the distribution year
    /// Leave absent if unmarried or the spouse does not qualify as sole beneficiary. Age alone does not qualify another beneficiary for the joint table.
    /// [NullText]
    /// No spouse qualifies as this account's sole beneficiary
    /// [SetText]
    /// The spouse qualifying as sole beneficiary is
    pub spouse: Option<RmdSpouse>,
}

/// Spouse beneficiary facts for this distribution year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RmdSpouse {
    /// Spouse's date of birth
    /// Use that year's beneficiary, which can differ from the spouse on the current return following divorce or death.
    #[libertas_date_only]
    pub birth: u32,
    /// Marriage and sole-beneficiary requirements confirmed
    /// Check the account’s beneficiary designation and marriage records for that distribution year.
    /// Marital status is determined January 1. Confirm sole-beneficiary status for the required year; a spouse's death alone does not disqualify that year, but divorce followed by a beneficiary change during the year does. For a deferred first withdrawal use the distribution year's designation, not the later payment year's. No uses the Uniform Lifetime table (a larger minimum).
    pub confirmed: bool,
}
impl RmdMinimum {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Administrator { amount } => (0..=100000000000).contains(amount),
            Self::Inherited { data } => data.valid(),
            Self::OwnIras { accounts, .. } => {
                !accounts.is_empty()
                    && accounts.len() <= 100
                    && accounts.iter().all(|a| {
                        [a.balance, a.rollover].iter().all(|v| (0..=100000000000).contains(v))
                            && a.spouse.as_ref().is_none_or(|s| valid_birth(s.birth))
                    })
            }
        }
    }
}
/// Inherited IRA minimum for an original designated beneficiary
/// This is the beneficiary's post-death requirement, not the original owner's unpaid year-of-death requirement. Use one account or a legally aggregatable group from the same deceased owner with the same payout rules.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InheritedRmd {
    /// IRA kind
    pub kind: InheritedIraKind,
    /// Original owner's birth date
    #[libertas_date_only]
    pub owner_birth: u32,
    /// Original owner's death date
    /// Owner must have died before 2026. Earlier life-expectancy inheritances use the 2022 table reset; year-of-death obligations and successor beneficiaries need their administrator's calculation.
    #[libertas_date_only]
    pub died: u32,
    /// December 31, 2025 inherited balance
    /// Use this beneficiary's allocated account, not the deceased owner's entire unsplit IRA.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub balance: i64,
    /// Outstanding incoming transfer absent from the year-end balance
    /// Use the receiving account's source adjustment, counted once. Nonspouse inherited IRAs cannot use a 60-day rollover.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub incoming: i64,
    /// Remaining account value at December 31, 2026, when life expectancy is exhausted
    /// Required only when the remaining life-expectancy factor is one year or less. Include all remaining assets and earnings; zero means the account was fully emptied. This is a current year-end value, not the preceding year's balance. Earlier overdue final deadlines need administrator review.
    /// [NullText]
    /// The remaining life-expectancy factor exceeds one year
    /// [SetText]
    /// The remaining account value on December 31, 2026 was
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub closing: Option<i64>,
    /// Additional eligibility at the original owner's death
    /// The App separately evaluates whether this filer is not more than ten years younger using exact birth dates. Select disability or chronic illness when applicable even if also the owner's child; that status can preserve life-expectancy treatment after age 21.
    pub eligibility: InheritedEligibility,
    /// Elected the fixed payout deadline instead of life expectancy
    /// Available when the original owner died before their required beginning date, including Roth IRAs: five years for pre-2020 deaths, ten for later deaths. Confirm the custodian permits the election and it was timely. Leave false for normal statutory treatment, which still uses ten years for a non-eligible post-2019 beneficiary.
    pub deadline_election: bool,
    /// Original direct beneficiary, source facts and applicable election confirmed
    /// The filer was individually designated and remains alive through 2026. Use the specific sole-spouse branch for a spouse; other branches describe nonspouse beneficiaries. The separate account was timely established and this filer alone controls its beneficiary life expectancy. For pre-2020 deaths, confirm life-expectancy treatment was timely established under the IRA terms; this choice cannot revive an expired five-year election. No successor or see-through trust, annuity, special deadline relief or unentered adjustment. Any original-owner shortfall is handled separately with administrator allocation. Confirm medical/relationship facts at death and retain required evidence; current disability alone does not establish status at death. The beneficiary designation and custodian records show it. No stops the estimate: the required minimum and any excise tax on a shortfall are not calculated for other situations.
    pub confirmed: bool,
}

/// Inherited IRA kind
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum InheritedIraKind {
    /// Traditional, SEP or SIMPLE IRA
    Traditional,
    /// Roth IRA
    /// Treated as an owner who died before the required beginning date, regardless of the owner's age.
    Roth,
}

/// Eligibility facts at death
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum InheritedEligibility {
    /// Sole surviving spouse, inherited Single Life treatment
    /// The account remains inherited and this spouse alone is its designated beneficiary. No election or deemed election to treat it as their own, no Uniform Lifetime spouse election, and no other beneficiary or trust. The App recalculates Single Life expectancy annually and applies the delayed start when the owner died before the required beginning date. An unpaid minimum requires review of deemed ownership and the personal IRA pool.
    SoleSpouse,
    /// No additional medical or child status
    Ordinary,
    /// Original owner's child
    /// The App checks whether the child had reached 21 when the owner died. A grandchild or unrelated minor does not qualify under this branch.
    OwnerChild,
    /// Disabled at the owner's death
    /// Age 18 or older: unable to engage in substantial gainful activity because of a medically determinable impairment expected to result in death or be long continued and indefinite. Under 18: marked and severe functional limitations of that duration. A qualifying Social Security disability determination is also accepted. Retain evidence of status at death.
    Disabled,
    /// Chronically ill at the owner's death
    /// Retain the licensed practitioner's qualifying certification: substantial assistance with at least two activities of daily living for an indefinite, reasonably lengthy period, or qualifying substantial supervision for severe cognitive impairment.
    ChronicallyIll,
    /// Sole surviving spouse using the inherited-account Uniform Lifetime election
    /// The spouse remains beneficiary, without an election or deemed election to own the IRA. The original owner died on or after their required beginning date in 2023 or later, and the surviving spouse is at least 73 in 2026. Retain the IRA custodian's confirmation of the Table III election, which is distinct from treating the account as the spouse's own IRA. An unpaid minimum requires deemed-ownership review.
    SoleSpouseUniformLifetime {
        /// Custodian confirms the surviving-spouse Table III election for this inherited IRA
        /// Ask the IRA custodian. No uses the inherited Single Life table instead.
        confirmed: bool,
    },
}

impl InheritedRmd {
    pub(crate) fn valid(&self) -> bool {
        valid_birth(self.owner_birth)
            && valid_birth(self.died)
            && self.owner_birth <= self.died
            && self.closing.is_none_or(|v| (0..=100000000000).contains(&v))
            && [self.balance, self.incoming].iter().all(|v| (0..=100000000000).contains(v))
    }
}

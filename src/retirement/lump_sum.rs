// Coverage: Form 4972 — Supported (unit B2, 2026-10-09; the former Missing audit of 2026-09-24 and
//   its block in the former nua_distribution.rs are replaced).
//   Lines: Part I is the box 7a code A treatment on the 1099-R page (7A normal distribution of a
//   participant born before 1936-01-02, 4A beneficiary of such a participant): the birth date is
//   tested day-exact (attained age 50 before 1986-01-01, federal_model attained_on) against People for
//   7A and the entered participant birth date for 4A; line 1 (entire balance, one tax year) and the
//   qualifying-event and earlier-payment facts are tested once, here (lump_sum_conditions: event on or
//   before the first payment, no earlier-year payment after the event); lines 2, 4(c), 5a, 5b and the "don't qualify" list are the one
//   `confirmed` answer; a No, a failed condition or a participant born on or after 1936-01-02 leaves
//   the election out with a notice and the record is taxed with the other income
//   (unknowns.rs lump_sums). The merged NUA kind is gone (unit B5, 2026-10-09): box 2a as the payer
//   prints it already leaves the deferred NUA out, so a 1099-R with box 6 is a payer-amount row and
//   Form 4972's include_nua is the only place the include-now election is asked. NuaEvent and
//   lump_sum_conditions below are the single qualifying-event test of Form 4972 (tests/retirement/lump_sum.rs
//   nua_age_event_reads_the_people_birth_date_day_exact, nua_reads_the_form_4972_election_facts_once).
//   Part II lines 6-7 (20% on box 3, with the NUA Worksheet and the Death
//   Benefit Worksheet capital-gain share); Part III lines 8-30 (10-year option on the 1986 rate
//   schedule in year_constants.rs, minimum distribution allowance, annuity contract share lines
//   11 and 20-22 and 28, estate tax line 18, the multiple-recipient Steps 2-5 with box 9a and box 8b
//   converted once to exact hundredths of a percent). The person chooses the method: Part III, Part II
//   or both. The amount taxed here is outside Form 1040 lines 5a and 5b (so outside AGI); Part II
//   alone leaves the ordinary part on 5a and 5b. Line 30 reaches Form 1040 line 16 (box 2) and the
//   regular tax of every credit limit, not Form 6251 line 10 and not Form 1116 line 20.
//   Unit B9-4 (2026-10-10): line 9 and the Death Benefit Worksheet (a participant who died before
//   1996-08-21: the recipient's entered share of the $5,000 exclusion, less the capital-gain share E
//   of Part II; LUMP_SUM_DEATH_BENEFIT_AFTER_1996) and line 18 with box 9a (the recipient's share of
//   the estate tax divided by the box 9a percentage, as Step 2 restates lines 8 and 11).
//   Not calculated: a trust-only Step 1 (Form 1041); an alternate
//   payee under a QDRO (taxed with the other income unless the filer's own birth date qualifies);
//   the Form 1116 separate lump-sum limitation (no credit claimed); the Form 1040 line 16 box
//   check mark is printed only (printing deferred). Two or more records of one participant treated
//   differently, or combined with a box 9a or 8b share, are refused by name
//   (ISSUE_LUMP_SUM_TREATED_DIFFERENTLY, ISSUE_LUMP_SUM_SHARED_AND_COMBINED).
//   Stored shape: RetirementTreatment::LumpSum { Form4972Election } (the name differs from the map's
//   LumpSumElection, which is the Social Security lump-sum type in social.rs).
//   Evidence: lump_sum.rs, retirement.rs, federal_rules.rs, unknowns.rs;
//   tests: tests/retirement/lump_sum.rs (Pub 575 (2024) Examples 1 and 2 at $24,270 and $28,070, the
//   allowance with an annuity share, Part II only, multiple recipients with a fractional 8b share,
//   NUA, estate tax, the 1936-01-01 and 1936-01-02 boundaries, left-out and consistency cases,
//   stored shapes, the AMT comparison), tests/retirement/interview.rs lump_sum_* (page mapping, rules on
//   both sides, round trip, import), lump_sum.rs unit tests (every schedule boundary and the
//   allowance); sources 2026 draft f4972 (created 6/9/26), f1099r and i1099r (2026), 2026 draft
//   f1040, f6251, i1116, Pub 575 (2024), TRA 1986 section 1122(h)(3), (5), (6), P.L. 104-188
//   section 1401(c)(2), former 26 U.S.C. 402(d), 26 U.S.C. 402(e)(4).
use crate::kernel::lines::push_line;
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::year_constants::{
    LUMP_SUM_MINIMUM_ALLOWANCE_CAP_DOLLARS, LUMP_SUM_MINIMUM_ALLOWANCE_PHASEOUT_START_DOLLARS,
    LUMP_SUM_RATE_SCHEDULE_CENTS,
};
use crate::{
    FederalDraft, FederalIncome, FederalIncomeEntry, FederalResult, FormLine, Owner,
    RetirementKind, RetirementTaxable, RetirementTreatment, TaxForm,
};
use alloc::vec::Vec;
use libertas_macros::*;

const SOURCE: &str = "https://www.irs.gov/pub/irs-dft/f4972--dft.pdf";
// The former death benefit exclusion applies to a participant who died before this date (Form
// 4972 line 9).
const DEATH_BENEFIT_ENDED: u32 = 19960821;
const MAX: i64 = 100_000_000_000;

/// Tax method
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum LumpSumMethod {
    /// 10-year tax option on the whole taxable amount
    /// Form 4972 Part III.
    TenYear,
    /// 20% rate on the box 3 capital gain; the rest with your other income
    /// Form 4972 Part II.
    CapitalGain,
    /// 20% rate on the box 3 capital gain and the 10-year tax option on the rest
    /// Form 4972 Parts II and III.
    Both,
}

impl LumpSumMethod {
    fn capital_gain(self) -> bool {
        !matches!(self, Self::TenYear)
    }
    fn ten_year(self) -> bool {
        !matches!(self, Self::CapitalGain)
    }
}

/// Qualifying event for your own lump sum
/// The distribution must follow this event. Choose the event after your last earlier payment from these plans; reaching age 59½ counts when no payment from these plans followed it in an earlier year.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum LumpSumEvent {
    /// You reached age 59½
    /// Your People page has your date of birth.
    Age,
    /// You separated from service
    Separation {
        /// You separated on
        #[libertas_date_only]
        date: u32,
        /// You were an employee, not self-employed
        /// A Form W-2 from the employer shows it. No taxes this distribution with your other income, with a notice.
        employee: bool,
    },
    /// You are self-employed and became totally and permanently disabled
    Disability {
        /// Total and permanent disability began on
        #[libertas_date_only]
        date: u32,
        /// Self-employed and a qualifying total and permanent disability confirmed
        /// The medical determination on record shows it. No taxes this distribution with your other income, with a notice.
        eligible: bool,
    },
}

impl LumpSumEvent {
    // The NUA module's event for the same facts; the participant's own birth date
    // comes from People when the conditions are tested.
    pub(crate) fn nua_event(&self) -> crate::NuaEvent {
        match *self {
            Self::Age => crate::NuaEvent::Age,
            Self::Separation { date, employee } => crate::NuaEvent::Separation { date, employee },
            Self::Disability { date, eligible } => crate::NuaEvent::Disability { date, eligible },
        }
    }
    fn valid(&self) -> bool {
        match *self {
            Self::Age => true,
            Self::Separation { date, .. } | Self::Disability { date, .. } => {
                crate::federal_rules::valid_birth(date)
            }
        }
    }
}

/// Form 4972 election
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Form4972Election {
    /// Plan participant
    pub participant: Form4972Participant,
    /// Tax method
    pub method: LumpSumMethod,
    /// Form 1099-R box 3 capital gain
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub capital_gain: i64,
    /// Form 1099-R box 6 net unrealized appreciation
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub nua: i64,
    /// Include the box 6 net unrealized appreciation in income now
    pub include_nua: bool,
    /// Form 1099-R box 8a current value of an annuity contract
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub annuity_value: i64,
    /// Form 1099-R box 8b percentage of the annuity contract, in hundredths of a percent
    // Form 4972: this optional partial-distribution percentage is strictly below 100%, in hundredths.
    #[libertas_number(min = 1, max = 9999)]
    pub annuity_percent: Option<u32>,
    /// Form 1099-R box 9a percentage of the total distribution, in hundredths of a percent
    // Form 4972: this optional partial-distribution percentage is strictly below 100%, in hundredths.
    #[libertas_number(min = 1, max = 9999)]
    pub percent: Option<u32>,
    /// Date of the first payment in this lump sum
    #[libertas_date_only]
    pub first_payment: u32,
    /// Date of the latest earlier payment from these plans
    #[libertas_date_only]
    pub last_prior_payment: Option<u32>,
    /// All of the employer's plans of the same kind were fully distributed within 2026
    pub entire_balance: bool,
    /// Form 4972 conditions met
    /// No taxes this distribution with your other income, with a notice.
    pub confirmed: bool,
}

/// Plan participant
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum Form4972Participant {
    /// You
    Own {
        /// Qualifying event
        event: LumpSumEvent,
    },
    /// A participant who died
    Decedent {
        /// Participant's date of birth
        #[libertas_date_only]
        born: u32,
        /// Date of death
        #[libertas_date_only]
        died: u32,
        /// Your share of federal estate tax on this distribution
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        estate_tax: i64,
        /// Your share of the death benefit exclusion
        /// Form 4972 line 9 and the Death Benefit Worksheet line D; only for a participant who died before August 21, 1996.
        #[libertas_money("USD", 2)]
        // Form 4972 death-benefit exclusion: at most $5,000 for a qualifying pre-August 21, 1996 death.
        #[libertas_number(min = 0, max = $DEATH_BENEFIT_EXCLUSION_CENTS)]
        death_benefit: i64,
    },
}

impl Form4972Participant {
    fn estate_tax(&self) -> i64 {
        match *self {
            Self::Own { .. } => 0,
            Self::Decedent { estate_tax, .. } => estate_tax,
        }
    }
    fn death_benefit(&self) -> i64 {
        match *self {
            Self::Own { .. } => 0,
            Self::Decedent { death_benefit, .. } => death_benefit,
        }
    }
}

impl Form4972Election {
    // The participant's date of birth: the filer's own for Own, the entered one for a decedent.
    pub(crate) fn birth(&self, own_birth: Option<u32>) -> Option<u32> {
        match self.participant {
            Form4972Participant::Own { .. } => own_birth,
            Form4972Participant::Decedent { born, .. } => Some(born),
        }
    }
    // TRA 1986 section 1122(h)(3)(C), (h)(5): the participant attained age 50 before 1986-01-01,
    // that is, was born before 1936-01-02 (day-exact: a person attains an age on the day before
    // the birthday).
    pub(crate) fn born_early_enough(birth: u32) -> bool {
        crate::federal_model::attained_on(birth, 50, true, 19851231)
    }
    // Part I: the entire balance within one tax year after a qualifying event (the NUA module's
    // test), and the filer's confirmation of the remaining conditions.
    pub(crate) fn qualifies(&self, own_birth: Option<u32>) -> bool {
        let (event, first_payment, last_prior_payment, entire_balance) = self.nua_facts();
        self.confirmed
            && lump_sum_conditions(
                &event,
                own_birth,
                first_payment,
                last_prior_payment,
                entire_balance,
            )
    }
    // The lump-sum facts this election states once, in the NUA module's terms: the qualifying
    // event, the first payment, the latest earlier payment and the entire-balance answer. A NUA
    // qualification that reads the election takes them from here.
    pub(crate) fn nua_facts(&self) -> (crate::NuaEvent, u32, Option<u32>, bool) {
        let event = match &self.participant {
            Form4972Participant::Own { event } => event.nua_event(),
            Form4972Participant::Decedent { died, .. } => crate::NuaEvent::Death { date: *died },
        };
        (event, self.first_payment, self.last_prior_payment, self.entire_balance)
    }
    // The fallback treatment when the election is left out: what "With your other income"
    // stores on the page.
    pub(crate) fn fallback(&self) -> RetirementTreatment {
        match self.participant {
            Form4972Participant::Own { .. } => RetirementTreatment::Ordinary,
            Form4972Participant::Decedent { died, .. } => RetirementTreatment::Exception {
                distributed: 20261231,
                facts: crate::RetirementException::Death { died },
                second: None,
                early_simple: false,
            },
        }
    }
    pub(crate) fn valid(&self, taxable: i64) -> bool {
        let percent = |p: Option<u32>| p.is_none_or(|p| (1..=9999).contains(&p));
        [self.capital_gain, self.nua, self.annuity_value].iter().all(|v| (0..=MAX).contains(v))
            && self.capital_gain <= taxable
            && (!self.include_nua || self.nua > 0)
            && (self.annuity_percent.is_none() || self.annuity_value > 0)
            && (!self.method.capital_gain() || self.capital_gain > 0)
            && percent(self.annuity_percent)
            && percent(self.percent)
            && crate::federal_rules::valid_date(self.first_payment)
            && (20260101..=20261231).contains(&self.first_payment)
            && self
                .last_prior_payment
                .is_none_or(|date| crate::federal_rules::valid_birth(date) && date < 20260101)
            && match &self.participant {
                Form4972Participant::Own { event } => event.valid(),
                Form4972Participant::Decedent { born, died, estate_tax, death_benefit } => {
                    crate::federal_rules::valid_birth(*born)
                        && crate::federal_rules::valid_birth(*died)
                        && born < died
                        && *died <= 20261231
                        && (0..=MAX).contains(estate_tax)
                        && (0..=crate::year_constants::DEATH_BENEFIT_EXCLUSION_CENTS)
                            .contains(death_benefit)
                        && (*death_benefit == 0 || *died < DEATH_BENEFIT_ENDED)
                }
            }
    }
    // The NUA Worksheet line E: the capital-gain share of box 6, when Part II is completed and the
    // net unrealized appreciation is included.
    fn nua_capital_share(&self, taxable: i64) -> i64 {
        if self.method.capital_gain() && self.include_nua && taxable > 0 {
            fraction(self.nua, self.capital_gain, taxable)
        } else {
            0
        }
    }
    // The ordinary part Part II alone leaves on Form 1040 lines 5a and 5b (How To Report item 2);
    // Part III takes the whole of it otherwise, so nothing stays on lines 5a and 5b.
    pub(crate) fn ordinary_part(&self, taxable: i64) -> i64 {
        if self.method.ten_year() {
            return 0;
        }
        let nua = if self.include_nua { self.nua - self.nua_capital_share(taxable) } else { 0 };
        taxable - self.capital_gain + nua
    }
}

// The 1099-R page onto the stored election and back. The boxes 3, 6, 8a, 8b and 9a come from the
// row's Other boxes; the two-decimal percents become exact hundredths here, once.
impl Form4972Election {
    pub(crate) fn from_own(tax: &crate::LumpSumTax, more: &crate::RMore) -> Option<Self> {
        let crate::LumpSumTax::Form4972 {
            method,
            include_nua,
            event,
            first_payment,
            last_prior_payment,
            entire_balance,
            confirmed,
        } = tax
        else {
            return None;
        };
        Some(Self {
            participant: Form4972Participant::Own { event: event.clone() },
            method: *method,
            include_nua: *include_nua,
            first_payment: *first_payment,
            last_prior_payment: *last_prior_payment,
            entire_balance: *entire_balance,
            confirmed: *confirmed,
            ..Self::with_boxes(more)
        })
    }
    pub(crate) fn from_decedent(
        died: u32,
        tax: &crate::InheritedLumpSumTax,
        more: &crate::RMore,
    ) -> Option<Self> {
        let crate::InheritedLumpSumTax::Form4972 {
            born,
            estate_tax,
            death_benefit,
            method,
            include_nua,
            first_payment,
            last_prior_payment,
            entire_balance,
            confirmed,
        } = tax
        else {
            return None;
        };
        Some(Self {
            participant: Form4972Participant::Decedent {
                born: *born,
                died,
                estate_tax: *estate_tax,
                death_benefit: *death_benefit,
            },
            method: *method,
            include_nua: *include_nua,
            first_payment: *first_payment,
            last_prior_payment: *last_prior_payment,
            entire_balance: *entire_balance,
            confirmed: *confirmed,
            ..Self::with_boxes(more)
        })
    }
    // The amounts and percents of the Other boxes; the caller sets the rest.
    fn with_boxes(more: &crate::RMore) -> Self {
        Self {
            participant: Form4972Participant::Own { event: LumpSumEvent::Age },
            method: LumpSumMethod::TenYear,
            capital_gain: more.box_3,
            nua: more.box_6,
            include_nua: false,
            annuity_value: more.box_8a,
            annuity_percent: hundredths(more.box_8b_percent),
            percent: hundredths(more.box_9a_percent),
            first_payment: 0,
            last_prior_payment: None,
            entire_balance: false,
            confirmed: false,
        }
    }
    pub(crate) fn own_page(&self) -> Option<crate::LumpSumTax> {
        let Form4972Participant::Own { event } = &self.participant else {
            return None;
        };
        Some(crate::LumpSumTax::Form4972 {
            method: self.method,
            include_nua: self.include_nua,
            event: event.clone(),
            first_payment: self.first_payment,
            last_prior_payment: self.last_prior_payment,
            entire_balance: self.entire_balance,
            confirmed: self.confirmed,
        })
    }
    pub(crate) fn decedent_page(&self) -> Option<(u32, crate::InheritedLumpSumTax)> {
        let Form4972Participant::Decedent { born, died, estate_tax, death_benefit } =
            self.participant
        else {
            return None;
        };
        Some((
            died,
            crate::InheritedLumpSumTax::Form4972 {
                born,
                estate_tax,
                death_benefit,
                method: self.method,
                include_nua: self.include_nua,
                first_payment: self.first_payment,
                last_prior_payment: self.last_prior_payment,
                entire_balance: self.entire_balance,
                confirmed: self.confirmed,
            },
        ))
    }
    // The Other boxes an elected record reopens with.
    pub(crate) fn boxes(&self, more: &mut crate::RMore) {
        more.box_3 = self.capital_gain;
        more.box_6 = self.nua;
        more.box_8a = self.annuity_value;
        more.box_8b_percent = percent_of(self.annuity_percent);
        more.box_9a_percent = percent_of(self.percent);
    }
}

// The server twins of the R1099 rules about the Other boxes and the Form 4972 answers (the keys are
// the client rules' keys), first failure first.
pub(crate) fn page_issue(v: &crate::R1099) -> Option<crate::validation::ValidationIssue> {
    use crate::{DistributionCode as C, InheritedLumpSumTax as I, LumpSumTax as L};
    let more = &v.more;
    if more.box_3 > v.taxable {
        return Some(
            crate::validation::ValidationIssue::new("R_BOX3_ABOVE_TAXABLE")
                .at_field("box_3")
                .in_field("more"),
        );
    }
    if more.box_8b_percent.is_some() && more.box_8a <= 0 {
        return Some(
            crate::validation::ValidationIssue::new("R_BOX8B_WITHOUT_BOX8A")
                .at_field("box_8b_percent")
                .in_field("more"),
        );
    }
    // (method, include_nua, first payment, last earlier payment, date of birth and death)
    let (method, include_nua, first, last, born_died) = match &v.code {
        C::NormalLumpSum {
            tax: L::Form4972 { method, include_nua, first_payment, last_prior_payment, .. },
        } => (*method, *include_nua, *first_payment, *last_prior_payment, None),
        C::DeathLumpSum {
            died,
            tax: I::Form4972 { born, method, include_nua, first_payment, last_prior_payment, .. },
        } => (*method, *include_nua, *first_payment, *last_prior_payment, Some((*born, *died))),
        _ => return None,
    };
    if method.capital_gain() && more.box_3 <= 0 {
        return Some(
            crate::validation::ValidationIssue::new("LUMP_SUM_CAPITAL_GAIN_WITHOUT_BOX_3")
                .at_field("method")
                .in_field("tax")
                .in_field("code"),
        );
    }
    if include_nua && more.box_6 <= 0 {
        return Some(
            crate::validation::ValidationIssue::new("LUMP_SUM_NUA_WITHOUT_BOX_6")
                .at_field("include_nua")
                .in_field("tax")
                .in_field("code"),
        );
    }
    if !(matches!(more.taxable_method, crate::TaxableMethod::BoxTwoA)
        && more.box_10 == 0
        && matches!(more.disability, crate::DisabilityIncome::None)
        && matches!(v.disaster, crate::DisasterDistribution::No)
        && v.public_safety.is_empty())
    {
        return Some(
            crate::validation::ValidationIssue::new("LUMP_SUM_OTHER_TREATMENTS")
                .at_field("taxable_method")
                .in_field("more"),
        );
    }
    if !(20260101..=20261231).contains(&first) {
        return Some(
            crate::validation::ValidationIssue::new("LUMP_SUM_FIRST_PAYMENT_NOT_IN_2026")
                .at_field("first_payment")
                .in_field("tax")
                .in_field("code"),
        );
    }
    if last.is_some_and(|date| date >= 20260101) {
        return Some(
            crate::validation::ValidationIssue::new("LUMP_SUM_PRIOR_PAYMENT_NOT_EARLIER")
                .at_field("last_prior_payment")
                .in_field("tax")
                .in_field("code"),
        );
    }
    if born_died.is_some_and(|(born, died)| born >= died) {
        return Some(
            crate::validation::ValidationIssue::new("LUMP_SUM_BORN_AFTER_DEATH")
                .at_field("born")
                .in_field("tax")
                .in_field("code"),
        );
    }
    None
}

// The 1986 single-individual tax on one tenth of `x`, times ten: for a bracket with base b, rate r
// and lower bound L, 10 * tax(x / 10) = 10 * b + r * (x - 10 * L). Exact; no intermediate rounding.
pub(crate) fn ten_times_tax(x: i64) -> i64 {
    if x <= 0 {
        return 0;
    }
    let (over, base, rate) = LUMP_SUM_RATE_SCHEDULE_CENTS
        .iter()
        .rev()
        .find(|(over, ..)| x >= over * 10)
        .copied()
        .unwrap_or(LUMP_SUM_RATE_SCHEDULE_CENTS[0]);
    base * 10 + fraction(x - over * 10, rate, 100)
}

// Lines 12 to 16: the minimum distribution allowance of an adjusted total taxable amount, in cents.
fn minimum_allowance(line12: i64) -> i64 {
    let line13 = phase::cap(fraction(line12, 1, 2), [LUMP_SUM_MINIMUM_ALLOWANCE_CAP_DOLLARS * 100]);
    let line14 = positive(line12 - LUMP_SUM_MINIMUM_ALLOWANCE_PHASEOUT_START_DOLLARS * 100);
    phase::phase_out(line13, fraction(line14, 20, 100))
}

// One participant's records, summed as the form sums them. Amounts in cents.
#[derive(Default)]
struct Group {
    method: Option<LumpSumMethod>,
    // Box 2a, box 3, box 6 (included or not), the NUA Worksheet line E, box 8a.
    taxable: i64,
    capital_gain: i64,
    nua: i64,
    nua_capital: i64,
    annuity: i64,
    estate_tax: i64,
    death_benefit: i64,
    // Hundredths of a percent; a share needs one statement.
    percent: Option<u32>,
    annuity_percent: Option<u32>,
    include_nua: bool,
}

// Every line of one Form 4972, in cents.
#[derive(Default, Debug, PartialEq, Eq)]
pub(crate) struct Form4972 {
    pub line6: i64,
    pub line7: i64,
    pub line8: i64,
    pub line9: i64,
    pub line10: i64,
    pub line11: i64,
    pub line12: i64,
    pub line16: i64,
    pub line17: i64,
    pub line18: i64,
    pub line19: i64,
    pub line21: i64,
    pub line22: i64,
    pub line25: i64,
    pub line28: i64,
    pub line29: i64,
    pub tax_cents: i64,
}

impl Form4972 {
    pub(crate) fn line30(&self) -> i64 {
        dollars(self.tax_cents)
    }
}

impl Group {
    fn figure(&self) -> Form4972 {
        let method = self.method.unwrap_or(LumpSumMethod::TenYear);
        let mut f = Form4972::default();
        // Step 2 of the multiple-recipient worksheet restates a recipient's share as the whole
        // distribution (box 9a); the form is silent on lines 9 and 18, so the recipient's share
        // of the death benefit exclusion and of the estate tax is divided the same way.
        let whole_of = |value: i64| match self.percent {
            Some(p) => fraction(value, 10_000, i64::from(p)),
            None => value,
        };
        let nua = if self.include_nua { self.nua } else { 0 };
        let mut estate_capital = 0;
        let mut death_capital = 0;
        if method.capital_gain() {
            // NUA Worksheet line G, then the Death Benefit Worksheet's capital-gain share
            // (line E = line D x the capital-gain share of the distribution) and the estate
            // tax's.
            let capital = self.capital_gain + self.nua_capital;
            let whole = self.taxable + nua;
            if whole > 0 {
                estate_capital = fraction(self.estate_tax, capital, whole);
                death_capital = fraction(self.death_benefit, capital, whole);
            }
            f.line6 = positive(capital - estate_capital - death_capital);
            f.line7 = fraction(f.line6, 20, 100);
        }
        if method.ten_year() {
            // Line 8: box 2a, less box 3 when Part II is completed, plus the net unrealized
            // appreciation (all of box 6 without Part II; the non-capital part with it).
            let ordinary = if method.capital_gain() {
                self.taxable - self.capital_gain + nua - self.nua_capital
            } else {
                self.taxable + nua
            };
            f.line8 = whole_of(ordinary);
            // Line 9: the death benefit exclusion of a participant who died before 1996-08-21,
            // line D less the capital-gain share E when Part II is completed.
            f.line9 = phase::cap(whole_of(self.death_benefit - death_capital), [f.line8]);
            f.line10 = f.line8 - f.line9;
            f.line11 = match self.annuity_percent {
                Some(p) => fraction(self.annuity, 10_000, i64::from(p)),
                None => self.annuity,
            };
            f.line12 = f.line10 + f.line11;
            f.line16 = minimum_allowance(f.line12);
            f.line17 = f.line12 - f.line16;
            f.line18 = positive(whole_of(self.estate_tax - estate_capital));
            f.line19 = positive(f.line17 - f.line18);
            f.line25 = ten_times_tax(f.line19);
            if f.line11 > 0 {
                f.line21 = fraction(f.line16, f.line11, f.line12);
                f.line22 = f.line11 - f.line21;
                f.line28 = ten_times_tax(f.line22);
            }
            f.line29 = f.line25 - f.line28;
            if let Some(p) = self.percent {
                f.line29 = fraction(f.line29, i64::from(p), 10_000);
            }
        }
        f.tax_cents = f.line7 + f.line29;
        f
    }
}

// The percent a page field holds, as exact hundredths: the one conversion of the page's
// two-decimal percent (box 9a, box 8b).
pub(crate) fn hundredths(percent: Option<f64>) -> Option<u32> {
    percent.map(|p| (p * 100.0 + 0.5) as u32)
}
pub(crate) fn percent_of(hundredths: Option<u32>) -> Option<f64> {
    hundredths.map(|h| f64::from(h) / 100.0)
}

// Who a record's participant is: the filer's own plan, or a decedent by birth and death date.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Participant {
    owner: u8,
    born: u32,
    died: u32,
}

fn participant(entry: &FederalIncomeEntry, data: &Form4972Election) -> Participant {
    let owner = u8::from(entry.owner == Owner::Spouse);
    match data.participant {
        Form4972Participant::Own { .. } => Participant { owner, born: 0, died: 0 },
        Form4972Participant::Decedent { born, died, .. } => Participant { owner, born, died },
    }
}

// A code A record that is not elected: its owner and date of death identify the participant as
// far as the record states it (a 7A is the owner's own plan; a 4A names only the date of death).
fn left_with_other_income(entry: &FederalIncomeEntry) -> Option<(u8, u32)> {
    let FederalIncome::Retirement { data: v } = &entry.income else {
        return None;
    };
    let a = crate::DistributionCodes::CODE_A.0;
    if v.box7 & a == 0 || v.kind != RetirementKind::Pension {
        return None;
    }
    let died = match &v.ordinary_distribution {
        RetirementTreatment::LumpSum { .. } => return None,
        RetirementTreatment::Exception {
            facts: crate::RetirementException::Death { died },
            ..
        } => *died,
        _ => 0,
    };
    Some((u8::from(entry.owner == Owner::Spouse), died))
}

// Every Form 4972 of the return: the issues about records of one participant, the form lines, and
// Form 1040 line 16 box 2 in whole dollars (the sum of the line 30 amounts). The records left out
// were already replaced by unknowns::supported.
pub(crate) fn calculate(d: &FederalDraft, r: &mut FederalResult) -> i64 {
    let mut groups: Vec<(Participant, Vec<&FederalIncomeEntry>, Group)> = Vec::new();
    for entry in &d.income {
        let FederalIncome::Retirement { data: v } = &entry.income else {
            continue;
        };
        let RetirementTreatment::LumpSum { data } = &v.ordinary_distribution else {
            continue;
        };
        let RetirementTaxable::Payer { amount: taxable } = v.taxable else {
            continue;
        };
        let key = participant(entry, data);
        let at = groups.iter().position(|(k, ..)| *k == key).unwrap_or_else(|| {
            groups.push((key, Vec::new(), Group::default()));
            groups.len() - 1
        });
        let (_, entries, group) = &mut groups[at];
        let first = entries.is_empty();
        let differs =
            !first && (group.method != Some(data.method) || group.include_nua != data.include_nua);
        entries.push(entry);
        if first {
            group.method = Some(data.method);
            group.include_nua = data.include_nua;
        }
        let shared = data.percent.is_some() || data.annuity_percent.is_some();
        if entries.len() > 1 {
            let context = crate::interview::income_issue_context(entry);
            if differs {
                crate::require_localized!(
                    r,
                    false,
                    &context,
                    "Form 4972",
                    "ISSUE_LUMP_SUM_TREATED_DIFFERENTLY",
                    "All of a participant's lump-sum distributions in one year must be taxed the same way. Choose the same Form 4972 method for each 1099-R of this participant, or tax none of them separately.",
                );
            }
            if shared || group.percent.is_some() || group.annuity_percent.is_some() {
                crate::require_localized!(
                    r,
                    false,
                    &context,
                    "Form 4972",
                    "ISSUE_LUMP_SUM_SHARED_AND_COMBINED",
                    "A lump sum shared with other recipients (box 9a) or with an annuity share (box 8b) is figured from its own statement. It cannot be combined with another 1099-R of the same participant's lump sum.",
                );
            }
        }
        group.taxable += taxable;
        group.capital_gain += data.capital_gain;
        group.nua += data.nua;
        group.nua_capital += data.nua_capital_share(taxable);
        group.annuity += data.annuity_value;
        group.estate_tax += data.participant.estate_tax();
        group.death_benefit += data.participant.death_benefit();
        group.percent = group.percent.or(data.percent);
        group.annuity_percent = group.annuity_percent.or(data.annuity_percent);
    }
    // A code A record of the same participant taxed with the other income.
    for (key, ..) in &groups {
        let other = d.income.iter().find(|e| {
            left_with_other_income(e)
                .is_some_and(|(owner, died)| owner == key.owner && died == key.died)
        });
        if let Some(other) = other {
            crate::require_localized!(
                r,
                false,
                &crate::interview::income_issue_context(other),
                "Form 4972",
                "ISSUE_LUMP_SUM_TREATED_DIFFERENTLY",
                "All of a participant's lump-sum distributions in one year must be taxed the same way. Choose the same Form 4972 method for each 1099-R of this participant, or tax none of them separately.",
            );
        }
    }
    let mut total = 0;
    for (_, entries, group) in &groups {
        let form = group.figure();
        let id = entries[0].id;
        let method = group.method.unwrap_or(LumpSumMethod::TenYear);
        let input = "income.retirement.ordinary_distribution";
        let mut row = |line: &str, amount: i64| {
            push_line(
                r,
                Some(FormLine::new(TaxForm::Form4972, line)),
                &alloc::format!("lump_sum_{id}_line_{line}"),
                amount,
                &[input, "income.retirement.taxable"],
                SOURCE,
                msg!("EXPLAIN_LUMP_SUM_N_LINE_N"),
            );
        };
        if method.capital_gain() {
            row("6", form.line6);
            row("7", form.line7);
        }
        if method.ten_year() {
            row("8", form.line8);
            if form.line9 > 0 {
                row("9", form.line9);
                row("10", form.line10);
            }
            if form.line11 > 0 {
                row("11", form.line11);
            }
            row("12", form.line12);
            row("16", form.line16);
            row("17", form.line17);
            row("18", form.line18);
            row("19", form.line19);
            if form.line11 > 0 {
                row("21", form.line21);
                row("22", form.line22);
            }
            row("25", form.line25);
            if form.line11 > 0 {
                row("28", form.line28);
            }
            row("29", form.line29);
        }
        row("30", form.line30() * 100);
        total += form.line30();
    }
    if !groups.is_empty() {
        crate::kernel::lines::line(
            r,
            "lump_sum_tax",
            total,
            &["income.retirement.ordinary_distribution"],
            SOURCE,
            msg!("EXPLAIN_LUMP_SUM_TAX"),
        );
    }
    total
}

// The qualifying-event and earlier-payment facts of a lump-sum distribution
// (section 402(e)(4)(D)(i)): the entire balance within one tax year, the first
// payment on or after the event, and no payment after the event in an earlier
// year. Form 4972 tests its Part I through this one function; `own_birth` is
// the filer's People birth date, which the age event needs and nothing else
// asks again.
pub(crate) fn lump_sum_conditions(
    event: &NuaEvent,
    own_birth: Option<u32>,
    first_payment: u32,
    last_prior_payment: Option<u32>,
    entire_balance: bool,
) -> bool {
    entire_balance
        && event.qualifies(first_payment, own_birth)
        && event
            .date(own_birth)
            .is_some_and(|date| last_prior_payment.is_none_or(|prior| prior < date))
}

// The qualifying event, in the terms of section 402(e)(4)(D)(i). Not stored
// and not a page type: `LumpSumEvent` and the decedent's death date map onto it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NuaEvent {
    // The participant's death.
    Death { date: u32 },
    // The participant reached age 59 1/2.
    Age,
    // An employee separated from service.
    Separation { date: u32, employee: bool },
    // A self-employed participant became totally and permanently disabled.
    Disability { date: u32, eligible: bool },
}

impl NuaEvent {
    // The date the event occurred; the age event is the filer's 59th half-year.
    pub(crate) fn date(&self, own_birth: Option<u32>) -> Option<u32> {
        match *self {
            Self::Age => own_birth.map(crate::roth::age_half),
            Self::Death { date }
            | Self::Separation { date, .. }
            | Self::Disability { date, .. } => Some(date),
        }
    }
    pub(crate) fn qualifies(&self, payment: u32, own_birth: Option<u32>) -> bool {
        if !crate::federal_rules::valid_date(payment) {
            return false;
        }
        match *self {
            Self::Age => own_birth.is_some_and(|birth| {
                crate::federal_rules::valid_birth(birth) && payment >= crate::roth::age_half(birth)
            }),
            Self::Death { date } => crate::federal_rules::valid_birth(date) && date <= payment,
            Self::Separation { date, employee } => {
                employee && crate::federal_rules::valid_birth(date) && date <= payment
            }
            Self::Disability { date, eligible } => {
                eligible && crate::federal_rules::valid_birth(date) && date <= payment
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn group(taxable: i64) -> Group {
        Group { method: Some(LumpSumMethod::TenYear), taxable: taxable * 100, ..Group::default() }
    }
    // Form 4972 line 30 in whole dollars for a plain 10-year distribution of `taxable` dollars.
    fn tax(taxable: i64) -> i64 {
        group(taxable).figure().line30()
    }

    // Form 4972 page 4 Tax Rate Schedule, hand-computed on the printed (one tenth) amount:
    // line 23 = line 19 / 10, line 24 = base + rate x (line 23 - over), line 25 = 10 x line 24.
    #[test]
    fn the_schedule_matches_the_printed_form_at_every_bracket_boundary() {
        // (line 19 dollars, expected line 25 dollars). A bracket's lower bound and one dollar above.
        let cases: &[(i64, i64)] = &[
            (0, 0),
            // 11% to 11,900: 11% x 11,900 = 1,309.00 (the next bracket's base).
            (11_900, 1_309),
            // + 1 dollar at 12%: 1,309 + 0.12 = 1,309.12 -> rounds to 1,309 in whole dollars.
            (11_901, 1_309),
            // 22,700: 2,605.
            (22_700, 2_605),
            (22_701, 2_605),
            // 45,300: 5,769.
            (45_300, 5_769),
            (45_301, 5_769),
            // 66,900: 9,009.
            (66_900, 9_009),
            (66_901, 9_009),
            // 91,700: 12,977.
            (91_700, 12_977),
            (91_701, 12_977),
            // 114,400: 17,063.
            (114_400, 17_063),
            (114_401, 17_063),
            // 137,100: 21,603.
            (137_100, 21_603),
            (137_101, 21_603),
            // 171,600: 29,538.
            (171_600, 29_538),
            (171_601, 29_538),
            // 228,800: 44,410.
            (228_800, 44_410),
            (228_801, 44_410),
            // 286,000: 61,570.
            (286_000, 61_570),
            (286_001, 61_570),
            // 343,200: 81,018.
            (343_200, 81_018),
            (343_201, 81_018),
            // 423,000: 111,342.
            (423_000, 111_342),
            (423_001, 111_342),
            // 571,900: 173,880.
            (571_900, 173_880),
            (571_901, 173_880),
            // 857,900: 311,160; one dollar above adds 50 cents, which rounds up.
            (857_900, 311_160),
            (857_901, 311_161),
            // 1,000,000: 311,160 + 50% x 142,100 = 382,210.
            (1_000_000, 382_210),
        ];
        for &(line19, expected) in cases {
            assert_eq!(dollars(ten_times_tax(line19 * 100)), expected, "line 19 {line19}");
        }
    }

    // Lines 13 to 16 on a plain distribution (no annuity, no estate tax): line 12 = 2a.
    #[test]
    fn the_minimum_distribution_allowance_boundaries() {
        // 2a 20,000: line 13 = min(10,000, 10,000) = 10,000; line 14 = 0; line 16 = 10,000;
        // line 19 = 10,000; line 25 = 10 x 11% x 1,000 = 1,100.
        assert_eq!(minimum_allowance(2_000_000), 1_000_000);
        assert_eq!(tax(20_000), 1_100);
        // 2a 50,000: line 13 = 10,000; line 14 = 30,000; line 15 = 6,000; line 16 = 4,000;
        // line 19 = 46,000; line 25 = 5,769 + 15% x 700 = 5,874.
        assert_eq!(minimum_allowance(5_000_000), 400_000);
        assert_eq!(tax(50_000), 5_874);
        // 2a 70,000: line 14 = 50,000; line 15 = 10,000; line 16 = 0; line 25 = 9,009 + 16% x
        // 3,100 = 9,505.
        assert_eq!(minimum_allowance(7_000_000), 0);
        assert_eq!(tax(70_000), 9_505);
        // Below 20,000 the allowance is half of line 12: 15,000 -> 7,500; line 19 = 7,500; line 25
        // = 10 x 11% x 750 = 825.
        assert_eq!(minimum_allowance(1_500_000), 750_000);
        assert_eq!(tax(15_000), 825);
        // One dollar under 70,000: line 14 = 49,999; line 15 = 9,999.80; line 16 = 0.20.
        assert_eq!(minimum_allowance(6_999_900), 20);
    }

    // Pub 575 (2024) Example 2, lines 11 to 28 on the annuity contract.
    #[test]
    fn the_annuity_contract_share_removes_its_own_tax() {
        let mut g = group(160_000);
        g.annuity = 10_000 * 100;
        let f = g.figure();
        assert_eq!(f.line12, 17_000_000);
        assert_eq!(f.line16, 0);
        assert_eq!(f.line21, 0);
        assert_eq!(f.line22, 1_000_000);
        assert_eq!(f.line25, 2_917_000);
        assert_eq!(f.line28, 110_000);
        assert_eq!(f.line30(), 28_070);
    }
}

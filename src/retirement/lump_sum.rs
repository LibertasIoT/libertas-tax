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
use crate::kernel::ratio::fraction;
use libertas_macros::*;

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
    pub(crate) fn capital_gain(self) -> bool {
        !matches!(self, Self::TenYear)
    }
    pub(crate) fn ten_year(self) -> bool {
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
                crate::dates::valid_birth(date)
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
    /// Include the box 6 net unrealized appreciation in income now
    pub include_nua: bool,
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
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
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
    pub(crate) fn estate_tax(&self) -> i64 {
        match *self {
            Self::Own { .. } => 0,
            Self::Decedent { estate_tax, .. } => estate_tax,
        }
    }
    pub(crate) fn death_benefit(&self) -> i64 {
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
        crate::accepted_filers::attained_on(birth, 50, true, 19851231)
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
    pub(crate) fn valid(&self, taxable: i64, source: &crate::FederalRetirement) -> bool {
        self.valid_with_source_alignment(taxable, source, true)
    }
    pub(crate) fn valid_with_source_alignment(
        &self,
        taxable: i64,
        source: &crate::FederalRetirement,
        source_alignment: bool,
    ) -> bool {
        [source.box_3, source.box_6, source.box_8a].iter().all(|v| (0..=MAX).contains(v))
            && (!source_alignment || source.box_3 <= taxable)
            && (!source_alignment || !self.include_nua || source.box_6 > 0)
            && (!source_alignment || source.box_8b_percent.is_none() || source.box_8a > 0)
            && (!source_alignment || !self.method.capital_gain() || source.box_3 > 0)
            && percent_valid(source.box_8b_percent)
            && percent_valid(source.box_9a_percent)
            && crate::dates::source_valid_date(self.first_payment)
            && (20260101..=20261231).contains(&self.first_payment)
            && self
                .last_prior_payment
                .is_none_or(|date| crate::dates::valid_birth(date) && date < 20260101)
            && match &self.participant {
                Form4972Participant::Own { event } => event.valid(),
                Form4972Participant::Decedent { born, died, estate_tax, death_benefit } => {
                    crate::dates::valid_birth(*born)
                        && crate::dates::valid_birth(*died)
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
    pub(crate) fn nua_capital_share(&self, taxable: i64, source: &crate::FederalRetirement) -> i64 {
        if self.method.capital_gain() && self.include_nua && taxable > 0 {
            fraction(source.box_6, source.box_3, taxable)
        } else {
            0
        }
    }
    // The ordinary part Part II alone leaves on Form 1040 lines 5a and 5b (How To Report item 2);
    // Part III takes the whole of it otherwise, so nothing stays on lines 5a and 5b.
    pub(crate) fn ordinary_part(&self, taxable: i64, source: &crate::FederalRetirement) -> i64 {
        if self.method.ten_year() {
            return 0;
        }
        let nua = if self.include_nua {
            source.box_6 - self.nua_capital_share(taxable, source)
        } else {
            0
        };
        taxable - source.box_3 + nua
    }
}

// The 1099-R page onto the stored election and back. The boxes 3, 6, 8a, 8b and 9a come from the
// row's Other boxes; the two-decimal percents become exact hundredths here, once.
impl Form4972Election {
    pub(crate) fn from_own(tax: &crate::LumpSumTax) -> Option<Self> {
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
        })
    }
    pub(crate) fn from_decedent(died: u32, tax: &crate::InheritedLumpSumTax) -> Option<Self> {
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
        })
    }
    #[cfg(test)]
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
    #[cfg(test)]
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
}

// The server twins of the R1099 rules about the Other boxes and the Form 4972 answers (the keys are
// the client rules' keys), first failure first.
pub(crate) fn page_issue(v: &crate::R1099Answered) -> Option<crate::validation::ValidationIssue> {
    use crate::{DistributionCode as C, InheritedLumpSumTax as I, LumpSumTax as L};
    let more = &**v;
    if more.box_3.unwrap_or(0) > v.taxable.unwrap_or(0) {
        return Some(
            crate::validation::ValidationIssue::new("R_BOX3_ABOVE_TAXABLE").at_field("box_3"),
        );
    }
    if more.box_8b_percent.is_some() && more.box_8a.unwrap_or(0) <= 0 {
        return Some(
            crate::validation::ValidationIssue::new("R_BOX8B_WITHOUT_BOX8A")
                .at_field("box_8b_percent"),
        );
    }
    // (method, include_nua, first payment, last earlier payment, date of birth and death)
    let (method, include_nua, first, last, born_died) = match v.code.as_ref()? {
        C::NormalLumpSum {
            tax: L::Form4972 { method, include_nua, first_payment, last_prior_payment, .. },
        } => (*method, *include_nua, *first_payment, *last_prior_payment, None),
        C::DeathLumpSum {
            died,
            tax: I::Form4972 { born, method, include_nua, first_payment, last_prior_payment, .. },
        } => (*method, *include_nua, *first_payment, *last_prior_payment, Some((*born, *died))),
        _ => return None,
    };
    if method.capital_gain() && more.box_3.unwrap_or(0) <= 0 {
        return Some(
            crate::validation::ValidationIssue::new("LUMP_SUM_CAPITAL_GAIN_WITHOUT_BOX_3")
                .at_field("method")
                .in_field("tax")
                .in_field("code"),
        );
    }
    if include_nua && more.box_6.unwrap_or(0) <= 0 {
        return Some(
            crate::validation::ValidationIssue::new("LUMP_SUM_NUA_WITHOUT_BOX_6")
                .at_field("include_nua")
                .in_field("tax")
                .in_field("code"),
        );
    }
    if !(matches!(v.taxable_method, Some(crate::TaxableMethod::BoxTwoA))
        && more.box_10.unwrap_or(0) == 0
        && matches!(v.disability(), crate::DisabilityIncome::None)
        && matches!(v.disaster(), crate::DisasterDistribution::No)
        && v.public_safety().is_empty())
    {
        return Some(
            crate::validation::ValidationIssue::new("LUMP_SUM_OTHER_TREATMENTS")
                .at_field("taxable_method"),
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

// The percent a stored box holds (box 9a, box 8b), as exact hundredths: the one conversion of
// the two-decimal percent, through the shared natural-unit boundary.
pub(crate) fn hundredths(percent: Option<f64>) -> Option<u32> {
    percent.and_then(|p| u32::try_from(crate::kernel::natural::units(p, 100)).ok())
}
// A blank box, or a printed percent from 0.01 to 99.99 on the two-decimal grid.
pub(crate) fn percent_valid(percent: Option<f64>) -> bool {
    percent.is_none_or(|p| {
        crate::kernel::natural::exact(p, 100).is_some_and(|h| (1..=9999).contains(&h))
    })
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
            Self::Age => own_birth.map(crate::roth_source::age_half),
            Self::Death { date }
            | Self::Separation { date, .. }
            | Self::Disability { date, .. } => Some(date),
        }
    }
    pub(crate) fn qualifies(&self, payment: u32, own_birth: Option<u32>) -> bool {
        if !crate::dates::source_valid_date(payment) {
            return false;
        }
        match *self {
            Self::Age => own_birth.is_some_and(|birth| {
                crate::dates::valid_birth(birth) && payment >= crate::roth_source::age_half(birth)
            }),
            Self::Death { date } => crate::dates::valid_birth(date) && date <= payment,
            Self::Separation { date, employee } => {
                employee && crate::dates::valid_birth(date) && date <= payment
            }
            Self::Disability { date, eligible } => {
                eligible && crate::dates::valid_birth(date) && date <= payment
            }
        }
    }
}

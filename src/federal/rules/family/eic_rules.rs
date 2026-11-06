// eic items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct FamilyCreditsIn<'a> {
    pub(super) bans: crate::forms::form_8862::Bans,
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) interest: i64,
    pub(super) exempt: i64,
    pub(super) dividends: i64,
    pub(super) short: i64,
    pub(super) long: i64,
    pub(super) property_character: &'a Option<crate::section1231::Result1231>,
    pub(super) property_net: i64,
    pub(super) foreign: &'a crate::extended_rules::ForeignResult,
    pub(super) earned: [i64; 2],
    pub(super) adoption_benefits: &'a crate::adoption::Benefits,
    pub(super) optional_eic: [i64; 2],
    pub(super) agi: i64,
    pub(super) foreign_magi: i64,
    pub(super) optional_child: [i64; 2],
    pub(super) payroll_withheld: i64,
    pub(super) medicare_withheld: i64,
    pub(super) additional_medicare_rrta_withheld: i64,
    pub(super) additional_medicare: i64,
    pub(super) payroll_other: i64,
    pub(super) half_se: i64,
    pub(super) excess_social: i64,
    pub(super) care: i64,
    pub(super) elderly_potential: i64,
    pub(super) education_nonrefund: i64,
    pub(super) saver_available: i64,
    pub(super) creditable_tax: i64,
    pub(super) clean_vehicle: &'a crate::clean_vehicle::PersonalCredits,
    pub(super) mortgage_credit_current: i64,
}

pub(super) struct FamilyCreditsOut {
    pub(super) adoption: crate::adoption::Credit,
    pub(super) homeowner_potentials: [i64; 4],
    pub(super) combat_pay_election: i64,
    pub(super) schedule_eic: crate::forms::schedule_eic::ScheduleEic,
    pub(super) eic: i64,
    pub(super) schedule_8812: crate::forms::schedule_8812::Schedule8812,
}

pub(super) fn family_credits_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    i: FamilyCreditsIn<'_>,
) -> FamilyCreditsOut {
    let &Pass { d, s, p, c, j, separate, dependent_filer, family, .. } = pass;
    let FamilyCreditsIn {
        bans,
        rental,
        interest,
        exempt,
        dividends,
        short,
        long,
        property_character,
        property_net,
        foreign,
        earned,
        adoption_benefits,
        optional_eic,
        agi,
        foreign_magi,
        optional_child,
        payroll_withheld,
        medicare_withheld,
        additional_medicare_rrta_withheld,
        additional_medicare,
        payroll_other,
        half_se,
        excess_social,
        care,
        elderly_potential,
        education_nonrefund,
        saver_available,
        creditable_tax,
        clean_vehicle,
        mortgage_credit_current,
    } = i;
    if c.decline_earned_income_credit {
        note_once_scoped(
            r,
            msg!("NOTE_EIC_DECLINED"),
            crate::IssueContext::Section(crate::TaxSection::MoreCredits),
        );
    }

    // Form 8880 follows care/education; Schedule 8812 also subtracts the allowed saver credit.
    // Applying it after CTC would understate ACTC when the nonrefundable child credit shrinks.
    // A running Form 8862 ban on the child credits leaves the people as
    // dependents but takes the credits (section 24(g)).
    let child_banned = bans.child.filter(|_| family.ctc + family.odc > 0);
    if let Some(ban) = child_banned {
        crate::forms::form_8862::note_ban(r, crate::forms::form_8862::CreditGroup::Child, ban);
    }
    let (credit_children, credit_others) =
        if child_banned.is_some() { (0, 0) } else { (family.ctc, family.odc) };
    let mut eic_context = crate::forms::schedule_eic::ScheduleEicContext {
        people: p,
        rental,
        interest,
        exempt,
        child_exempt: t.child.exempt,
        child_other: t.child.investment,
        dividends,
        capital: short + long,
        property_long_term: property_character.as_ref().map_or(0, |value| value.long_term),
        property_net,
        foreign_election: foreign.election(),
        separate,
        separated: s.eic_separated(),
        dependent_filer,
        children: family.eic_children,
        children_without_ssn: family.eic_ssnless_ids.len(),
        dependents: d.dependents(),
        child_ids: &family.eic_child_ids,
        children_without_ssn_ids: &family.eic_ssnless_ids,
        earned: earned.iter().sum::<i64>() - adoption_benefits.deemed_excess
            + optional_eic.iter().sum::<i64>(),
        agi,
        joint: j,
        banned: bans.eic.is_some() || c.decline_earned_income_credit,
    };
    let mut child_context = crate::forms::schedule_8812::Schedule8812Context {
        children: credit_children,
        other_dependents: credit_others,
        agi,
        magi: foreign_magi,
        joint: j,
        foreign_election: foreign.election(),
        // Form 1040 line 1z: the joint special-needs deemed excess is the
        // negative line 1f (Form 8839 line 31), which reduces earned income.
        earned: earned.iter().sum::<i64>() + dollars(t.combat_pay.iter().sum())
            - adoption_benefits.deemed_excess
            + optional_child.iter().sum::<i64>(),
        withheld: payroll_withheld - (medicare_withheld - additional_medicare_rrta_withheld)
            + additional_medicare
            + dollars(t.rrta_medicare.iter().sum()),
        other_payroll: payroll_other,
        half_se,
        eic: 0,
        excess_social,
    };
    let adoption = crate::adoption::calculate(&c.adoptions, &c.adoption_carry, foreign_magi, s, r);
    let fixed_limits = crate::income_tax::PersonalCreditPotentials {
        care,
        elderly: elderly_potential,
        education: education_nonrefund,
        saver: saver_available,
        adoption: 0,
        child: 0,
        actc_limit: 0,
    }
    .limit(creditable_tax);
    let fixed_preceding =
        fixed_limits.care + fixed_limits.elderly + fixed_limits.education + fixed_limits.saver;
    let clean_used_for_choices = phase::cap(
        clean_vehicle.used,
        [crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
            creditable_tax,
            fixed_preceding,
        )
        .get()],
    );
    let clean_new_for_choices = phase::cap(
        clean_vehicle.new,
        [crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
            creditable_tax - fixed_preceding,
            clean_used_for_choices,
        )
        .get()],
    );
    let preceding_for_choices = fixed_preceding + clean_used_for_choices + clean_new_for_choices;
    let mortgage_for_choices = mortgage_credit_current
        + c.mortgage_credit.as_ref().map_or(0, |v| {
            dollars(v.carry_three_years_prior)
                + dollars(v.carry_two_years_prior)
                + dollars(v.carry_prior_year)
        });
    let homeowner_potentials = [
        mortgage_for_choices,
        adoption.nonrefundable,
        c.dc_homebuyer_carry.map_or(0, dollars),
        c.residential_energy_carry.map_or(0, dollars),
    ];
    let base_eic_earned = eic_context.earned;
    let base_child_earned = child_context.earned;
    let excluded_medicaid = core::array::from_fn::<_, 2, _>(|i| {
        dollars(t.medicaid_excluded[i] + t.medicaid_in_box1[i])
    });
    let combat = t.combat_pay.map(dollars);
    // Exclude first and keep the first exact tie. The owner's combat and waiver choices are independent.
    let mut winner = None;
    let mut best = -1;
    for bits in 0u32..16 {
        if (0..2).any(|i| {
            ((bits & (1 << (2 * i))) != 0 && combat[i] == 0)
                || ((bits & (1 << (2 * i + 1))) != 0 && excluded_medicaid[i] == 0)
        }) {
            continue;
        }
        let elected_combat: i64 =
            (0..2).filter(|i| bits & (1 << (2 * i)) != 0).map(|i| combat[i]).sum();
        let elected_medicaid: i64 =
            (0..2).filter(|i| bits & (1 << (2 * i + 1)) != 0).map(|i| excluded_medicaid[i]).sum();
        eic_context.earned = base_eic_earned + elected_combat + elected_medicaid;
        let candidate_eic = crate::forms::schedule_eic::compute(
            &eic_context,
            &mut FederalResult::empty(d.revision),
        );
        child_context.eic = candidate_eic.credit;
        child_context.earned = base_child_earned
            + if candidate_eic.credit > 0 {
                elected_medicaid
            } else {
                excluded_medicaid.iter().sum()
            };
        let candidate_child = crate::forms::schedule_8812::compute(&child_context);
        let homeowners = candidate_child.homeowner_credits(
            creditable_tax,
            preceding_for_choices,
            homeowner_potentials,
        );
        let candidate_additional = candidate_child
            .limit(creditable_tax, preceding_for_choices + homeowners.total())
            .additional;
        let total = candidate_eic.credit + candidate_additional;
        if total > best {
            best = total;
            winner = Some((eic_context.earned, child_context.earned, elected_combat));
        }
    }
    let (elected_eic_earned, elected_child_earned, combat_pay_election) =
        winner.expect("the exclude-all election is always available");
    eic_context.earned = elected_eic_earned;
    let schedule_eic = crate::forms::schedule_eic::compute(&eic_context, r);
    let eic = schedule_eic.credit;
    child_context.eic = eic;
    child_context.earned = elected_child_earned;
    let schedule_8812 = crate::forms::schedule_8812::compute(&child_context);
    form_line(
        r,
        FormLine::new(TaxForm::Schedule8812, "18b"),
        "child_credit_combat_pay",
        dollars(t.combat_pay.iter().sum()),
        &["income.wages"],
        "https://www.irs.gov/instructions/i1040s8",
        msg!("EXPLAIN_8812_COMBAT_PAY"),
    );
    line(
        r,
        "combat_pay_election",
        combat_pay_election,
        &["income.wages"],
        "https://www.irs.gov/instructions/i1040gi",
        msg!("EXPLAIN_1040_1I"),
    );
    if let Some(ban) = bans.eic
        && schedule_eic.banned > 0
        && !c.decline_earned_income_credit
    {
        crate::forms::form_8862::note_ban(
            r,
            crate::forms::form_8862::CreditGroup::EarnedIncome,
            ban,
        );
    }
    FamilyCreditsOut {
        adoption,
        homeowner_potentials,
        combat_pay_election,
        schedule_eic,
        eic,
        schedule_8812,
    }
}

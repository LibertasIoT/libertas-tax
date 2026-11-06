// Unit B11 KIDS (2026-10-11): the Form 8863 scholarship election, the credit for
// other dependents for another taxpayer's qualifying child, the Form 8862 bans, a
// child who died, Form 8839 Part III for joint employees, Form 8615 with the
// parent's Form 2555 and Form 4952 election and a mixed-source business, the
// Coverdell prohibited transaction and W-2 box 12 code T. Every expected amount
// is worked by hand from the 2026 brackets (Rev. Proc. 2025-32 section 3.01) and
// the 2026 Tax Table, whose rows tax a midpoint (a row "at least 57,800 but less
// than 57,850" taxes 57,825), so a hand figure ending in .5 rounds up.
use super::*;
use crate::tests::cross::interview::{accepted, rejected_with, row, save_children};

fn line_amount(r: &FederalResult, id: &str) -> i64 {
    r.lines.iter().find(|l| l.id == id).map_or(0, |l| l.amount / 100)
}

fn note_args(r: &FederalResult, key: &str) -> Vec<MessageArg> {
    r.review_notes
        .iter()
        .find(|n| n.display.key == key)
        .unwrap_or_else(|| panic!("no note {key}: {:?}", r.review_notes))
        .display
        .args
        .clone()
}

// ---------------------------------------------------------------------------
// Form 8863: the part of the box 5 grant a student includes in income
// ---------------------------------------------------------------------------

fn entry(
    student: i64,
    assistance: i64,
    inclusion: ScholarshipInclusion,
) -> (FederalEducation, EducationStudent) {
    {
        let mut pair = native_education(
            student,
            FederalEducationExpenses {
                assistance,
                scholarship_in_income: inclusion,
                ..education_expenses(assistance)
            },
            EducationChoice::AmericanOpportunity,
        );
        pair.0.id = 0;
        pair.0.half_time = true;
        pair.0.graduate = false;
        pair.1.more.american_opportunity_used_up = Some(false);
        pair
    }
}

// Married filing jointly, W-2 wages $90,000, one child born 2007-03-01 (19 at
// the end of 2026, a full-time student with an SSN who lived at home all year).
// The child is the student: 1098-T box 1 $5,000, box 5 $5,000 (a Pell grant).
fn pell_family(inclusion: ScholarshipInclusion) -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 9_000_000, 800_000);
    add_child(&mut d);
    d.dependents[0].birth_date = 20070301;
    d.dependents[0].situation = DependentSituation::Student;
    let student = d.dependents[0].id;
    set_education(d.credits.as_mut().unwrap(), vec![entry(student, 500_000, inclusion)]);
    d
}

#[test]
fn f1_a_dependent_students_pell_grant_raises_the_parents_credit() {
    // Without the choice the whole $5,000 grant pays the $5,000 tuition:
    // adjusted qualified expenses are $0 and there is no credit.
    let none = supported(&pell_family(ScholarshipInclusion::None));
    assert_eq!(line_amount(&none, "education_credit"), 0);
    assert_eq!(line_amount(&none, "refundable_education_credit"), 0);
    // The student includes $4,000 as paying room and board (Pub. 970 ch. 2):
    // the credit's assistance is $1,000, so line 27 is $5,000 - $1,000 = $4,000;
    // line 28 $2,000, line 29 25% = $500, line 30 $2,000 + $500 = $2,500. MAGI
    // $90,000 is under $160,000. The parents are over 24: 40% refundable =
    // $1,000, and $1,500 nonrefundable.
    //   Tax: $90,000 - $32,200 = $57,800, row midpoint $57,825:
    //   $2,480 + 12% x ($57,825 - $24,800) = $2,480 + $3,963 = $6,443.
    //   Credit for other dependents $500 (a 19-year-old student): $5,943;
    //   less the $1,500 nonrefundable education credit = $4,443.
    for inclusion in [
        ScholarshipInclusion::Amount { included: 400_000 },
        ScholarshipInclusion::Automatic { living_costs: 400_000 },
    ] {
        let d = pell_family(inclusion.clone());
        let r = supported(&d);
        assert_eq!(line_amount(&r, "refundable_education_credit"), 1_000);
        assert_eq!(line_amount(&r, "education_credit"), 1_500);
        assert_eq!(return_line(&r, "16"), 6_443 * 100);
        assert_eq!(value(&r, "child_credit"), 500);
        assert_eq!(r.tax, Some(4_443 * 100));
        // A dependent student's part is the student's own income: it is not on
        // this return's Schedule 1 line 8r, and the family is told.
        assert_eq!(value(&r, "agi"), 90_000);
        assert_eq!(line_amount(&r, "taxable_scholarships"), 0);
        assert_eq!(value(&r, "education_1_scholarship_included"), 4_000);
        let args = note_args(&r, "NOTE_SCHOLARSHIP_INCLUDED_BY_STUDENT");
        assert_eq!(args[1], MessageArg::usd_dollars(4_000), "{inclusion:?}");
        assert!(r.review_notes.iter().any(|n| {
            n.context
                .as_ref()
                .is_some_and(|c| *c == crate::IssueContext::Section(TaxSection::Education))
        }));
    }
    // F1b: a grant restricted to tuition cannot be included; None is the choice
    // and the credit is $0 (shown above).
}

#[test]
fn an_automatic_choice_for_a_dependent_keeps_the_smallest_part_that_reaches_the_cap() {
    // Box 1 $5,000, box 5 $5,000, living costs $9,000: the candidates are $0,
    // $2,000 (credit $2,000), $4,000 (credit $2,500, the cap) and the most the
    // grant allows ($5,000, credit $2,500). The smallest part that reaches the
    // largest credit is $4,000.
    let r = supported(&pell_family(ScholarshipInclusion::Automatic { living_costs: 900_000 }));
    assert_eq!(value(&r, "education_1_scholarship_included"), 4_000);
    assert_eq!(line_amount(&r, "refundable_education_credit"), 1_000);
    // Living costs of $1,500 limit the part to $1,500: line 27 is $5,000 -
    // ($5,000 - $1,500) = $1,500, and the credit is 100% of it.
    let r = supported(&pell_family(ScholarshipInclusion::Automatic { living_costs: 150_000 }));
    assert_eq!(value(&r, "education_1_scholarship_included"), 1_500);
    assert_eq!(line_amount(&r, "refundable_education_credit"), 600);
    assert_eq!(line_amount(&r, "education_credit"), 900);
}

// Bill (Pub. 970 Example 3 with 2026 amounts): single, 28, W-2 $39,250 with
// $3,000 withheld, the student himself; 1098-T box 1 $5,600, box 5 $5,600.
fn bill(inclusion: ScholarshipInclusion) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19980301;
    d.income[0] = wage(1, Owner::Taxpayer, 3_925_000, 300_000);
    set_education(
        d.credits.as_mut().unwrap(),
        vec![{
            let mut pair = native_education(
                -1,
                FederalEducationExpenses {
                    tuition: 560_000,
                    assistance: 560_000,
                    scholarship_in_income: inclusion,
                    ..education_expenses(560_000)
                },
                EducationChoice::AmericanOpportunity,
            );
            pair.0.id = 0;
            pair.0.half_time = true;
            pair.0.graduate = false;
            pair.1.more.american_opportunity_used_up = Some(false);
            pair
        }],
    );
    d
}

#[test]
fn f2_bill_includes_the_part_with_the_lowest_total_tax() {
    // X = part of the grant included: tax from the Tax Table midpoints, less the
    // American opportunity credit (all of it, refundable and not):
    //   X      taxable   row midpoint   tax                    credit  net
    //   0      23,150    23,175         1,240 + 12% x 10,775 = 2,533    0   2,533
    //   2,000  25,150    25,175         1,240 + 12% x 12,775 = 2,773 2,000   773
    //   4,000  27,150    27,175         1,240 + 12% x 14,775 = 3,013 2,500   513
    //   4,400  27,550    27,575         1,240 + 12% x 15,175 = 3,061 2,500   561
    // The App keeps $4,000.
    let automatic = supported(&bill(ScholarshipInclusion::Automatic { living_costs: 440_000 }));
    assert_eq!(value(&automatic, "education_1_scholarship_included"), 4_000);
    assert_eq!(value(&automatic, "taxable_scholarships"), 4_000);
    assert_eq!(value(&automatic, "agi"), 43_250);
    assert_eq!(return_line(&automatic, "16"), 3_013 * 100);
    assert_eq!(value(&automatic, "refundable_education_credit"), 1_000);
    assert_eq!(value(&automatic, "education_credit"), 1_500);
    assert_eq!(automatic.tax, Some(1_513 * 100));
    // Payments $3,000 + refundable $1,000 against the tax $1,513.
    assert_eq!(automatic.refund, Some(2_487 * 100));
    assert_eq!(printed_line(&automatic, "taxable_scholarships"), (TaxForm::Schedule1, "8r".into()));
    // The same figures as an explicit choice of $4,000; and each other
    // candidate worked above leaves a larger net tax.
    for (part, net) in [(0, 2_533), (200_000, 773), (400_000, 513), (440_000, 561)] {
        let inclusion = if part == 0 {
            ScholarshipInclusion::None
        } else {
            ScholarshipInclusion::Amount { included: part }
        };
        let r = supported(&bill(inclusion));
        let cost = r.tax.unwrap() - r.refundable_credits.unwrap();
        assert_eq!(cost, net * 100, "{part}");
    }
    // F2b: choosing $4,400 outright.
    let r = supported(&bill(ScholarshipInclusion::Amount { included: 440_000 }));
    assert_eq!(return_line(&r, "16"), 3_061 * 100);
    assert_eq!(value(&r, "taxable_scholarships"), 4_400);
}

#[test]
fn f2c_an_included_part_that_lowers_the_earned_income_credit_is_weighed() {
    // A head of household with one child and wages of $22,000 who is the student:
    // the part included raises AGI and the earned income credit falls once AGI
    // passes $23,890 (the plateau's end for one child in the draft table), so
    // the best part is not simply the largest.
    let build = |inclusion| {
        let mut d = complete(FederalStatus::Head);
        d.people.as_mut().unwrap().taxpayer.birth_date = 19980301;
        d.income[0] = wage(1, Owner::Taxpayer, 2_200_000, 100_000);
        add_child(&mut d);
        set_education(
            d.credits.as_mut().unwrap(),
            vec![{
                let mut pair = native_education(
                    -1,
                    FederalEducationExpenses {
                        tuition: 560_000,
                        assistance: 560_000,
                        scholarship_in_income: inclusion,
                        ..education_expenses(560_000)
                    },
                    EducationChoice::AmericanOpportunity,
                );
                pair.0.id = 0;
                pair.0.half_time = true;
                pair.0.graduate = false;
                pair.1.more.american_opportunity_used_up = Some(false);
                pair
            }],
        );
        d
    };
    let automatic = supported(&build(ScholarshipInclusion::Automatic { living_costs: 440_000 }));
    let chosen = value(&automatic, "education_1_scholarship_included");
    let net = |r: &FederalResult| r.tax.unwrap() - r.refundable_credits.unwrap();
    let mut best = i64::MAX;
    for part in [0, 200_000, 400_000, 440_000] {
        let inclusion = if part == 0 {
            ScholarshipInclusion::None
        } else {
            ScholarshipInclusion::Amount { included: part }
        };
        best = best.min(net(&supported(&build(inclusion))));
    }
    assert_eq!(net(&automatic), best);
    // The earned income credit at the chosen AGI is the table amount: it is the
    // larger credit at $0 (earned income $22,000, the plateau), less once the
    // chosen part pushes AGI past $23,890.
    let none = supported(&build(ScholarshipInclusion::None));
    assert_eq!(return_line(&none, "27a"), 4_427 * 100);
    let eic = return_line(&automatic, "27a");
    // Any included part pushes AGI past the plateau and lowers the credit.
    assert!(chosen == 0 || eic < 4_427 * 100, "{chosen} {eic}");
    assert!(chosen > 0 || eic == 4_427 * 100, "{chosen} {eic}");
}

#[test]
fn the_inclusion_choice_is_stored_and_checked_on_the_education_page() {
    let mut s = Interview::new(TEST_USER, pell_family(ScholarshipInclusion::None));
    let people = crate::federal_rules::people_students(&s.draft);
    let student = s.draft.dependents[0].id;
    let page = |inclusion: ScholarshipInclusion| crate::EducationDocumentView {
        documents: vec![],
        students: vec![
            EducationStudent {
                student,
                method: Some(EducationChoice::AmericanOpportunity),
                more: StudentMore {
                    scholarship_in_income: inclusion,
                    american_opportunity_used_up: Some(false),
                    ..StudentMore::default()
                },
            }
            .into(),
        ],
        people: people.clone(),
    };
    // More than the box 5 grant is refused with the rule's key.
    let reply = crate::tests::cross::interview::save_education_answers(
        &mut s,
        page(ScholarshipInclusion::Amount { included: 500_001 })
            .students
            .iter()
            .map(EducationStudent::from)
            .collect(),
    );
    rejected_with(reply, "SCHOLARSHIP_INCLUDED_ABOVE_BOX_5");
    // Up to the grant is accepted and stored as chosen, and reopens the same.
    let reply = crate::tests::cross::interview::save_education_answers(
        &mut s,
        page(ScholarshipInclusion::Automatic { living_costs: 400_000 })
            .students
            .iter()
            .map(EducationStudent::from)
            .collect(),
    );
    accepted(&reply);
    assert_eq!(
        s.draft.credits.as_ref().unwrap().students[0].more.scholarship_in_income,
        ScholarshipInclusion::Automatic { living_costs: 400_000 }
    );
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
}

// ---------------------------------------------------------------------------
// Another taxpayer's qualifying child (section 152(d)(1)(D))
// ---------------------------------------------------------------------------

fn partners_child(facts: PersonFacts) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 700_000);
    add_child(&mut d);
    let c = &mut d.dependents[0];
    c.birth_date = 20210210;
    c.relationship = Relationship::Unrelated;
    c.facts = facts.0;
    d
}

#[test]
fn f3_a_partners_child_who_is_anothers_qualifying_child_gives_no_credit() {
    // $60,000 - $16,100 = $43,900; row midpoint $43,925:
    // $1,240 + 12% x ($43,925 - $12,400) = $1,240 + $3,783 = $5,023.
    let ticked = supported(&partners_child(PersonFacts::ANOTHERS_QUALIFYING_CHILD));
    assert_eq!(return_line(&ticked, "16"), 5_023 * 100);
    assert_eq!(line_amount(&ticked, "child_credit"), 0);
    assert!(noted(&ticked, "NOTE_ANOTHERS_QUALIFYING_CHILD"));
    // F3b: the Pub. 501 exception (the other parent need not file): unticked,
    // the credit for other dependents is $500.
    let unticked = supported(&partners_child(PersonFacts::NONE));
    assert_eq!(return_line(&unticked, "16"), 5_023 * 100);
    assert_eq!(value(&unticked, "child_credit"), 500);
    assert_eq!(unticked.tax, Some(4_523 * 100));
    assert!(!noted(&unticked, "NOTE_ANOTHERS_QUALIFYING_CHILD"));
}

#[test]
fn another_taxpayers_child_is_no_head_of_household_relative_either() {
    // Head of household through a relative needs a dependent; the tick removes
    // the person, so the return no longer qualifies.
    let mut d = partners_child(PersonFacts::ANOTHERS_QUALIFYING_CHILD);
    d.setup.as_mut().unwrap().status = FilingChoice::Head;
    d.dependents[0].relationship = Relationship::OtherRelative;
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
    assert!(r.issues.iter().any(|i| i.section.contains("Children")), "{r:?}");
    d.dependents[0].facts = 0;
    supported(&d);
}

#[test]
fn f3c_the_tick_is_only_for_another_person_you_support() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let loaded = s.draft.clone();
    let reply = send(
        &mut s,
        save_children(
            &loaded,
            vec![Dependent {
                facts: PersonFacts::ANOTHERS_QUALIFYING_CHILD.0,
                ..row("Pat", "Sample", 20210210)
            }],
        ),
    );
    rejected_with(reply, "ANOTHERS_CHILD_NEEDS_SUPPORTED");
    let reply = send(
        &mut s,
        save_children(
            &loaded,
            vec![Dependent {
                facts: PersonFacts::ANOTHERS_QUALIFYING_CHILD.0,
                relationship: Relationship::Unrelated,
                situation: DependentSituation::Supported {
                    your_support_over_half: true,
                    gross_income: 0,
                },
                ..row("Pat", "Sample", 20210210)
            }],
        ),
    );
    accepted(&reply);
    assert!(s.draft.dependents()[0].has(PersonFacts::ANOTHERS_QUALIFYING_CHILD));
}

#[test]
fn a_parent_who_could_also_claim_the_child_and_lived_with_the_child_less_is_a_tiebreaker_basis() {
    let mut d = complete(FederalStatus::Single);
    add_child(&mut d);
    d.dependents[0].competing =
        CompetingClaim::You { basis: ClaimBasis::LongerResidenceThanTheOtherParent };
    let r = supported(&d);
    assert_eq!(value(&r, "child_credit"), 2_200);
    assert!(!noted(&r, "WARN_TIEBREAKER_NOT_MET"));
    assert_eq!(restart(d.clone()).draft, d);
}

// ---------------------------------------------------------------------------
// Form 8862: a running ban is a zero credit, not a refusal
// ---------------------------------------------------------------------------

fn banned_eic(finding: DisallowanceFinding, year: u32) -> CreditsAfterDisallowance {
    CreditsAfterDisallowance {
        earned_income: EicDisallowance::Disallowed {
            facts: DisallowanceFacts { year, finding, reinstated: false },
            cause: EicDisallowanceCause::Other,
            your_us_home_days: None,
            spouse_us_home_days: None,
        },
        child_credits: CreditDisallowance::Never,
        education_credit: CreditDisallowance::Never,
    }
}

// Single, born 1990-04-10, W-2 $20,000 with $500 withheld, one child (SSN).
fn one_child(claim: CreditsAfterDisallowance) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19900410;
    d.income[0] = wage(1, Owner::Taxpayer, 2_000_000, 50_000);
    add_child(&mut d);
    d.credits.as_mut().unwrap().after_disallowance = Some(claim);
    d
}

#[test]
fn f4_an_earned_income_credit_ban_leaves_the_credit_out_and_keeps_the_rest() {
    // Taxable $20,000 - $16,100 = $3,900; row midpoint $3,925: tax $393 (10%
    // of 3,925 = 392.5, rounded). Child tax credit $2,200: nonrefundable $393,
    // additional child tax credit min($1,807, $1,700, 15% x $17,500) = $1,700.
    // Unbanned (a 2023 finding: the 2-year ban ended with 2025): the earned
    // income credit is $4,427 (one child, earned income $20,000 on the plateau)
    // and the refund $500 + $1,700 + $4,427 = $6,627.
    let open = supported(&one_child(banned_eic(DisallowanceFinding::RecklessDisregard, 2023)));
    assert_eq!(return_line(&open, "27a"), 4_427 * 100);
    assert_eq!(open.refund, Some(6_627 * 100));
    assert!(open.lines.iter().any(|l| l.id == "form_8862_line_2_eic"));
    // Banned (a 2024 finding bars 2025 and 2026): no earned income credit, no
    // Form 8862 rows for it, no Schedule EIC rows; the child credits stand:
    // refund $500 + $1,700 = $2,200.
    let banned = supported(&one_child(banned_eic(DisallowanceFinding::RecklessDisregard, 2024)));
    assert_eq!(return_line(&banned, "27a"), 0);
    assert_eq!(return_line(&banned, "28"), 170_000);
    assert_eq!(banned.refund, Some(2_200 * 100));
    assert!(banned.lines.iter().all(|l| !l.id.starts_with("form_8862_")));
    assert!(banned.lines.iter().all(|l| !l.id.starts_with("schedule_eic_")));
    let args = note_args(&banned, "NOTE_8862_EIC_BANNED");
    assert_eq!(
        args,
        vec![MessageArg::Number(2), MessageArg::Number(2024), MessageArg::Number(2026)]
    );
    // The ten-year fraud ban reads the same way.
    let fraud = supported(&one_child(banned_eic(DisallowanceFinding::Fraud, 2016)));
    assert_eq!(return_line(&fraud, "27a"), 0);
    assert!(noted(&fraud, "NOTE_8862_EIC_BANNED"));
    // A banned credit the return could not claim anyway (no qualifying child
    // and an age under 25) is no note.
    let mut childless = one_child(banned_eic(DisallowanceFinding::Fraud, 2016));
    childless.dependents.clear();
    childless.people.as_mut().unwrap().taxpayer.birth_date = 20080410;
    let r = supported(&childless);
    assert!(!noted(&r, "NOTE_8862_EIC_BANNED"));
}

#[test]
fn f4b_a_child_credit_ban_zeroes_the_credits_and_keeps_the_dependent() {
    // The credits for 2024 fraud: no child tax credit, no additional child tax
    // credit; the child stays a dependent and the earned income credit stands.
    //   Tax $393 (no credit); payments $500 + earned income credit $4,427 =
    //   $4,927; refund $4,927 - $393 = $4,534.
    let mut claim = banned_eic(DisallowanceFinding::Ineligible, 2024);
    claim.earned_income = EicDisallowance::Never;
    claim.child_credits = CreditDisallowance::Disallowed(DisallowanceFacts {
        year: 2024,
        finding: DisallowanceFinding::Fraud,
        reinstated: false,
    });
    let d = one_child(claim);
    let r = supported(&d);
    assert_eq!(return_line(&r, "27a"), 4_427 * 100);
    assert_eq!(return_line(&r, "19"), 0);
    assert_eq!(return_line(&r, "28"), 0);
    assert_eq!(r.refund, Some(4_534 * 100));
    assert!(noted(&r, "NOTE_8862_CHILD_CREDITS_BANNED"));
    assert!(r.lines.iter().all(|l| !l.id.starts_with("form_8862_")));
    // The child still qualifies the filer for head of household.
    let mut head = d.clone();
    head.setup.as_mut().unwrap().status = FilingChoice::Head;
    supported(&head);
}

#[test]
fn f4c_a_banned_american_opportunity_student_is_figured_with_the_lifetime_learning_credit() {
    // Single, wages $40,000, the filer is the student with $4,000 of tuition.
    // 2025 reckless disregard bans the American opportunity credit through 2027;
    // section 25A(b)(4) does not name the lifetime learning credit: 20% x $4,000.
    let build = |claim| {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 4_000_000, 400_000);
        set_education(
            d.credits.as_mut().unwrap(),
            vec![{
                let mut pair = native_education(
                    -1,
                    education_expenses(400_000),
                    EducationChoice::AmericanOpportunity,
                );
                pair.0.id = 0;
                pair.0.half_time = true;
                pair.0.graduate = false;
                pair.1.more.american_opportunity_used_up = Some(false);
                pair
            }],
        );
        d.credits.as_mut().unwrap().after_disallowance = Some(claim);
        d
    };
    let claim = |year, finding| CreditsAfterDisallowance {
        earned_income: EicDisallowance::Never,
        child_credits: CreditDisallowance::Never,
        education_credit: CreditDisallowance::Disallowed(DisallowanceFacts {
            year,
            finding,
            reinstated: false,
        }),
    };
    let banned = supported(&build(claim(2025, DisallowanceFinding::RecklessDisregard)));
    assert_eq!(value(&banned, "education_credit"), 800);
    assert!(noted(&banned, "NOTE_8862_AOTC_BANNED"));
    assert!(noted(&banned, "NOTE_8862_AOTC_BANNED_LIFETIME_LEARNING_USED"));
    assert!(banned.lines.iter().all(|l| !l.id.starts_with("form_8862_")));
    // An ended ban (2022 finding) only needs the form: the full credit,
    // $2,000 + 25% x $2,000 = $2,500.
    let open = supported(&build(claim(2022, DisallowanceFinding::Ineligible)));
    assert_eq!(value(&open, "education_credit"), 1_500);
    assert_eq!(value(&open, "refundable_education_credit"), 1_000);
    assert!(open.lines.iter().any(|l| l.id == "form_8862_line_2_aotc"));
}

// ---------------------------------------------------------------------------
// A child who died in 2026
// ---------------------------------------------------------------------------

#[test]
fn f6_a_child_who_died_counts_over_the_days_alive_and_prints_the_date_on_form_8862() {
    // Head of household, wages $30,000. A child born 2019-03-02 who died
    // 2026-05-20 (140 days alive) lived with the filer every day alive.
    //   Taxable $30,000 - $24,150 = $5,850; row midpoint $5,875; 10% = $587.5
    //   -> $588. Child tax credit $2,200: nonrefundable $588, additional
    //   min($1,612, $1,700, 15% x $27,500) = $1,612. Earned income credit, one
    //   child, $30,000 row of the draft table: $3,447.
    let mut d = complete(FederalStatus::Head);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19900410;
    d.income[0] = wage(1, Owner::Taxpayer, 3_000_000, 0);
    add_child(&mut d);
    let id = d.dependents[0].id;
    d.dependents[0].birth_date = 20190302;
    d.dependents[0].death_date = Some(20260520);
    d.dependents[0].residency = DependentResidency::WholeYearException;
    d.credits.as_mut().unwrap().after_disallowance = Some(CreditsAfterDisallowance {
        earned_income: EicDisallowance::Disallowed {
            facts: DisallowanceFacts {
                year: 2023,
                finding: DisallowanceFinding::Ineligible,
                reinstated: false,
            },
            cause: EicDisallowanceCause::Other,
            your_us_home_days: None,
            spouse_us_home_days: None,
        },
        child_credits: CreditDisallowance::Never,
        education_credit: CreditDisallowance::Never,
    });
    let r = supported(&d);
    assert_eq!(return_line(&r, "16"), 588 * 100);
    assert_eq!(return_line(&r, "27a"), 3_447 * 100);
    assert_eq!(return_line(&r, "28"), 1_612 * 100);
    assert_eq!(value(&r, "form_8862_line_5a"), 1);
    assert_eq!(value(&r, "form_8862_line_7_child_1"), 365);
    assert_eq!(value(&r, "form_8862_line_8_child_1_death"), 520);
    assert!(!r.lines.iter().any(|l| l.id.ends_with("_birth")));
    assert_eq!(printed_line(&r, "form_8862_line_8_child_1_death"), (TaxForm::Form8862, "8".into()));
    let _ = id;
}

#[test]
fn a_part_year_child_who_died_is_tested_over_the_days_alive() {
    // The child died on 2026-05-20 (140 days alive) and lived with the filer 100
    // days: 100 x 2 > 140, so the child qualifies although 100 days is under
    // half of a year. Without the date it is 100 x 2 < 365 and does not.
    let build = |death_date| {
        let mut d = complete(FederalStatus::Single);
        add_child(&mut d);
        d.dependents[0].birth_date = 20190302;
        d.dependents[0].death_date = death_date;
        d.dependents[0].residency = DependentResidency::Days { days: 100 };
        d
    };
    // Not a qualifying child, the person is still a qualifying relative (the
    // $500 credit for other dependents); a qualifying child under 17 is $2,200.
    let alive = supported(&build(None));
    assert_eq!(value(&alive, "child_credit"), 500);
    let died = supported(&build(Some(20260520)));
    assert_eq!(value(&died, "child_credit"), 2_200);
}

#[test]
fn f6b_a_child_born_and_died_prints_both_dates_and_needs_the_date_of_death() {
    let mut d = complete(FederalStatus::Head);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19900410;
    d.income[0] = wage(1, Owner::Taxpayer, 3_000_000, 0);
    add_child(&mut d);
    let c = &mut d.dependents[0];
    c.birth_date = 20260210;
    c.death_date = Some(20260401);
    c.residency = DependentResidency::WholeYearException;
    c.valid_ssn = false;
    c.valid_tin = false;
    d.credits.as_mut().unwrap().after_disallowance = Some(CreditsAfterDisallowance {
        earned_income: EicDisallowance::Disallowed {
            facts: DisallowanceFacts {
                year: 2023,
                finding: DisallowanceFinding::Ineligible,
                reinstated: false,
            },
            cause: EicDisallowanceCause::Other,
            your_us_home_days: None,
            spouse_us_home_days: None,
        },
        child_credits: CreditDisallowance::Never,
        education_credit: CreditDisallowance::Never,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "form_8862_line_8_child_1_birth"), 210);
    assert_eq!(value(&r, "form_8862_line_8_child_1_death"), 401);
    assert_eq!(return_line(&r, "27a"), 3_447 * 100);
    // The page asks no separate fact: the 2026 birth, the date of death and
    // no Social Security number decide it; the date is not before the birth.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let loaded = s.draft.clone();
    let born_and_died = Dependent {
        identification: Identification::NoneByDueDate,
        ..row("Eli", "Sample", 20260210)
    };
    rejected_with(
        send(
            &mut s,
            save_children(
                &loaded,
                vec![Dependent { death_date: Some(20260101), ..born_and_died.clone() }],
            ),
        ),
        "DEATH_BEFORE_BIRTH",
    );
    accepted(&send(&mut s, save_children(&loaded, vec![born_and_died.clone()])));
    assert!(!s.draft.dependents()[0].born_and_died());
    let living = s.draft.clone();
    accepted(&send(
        &mut s,
        save_children(&living, vec![Dependent { death_date: Some(20260401), ..born_and_died }]),
    ));
    let stored = &s.draft.dependents()[0];
    assert!(stored.born_and_died());
    assert_eq!(stored.death_date, Some(20260401));
    assert_eq!(stored.residency, DependentResidency::WholeYearException);
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
}

// ---------------------------------------------------------------------------
// Form 8839 Part III for joint employees
// ---------------------------------------------------------------------------

fn code_t_wage(d: &mut FederalDraft, entry: usize, wages: i64, code_t: i64) {
    set_joint_adoption_w2(d, entry, wages, code_t);
    let FederalIncome::Wage { data } = &mut d.income[entry].income else { unreachable!() };
    let mut data = edit_wage(data);
    data.other_box12.push(W2CodeAmount { code: W2Code::T, amount: code_t });
}

fn joint_child(
    d: &FederalDraft,
    label: &str,
    special: bool,
    wages: [i64; 2],
    history: AdoptionBenefitHistory,
    received: [i64; 2],
) -> AdoptionChild {
    let mut child = adoption_child(
        AdoptionKind::Domestic {
            finalized: AdoptionFinalization::Final { year: 2026 },
            special_needs: special,
        },
        &[],
    );
    child.label = label.into();
    child.benefits = AdoptionBenefits::JointEmployees {
        payments: vec![
            AdoptionEmployerPayment {
                owner: Owner::Taxpayer,
                wage: wages[0],
                received: received[0],
            },
            AdoptionEmployerPayment { owner: Owner::Spouse, wage: wages[1], received: received[1] },
        ],
        allocation: JointAdoptionExclusion::Automatic,
        history,
        ordinary: true,
    };
    let _ = d;
    child
}

#[test]
fn f7_joint_employees_with_an_earlier_exclusion_split_in_proportion_to_code_t() {
    // Taxpayer W-2 box 1 $80,000 with code T $10,000, spouse $60,000 with code
    // T $9,000, for one domestic child final in 2026; $5,000 was excluded for
    // the child on the 2025 Form 8839 (line 20).
    //   Line 19 $17,670; 20 $5,000; 21 $12,670; 22 $19,000; 23 $19,000;
    //   24 min($12,670, $19,000) = $12,670; MAGI $140,000 is under $265,080 so
    //   28 = 0; 29 = $12,670; 30 = $12,670; 31 = $19,000 - $12,670 = $6,330.
    //   Split by code T: taxpayer $12,670 x 10/19 = $6,668.4 -> $6,668, spouse
    //   $6,002; taxable $3,332 and $2,998.
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 8_000_000, 800_000);
    add(&mut d, wage(2, Owner::Spouse, 6_000_000, 600_000));
    code_t_wage(&mut d, 0, 8_000_000, 1_000_000);
    code_t_wage(&mut d, 1, 6_000_000, 900_000);
    let child = joint_child(
        &d,
        "Synthetic child",
        false,
        [w2(&d, 1), w2(&d, 2)],
        AdoptionBenefitHistory::Exclusion { used: 500_000 },
        [1_000_000, 900_000],
    );
    d.credits.as_mut().unwrap().adoptions = vec![child];
    let r = supported(&d);
    // Form 8839 line 25 MAGI counts all the code T: $140,000 + $19,000.
    assert_eq!(value(&r, "adoption_exclusion_magi"), 159_000);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 12_670);
    assert_eq!(value(&r, "adoption_wage_adjustment"), 6_330);
    assert_eq!(value(&r, "adoption_employer_excluded_taxpayer"), 6_668);
    assert_eq!(value(&r, "adoption_employer_excluded_spouse"), 6_002);
    assert_eq!(value(&r, "adoption_employer_taxable_taxpayer"), 3_332);
    assert_eq!(value(&r, "adoption_employer_taxable_spouse"), 2_998);
    assert_eq!(return_line(&r, "1a"), 14_000_000);
    assert_eq!(return_line(&r, "1f"), 633_000);
    assert_eq!(value(&r, "agi"), 146_330);
    // Every dollar of code T is accounted for by the child: no W-2 warning.
    assert!(!noted(&r, "WARN_W2_CODE_T_TAXABLE_NO_ADOPTION_RECORD"));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn f7b_three_joint_employee_children_each_have_their_own_part_iii_column() {
    // Three children, each paid $6,000 (taxpayer) and $4,000 (spouse) of code
    // T by employers of their own: each line 24 is min($17,670, $10,000) = $10,000
    // and all of it is excluded; line 23 and line 30 sum to $30,000.
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 8_000_000, 800_000);
    add(&mut d, wage(2, Owner::Spouse, 6_000_000, 600_000));
    add(&mut d, wage(3, Owner::Taxpayer, 1_000_000, 100_000));
    add(&mut d, wage(4, Owner::Spouse, 1_000_000, 100_000));
    add(&mut d, wage(5, Owner::Taxpayer, 1_000_000, 100_000));
    add(&mut d, wage(6, Owner::Spouse, 1_000_000, 100_000));
    for (index, wages) in
        [8_000_000, 6_000_000, 1_000_000, 1_000_000, 1_000_000, 1_000_000].into_iter().enumerate()
    {
        code_t_wage(&mut d, index, wages, if index % 2 == 0 { 600_000 } else { 400_000 });
    }
    d.credits.as_mut().unwrap().adoptions = (0..3)
        .map(|n| {
            joint_child(
                &d,
                &alloc::format!("Child {}", n + 1),
                false,
                [w2(&d, 1 + 2 * n), w2(&d, 2 + 2 * n)],
                AdoptionBenefitHistory::None,
                [600_000, 400_000],
            )
        })
        .collect();
    let r = supported(&d);
    for n in 1..=3 {
        assert_eq!(value(&r, &alloc::format!("adoption_child_{n}_benefits_excluded")), 10_000);
        assert_eq!(value(&r, &alloc::format!("adoption_child_{n}_benefits_taxable")), 0);
    }
    assert_eq!(value(&r, "adoption_benefits_excluded"), 30_000);
    assert_eq!(value(&r, "adoption_wage_adjustment"), 0);
}

#[test]
fn f7c_a_special_needs_child_excludes_the_whole_limit_whatever_the_code_t() {
    // Final in 2026, code T $12,000 + $8,000 = $20,000, no history: line 21
    // $17,670; line 24 = line 21 (special needs); line 29 $17,670; line 31 =
    // $20,000 - $17,670 = $2,330 taxable, split 12/20 and 8/20 of the exclusion:
    // taxpayer $10,602 excluded ($1,398 taxable), spouse $7,068 ($932).
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 8_000_000, 800_000);
    add(&mut d, wage(2, Owner::Spouse, 6_000_000, 600_000));
    code_t_wage(&mut d, 0, 8_000_000, 1_200_000);
    code_t_wage(&mut d, 1, 6_000_000, 800_000);
    let child = joint_child(
        &d,
        "Special",
        true,
        [w2(&d, 1), w2(&d, 2)],
        AdoptionBenefitHistory::None,
        [1_200_000, 800_000],
    );
    d.credits.as_mut().unwrap().adoptions = vec![child];
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 17_670);
    assert_eq!(value(&r, "adoption_wage_adjustment"), 2_330);
    assert_eq!(value(&r, "adoption_employer_taxable_taxpayer"), 1_398);
    assert_eq!(value(&r, "adoption_employer_taxable_spouse"), 932);
    // A special-needs child with less code T than the limit still excludes all
    // of it and the excess reduces line 1f (the existing deemed-excess path).
    let mut small = d.clone();
    code_t_wage(&mut small, 0, 8_000_000, 0);
    let AdoptionBenefits::JointEmployees { payments, .. } =
        &mut small.credits.as_mut().unwrap().adoptions[0].benefits
    else {
        unreachable!()
    };
    payments[0].received = 500_000;
    payments[1].received = 400_000;
    for (entry, code_t) in [(0, 500_000), (1, 400_000)] {
        let FederalIncome::Wage { data } = &mut small.income[entry].income else { unreachable!() };
        let mut data = edit_wage(data);
        data.other_box12.clear();
        data.other_box12.push(W2CodeAmount { code: W2Code::T, amount: code_t });
        let wages = data.wages;
        drop(data);
        set_joint_adoption_w2(&mut small, entry, wages, code_t);
    }
    let r = supported(&small);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 17_670);
    assert_eq!(value(&r, "adoption_joint_special_needs_deemed_excess"), 8_670);
}

#[test]
fn a_foreign_child_paid_to_both_spouses_is_still_refused() {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 8_000_000, 800_000);
    add(&mut d, wage(2, Owner::Spouse, 6_000_000, 600_000));
    code_t_wage(&mut d, 0, 8_000_000, 1_000_000);
    code_t_wage(&mut d, 1, 6_000_000, 900_000);
    let mut child = joint_child(
        &d,
        "Foreign",
        false,
        [w2(&d, 1), w2(&d, 2)],
        AdoptionBenefitHistory::None,
        [1_000_000, 900_000],
    );
    child.kind = AdoptionKind::Foreign { finalized: AdoptionFinalization::Final { year: 2026 } };
    d.credits.as_mut().unwrap().adoptions = vec![child];
    unsupported(&d, "Adoption benefits");
}

// ---------------------------------------------------------------------------
// W-2 box 12 code T
// ---------------------------------------------------------------------------

#[test]
fn w2_code_t_with_no_adoption_record_is_taxable_wages_with_a_warning() {
    // Wages $50,000 with code T $6,000 not entered under an adoption: the
    // $6,000 is taxable on Form 1040 line 1f.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 5_000_000, 500_000);
    code_t_wage(&mut d, 0, 5_000_000, 600_000);
    let r = supported(&d);
    assert_eq!(return_line(&r, "1f"), 600_000);
    assert_eq!(value(&r, "agi"), 56_000);
    assert_eq!(value(&r, "adoption_w2_1_code_t_taxable"), 6_000);
    assert!(noted(&r, "WARN_W2_CODE_T_TAXABLE_NO_ADOPTION_RECORD"));
}

#[test]
fn w2_code_t_matches_an_employee_program_and_a_larger_record_is_refused() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 5_000_000, 500_000);
    code_t_wage(&mut d, 0, 5_000_000, 600_000);
    employer_adoption(&mut d, 6_000, 0, false);
    let r = supported(&d);
    // The whole $6,000 is excluded: nothing is taxable on line 1f.
    assert_eq!(value(&r, "adoption_benefits_excluded"), 6_000);
    assert_eq!(return_line(&r, "1f"), 0);
    assert!(!noted(&r, "WARN_W2_CODE_T_TAXABLE_NO_ADOPTION_RECORD"));
    // A partly matched W-2: the unmatched $2,000 is taxable.
    code_t_wage(&mut d, 0, 5_000_000, 200_000);
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_w2_1_code_t_taxable"), 2_000);
    assert_eq!(return_line(&r, "1f"), 200_000);
    assert!(noted(&r, "WARN_W2_CODE_T_TAXABLE_NO_ADOPTION_RECORD"));
    // A record above the W-2's code T is refused.
    let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
    let mut data = edit_wage(data);
    data.other_box12.clear();
    data.other_box12.push(W2CodeAmount { code: W2Code::T, amount: 300_000 });
    drop(data);
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_ADOPTION_CODE_T_ABOVE_W_2"), "{r:?}");
}

// ---------------------------------------------------------------------------
// Form 8615
// ---------------------------------------------------------------------------

#[test]
fn f8_a_dependent_with_a_mixed_source_business_is_calculated() {
    // Single, born 2009-06-01 (17), can be claimed, a parent alive; Schedule C
    // net profit $10,000, both services and capital with work worth at least
    // 30%; interest $5,000. The parent: joint, taxable income $150,000.
    //   Self-employment tax 15.3% x $10,000 x 92.35% = $1,413 (rounded), half
    //   $707 (Schedule 1 line 15). AGI = $5,000 + $10,000 - $707 = $14,293.
    //   Dependent standard deduction: earned income $10,000 - $707 = $9,293
    //   plus $450 = $9,743; $14,293 - $9,743 = $4,550 before the qualified
    //   business income deduction, which is the smaller of 20% x ($10,000 -
    //   $707) = $1,859 and 20% x $4,550 = $910: taxable income $3,640.
    //   Form 8615 earned income: 30% x ($10,000 - $707) = $2,788; line 1 =
    //   gross income $15,000 - $2,788 = $12,212 (the worksheet starts from
    //   gross income less the savings penalty, not from AGI, so half of the
    //   self-employment tax is not subtracted); 2 $2,700; 3 $9,512; 4 $3,640 (Form 1040
    //   line 15); 5 $3,640; the parent's rate on $3,640 at 22% (joint $150,000
    //   + $3,640 is inside the 22% band to $211,400): $800.80 -> $801.
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.birth_date = 20090601;
    d.income.clear();
    add(&mut d, income(savings_income(5_000, 0)));
    let mut business = cash_business(10_000);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    data.income_source = BusinessIncomeSource::MixedThirtyPercent;
    add(&mut d, income(business));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 14_293);
    assert_eq!(return_line(&r, "12e"), 9_743 * 100);
    assert_eq!(return_line(&r, "15"), 3_640 * 100);
    let id = d.income[1].id;
    assert_eq!(value(&r, &alloc::format!("business_{id}_kiddie_earned_income")), 2_788);
    assert_eq!(value(&r, "kiddie_tax_unearned_income"), 12_212);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 3_640);
    assert_eq!(return_line(&r, "16"), 801 * 100);
    // Capital alone: no earned income from the business, so more of the income
    // is unearned; the dependent standard deduction still counts the full profit.
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    data.income_source = BusinessIncomeSource::Capital;
    let r = supported(&d);
    assert_eq!(return_line(&r, "12e"), 9_743 * 100);
    assert_eq!(value(&r, "kiddie_tax_unearned_income"), 14_293);
}

fn parent_form_2555() -> FederalDraft {
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.birth_date = 20110615;
    d.income[0].income = savings_income(12_700, 0);
    let parent = d.people.as_mut().unwrap().parent_tax.as_mut().unwrap();
    parent.status = FederalStatus::Joint;
    parent.taxable = 6_000_000;
    d
}

#[test]
fn f9_the_parents_form_2555_exclusion_is_added_to_line_6_and_line_10() {
    // Filer 15, interest $12,700, no earned income; the parent's joint taxable
    // income $60,000 and Form 2555 exclusion $50,000.
    //   Line 1 $12,700; 2 $2,700; 3 $10,000; 4 = $12,700 - $1,350 = $11,350;
    //   5 $10,000; 6 = $60,000 + $50,000 = $110,000; 7 $10,000; 8 $120,000;
    //   9 = tax on $120,000 = $2,480 + 12% x $76,000 + 22% x $19,200 = $15,824;
    //   10 = tax on $110,000 = $11,600 + 22% x $9,200 = $13,624; 11 = $2,200;
    //   the child's other $1,350 at the child's rate.
    // Without the exclusion line 6 is $60,000 and the parent's rate 12%: $1,200.
    let mut d = parent_form_2555();
    let plain = supported(&d);
    assert_eq!(value(&plain, "kiddie_tax_parent_share"), 1_200);
    d.people.as_mut().unwrap().parent_tax.as_mut().unwrap().foreign_excluded = Some(5_000_000);
    let r = supported(&d);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 10_000);
    assert_eq!(value(&r, "kiddie_tax_parent_share"), 2_200);
    assert!(return_line(&r, "16") > return_line(&plain, "16"));
    // Zero is no exclusion.
    d.people.as_mut().unwrap().parent_tax.as_mut().unwrap().foreign_excluded = Some(0);
    assert_eq!(return_line(&supported(&d), "16"), return_line(&plain, "16"));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn f9b_a_form_4952_election_comes_off_the_preferential_income() {
    // The parent's qualified dividends $10,000 with a Form 4952 line 4g election
    // of $4,000: line 3 of the line 9 worksheet is $4,000, so only $6,000 is
    // taxed at the zero or fifteen percent rate and the rest at ordinary rates.
    let build = |election| {
        let mut d = young_interest_return();
        d.people.as_mut().unwrap().taxpayer.birth_date = 20110615;
        d.income[0].income = savings_income(12_700, 0);
        let parent = d.people.as_mut().unwrap().parent_tax.as_mut().unwrap();
        parent.status = FederalStatus::Joint;
        parent.taxable = 9_000_000;
        parent.gains = Some(ParentGains {
            investment_election: election,
            collectibles: 0,
            unrecaptured: 0,
            qualified: 1_000_000,
            long_term: 0,
            combined: 0,
        });
        d
    };
    let without = supported(&build(0));
    let with = supported(&build(400_000));
    // The election cannot lower the tax; it moves dividends into the ordinary
    // brackets of the combined stack.
    assert!(
        with.refund.unwrap_or(0) <= without.refund.unwrap_or(0)
            || with.balance.unwrap_or(0) >= without.balance.unwrap_or(0)
    );
    let mut over = build(1_100_000);
    assert!(!over.people.as_ref().unwrap().parent_tax.as_ref().unwrap().valid());
    over.people
        .as_mut()
        .unwrap()
        .parent_tax
        .as_mut()
        .unwrap()
        .gains
        .as_mut()
        .unwrap()
        .investment_election = 0;
    assert!(over.people.as_ref().unwrap().parent_tax.as_ref().unwrap().valid());
}

#[test]
fn schedule_j_on_the_parents_return_stays_refused() {
    let mut d = parent_form_2555();
    d.people.as_mut().unwrap().parent_tax.as_mut().unwrap().ordinary = false;
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
    assert!(r.issues.iter().any(|i| i.display.key
        == "ISSUE_FORM_8615_LINES_9_AND_15_WITH_SCHEDULE_J_ON_THE_PARENT_S_RETURN"));
}

// ---------------------------------------------------------------------------
// Coverdell prohibited transaction (1099-Q box 7 code 6)
// ---------------------------------------------------------------------------

#[test]
fn a_prohibited_coverdell_transaction_is_a_withdrawal_with_no_expenses_and_the_additional_tax() {
    // Box 1 $10,000 (the account's value on January 1), box 2 earnings $3,000,
    // box 3 basis $7,000: the earnings are taxable on Schedule 1 line 8z and the
    // 10% additional tax is $300 (section 530(d)(4), no exception).
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        crate::tests::family::education_accounts::q1099_form(
            Owner::Taxpayer,
            10_000,
            crate::tests::family::education_accounts::coverdell(3_000, 7_000),
            Q1099Transfer::ProhibitedTransaction,
            FederalEducationBeneficiary::Recipient,
        ),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "education_account_taxable_earnings"), 3_000);
    assert_eq!(value(&r, "account_additional_tax"), 300);
    assert_eq!(value(&r, "agi"), 63_000);
}

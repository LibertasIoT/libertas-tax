// Fixtures of the Form 1040 and Schedules 1 to 3 line audit (unit fix-f5, 2026-10-10). Each
// fixture is the audit's, with its hand-worked expected numbers. Amounts are cents.
use super::*;

// A W-2 with box 1 wages and box 12 deferrals; the cents are whole dollars times 100.
fn deferral_wage(id: i64, owner: Owner, wages: i64, codes: &[(W2Code, i64)]) -> FederalIncomeEntry {
    let mut e = wage(id, owner, wages, 0);
    let FederalIncome::Wage { data } = &mut e.income else { unreachable!() };
    data.retirement_plan = true;
    data.other_box12 = codes.iter().map(|&(code, amount)| W2CodeAmount { code, amount }).collect();
    e
}

#[test]
fn d1_excess_elective_deferrals_reach_line_1h_with_a_notice() {
    // Single, 46, two W-2s of $60,000 each with code D $15,000: deferrals 30,000 against the
    // $24,500 limit leave 5,500 on line 1h, 1z = 120,000 + 5,500, and AGI the same.
    let mut d = empty_return(FederalStatus::Single);
    for id in 1..=2 {
        add(&mut d, deferral_wage(id, Owner::Taxpayer, 6_000_000, &[(W2Code::D, 1_500_000)]));
    }
    let r = supported(&d);
    assert_eq!(return_line(&r, "1h"), 550_000);
    assert_eq!(return_line(&r, "1z"), 12_550_000);
    assert_eq!(return_line(&r, "9"), 12_550_000);
    assert_eq!(return_line(&r, "11a / 11b"), 12_550_000);
    assert!(noted(&r, "NOTE_EXCESS_ELECTIVE_DEFERRALS"));
    assert!(noted(&r, "NOTE_EXCESS_ELECTIVE_DEFERRALS_SEVERAL_W2S"));
    // At the limit nothing is added and nothing is said.
    let mut at_limit = empty_return(FederalStatus::Single);
    add(&mut at_limit, deferral_wage(1, Owner::Taxpayer, 6_000_000, &[(W2Code::D, 2_450_000)]));
    let r = supported(&at_limit);
    assert_eq!(return_line(&r, "1z"), 6_000_000);
    assert!(!noted(&r, "NOTE_EXCESS_ELECTIVE_DEFERRALS"));
}

#[test]
fn d1_catch_up_roth_share_and_simple_limits() {
    // Born 1970 (56 at the end of 2026): the $8,000 catch-up joins the limit, 32,500 in all.
    let mut d = empty_return(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19700301;
    add(&mut d, deferral_wage(1, Owner::Taxpayer, 6_000_000, &[(W2Code::D, 3_400_000)]));
    let r = supported(&d);
    assert_eq!(return_line(&r, "1h"), 150_000);
    // The same deferrals, half of them designated Roth, put half the excess on line 1h.
    let mut roth = empty_return(FederalStatus::Single);
    add(
        &mut roth,
        deferral_wage(
            1,
            Owner::Taxpayer,
            6_000_000,
            &[(W2Code::D, 1_500_000), (W2Code::AA, 1_500_000)],
        ),
    );
    let r = supported(&roth);
    assert_eq!(return_line(&r, "1h"), 275_000);
    // Only SIMPLE deferrals: the $17,000 limit.
    let mut simple = empty_return(FederalStatus::Single);
    add(&mut simple, deferral_wage(1, Owner::Taxpayer, 6_000_000, &[(W2Code::S, 1_900_000)]));
    let r = supported(&simple);
    assert_eq!(return_line(&r, "1h"), 200_000);
    assert!(!noted(&r, "NOTE_EXCESS_ELECTIVE_DEFERRALS_SEVERAL_W2S"));
}

// A performing artist's W-2s (box 1 $9,000 with code L $300, and $6,500) and Form 2106 with
// line 10 = $1,814 (travel $600, other $1,400, meals $200; the form's own allocation).
fn artist_with_benefits(benefits: i64) -> FederalDraft {
    let mut d = empty_return(FederalStatus::Single);
    for (id, amount, code_l) in [(1, 900_000, 30_000), (2, 650_000, 0)] {
        let mut e = wage(id, Owner::Taxpayer, amount, 0);
        if code_l > 0
            && let FederalIncome::Wage { data } = &mut e.income
        {
            data.other_box12.push(W2CodeAmount { code: W2Code::L, amount: code_l });
        }
        d.income.push(e);
    }
    d.next_id = 3;
    add(&mut d, income(social_security_net(benefits, 0)));
    d.adjustments.as_mut().unwrap().employee_expenses = vec![EmployeeExpenseClaim {
        dot_hours_of_service: false,
        leased_vehicles: vec![],
        owner: Owner::Taxpayer,
        occupation: "Stage actor".into(),
        capacity: EmployeeCapacity::PerformingArtist,
        employers: vec![1, 2],
        local_transportation: 0,
        travel: 60_000,
        other: 140_000,
        meals: 20_000,
        reimbursements: 0,
        mileage: vec![],
        vehicle_assets: vec![],
    }];
    d
}

#[test]
fn d2_the_social_security_worksheet_subtracts_the_performing_artist_deduction() {
    // Before line 12 the provisional income is 15,500 + 10,000 = 25,500 (taxable benefits 250,
    // AGI 15,750 within the artist's $16,000), so the artist qualifies. The worksheet then
    // subtracts line 12: 25,500 - 1,814 = 23,686, under the $25,000 base, so 6b is 0.
    let r = supported(&artist_with_benefits(2_000_000));
    assert_eq!(value(&r, "schedule_1_employee_business_expenses"), 1_814);
    assert_eq!(return_line(&r, "6b"), 0);
    assert_eq!(return_line(&r, "9"), 1_550_000);
    assert_eq!(return_line(&r, "11a / 11b"), 1_368_600);
}

#[test]
fn d4_a_business_loss_reduces_the_dependents_earned_income() {
    // Single, claimable as a dependent: W-2 $5,000, Schedule C net loss $3,000, interest $4,000.
    // AGI 6,000. Earned income 5,000 - 3,000 = 2,000, so the worksheet's line 4a is
    // 2,000 + 450 = 2,450 (not 5,000 + 450); taxable income 3,550, Tax Table row 3,550-3,600
    // midpoint 3,575 at 10% = 358.
    let mut d = empty_return(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    add(&mut d, wage(1, Owner::Taxpayer, 500_000, 0));
    add(&mut d, income(cash_business(-3_000)));
    add(&mut d, income(savings_income(4_000, 0)));
    let r = supported(&d);
    assert_eq!(return_line(&r, "11a / 11b"), 600_000);
    assert_eq!(return_line(&r, "12e"), 245_000);
    assert_eq!(return_line(&r, "15"), 355_000);
    assert_eq!(return_line(&r, "16"), 35_800);
}

#[test]
fn d5_educator_expenses_are_reduced_by_the_form_8815_exclusion() {
    // Single, wages $60,000, Series EE interest $3,000 excluded on Form 8815 (the audit's
    // fixture, line 14 = 3,000), educator expenses $350: 350 - 3,000 leaves nothing, so
    // Schedule 1 line 11 is 0, AGI 60,000 and line 15 = 60,000 - 16,100 = 43,900.
    let mut d =
        crate::tests::investment::investment::bond_return(FederalStatus::Single, 60_000, 3_000);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19800501;
    let child = d.dependents[0].id;
    add(
        &mut d,
        crate::tests::investment::investment::bond_record(
            Owner::Taxpayer,
            12_000,
            3_000,
            20100601,
            vec![crate::tests::investment::investment::bond_student(child, 14_000, 0)],
        ),
    );
    d.adjustments.as_mut().unwrap().set_educator_taxpayer(35_000);
    let r = supported(&d);
    assert_eq!(value(&r, "savings_bond_exclusion"), 3_000);
    assert!(!r.lines.iter().any(|l| l.id == "schedule_1_educator_expenses"));
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(return_line(&r, "15"), 4_390_000);
    // An exclusion smaller than the expenses leaves the rest: interest $200 excluded.
    let mut small =
        crate::tests::investment::investment::bond_return(FederalStatus::Single, 60_000, 200);
    small.people.as_mut().unwrap().taxpayer.birth_date = 19800501;
    let child = small.dependents[0].id;
    add(
        &mut small,
        crate::tests::investment::investment::bond_record(
            Owner::Taxpayer,
            12_000,
            200,
            20100601,
            vec![crate::tests::investment::investment::bond_student(child, 14_000, 0)],
        ),
    );
    small.adjustments.as_mut().unwrap().set_educator_taxpayer(35_000);
    let r = supported(&small);
    assert_eq!(value(&r, "savings_bond_exclusion"), 200);
    assert_eq!(value(&r, "schedule_1_educator_expenses"), 150);
}

// Joint, one child of the given birth date, wages in dollars, and the child's Form 8814 election.
fn joint_8814(wages: i64, birth_date: u32, statements: Vec<ChildIncome>) -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, wages * 100, 0)];
    let id = d.next_id;
    add_child(&mut d);
    d.dependents[0].birth_date = birth_date;
    add(&mut d, child_election(id, statements, 0, OtherParentChoice::NotNeeded));
    d
}

#[test]
fn d8_the_form_8814_tax_is_in_total_tax_and_the_balance() {
    // The i8814 Fred example: line 16 = 6,443 + 135 = 6,578, credits 2,200, line 22 = 4,378, so
    // 24a = 24c = 4,378 and, with no withholding, the balance due is 4,378.
    let d = joint_8814(90_000, 20200615, vec![child_dividends(2_190, 2_190, 610)]);
    let r = supported(&d);
    assert_eq!(return_line(&r, "16"), 657_800);
    assert_eq!(return_line(&r, "19"), 220_000);
    assert_eq!(return_line(&r, "22"), 437_800);
    assert_eq!(return_line(&r, "23"), 0);
    assert_eq!(return_line(&r, "24a"), 437_800);
    assert_eq!(return_line(&r, "37"), 437_800);
}

#[test]
fn d9_credits_are_limited_by_line_18_with_the_form_8814_tax() {
    // Joint, wages $30,000, a child of 8 whose interest $2,700 is on Form 8814: the tax is
    // 10% x min(1,350, 2,700 - 1,350) = 135 and taxable income is 0, so line 16 = line 18 = 135.
    // The child tax credit of 2,200 first removes the 135 (line 19), then the refundable part
    // 1,700 and the earned income credit 4,427 give 6,127.
    let d = joint_8814(30_000, 20180615, vec![child_interest(2_700)]);
    let r = supported(&d);
    assert_eq!(return_line(&r, "16"), 13_500);
    assert_eq!(return_line(&r, "18"), 13_500);
    assert_eq!(return_line(&r, "19"), 13_500);
    assert_eq!(return_line(&r, "22"), 0);
    assert_eq!(return_line(&r, "28"), 170_000);
    assert_eq!(return_line(&r, "27a"), 442_700);
    assert_eq!(return_line(&r, "34"), 612_700);
    // The second fixture: wages $40,000 and the dividends of the D8 fixture: line 19 = 918, the
    // refundable child credit 1,282 and a refund of 4,276.
    let d = joint_8814(40_000, 20200615, vec![child_dividends(2_190, 2_190, 610)]);
    let r = supported(&d);
    assert_eq!(return_line(&r, "19"), 91_800);
    assert_eq!(return_line(&r, "28"), 128_200);
    assert_eq!(return_line(&r, "34"), 427_600);
}

// A Schedule 1 line as the review shows it, in whole dollars (0 when the line is not shown).
pub(in crate::tests) fn schedule_1_line(r: &FederalResult, line: &str) -> i64 {
    crate::draft_return(r)
        .iter()
        .filter(|row| row.form_line == Some(FormLine::new(TaxForm::Schedule1, line)))
        .map(|row| row.amount)
        .sum::<i64>()
        / 100
}

// ---- Corpus-wide invariants, checked on every supported result of the suite (`supported`) ----

// One schedule's shown lines by printed line, in whole cents; the sum of the lines whose
// printed line `pick` accepts.
fn shown(
    r: &FederalResult,
    form: TaxForm,
) -> alloc::collections::BTreeMap<alloc::string::String, i64> {
    let mut lines = alloc::collections::BTreeMap::new();
    for row in crate::draft_return(r) {
        if let Some(f) = row.form_line
            && f.form == form
            && f.kind == crate::FormLineKind::Line
        {
            *lines.entry(f.line).or_insert(0) += row.amount;
        }
    }
    lines
}

fn sum_of(
    lines: &alloc::collections::BTreeMap<alloc::string::String, i64>,
    pick: impl Fn(&str) -> bool,
) -> i64 {
    lines.iter().filter(|(line, _)| pick(line)).map(|(_, amount)| amount).sum()
}

fn number(line: &str) -> u32 {
    line.trim_end_matches(|c: char| c.is_ascii_lowercase())
        .parse()
        .unwrap_or_else(|_| panic!("unknown schedule line {line}"))
}

// Schedules 2 and 3 are shown as the forms print them: each subtotal is the sum of the lines it
// names, and lines 3 and 21 of Schedule 2 and lines 8 and 15 of Schedule 3 are Form 1040 lines 17,
// 23, 20 and 31.
pub(in crate::tests) fn assert_schedules_2_and_3_foot(r: &FederalResult) {
    let two = shown(r, TaxForm::Schedule2);
    let at = |lines: &alloc::collections::BTreeMap<alloc::string::String, i64>, line: &str| {
        lines.get(line).copied().unwrap_or(0)
    };
    let part = |line: &str| number(line);
    let check = |name: &str, shown: i64, parts: i64| {
        assert_eq!(shown, parts, "Schedule 2/3 {name}: {two:?}");
    };
    check("1z", at(&two, "1z"), sum_of(&two, |l| part(l) == 1 && l != "1z"));
    check("3", at(&two, "3"), at(&two, "1z") + at(&two, "2"));
    check("3 against line 17", at(&two, "3"), return_line(r, "17"));
    check("14", at(&two, "14"), sum_of(&two, |l| part(l) == 13));
    check("15", at(&two, "15"), sum_of(&two, |l| (4..=11).contains(&part(l))) + at(&two, "14"));
    check("16c", at(&two, "16c"), at(&two, "16a") + at(&two, "16b"));
    check("17d", at(&two, "17d"), at(&two, "17a") + at(&two, "17b") + at(&two, "17c"));
    check(
        "20",
        at(&two, "20"),
        at(&two, "16c") + at(&two, "17d") + at(&two, "18") + at(&two, "19c"),
    );
    check("21", at(&two, "21"), at(&two, "15") + at(&two, "20"));
    check("21 against line 23", at(&two, "21"), return_line(r, "23"));
    let three = shown(r, TaxForm::Schedule3);
    check("7", at(&three, "7"), sum_of(&three, |l| part(l) == 6));
    check(
        "8",
        at(&three, "8"),
        at(&three, "1")
            + at(&three, "2")
            + at(&three, "3")
            + at(&three, "4")
            + at(&three, "5a")
            + at(&three, "7"),
    );
    check("8 against line 20", at(&three, "8"), return_line(r, "20"));
    check("14", at(&three, "14"), sum_of(&three, |l| part(l) == 13));
    check(
        "15",
        at(&three, "15"),
        sum_of(&three, |l| (9..=12).contains(&part(l))) + at(&three, "14"),
    );
    check("15 against line 31", at(&three, "15"), return_line(r, "31"));
}

// Form 1040 line 24a is line 22 plus line 23, so a Form 8814 child's tax (line 16, box 1) is
// in total tax on every return that carries one.
pub(in crate::tests) fn assert_total_tax_adds(r: &FederalResult) {
    assert_eq!(
        return_line(r, "24a"),
        return_line(r, "22") + return_line(r, "23"),
        "line 24a against lines 22 and 23"
    );
}

// Every credit limit starts from the one regular-tax base: Form 1040 line 18 less the alternative
// minimum tax (F9, MTC-D1 and FTC-D1). The Form 1116 line 20, Form 8801 line 22, Form 3800, Form
// 8834 and Form 8911 bases, each with the credits it subtracts added back, equal that figure on
// every fixture, so the Form 8814 tax and any later line 16 addition reach all of them.
pub(in crate::tests) fn assert_credit_bases_are_income_tax(_r: &FederalResult) {
    // The calculation records the bases of its last run, with line 18 less the AMT of the same run.
    crate::federal_rules::CREDIT_BASES.with(|bases| {
        let bases = bases.borrow();
        let line_18 = bases
            .iter()
            .find(|(name, _)| name.starts_with("Form 1040 line 18"))
            .expect("the run's line 18")
            .1;
        for (name, base) in bases.iter() {
            assert_eq!(*base, line_18, "{name} against line 18 less the AMT");
        }
    });
}

// The Schedule 1 review rows add up the way the form does: lines 1 through 7 and 9 give line
// 10, lines 8a through 8z give line 9, lines 11 through 23 and 25 give line 26 and lines 24a
// through 24z give line 25. A line left out of the rows shows as a difference.
pub(in crate::tests) fn assert_schedule_1_foots(r: &FederalResult) {
    let rows = crate::draft_return(r);
    let mut part_one = 0;
    let mut part_two = 0;
    let (mut line_8, mut line_24) = (0, 0);
    let (mut shown_9, mut shown_10, mut shown_25, mut shown_26) = (0, 0, 0, 0);
    for row in &rows {
        let Some(f) = &row.form_line else { continue };
        if f.form != TaxForm::Schedule1 {
            continue;
        }
        match f.line.as_str() {
            "9" => {
                shown_9 += row.amount;
                part_one += row.amount;
            }
            "10" => shown_10 += row.amount,
            "25" => {
                shown_25 += row.amount;
                part_two += row.amount;
            }
            "26" => shown_26 += row.amount,
            line if line.starts_with('8') => line_8 += row.amount,
            line if line.starts_with("24") => line_24 += row.amount,
            line => {
                let number: u32 = line
                    .trim_end_matches(|c: char| c.is_ascii_lowercase())
                    .parse()
                    .unwrap_or_else(|_| panic!("unknown Schedule 1 line {line}"));
                if number < 8 {
                    part_one += row.amount;
                } else {
                    part_two += row.amount;
                }
            }
        }
    }
    let lines: Vec<_> = rows
        .iter()
        .filter_map(|row| row.form_line.as_ref().map(|f| (f, row.amount)))
        .filter(|(f, _)| f.form == TaxForm::Schedule1)
        .map(|(f, a)| (f.line.clone(), a))
        .collect();
    assert_eq!(line_8, shown_9, "lines 8a-8z against line 9: {lines:?}");
    assert_eq!(part_one, shown_10, "lines 1-7 and 9 against line 10: {lines:?}");
    assert_eq!(line_24, shown_25, "lines 24a-24z against line 25: {lines:?}");
    assert_eq!(part_two, shown_26, "lines 11-23 and 25 against line 26: {lines:?}");
}

// Form 8582 Parts IV and V foot the way the form does (F14, merge review D3): each part's
// column totals (a), (b) and (c) give its line (1d, 2d), the two give line 3, and, with a net
// loss, line 10 is the income on lines 1a and 2a. A row that enters a column twice (a
// publicly traded partnership's prior loss in both (a) and (c)) shows as a difference.
pub(in crate::tests) fn assert_form_8582_foots(r: &FederalResult) {
    let at = |id: &str| r.lines.iter().find(|l| l.id == id).map(|l| l.amount);
    let Some(line_3) = at("form8582_line_3") else {
        return;
    };
    let get = |id: &str| at(id).unwrap_or(0);
    let columns = |first: &str, total: &str| {
        let sum: i64 =
            ["a", "b", "c"].iter().map(|c| get(&alloc::format!("form8582_line_{first}{c}"))).sum();
        if let Some(shown) = at(&alloc::format!("form8582_line_{total}")) {
            assert_eq!(shown, sum, "Form 8582 line {total} against its columns");
        }
    };
    columns("1", "1d");
    columns("2", "2d");
    assert_eq!(
        line_3,
        get("form8582_line_1d") + get("form8582_line_2d"),
        "Form 8582 line 3 against lines 1d and 2d"
    );
    if let Some(line_10) = at("form8582_line_10") {
        assert_eq!(
            line_10,
            get("form8582_line_1a") + get("form8582_line_2a"),
            "Form 8582 line 10 against the income on lines 1a and 2a"
        );
    }
}

// Form 8958 lists each item of income once (F14, merge review D2): on a return made only of
// the documents below, the income this return reports in the rows (`this_return`) is Form 1040
// line 9, so a Form 1099 that became business income is not also a row of its own.
pub(in crate::tests) fn assert_form_8958_foots(d: &FederalDraft, r: &FederalResult) {
    let rows: i64 = r
        .lines
        .iter()
        .filter(|l| l.id.starts_with("form8958_") && l.id.ends_with("_this_return"))
        .filter(|l| !l.id.starts_with("form8958_withholding_"))
        .map(|l| l.amount)
        .sum();
    let plain = d.income.iter().all(|e| {
        matches!(
            e.income,
            FederalIncome::Wage { .. }
                | FederalIncome::Business { .. }
                | FederalIncome::Nec1099 { .. }
                | FederalIncome::Misc1099 { .. }
                | FederalIncome::K1099 { .. }
        )
    });
    // Tax credit bond interest is income the credits page holds, not a row of the form.
    let bonds = d.credits.as_ref().is_some_and(|credits| !credits.tax_credit_bonds.is_empty());
    if rows == 0 || !plain || bonds {
        return;
    }
    assert_eq!(rows, return_line(r, "9"), "Form 8958 income rows against Form 1040 line 9");
}

// Independently calculate the other separate return from the same whole
// accepted sources. Each source's two actual reported columns must conserve
// its total, including SE amounts that belong only to the actual partner.
pub(in crate::tests) fn assert_form_8958_divided_sources_conserve(
    d: &FederalDraft,
    r: &FederalResult,
) {
    let mut mirror = d.clone();
    for entry in &mut mirror.income {
        entry.owner = if entry.owner == Owner::Taxpayer { Owner::Spouse } else { Owner::Taxpayer };
    }
    let mirror = estimate(&mirror);
    assert_eq!(mirror.state, ResultState::EstimatedForSupportedScope, "{:?}", mirror.issues);
    for total in r
        .lines
        .iter()
        .filter(|line| line.id.starts_with("form8958_") && line.id.ends_with("_total"))
    {
        let this = alloc::format!("{}_this_return", total.id.strip_suffix("_total").unwrap());
        let own = r.lines.iter().find(|line| line.id == this).unwrap();
        let other = mirror.lines.iter().find(|line| line.id == this).unwrap();
        let same_total = mirror.lines.iter().find(|line| line.id == total.id).unwrap();
        assert_eq!(total.amount, same_total.amount, "{} has one whole source", total.id);
        assert_eq!(own.amount + other.amount, total.amount, "{this}: columns B+C=A");
    }
}

#[test]
fn d3_schedule_1_part_one_shows_every_computed_line() {
    // Single, W-2 $40,000, unemployment $5,000: line 7 = line 10 = 5,000 and no other line is
    // shown (line 9 is zero).
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(1, Owner::Taxpayer, 4_000_000, 0));
    add(&mut d, government_unemployment(5_000, &[], 0));
    let r = supported(&d);
    assert_eq!(schedule_1_line(&r, "7"), 5_000);
    assert_eq!(schedule_1_line(&r, "9"), 0);
    assert_eq!(schedule_1_line(&r, "10"), 5_000);
    // Lines 8a through 8z add to line 9: a net operating loss deduction is a negative 8a,
    // canceled debt is 8c and the sum is line 9 (the footing check runs on this result too).
    assert_eq!(return_line(&r, "8"), 500_000);
}

#[test]
fn d10_education_credit_recapture_is_on_line_16_inside_the_credit_base() {
    // Single, W-2 $20,000, saver's contributions $2,000 (rate 50%, potential credit 1,000) and
    // an American opportunity credit recapture of 250: taxable income 3,900 and table tax 393;
    // line 16 = 393 + 250 = 643 = line 18. The saver's credit is limited by 643, so lines 22, 23
    // and 24a are zero.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2_000_000, 0);
    d.credits.as_mut().unwrap().saver = Some(saver(200_000, 0)).into();
    d.credits.as_mut().unwrap().education_recapture = Some(PriorYearEducationCreditRecapture {
        method: EducationMethod::AmericanOpportunity,
        filed_qualified_expenses: 700_000,
        after_filing_reduction: 400_000,
        filed_education_magi: 5_000_000,
        filed_joint: false,
        filed_credit_used: 250_000,
        ordinary_prior_return: true,
    });
    let r = supported(&d);
    assert_eq!(return_line(&r, "15"), 390_000);
    assert_eq!(value(&r, "education_credit_recapture_tax"), 250);
    assert_eq!(return_line(&r, "16"), 64_300);
    assert_eq!(return_line(&r, "18"), 64_300);
    assert_eq!(value(&r, "saver_credit"), 643);
    assert_eq!(return_line(&r, "20"), 64_300);
    assert_eq!(return_line(&r, "22"), 0);
    assert_eq!(return_line(&r, "23"), 0);
    assert_eq!(return_line(&r, "24a"), 0);
}

#[test]
fn d11_additional_medicare_tax_is_the_sum_of_the_rounded_schedule_2_lines() {
    // Single, W-2 Medicare wages $200,050 and Schedule C profit $1,137 (net earnings from
    // self-employment 1,050): line 17b = round(0.009 x 50) = 0, line 11 = round(0.009 x 1,050)
    // = 9, so line 23 includes 9 (not 9.45 + 0.45 = 9.9 rounded to 10).
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(1, Owner::Taxpayer, 20_005_000, 0));
    add(&mut d, income(cash_business(1_137)));
    let r = supported(&d);
    let lines = crate::draft_return(&r);
    let schedule_2 = |line: &str| -> i64 {
        lines
            .iter()
            .filter(|row| row.form_line == Some(FormLine::new(TaxForm::Schedule2, line)))
            .map(|row| row.amount)
            .sum::<i64>()
            / 100
    };
    assert_eq!(schedule_2("17b"), 0);
    assert_eq!(schedule_2("11"), 9);
    assert_eq!(value(&r, "additional_medicare_tax"), 9);
    // Schedule 2 shows its subtotals: 4 (30) and 11 (9) make 15 and 21, the Form 1040 line 23.
    assert_eq!(schedule_2("4"), 30);
    assert_eq!(schedule_2("15"), 39);
    assert_eq!(schedule_2("17d"), 0);
    assert_eq!(schedule_2("21"), 39);
    assert_eq!(return_line(&r, "23"), 3_000 + 900);
}

// IRS2026 draft Form8959: wages/RRTA and SE have separate Schedule2 destinations.
// All24 computed lines must survive projection, including withholding in cents.
pub(in crate::tests) fn assert_form_8959_foots(r: &FederalResult) {
    let rows: alloc::collections::BTreeMap<_, _> = r
        .lines
        .iter()
        .filter_map(|row| {
            row.form_line
                .as_ref()
                .filter(|form| form.form == TaxForm::Form8959)
                .map(|form| (form.line.clone(), row.amount))
        })
        .collect();
    if rows.is_empty() {
        return;
    }
    assert_eq!(rows.len(), 24, "complete Form8959 computed-line projection");
    for number in 1u32..=24 {
        assert!(rows.contains_key(&alloc::format!("{number}")), "Form8959 {number}");
    }
    let displayed = shown(r, TaxForm::Form8959);
    for (number, amount) in &rows {
        assert_eq!(
            displayed.get(number).copied().unwrap_or(0),
            *amount,
            "Form8959 {number} projection (zero money rows are withheld)"
        );
    }
    let at = |id: &str| rows[id] / 100;
    assert_eq!(at("4"), at("1") + at("2") + at("3"), "Form 8959 wage-source footing");
    assert_eq!(at("6"), (at("4") - at("5")).max(0), "Form 8959 wage threshold");
    assert_eq!(at("7"), crate::kernel::ratio::fraction(at("6"), 9, 1000), "Form 8959 wage tax");
    assert_eq!(at("10"), (at("8") - at("9")).max(0), "Form 8959 separate RRTA threshold");
    assert_eq!(at("11"), crate::kernel::ratio::fraction(at("10"), 9, 1000), "Form 8959 RRTA tax");
    assert_eq!(at("12"), at("7") + at("11"), "Form 8959 wages and RRTA tax");
    assert_eq!(at("15"), at("4"));
    assert_eq!(at("16"), (at("14") - at("15")).max(0));
    assert_eq!(at("17"), (at("13") - at("16")).max(0));
    assert_eq!(at("18"), crate::kernel::ratio::fraction(at("17"), 9, 1000));
    let schedule2 = shown(r, TaxForm::Schedule2);
    assert_eq!(
        rows["12"],
        schedule2.get("17b").copied().unwrap_or(0),
        "Form8959 payroll destination"
    );
    assert_eq!(rows["18"], schedule2.get("11").copied().unwrap_or(0), "Form8959 SE destination");
    // D-2026-10-19-12: lines 19 and 20 round their source totals once; lines 21 and 22 are
    // figured from those printed whole-dollar lines, and line 24 adds the printed lines.
    let cents = |id: &str| rows[id];
    assert!(["19", "20", "21", "22"].iter().all(|id| cents(id) % 100 == 0), "whole dollars");
    assert_eq!(
        cents("21"),
        crate::kernel::ratio::dollars(crate::kernel::ratio::fraction(cents("20"), 145, 10000))
            * 100
    );
    assert_eq!(cents("22"), (cents("19") - cents("21")).max(0));
    assert_eq!(at("24"), at("22") + at("23"));
}

// IRS2026 draft Form6251 PartII: each comparison uses its actual owner value.
// Check the displayed return, including completeness: absent rows cannot pass
// merely because their fallback zero happens to satisfy an equation.
pub(in crate::tests) fn assert_form_6251_comparison_foots(r: &FederalResult) {
    let rows = shown(r, TaxForm::Form6251);
    let amt = value(r, "alternative_minimum_tax") * 100;
    if amt == 0 {
        assert!(!rows.contains_key("11"), "no positive-AMT filing condition");
        return;
    }
    let calculated: alloc::collections::BTreeMap<_, _> = r
        .lines
        .iter()
        .filter_map(|row| {
            row.form_line
                .as_ref()
                .filter(|form| form.form == TaxForm::Form6251)
                .map(|form| (form.line.clone(), row.amount))
        })
        .collect();
    let at = |id: &str| *calculated.get(id).expect("complete Form6251 comparison");
    for id in ["4", "5", "6", "7", "8", "9", "10", "11"] {
        assert_eq!(at(id) % 100, 0, "Form6251 {id} whole-dollar owner");
        assert_eq!(
            rows.get(id).copied().unwrap_or(0),
            at(id),
            "Form6251 {id} projection (zero money rows are withheld)"
        );
    }
    assert_eq!(at("6"), (at("4") - at("5")).max(0));
    assert_eq!(at("9"), (at("7") - at("8")).max(0));
    assert_eq!(at("11"), (at("9") - at("10")).max(0));
    assert_eq!(at("11"), amt);
    assert_eq!(at("4"), value(r, "alternative_minimum_taxable_income") * 100);
}

#[test]
fn print_amt_comparison_uses_actual_iso_income_and_does_not_create_zero_amt_form() {
    //2026 single: wages200000 plus ISO spread100000 produces AMTI300000,
    // exemption90100 and taxable209900. At26% line7=54574. Regular tax
    // on183900 is36734, so AMT17840. No rates or financial oracle changed.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(1, Owner::Taxpayer, 20_000_000, 0));
    let ordinary = supported(&d);
    assert_eq!(value(&ordinary, "alternative_minimum_tax"), 0);
    assert!(!shown(&ordinary, TaxForm::Form6251).contains_key("5"));
    d.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Synthetic print ISO", 1000.0, 10000, 20000)];
    let r = supported(&d);
    let rows = shown(&r, TaxForm::Form6251);
    for (id, dollars) in [
        ("4", 300000),
        ("5", 90100),
        ("6", 209900),
        ("7", 54574),
        ("8", 0),
        ("9", 54574),
        ("10", 36734),
        ("11", 17840),
    ] {
        assert_eq!(rows.get(id).copied().unwrap_or(0), dollars * 100, "Form6251 {id}");
    }
    assert_eq!(value(&r, "regular_income_tax"), 36734);
    assert_eq!(value(&r, "alternative_minimum_tax"), 17840);
}

// Form8801 footings are per actual retained return, before the shared current
// credit limit. Never net two histories' printed line numbers before checking
// the statutory signed generation and each gain worksheet's rounded tax lines.
pub(in crate::tests) fn assert_form_8801_foots(r: &FederalResult) {
    let mut histories: alloc::collections::BTreeMap<usize, alloc::collections::BTreeMap<u32, i64>> =
        alloc::collections::BTreeMap::new();
    let mut projected: alloc::collections::BTreeMap<alloc::string::String, i64> =
        alloc::collections::BTreeMap::new();
    for row in &r.lines {
        if let Some(form) = &row.form_line
            && form.form == TaxForm::Form8801
        {
            *projected.entry(form.line.clone()).or_default() += row.amount;
        }
        let Some((index, number)) =
            row.id.strip_prefix("minimum_tax_history_").and_then(|rest| rest.split_once("_line_"))
        else {
            continue;
        };
        let Ok(index) = index.parse::<usize>() else {
            continue;
        };
        let number = number.parse::<u32>().expect("numeric Form8801 line");
        assert_eq!(row.kind, LineKind::Money);
        assert_eq!(row.amount % 100, 0, "Form8801 whole-dollar owner");
        assert_eq!(
            row.form_line,
            Some(FormLine::new(TaxForm::Form8801, &alloc::format!("{number}")))
        );
        assert!(
            histories.entry(index).or_default().insert(number, row.amount / 100).is_none(),
            "duplicate Form8801 source line"
        );
    }
    if histories.is_empty() {
        return;
    }
    let shown = shown(r, TaxForm::Form8801);
    for (number, amount) in projected {
        assert_eq!(
            shown.get(&number).copied().unwrap_or(0),
            amount,
            "Form8801 {number} retains every source copy"
        );
    }
    let mut available = 0;
    for (index, rows) in histories {
        let at = |number| rows.get(&number).copied().unwrap_or(0);
        for number in [1, 2, 3, 4, 15, 16, 17, 18, 19, 20, 21] {
            assert!(rows.contains_key(&number), "Form8801 source{index} line{number}");
        }
        assert!(at(3) <= 0);
        assert!(at(4) >= 0);
        if at(4) > 0 {
            for number in 5..=10 {
                assert!(rows.contains_key(&number), "Form8801 source{index} line{number}");
            }
            assert_eq!(at(7), (at(4) - at(6)).max(0));
            assert_eq!(at(9), (at(5) - at(8)).max(0));
            assert_eq!(at(10), (at(4) - at(9)).max(0));
        }
        if at(10) > 0 {
            for number in 11..=14 {
                assert!(rows.contains_key(&number), "Form8801 source{index} line{number}");
            }
            assert_eq!(at(13), at(11) - at(12));
            assert_eq!(at(15), (at(13) - at(14)).max(0));
        } else {
            assert_eq!(at(15), 0, "Form8801 exclusion-tax stop");
        }
        assert_eq!(at(17), at(15));
        assert_eq!(at(18), at(16) - at(17), "Form8801 signed credit generation");
        assert_eq!(at(21), at(18) + at(19) + at(20));
        available += at(21).max(0);
        if rows.contains_key(&27) {
            for number in [
                27, 28, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 53,
                54, 55,
            ] {
                assert!(rows.contains_key(&number), "Form8801 gain source{index} line{number}");
            }
            assert_eq!(at(31), at(27).min(at(30)));
            assert_eq!(at(32), at(27) - at(31));
            assert_eq!(at(36), (at(34) - at(35)).max(0));
            assert_eq!(at(37), at(27).min(at(28)));
            assert_eq!(at(38), at(36).min(at(37)));
            assert_eq!(at(39), at(37) - at(38));
            assert_eq!(at(41), at(36));
            assert_eq!(at(43), at(41) + at(42));
            assert_eq!(at(44), (at(40) - at(43)).max(0));
            assert_eq!(at(45), at(39).min(at(44)));
            assert_eq!(at(46), crate::kernel::ratio::fraction(at(45), 15, 100));
            assert_eq!(at(47), at(38) + at(45));
            if at(47) != at(27) {
                assert!(rows.contains_key(&48) && rows.contains_key(&49));
                assert_eq!(at(48), at(37) - at(47));
                assert_eq!(at(49), crate::kernel::ratio::fraction(at(48), 20, 100));
                if at(29) != 0 {
                    for number in 50..=52 {
                        assert!(rows.contains_key(&number));
                    }
                    assert_eq!(at(50), at(32) + at(47) + at(48));
                    assert_eq!(at(51), at(27) - at(50));
                    assert_eq!(at(52), crate::kernel::ratio::fraction(at(51), 25, 100));
                }
            }
            assert_eq!(at(53), at(33) + at(46) + at(49) + at(52));
            assert_eq!(at(55), at(53).min(at(54)));
        }
    }
    assert_eq!(value(r, "minimum_tax_credit_available"), available);
    let used = value(r, "minimum_tax_credit");
    let carry = value(r, "minimum_tax_credit_carryforward");
    assert_eq!(used + carry, available, "Form8801 current credit conservation");
    if available > 0 {
        let regular = value(r, "minimum_tax_current_regular_liability");
        let tentative = value(r, "minimum_tax_current_tentative");
        assert_eq!(value(r, "minimum_tax_credit_limit"), (regular - tentative).max(0));
        assert_eq!(used, available.min((regular - tentative).max(0)));
    }
}

// Form5329 PartsIII/IV have one copy per filer. Verify the missing13/21
// subtotal and retained carry relationships before checking displayed totals.
pub(in crate::tests) fn assert_form_5329_ira_foots(r: &FederalResult) {
    let mut expected: alloc::collections::BTreeMap<alloc::string::String, i64> =
        alloc::collections::BTreeMap::new();
    for who in ["taxpayer", "spouse"] {
        let prefix = alloc::format!("ira_excess_{who}_line");
        let rows: alloc::collections::BTreeMap<u32, _> = r
            .lines
            .iter()
            .filter_map(|row| {
                let (number, _) = row.id.strip_prefix(&prefix)?.split_once('_')?;
                Some((number.parse().expect("Form5329 IRA line"), row))
            })
            .collect();
        if rows.is_empty() {
            continue;
        }
        assert_eq!(rows.len(), 17, "Form5329 complete IRA computation, {who}");
        let at = |number| rows[&number].amount / 100;
        assert_eq!(at(13), at(10) + at(11) + at(12));
        assert_eq!(at(14), (at(9) - at(13)).max(0));
        assert_eq!(at(16), at(14) + at(15));
        assert_eq!(at(21), at(19) + at(20));
        assert_eq!(at(22), (at(18) - at(21)).max(0));
        assert_eq!(at(24), at(22) + at(23));
        for part in [9..=17, 18..=25] {
            let required = rows[part.start()].form_line.is_some();
            for number in part {
                let row = rows[&number];
                assert_eq!(row.kind, LineKind::Money);
                assert_eq!(row.amount % 100, 0);
                assert_eq!(
                    row.form_line,
                    required.then(|| FormLine::new(TaxForm::Form5329, &alloc::format!("{number}"))),
                    "Form5329 {who} line{number}"
                );
                if required {
                    *expected.entry(alloc::format!("{number}")).or_default() += row.amount;
                }
            }
        }
    }
    let shown = shown(r, TaxForm::Form5329);
    for (number, amount) in expected {
        assert_eq!(
            shown.get(&number).copied().unwrap_or(0),
            amount,
            "Form5329 {number} filer-copy projection"
        );
    }
}

// Form5329 PartsVI/VII preserve native source cents through the carry
// projection; each person's already-rounded tax must still reach the total.
pub(in crate::tests) fn assert_form_5329_health_foots(r: &FederalResult) {
    let shown = shown(r, TaxForm::Form5329);
    let mut projected: alloc::collections::BTreeMap<alloc::string::String, i64> =
        alloc::collections::BTreeMap::new();
    for (kind, start, total_id) in [
        ("archer", 34, "archer_return_year_excess_contribution_tax"),
        ("hsa", 42, "hsa_return_year_excess_contribution_tax"),
    ] {
        let mut tax = 0;
        let mut found = false;
        for owner in 1..=2 {
            let prefix = alloc::format!("{kind}_{owner}_excess_line");
            let rows: alloc::collections::BTreeMap<u32, _> = r
                .lines
                .iter()
                .filter_map(|row| {
                    let number = row.id.strip_prefix(&prefix)?.parse().expect("health excess line");
                    Some((number, row))
                })
                .collect();
            if rows.is_empty() {
                continue;
            }
            found = true;
            assert_eq!(rows.len(), 8, "{kind} Form5329 full part");
            let at = |number| rows[&number].amount;
            for number in start..start + 8 {
                let row = rows[&number];
                assert_eq!(row.kind, LineKind::Money);
                assert_eq!(
                    row.form_line,
                    Some(FormLine::new(TaxForm::Form5329, &alloc::format!("{number}")))
                );
                *projected.entry(alloc::format!("{number}")).or_default() += row.amount;
            }
            assert_eq!(at(start + 3), at(start + 1) + at(start + 2));
            assert_eq!(at(start + 4), (at(start) - at(start + 3)).max(0));
            assert_eq!(at(start + 6), at(start + 4) + at(start + 5));
            assert_eq!(at(start + 7) % 100, 0, "whole-dollar owner tax");
            tax += at(start + 7);
        }
        if found {
            assert_eq!(tax, value(r, total_id) * 100, "{kind} per-filer tax conservation");
        }
    }
    for (number, amount) in projected {
        assert_eq!(
            shown.get(&number).copied().unwrap_or(0),
            amount,
            "health Form5329 {number} filer-copy projection"
        );
    }
}

// Form5329 PartIX carries separate corrected/other rates for each filer.
// Waiver requests are supporting subrows, never another tax-line amount.
pub(in crate::tests) fn assert_form_5329_rmd_foots(r: &FederalResult) {
    let mut expected: alloc::collections::BTreeMap<alloc::string::String, i64> =
        alloc::collections::BTreeMap::new();
    let mut total = 0;
    let mut found = false;
    for who in ["taxpayer", "spouse"] {
        let prefix = alloc::format!("{who}_rmd_line");
        let rows: alloc::collections::BTreeMap<_, _> = r
            .lines
            .iter()
            .filter_map(|row| row.id.strip_prefix(&prefix).map(|number| (number, row)))
            .collect();
        if rows.is_empty() {
            continue;
        }
        found = true;
        assert_eq!(rows.len(), 7, "complete RMD form copy");
        let at = |number: &str| rows.get(number).expect("computed RMD line").amount;
        for number in ["52a", "52b", "53a", "53b", "54a", "54b", "55"] {
            let row = rows[number];
            assert_eq!(row.kind, LineKind::Money);
            assert_eq!(row.form_line, Some(FormLine::new(TaxForm::Form5329, number)));
            *expected.entry(number.into()).or_default() += row.amount;
        }
        for (suffix, minimum, distributed, tax, rate) in
            [("a", "52a", "53a", "54a", 10), ("b", "52b", "53b", "54b", 25)]
        {
            let id = alloc::format!("{who}_rmd_waiver54{suffix}");
            let waived = r.lines.iter().find(|row| row.id == id).map_or(0, |row| {
                assert_eq!(
                    row.form_line,
                    Some(FormLine::subrow(TaxForm::Form5329, &alloc::format!("54{suffix}")))
                );
                row.amount
            });
            let difference = at(minimum) - at(distributed);
            assert!(difference >= waived, "waiver is capped by actual shortfall");
            assert_eq!(
                at(tax),
                crate::kernel::ratio::fraction(
                    crate::kernel::ratio::dollars(difference - waived),
                    rate,
                    100
                ) * 100
            );
        }
        assert_eq!(at("55"), at("54a") + at("54b"));
        total += at("55");
    }
    if found {
        assert_eq!(total, value(r, "required_distribution_tax") * 100);
    }
    let shown = shown(r, TaxForm::Form5329);
    for (number, amount) in expected {
        assert_eq!(
            shown.get(&number).copied().unwrap_or(0),
            amount,
            "Form5329 {number}: waiver subrows do not duplicate tax"
        );
    }
}

// Form4952 carries independent regular and AMT computations. Never add their
// repeated printed line numbers together: each system must foot on its own.
// IRS Form4952 (2025 final), lines3,4c,4f,4h,6–8; current draft uses the
// same line relationships. Whole-dollar rounding belongs to the tax owner.
pub(in crate::tests) fn assert_form_4952_foots(r: &FederalResult) {
    for prefix in ["investment_form_line", "amt_investment_form_line"] {
        let rows: alloc::collections::BTreeMap<_, _> = r
            .lines
            .iter()
            .filter_map(|row| {
                row.id.strip_prefix(prefix).map(|number| {
                    assert_eq!(row.kind, LineKind::Money, "{prefix}{number} units");
                    assert_eq!(row.amount % 100, 0, "{prefix}{number} owner rounding");
                    assert_eq!(row.form_line, Some(FormLine::new(TaxForm::Form4952, number)));
                    (number, row.amount)
                })
            })
            .collect();
        if rows.is_empty() {
            continue;
        }
        assert_eq!(rows.len(), 15, "{prefix} complete computed lines");
        let at = |number: &str| *rows.get(number).expect("computed Form4952 line");
        assert_eq!(at("3"), at("1") + at("2"), "{prefix} total expense");
        assert_eq!(at("4c"), at("4a") - at("4b"), "{prefix} ordinary income");
        assert_eq!(at("4f"), at("4d") - at("4e"), "{prefix} ordinary gain");
        assert_eq!(at("4h"), at("4c") + at("4f") + at("4g"), "{prefix} investment income");
        assert_eq!(at("6"), (at("4h") - at("5")).max(0), "{prefix} net income");
        assert_eq!(at("7"), (at("3") - at("6")).max(0), "{prefix} carryforward");
        assert_eq!(at("8"), at("3").min(at("6")), "{prefix} deduction");
        assert_eq!(at("7") + at("8"), at("3"), "{prefix} interest conservation");
    }
}

// ScheduleB printed payer/adjustment subrows must survive projection and add
// to their totals. Text answers/countries contribute no money. IRS ScheduleB
// (2025 final), lines2–4 and6; the 2026 presentation retains this arithmetic.
pub(in crate::tests) fn assert_schedule_b_foots(r: &FederalResult) {
    let rows = crate::draft_return(r);
    let subrows = |number: &str| {
        rows.iter()
            .filter(|row| {
                row.form_line.as_ref().is_some_and(|line| {
                    line.form == TaxForm::ScheduleB
                        && line.kind == crate::FormLineKind::Subrow
                        && line.line == number
                })
            })
            .map(|row| row.money.unwrap_or(0))
            .sum::<i64>()
    };
    let lines = shown(r, TaxForm::ScheduleB);
    let at = |number: &str| lines.get(number).copied().unwrap_or(0);
    let present = |id: &str| r.lines.iter().any(|line| line.id == id);
    if present("schedule_b_total_interest") {
        assert_eq!(subrows("1"), at("2"), "ScheduleB payer interest subtotal");
        assert_eq!(at("4"), at("2") - at("3"), "ScheduleB savings-bond subtraction");
        assert_eq!(at("4"), super::return_line(r, "2b"), "ScheduleB to1040 interest");
    }
    if present("schedule_b_ordinary_dividends") {
        assert_eq!(subrows("5"), at("6"), "ScheduleB payer dividend subtotal");
        assert_eq!(at("6"), super::return_line(r, "3b"), "ScheduleB to1040 dividends");
    }
}

// Annual Roth ordering and special repayments retain source cents. Check the
// form's stop rules as well as conservation to the existing per-entry income.
pub(in crate::tests) fn assert_form_8606_roth_foots(d: &FederalDraft, r: &FederalResult) {
    let mut projected: alloc::collections::BTreeMap<alloc::string::String, i64> =
        alloc::collections::BTreeMap::new();
    let mut copies = vec![
        (Some(Owner::Taxpayer), alloc::string::String::from("taxpayer")),
        (Some(Owner::Spouse), alloc::string::String::from("spouse")),
    ];
    copies.extend(d.income.iter().filter_map(|entry| match &entry.income {
        FederalIncome::InheritedIra { data } if data.roth() => {
            Some((None, alloc::format!("inherited_roth_{}", entry.id)))
        }
        _ => None,
    }));
    for (owner, who) in copies {
        let prefix = if owner.is_some() {
            alloc::format!("{who}_roth_form_line")
        } else {
            alloc::format!("{who}_form_line")
        };
        let rows: alloc::collections::BTreeMap<&str, _> =
            r.lines.iter().filter_map(|row| Some((row.id.strip_prefix(&prefix)?, row))).collect();
        if rows.is_empty() {
            continue;
        }
        let at = |number| rows.get(number).map_or(0, |row| row.amount);
        for number in ["19", "20", "21", "22"] {
            assert!(rows.contains_key(number), "Form8606 {who} line{number}");
        }
        assert!(at("19") > 0, "No form for qualified-only Roth payments");
        assert_eq!(at("21"), at("19") - at("20"));
        assert_eq!(rows.contains_key("23"), at("21") > 0);
        if at("21") > 0 {
            assert_eq!(at("23"), (at("21") - at("22")).max(0));
        }
        for number in ["24", "25a"] {
            assert_eq!(rows.contains_key(number), at("23") > 0);
        }
        if at("23") > 0 {
            assert_eq!(at("25a"), (at("23") - at("24")).max(0));
        }
        for number in ["25b", "25c"] {
            assert_eq!(rows.contains_key(number), at("25a") > 0);
        }
        assert!(at("25b") + at("25c") <= at("25a"));
        let taxable: i64 = if let Some(owner) = owner {
            d
            .income
            .iter()
             .filter(|e| {
                e.owner == owner
                    && matches!(&e.income, FederalIncome::Roth { data } if data.inherited == crate::InheritedLink::No)
            })
            .filter_map(|e| {
                r.lines
                    .iter()
                    .find(|row| row.id == alloc::format!("roth_{}_taxable", e.id))
            })
            .map(|row| row.amount)
            .sum()
        } else {
            line_amount(r, &alloc::format!("{who}_taxable"))
        };
        assert_eq!(at("25c"), taxable, "Form8606 {who} taxable source conservation");
        for (number, row) in rows {
            assert_eq!(row.kind, LineKind::Money);
            assert_eq!(row.form_line, Some(FormLine::new(TaxForm::Form8606, number)));
            *projected.entry(number.into()).or_default() += row.amount;
        }
    }
    let actual = shown(r, TaxForm::Form8606);
    for (number, amount) in projected {
        assert_eq!(
            actual.get(&number).copied().unwrap_or(0),
            amount,
            "Form8606 line{number} filer-copy projection"
        );
    }
}

pub(in crate::tests) fn assert_form_8606_traditional_foots(r: &FederalResult) {
    let mut projected: alloc::collections::BTreeMap<alloc::string::String, i64> =
        alloc::collections::BTreeMap::new();
    let inherited: alloc::collections::BTreeSet<_> = r
        .lines
        .iter()
        .filter_map(|row| {
            let (copy, _) = row.id.split_once("_form_line")?;
            copy.starts_with("inherited_ira_").then(|| alloc::string::String::from(copy))
        })
        .collect();
    let copies = [alloc::string::String::from("taxpayer"), alloc::string::String::from("spouse")]
        .into_iter()
        .chain(inherited);
    for who in copies {
        let inherited = who.starts_with("inherited_ira_");
        let prefix = if inherited {
            alloc::format!("{who}_form_line")
        } else {
            alloc::format!("{who}_ira_form_line")
        };
        let rows: alloc::collections::BTreeMap<&str, _> =
            r.lines.iter().filter_map(|row| Some((row.id.strip_prefix(&prefix)?, row))).collect();
        if rows.is_empty() {
            continue;
        }
        let at = |number| rows.get(number).map_or(0, |row| row.amount);
        let old = |suffix| {
            let suffix = if inherited {
                match suffix {
                    "ira_basis_carryforward" => "basis_remaining",
                    "ira_basis_recovered" => "basis_recovered",
                    "ira_basis_taxable" => "taxable",
                    other => other,
                }
            } else {
                suffix
            };
            r.lines
                .iter()
                .find(|row| row.id == alloc::format!("{who}_{suffix}"))
                .map_or(0, |row| row.amount)
        };
        if rows.contains_key("1") {
            for number in ["2", "3", "14"] {
                assert!(rows.contains_key(number));
            }
            assert_eq!(at("3"), at("1") + at("2"));
            assert_eq!(at("14"), (at("3") - at("13")).max(0));
            assert_eq!(at("14"), old("ira_basis_carryforward"));
            if rows.contains_key("4") {
                for number in ["5", "13", "15a", "15b", "15c"] {
                    assert!(rows.contains_key(number));
                }
                assert_eq!(at("5"), at("3") - at("4"));
                assert_eq!(at("13"), old("ira_basis_recovered"));
                if inherited {
                    // The existing inherited taxable owner includes QCD gross
                    // for its later exclusion; this form excludes that QCD.
                    assert!(at("15a") <= old("ira_basis_taxable"));
                } else {
                    assert_eq!(at("15a") + at("18").max(0), old("ira_basis_taxable"));
                }
                assert_eq!(
                    at("15c"),
                    at("15a") - at("15b") - old("ira_repayment_income_reduction")
                );
                if rows.contains_key("6") {
                    for number in ["7", "8", "9", "10", "11", "12"] {
                        assert!(rows.contains_key(number));
                    }
                    assert_eq!(at("9"), at("6") + at("7") + at("8"));
                    assert_eq!(rows["10"].kind, LineKind::Ratio);
                    let numerator = at("5").min(at("9"));
                    let fraction = |value| {
                        if at("9") == 0 {
                            0
                        } else {
                            crate::kernel::ratio::fraction(value, numerator, at("9"))
                        }
                    };
                    assert_eq!(at("10"), fraction(10_000));
                    assert_eq!(at("11"), fraction(at("8") / 100).min(at("5") / 100) * 100);
                    assert_eq!(
                        at("12"),
                        fraction(at("7") / 100).min((at("5") - at("11")) / 100) * 100
                    );
                    assert_eq!(at("13"), at("11") + at("12"));
                    assert_eq!(at("15a"), at("7") - at("12"));
                } else {
                    for number in ["7", "8", "9", "10", "11", "12"] {
                        assert!(!rows.contains_key(number));
                    }
                }
            } else {
                assert_eq!(rows.len(), 4, "No-distribution PartI stop rule");
                assert_eq!(at("14"), at("3"));
            }
        }
        if rows.contains_key("16") {
            assert!(rows.contains_key("17") && rows.contains_key("18"));
            assert_eq!(at("18"), at("16") - at("17"));
            if rows.contains_key("8") {
                assert_eq!(at("16"), at("8"));
                assert_eq!(at("17"), at("11"));
            }
            if !rows.contains_key("1") {
                assert_eq!(at("18").max(0), old("ira_basis_taxable"));
            }
        }
        for (number, row) in rows {
            assert_eq!(row.form_line, Some(FormLine::new(TaxForm::Form8606, number)));
            if number != "10" {
                assert_eq!(row.kind, LineKind::Money);
                *projected.entry(number.into()).or_default() += row.amount;
            }
        }
    }
    // A current nondeductible contribution is already computed even when
    // no distribution required an entered prior-basis record. Show that known
    // line1 without inventing an opening basis or a complete worksheet.
    for row in &r.lines {
        if row.id.ends_with("_nondeductible_ira")
            && row.form_line.as_ref() == Some(&FormLine::new(TaxForm::Form8606, "1"))
        {
            *projected.entry("1".into()).or_default() += row.amount;
        }
    }
    let actual = shown(r, TaxForm::Form8606);
    for (number, amount) in projected {
        assert_eq!(
            actual.get(&number).copied().unwrap_or(0),
            amount,
            "Form8606 traditional line{number} projection"
        );
    }
}

// 2026 draft8915-F and its instructions: every filer/disaster-year copy
// conserves the same native taxable/repayment owners and source gross.
pub(in crate::tests) fn assert_form_8915_foots(d: &FederalDraft, r: &FederalResult) {
    let mut copies: alloc::collections::BTreeMap<&str, alloc::collections::BTreeMap<&str, i64>> =
        alloc::collections::BTreeMap::new();
    for row in &r.lines {
        if let Some((copy, number)) = row.id.split_once("_print_")
            && copy.starts_with("form_8915_")
        {
            assert_eq!(row.form_line, Some(FormLine::new(TaxForm::Form8915F, number)));
            assert_eq!(
                row.kind,
                if matches!(number, "A" | "B") { LineKind::Year } else { LineKind::Money }
            );
            assert!(copies.entry(copy).or_default().insert(number, row.amount).is_none());
        }
    }
    if copies.is_empty() {
        return;
    }
    // Use the owner's ordinary conservative source projection; a No answer
    // may remove only the after-death part of a1099-R before tax is computed.
    // The independent form arithmetic below must not count that omitted part.
    let working = crate::unknowns::supported(d, &mut Vec::new());
    let d = working.as_ref().unwrap_or(d);
    let mut expected_projection: alloc::collections::BTreeMap<alloc::string::String, i64> =
        alloc::collections::BTreeMap::new();
    for (copy, rows) in &copies {
        let at = |number: &str| rows.get(number).copied().unwrap_or(0);
        let old = |number: &str| {
            r.lines
                .iter()
                .find(|line| line.id == alloc::format!("{copy}_line_{number}"))
                .map_or(0, |line| line.amount)
        };
        let (_, year) = copy.rsplit_once('_').unwrap();
        assert_eq!(at("A"), 202600);
        assert_eq!(at("B"), year.parse::<i64>().unwrap() * 100);
        if rows.contains_key("6") {
            assert_eq!(at("5a"), at("2(a)") + at("3(a)") + at("4(a)"));
            assert_eq!(at("5c(a)"), at("5a") - at("5b"));
            assert_eq!(at("5c(b)"), at("2(b)") + at("3(b)") + at("4(b)"));
            assert_eq!(at("6"), at("5c(b)"));
            assert_eq!(at("6"), old("6"));
            if rows.contains_key("1e") {
                assert!(at("6") <= at("1e"));
                if rows.contains_key("1a") {
                    assert_eq!(at("1c"), at("1a") - at("1b"));
                    assert_eq!(at("1e"), at("1c") + at("1d"));
                } else {
                    assert_eq!(at("1e"), 2_200_000);
                }
            }
            let mut cap_qualified = 0;
            for line in &r.lines {
                let Some(fema) = line.id.strip_prefix(&alloc::format!("{copy}_worksheet_prior_"))
                else {
                    continue;
                };
                assert_eq!(line.form_line, Some(FormLine::subrow(TaxForm::Form8915F, "1")));
                let source =
                    |kind| line_amount(r, &alloc::format!("{copy}_worksheet_{kind}_{fema}"));
                assert_eq!(source("prior") + source("limit"), 2_200_000);
                assert!(source("qualified") <= source("limit"));
                cap_qualified += source("qualified");
            }
            assert_eq!(cap_qualified, at("6"));
        }
        if rows.contains_key("8") {
            assert_eq!(at("8"), at("2(b)"));
            assert_eq!(at("10"), at("8") - at("9"));
            assert_eq!(at("10"), old("10"));
            assert_eq!(at("11"), old("11"));
        }
        if rows.contains_key("13") {
            let terminal = r
                .lines
                .iter()
                .find(|line| line.id == alloc::format!("{copy}_terminal_plan"))
                .map_or(0, |line| line.amount);
            assert_eq!(at("13"), at("11") + at("12") + terminal);
            assert_eq!(at("15"), (at("13") - at("14")).max(0));
            for n in ["13", "14", "15"] {
                assert_eq!(at(n), old(n));
            }
        }
        if rows.contains_key("21") {
            assert_eq!(at("21"), at("18") + at("19") + at("20"));
            assert_eq!(at("21"), old("20"));
            assert_eq!(at("22"), old("22"));
        }
        if rows.contains_key("24") {
            let terminal = r
                .lines
                .iter()
                .find(|line| line.id == alloc::format!("{copy}_terminal_ira"))
                .map_or(0, |line| line.amount);
            assert_eq!(at("24"), at("22") + at("23") + terminal);
            assert_eq!(at("26"), (at("24") - at("25")).max(0));
            for n in ["24", "25", "26"] {
                assert_eq!(at(n), old(n));
            }
        }
        if rows.contains_key("28") {
            assert_eq!(at("30"), at("28") - at("29"));
            assert!(at("31") <= at("30"));
            assert!(at("32") == 0 || at("32") == at("30") - at("31"));
        }
        for (number, value) in rows {
            if !matches!(*number, "A" | "B") {
                *expected_projection.entry((*number).into()).or_default() += value;
            }
        }
    }
    // Compare the actual review projection, excluding cap supporting rows and
    // choices whose Text amounts are deliberately not monetary amounts.
    let actual = shown(r, TaxForm::Form8915F);
    for (number, amount) in expected_projection {
        assert_eq!(
            actual.get(&number).copied().unwrap_or(0),
            amount,
            "Form8915-F {number} copy projection"
        );
    }
    // Actual gross sources occur once, then each owner's forms consume the
    // selected amounts in increasing disaster year. Carry-only copies do not
    // consume an ordinary distribution or another filer's balance.
    for who in ["taxpayer", "spouse"] {
        let owner = if who == "taxpayer" { Owner::Taxpayer } else { Owner::Spouse };
        let mut gross = [0; 3];
        for entry in &d.income {
            if entry.owner != owner {
                continue;
            }
            match &entry.income {
                FederalIncome::Retirement { data } => {
                    gross[usize::from(data.kind.ira())] += data.gross
                }
                FederalIncome::Roth { data } => gross[2] += data.gross,
                _ => {}
            }
        }
        let owner_copies: Vec<_> = copies
            .iter()
            .filter(|(copy, _)| copy.starts_with(&alloc::format!("form_8915_{who}_")))
            .collect();
        for (kind, form_number, disaster_number) in [("ira", "15b", "18"), ("roth", "25b", "19")] {
            let own_id = alloc::format!("{who}_{kind}_form_line{form_number}");
            let inherited_prefix = alloc::format!("inherited_{kind}_");
            let expected: i64 = r
                .lines
                .iter()
                .filter(|line| {
                    if line.id == own_id {
                        return true;
                    }
                    let Some(tail) = line.id.strip_prefix(&inherited_prefix) else {
                        return false;
                    };
                    let Some((id, number)) = tail.split_once("_form_line") else {
                        return false;
                    };
                    number == form_number
                        && id.parse::<i64>().ok().is_some_and(|id| {
                            d.income.iter().any(|entry| entry.id == id && entry.owner == owner)
                        })
                })
                .map(|line| line.amount)
                .sum();
            let projected: i64 = owner_copies
                .iter()
                .map(|(_, rows)| rows.get(disaster_number).copied().unwrap_or(0))
                .sum();
            assert_eq!(
                projected, expected,
                "Form8915-F {who} line{disaster_number} = all own/inherited Form8606 line{form_number}"
            );
        }
        for (i, (copy, rows)) in owner_copies.iter().enumerate() {
            let at = |number: &str| rows.get(number).copied().unwrap_or(0);
            let later = owner_copies[i + 1..].iter().any(|(_, rows)| rows.contains_key("6"));
            if rows.contains_key("6") {
                for (j, n) in ["2", "3", "4"].iter().enumerate() {
                    assert_eq!(
                        at(&alloc::format!("{n}(a)")),
                        gross[j],
                        "{copy}: source gross, category{n}"
                    );
                    gross[j] -= at(&alloc::format!("{n}(b)"));
                }
                assert_eq!(at("7"), if later { 0 } else { gross.iter().sum::<i64>() - at("28") });
            }
            let mut components = [[0; 3]; 3];
            for line in &r.lines {
                let Some(tail) = line.id.strip_prefix(&alloc::format!("{copy}_home_")) else {
                    continue;
                };
                let parts: Vec<_> = tail.split('_').collect();
                let category = ["plan", "ira", "roth"].iter().position(|s| *s == parts[0]).unwrap();
                let component =
                    ["gross", "cost", "repaid"].iter().position(|s| *s == parts[2]).unwrap();
                assert_eq!(
                    line.form_line,
                    Some(FormLine::subrow(TaxForm::Form8915F, ["28", "29", "31"][component]))
                );
                components[category][component] += line.amount;
            }
            for (i, n) in ["28", "29", "31"].iter().enumerate() {
                assert_eq!(components.iter().map(|c| c[i]).sum::<i64>(), at(n));
            }
            for (category, components) in components.iter().enumerate() {
                let [home, cost, repaid] = *components;
                gross[category] -= home;
                if later {
                    gross[category] += home - cost - repaid;
                }
            }
        }
    }
}

// Source-owned ScheduleD summary; whole-dollar netting/carries remain distinct
// from admitted K-1/fund cents and the tax-rate-capped special gain pools.
pub(in crate::tests) fn assert_schedule_d_summary_foots(d: &FederalDraft, r: &FederalResult) {
    let rows: alloc::collections::BTreeMap<_, _> = r
        .lines
        .iter()
        .filter_map(|row| {
            row.id.strip_prefix("schedule_d_line_").map(|number| {
                assert_eq!(row.kind, LineKind::Money);
                assert_eq!(row.form_line, Some(FormLine::new(TaxForm::ScheduleD, number)));
                (number, row.amount)
            })
        })
        .collect();
    if rows.is_empty() {
        return;
    }
    for number in ["5", "6", "7", "12", "13", "14", "15", "16"] {
        assert!(rows.contains_key(number), "ScheduleD line{number}");
    }
    let at = |number: &str| *rows.get(number).expect("computed ScheduleD summary");
    assert!(at("6") <= 0 && at("14") <= 0);
    assert!(at("13") >= 0);
    assert_eq!(at("16"), at("7") + at("15"));
    // Precise supporting subrows retain actual cents beneath the unchanged
    // whole-dollar cells. Verify each cell's rounding, then net the actual
    // source partition once. No tolerance or invented balancing amount.
    let exact_cell = |number: &str| {
        let value: i64 = r
            .lines
            .iter()
            .filter(|row| {
                row.form_line.as_ref().is_some_and(|line| {
                    line.form == TaxForm::ScheduleD
                        && line.kind == crate::FormLineKind::Line
                        && line.line == number
                })
            })
            .map(|row| row.amount)
            .sum();
        let precision: Vec<_> = r
            .lines
            .iter()
            .filter(|row| {
                row.id.ends_with("_unrounded")
                    && row.form_line.as_ref().is_some_and(|line| {
                        line.form == TaxForm::ScheduleD
                            && line.kind == crate::FormLineKind::Subrow
                            && line.line == number
                    })
            })
            .collect();
        assert!(precision.len() <= 1, "one source precision row per ScheduleD cell");
        precision.first().map_or(value, |row| {
            assert_eq!(
                crate::kernel::ratio::dollars(row.amount) * 100,
                value,
                "ScheduleD source cell{number} rounds by its original owner"
            );
            row.amount
        })
    };
    for number in ["1a", "1b", "2", "3", "8a", "8b", "9", "10"] {
        assert_eq!(
            exact_cell(number),
            exact_cell(&alloc::format!("{number}(d)")) - exact_cell(&alloc::format!("{number}(e)"))
                + exact_cell(&alloc::format!("{number}(g)")),
            "ScheduleD source{number} columns(d)-(e)+(g)=(h)"
        );
    }
    for (target, numbers) in [
        ("7", &["1a", "1b", "2", "3", "4", "5", "6"][..]),
        ("15", &["8a", "8b", "9", "10", "11", "12", "13", "14"][..]),
    ] {
        let components: Vec<_> =
            numbers.iter().map(|number| (*number, exact_cell(number))).collect();
        assert_eq!(
            crate::kernel::ratio::dollars(components.iter().map(|(_, amount)| amount).sum()),
            at(target) / 100,
            "ScheduleD line{target} source partition {components:?}"
        );
    }

    let gains = at("15") > 0 && at("16") > 0;
    assert_eq!(rows.contains_key("18"), gains);
    assert_eq!(rows.contains_key("19"), gains);
    assert_eq!(rows.contains_key("21"), at("16") < 0);
    let income = line_amount(r, "capital_gain");
    if at("16") < 0 {
        assert_eq!(at("21"), income);
        assert!(at("21") >= at("16") && at("21") <= 0);
        let limit = if d.setup.as_ref().unwrap().filing_status() == FederalStatus::Separate {
            150000
        } else {
            300000
        };
        assert_eq!(at("21"), at("16").max(-limit));
    } else {
        assert_eq!(at("16"), income);
    }
    let visible = r.lines.iter().any(|row| {
        row.amount != 0
            && row.form_line.as_ref().is_some_and(|line| line.form == TaxForm::ScheduleD)
    });
    let investment = r.lines.iter().any(|row| {
        row.amount != 0 && row.form_line.as_ref().is_some_and(|line| line.form == TaxForm::Form4952)
    });
    let qualified = r
        .return_lines
        .iter()
        .find(|row| {
            row.form_line
                .as_ref()
                .is_some_and(|line| line.form == TaxForm::Form1040 && line.line == "3a")
        })
        .map_or(0, |row| row.amount);
    for (number, answer, applies) in [
        ("17", gains, at("16") > 0),
        (
            "20",
            rows.get("18").copied().unwrap_or(0) == 0
                && rows.get("19").copied().unwrap_or(0) == 0
                && !investment,
            gains,
        ),
        ("22", qualified > 0, !gains),
    ] {
        let id = alloc::format!("schedule_d_choice_{number}_{}", if answer { "yes" } else { "no" });
        let choices: Vec<_> = r
            .lines
            .iter()
            .filter(|row| row.id.starts_with(&alloc::format!("schedule_d_choice_{number}_")))
            .collect();
        assert_eq!(choices.len(), usize::from(visible && applies));
        if visible && applies {
            assert_eq!(choices[0].id, id);
            assert_eq!(choices[0].kind, LineKind::Text);
            assert_eq!(choices[0].amount, 0);
            assert_eq!(choices[0].form_line, Some(FormLine::new(TaxForm::ScheduleD, number)));
        }
    }

    // Check the actual derived review, including preserved sign and source cents.
    let review = crate::draft_return(r);
    for (&number, &amount) in &rows {
        let printed: i64 = review
            .iter()
            .filter(|row| {
                row.form_line.as_ref().is_some_and(|line| {
                    line.form == TaxForm::ScheduleD
                        && line.kind == crate::FormLineKind::Line
                        && line.line == number
                })
            })
            .map(|row| row.amount)
            .sum();
        assert_eq!(printed, amount, "ScheduleD review line{number}");
    }
}

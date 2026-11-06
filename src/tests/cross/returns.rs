//! Full-return fixtures as data (R5). Each `tests/fixtures/returns/<name>.json`
//! file holds one filer's document-level input and its hand-computed expected
//! Form 1040 return lines and result state, at the document level (W-2s,
//! 1099-INTs, 1099-DIVs, 1099-OIDs, the household statuses and dependents
//! page), not the
//! internal `FederalDraft` model. This loader deserializes each fixture, maps
//! it onto a complete accepted `FederalDraft` through the crate's existing
//! test helpers (the same "nothing applies" defaults `complete()` uses for
//! every section the fixture does not mention), runs `estimate()`, and
//! compares every expected line and the result state, collecting every
//! mismatch before failing.
use super::*;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(serde::Deserialize)]
struct ReturnFixture {
    name: String,
    filing_status: String,
    // Head of household/qualifying surviving spouse home-cost test; ignored
    // by every other status. Defaults to `complete()`'s own default so a
    // fixture that does not mention it (every M1 fixture) is unaffected.
    #[serde(default = "default_paid_over_half_home")]
    paid_over_half_home: bool,
    birth_dates: BirthDatesFixture,
    #[serde(default)]
    dependents: Vec<DependentFixture>,
    #[serde(default)]
    w2: Vec<W2Fixture>,
    #[serde(default)]
    int1099: Vec<Int1099Fixture>,
    #[serde(default)]
    div1099: Vec<Div1099Fixture>,
    #[serde(default)]
    oid1099: Vec<Oid1099Fixture>,
    // Forms 2106 (employee_expenses.rs), one per job.
    #[serde(default)]
    form2106: Vec<Form2106Fixture>,
    // Pay from a foreign employer with no W-2 (the Work abroad topic).
    #[serde(default)]
    foreign_pay: Vec<ForeignPayFixture>,
    // Forms 2555, one per person.
    #[serde(default)]
    form2555: Vec<Form2555Fixture>,
    // Education credits (Form 8863): one 1098-T per student.
    #[serde(default)]
    education: Vec<EducationFixture>,
    #[serde(default)]
    estimated_payments: f64,
    #[serde(default)]
    taxable_bond_premium_election: Option<bool>,
    #[serde(default)]
    care: Option<CareFixture>,
    #[serde(default)]
    business: Vec<BusinessFixture>,
    #[serde(default)]
    clergy: Vec<ClergyFixture>,
    expected: ExpectedFixture,
    // Hand computation supporting the expected lines; not used by the loader.
    #[allow(dead_code)]
    computation: String,
    // Count of information-path fields the fixture's facts touch; not used by the loader.
    // Native acceptance limits: COVERAGE.md.
    #[serde(default)]
    #[allow(dead_code)]
    fields_touched: u32,
}
fn default_paid_over_half_home() -> bool {
    true
}

// One row of the "Children and people you support" page.
// Only `first_name`, `last_name` and `birth_date` are required; the rest
// default to a child who lives with you all year, exactly as the page's own
// defaults do, so a fixture only overrides what the facts require.
#[derive(serde::Deserialize)]
struct DependentFixture {
    first_name: String,
    last_name: String,
    birth_date: String,
    #[serde(default = "default_relationship")]
    relationship: String,
    // Months lived with you in the U.S. in 2026; omitted means all year.
    #[serde(default)]
    months_lived_with_you: Option<u32>,
    #[serde(default = "default_situation")]
    situation: String,
    #[serde(default = "default_identification")]
    identification: String,
    // Work-related care paid for this person in 2026; omitted means none.
    #[serde(default)]
    care_expenses: f64,
}
fn default_relationship() -> String {
    "Child".into()
}
fn default_situation() -> String {
    "ChildUnder19".into()
}
fn default_identification() -> String {
    "SocialSecurityNumber".into()
}
fn dependent_relationship(name: &str) -> Relationship {
    match name {
        "Child" => Relationship::Child,
        "Foster" => Relationship::Foster,
        "Grandchild" => Relationship::Grandchild,
        "Sibling" => Relationship::Sibling,
        "NieceNephew" => Relationship::NieceNephew,
        "Parent" => Relationship::Parent,
        "OtherRelative" => Relationship::OtherRelative,
        "Unrelated" => Relationship::Unrelated,
        other => panic!("unknown dependent relationship {other}"),
    }
}
// (valid_ssn, valid_tin), matching interview.rs's Identification mapping.
fn dependent_identification(name: &str) -> (bool, bool) {
    match name {
        "SocialSecurityNumber" => (true, true),
        "Itin" => (false, true),
        "NoneByDueDate" => (false, false),
        other => panic!("unknown dependent identification {other}"),
    }
}
// One page row onto its stored record, matching interview.rs's private
// `federal_dependent` mapping for the situations this loader supports
// (`ChildUnder19`/`Student`/`Disabled`; `Supported` is not needed by any
// current fixture and is left unimplemented here rather than widened).
fn build_dependent(id: i64, person: &DependentFixture) -> FederalDependent {
    let birth_date = parse_date(&person.birth_date);
    let (residency, us_home_over_half_year) = match person.months_lived_with_you {
        None if birth_date / 10000 == 2026 => (DependentResidency::WholeYearException, true),
        None => (DependentResidency::Days { days: 365 }, true),
        Some(months) => (DependentResidency::Days { days: months * 30 }, months >= 7),
    };
    if !["ChildUnder19", "Student", "Disabled"].contains(&person.situation.as_str()) {
        panic!(
            "dependent situation {} is not supported by this loader (only ChildUnder19/Student/Disabled)",
            person.situation
        );
    }
    let (valid_ssn, valid_tin) = dependent_identification(&person.identification);
    let care_expenses = cents(person.care_expenses);
    let care =
        if care_expenses > 0 { DependentCare::Yes { care_expenses } } else { DependentCare::No };
    FederalDependent {
        use_for: PersonUse::TaxDependent,
        id,
        first_name: person.first_name.clone(),
        last_name: person.last_name.clone(),
        birth_date,
        death_date: None,
        relationship: dependent_relationship(&person.relationship),
        residency,
        us_home_over_half_year,
        incapable_self_care: false,
        valid_ssn,
        valid_tin,
        situation: match person.situation.as_str() {
            "Student" => DependentSituation::Student,
            "Disabled" => DependentSituation::Disabled,
            _ => DependentSituation::ChildUnder19,
        },
        facts: PersonFacts::NONE.0,
        care: Some(care),
        competing: CompetingClaim::NoOne,
    }
}

// One Form 1098-T for an American opportunity student: the filer ("Taxpayer")
// or a dependent by position in `dependents`. `living_costs` lets the App
// choose the part of the box 5 grant the student includes in income.
#[derive(serde::Deserialize)]
struct EducationFixture {
    student: String,
    box1_tuition: f64,
    box5_scholarships: f64,
    #[serde(default)]
    living_costs: Option<f64>,
}

#[derive(serde::Deserialize)]
struct BirthDatesFixture {
    taxpayer: String,
    #[serde(default)]
    spouse: Option<String>,
}

#[derive(serde::Deserialize)]
struct W2Fixture {
    #[serde(default)]
    box10: f64,
    #[serde(default)]
    care_in_box_1: f64,
    #[serde(default)]
    care_forfeited: f64,
    #[serde(default = "default_coverage")]
    coverage: String,
    // The employer name: the W-2's label, which a Form 2106 fixture names.
    employer: String,
    box1_wages: f64,
    box2_withheld: f64,
    box3: f64,
    box4: f64,
    box5: f64,
    box6: f64,
    // Which filer this W-2 belongs to; every existing fixture omits it and
    // gets the prior "always taxpayer" behavior. A joint return with a
    // second earner's own W-2 needs "Spouse" so each person's own wage base
    // and per-employer Social Security/Medicare limits are tracked
    // separately (federal_rules.rs Totals is per-owner), instead of folding
    // a second employer's withholding into the taxpayer's own totals and
    // manufacturing a fictitious excess-Social-Security credit.
    #[serde(default = "default_owner")]
    owner: String,
    // Box 12 code L, substantiated employee business expense reimbursements.
    #[serde(default)]
    box12_l: f64,
    // Box 1 pay for work abroad ("Your answers about this W-2").
    #[serde(default)]
    abroad: f64,
}
fn default_coverage() -> String {
    "Ordinary".into()
}

#[derive(serde::Deserialize)]
struct BusinessFixture {
    label: String,
    receipts: f64,
    expenses: f64,
    ministerial: bool,
}
#[derive(serde::Deserialize)]
struct ClergyFixture {
    fair_rental_value: f64,
    utility_allowance: f64,
    utilities_paid: f64,
    form_4361: bool,
    employee_expenses: f64,
}

// Pay from a foreign employer (ForeignPay).
#[derive(serde::Deserialize)]
struct ForeignPayFixture {
    employer: String,
    wages: f64,
    #[serde(default = "default_owner")]
    owner: String,
}
// One Form 2555 (the page's Form2555): the qualifying test, the tax home, the
// exclusion choice and an optional housing claim.
#[derive(serde::Deserialize)]
struct Form2555Fixture {
    #[serde(default)]
    prior_year_history: Option<PriorYearHistoryFixture>,
    #[serde(default)]
    prior_year_pay: f64,
    #[serde(default)]
    multi_year_pay: Vec<MultiYearPayFixture>,
    #[serde(default = "default_owner")]
    owner: String,
    // "Residence" (began, ended) or "Presence" (starts_on, foreign_days).
    qualification: String,
    #[serde(default)]
    began: Option<String>,
    #[serde(default)]
    ended: Option<String>,
    #[serde(default)]
    starts_on: Option<String>,
    #[serde(default)]
    foreign_days: i32,
    tax_home: String,
    exclusion: bool,
    #[serde(default)]
    housing: Option<HousingFixture>,
}
#[derive(serde::Deserialize)]
struct PriorYearHistoryFixture {
    prior_year_days: i32,
    prior_year_excluded: f64,
}
#[derive(serde::Deserialize)]
struct MultiYearPayFixture {
    description: String,
    amount: f64,
    months_total: i32,
    months_in_return_year: i32,
    months_in_prior_year: i32,
}
#[derive(serde::Deserialize)]
struct HousingFixture {
    stays: Vec<StayFixture>,
    expenses: f64,
}
#[derive(serde::Deserialize)]
struct StayFixture {
    location: String,
    starts_on: String,
    ends_on: String,
}
fn housing_location(name: &str) -> ForeignHousingLocation {
    match name {
        "General" => ForeignHousingLocation::General,
        "SingaporeSingapore" => ForeignHousingLocation::SingaporeSingapore,
        "UnitedArabEmiratesDubai" => ForeignHousingLocation::UnitedArabEmiratesDubai,
        "UnitedKingdomLondon" => ForeignHousingLocation::UnitedKingdomLondon,
        other => panic!("unknown housing location {other}"),
    }
}
// One Form 2106 (page-level facts of EmployeeExpenseForm): the chosen W-2s by
// employer name, the five amounts, and the vehicles by method.
#[derive(serde::Deserialize)]
struct Form2106Fixture {
    #[serde(default = "default_owner")]
    owner: String,
    occupation: String,
    capacity: String,
    employers: Vec<String>,
    #[serde(default)]
    local_transportation: f64,
    #[serde(default)]
    travel: f64,
    #[serde(default)]
    other: f64,
    #[serde(default)]
    meals: f64,
    #[serde(default)]
    reimbursements: f64,
    #[serde(default)]
    standard_mileage: Vec<MileageFixture>,
}
#[derive(serde::Deserialize)]
struct MileageFixture {
    first_half: u32,
    second_half: u32,
    total: u32,
    #[serde(default)]
    commuting: u32,
    #[serde(default)]
    parking_tolls: f64,
}
fn default_owner() -> String {
    "Taxpayer".into()
}
fn w2_owner(name: &str) -> Owner {
    match name {
        "Taxpayer" => Owner::Taxpayer,
        "Spouse" => Owner::Spouse,
        other => panic!("unknown W-2 owner {other}"),
    }
}

#[derive(serde::Deserialize)]
struct Int1099Fixture {
    #[allow(dead_code)]
    payer: String,
    box1_interest: f64,
    // Early withdrawal penalty on savings.
    #[serde(default)]
    box2_penalty: f64,
    // Federal income tax withheld.
    #[serde(default)]
    box4_withholding: f64,
    // Total tax-exempt interest, including private-activity bond interest.
    #[serde(default)]
    box8_exempt: f64,
    // Specified private-activity bond interest included in box 8.
    #[serde(default)]
    box9_private_activity: f64,
    // Bond premium, reducing this payer's own taxable interest.
    #[serde(default)]
    box11_bond_premium: f64,
    // Bond premium on a tax-exempt bond, reducing this payer's own
    // tax-exempt interest.
    #[serde(default)]
    box13_tax_exempt_bond_premium: f64,
    // Accrued interest paid to the seller when the bond was purchased.
    #[serde(default)]
    accrued_interest: f64,
}

#[derive(serde::Deserialize)]
struct Div1099Fixture {
    #[allow(dead_code)]
    payer: String,
    ordinary: f64,
    #[serde(default)]
    qualified: f64,
    // Total capital gain distributions (box 2a).
    #[serde(default)]
    capital_distributions: f64,
    // Unrecaptured section 1250 gain included in box 2a (box 2b).
    #[serde(default)]
    unrecaptured_distributions: f64,
    // Collectibles (28%) gain included in box 2a (box 2d).
    #[serde(default)]
    collectible_distributions: f64,
    // Federal income tax withheld (box 4).
    #[serde(default)]
    withholding: f64,
    // Section 199A REIT dividends (box 5); zero means none reported.
    #[serde(default)]
    section199a: f64,
    // Exempt-interest dividends (box 12).
    #[serde(default)]
    exempt_dividends: f64,
    // Specified private-activity bond interest dividends included in box 12
    // (box 13).
    #[serde(default)]
    private_activity_dividends: f64,
}

// 1099-OID, mirroring `FederalOid`'s modeled boxes (unit C4a, corrected by
// C4a-fix): 1, 2, 3, 4, 5, 6, 8, 10, 11.
#[derive(serde::Deserialize)]
struct Oid1099Fixture {
    #[allow(dead_code)]
    payer: String,
    box1_oid: f64,
    #[serde(default)]
    box2_other_periodic_interest: f64,
    // Early withdrawal penalty; shares the same field/limit as 1099-INT box 2.
    #[serde(default)]
    box3_early_penalty: f64,
    #[serde(default)]
    box4_withholding: f64,
    #[serde(default)]
    box5_market_discount: f64,
    #[serde(default)]
    box6_acquisition_premium: f64,
    // Original issue discount on U.S. Treasury obligations; may be negative.
    #[serde(default)]
    box8_treasury_oid: f64,
    #[serde(default)]
    box10_bond_premium: f64,
    // Total tax-exempt OID, including private-activity bond discount.
    #[serde(default)]
    box11_exempt: f64,
    // Specified private-activity bond discount within box 11, from the
    // payer's supplemental statement; not itself a numbered box.
    #[serde(default)]
    private_activity: f64,
}

#[derive(serde::Deserialize)]
struct CareFixture {
    provider_name: String,
    provider_address: String,
    combined_plan_maximum: f64,
}

#[derive(serde::Deserialize)]
struct ExpectedFixture {
    state: String,
    lines: BTreeMap<String, f64>,
}

/// Dollars-with-cents JSON number to exact cents.
fn cents(dollars: f64) -> i64 {
    (dollars * 100.0).round() as i64
}

/// "YYYY-MM-DD" to the model's YYYYMMDD-as-integer date encoding.
fn parse_date(date: &str) -> u32 {
    let parts: Vec<&str> = date.split('-').collect();
    assert_eq!(parts.len(), 3, "expected YYYY-MM-DD, got {date}");
    let year: u32 = parts[0].parse().unwrap();
    let month: u32 = parts[1].parse().unwrap();
    let day: u32 = parts[2].parse().unwrap();
    year * 10000 + month * 100 + day
}

fn filing_status(name: &str) -> FederalStatus {
    match name {
        "Single" => FederalStatus::Single,
        "Joint" => FederalStatus::Joint,
        "Separate" => FederalStatus::Separate,
        "Head" => FederalStatus::Head,
        "Surviving" => FederalStatus::Surviving,
        other => panic!("unknown filing_status {other}"),
    }
}

/// Builds a complete accepted `FederalDraft` from a fixture the same way
/// `complete()` does: every section the fixture does not mention keeps
/// `complete()`'s "nothing applies" defaults. Only the filer's filing status,
/// birth dates, income documents and estimated payments are overridden.
fn build_draft(fixture: &ReturnFixture) -> FederalDraft {
    let status = filing_status(&fixture.filing_status);
    let mut d = complete(status);
    d.screening.as_mut().unwrap().taxable_premium_election = fixture.taxable_bond_premium_election;
    d.cookie = fixture.name.clone();
    {
        let people = d.people.as_mut().unwrap();
        people.taxpayer.birth_date = parse_date(&fixture.birth_dates.taxpayer);
        match (&fixture.birth_dates.spouse, people.spouse.as_mut()) {
            (Some(date), Some(spouse)) => spouse.birth_date = parse_date(date),
            (None, None) => {}
            (Some(_), None) => {
                panic!("fixture {}: a spouse birth date needs a joint status", fixture.name)
            }
            (None, Some(_)) => {
                panic!("fixture {}: a joint status needs the spouse's birth date", fixture.name)
            }
        }
    }
    d.setup.as_mut().unwrap().paid_over_half_home = fixture.paid_over_half_home;
    d.income.clear();
    d.next_id = 1;
    for w2 in &fixture.w2 {
        let mut entry = income(FederalIncome::Wage {
            data: wage_source(FederalWage {
                tips_without_w2: false,
                abroad: cents(w2.abroad),
                deferred_pension: 0,
                inmate_pay: 0,
                disability: DisabilityIncome::None,
                wages: cents(w2.box1_wages),
                withholding: cents(w2.box2_withheld),
                social_wages: cents(w2.box3),
                social_tips: 0,
                employee_tips: None,
                work_deductions: FederalWorkDeductions::NONE,
                social_tax: cents(w2.box4),
                medicare_wages: cents(w2.box5),
                medicare_tax: cents(w2.box6),
                uncollected_tax: None,
                medicaid_waiver: 0,
                statutory_business: None,
                paying_scorporation: None,
                payroll: crate::W2Payroll::SocialSecurity,
                coverage: match w2.coverage.as_str() {
                    "Ordinary" => W2Coverage::Ordinary,
                    "Ministerial" => W2Coverage::Ministerial,
                    other => panic!("unsupported W-2 coverage {other}"),
                },
                care_benefits: 0,
                statutory_employee: false,
                retirement_plan: false,
                other_box12: Vec::new(),
            }),
        });
        entry.owner = crate::DocumentOwner::from(w2_owner(&w2.owner));
        entry.label = w2.employer.clone();
        if let FederalIncome::Wage { data: crate::WageSource::W2(data) } = &mut entry.income
            && w2.box12_l > 0.0
        {
            data.form.box12.push(crate::PrintedBox12Entry {
                code: W2Code::L,
                amount: Some(cents(w2.box12_l)),
            });
        }
        let id = d.next_id;
        add(&mut d, entry);
        if w2.box10 > 0.0 {
            let plan = fixture.care.as_ref().expect("box 10 requires plan documents");
            care_w2(
                &mut d,
                id,
                cents(w2.box10),
                cents(w2.care_in_box_1),
                cents(w2.care_forfeited),
                cents(plan.combined_plan_maximum),
            );
        }
    }
    for int in &fixture.int1099 {
        add(
            &mut d,
            income(FederalIncome::Interest {
                data: interest_source(FederalInterest {
                    taxable: cents(int.box1_interest),
                    early_penalty: cents(int.box2_penalty),
                    exempt: cents(int.box8_exempt),
                    private_activity: cents(int.box9_private_activity),
                    withholding: cents(int.box4_withholding),
                    bond_premium: cents(int.box11_bond_premium),
                    tax_exempt_bond_premium: cents(int.box13_tax_exempt_bond_premium),
                    accrued_interest: cents(int.accrued_interest),
                    ..Default::default()
                }),
            }),
        );
    }
    for div in &fixture.div1099 {
        add(
            &mut d,
            income(FederalIncome::Dividend {
                data: dividend_source(FederalDividend {
                    ordinary: cents(div.ordinary),
                    qualified: cents(div.qualified),
                    section199a: if div.section199a > 0.0 {
                        ReitDividends::Reported {
                            amount: cents(div.section199a),
                            eligible: Some(true),
                        }
                    } else {
                        ReitDividends::None
                    },
                    capital_distributions: cents(div.capital_distributions),
                    collectible_distributions: cents(div.collectible_distributions),
                    unrecaptured_distributions: cents(div.unrecaptured_distributions),
                    withholding: cents(div.withholding),
                    exempt_dividends: cents(div.exempt_dividends),
                    private_activity_dividends: cents(div.private_activity_dividends),
                    ..Default::default()
                }),
            }),
        );
    }
    for oid in &fixture.oid1099 {
        add(
            &mut d,
            income(FederalIncome::Oid {
                data: oid_source(FederalOid {
                    oid: cents(oid.box1_oid),
                    other_periodic_interest: cents(oid.box2_other_periodic_interest),
                    early_penalty: cents(oid.box3_early_penalty),
                    withholding: cents(oid.box4_withholding),
                    market_discount: cents(oid.box5_market_discount),
                    acquisition_premium: cents(oid.box6_acquisition_premium),
                    exempt: cents(oid.box11_exempt),
                    bond_premium: cents(oid.box10_bond_premium),
                    private_activity: cents(oid.private_activity),
                    treasury_oid: cents(oid.box8_treasury_oid),
                    ..Default::default()
                }),
            }),
        );
    }
    for pay in &fixture.foreign_pay {
        let mut entry = income(FederalIncome::ForeignEmployment {
            data: ForeignWages { wages: cents(pay.wages) },
        });
        entry.owner = crate::DocumentOwner::from(w2_owner(&pay.owner));
        entry.label = pay.employer.clone();
        add(&mut d, entry);
    }
    for business in &fixture.business {
        let mut entry = income(cash_business(0));
        entry.label = business.label.clone();
        let FederalIncome::Business { data } = &mut entry.income else { unreachable!() };
        data.activity = BusinessActivity::Ordinary {
            returns: 0,
            receipts: cents(business.receipts),
            accounting: BusinessAccounting::Cash,
        };
        data.expenses = vec![ordinary_business_expense(
            BusinessExpenseCategory::Other { label: "Operating costs".into() },
            cents(business.expenses),
        )];
        data.ministerial = business.ministerial;
        add(&mut d, entry);
    }
    for clergy in &fixture.clergy {
        d.screening.as_mut().unwrap().clergy.push(ClergyClaim {
            owner: Owner::Taxpayer,
            role: ClergyRole::Minister {
                housing: MinisterHousing::Parsonage {
                    fair_rental_value: cents(clergy.fair_rental_value),
                    utility_allowance: cents(clergy.utility_allowance),
                    utilities_paid: cents(clergy.utilities_paid),
                },
                form_4361: clergy.form_4361,
                employee_expenses: cents(clergy.employee_expenses),
            },
        });
    }
    for form in &fixture.form2555 {
        let answer = |name: &str| match name {
            "Yes" => true,
            "No" => false,
            other => panic!("unknown answer {other}"),
        };
        let qualification = match form.qualification.as_str() {
            "Residence" => ForeignQualification::Residence {
                began: parse_date(form.began.as_ref().expect("began")),
                ended: form.ended.as_ref().map(|date| parse_date(date)),
                basis: crate::ResidenceBasis::Citizen,
                nonresident_statement: false,
            },
            "Presence" => ForeignQualification::Presence {
                starts_on: parse_date(form.starts_on.as_ref().expect("starts_on")),
                foreign_days: form.foreign_days,
            },
            other => panic!("unknown qualification {other}"),
        };
        let housing =
            form.housing.as_ref().map_or(ForeignHousing::None, |h| ForeignHousing::Claim {
                stays: h
                    .stays
                    .iter()
                    .map(|stay| ForeignHousingStay {
                        location: housing_location(&stay.location),
                        starts_on: parse_date(&stay.starts_on),
                        ends_on: parse_date(&stay.ends_on),
                    })
                    .collect(),
                expenses: cents(h.expenses),
                nontaxable_allowances: 0,
                prior: None,
                household: ForeignHousingHousehold::OnlyClaim,
            });
        let claim = Form2555 {
            qualification,
            tax_home: answer(&form.tax_home),
            revocation: ExclusionRevocation::None,
            prior_year_pay: cents(form.prior_year_pay),
            prior_year_history: form.prior_year_history.as_ref().map(|p| PriorYearHistory {
                prior_year_days: p.prior_year_days,
                prior_year_excluded: cents(p.prior_year_excluded),
            }),
            multi_year_pay: form
                .multi_year_pay
                .iter()
                .map(|a| MultiYearPay {
                    description: a.description.clone(),
                    amount: cents(a.amount),
                    months_total: a.months_total,
                    months_in_return_year: a.months_in_return_year,
                    months_in_prior_year: a.months_in_prior_year,
                })
                .collect(),
            exclusion: form.exclusion,
            housing,
        };
        assert_eq!(claim.rule(), None, "fixture {}", fixture.name);
        set_claim(&mut d, claim.record(w2_owner(&form.owner)));
    }
    for form in &fixture.form2106 {
        let owner = w2_owner(&form.owner);
        let capacity = match form.capacity.as_str() {
            "Reservist" => EmployeeCapacity::Reservist,
            "PerformingArtist" => EmployeeCapacity::PerformingArtist,
            "FeeBasisOfficial" => EmployeeCapacity::FeeBasisOfficial,
            other => panic!("unknown Form 2106 capacity {other}"),
        };
        let employers = form
            .employers
            .iter()
            .map(|name| {
                d.income
                    .iter()
                    .find(|e| e.label == *name && e.owner == crate::DocumentOwner::from(owner))
                    .unwrap_or_else(|| panic!("no W-2 for {name}"))
                    .id
            })
            .collect();
        let (mileage, vehicle_assets) = crate::interview::vehicle_and_asset_records(
            &form
                .standard_mileage
                .iter()
                .map(|m| Vehicle {
                    method: VehicleMethod::StandardMileage {
                        id: 0,
                        first_half: m.first_half,
                        second_half: m.second_half,
                        total: m.total,
                        more: MileageMore {
                            placed: None,
                            personal_use_available: true,
                            other_vehicle: true,
                            evidence: VehicleEvidence::Written,
                            commuting: m.commuting,
                            parking_tolls: cents(m.parking_tolls),
                            interest: 0,
                            property_tax: 0,
                            leased: false,
                        },
                    },
                })
                .collect::<Vec<_>>(),
            &[],
        );
        d.adjustments.as_mut().unwrap().employee_expenses.push(EmployeeExpenseClaim {
            improvement_buildings: Vec::new(),
            equipment_assets: vec![],
            dot_hours_of_service: false,
            leased_vehicles: vec![],
            owner,
            occupation: form.occupation.clone(),
            capacity,
            employers,
            local_transportation: cents(form.local_transportation),
            travel: cents(form.travel),
            other: cents(form.other),
            meals: cents(form.meals),
            reimbursements: cents(form.reimbursements),
            mileage,
            vehicle_assets,
        });
    }
    for person in &fixture.dependents {
        let id = d.next_id;
        d.next_id += 1;
        d.dependents.push(build_dependent(id, person));
    }
    for e in &fixture.education {
        let student = match e.student.as_str() {
            "Taxpayer" => -1,
            "Spouse" => -2,
            other => d.dependents()[other.parse::<usize>().expect("student position")].id,
        };
        add_education(d.credits.as_mut().unwrap(), {
            let mut pair = native_education(
                student,
                FederalEducationExpenses {
                    tuition: cents(e.box1_tuition),
                    school_materials: 0,
                    other_materials: 0,
                    assistance: cents(e.box5_scholarships),
                    refunds: 0,
                    other_benefits: 0,
                    scholarship_in_income: e.living_costs.map_or(ScholarshipInclusion::None, |c| {
                        ScholarshipInclusion::Automatic { living_costs: cents(c) }
                    }),
                },
                EducationChoice::AmericanOpportunity,
            );

            pair.0.id = 0;
            pair.0.half_time = true;
            pair.0.graduate = false;
            pair.1.more.american_opportunity_used_up = Some(false);
            pair
        });
    }
    // A completed care page includes the provider's records. The historical
    // no-benefit fixture uses its synthetic provider; the Form2441 fixture
    // supplies the actual synthetic document names and plan statement.
    document_care(&mut d);
    if let Some(care) = &fixture.care {
        let facts = &mut d.credits.as_mut().unwrap().care;
        if let Some(provider) = facts.providers.first_mut() {
            provider.name = care.provider_name.clone();
            provider.address = care.provider_address.clone();
        }
    }
    d.payments.as_mut().unwrap().estimated = crate::tests::support::builders::unknown_estimated(
        cents(fixture.estimated_payments),
        PaymentPayer::Taxpayer,
    );
    d
}

/// Finds a Form 1040 return line by its reference (e.g. "1a", "9", "12e"),
/// matching the "1040 <reference> — <label>" prefix `federal_rules.rs` emits.
fn find_return_line(result: &FederalResult, reference: &str) -> Option<i64> {
    let prefix = alloc::format!("1040 {reference} —");
    result
        .return_lines
        .iter()
        .find(|line| status_text(&line.label).starts_with(prefix.as_str()))
        .map(|line| line.amount)
}

fn check_fixture(raw: &str, mismatches: &mut Vec<String>) {
    let fixture: ReturnFixture = serde_json::from_str(raw).expect("valid fixture JSON");
    let draft = build_draft(&fixture);
    let result = estimate(&draft);
    let actual_state = alloc::format!("{:?}", result.state);
    if actual_state != fixture.expected.state {
        mismatches.push(alloc::format!(
            "{}: state expected {}, got {}",
            fixture.name,
            fixture.expected.state,
            actual_state
        ));
    }
    for (reference, expected_dollars) in &fixture.expected.lines {
        let expected = cents(*expected_dollars);
        match find_return_line(&result, reference) {
            Some(actual) if actual == expected => {}
            Some(actual) => mismatches.push(alloc::format!(
                "{}: line {} expected {:.2}, got {:.2}",
                fixture.name,
                reference,
                expected as f64 / 100.0,
                actual as f64 / 100.0
            )),
            None => mismatches.push(alloc::format!(
                "{}: line {} not found in return_lines",
                fixture.name,
                reference
            )),
        }
    }
}

#[test]
fn returns_fixtures_match_expected_lines() {
    let fixtures: &[&str] = &[
        include_str!("../../../tests/fixtures/returns/m1-single-w2.json"),
        include_str!("../../../tests/fixtures/returns/form2441-two-children-w2-benefits.json"),
        include_str!("../../../tests/fixtures/returns/m2-hoh-two-children.json"),
        include_str!(
            "../../../tests/fixtures/returns/w2-two-employers-excess-ss-additional-medicare.json"
        ),
        include_str!("../../../tests/fixtures/returns/int-div-oid-single.json"),
        include_str!("../../../tests/fixtures/returns/schedule-b-joint-typical.json"),
        include_str!("../../../tests/fixtures/returns/mfj-three-children-magi-phaseout.json"),
        include_str!("../../../tests/fixtures/returns/reservist-drill-travel.json"),
        include_str!("../../../tests/fixtures/returns/performing-artist-agi-boundary.json"),
        include_str!(
            "../../../tests/fixtures/returns/fee-basis-official-excess-reimbursement.json"
        ),
        include_str!("../../../tests/fixtures/returns/bona-fide-resident-w2-at-limit.json"),
        include_str!("../../../tests/fixtures/returns/physical-presence-partial-year-housing.json"),
        include_str!("../../../tests/fixtures/returns/scholarship-election-bill.json"),
        include_str!("../../../tests/fixtures/returns/minister-parsonage-and-fees.json"),
        include_str!("../../../tests/fixtures/returns/bonus-across-years-and-prior-year-room.json"),
    ];
    let mut mismatches: Vec<String> = Vec::new();
    for raw in fixtures {
        check_fixture(raw, &mut mismatches);
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

fn m1_single_w2_draft() -> FederalDraft {
    let raw = include_str!("../../../tests/fixtures/returns/m1-single-w2.json");
    let fixture: ReturnFixture = serde_json::from_str(raw).expect("valid fixture JSON");
    build_draft(&fixture)
}

fn m1_single_w2_result() -> FederalResult {
    estimate(&m1_single_w2_draft())
}

#[test]
fn m1_forms_emit_their_typed_lines() {
    let result = m1_single_w2_result();
    let coverage = form_coverage(&result);
    let form_1040 = coverage.get(&TaxForm::Form1040).expect("Form 1040 lines are typed");
    let missing: Vec<&str> = ["1a", "2b", "9", "12e", "15", "16", "24a", "25a", "33", "34"]
        .into_iter()
        .filter(|line| !form_1040.contains(*line))
        .collect();
    assert!(missing.is_empty(), "Form 1040 lines missing: {missing:?}");
    assert!(
        coverage.get(&TaxForm::FormW2).is_some_and(|lines| !lines.is_empty()),
        "W-2 emits no typed line"
    );
    // $300 of interest is below the $1,500 threshold: no Schedule B is shown.
    assert!(!coverage.contains_key(&TaxForm::ScheduleB));
    // Above the threshold the typed Schedule B lines appear.
    let mut d = m1_single_w2_draft();
    for entry in d.income.iter_mut() {
        if let FederalIncome::Interest { data: crate::InterestSource::Int1099(data) } =
            &mut entry.income
        {
            data.form.interest_income = 200_000; // $2,000.00
        }
    }
    let over = form_coverage(&estimate(&d));
    assert!(
        over.get(&TaxForm::ScheduleB).is_some_and(|lines| lines.contains("4")),
        "Schedule B line 4 is typed above the threshold"
    );
}

#[test]
fn typed_lines_agree_with_labels() {
    let result = m1_single_w2_result();
    assert!(!result.return_lines.is_empty());
    for line in rows(&result.return_lines) {
        let reference = line
            .label
            .strip_prefix("1040 ")
            .and_then(|rest| rest.split_once(" — "))
            .map(|(reference, _)| reference)
            .unwrap_or_else(|| panic!("unexpected return line label {}", line.label));
        let typed = line
            .form_line
            .as_ref()
            .unwrap_or_else(|| panic!("return line {} has no form line", line.label));
        assert_eq!(typed.form, TaxForm::Form1040, "{}", line.label);
        assert_eq!(typed.line, reference, "{}", line.label);
    }
}

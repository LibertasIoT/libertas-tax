// Merge review 3 (2026-10-11) seam defects, F17: Form 1116 line 3e holds each K-1 gain and each
// K-3 box 11 item once, a community royalty's depletion follows the filer's share and has its
// Form 8958 row, the scholarship election searches the whole included range, and a Roth IRA
// excess contribution returned with earnings (code 8J, box 7b unchecked) is an IRA amount.
use super::*;
use crate::tests::cross::interview::{accepted, document, r1099, save_topic};
extern crate std;
use crate::{PartnerIncome, PartnerIncomeCode};

fn cents(d: i64) -> i64 {
    d * 100
}
fn claim_no_sources() -> ForeignTaxClaim {
    let mut claim =
        crate::tests::foreign::foreign::fund_claim(0, 0, 0, ForeignRateAdjustment::Exception);
    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut claim {
        sources.clear();
    }
    claim
}
// Fixture A of foreign.rs: joint, W-2 $150,000, the K-3 partner (box 5 $2,000, 6a $10,000, 21
// $900; K-3 Japan passive $6,000 / $900, U.S. gross income $6,000 plus `extra_us`).
pub(in crate::tests) fn k3_base(extra_us: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 15_000_000, 0);
    let mut k3 = crate::tests::foreign::foreign::k3_blank();
    k3.countries =
        vec![crate::tests::foreign::foreign::k3_row("JA", K3Category::Passive, 6_000, 0, 900)];
    k3.withholding =
        vec![crate::tests::foreign::foreign::k3_answer("JA", ForeignWithholding::AtLeast15)];
    k3.us_gross = cents(6_000 + extra_us);
    crate::tests::foreign::foreign::add_named(
        &mut d,
        "Synthetic Global Fund LP",
        FederalIncome::Partnership {
            data: crate::tests::foreign::foreign::k3_partner(2_000, 10_000, 900, Some(k3)),
        },
    );
    d.credits.as_mut().unwrap().foreign_tax = claim_no_sources();
    d
}
fn funded(partner: &mut PartnerK1) {
    partner.regular.basis.opening = cents(50_000);
    partner.regular.risk.start =
        ScorpRiskStart::Detailed { capacity: cents(50_000), profit: 0, loss: 0 };
}
fn with_k3_partner(d: &mut FederalDraft, items: Vec<PartnerIncome>) {
    if let FederalIncome::Partnership { data } = &mut d.income[1].income {
        funded(data);
        data.income = items;
    }
}
pub(in crate::tests) fn line_3e(r: &FederalResult) -> i64 {
    form_1116_any(r, false, "_gross_income_all_sources")
}
fn item(code: PartnerIncomeCode, dollars: i64) -> PartnerIncome {
    PartnerIncome { code, amount: cents(dollars) }
}

#[test]
fn d1_code_r_gains_of_two_k1s_enter_line_3e_each_whole() {
    let mut d = k3_base(0);
    let mut x = ordinary_partner_source(0, true);
    funded(&mut x);
    x.income = vec![item(PartnerIncomeCode::SpeciallyAllocatedOrdinary, 8_000)];
    let mut y = ordinary_partner_source(0, true);
    funded(&mut y);
    y.income = vec![item(PartnerIncomeCode::SpeciallyAllocatedOrdinary, -5_000)];
    crate::tests::foreign::foreign::add_named(
        &mut d,
        "X LP",
        FederalIncome::Partnership { data: x },
    );
    crate::tests::foreign::foreign::add_named(
        &mut d,
        "Y LP",
        FederalIncome::Partnership { data: y },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 165_000);
    // The $8,000 gain whole, the $5,000 loss restored (no loss subtracted): 150,000 + 2,000 +
    // 10,000 (K-3 gross 6,000 replaces... fixture A's 162,000) + 8,000 = 170,000.
    assert_eq!(line_3e(&r), 170_000);
}

#[test]
fn d2_k3_partner_code_a_is_inside_the_k3_gross_once() {
    let mut d = k3_base(4_000);
    with_k3_partner(&mut d, vec![item(PartnerIncomeCode::PortfolioIncome, 4_000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 166_000);
    assert_eq!(line_3e(&r), 166_000);
}

#[test]
fn d2_k3_partner_code_s_and_r_are_inside_the_k3_gross_once() {
    let mut d = k3_base(5_000);
    with_k3_partner(
        &mut d,
        vec![
            item(PartnerIncomeCode::NonportfolioLong, 3_000),
            item(PartnerIncomeCode::SpeciallyAllocatedOrdinary, 2_000),
        ],
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 167_000);
    assert_eq!(line_3e(&r), 167_000);
}

fn community_royalty(depletion: RoyaltyDepletion) -> FederalResult {
    let mut d = complete(FederalStatus::Separate);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    d.income = vec![wage(1, Owner::Taxpayer, 3_000_000, 300_000)];
    d.next_id = 2;
    add(
        &mut d,
        income(FederalIncome::Royalty {
            data: FederalRoyalty {
                kind: RoyaltyKind::OilGas,
                receipts: cents(10_000),
                advance: 0,
                expenses: vec![],
                depletion,
                ordinary: true,
            },
        }),
    );
    d.setup.as_mut().unwrap().community_property =
        CommunityProperty::Yes { state: CommunityState::Arizona, rule: CommunityRule::Halves };
    for e in &mut d.income {
        e.community = CommunityShare::Community;
    }
    supported(&d)
}

#[test]
fn d3_community_royalty_depletion_follows_the_filers_share() {
    let r = community_royalty(RoyaltyDepletion::Percentage {
        adjusted_basis: 0,
        recoverable_units: 0,
        units_sold: 0,
        carryover: 0,
        amt_carryover: 0,
    });
    // Wages 15,000; royalty 5,000 less 15% of the filer's 5,000 half: 750.
    assert_eq!(value(&r, "agi"), 19_250);
    // Schedule E rows are the filer's half: line 4 5,000, line 18 750, line 21 4,250.
    let row = |suffix: &str| {
        r.lines
            .iter()
            .find(|l| l.id.starts_with("royalty_") && l.id.ends_with(suffix))
            .map(|l| l.amount / 100)
    };
    assert_eq!(row("line4_royalties"), Some(5_000));
    assert_eq!(row("line18_depletion"), Some(750));
    assert_eq!(row("line21_income"), Some(4_250));
    // Form 8958 lists the royalty (receipts less expenses) and the filer's half.
    let form_8958 = |suffix: &str| {
        r.lines
            .iter()
            .filter(|l| l.id.starts_with("form8958_") && l.id.ends_with(suffix))
            .map(|l| l.amount / 100)
            .collect::<Vec<_>>()
    };
    assert!(form_8958("_total").contains(&10_000));
    assert!(form_8958("_this_return").contains(&5_000));
}

fn roth_return(printed: &str, ira: bool) -> FederalResult {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19860202;
    let mut r = printed_as(
        r1099(cents(7_650), DistributionCode::Corrective { excess_deferral: 0 }, ira),
        printed,
    );
    r.taxable = cents(150);
    let mut s = Interview::new(TEST_USER, d.clone());
    let reply = send(
        &mut s,
        save_topic(&d, TaxSection::Retirement, vec![document(0, IncomeDocument::R1099(r))]),
    );
    accepted(&reply);
    supported(&s.draft)
}

#[test]
fn d8_code_8j_with_box_7b_unchecked_is_an_ira_line_4b_amount() {
    let base = {
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.birth_date = 19860202;
        supported(&d)
    };
    let r = roth_return("8J", false);
    assert_eq!(return_line(&r, "1h"), return_line(&base, "1h"));
    assert_eq!(return_line(&r, "4a"), cents(7_650));
    assert_eq!(return_line(&r, "4b"), cents(150));
}

fn hoh_student(wages: i64, inclusion: ScholarshipInclusion) -> FederalDraft {
    let mut d = complete(FederalStatus::Head);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19980301;
    d.income[0] = wage(1, Owner::Taxpayer, wages, 100_000);
    add_child(&mut d);
    d.credits.as_mut().unwrap().education = vec![FederalEducation {
        student: -1,
        method: EducationChoice::AmericanOpportunity,
        expenses: FederalEducationExpenses {
            tuition: 400_000,
            assistance: 400_000,
            scholarship_in_income: inclusion,
            ..education_expenses(400_000)
        },
    }];
    d
}
fn net(r: &FederalResult) -> i64 {
    r.tax.unwrap() - r.refundable_credits.unwrap()
}

// Wages $20,890: the one-child EIC plateau (a $50 row of the table) ends at AGI $23,899, an
// included part of $3,009 between the candidates $2,000 and $4,000. The Automatic election must
// be no worse than any explicit part of the range (i1040 / Pub. 970 chapter 2: the one with the
// lowest total tax).
#[test]
fn d4_scholarship_automatic_election_searches_the_whole_range() {
    let wages = 2_089_000;
    let automatic =
        supported(&hoh_student(wages, ScholarshipInclusion::Automatic { living_costs: 440_000 }));
    let chosen = value(&automatic, "education_1_scholarship_included");
    let mut best = i64::MAX;
    for part in (0..=4_000).filter(|p| p % 250 == 0 || (2_950..=3_100).contains(p)) {
        let inclusion = if part == 0 {
            ScholarshipInclusion::None
        } else {
            ScholarshipInclusion::Amount { included: cents(part) }
        };
        best = best.min(net(&supported(&hoh_student(wages, inclusion))));
    }
    assert_eq!(net(&automatic), best, "chose {chosen}");
    // The plateau's last dollar, not either breakpoint around it.
    assert_eq!(chosen, 3_009);
}

// S1: the S corporation loan-repayment gain (Form 4797 Part II line 11) is in AGI through
// Schedule 1 line 4 and counted once in line 3e, not added again on top of AGI.
#[test]
fn s1_scorp_loan_repayment_gain_is_in_line_3e_once() {
    let source = scorp_repaid_reduced_open_account(ScorpLoanKind::OpenAccount);
    let mut d = k3_base(0);
    crate::tests::foreign::foreign::add_named(
        &mut d,
        "S LLC",
        FederalIncome::Scorp { data: source },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "scorp_ordinary_loan_gain"), 2_000);
    assert_eq!(value(&r, "agi"), 164_000);
    assert_eq!(line_3e(&r), 164_000);
}

// S2: a sold partnership interest's Form 8949 box F gain (proceeds 72,000, basis 41,000) is also
// in Schedule D line 10's columns (d) and (e), so line 10 foots as (d) less (e).
#[test]
fn s2_k1_sale_is_in_the_schedule_d_columns() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Partnership {
            data: crate::tests::passthrough::k1_items::fixture_4(),
        }),
    );
    d.income[0] = wage(1, Owner::Taxpayer, cents(90_000), cents(9_000));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain_10"), 31_000);
    assert_eq!(value(&r, "capital_gain_10_proceeds"), 72_000);
    assert_eq!(value(&r, "capital_gain_10_basis"), 41_000);
}

// S3: a child who died before the 17th birthday never attains 17 (IRC 24(c)(1)): the child tax
// credit, not the credit for other dependents. Head of household, W-2 $30,000, child born
// 2009-11-15, died 2026-03-01; the control is a child born 2010-01-01 who lived all year.
#[test]
fn s3_child_who_died_before_the_17th_birthday_is_a_child_tax_credit_child() {
    let mut d = complete(FederalStatus::Head);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19850410;
    d.income[0] = wage(1, Owner::Taxpayer, 3_000_000, 0);
    add_child(&mut d);
    d.dependents[0].birth_date = 20091115;
    d.dependents[0].death_date = Some(20260301);
    d.dependents[0].residency = DependentResidency::WholeYearException;
    let r = supported(&d);
    // 2,200 = line 19 588 (the tax) + line 28 1,612.
    assert_eq!(value(&r, "child_credit"), 588);
    assert_eq!(return_line(&r, "19"), cents(588));
    assert_eq!(return_line(&r, "28"), cents(1_612));
    // Died after the 17th birthday: not a qualifying child for the credit (credit for other
    // dependents).
    d.dependents[0].birth_date = 20090115;
    assert_eq!(value(&supported(&d), "child_credit"), 500);
}

fn filer(birth: u32) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = birth;
    d
}
fn form(printed: &str, ira: bool, gross: i64, taxable: i64, code: DistributionCode) -> R1099 {
    let mut r = printed_as(r1099(cents(gross), code, ira), printed);
    r.taxable = cents(taxable);
    r
}
fn save_forms(d: &FederalDraft, forms: Vec<R1099>) -> FederalResult {
    let mut s = Interview::new(TEST_USER, d.clone());
    let documents = forms.into_iter().map(|r| document(0, IncomeDocument::R1099(r))).collect();
    accepted(&send(&mut s, save_topic(d, TaxSection::Retirement, documents)));
    supported(&s.draft)
}
fn additional_tax(r: &FederalResult) -> i64 {
    r.lines.iter().find(|l| l.id == "early_distribution_tax").map_or(0, |l| l.amount / 100)
}

// S9: an owner who was 59 1/2 before 2026 is never taxed the additional 10 percent on a code 1
// distribution, whatever exception was entered (IRC 72(t)(2)(A)(i); i5329 exception 12). Born
// 1960-01-01, code 1 $20,000, education exception $5,000.
#[test]
fn s9_owner_over_59_half_keeps_no_additional_tax_with_a_partial_exception() {
    let code = DistributionCode::Early {
        what_happened: Disposition::Kept,
        exception: EarlyException::Applies {
            second: None,
            distributed: 20260615,
            facts: RetirementException::Education {
                education: cents(5_000),
                room_board: 0,
                allowance: 0,
                school_housing: 0,
                half_time: false,
                assistance: 0,
                eligible: true,
            },
            repaid: None,
        },
    };
    let r = save_forms(&filer(19600101), vec![form("1", true, 20_000, 20_000, code)]);
    assert_eq!(additional_tax(&r), 0);
}

// S11: a code 5 prohibited transaction is a distribution on January 1 (IRC 408(e)(2)(B)): an
// owner born 1966-08-20 (59 1/2 on 2026-02-20) is 59 and 4 months old, so exception 12 does not
// apply and the 10 percent of $50,000 is $5,000, whatever date the page holds.
#[test]
fn s11_code_5_is_tested_on_january_1() {
    let code = DistributionCode::ProhibitedTransaction {
        exception: EarlyException::Applies {
            second: None,
            distributed: 20260601,
            facts: RetirementException::ReachedAge,
            repaid: None,
        },
    };
    let r = save_forms(&filer(19660820), vec![form("5", true, 50_000, 50_000, code)]);
    assert_eq!(additional_tax(&r), 5_000);
    // Without the exception the same row is supported and taxed the same.
    let r = save_forms(
        &filer(19660820),
        vec![form(
            "5",
            true,
            50_000,
            50_000,
            DistributionCode::ProhibitedTransaction { exception: EarlyException::None },
        )],
    );
    assert_eq!(additional_tax(&r), 5_000);
}

// S8: a first-time-homebuyer IRA distribution (Form 8915-F Part IV) repaid in 2026 leaves Form
// 8606 line 7 (i8606 line 7) and line 6 stays (i8606 line 6): basis $6,000, December 31 value
// $30,000, $20,000 distributed, $8,000 repaid. Line 7 is $12,000, line 9 $42,000, so $1,714 is
// nontaxable and line 4b is $10,286; Form 8915-F line 29 is zero for an IRA; the 10 percent
// applies to the $10,286 kept ($1,029).
#[test]
fn s8_home_purchase_repayment_leaves_form_8606_line_7() {
    let mut d = filer(19860909);
    d.income[0] = wage(1, Owner::Taxpayer, 12_000_000, 2_000_000);
    let mut e = basis_distribution(Owner::Taxpayer, 20_000, RetirementTreatment::Early);
    let FederalIncome::Retirement { data } = &mut e.income else { panic!() };
    data.box7 = crate::DistributionCodes::CODE_1.0;
    data.disaster = DisasterDistribution::HomePurchase {
        fema: 4900,
        began: 20260301,
        ended: 20260320,
        declared: 20260415,
        distributed: 20260210,
        taxable_repaid: cents(8_000),
    };
    add(&mut d, e);
    d.adjustments.as_mut().unwrap().ira_basis =
        vec![prior_ira_basis(Owner::Taxpayer, 6_000, 30_000)];
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), cents(10_286));
    assert_eq!(additional_tax(&r), 1_029);
    let id = |suffix: &str| {
        r.lines
            .iter()
            .find(|l| l.id.starts_with("form_8915_taxpayer_2026_line_") && l.id.ends_with(suffix))
            .map(|l| l.amount / 100)
    };
    assert_eq!(id("28"), Some(20_000));
    assert_eq!(id("29"), Some(0));
    assert_eq!(id("31"), Some(8_000));
    assert_eq!(id("32"), Some(12_000));
}

// S7: the two new nullable fields of exported types default to null.
#[test]
fn s7_new_nullable_fields_declare_a_null_default() {
    let partnership = include_str!("../../passthrough/partnership.rs");
    assert!(
        partnership.contains(
            "#[libertas_default($null)]\n    pub interest_term: Option<crate::GainTerm>,"
        )
    );
    let model = include_str!("../../federal/federal_model.rs");
    assert!(model.contains(
        "#[libertas_default($null)]\n        current: Option<FederalMortgageCertificate>,"
    ));
}

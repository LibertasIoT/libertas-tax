use super::*;
use crate::tests::federal::unasked_pages::{
    item, lump_sum_return, officer, pension, problem, retirement_page, save_retirement,
};
use crate::tests::support::assert::{accepted, rejected_with};
fn retirement_documents(s: &mut Interview) -> Vec<IncomeDocumentItem> {
    let P::SaveIncomes { documents, .. } = retirement_page(s) else { unreachable!() };
    documents.documents
}
fn excluded(r: &FederalResult) -> i64 {
    r.lines.iter().find(|l| l.id == "public_safety_1_excluded").map_or(0, |l| l.amount)
}

#[test]
fn public_safety_premiums_through_the_1099_r_by_hand() {
    // Single filer, W-2 $60,000; a governmental pension 1099-R of $30,000
    // (box 2a $30,000, code 7). The officer's health premiums were $5,000,
    // $4,000 of them paid from this pension (Pub. 575: up to $3,000 of the
    // premiums paid from the distribution is excluded).
    //   excluded = min(4,000, 3,000, 30,000)       = $3,000
    //   line 5a = $30,000; 5b = 30,000 - 3,000     = $27,000
    //   AGI = 60,000 + 27,000                      = $87,000
    //   line 15 = 87,000 - 16,100                  = $70,900
    //   line 16: row $70,900-$70,950, midpoint $70,925:
    //     1,240 + 4,560 + 22% x (70,925 - 50,400) = 5,800 + 4,515.50
    //                                                = $10,315.50 -> $10,316
    // The other $2,000 of premiums is a medical expense; the standard
    // deduction stays larger.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_retirement(
        &mut s,
        vec![item(
            Owner::Taxpayer,
            IncomeDocument::R1099(pension(3_000_000, officer(500_000, 400_000))),
        )],
    ));
    let id = s
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::Retirement { .. }))
        .unwrap()
        .id;
    assert_eq!(
        s.draft.screening.as_ref().unwrap().public_safety,
        vec![PublicSafetyPremium {
            income_id: id,
            premiums: 500_000,
            funded: 400_000,
            coverage: PublicSafetyCoverage::Medical,
        }]
    );
    let r = supported(&s.draft);
    assert_eq!(excluded(&r), 300_000);
    assert_eq!(return_line(&r, "5b"), 2_700_000);
    assert_eq!(return_line(&r, "11a / 11b"), 8_700_000);
    assert_eq!(return_line(&r, "15"), 7_090_000);
    assert_eq!(return_line(&r, "16"), 1_031_600);
    // Round trip: restarted, the 1099-R reopens with its premiums and an
    // unchanged save keeps the revision.
    let saved = s.draft.clone();
    let mut again = restart(saved.clone());
    let documents = retirement_documents(&mut again);
    let IncomeDocument::R1099(r) = &documents[0].document else { panic!("{documents:?}") };
    assert_eq!(r.public_safety, officer(500_000, 400_000));
    let request = retirement_page(&mut again);
    accepted(&send(&mut again, request));
    assert_eq!(again.draft, saved);
}

#[test]
fn public_safety_exclusion_is_capped_at_3000() {
    // Paid from the pension: $2,999.99 is all excluded; $3,000 is all
    // excluded; $3,000.01 is limited to $3,000.
    for (funded, expected) in [(299_999, 299_999), (300_000, 300_000), (300_001, 300_000)] {
        let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
        accepted(&save_retirement(
            &mut s,
            vec![item(
                Owner::Taxpayer,
                IncomeDocument::R1099(pension(3_000_000, officer(500_000, funded))),
            )],
        ));
        assert_eq!(excluded(&supported(&s.draft)), expected, "{funded}");
    }
}

#[test]
fn public_safety_answer_rules_refuse_on_the_server() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = s.draft.clone();
    // Not from an IRA (box 7 IRA/SEP/SIMPLE).
    let mut ira = pension(3_000_000, officer(500_000, 400_000));
    ira.ira = true;
    rejected_with(
        save_retirement(&mut s, vec![item(Owner::Taxpayer, IncomeDocument::R1099(ira))]),
        "PSO_NOT_FROM_AN_IRA",
    );
    // The part paid from the pension inside the premiums: $5,000.01 of
    // $5,000 is refused, $5,000 is accepted below.
    rejected_with(
        save_retirement(
            &mut s,
            vec![item(
                Owner::Taxpayer,
                IncomeDocument::R1099(pension(3_000_000, officer(500_000, 500_001))),
            )],
        ),
        "PSO_FUNDED_ABOVE_PREMIUMS",
    );
    // The parts together inside box 1.
    rejected_with(
        save_retirement(
            &mut s,
            vec![item(
                Owner::Taxpayer,
                IncomeDocument::R1099(pension(400_000, officer(500_000, 400_001))),
            )],
        ),
        "PSO_FUNDED_ABOVE_GROSS",
    );
    assert_eq!(s.draft, before);
    accepted(&save_retirement(
        &mut s,
        vec![item(
            Owner::Taxpayer,
            IncomeDocument::R1099(pension(3_000_000, officer(500_000, 500_000))),
        )],
    ));
}

#[test]
fn lump_sum_election_is_used_only_when_lower() {
    // 2024 AGI $30,000: Worksheet 2 line 6 = 4,000 + 30,000 = $34,000;
    //   10 = $9,000; 13 = $9,000; 14 = $4,500; 15 = min(4,000, 4,500) =
    //   $4,000; 17 = $4,000; 18 = $6,800; 19 = $4,000; 21 = $4,000.
    //   Worksheet 4 line 21 = 6,200 + 4,000 = $10,200, not smaller than
    //   $9,600: the regular $9,600 stays.
    let (mut s, documents) = lump_sum_return(3_000_000);
    accepted(&save_retirement(&mut s, documents));
    assert_eq!(return_line(&supported(&s.draft), "6b"), 960_000);
    // Without the kind the regular method applies: $9,600 too.
    let (mut s, mut documents) = lump_sum_return(1_000_000);
    documents.pop();
    accepted(&save_retirement(&mut s, documents));
    assert_eq!(s.draft.adjustments.as_ref().unwrap().social_election, SocialElection::Regular);
    assert_eq!(return_line(&supported(&s.draft), "6b"), 960_000);
}

#[test]
fn lump_sum_rules_refuse_on_the_server() {
    let (mut s, documents) = lump_sum_return(1_000_000);
    let before = s.draft.clone();
    let mut twice = documents.clone();
    twice.push(documents[1].clone());
    rejected_with(save_retirement(&mut s, twice), "LUMP_SUM_ELECTION_TWICE");
    let with_year = |f: &dyn Fn(&mut SocialBenefitYear)| {
        let mut documents = documents.clone();
        let IncomeDocument::SocialLumpSums(v) = &mut documents[1].document else { unreachable!() };
        f(&mut v.years[0]);
        documents
    };
    // Taxable benefits of the earlier return: $1 above benefits refused.
    rejected_with(
        save_retirement(
            &mut s,
            with_year(&|y| {
                y.benefits = 500_000;
                y.taxable = 500_001;
            }),
        ),
        "LUMP_SUM_TAXABLE_ABOVE_BENEFITS",
    );
    rejected_with(
        save_retirement(
            &mut s,
            with_year(&|y| {
                y.benefits = 500_000;
                y.earlier_lumps = 100_000;
                y.refigured = 600_001;
            }),
        ),
        "LUMP_SUM_REFIGURED_ABOVE_BENEFITS",
    );
    let mut two_years = documents.clone();
    let IncomeDocument::SocialLumpSums(v) = &mut two_years[1].document else { unreachable!() };
    v.years.push(v.years[0].clone());
    rejected_with(save_retirement(&mut s, two_years), "LUMP_SUM_YEAR_TWICE");
    assert_eq!(s.draft, before);
    // At the limits: taxable = benefits, refigured = benefits + lumps.
    accepted(&save_retirement(
        &mut s,
        with_year(&|y| {
            y.benefits = 500_000;
            y.earlier_lumps = 100_000;
            y.taxable = 500_000;
            y.refigured = 600_000;
        }),
    ));
}

// A missed required minimum from a plan administrator's figure.
fn missed(required: i64, timely: i64, correction: RmdCorrection) -> MissedDistribution {
    MissedDistribution {
        label: "Employer plan".into(),
        required: RmdMinimum::Administrator { amount: required },
        timely,
        correction,
        waiver: None,
    }
}
fn corrected(received: u32) -> RmdCorrection {
    RmdCorrection::Corrected {
        received,
        filed: RmdFiling::Planned { date: 20270415 },
        notice: None,
        assessed: None,
    }
}
fn rmd_tax(s: &mut Interview, value: MissedDistribution) -> i64 {
    accepted(&save_retirement(s, vec![item(Owner::Taxpayer, IncomeDocument::MissedRmd(value))]));
    let r = estimate(&s.draft);
    r.lines.iter().find(|l| l.id == "required_distribution_tax").map_or(0, |l| l.amount)
}

#[test]
fn missed_required_distribution_through_the_retirement_topic_by_hand() {
    // Form 5329 Part IX: required $10,000, taken by the deadline $4,000.
    //   Uncorrected: 52b - 53b = $6,000; line 54b = 25% = $1,500.
    //   Corrected by March 1, 2027 with the return planned April 15, 2027
    //   (inside the window ending December 31, 2028): line 54a = 10% x
    //   6,000 = $600.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    assert_eq!(rmd_tax(&mut s, missed(1_000_000, 400_000, RmdCorrection::Uncorrected)), 150_000);
    assert_eq!(rmd_tax(&mut s, missed(1_000_000, 400_000, corrected(20270301))), 60_000);
    // Round trip.
    let saved = s.draft.clone();
    let mut again = restart(saved.clone());
    let documents = retirement_documents(&mut again);
    assert_eq!(
        documents[0].document,
        IncomeDocument::MissedRmd(missed(1_000_000, 400_000, corrected(20270301)))
    );
    let request = retirement_page(&mut again);
    accepted(&send(&mut again, request));
    assert_eq!(again.draft, saved);
}

#[test]
fn missed_distribution_correction_window_ends_december_31_2028() {
    // Received December 31, 2028: 10% ($600); January 1, 2029: 25%.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let late = |received| RmdCorrection::Corrected {
        received,
        filed: RmdFiling::Planned { date: 20281231 },
        notice: None,
        assessed: None,
    };
    assert_eq!(rmd_tax(&mut s, missed(1_000_000, 400_000, late(20281231))), 60_000);
    assert_eq!(rmd_tax(&mut s, missed(1_000_000, 400_000, late(20290101))), 150_000);
    // Taken in full by the deadline: no shortfall, no tax.
    assert_eq!(rmd_tax(&mut s, missed(1_000_000, 1_000_000, RmdCorrection::Uncorrected)), 0);
}

#[test]
fn missed_distribution_waiver_and_owner_rules() {
    // A waiver of the whole $6,000 shortfall leaves no tax (pending IRS
    // approval).
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut value = missed(1_000_000, 400_000, RmdCorrection::Uncorrected);
    value.waiver =
        Some(RmdWaiver { amount: 600_000, explanation: "Custodian error, corrected".into() });
    assert_eq!(rmd_tax(&mut s, value), 0);
    // The spouse's shortfall on a single return is refused.
    let before = s.draft.clone();
    problem(current_document_refusal(
        "income-source-9524",
        &mut s,
        vec![item(
            Owner::Spouse,
            IncomeDocument::MissedRmd(missed(1_000_000, 0, RmdCorrection::Uncorrected)),
        )],
        "MESSAGE_MISSED_REQUIRED_DISTRIBUTION_YOUR_SPOUSE_S_GO_ON_THIS_RETURN_ONLY_WHEN_YOU_F",
        "[documents/documents/0/owner]",
    ));
    assert_eq!(s.draft, before);
}

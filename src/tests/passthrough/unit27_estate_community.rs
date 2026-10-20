use crate::tests::cross::interview::{accepted, document, open_topic, page_documents, rejected_with, save_topic};
use super::*;

// Synthetic source-composition fixture: the components are independent of
// the projection implementation; this is not a published IRS tax-total oracle.
pub(super) fn source() -> FederalEstateTrustPortfolioK1 {
    let mut seed = estate_trust_code_c_draft(20_000, 100_000);
    let data = estate_trust_code_d_data(&mut seed);
    data.interest = 100_000;
    data.ordinary_dividends = 40_000;
    data.qualified_dividends = 30_000;
    data.amt_box12a = -35_000;
    data.amt_box12b = -10_000;
    data.amt_box12c = -5_000;
    data.amt_box12d = -20_000;
    data.amt_box12_components_confirmed = true;
    data.investment_income = Some(crate::EstateInvestmentIncome {
        regular: 260_000,
        alternative: 225_000,
        character: crate::EstateInvestmentCharacter::Preferential {
            qualified: 30_000,
            gains: 120_000,
            amt_qualified: 20_000,
            amt_gains: 95_000,
        },
    });
    data.clone()
}

// Owner is the actual accepted source owner; switching it represents the
// counterpart return without changing any source monetary amount.
pub(super) fn draft(owner: Owner) -> FederalDraft {
    let mut draft = complete(FederalStatus::Separate);
    draft.setup.as_mut().unwrap().community_property = CommunityProperty::Yes {
        state: CommunityState::Texas,
        rule: CommunityRule::Halves,
    };
    draft.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    draft.income = vec![wage(1, Owner::Taxpayer, 6_000_000, 600_000)];
    draft.income[0].community = CommunityShare::Community;
    let mut entry = income(FederalIncome::EstateTrustPortfolio { data: source() });
    entry.id = 2;
    entry.label = "Synthetic componentized trust".into();
    entry.owner = owner;
    entry.community = CommunityShare::Separate;
    draft.income.push(entry);
    draft.next_id = 3;
    draft
}

fn projected(draft: &FederalDraft) -> FederalEstateTrustPortfolioK1 {
    let entry = crate::community::entry(draft, &draft.income[1]);
    let FederalIncome::EstateTrustPortfolio { data } = &entry.income else {
        unreachable!()
    };
    data.clone()
}
fn replace(draft: &mut FederalDraft, data: FederalEstateTrustPortfolioK1) {
    draft.income[1].income = FederalIncome::EstateTrustPortfolio { data };
}
fn halves(owner: Owner, data: FederalEstateTrustPortfolioK1) -> FederalDraft {
    let mut d = draft(owner);
    d.income[1].community = CommunityShare::Community;
    replace(&mut d, data);
    d
}

#[test]
fn unit27_estate_differential_share_reuses_actual_regular_and_amt_components() {
    let own_draft = draft(Owner::Taxpayer);
    let other_draft = draft(Owner::Spouse);
    let before = (own_draft.clone(), other_draft.clone());
    let own = projected(&own_draft);
    let other = projected(&other_draft);
    assert_eq!(
        [
            own.interest,
            own.ordinary_dividends,
            own.qualified_dividends,
            own.short_gain,
            own.long_gain
        ],
        [50_000, 20_000, 15_000, 20_000, 100_000]
    );
    assert_eq!(
        [
            own.amt_box12a,
            own.amt_box12b,
            own.amt_box12c,
            own.amt_box12d
        ],
        [-30_000, -5_000, -5_000, -20_000]
    );
    assert_eq!(
        [
            other.interest,
            other.ordinary_dividends,
            other.qualified_dividends,
            other.short_gain,
            other.long_gain
        ],
        [50_000, 20_000, 15_000, 0, 0]
    );
    assert_eq!(
        [
            other.amt_box12a,
            other.amt_box12b,
            other.amt_box12c,
            other.amt_box12d
        ],
        [-5_000, -5_000, 0, 0]
    );
    assert_eq!(
        crate::estate_trust::investment_adjustment(&own),
        Some([0, 0])
    );
    assert_eq!(
        crate::estate_trust::investment_adjustment(&other),
        Some([0, 0])
    );
    let a = own.investment_income.unwrap();
    let b = other.investment_income.unwrap();
    assert_eq!([a.regular, a.alternative], [190_000, 160_000]);
    assert_eq!([b.regular, b.alternative], [70_000, 65_000]);
    assert_eq!(
        [a.regular + b.regular, a.alternative + b.alternative],
        [260_000, 225_000]
    );
    assert_eq!((own_draft, other_draft), before);
}

#[test]
fn unit27_estate_odd_cent_special_categories_conserve_parent_and_counterpart() {
    let mut data = source();
    data.investment_income = None;
    data.short_gain = 0;
    data.long_gain = 2;
    data.rate_28_gain = 1;
    data.unrecaptured_gain = 1;
    data.amt_box12a = 2;
    data.amt_box12b = 0;
    data.amt_box12c = 0;
    data.amt_box12d = 2;
    data.amt_box12e = 1;
    data.amt_box12f = 1;
    data.amt_box12e_included_in_d = true;
    data.amt_box12f_included_in_d = true;
    let own = projected(&halves(Owner::Taxpayer, data.clone()));
    let other = projected(&halves(Owner::Spouse, data.clone()));
    // Fixed category order28%,1250,other: first tied cent belongs to28%.
    assert_eq!(
        [own.long_gain, own.rate_28_gain, own.unrecaptured_gain],
        [1, 1, 0]
    );
    assert_eq!(
        [other.long_gain, other.rate_28_gain, other.unrecaptured_gain],
        [1, 0, 1]
    );
    for allocated in [&own, &other] {
        assert!(allocated.rate_28_gain + allocated.unrecaptured_gain <= allocated.long_gain);
        assert_eq!(
            allocated.amt_box12a,
            allocated.amt_box12b + allocated.amt_box12c + allocated.amt_box12d
        );
        assert!(
            allocated.rate_28_gain
                + allocated.amt_box12f
                + allocated.unrecaptured_gain
                + allocated.amt_box12e
                <= allocated.long_gain + allocated.amt_box12d
        );
    }
    assert_eq!(
        [
            own.long_gain + other.long_gain,
            own.rate_28_gain + other.rate_28_gain,
            own.unrecaptured_gain + other.unrecaptured_gain,
            own.amt_box12d + other.amt_box12d
        ],
        [2, 1, 1, 2]
    );
}

#[test]
fn unit27_estate_signed_amt_underlying_loss_is_not_floored_to_zero() {
    let mut data = source();
    data.investment_income = None;
    data.amt_box12c = -20_003;
    data.amt_box12d = -100_002;
    data.amt_box12b = -10_001;
    data.amt_box12a = -130_006;
    data.amt_reconciliation = Some(crate::EstateAmtReconciliation {
        other_in_a: 0,
        adjustments: vec![],
    });
    let own = projected(&halves(Owner::Taxpayer, data.clone()));
    let other = projected(&halves(Owner::Spouse, data.clone()));
    assert_eq!(own.short_gain + own.amt_box12c, -2);
    assert_eq!(other.short_gain + other.amt_box12c, -1);
    assert_eq!(own.long_gain + own.amt_box12d, -1);
    assert_eq!(other.long_gain + other.amt_box12d, -1);
    assert_eq!(own.amt_box12a + other.amt_box12a, data.amt_box12a);
    assert_eq!(crate::estate_trust::source_rule(&own), None);
    assert_eq!(crate::estate_trust::source_rule(&other), None);
}

#[test]
fn unit27_estate_source_preflight_retains_unclassified_amt_and_niit_boundaries() {
    for (j, h, a, key) in [
        (Some(1), None, false, "ISSUE_ESTATE_AMT_SOURCE_UNAVAILABLE"),
        (
            None,
            Some(crate::EstateTrustNiitAdjustment {
                line_7: 1,
                modified_agi: 1,
            }),
            false,
            "ISSUE_ESTATE_COMMUNITY_NIIT_SOURCE_UNAVAILABLE",
        ),
        (None, None, true, "ISSUE_ESTATE_AMT_SOURCE_UNAVAILABLE"),
    ] {
        let mut d = draft(Owner::Taxpayer);
        let data = estate_trust_code_d_data(&mut d);
        data.minimum_tax_exclusion = j;
        data.niit_adjustment = h;
        if a {
            data.amt_box12a -= 1;
        }
        assert_eq!(
            crate::estate_trust::community_source_rule(&d, &d.income[1]),
            Some(key)
        );
        d.income[1].community = CommunityShare::Community;
        assert_eq!(
            crate::estate_trust::community_source_rule(&d, &d.income[1]),
            None
        );
    }
    let mut zero = draft(Owner::Taxpayer);
    let data = estate_trust_code_d_data(&mut zero);
    data.minimum_tax_exclusion = Some(0);
    data.niit_adjustment = Some(crate::EstateTrustNiitAdjustment {
        line_7: 0,
        modified_agi: 0,
    });
    assert_eq!(
        crate::estate_trust::community_source_rule(&zero, &zero.income[1]),
        None
    );
}

#[test]
fn unit27_estate_optional_mismatched_investment_character_does_not_change_income() {
    let mut data = source();
    let crate::EstateInvestmentCharacter::Preferential { qualified, .. } =
        &mut data.investment_income.as_mut().unwrap().character
    else {
        unreachable!()
    };
    *qualified += 1;
    let own = projected(&draft(Owner::Taxpayer));
    let unknown = projected(&{
        let mut d = draft(Owner::Taxpayer);
        replace(&mut d, data);
        d
    });
    assert_eq!(
        [
            unknown.interest,
            unknown.ordinary_dividends,
            unknown.short_gain,
            unknown.long_gain
        ],
        [
            own.interest,
            own.ordinary_dividends,
            own.short_gain,
            own.long_gain
        ]
    );
    assert_eq!(
        unknown.investment_income.as_ref().unwrap().character,
        crate::EstateInvestmentCharacter::Unavailable
    );
    assert_eq!(crate::estate_trust::investment_adjustment(&unknown), None);
}

#[test]
fn unit27_estate_normal_save_reopen_restart_preserves_whole_source_and_correction_id() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut seed = draft(owner);
        seed.income.pop();
        let mut session = Interview::new(TEST_USER, seed);
        let page = K1EstateTrust::from_record("Synthetic componentized trust", &source()).unwrap();
        let mut item = document(0, IncomeDocument::K1EstateTrust(page.clone()));
        item.owner = owner;
        item.community = CommunityShare::Separate;
        let request = save_topic(&session.draft, TaxSection::PassThrough, vec![item]);
        accepted(&send(&mut session, request));
        let stored = session.draft.clone();
        let saved = stored
            .income
            .iter()
            .find(|entry| matches!(entry.income, FederalIncome::EstateTrustPortfolio { .. }))
            .unwrap();
        let FederalIncome::EstateTrustPortfolio { data } = &saved.income else {
            unreachable!()
        };
        assert_eq!(data, &source());
        let reopened = open_topic(&mut session, TaxSection::PassThrough);
        assert_eq!(
            page_documents(&reopened)[0].document,
            IncomeDocument::K1EstateTrust(page)
        );
        assert_eq!(restart(stored.clone()).draft, stored);
        let mut correction = reopened;
        let P::SaveIncomes { documents, .. } = &mut correction else {
            unreachable!()
        };
        let IncomeDocument::K1EstateTrust(page) = &mut documents.documents[0].document else {
            unreachable!()
        };
        page.interest += 100;
        page.investment_income.as_mut().unwrap().regular += 100;
        page.investment_income.as_mut().unwrap().alternative += 100;
        accepted(&send(&mut session, correction));
        let corrected = session
            .draft
            .income
            .iter()
            .find(|entry| matches!(entry.income, FederalIncome::EstateTrustPortfolio { .. }))
            .unwrap();
        assert_eq!(corrected.id, saved.id);
        assert_eq!(corrected.owner, owner);
        let before = session.draft.clone();
        let mut invalid = open_topic(&mut session, TaxSection::PassThrough);
        let P::SaveIncomes { documents, .. } = &mut invalid else {
            unreachable!()
        };
        let IncomeDocument::K1EstateTrust(page) = &mut documents.documents[0].document else {
            unreachable!()
        };
        page.amt_reconciliation = Some(crate::EstateAmtReconciliationPage {
            other_in_a: 1,
            adjustments: vec![],
        });
        rejected_with(send(&mut session, invalid), "ESTATE_AMT_RECONCILE");
        assert_eq!(session.draft, before);
    }
}

fn final_capital() -> crate::EstateFinalCapital {
    crate::EstateFinalCapital {
        alternative_short: 1,
        alternative_long: 2,
        rate28: 1,
        unrecaptured: 1,
        niit_difference: Some(3),
        regular_investment: Some(crate::EstateCapitalLossParts { short: 1, long: 2 }),
        alternative_investment: Some(crate::EstateCapitalLossParts { short: 1, long: 2 }),
        regular_business: None,
        alternative_business: None,
    }
}

#[test]
fn unit27_estate_final_capital_parts_use_gain_axis_and_preserve_odd_cent_caps() {
    let mut data = source();
    data.final_short_carry = 1;
    data.final_long_carry = 2;
    data.final_character = Some(crate::EstateFinalCharacter {
        entity_terminated: true,
        final_beneficiary_k1: true,
        successor: true,
        section67e_nii: 0,
        section67e_in_h: false,
        additional: vec![],
        capital: final_capital(),
    });
    assert_eq!(crate::estate_trust::source_rule(&data), None);
    let own = projected(&halves(Owner::Taxpayer, data.clone()));
    let other = projected(&halves(Owner::Spouse, data.clone()));
    let own_c = &own.final_character.as_ref().unwrap().capital;
    let other_c = &other.final_character.as_ref().unwrap().capital;
    assert_eq!(
        [
            own.final_short_carry,
            own.final_long_carry,
            own_c.rate28,
            own_c.unrecaptured
        ],
        [1, 1, 1, 0]
    );
    assert_eq!(
        [
            other.final_short_carry,
            other.final_long_carry,
            other_c.rate28,
            other_c.unrecaptured
        ],
        [0, 1, 0, 1]
    );
    assert_eq!(own_c.niit_difference, Some(2));
    assert_eq!(other_c.niit_difference, Some(1));
    for projected in [&own, &other] {
        assert_eq!(crate::estate_trust::source_rule(projected), None);
    }
    assert_eq!(own_c.alternative_investment.as_ref().unwrap().short, 1);
    assert_eq!(other_c.alternative_investment.as_ref().unwrap().short, 0);
    // An actual partial aggregate with unavailable short/long attribution
    // retains only its existing optional NIIT benefit boundary if it no longer fits.
    data.final_short_carry = 2;
    data.final_long_carry = 2;
    let c = &mut data.final_character.as_mut().unwrap().capital;
    c.rate28 = 0;
    c.unrecaptured = 0;
    c.niit_difference = Some(3);
    c.regular_investment = None;
    assert_eq!(crate::estate_trust::source_rule(&data), None);
    let mut partial = halves(Owner::Spouse, data);
    partial.income[1].community = CommunityShare::Partly { percent: 48 };
    let other = projected(&partial);
    assert_eq!([other.final_short_carry, other.final_long_carry], [0, 0]);
    assert_eq!(other.final_character.unwrap().capital.niit_difference, None);
}

fn activity(income: i64, depreciation: i64, carried: i64) -> crate::EstateActivity {
    crate::EstateActivity {
        id: 3,
        name: "Synthetic business".into(),
        kind: crate::EstateActivityKind::Business,
        income,
        depreciation,
        depletion: 0,
        amortization: 0,
        participation: crate::EstateParticipation::Passive,
        carried: crate::EstatePassiveCarry {
            regular: carried,
            alternative: carried,
        },
        amt_difference: 0,
    }
}
fn qbi_statement(
    name: &str,
    income: i64,
    history: Vec<crate::ScorpLossYear>,
) -> crate::EstateQbiStatement {
    crate::EstateQbiStatement {
        name: name.into(),
        income,
        wages: 0,
        property: 0,
        service: false,
        kind: crate::EstateQbiKind::Business,
        activities: vec![3],
        passive_history: vec![],
        allocations: vec![crate::EstateQbiAllocation {
            activity: 3,
            income,
            passive_history: history,
        }],
    }
}

#[test]
fn unit27_estate_shared_qbi_history_uses_existing_name_prefix_before_limits() {
    let mut data = source();
    data.activities.box6 = 0;
    data.activities.activities = vec![activity(0, 0, 2)];
    data.qbi.statements = ["First", "Second"]
        .into_iter()
        .map(|name| {
            qbi_statement(
                name,
                0,
                vec![crate::ScorpLossYear {
                    year: 2025,
                    original: 2,
                    qualified: 1,
                    remaining: 2,
                    remaining_qualified: 1,
                }],
            )
        })
        .collect();
    assert_eq!(data.qbi.rule(&data.activities), None);
    let own = projected(&halves(Owner::Taxpayer, data.clone()));
    let other = projected(&halves(Owner::Spouse, data.clone()));
    for allocation in [&own, &other] {
        assert_eq!(allocation.qbi.rule(&allocation.activities), None);
        let qualified: i64 = allocation
            .qbi
            .statements
            .iter()
            .map(|s| s.allocations[0].passive_history[0].qualified)
            .sum();
        assert_eq!(qualified, 1);
    }
    assert_eq!(
        own.qbi.statements[0].allocations[0].passive_history[0].qualified,
        1
    );
    assert_eq!(
        own.qbi.statements[1].allocations[0].passive_history[0].qualified,
        0
    );
    data.qbi.statements.reverse();
    let reordered = projected(&halves(Owner::Taxpayer, data));
    for statement in &own.qbi.statements {
        assert_eq!(
            Some(statement),
            reordered
                .qbi
                .statements
                .iter()
                .find(|s| s.name == statement.name)
        );
    }
}

#[test]
fn unit27_estate_optional_qbi_odd_cent_boundary_preserves_actual_source_components() {
    let mut data = source();
    data.activities.box6 = -1;
    data.activities.activities = vec![activity(-1, 1, 0)];
    data.qbi.statements = vec![qbi_statement("Entire qualified loss", -2, vec![])];
    assert_eq!(data.qbi.rule(&data.activities), None);
    let original = data.clone();
    let own = projected(&halves(Owner::Taxpayer, data.clone()));
    let other = projected(&halves(Owner::Spouse, data.clone()));
    assert_eq!(
        [
            own.activities.activities[0].income,
            own.activities.activities[0].depreciation
        ],
        [-1, 1]
    );
    assert_eq!(
        [
            other.activities.activities[0].income,
            other.activities.activities[0].depreciation
        ],
        [0, 0]
    );
    assert_eq!(
        crate::estate_trust::qbi_projection(
            2,
            &other,
            &other.qbi.statements[0],
            &[],
            0,
            FederalStatus::Separate
        )
        .err(),
        Some("WARN_ESTATE_QBI_SOURCE_LEFT_OUT")
    );
    assert_eq!(
        own.activities.box6 + other.activities.box6,
        original.activities.box6
    );
    assert_eq!(
        own.activities.activities[0].depreciation + other.activities.activities[0].depreciation,
        1
    );
    assert_eq!(data, original);
}

#[test]
fn unit27_estate_raw_nonzero_amt_confirmations_remain_valid_when_projection_is_zero() {
    let mut data = source();
    data.investment_income = None;
    data.interest = 0;
    data.ordinary_dividends = 0;
    data.qualified_dividends = 0;
    data.short_gain = 0;
    data.long_gain = 2;
    data.rate_28_gain = 2;
    data.amt_box12a = -1;
    data.amt_box12b = 0;
    data.amt_box12c = 0;
    data.amt_box12d = -1;
    data.amt_box12f = -1;
    data.amt_box12f_included_in_d = true;
    let d = halves(Owner::Taxpayer, data.clone());
    let allocated = projected(&d);
    assert_eq!(
        [
            allocated.amt_box12a,
            allocated.amt_box12d,
            allocated.amt_box12f
        ],
        [0; 3]
    );
    assert!(allocated.amt_box12_components_confirmed && allocated.amt_box12f_included_in_d);
    let _ = supported(&d);
    assert_eq!(estate_trust_code_d_data(&mut d.clone()), &data);
}

#[test]
fn unit27_estate_zero_projected_qbi_vintage_is_omitted_without_changing_retained_source() {
    let mut data = source();
    data.activities.activities = vec![activity(0, 0, 1)];
    data.qbi.statements = vec![qbi_statement(
        "Entire qualified vintage",
        0,
        vec![crate::ScorpLossYear {
            year: 2025,
            original: 1,
            qualified: 1,
            remaining: 1,
            remaining_qualified: 1,
        }],
    )];
    let original = data.clone();
    let d = halves(Owner::Spouse, data);
    let allocated = projected(&d);
    assert!(
        allocated.qbi.statements[0].allocations[0]
            .passive_history
            .is_empty()
    );
    assert_eq!(allocated.qbi.rule(&allocated.activities), None);
    let FederalIncome::EstateTrustPortfolio { data } = &d.income[1].income else {
        unreachable!()
    };
    assert_eq!(data, &original);
}

#[test]
fn unit27_estate_page_relationships_reject_overflowing_components_at_the_original_field() {
    let mut page = crate::K1EstateTrust::from_record("Synthetic trust", &source()).unwrap();
    page.ordinary_dividends = i64::MAX;
    page.qualified_dividends = i64::MAX;
    page.more.section199a_dividends = 1;
    let issue = page.issue().unwrap();
    assert_eq!(issue.key, "K1_1041_QUALIFIED_ABOVE_2A");
    assert_eq!(issue.field.unwrap().square(), "[qualified_dividends]");

    page.qualified_dividends = 0;
    page.more.section199a_dividends = 0;
    page.long_gain = i64::MAX;
    page.rate_28_gain = i64::MAX;
    page.unrecaptured_gain = 1;
    let issue = page.issue().unwrap();
    assert_eq!(issue.key, "K1_1041_4B_4C_ABOVE_4A");
    assert_eq!(issue.field.unwrap().square(), "[rate_28_gain]");
}

#[test]
fn unit27_estate_annual_vintage_keeps_dollar_release_and_complement_quantum() {
    let mut data = source();
    data.activities.box6 = 0;
    data.activities.activities = vec![activity(0, 0, 200)];
    data.qbi.statements = vec![qbi_statement("Half-qualified annual vintage", 0,
        vec![crate::ScorpLossYear { year: 2025, original: 200, qualified: 100,
            remaining: 200, remaining_qualified: 100 }])];
    let original = data.clone();
    let result = crate::estate_trust::qbi_projection(2, &data, &data.qbi.statements[0],
        &[crate::estate_trust::EstateActivityResult { source: 2, activity: 3,
            current: 0, prior: 2, passive: -2, nonpassive: 0,
            suspended: 1, prior_released: 0 }], 0, FederalStatus::Single).unwrap();
    // The estate has already reported $2 loss/$1 qualified. Its owning FIFO
    // releases $1 qualified, leaving $1 loss with no qualified complement.
    assert_eq!(result.historical, 1);
    assert_eq!((result.remaining[0].remaining.get(), result.remaining[0].remaining_qbi.get()), (1, 0));
    assert_eq!(data, original);
}

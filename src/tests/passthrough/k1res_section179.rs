use crate::k1_activity::*;
use crate::tests::cross::interview::{accepted, open_topic};
use crate::tests::support::assert::k1res::assert_located_rejection;
use crate::tests::support::builders::k1res::{
    before_section179, documents, entity_exempt_other_fields, input,
};
use crate::tests::support::client::income_pages::catalog_preview_document;
use crate::tests::support::client::k1res::{
    assert_native_review, section179_direct, section179_session,
};
use crate::tests::*;
use alloc::format;

#[test]
fn native_and_direct_share_business_income_limit_with_activity_cent_carry() {
    for scorp in [false, true] {
        let mut session = section179_session(scorp, [0, 200_001], [0, 200_001]);
        let direct = section179_direct(&mut session, 100_000, 400_002);
        let selected =
            crate::section179_return::prepare(&session.draft, Section179IncomeSystem::Regular)
                .unwrap();
        let source = crate::k1_activity::source(&session.draft.income[0].income).unwrap();
        let activity = source.components[1].id;
        // $6,000.03 elections divide the $3,000 income in an exact 2:1 ratio.
        assert_eq!(selected.allocation.income.business_income_limit_cents, 300_000);
        assert_eq!(selected.allocation.allowed(direct).unwrap(), 200_000);
        assert_eq!(selected.allocation.native_allowed(activity), 100_000);
        let output = before_section179(&session.draft.income[0].income, false)
            .unwrap()
            .apply_section179(Some(&selected.allocation))
            .unwrap();
        assert_eq!(
            (output.components[1].section179_allowed, output.components[1].section179_carry),
            (100_000, 100_001)
        );
        assert_eq!(output.components[0].section179_carry, 0);
        assert_eq!(output.groups[0].nonpassive, 100_000);
        assert_eq!(output.groups[1].nonpassive, 0);
        assert!(estimate(&session.draft).tax.is_some());
    }
}

#[test]
fn native_and_direct_share_dollar_limit_and_preserve_over_limit_boundary() {
    let mut session = section179_session(false, [0, 128_000_000], [0, 128_000_000]);
    section179_direct(&mut session, 400_000_000, 128_000_000);
    let selected =
        crate::section179_return::prepare(&session.draft, Section179IncomeSystem::Regular).unwrap();
    assert_eq!(selected.allocation.income.total_allowed_cents, 256_000_000);
    let mut income = session.draft.income[0].income.clone();
    let (source, current, alternative) = entity_exempt_other_fields(&mut income);
    source.components[1].regular.as_mut().unwrap().section179 += 1;
    current.section179 += 1;
    *alternative = Some(current.clone());
    source.components[1].alternative = K1StatementAmt::SameAsRegular;
    session.draft.income[0].income = income;
    let selected =
        crate::section179_return::prepare(&session.draft, Section179IncomeSystem::Regular).unwrap();
    let mut result = FederalResult::empty(session.draft.revision);
    selected.report(&mut result).unwrap();
    assert!(result.issues.iter().any(|issue| issue.display.key
        == "ISSUE_THE_ASSET_ELECTIONS_AND_SCHEDULE_K_1_SECTION_179_AMOUNTS_EXCEED_THE_CE43F95A"));
}

#[test]
fn native_section179_keeps_whole_parent_basis_and_reg_amt_independent() {
    for scorp in [false, true] {
        let mut session = section179_session(scorp, [0, 100_000], [0, 100_000]);
        let income = &mut session.draft.income[0].income;
        match income {
            FederalIncome::PartnershipActivities { data } => {
                data.entity.regular.opening = Some(100_000);
                data.entity.alternative.as_mut().unwrap().opening = Some(200_000);
            }
            FederalIncome::ScorpActivities { data } => {
                data.entity.regular.opening = Some(100_000);
                data.entity.alternative.as_mut().unwrap().opening = Some(200_000);
            }
            _ => unreachable!(),
        }
        let (source, current, alternative) = entity_exempt_other_fields(income);
        source.components[0].regular.as_mut().unwrap().operating = -100_000;
        source.components[1].regular.as_mut().unwrap().operating = 0;
        for component in &mut source.components {
            component.alternative = K1StatementAmt::SameAsRegular;
        }
        current.ordinary = -100_000;
        *alternative = Some(current.clone());
        let regular = before_section179(income, false).unwrap();
        let amt = before_section179(income, true).unwrap();
        assert_eq!(regular.basis.ending.get(), 0);
        assert_eq!(regular.components[0].basis_allowed.get(), 50_000);
        assert_eq!(
            (regular.components[1].section179, regular.components[1].section179_basis_suspended),
            (50_000, 50_000)
        );
        assert_eq!(amt.components[0].basis_allowed.get(), 100_000);
        assert_eq!(
            (amt.components[1].section179, amt.components[1].section179_basis_suspended),
            (100_000, 0)
        );
    }
}

#[test]
fn native_section179_preview_save_review_reopen_cold_correction_keep_ids() {
    for scorp in [false, true] {
        let mut session = section179_session(scorp, [0, 200_001], [0, 100_001]);
        let parent = session.draft.income[0].id;
        let ids = crate::k1_activity::source_ids(&session.draft.income[0].income);
        let before = session.draft.clone();
        let dirty = session.dirty;
        let mut source_preview = catalog_preview_document(input(&before.income[0].income));
        let P::PreviewDocument { id, .. } = &mut source_preview else { unreachable!() };
        *id = parent;
        assert!(matches!(
            send(&mut session, source_preview),
            Reply::Response(P::IncomePreview { preview: Some(0), .. })
        ));
        let request = open_topic(&mut session, TaxSection::PassThrough);
        let preview = crate::tests::support::client::estimate_documents_from_save(&request);
        assert!(matches!(
            send(&mut session, preview),
            Reply::Response(P::IncomeRunningEstimate { .. })
        ));
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
        accepted(&send(&mut session, request));
        let regular =
            crate::section179_return::prepare(&session.draft, Section179IncomeSystem::Regular)
                .unwrap();
        let alternative =
            crate::section179_return::prepare(&session.draft, Section179IncomeSystem::Alternative)
                .unwrap();
        let activity =
            crate::k1_activity::source(&session.draft.income[0].income).unwrap().components[1].id;
        assert_eq!(
            (
                regular.allocation.native_allowed(activity),
                alternative.allocation.native_allowed(activity)
            ),
            (200_000, 100_001)
        );
        let result = estimate(&session.draft);
        assert!(result.tax.is_some(), "{:?}", result.issues);
        let kind = if scorp { "scorp" } else { "partnership" };
        assert_eq!(line_amount(&result, &format!("{kind}_{parent}_schedule_e")), 0);
        assert_eq!(line_amount(&result, &format!("amt_{kind}_{parent}_schedule_e")), 100_000);
        assert_native_review(&mut session);
        let mut cold = restart(session.draft.clone());
        let mut request = open_topic(&mut cold, TaxSection::PassThrough);
        assert_eq!(documents(&mut request)[0].document, input(&session.draft.income[0].income));
        let mut corrected = cold.draft.income[0].income.clone();
        let (source, current, _) = entity_exempt_other_fields(&mut corrected);
        source.components[1].regular.as_mut().unwrap().section179 = 50_001;
        current.section179 = 50_001;
        documents(&mut request)[0].document = input(&corrected);
        accepted(&send(&mut cold, request));
        assert_eq!(cold.draft.next_id, before.next_id);
        assert_eq!(crate::k1_activity::source_ids(&cold.draft.income[0].income), ids);
        assert_eq!(
            line_amount(&estimate(&cold.draft), &format!("{kind}_{parent}_schedule_e")),
            150_000
        );
        let mut source_preview = catalog_preview_document(input(&cold.draft.income[0].income));
        let P::PreviewDocument { id, .. } = &mut source_preview else { unreachable!() };
        *id = parent;
        assert!(matches!(
            send(&mut cold, source_preview),
            Reply::Response(P::IncomePreview { preview: Some(150_000), .. })
        ));
        assert_native_review(&mut cold);
    }
}

#[test]
fn native_section179_invalid_value_is_located_and_atomic() {
    let mut session = section179_session(false, [0, 100_001], [0, 100_001]);
    let mut income = session.draft.income[0].income.clone();
    let (source, _, _) = entity_exempt_other_fields(&mut income);
    source.components[1].regular.as_mut().unwrap().section179 = -1;
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    documents(&mut request)[0].document = input(&income);
    assert_located_rejection(
        &mut session,
        request,
        "K1_ACTIVITY_SOURCE_AMOUNT_INVALID",
        "[documents/documents/0/document/activities/statement/source/components/1/regular/section179]",
    );
}

#[test]
fn native_section179_requires_separate_spouse_allocation() {
    for allocation in [
        Section179Spouse::NotSupplied,
        Section179Spouse::Records { cost: 0, allocation: Section179Share::Equal, eligible: false },
    ] {
        let mut session = section179_session(false, [0, 100_001], [0, 100_001]);
        session.draft.setup.as_mut().unwrap().status =
            FilingChoice::Separate { section179: allocation };
        let selected =
            crate::section179_return::prepare(&session.draft, Section179IncomeSystem::Regular)
                .unwrap();
        let mut result = FederalResult::empty(session.draft.revision);
        selected.report(&mut result).unwrap();
        assert!(result.issues.iter().any(|issue| issue.display.key == "ISSUE_MARRIED_SEPARATE_SECTION_179_NEEDS_THE_SPOUSE_S_ELIGIBLE_PURCHASES_C6523EBC"));
        session.draft.setup.as_mut().unwrap().status = FilingChoice::Separate {
            section179: Section179Spouse::Records {
                cost: 0,
                allocation: Section179Share::Equal,
                eligible: true,
            },
        };
        let selected =
            crate::section179_return::prepare(&session.draft, Section179IncomeSystem::Regular)
                .unwrap();
        let mut result = FederalResult::empty(session.draft.revision);
        selected.report(&mut result).unwrap();
        assert!(!result.issues.iter().any(|issue| issue.display.key == "ISSUE_MARRIED_SEPARATE_SECTION_179_NEEDS_THE_SPOUSE_S_ELIGIBLE_PURCHASES_C6523EBC"));
    }
}

#[test]
fn native_section179_actual_groups_keep_passive_income_and_risk_boundary() {
    let mut session = section179_session(false, [0, 200_001], [0, 200_001]);
    let income = &mut session.draft.income[0].income;
    let (source, _, _) = entity_exempt_other_fields(income);
    let K1ActivityParticipation::Partnership { facts, .. } =
        &mut source.passive_groups[0].participation
    else {
        unreachable!()
    };
    *facts = PartnerParticipation::Limited {
        minutes: 0,
        prior_ten: 0,
        personal_service: false,
        service_years: 0,
    };
    let selected =
        crate::section179_return::prepare(&session.draft, Section179IncomeSystem::Regular).unwrap();
    let output = before_section179(&session.draft.income[0].income, false)
        .unwrap()
        .apply_section179(Some(&selected.allocation))
        .unwrap();
    assert_eq!((output.groups[0].passive, output.groups[1].nonpassive), (100_000, 0));
    assert_eq!(output.components[1].section179_carry, 100_001);
    // Actual group B has no remaining risk; group A's capacity is unrelated.
    let income = &mut session.draft.income[0].income;
    let (source, current, alternative) = entity_exempt_other_fields(income);
    source.components[1].regular.as_mut().unwrap().operating = 0;
    source.components[1].alternative = K1StatementAmt::SameAsRegular;
    current.ordinary = 100_000;
    *alternative = Some(current.clone());
    let K1RiskSource::Partnership { regular, alternative } = &mut source.risk_groups[1].source
    else {
        unreachable!()
    };
    regular.start = ScorpRiskStart::Detailed { capacity: 0, profit: 0, loss: 0 };
    *alternative = Some(regular.clone());
    add(&mut session.draft, wage(0, Owner::Taxpayer, 25_000_000, 0));
    let selected =
        crate::section179_return::prepare(&session.draft, Section179IncomeSystem::Regular).unwrap();
    assert_eq!(
        before_section179(&session.draft.income[0].income, false)
            .unwrap()
            .apply_section179(Some(&selected.allocation))
            .unwrap_err()
            .0
            .key,
        crate::k1_activity::ordinary::UNAVAILABLE.0.key
    );
    let result = estimate(&session.draft);
    assert!(result.tax.is_none());
    assert_eq!(
        result
            .issues
            .iter()
            .filter(|issue| issue.display.key == crate::k1_activity::ordinary::UNAVAILABLE.0.key)
            .count(),
        1
    );
}

#[test]
fn native_section179_cross_parent_output_overflow_is_located_without_raw_cap() {
    use crate::tests::support::builders::k1res::section179_huge_source;
    use crate::tests::support::client::k1res::section179_second_parent;
    let mut session = section179_session(false, [0, 100_000], [0, 100_000]);
    section179_second_parent(&mut session);
    // Huge source boxes are legal while basis suspends almost all of each.
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    for (index, row) in documents(&mut request).iter_mut().enumerate() {
        let mut income = session.draft.income[index].income.clone();
        section179_huge_source(&mut income, 100_000);
        row.document = input(&income);
    }
    accepted(&send(&mut session, request));
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    for (index, row) in documents(&mut request).iter_mut().enumerate() {
        let mut income = session.draft.income[index].income.clone();
        section179_huge_source(&mut income, i64::MAX / 2 + 1);
        row.document = input(&income);
    }
    assert_located_rejection(
        &mut session,
        request,
        crate::k1_activity::ordinary::RANGE_KEY,
        "[documents/documents]",
    );
}

#[test]
fn native_section179_finalization_issues_keep_each_parent_once_across_tax_systems() {
    use crate::tests::support::client::k1res::section179_second_parent;
    let mut session = section179_session(false, [0, 100_000], [0, 100_000]);
    section179_second_parent(&mut session);
    for entry in &mut session.draft.income {
        let (actual_source, current, alternative) = entity_exempt_other_fields(&mut entry.income);
        actual_source.components[1].regular.as_mut().unwrap().operating = 0;
        actual_source.components[1].alternative = K1StatementAmt::SameAsRegular;
        current.ordinary = 100_000;
        *alternative = Some(current.clone());
        let K1RiskSource::Partnership { regular, alternative } =
            &mut actual_source.risk_groups[1].source
        else {
            unreachable!()
        };
        regular.start = ScorpRiskStart::Detailed { capacity: 0, profit: 0, loss: 0 };
        *alternative = Some(regular.clone());
    }
    let result = estimate(&session.draft);
    assert!(result.tax.is_none());
    let issues: Vec<_> = result
        .issues
        .iter()
        .filter(|issue| issue.display.key == crate::k1_activity::ordinary::UNAVAILABLE.0.key)
        .collect();
    assert_eq!(issues.len(), 2);
    assert_ne!(issues[0].context.source(), issues[1].context.source());
}

// IRS i8582, Special Instructions for PTPs: same-PTP offsets only, no
// Form 8582 or rental allowance, and retain each allowed form character.
use super::*;
use crate::tests::cross::interview::{document, open_document_center};
use crate::tests::support::client::current_source_acceptance;
use alloc::format;

#[test]
fn ptp_rental_opposite_signs_and_both_boxes_retain_own_allowed_amounts() {
    for (ordinary, estate, other, estate_allowed, held, agi) in [
        (1_000_000, -1_500_000, 200_000, -12_000, 300_000, 60_000),
        (-1_000_000, 1_500_000, -200_000, 15_000, 0, 63_000),
        (0, -500_000, 500_000, -5_000, 0, 60_000),
    ] {
        let mut source = ptp_rental_source(ordinary, estate, other);
        source.rental_facts.participation = Some(K1RentalParticipation::Active);
        let draft = ptp_rental_draft(source);
        let result = supported(&draft);
        assert_eq!(value(&result, "agi"), agi);
        assert_eq!(value(&result, "partnership_2_rental_real_estate_net"), estate_allowed);
        assert_eq!(value(&result, "partnership_2_other_rental_net"), other / 100);
        let mut scratch = FederalResult::empty(draft.revision);
        let pool = crate::extended_rules::rentals_for(&draft, 60_000, &mut scratch, false);
        assert_eq!(pool.special_allowance, 0);
        assert_eq!(pool.net_passive_income, 0);
        assert_eq!(
            pool.k1_ledgers[0].1.passive_suspended[K1LossClass::RentalRealEstate].get(),
            held
        );
        assert_eq!(pool.k1_amounts[0].source_net_cents, Some((agi - 60_000) * 100));
    }
}

#[test]
fn ptp_rental_prior_losses_and_amt_sources_stay_distinct() {
    let mut source = ptp_rental_source(1_000_000, -1_200_000, 500_000);
    source.regular.rentals.real_estate_carryover = 300_000;
    source.regular.rentals.other_carryover = 100_000;
    let mut alternative = source.regular.clone();
    alternative.ordinary = 600_000;
    alternative.rentals.real_estate = -800_000;
    alternative.rentals.real_estate_carryover = 200_000;
    alternative.rentals.other = 100_000;
    alternative.rentals.other_carryover = 0;
    source.alternative = Some(alternative);
    let draft = ptp_rental_draft(source);
    let result = supported(&draft);
    assert_eq!(value(&result, "agi"), 60_000);
    for (prefix, net, held, other) in [("", -14_000, 1_000, 4_000), ("amt_", -7_000, 3_000, 1_000)]
    {
        assert_eq!(value(&result, &format!("{prefix}partnership_2_rental_real_estate_net")), net);
        assert_eq!(
            value(&result, &format!("{prefix}partnership_2_rental_real_estate_suspended")),
            held
        );
        assert_eq!(value(&result, &format!("{prefix}partnership_2_other_rental_net")), other);
    }
}

#[test]
fn ptp_rental_basis_and_at_risk_suspensions_precede_own_pal() {
    for (basis, risk, basis_held, risk_held, pal_held) in
        [(400_000, 1_000_000, 600_000, 0, 400_000), (1_000_000, 400_000, 0, 600_000, 400_000)]
    {
        let mut source = ptp_rental_source(0, -1_000_000, 0);
        source.regular.basis.opening = Some(basis);
        source.regular.risk.start = ScorpRiskStart::Detailed { capacity: risk, profit: 0, loss: 0 };
        source.alternative = Some(source.regular.clone());
        let draft = ptp_rental_draft(source);
        let result = supported(&draft);
        assert_eq!(value(&result, "agi"), 60_000);
        let mut scratch = FederalResult::empty(draft.revision);
        let pool = crate::extended_rules::rentals_for(&draft, 60_000, &mut scratch, false);
        let ledger = &pool.k1_ledgers[0].1;
        let class = K1LossClass::RentalRealEstate;
        assert_eq!(ledger.basis_suspended[class].get(), basis_held);
        assert_eq!(ledger.risk_suspended[class].get(), risk_held);
        assert_eq!(ledger.passive_suspended[class].get(), pal_held);
    }
}

#[test]
fn ptp_rental_cannot_offset_another_ptp_or_common_pool() {
    let mut draft = ptp_rental_draft(ptp_rental_source(0, -1_000_000, 0));
    add(&mut draft, income(FederalIncome::Partnership { data: ptp_rental_source(0, 400_000, 0) }));
    let mut common = ptp_rental_source(0, 600_000, 0);
    common.publicly_traded = false;
    add(&mut draft, income(FederalIncome::Partnership { data: common }));
    let result = supported(&draft);
    assert_eq!(value(&result, "agi"), 70_000);
    assert_eq!(value(&result, "partnership_2_rental_real_estate_suspended"), 10_000);
    let mut scratch = FederalResult::empty(draft.revision);
    let pool = crate::extended_rules::rentals_for(&draft, 60_000, &mut scratch, false);
    assert_eq!(pool.ptp_income, 4_000);
    assert_eq!(pool.net_passive_income, 6_000);
    assert_eq!(pool.income_by_id[&2], 0);
    assert_eq!(pool.income_by_id[&3], 4_000);
    assert_eq!(pool.income_by_id[&4], 6_000);
}

#[test]
fn ptp_rental_source_cents_survive_loss_allocation() {
    let draft = ptp_rental_draft(ptp_rental_source(101, -303, 101));
    let selected =
        crate::section179_return::prepare(&draft, Section179IncomeSystem::Regular).unwrap();
    let source = &selected.ptp[&2];
    assert_eq!((source.net_gain_cents, source.suspended_cents), (0, 101));
    let rental = source
        .components
        .iter()
        .find(|part| part.original.character == K1LossClass::RentalRealEstate)
        .unwrap();
    assert_eq!((rental.allowed_passive_net_cents, rental.suspended_cents), (-202, 101));
    let result = supported(&draft);
    assert_eq!(value(&result, "partnership_2_rental_real_estate_net"), -2);
    assert_eq!(value(&result, "partnership_2_other_rental_net"), 1);
    assert_eq!(value(&result, "agi"), 60_000);
}

#[test]
fn ptp_rental_uses_checked_section179_supplier_once() {
    let mut source = ptp_rental_source(2_000_000, -500_000, 0);
    source.section179 = 300_000;
    source.amt_section179_income_carry =
        Some(Section179IncomeCarryHistory { gross: 0, sold: 0, vintages: vec![] });
    let draft = ptp_rental_draft(source);
    for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
        let selected = crate::section179_return::prepare(&draft, system).unwrap();
        let ptp = &selected.ptp[&2];
        assert_eq!(ptp.net_gain_cents, 1_200_000);
        assert_eq!(ptp.suspended_cents, 0);
        let mut report = FederalResult::empty(draft.revision);
        selected.report(&mut report).unwrap();
        assert!(
            !report
                .issues
                .iter()
                .any(|issue| issue.display.key == "ISSUE_SECTION179_PUBLICLY_TRADED")
        );
    }
    let result = supported(&draft);
    assert_eq!(value(&result, "agi"), 72_000);
    assert_eq!(value(&result, "partnership_2_rental_real_estate_net"), -5_000);
}

#[test]
fn ptp_rental_residual_character_boundaries_are_located() {
    for (case, key) in [
        (0, "ISSUE_K1_RENTAL_PTP_PROPERTY"),
        (1, "ISSUE_K1_RENTAL_PTP_PARTICIPATION"),
        (2, "ISSUE_K1_RENTAL_PTP_DISPOSITION"),
        (3, "ISSUE_K1_RENTAL_PTP_QBI"),
    ] {
        let mut source = ptp_rental_source(0, 100_000, 0);
        match case {
            0 => source.regular.rentals.real_estate_property.net = 100,
            1 => source.rental_facts.participation = Some(K1RentalParticipation::Professional),
            2 => source.rental_facts.disposed = vec![K1RentalActivity::RealEstate],
            _ => source.rental_facts.real_estate_qbi = Some(K1RentalQbi { wages: 0, property: 0 }),
        }
        source.alternative = Some(source.regular.clone());
        let result = estimate(&ptp_rental_draft(source));
        assert_eq!(result.state, ResultState::Unsupported);
        let issue = result.issues.iter().find(|issue| issue.display.key == key).unwrap();
        assert!(format!("{:?}", issue.context).contains("2"));
    }
}

#[test]
fn ptp_rental_native_preview_save_correction_reopen_and_cold_restart() {
    let source = ptp_rental_source(1_000_000, -500_000, 200_000);
    let mut page = K1PartnershipPaper::from_record("Synthetic PTP rental", &source).unwrap();
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::K1 { value: crate::K1Document::K1Partnership((page.clone()).into()) },
        )],
    );
    current_source_acceptance("PTP-rental-save", &mut session, request);
    assert_eq!(value(&supported(&session.draft), "agi"), 67_000);
    let reopened = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = reopened else { panic!("K-1 source editor") };
    assert_eq!(documents.common.k1.documents.len(), 1);
    let id = documents.common.k1.documents[0].id;
    page.rentals.real_estate = -800_000;
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![document(
            id,
            IncomeSource::K1 { value: crate::K1Document::K1Partnership((page.clone()).into()) },
        )],
    );
    current_source_acceptance("PTP-rental-correction", &mut session, request);
    assert_eq!(value(&supported(&session.draft), "agi"), 64_000);
}

#[test]
fn ptp_rental_printed_rows_transfer_without_a_rounding_residue() {
    // Source cents net to zero; the three independently printed Schedule E
    // rows are -$1, +$1 and +$1. Their published total is $1.
    let draft = ptp_rental_draft(ptp_rental_source(-101, 50, 51));
    let result = supported(&draft);
    assert_eq!(value(&result, "partnership_2_activity_net"), -1);
    assert_eq!(value(&result, "partnership_2_rental_real_estate_net"), 1);
    assert_eq!(value(&result, "partnership_2_other_rental_net"), 1);
    assert_eq!(value(&result, "agi"), 60_001);
    let mut scratch = FederalResult::empty(draft.revision);
    let pool = crate::extended_rules::rentals_for(&draft, 60_000, &mut scratch, false);
    assert_eq!(pool.ptp_income, 1);
    assert_eq!(pool.k1_amounts[0].source_net_cents, Some(0));
    assert_eq!(pool.k1_amounts[0].net, 1);
}

#[test]
fn ptp_rental_missing_amt_and_section179_history_stay_unavailable() {
    let mut source = ptp_rental_source(1_000_000, -500_000, 0);
    source.alternative = None;
    let draft = ptp_rental_draft(source);
    assert!(matches!(
        crate::section179_return::prepare(&draft, Section179IncomeSystem::Alternative),
        Err(crate::section179_return::Issue::Prepared(Section179PreparedIssue::K1Availability(
            2,
            K1Section179BasisIssue::AlternativeHistoryUnavailable
        )))
    ));
    let mut source = ptp_rental_source(1_000_000, -500_000, 0);
    source.section179 = 100_000;
    let result = estimate(&ptp_rental_draft(source));
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_SECTION179_SELECTED_SOURCE_UNAVAILABLE")
    );
}

#[test]
fn ptp_rental_offsets_do_not_replace_ordinary_qualified_loss_character() {
    let mut source = ptp_rental_source(-1_500_000, 1_000_000, 0);
    source.qbi = Some(partner_qbi_statement(-15_000));
    let draft = ptp_rental_draft(source);
    let result = supported(&draft);
    assert_eq!(value(&result, "agi"), 60_000);
    assert_eq!(value(&result, "reit_ptp_loss_carryforward"), 10_000);
    assert_eq!(value(&result, "qbi_deduction"), 0);
    let selected =
        crate::section179_return::prepare(&draft, Section179IncomeSystem::Regular).unwrap();
    let ordinary = selected.ptp[&2]
        .components
        .iter()
        .find(|part| part.original.character == K1LossClass::Ordinary)
        .unwrap();
    assert_eq!(
        (ordinary.allowed_passive_net_cents, ordinary.suspended_cents),
        (-1_000_000, 500_000)
    );
}

#[test]
fn ptp_rental_regular_magi_controls_both_common_rental_allowances() {
    let mut source = ptp_rental_source(0, 2_000_000, 0);
    source.alternative.as_mut().unwrap().rentals.real_estate = 6_000_000;
    let mut draft = ptp_rental_draft(source);
    draft.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 650_000);
    let mut common = ptp_rental_source(0, -2_500_000, 0);
    common.publicly_traded = false;
    common.rental_facts.participation = Some(K1RentalParticipation::Active);
    add(&mut draft, income(FederalIncome::Partnership { data: common }));
    let result = supported(&draft);
    assert_eq!(value(&result, "agi"), 105_000);
    for prefix in ["", "amt_"] {
        assert_eq!(value(&result, &format!("{prefix}rental_magi")), 120_000);
        assert_eq!(
            value(&result, &format!("{prefix}partnership_3_rental_real_estate_net")),
            -15_000
        );
    }
}

#[test]
fn ptp_rental_net_investment_income_keeps_allowed_components_once() {
    let mut draft = ptp_rental_draft(ptp_rental_source(1_000_000, -500_000, 200_000));
    draft.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 650_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "agi"), 257_000);
    assert_eq!(value(&result, "form_8960_net_investment_income"), 7_000);
    assert_eq!(value(&result, "net_investment_income_tax"), 266);
}

#[test]
fn ptp_rental_no_section179_entry_rejects_an_unchecked_deduction() {
    let input = crate::PtpPalActivity {
        k1: 2,
        source: 2,
        components: vec![crate::PtpPalComponent {
            character: K1LossClass::Section179,
            passive_current_cents: -100,
            prior_passive_loss_cents: 0,
            nonpassive_cents: 0,
        }],
    };
    assert_eq!(
        crate::ptp_passive::without_section179(2, false, &[input]),
        Err(crate::PtpPalIssue::Section179SourceMismatch)
    );
}

#[test]
fn ptp_rental_fractional_section179_keeps_its_separately_reported_line() {
    // i1040se line 28: ordinary $1.01, section 179 -$0.50 and rental
    // $1.00 print as $1, -$1 and $1. Published total $1; exact net $1.51.
    let mut source = ptp_rental_source(101, 100, 0);
    source.section179 = 50;
    source.amt_section179_income_carry =
        Some(Section179IncomeCarryHistory { gross: 0, sold: 0, vintages: vec![] });
    let draft = ptp_rental_draft(source);
    for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
        let selected = crate::section179_return::prepare(&draft, system).unwrap();
        let allocation = selected.allocation.ptp_section179_pal_input(2).unwrap();
        assert_eq!(allocation.allowed_cents, 50);
        assert_eq!(allocation.business_income_carry_cents, 0);
        let ptp = &selected.ptp[&2];
        assert_eq!((ptp.net_gain_cents, ptp.suspended_cents), (151, 0));
        let ordinary = ptp
            .components
            .iter()
            .find(|part| part.original.character == K1LossClass::Ordinary)
            .unwrap();
        let deduction = ptp
            .components
            .iter()
            .find(|part| part.original.character == K1LossClass::Section179)
            .unwrap();
        let rental = ptp
            .components
            .iter()
            .find(|part| part.original.character == K1LossClass::RentalRealEstate)
            .unwrap();
        assert_eq!(
            (
                ordinary.allowed_passive_net_cents,
                deduction.allowed_passive_net_cents,
                rental.allowed_passive_net_cents
            ),
            (101, -50, 100)
        );
        assert_eq!(ptp.ordinary_net_cents(), Some(51));
        assert_eq!(ptp.ordinary_reported_net(), Some(0));
        let mut report = FederalResult::empty(draft.revision);
        selected.report(&mut report).unwrap();
        assert_eq!(
            report.lines.iter().find(|line| line.id == "section179_deduction").unwrap().amount,
            50
        );
    }
    let result = supported(&draft);
    assert_eq!(value(&result, "partnership_2_activity_net"), 0);
    assert_eq!(value(&result, "partnership_2_rental_real_estate_net"), 1);
    assert_eq!(value(&result, "agi"), 60_001);
}

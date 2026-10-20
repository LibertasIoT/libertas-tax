use super::*;

#[test]
fn unit17_ptp_report_context_omits_only_consumed_sources_and_keeps_native_rows() {
    let mut draft = ptp179_draft(20_000, 5_000);
    let mut second = draft.income[1].clone();
    let FederalIncome::Partnership { data } = &mut second.income else { unreachable!() };
    data.participation = ordinary_partner_source(20_000, false).participation;
    add(&mut draft, second);
    let system = Section179IncomeSystem::Regular;
    let sources: Vec<_> = [2, 3]
        .into_iter()
        .map(|source| Section179BusinessIncomeSource {
            source,
            system,
            ordinary: Section179IncomeClass {
                current_net_cents: 2_000_000,
                prior_allowed_loss_cents: 0,
            },
            section1231: Section179IncomeClass {
                current_net_cents: 0,
                prior_allowed_loss_cents: 0,
            },
        })
        .collect();
    let allocation =
        crate::section179::section179_with_native_prepared_income_and_operating_context(
            &draft,
            system,
            &[],
            &sources,
            |_, _, _| Some(Section179OtherLimitedDeduction { system, amount_cents: 0 }),
        )
        .unwrap();
    let first = allocation.ptp_section179_pal_input(2).unwrap();
    let second = allocation.ptp_section179_pal_input(3).unwrap();
    assert!(first.active);
    assert!(!second.active);
    assert_eq!(first.allowed_cents, 500_000);
    assert_eq!(second.allowed_cents, 500_000);
    let mut old = FederalResult::empty(draft.revision);
    allocation.ptp_report_context(&[]).unwrap().report(&mut old);
    assert_eq!(value(&old, "section179_deduction"), 10_000);
    let ptp = "ISSUE_SECTION179_PUBLICLY_TRADED";
    let passive = "ISSUE_SECTION179_PASSIVE_ACTIVITY";
    assert_eq!(old.issues.iter().filter(|issue| issue.display.key == ptp).count(), 2);
    assert_eq!(old.issues.iter().filter(|issue| issue.display.key == passive).count(), 1);
    let other: Vec<_> = old
        .issues
        .iter()
        .filter(|issue| issue.display.key != ptp && issue.display.key != passive)
        .cloned()
        .collect();
    for (consumed, remaining, remaining_passive) in
        [(&[][..], 2, 1), (&[first][..], 1, 1), (&[second][..], 1, 0), (&[first, second][..], 0, 0)]
    {
        let context = allocation.ptp_report_context(consumed).unwrap();
        let mut report = FederalResult::empty(draft.revision);
        context.report(&mut report);
        assert_eq!(report.lines, old.lines);
        assert_eq!(
            report.issues.iter().filter(|issue| issue.display.key == ptp).count(),
            remaining
        );
        assert_eq!(
            report.issues.iter().filter(|issue| issue.display.key == passive).count(),
            remaining_passive
        );
        assert_eq!(
            report
                .issues
                .iter()
                .filter(|issue| issue.display.key != ptp && issue.display.key != passive)
                .cloned()
                .collect::<Vec<_>>(),
            other
        );
    }
    let mut unchanged = FederalResult::empty(draft.revision);
    allocation.ptp_report_context(&[]).unwrap().report(&mut unchanged);
    assert_eq!(unchanged, old);
}

#[test]
fn unit17_ptp_report_context_requires_exact_allocation_and_preserves_owner_refusals() {
    let mut draft = ptp179_draft(20_000, 5_000);
    let FederalIncome::Partnership { data } = &mut draft.income[1].income else { unreachable!() };
    data.amt_section179_income_carry =
        Some(Section179IncomeCarryHistory { gross: 0, sold: 0, vintages: vec![] });
    let regular =
        selected_ptp179(&draft, Section179IncomeSystem::Regular, 2, (20_000) * 100).unwrap();
    let foreign =
        selected_ptp179(&draft, Section179IncomeSystem::Regular, 2, (20_000) * 100).unwrap();
    let alternative =
        selected_ptp179(&draft, Section179IncomeSystem::Alternative, 2, (20_000) * 100).unwrap();
    let source = regular.ptp_section179_pal_input(2).unwrap();
    let cloned = source.clone();
    for other in [
        foreign.ptp_section179_pal_input(2).unwrap(),
        alternative.ptp_section179_pal_input(2).unwrap(),
        &cloned,
    ] {
        assert_eq!(
            regular.ptp_report_context(&[other]).unwrap_err(),
            Ptp179SourceIssue::DifferentAllocation
        );
    }
    let FederalIncome::Partnership { data } = &mut draft.income[1].income else { unreachable!() };
    data.pensions = 100;
    let refused =
        selected_ptp179(&draft, Section179IncomeSystem::Regular, 2, (20_000) * 100).unwrap();
    let Err(Ptp179SourceIssue::OwnerIssues(issues)) = refused.ptp_report_context(&[source]) else {
        panic!("existing owner refusal must precede consumption")
    };
    assert!(issues.iter().any(|issue| issue.display.key == "ISSUE_SECTION179_OWNER_DEDUCTIONS"));
    let mut unconsumed = FederalResult::empty(draft.revision);
    refused.ptp_report_context(&[]).unwrap().report(&mut unconsumed);
    assert!(
        unconsumed
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_SECTION179_PUBLICLY_TRADED")
    );
}

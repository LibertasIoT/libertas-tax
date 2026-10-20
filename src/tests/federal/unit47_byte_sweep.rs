// Native Save preserves descriptive source text without truncation.
use super::*;

fn student(label: &str) -> crate::HostedStudent {
    crate::HostedStudent {
        label: label.into(),
        sponsor: "Synthetic placement charity".into(),
        agreement: "Written placement agreement".into(),
        school: "Synthetic high school".into(),
        school_location: "Example city and state".into(),
        grade: 11,
        joined: 20260601,
        school_start: 20260601,
        school_end: 20260831,
        months: vec![crate::HostedStudentMonth {
            month: 6,
            qualifying_days: 15,
            cost: 8_000,
        }],
        documented: true,
    }
}

fn sources() -> GiftCarryovers {
    let expense = whaling_expense(WhalingExpenseKind::WeaponsAndGear, 10_000);
    let claim = whaling_claim(vec![expense.clone(), expense]);
    let mut page = gifts(vec![], vec![], vec![]);
    page.volunteer = vec![
        volunteer_travel(VolunteerCarCost::Mileage { miles: 10 }, 0),
        volunteer_travel(VolunteerCarCost::Mileage { miles: 20 }, 0),
    ];
    page.less_common = vec![
        LessCommonGift::HostedStudent(student("First hosted student")),
        LessCommonGift::WhalingCaptain(WhalingCaptain {
            owner: claim.owner,
            captain_record_reference: claim.captain_record_reference,
            aewc_report_reference: claim.aewc_report_reference,
            trip_log_reference: claim.trip_log_reference,
            expenses: claim.expenses,
            eligibility_and_records_confirmed: claim.eligibility_and_records_confirmed,
        }),
        LessCommonGift::HostedStudent(student("Second hosted student")),
    ];
    page
}

// Select the actual source, including after the stored page groups its kinds.
fn source_text(page: &mut GiftCarryovers, field: usize) -> &mut alloc::string::String {
    if field < 2 {
        return if field == 0 {
            &mut page.volunteer[1].charity
        } else {
            &mut page.volunteer[1].service
        };
    }
    if field < 7 {
        let student = page
            .less_common
            .iter_mut()
            .filter_map(|row| match row {
                LessCommonGift::HostedStudent(student) => Some(student),
                _ => None,
            })
            .nth(1)
            .unwrap();
        return match field {
            2 => &mut student.label,
            3 => &mut student.sponsor,
            4 => &mut student.agreement,
            5 => &mut student.school,
            6 => &mut student.school_location,
            _ => unreachable!(),
        };
    }
    let captain = page
        .less_common
        .iter_mut()
        .find_map(|row| match row {
            LessCommonGift::WhalingCaptain(captain) => Some(captain),
            _ => None,
        })
        .unwrap();
    match field {
        7 => &mut captain.captain_record_reference,
        8 => &mut captain.aewc_report_reference,
        9 => &mut captain.trip_log_reference,
        10 => &mut captain.expenses[1].place,
        11 => &mut captain.expenses[1].description,
        12 => &mut captain.expenses[1].report_reference,
        13 => match captain.expenses[1].payment_evidence.as_mut().unwrap() {
            WhalingPaymentEvidence::Original { record_reference } => record_reference,
            _ => unreachable!(),
        },
        14 | 15 => match captain.expenses[1].payment_evidence.as_mut().unwrap() {
            WhalingPaymentEvidence::Reconstructed {
                loss_circumstances,
                reconstruction_reference,
            } => {
                if field == 14 {
                    loss_circumstances
                } else {
                    reconstruction_reference
                }
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

#[test]
fn unit47_byte_sweep_gift_sources_save_long_utf8_without_truncation() {
    // 2026-10-15 D-13: replace 112 obsolete encoding/ceiling cases with one
    // mixed UTF-8 value per source field; preserve each complete native lifecycle.
    let exact = "aé😀e\u{301}".repeat(40);
    assert_eq!(exact.len(), 400);
    for field in 0..16 {
        let mut page = sources();
        if field >= 14 {
            let LessCommonGift::WhalingCaptain(captain) = &mut page.less_common[1] else {
                unreachable!()
            };
            captain.expenses[1].payment_evidence = Some(WhalingPaymentEvidence::Reconstructed {
                loss_circumstances: "Flood destroyed the original records".into(),
                reconstruction_reference: "Reconstruction with supporting evidence".into(),
            });
        }
        *source_text(&mut page, field) = exact.clone();
        let mut session = at_the_gifts_page(DeductionRoute::Itemize, 0);
        let request = save_gift_carryovers(&session.draft, page);
        itemized_hub(send_past_less_common(&mut session, request));
        let saved = session.draft.clone();
        let mut reopened = open_itemized_topic(&mut session, TaxSection::GiftCarryovers);
        let P::SaveGiftCarryovers { value, .. } = &mut reopened else {
            unreachable!()
        };
        assert_eq!(*source_text(value, field), exact);
        itemized_hub(send_past_less_common(&mut session, reopened));
        assert_eq!(session.draft, saved);
        let mut restored = restart(saved.clone());
        let mut reopened = open_itemized_topic(&mut restored, TaxSection::GiftCarryovers);
        let P::SaveGiftCarryovers { value, .. } = &mut reopened else {
            unreachable!()
        };
        assert_eq!(*source_text(value, field), exact);
        itemized_hub(send_past_less_common(&mut restored, reopened));
        assert_eq!(restored.draft, saved);
    }
}

#[test]
fn unit47_byte_sweep_mixed_gift_rows_keep_submitted_duplicate_indices_and_order() {
    let mut session = at_the_gifts_page(DeductionRoute::Itemize, 0);
    let before = session.draft.clone();
    let mut page = sources();
    let LessCommonGift::HostedStudent(first) = &page.less_common[0] else {
        unreachable!()
    };
    let duplicate = first.clone();
    page.less_common
        .push(LessCommonGift::HostedStudent(duplicate));
    let LessCommonGift::WhalingCaptain(captain) = &page.less_common[1] else {
        unreachable!()
    };
    let duplicate = captain.clone();
    page.less_common
        .push(LessCommonGift::WhalingCaptain(duplicate));
    for (name, key, path) in [
        (
            "captain",
            "WHALING_CAPTAIN_TWICE",
            "[value/less_common/4/owner]",
        ),
        (
            "student",
            "HOSTED_STUDENT_TWICE",
            "[value/less_common/3/label]",
        ),
    ] {
        let request = save_gift_carryovers(&session.draft, page.clone());
        let Reply::Response(problem @ P::Problem { .. }) = send(&mut session, request.clone())
        else {
            panic!("duplicate")
        };
        let P::Problem { message, field, .. } = &problem else {
            unreachable!()
        };
        assert_eq!(*message, libertas::libertas_formatted_text(key, &[]));
        assert_eq!(field.as_deref(), Some(path));
        assert_eq!(session.draft, before);
        crate::tests::unit47_rule_cases::export_unit47_rule_case(
            &alloc::format!("gift-duplicate-{name}"),
            &request,
            &problem,
        );
        page.less_common.pop();
    }
    itemized_hub(send_past_less_common(
        &mut session,
        save_gift_carryovers(&before, page),
    ));
}

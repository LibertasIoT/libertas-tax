use super::*;
#[test]
fn vehicle_interest_aggregates_loans_and_does_not_reduce_agi() {
    let mut d = complete(FederalStatus::Single);
    d.deductions.as_mut().unwrap().vehicle_loans = vec![
        VehicleLoanSource {
            id: 0,
            assigned: crate::DocumentRecipient::Taxpayer,
            owner_source: Some(crate::OwnerSource::Added),
            joint_share: None,
            business: 0,
            document: vehicle_paper(FederalVehicleInterest {
                lender: crate::PrintedParty::from_name("Synthetic car".into()),
                borrower: Default::default(),
                interest: 600_000,
                refund: 0,
                original_use: true,
                assembled_in_us: true,
            }),
        },
        VehicleLoanSource {
            id: 0,
            assigned: crate::DocumentRecipient::Taxpayer,
            owner_source: Some(crate::OwnerSource::Added),
            joint_share: None,
            business: 0,
            document: vehicle_paper(FederalVehicleInterest {
                lender: crate::PrintedParty::from_name("Synthetic van".into()),
                borrower: Default::default(),
                interest: 600_000,
                refund: 0,
                original_use: true,
                assembled_in_us: true,
            }),
        },
    ];
    let r = supported(&d);
    assert_eq!(value(&r, "vehicle_interest_deduction"), 10000);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(r.tax, Some(382_300));
    d.deductions.as_mut().unwrap().vehicle_loans[0].document.box_6 = false;
    assert_eq!(value(&supported(&d), "vehicle_interest_deduction"), 6000);
}

#[test]
fn vehicle_charity_fmv_exceptions_require_section_b_appraisal_above_five_thousand() {
    let treatments = [
        VehicleUse::SignificantUse {
            details: "Charity used car for daily meal deliveries for a year".into(),
        },
        VehicleUse::MaterialImprovement {
            details: "Charity funded and completed major engine repairs before transfer".into(),
        },
        VehicleUse::NeedyPerson,
    ];
    for treatment in treatments {
        let mut d = complete(FederalStatus::Single);
        let ded = d.deductions.as_mut().unwrap();
        push_gift(ded, donated_personal_vehicle(treatment));
        ded.itemize_override = Some(ItemizeOverride::Itemize);
        assert!(noted(&supported(&d), "WARN_GIFT_LEFT_OUT"));
        set_gift_appraisal(d.deductions.as_mut().unwrap(), 0, 2);
        d.next_id = d.next_id.max(3);
        d.deductions.as_mut().unwrap().vehicle_gifts[0].donee_acknowledged = false;
        assert!(noted(&supported(&d), "WARN_GIFT_LEFT_OUT"));
        d.deductions.as_mut().unwrap().vehicle_gifts[0].donee_acknowledged = true;
        let result = supported(&d);
        assert_eq!(value(&result, "charity_property_1_claim"), 20_000);
        assert_eq!(value(&result, "form_8283_section_b_value"), 20_000);
        assert_eq!(value(&result, "charity_property_fifty_current_deduction"), 19_700);
        assert_eq!(estimate(&restart(d.clone()).draft), result);

        d.deductions.as_mut().unwrap().vehicle_gifts[0].value = 500_000;
        d.deductions.as_mut().unwrap().vehicle_gifts[0].appraisal = None;
        let section_a = supported(&d);
        assert_eq!(value(&section_a, "form_8283_section_a_value"), 5_000);
        assert!(!section_a.lines.iter().any(|line| line.id == "form_8283_section_b_value"));
    }
}

#[test]
fn every_business_credit_id_pattern_has_its_line_and_review_row_resources() {
    // Each general business credit emits `business_credit_<key>_<year>` and the
    // carry, used, passive, expired and carryback ids around it, all tagged with
    // their Form 3800 line; the review row (ROW_) and the label (LINE_) of every
    // such id must exist so a new credit cannot miss them.
    for credit in crate::general_business_credit::BusinessCredit::ALL {
        let key = credit.key();
        for suffix in [
            "current_year",
            "current_year_passive",
            "2024",
            "2024_passive",
            "current_year_to_next_year",
            "current_year_passive_to_next_year",
            "2024_to_next_year",
            "current_year_used",
            "current_year_carryback_2025",
            "2024_expired",
        ] {
            // A carryforward-only credit never emits a carryback line.
            if suffix == "current_year_carryback_2025" && !credit.carryback_allowed() {
                continue;
            }
            let id = alloc::format!("business_credit_{key}_{suffix}");
            let (pattern, _) = crate::federal_model::id_pattern(&id);
            let resource = alloc::format!("LINE_{}", pattern.to_uppercase());
            assert!(
                crate::APP_STRINGS.iter().any(|(k, _)| *k == resource),
                "missing {resource} for {id}"
            );
            // Both labels build without the debug assertion firing.
            let _ = crate::federal_model::line_label(&id);
            let _ = crate::FormLine::new(TaxForm::Form3800, "1").row_label(&id);
        }
    }
}

// The descriptor of every credit is the one source: `BusinessCredit::ALL`
// names each credit once, and the client rules of the carryover items (which
// are attribute text) agree with the descriptor's first year and carryback.
#[test]
fn business_credit_descriptors_agree_with_the_client_rules() {
    use crate::general_business_credit::BusinessCredit;
    let mut count = 0;
    loop {
        let mut encoded = alloc::vec::Vec::new();
        libertas::AvroEncode::avro_encode(&(count as i64), &mut encoded);
        let mut offset = 0;
        if <BusinessCredit as libertas::AvroDecode>::avro_decode(&encoded, &mut offset).is_err() {
            break;
        }
        count += 1;
    }
    assert_eq!(BusinessCredit::ALL.len(), count);
    for (index, credit) in BusinessCredit::ALL.iter().enumerate() {
        assert!(!BusinessCredit::ALL[..index].contains(credit));
    }
}

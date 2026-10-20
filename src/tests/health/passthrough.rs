use super::*;

#[test]
fn current_hsa_funding_rejects_unlinked_wrong_owner_and_mismatched_basis() {
    let mut d = complete(FederalStatus::Single);
    current_hsa_funding(&mut d, 4000);
    let id = d.adjustments.as_ref().unwrap().hsa[0].funding[0].income;
    d.adjustments.as_mut().unwrap().hsa[0].funding[0].income = 999999;
    assert!(!d.valid_stored());
    assert_eq!(estimate(&d).state, ResultState::Incomplete);
    d.adjustments.as_mut().unwrap().hsa[0].funding[0].income = id;
    d.adjustments.as_mut().unwrap().ira_basis = vec![prior_ira_basis(Owner::Taxpayer, 200, 500)];
    unsupported(&d, "HSA funding");
    d.adjustments.as_mut().unwrap().ira_basis.clear();
    d.income.iter_mut().find(|v| v.id == id).unwrap().owner = Owner::Spouse;
    assert!(!d.valid_stored());
    assert_eq!(estimate(&d).state, ResultState::Incomplete);
}

#[test]
fn hsa_funding_notice_2008_51_preserves_basis_and_excludes_transfer_from_pro_rata() {
    let d = hsa_prior_basis_funding(1500, 200, 2000, 500, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 200);
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 0);
    assert_eq!(return_line(&r, "4a"), 150000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    // A later $100 withdrawal uses $40 of the $200 basis in the remaining $500 pool.
    let d = hsa_prior_basis_funding(1500, 200, 2000, 400, 100);
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 6000);
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 40);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 160);
    assert_eq!(value(&r, "early_distribution_tax"), 6);
}

#[test]
fn hsa_funding_consumes_only_basis_above_available_taxable_money() {
    let d = hsa_prior_basis_funding(1900, 200, 2000, 100, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 100);
    assert_eq!(return_line(&r, "4b"), 0);
    let id = d.adjustments.as_ref().unwrap().hsa[0].funding[0].income;
    assert_eq!(value(&r, &alloc::format!("hsa_funding_basis_{id}_20260115")), 100);
    // Historical basis exceeding market value is not a negative taxable amount.
    let d = hsa_prior_basis_funding(1500, 2500, 2000, 500, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 1000);
    assert_eq!(return_line(&r, "4b"), 0);
}

#[test]
fn hsa_funding_second_transfer_uses_remaining_basis_and_contemporaneous_balance() {
    let mut d = hsa_prior_basis_funding(4000, 2000, 10000, 1000, 4000);
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    let mut second = h.funding[0].clone();
    second.funded = 20260215;
    second.coverage = HsaCoverage::Family;
    second.basis = HsaFundingBasis::PriorBasis {
        distributed: 20260215,
        value: 500000,
        before_other_activity: true,
    };
    h.funding.push(second);
    // Actual first-day coverage changes after the January source transfer.
    h.coverage = HsaEligibility::Monthly {
        months: (0..12)
            .map(|month| if month == 0 { HsaCoverage::SelfOnly } else { HsaCoverage::Family })
            .collect(),
    };
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 1000);
    assert_eq!(return_line(&r, "4b"), 0);
    d.adjustments.as_mut().unwrap().hsa[0].funding.reverse();
    assert_eq!(supported(&d), r);
}

#[test]
fn hsa_funding_basis_and_later_qcd_do_not_allocate_the_same_source_dollars() {
    let mut d = hsa_prior_basis_funding(1500, 200, 2000, 0, 500);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    d.adjustments.as_mut().unwrap().qualified_charity =
        vec![qcd(Owner::Taxpayer, 20261201, 300, 0)];
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 300);
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 200);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 0);
    assert_eq!(return_line(&r, "4b"), 0);
    // The wholly pretax path likewise assigns QCD only from dollars left after funding.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    current_hsa_funding(&mut d, 4400);
    d.adjustments.as_mut().unwrap().qualified_charity =
        vec![qcd(Owner::Taxpayer, 20261201, 1600, 0)];
    assert_eq!(return_line(&supported(&d), "4b"), 0);
    d.adjustments.as_mut().unwrap().qualified_charity[0].payments[0].amount += 100;
    unsupported(&d, "QCD");
}

#[test]
fn hsa_funding_basis_requires_source_balances_and_supported_transaction_order() {
    let mut d = hsa_prior_basis_funding(1500, 200, 2000, 500, 0);
    d.adjustments.as_mut().unwrap().hsa[0].funding[0].basis = HsaFundingBasis::PriorBasis {
        distributed: 20260115,
        value: 149900,
        before_other_activity: true,
    };
    unsupported(&d, "HSA funding");
    d.adjustments.as_mut().unwrap().hsa[0].funding[0].basis = HsaFundingBasis::PriorBasis {
        distributed: 20260115,
        value: 169900,
        before_other_activity: false,
    };
    // 2026-10-19 (inv-66): below the opening-basis bound the QHFD still comes first
    // (Notice 2008-51; Form 8606 line 7 excludes it); 1699 less 200 basis uses 1 of basis.
    let r = supported(&d);
    assert!(!noted(&r, "WARN_HSA_FUNDING_LEFT_OUT"), "{r:?}");
    assert_eq!(value(&r, "hsa_funding_basis_2_20260115"), 1);
    d.adjustments.as_mut().unwrap().hsa[0].funding[0].basis = HsaFundingBasis::PriorBasis {
        distributed: 20260115,
        value: 200000,
        before_other_activity: false,
    };
    // Actual FMV2000 covers1500 funding plus opening200 basis, with no basis
    // additions/changes. The retirement owner now proves zero funding basis
    // use without requiring a funding-first answer.
    let r = supported(&d);
    assert!(!noted(&r, "WARN_HSA_FUNDING_LEFT_OUT"));
    assert_eq!(value(&r, "hsa_funding_basis_2_20260115"), 0);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 200);
    d.adjustments.as_mut().unwrap().hsa[0].funding[0].basis = HsaFundingBasis::PriorBasis {
        distributed: 20260115,
        value: 200000,
        before_other_activity: true,
    };
    d.adjustments.as_mut().unwrap().ira_basis.clear();
    unsupported(&d, "HSA funding");
}

#[test]
fn hsa_funding_remaining_basis_applies_to_later_roth_conversion() {
    let mut d = hsa_prior_basis_funding(1500, 200, 2000, 400, 0);
    add(
        &mut d,
        basis_distribution(
            Owner::Taxpayer,
            100,
            RetirementTreatment::Conversion { amount: 10000, early: false },
        ),
    );
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 160000);
    assert_eq!(return_line(&r, "4b"), 6000);
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 40);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 160);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
}

#[test]
fn hsa_payment_reference_rejects_wrong_owner_kind_and_inherited_group() {
    let good = inherited_hsa_funding(4000, 0, 4000, 0, 4000);
    let mut inferred = good.clone();
    if let FederalIncome::Retirement { data } = &mut inferred.income[2].income {
        data.taxable = RetirementTaxable::Inherited { account: None, payer: data.gross };
    }
    assert_eq!(
        crate::retirement::ira_funding_source(&inferred, 3).unwrap().inherited.unwrap().0,
        2
    );
    assert!(inferred.valid_stored());

    for variant in 0..7 {
        let mut d = good.clone();
        match variant {
            0 => d.income[2].owner = Owner::Spouse,
            1 => d.income[1].owner = Owner::Spouse,
            2 => {
                let mut group = inherited_roth(2020, 0, 0);
                group.id = 2;
                d.income[1] = group;
            }
            3 => d.income[2].income = good.income[0].income.clone(),
            4 => d.income.retain(|entry| entry.id != 3),
            5 => d.income.retain(|entry| entry.id != 2),
            _ => {
                if let FederalIncome::Retirement { data } = &mut d.income[2].income {
                    data.taxable =
                        RetirementTaxable::Inherited { account: None, payer: data.gross };
                }
                add(&mut d, inherited_ira(0, 0)); // Two inferred same-kind groups: ambiguous.
            }
        }
        assert!(
            crate::retirement::ira_funding_source(&d, 3).is_none()
                || crate::retirement::ira_funding_source(&d, 3).unwrap().entry.owner
                    != Owner::Taxpayer
        );
        assert_eq!(
            crate::hsa_funding::source_reference_rule(&d),
            Some("HSA_FUNDING_SOURCE_NOT_OFFERED")
        );
        assert!(!d.valid_stored());
        assert_eq!(estimate(&d).state, ResultState::Incomplete);
        assert_eq!(restart(d.clone()).draft, d); // Invalid retained facts are preserved, never repaired.
    }
}

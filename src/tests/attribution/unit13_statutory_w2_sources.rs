use super::*;

#[test]
fn unit13_statutory_w2_sources_unlinked_and_expenses_only_public_lifecycle() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for linked in [false, true] {
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                for reversed in [false, true] {
                    let mut d = injured_return((60_000, 5000), (20_000, 500), 0, allocation());
                    d.injured_spouse.as_mut().unwrap().injured = injured;
                    let target = linked.then(|| source_expenses_business(&mut d, owner, 6000));
                    source_statutory_wage(&mut d, owner, target);
                    if reversed {
                        d.income.reverse();
                    }
                    let r = supported(&d);
                    // Actual source_statutory_wage net is60000/20000 without source_expenses_business;
                    // linked6000 expense changes native jointAGI to74000.
                    assert_eq!(value(&r, "agi"), if linked { 74_000 } else { 80_000 });
                    assert_eq!(value(&r, "self_employment_tax"), 0);
                    let (qbi, taxable, tax) = match (owner, linked) {
                        (Owner::Taxpayer, false) => (9560, 38_240, 4091),
                        (Owner::Spouse, false) => (4000, 43_800, 4763),
                        (Owner::Taxpayer, true) => (8360, 33_440, 3515),
                        (Owner::Spouse, true) => (2800, 39_000, 4187),
                    };
                    assert_eq!(value(&r, "qbi_deduction"), qbi);
                    assert_eq!(value(&r, "taxable_income"), taxable);
                    assert_eq!(r.tax, Some(tax * 100));
                    assert_eq!(r.refund, Some((5500 - tax) * 100));
                    let mut ordinary = d.clone();
                    ordinary.injured_spouse = None;
                    ordinary.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
                    let plain = supported(&ordinary);
                    assert_eq!(plain.return_lines, r.return_lines);
                    assert_eq!(
                        (plain.tax, plain.refund, plain.balance),
                        (r.tax, r.refund, r.balance)
                    );
                    let saved = save_reopen_native_sources(
                        d,
                        &[TaxSection::Documents, TaxSection::SelfEmployment, TaxSection::Farming],
                        &[TaxSection::Documents, TaxSection::SelfEmployment],
                    );
                    let r = supported(&saved);
                    let mut wages = [60_000, 20_000];
                    if linked {
                        wages[usize::from(owner == Owner::Spouse)] -= 6000;
                    }
                    assert_income_source_columns(&r, injured, wages, [0; 2]);
                }
            }
        }
    }
}

#[test]
fn unit13_statutory_w2_sources_multiple_w2_c_and_f_preserve_native_category_residuals() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reversed in [false, true] {
            let mut d = statutory_multiple_return();
            d.injured_spouse.as_mut().unwrap().injured = injured;
            if reversed {
                d.income.reverse();
            }
            let r = supported(&d);
            assert_eq!(value(&r, "agi"), 86_803);
            assert_eq!(value(&r, "schedule_1_business_income"), 6803);
            assert_eq!(
                r.lines
                    .iter()
                    .find(|row| row.id == "schedule_1_farm_income")
                    .map_or(0, |row| row.amount / 100),
                0
            );
            assert_eq!(return_line(&r, "1a"), 8_000_000);
            let mut plain = d.clone();
            plain.injured_spouse = None;
            plain.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
            let financial = supported(&plain);
            assert_eq!(r.return_lines, financial.return_lines);
            assert_eq!(
                (r.tax, r.refund, r.balance),
                (financial.tax, financial.refund, financial.balance)
            );
            let saved = save_reopen_native_sources(
                d,
                &[TaxSection::Documents, TaxSection::SelfEmployment, TaxSection::Farming],
                &[TaxSection::Documents, TaxSection::SelfEmployment],
            );
            let r = supported(&saved);
            // W2 net6802 plus Other C residual1; F receives native residual0.
            assert_income_source_columns(&r, injured, [63_901, 22_901], [1, 0]);
        }
    }
}
#[test]
fn unit13_statutory_w2_sources_incomplete_or_wrong_owner_packet_never_infers_class() {
    use crate::injured_spouse::income_categories::NativeBusinessIncome;
    let mut d = injured_return((60_000, 5000), (20_000, 500), 0, allocation());
    source_statutory_wage(&mut d, Owner::Taxpayer, None);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 80_000);
    assert_eq!(r.tax, Some(409_100));
    let mut facts =
        liability_specialized_wage_facts(&d, Some([0; 2]), SpecializedWageFixture::Statutory);
    facts.w2_wages = [0, 2_000_000];
    facts.business = [6_000_000, 0];
    for packet in [
        None,
        Some(NativeBusinessIncome {
            joint_dollars: [60_000, 0],
            w2_cents: [[0, 6_000_000], [0; 2]],
            other_cents: [[0; 2]; 2],
            foreign_exclusion_dollars: [0; 2],
        }),
        Some(NativeBusinessIncome {
            joint_dollars: [59_999, 1],
            w2_cents: [[6_000_000, 0], [0; 2]],
            other_cents: [[0; 2]; 2],
            foreign_exclusion_dollars: [0; 2],
        }),
    ] {
        facts.income_categories.statutory_business = packet;
        let projection = crate::injured_spouse::owners::project_sources(
            crate::related_forms::RelatedFormContext::original(
                &d,
                &crate::related_forms::effective_claimants(&d),
            ),
            &r,
            None,
            &facts,
            None,
        )
        .unwrap();
        assert_eq!(projection.agi(), [60_000, 20_000]);
        let mut printed = r.clone();
        printed.lines.retain(|row| !row.id.starts_with("form_8379_"));
        crate::injured_spouse::allocate(
            crate::related_forms::RelatedFormContext::original(
                &d,
                &crate::related_forms::effective_claimants(&d),
            ),
            &mut printed,
            d.injured_spouse.as_ref().unwrap(),
            &facts,
            r.refund.expect("ordinary pre-election overpayment"),
        );
        for line in ["13a", "13b"] {
            for suffix in ["b", "c"] {
                assert!(
                    printed
                        .lines
                        .iter()
                        .all(|row| row.id != alloc::format!("form_8379_line_{line}_{suffix}"))
                );
            }
        }
        assert!(
            printed
                .review_notes
                .iter()
                .any(|note| note.display.key == "ISSUE_8379_BUSINESS_UNATTRIBUTABLE")
        );
        assert_eq!(printed.return_lines, r.return_lines);
    }
}

#[test]
fn unit13_statutory_w2_sources_depreciation_uses_actual_native_net() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut d = injured_return((60_000, 5000), (20_000, 500), 0, allocation());
        let id = source_expenses_business(&mut d, owner, 0);
        source_statutory_wage(&mut d, owner, Some(id));
        let FederalIncome::Business { data } =
            &mut d.income.iter_mut().find(|e| e.id == id).unwrap().income
        else {
            panic!()
        };
        let mut asset = elected_asset(10_000, 0, BonusElection::ElectOut);
        asset.acquired = 20170927;
        // The actual public equipment editor writes the table schedule;
        // the old calculation-only helper selected its manual equivalent.
        // Both first-year amounts remain2000regular/1500AMT.
        let EquipmentHistory::New { tables, .. } = &mut asset.history else { panic!() };
        *tables = true;
        data.assets = vec![asset];
        let mut page = crate::tests::cross::interview::business(0);
        let mut property =
            crate::tests::cross::interview::asset(1_000_000, 20260601, AssetClass::Five);
        property.acquired = Some(20170927);
        property.gds_mut().unwrap().history =
            AssetHistory::New { first_year: FirstYearDeduction::Regular };
        page.assets = vec![property];
        // Obtain only the accepted durable identity; retain the native source
        // and its original regular/AMT depreciation operands unchanged.
        let d = save_native_asset_identities(
            d,
            id,
            IncomeSource::Record { value: crate::IncomeRecord::Business(page) },
            TaxSection::SelfEmployment,
        );
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 78_000);
        assert_eq!(value(&r, "self_employment_tax"), 0);
        assert_eq!(value(&r, &alloc::format!("business_{id}_asset_0_macrs")), 2000);
        assert_eq!(value(&r, &alloc::format!("business_{id}_asset_0_amt_depreciation")), 1500);
        let saved = save_reopen_native_sources(
            d,
            &[TaxSection::Documents, TaxSection::SelfEmployment, TaxSection::Farming],
            &[TaxSection::Documents, TaxSection::SelfEmployment],
        );
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            let mut next = saved.clone();
            next.injured_spouse.as_mut().unwrap().injured = injured;
            let r = supported(&next);
            let mut wages = [60_000, 20_000];
            wages[usize::from(owner == Owner::Spouse)] -= 2000;
            assert_income_source_columns(&r, injured, wages, [0; 2]);
        }
    }
}
#[test]
fn unit13_statutory_w2_sources_unrelated_foreign_election_keeps_known_class() {
    let mut d = injured_return((60_000, 5000), (20_000, 500), 0, allocation());
    source_statutory_wage(&mut d, Owner::Taxpayer, None);
    let mut foreign = income(foreign_pay(30_000));
    foreign.owner = crate::DocumentOwner::Spouses;
    add(&mut d, foreign);
    set_claim(&mut d, foreign_claim(Owner::Spouse));
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 30_000);
    assert_eq!(value(&r, "agi"), 80_000);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    let mut plain = d.clone();
    plain.injured_spouse = None;
    assert_eq!(r.return_lines, supported(&plain).return_lines);
    let saved = save_reopen_native_sources(
        d,
        &[TaxSection::Documents, TaxSection::SelfEmployment, TaxSection::Farming],
        &[TaxSection::Documents, TaxSection::SelfEmployment],
    );
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        let mut next = saved.clone();
        next.injured_spouse.as_mut().unwrap().injured = injured;
        assert_income_source_columns(&supported(&next), injured, [60_000, 20_000], [0; 2]);
    }
}
#[test]
fn unit13_statutory_w2_sources_actual_foreign_exclusion_uses_native_per_c_owners() {
    let mut d = injured_return((60_000, 5000), (20_000, 500), 0, allocation());
    let mut c = income(foreign_service_business(0));
    let FederalIncome::Business { data } = &mut c.income else { panic!() };
    data.expenses = vec![ordinary_business_expense(BusinessExpenseCategory::Office, 600_000)];
    add(&mut d, c);
    let id = d.income.last().unwrap().id;
    source_statutory_wage(&mut d, Owner::Taxpayer, Some(id));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("business_{id}_net_profit")), 54_000);
    assert_eq!(value(&r, "agi"), 20_000);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(r.tax, Some(0));
    assert_eq!(r.refund, Some(550_000));
    let saved = save_reopen_native_sources(
        d,
        &[TaxSection::Documents, TaxSection::SelfEmployment, TaxSection::Farming],
        &[TaxSection::Documents, TaxSection::SelfEmployment],
    );
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        let mut next = saved.clone();
        next.injured_spouse.as_mut().unwrap().injured = injured;
        let r = supported(&next);
        assert_income_source_columns(&r, injured, [0, 20_000], [0; 2]);
    }
}

#[test]
fn unit13_statutory_w2_sources_supported_loss_preserves_signed_owner_net() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut d = injured_return((60_000, 5000), (20_000, 500), 0, allocation());
        let costs = if owner == Owner::Taxpayer { 70_000 } else { 30_000 };
        let id = source_expenses_business(&mut d, owner, costs);
        source_statutory_wage(&mut d, owner, Some(id));
        let r = supported(&d);
        let joint = if owner == Owner::Taxpayer { 10_000 } else { 50_000 };
        assert_eq!(value(&r, "agi"), joint);
        assert_eq!(value(&r, "self_employment_tax"), 0);
        assert_eq!(value(&r, "schedule_1_business_income"), -10_000);
        let mut plain = d.clone();
        plain.injured_spouse = None;
        assert_eq!(r.return_lines, supported(&plain).return_lines);
        let saved = save_reopen_native_sources(
            d,
            &[TaxSection::Documents, TaxSection::SelfEmployment, TaxSection::Farming],
            &[TaxSection::Documents, TaxSection::SelfEmployment],
        );
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            let mut next = saved.clone();
            next.injured_spouse.as_mut().unwrap().injured = injured;
            let mut wages = [60_000, 20_000];
            wages[usize::from(owner == Owner::Spouse)] = -10_000;
            assert_income_source_columns(&supported(&next), injured, wages, [0; 2]);
        }
    }
}

#[test]
fn unit13_statutory_w2_sources_financial_context_keeps_original_combined_owner_rounding() {
    use crate::injured_spouse::income_categories::NativeBusinessIncome;
    let base = injured_return((60_000, 5000), (20_000, 500), 0, allocation());
    let mut d = statutory_multiple_return();
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        d.injured_spouse.as_mut().unwrap().injured = injured;
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 86_803);
        assert_eq!(value(&r, "qbi_deduction"), 1361);
        assert_eq!(value(&r, "taxable_income"), 53_242);
        // MFJ53200–53250 TaxTable midpoint53225:
        // 2480 +12%*(53225−24800) =5891;5500paid leaves391.
        assert_eq!(r.tax, Some(589_100));
        assert_eq!(r.balance, Some(39_100));
        let mut facts = liability_portfolio_facts(&base, Some([0; 2]));
        // Independent actual receipt/expense arithmetic, genuine netCents:
        // TP3901.50, SP2901.50 including the50c ordinaryC and50c F.
        facts.business = [390_150, 290_150];
        facts.income_categories.statutory_business = Some(NativeBusinessIncome {
            joint_dollars: [6803, 0],
            w2_cents: [[390_100, 290_100], [0; 2]],
            other_cents: [[50, 0], [0, 50]],
            foreign_exclusion_dollars: [0; 2],
        });
        let projection = crate::injured_spouse::owners::project_sources(
            crate::related_forms::RelatedFormContext::original(
                &d,
                &crate::related_forms::effective_claimants(&d),
            ),
            &r,
            d.injured_spouse.as_ref(),
            &facts,
            None,
        )
        .unwrap();
        assert_eq!(
            projection.agi(),
            if injured == Owner::Taxpayer { [63_902, 22_901] } else { [63_901, 22_902] }
        );
        // Printed class residual is fixedC first; it must not replace that
        // original whole-category financial owner projection.
        assert_income_source_columns(&r, injured, [63_901, 22_901], [1, 0]);
    }
}

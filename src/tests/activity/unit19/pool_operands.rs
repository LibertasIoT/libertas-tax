use super::*;

#[test]
fn unit19_pool_operands_preserve_original_items_and_classes_across_selected_community_views() {
    let mut partner = ordinary_partner_source(8_000, true);
    partner.regular.passive_loss = 600_000;
    partner.regular.section1231 =
        Some(K1Section1231 { passive_loss: 1_400_000, ..k1_section1231(2_000, 0) });
    partner.income = vec![PartnerIncome {
        code: PartnerIncomeCode::SpeciallyAllocatedOrdinary,
        amount: 100_000,
    }];
    let mut alternative = partner.regular.clone();
    alternative.ordinary = 400_000;
    alternative.passive_loss = 300_000;
    alternative.section1231 =
        Some(K1Section1231 { passive_loss: 700_000, ..k1_section1231(1_000, 0) });
    partner.alternative = Some(alternative);
    let mut scorp = former_passive_property_scorp(8_000, 2_000, 6_000, 14_000);
    scorp.income = vec![ScorpIncome { code: ScorpIncomeCode::NonportfolioShort, amount: 100_000 }];
    let mut alternative = amt_records(&scorp);
    alternative.adjustment = -400_000;
    alternative.passive.loss = 300_000;
    alternative.section1231 =
        Some(K1Section1231 { passive_loss: 700_000, ..k1_section1231(1_000, 0) });
    scorp.amt = Some(alternative);
    for community in [false, true] {
        let mut draft = if community {
            community_w2_draft(1_000_000, 1_000_000, 0, 0)
        } else {
            complete(FederalStatus::Single)
        };
        add(&mut draft, income(FederalIncome::Partnership { data: partner.clone() }));
        add(&mut draft, income(FederalIncome::Scorp { data: scorp.clone() }));
        let before = draft.clone();
        for original in &draft.income[draft.income.len() - 2..] {
            let entry = crate::community::entry(&draft, original);
            assert_eq!((entry.id, entry.owner), (original.id, original.owner));
            for amt in [false, true] {
                let (current, releases, passive, nonpassive) = match &entry.income {
                    FederalIncome::Partnership { data } => {
                        let source =
                            if amt { data.alternative.as_ref().unwrap() } else { &data.regular };
                        let pool =
                            crate::community::partner_pool(&draft, entry.id, data, source).unwrap();
                        (
                            [pool.current_ordinary_with_items, pool.current_section1231],
                            pool.combined.characters.unwrap().prior_released,
                            pool.combined.passive,
                            pool.combined.nonpassive,
                        )
                    }
                    FederalIncome::Scorp { data } => {
                        let source = if amt {
                            data.amt.as_ref().unwrap().source(data).unwrap()
                        } else {
                            data.into()
                        };
                        let pool =
                            crate::community::scorp_pool(&draft, entry.id, source, amt).unwrap();
                        (
                            [pool.current_ordinary_with_items, pool.current_section1231],
                            pool.combined.characters.unwrap().prior_released,
                            pool.combined.passive,
                            pool.combined.nonpassive,
                        )
                    }
                    _ => unreachable!(),
                };
                let divisor = if community { 2 } else { 1 };
                // 2026-10-19: actual source operands remain cents before community division.
                let (expected_current, expected_releases, expected_passive) = if amt {
                    ([500_000, 100_000], [300_000, 300_000], -400_000)
                } else {
                    ([900_000, 200_000], [600_000, 500_000], -900_000)
                };
                assert_eq!(current, expected_current.map(|value| value / divisor));
                assert_eq!(releases, expected_releases.map(|value| value / divisor));
                assert_eq!((passive, nonpassive), (expected_passive / divisor, 0));
                assert_ne!(current[0], passive + nonpassive);
            }
        }
        assert_eq!(draft, before, "forwarding preserves actual source facts");
    }
}

#[test]
fn unit19_pool_operands_keep_signed_current_loss_and_known_absent_property() {
    let partner = ordinary_partner_source(-1_000, true);
    let partner_pool = partner.pool(&partner.regular).unwrap();
    let scorp = complete_ordinary_scorp(-1_000, true);
    let scorp_pool = scorp.pool_activity_result().unwrap();
    for (current, property, prior, nonpassive) in [
        (
            partner_pool.current_ordinary_with_items,
            partner_pool.current_section1231,
            partner_pool.combined.prior_released,
            partner_pool.combined.nonpassive,
        ),
        (
            scorp_pool.current_ordinary_with_items,
            scorp_pool.current_section1231,
            scorp_pool.combined.prior_released,
            scorp_pool.combined.nonpassive,
        ),
    ] {
        // 2026-10-19: the builders' -$1,000 current loss is -100,000 source cents.
        assert_eq!((current, property, prior, nonpassive), (-100_000, 0, 0, -100_000));
    }
}

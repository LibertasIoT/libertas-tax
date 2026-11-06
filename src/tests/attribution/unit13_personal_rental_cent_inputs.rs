use super::*;
use crate::tests::support::client::{
    draft_from_native_records, native_draft_records, write_native_draft_records,
};

fn inputs(d: &FederalDraft, alternative: bool) -> crate::extended_rules::RentalResult {
    let mut result = FederalResult::empty(d.revision);
    let pool = crate::extended_rules::rentals_for(d, 100_000, &mut result, alternative);
    assert!(result.issues.is_empty(), "{:?}", result.issues);
    assert!(pool.inputs_complete);
    pool
}

fn fixture() -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    let mut asset = rental_equipment(5000, BonusElection::ElectOut);
    asset.description = "Equipment rented to others".into();
    add(&mut d, income(personal_rent_with_assets(11_000, 2000, vec![asset])));
    let mut second = income(personal_rent_with_history(20_000, 4000, 3000, 5000, false));
    second.owner = crate::DocumentOwner::Spouses;
    add(&mut d, second);
    d
}

#[test]
fn unit13_personal_rental_inputs_keep_real_cents_and_independent_system_history() {
    let mut d = fixture();
    let FederalIncome::NonbusinessProperty { receipts, expenses, kind, .. } =
        &mut d.income[1].income
    else {
        panic!()
    };
    *receipts += 51;
    *expenses += 49;
    let NonbusinessPropertyKind::PersonalRental { history, .. } = kind;
    history.regular = 149;
    history.amt = 249;
    for alternative in [false, true] {
        let pool = inputs(&d, alternative);
        assert_eq!(pool.personal_inputs.len(), 2);
        assert!(!pool.personal_inputs.contains_key(&1));
        let first = &pool.personal_inputs[&2];
        assert_eq!(first.owner, Owner::Taxpayer);
        assert_eq!(first.receipts_cents, 1_100_051);
        assert_eq!(first.regular_expenses_cents, 300_049);
        // Current bonus-eligible property uses the same regular/AMT MACRS.
        assert_eq!(first.expenses_cents, 300_049);
        assert_eq!(
            (first.regular_prior_cents, first.prior_cents),
            (149, if alternative { 249 } else { 149 })
        );
        let second = &pool.personal_inputs[&3];
        assert_eq!(second.owner, Owner::Spouse);
        assert_eq!(
            (second.receipts_cents, second.expenses_cents, second.prior_cents),
            (2_000_000, 400_000, if alternative { 500_000 } else { 300_000 })
        );
        let before = pool.personal_inputs;
        d.income.reverse();
        assert_eq!(inputs(&d, alternative).personal_inputs, before);
        d.income.reverse();
    }
}

#[test]
fn unit13_personal_rental_inputs_keep_actual_separate_amt_depreciation() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, income(personal_rent_with_assets(11_000, 2000, vec![separate_amt_equipment()])));
    // Existing pre-TCJA retained manual ten-year HY schedules: regular200%,
    // AMT150%. The native owner has current65536/87395 cents, not whole
    // annual dollars converted back to cents.
    let regular = inputs(&d, false);
    let alternative = inputs(&d, true);
    assert_eq!(regular.personal_inputs[&2].expenses_cents, 265_536);
    assert_eq!(alternative.personal_inputs[&2].expenses_cents, 287_395);
    assert_eq!(alternative.personal_inputs[&2].regular_expenses_cents, 265_536);
}

#[test]
fn unit13_personal_rental_inputs_do_not_invent_an_allowed_or_zero_cent_loss() {
    let mut d = complete(FederalStatus::Joint);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut entry = income(personal_rent(0, 0));
        entry.owner = crate::DocumentOwner::from(owner);
        let FederalIncome::NonbusinessProperty { expenses, .. } = &mut entry.income else {
            panic!()
        };
        *expenses = 50;
        add(&mut d, entry);
    }
    let source = inputs(&d, false).personal_inputs;
    assert_eq!(source[&2].expenses_cents, 50);
    assert_eq!(source[&3].expenses_cents, 50);
    assert_eq!(source.values().map(|v| v.expenses_cents).sum::<i64>(), 100);
    // This packet is before limits. The original end-to-end failed witness
    // is retained in the owning correction branch; no allowed amount is
    // fabricated from these inputs or from the old rounded suspension.
}

#[test]
fn unit13_personal_rental_inputs_actual_client_save_reopen_restart() {
    extern crate std;
    use crate::tests::cross::interview::{accepted, open_topic};
    let selected = std::env::var("TAX_UNIT13_CENT_INPUT_PROFILE").ok();
    for profile in ["receipt", "expense"] {
        if selected.as_deref().is_some_and(|p| p != profile) {
            continue;
        }
        let mut session = Interview::new(TEST_USER, fixture());
        let mut page = open_topic(&mut session, TaxSection::OtherIncome);
        if let Ok(input) = std::env::var("TAX_UNIT13_CENT_INPUT_CALC_INPUT") {
            let bytes = std::fs::read(input).unwrap();
            let request = P::from_avro(&bytes).unwrap();
            assert_eq!(request.to_avro(), bytes);
            let Reply::Response(response) = send(&mut session, request) else { panic!() };
            std::fs::write(
                std::env::var("TAX_UNIT13_CENT_INPUT_CALC_OUTPUT").unwrap(),
                response.to_avro(),
            )
            .unwrap();
            return;
        }
        if let Ok(dir) = std::env::var("TAX_UNIT13_CENT_INPUT_SEED_DIR") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(std::path::Path::new(&dir).join("seed.avro"), page.to_avro()).unwrap();
            return;
        }
        let request = if let Ok(input) = std::env::var("TAX_UNIT13_CENT_INPUT_PAYLOAD") {
            let bytes = std::fs::read(input).unwrap();
            let request = P::from_avro(&bytes).unwrap();
            assert_eq!(request.to_avro(), bytes);
            request
        } else {
            let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
            let row = &mut documents.documents[if profile == "receipt" { 0 } else { 1 }];
            row.owner = if profile == "receipt" { Owner::Spouse } else { Owner::Taxpayer };
            let crate::IncomeRecord::EquipmentRental(value) = &mut row.document else { panic!() };
            if profile == "receipt" {
                value.receipts = 1_200_000;
            } else {
                value.expenses = 500_000;
            }
            page
        };
        accepted(&send(&mut session, request));
        let check = |d: &FederalDraft| {
            let r = supported(d);
            assert_eq!(value(&r, "agi"), if profile == "receipt" { 122_000 } else { 120_000 });
            assert_eq!(
                value(&r, "nonbusiness_personal_rent_expenses"),
                if profile == "receipt" { 10_000 } else { 11_000 }
            );
            for alternative in [false, true] {
                let pool = inputs(d, alternative);
                if profile == "receipt" {
                    assert_eq!(
                        (pool.personal_inputs[&2].owner, pool.personal_inputs[&2].receipts_cents),
                        (Owner::Spouse, 1_200_000)
                    );
                } else {
                    assert_eq!(
                        (pool.personal_inputs[&3].owner, pool.personal_inputs[&3].expenses_cents),
                        (Owner::Taxpayer, 500_000)
                    );
                }
            }
        };
        check(&session.draft);
        let tax_year = session.draft.tax_year;
        let stored = native_draft_records(&session.draft);
        let draft = draft_from_native_records(tax_year, &stored);
        assert_eq!(draft, session.draft);
        let mut reopened = restart(draft);
        let shown = open_topic(&mut reopened, TaxSection::OtherIncome);
        if let Ok(dir) = std::env::var("TAX_UNIT13_CENT_INPUT_OUTPUT") {
            let dir = std::path::Path::new(&dir);
            std::fs::create_dir_all(dir).unwrap();
            std::fs::write(dir.join("reopened.avro"), shown.to_avro()).unwrap();
            write_native_draft_records(dir.join("persisted.avro"), &stored);
        }
        let accepted_draft = reopened.draft.clone();
        accepted(&send(&mut reopened, shown));
        assert_eq!(reopened.draft, accepted_draft);
        check(&reopened.draft);
    }
}

#[test]
fn unit13_personal_rental_native_cent_pool_preserves_complete_suspensions() {
    use crate::kernel::carry::Cents;
    use crate::tests::support::loss_limits::{PassiveActivity, passive_suspensions_in_units};
    for cents in [49, 50, 51, 149, 150] {
        let activities = [cents, cents].map(|expense| PassiveActivity {
            key: (),
            net: -expense,
            rental_allowance: false,
            ineligible_prior: 0,
        });
        let closing = passive_suspensions_in_units::<Cents>(&activities, 0, 0).unwrap();
        assert_eq!(closing, vec![cents, cents]);
        assert_eq!(2 * cents - closing.iter().sum::<i64>(), 0);
    }
}

#[test]
fn unit13_personal_rental_native_cent_pool_shares_profit_without_special_allowance() {
    use crate::kernel::carry::Cents;
    use crate::tests::support::loss_limits::{PassiveActivity, passive_suspensions_in_units};
    let activities = [50, 100].map(|loss| PassiveActivity {
        key: (),
        net: -loss,
        rental_allowance: false,
        ineligible_prior: 0,
    });
    // 50 cents of passive profit admits50 of the150-cent combined loss.
    // The same native residual allocator keeps33/67 cents suspended.
    let closing = passive_suspensions_in_units::<Cents>(&activities, 50, 0).unwrap();
    assert_eq!(closing, vec![33, 67]);
    assert_eq!([50 - closing[0], 100 - closing[1]], [17, 33]);
    let eligible = [
        PassiveActivity { key: (), net: -10_000, rental_allowance: true, ineligible_prior: 0 },
        PassiveActivity { key: (), net: -10_000, rental_allowance: false, ineligible_prior: 0 },
    ];
    assert_eq!(passive_suspensions_in_units::<Cents>(&eligible, 0, 5000), Some(vec![5000, 10_000]));
}

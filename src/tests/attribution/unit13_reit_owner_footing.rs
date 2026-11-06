use super::*;
use crate::tests::cross::interview::{accepted, from_review, open_topic};
use crate::tests::support::assert::injured_income::reit_owner_footing::{optional, profiles};
use crate::tests::support::builders::injured_income::reit_owner_footing::{Source, source};
use crate::tests::support::client::{draft_from_native_records, native_draft_records};
// The actual native joint REIT category must foot to its retained owner cents.
// A missing producer contribution cannot become the residual spouse's income.

#[test]
fn unit13_k1_reit_source_owner_is_retained_after_native_projection() {
    profiles(&[Source::Partner, Source::Scorp]);
}

#[test]
fn unit13_known_investment_and_estate_reit_owners_stay_available() {
    profiles(&[Source::Investment, Source::Estate]);
}

#[test]
fn unit13_reit_owner_cent_edge_keeps_true_supplement() {
    let mut observations = vec![];
    for kind in [Source::Partner, Source::Scorp] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reverse in [false, true] {
                let mut a = allocation();
                a.injured = injured;
                let mut d = injured_return((160000, 20000), (20000, 8000), 0, a);
                let mut e = source(kind);
                e.owner = crate::DocumentOwner::Yours;
                match &mut e.income {
                    FederalIncome::Partnership { data } => {
                        data.separate.as_mut().unwrap().dividends = 49;
                        data.qbi.as_mut().unwrap().reit_dividends = 49;
                    }
                    FederalIncome::Scorp { data } => {
                        edit_scorp(data).dividends = 49;
                        edit_scorp(data).qbi_statement.as_mut().unwrap().reit_dividends = 49;
                    }
                    _ => panic!(),
                }
                add(&mut d, e);
                for (owner, amount) in [(Owner::Taxpayer, 249), (Owner::Spouse, 301)] {
                    let mut e = income(reit_income(0, 0, true));
                    e.owner = crate::DocumentOwner::from(owner);
                    let FederalIncome::Dividend { data } = &mut e.income else { panic!() };
                    edit_dividend(data).ordinary = amount;
                    edit_dividend(data).section199a =
                        ReitDividends::Reported { amount, eligible: Some(true) };
                    add(&mut d, e);
                }
                let originals = d.income.clone();
                if reverse {
                    d.income.reverse();
                }
                let money = |d: &FederalDraft| {
                    let r = supported(d);
                    // Actual dividends599c report6D once; REIT6D gives1D QBI.
                    // TI180006-32200-1=147805;11600+22%*47005=21941.10→21941D.
                    assert_eq!(value(&r, "agi"), 180006);
                    assert_eq!(return_line(&r, "3b"), 600);
                    assert_eq!(value(&r, "qbi_deduction"), 1);
                    assert_eq!(value(&r, "taxable_income"), 147805);
                    assert_eq!(value(&r, "regular_income_tax"), 21941);
                    assert_eq!(return_line(&r, "23"), 0);
                    assert_eq!(
                        (r.tax, r.payments, r.refund, r.balance),
                        (Some(2194100), Some(2800000), Some(605900), Some(0))
                    );
                    let mut plain = d.clone();
                    plain.injured_spouse = None;
                    let ordinary = supported(&plain);
                    assert_eq!(r.return_lines, ordinary.return_lines);
                    assert_eq!(
                        (r.tax, r.payments, r.refund, r.balance),
                        (ordinary.tax, ordinary.payments, ordinary.refund, ordinary.balance)
                    );
                    r
                };
                money(&d);
                let mut s = restart(d);
                for _ in 0..2 {
                    for topic in [kind.topic(), TaxSection::InterestDividends] {
                        let p = open_topic(&mut s, topic);
                        accepted(&send(&mut s, p));
                    }
                    let Reply::Edit(p) = crate::tests::support::client::native_editor(
                        from_review(&mut s, TaxSection::InjuredSpouse),
                        &[],
                    ) else {
                        panic!()
                    };
                    accepted(&send(&mut s, p));
                    for original in &originals {
                        assert_eq!(
                            s.draft.income.iter().find(|e| e.id == original.id),
                            Some(original)
                        );
                    }
                    money(&s.draft);
                    let tax_year = s.draft.tax_year;
                    let stored = native_draft_records(&s.draft);
                    let draft = draft_from_native_records(tax_year, &stored);
                    assert_eq!(draft, s.draft);
                    assert_eq!(native_draft_records(&draft), stored);
                    s = restart(draft);
                }
                let r = money(&s.draft);
                let qbi = ["form_8379_supplemental_qbi_b", "form_8379_supplemental_qbi_c"]
                    .map(|id| optional(&r, id));
                observations.push((
                    kind,
                    injured,
                    reverse,
                    qbi,
                    noted(&r, "WARN_8379_QBI_OWNER_UNAVAILABLE"),
                    optional(&r, crate::injured_spouse::ESTIMATE_LINE),
                ));
            }
        }
    }
    // The observed owner550c and actual category599c both round6D, but
    // observed TP249c→2D versus true TP298c→3D changes a1D deduction split.
    assert_eq!(crate::kernel::ratio::project_rounded_pair(6, [249, 301], 0), Some([2, 4]));
    assert_eq!(crate::kernel::ratio::project_rounded_pair(6, [298, 301], 0), Some([3, 3]));
    assert!(
        observations.iter().all(|(_, _, _, qbi, warning, benefit)| *qbi == [Some(100), Some(0)]
            && !*warning
            && benefit.is_some()),
        "true3/3D owner weights: the unchanged final injured-first split gives the1D deduction to the injured column: {observations:#?}"
    );
}

use crate::tests::support::builders::additional_medicare::Source as MedicareSource;
use crate::tests::support::builders::home_employee::Source;
use crate::tests::{return_line, supported, value};
use crate::{FederalDraft, FederalIncome, FederalResult, Owner};

pub(in crate::tests) fn injured_home_employee_money(
    d: &FederalDraft,
    source: Source,
    owner: Owner,
) -> FederalResult {
    let r = supported(d);
    assert_eq!(
        return_line(&r, "1a"),
        if matches!(source, Source::OppositeEmployee) { 10_000_000 } else { 25_000_000 }
    );
    assert_eq!(return_line(&r, "2b"), 2_000_000);
    let (agi, taxable, regular, nii, niit, medicare, payroll, total, refund) = match source {
        Source::OppositeEmployee => (280000, 247800, 44668, 20000, 760, 90, 12240, 57758, 2242),
        Source::ExcludedHome => (270000, 237800, 42268, 20000, 760, 0, 0, 43028, 26972),
        Source::TaxableHome => (280000, 247800, 43768, 30000, 1140, 0, 0, 44908, 25092),
        Source::UncoveredEmployee => (275000, 242800, 43468, 20000, 760, 0, 0, 44228, 25772),
        Source::CoveredEmployee if owner == Owner::Taxpayer => {
            (275000, 242800, 43468, 20000, 760, 45, 73, 44346, 25654)
        }
        Source::CoveredEmployee => (275000, 242800, 43468, 20000, 760, 45, 383, 44656, 25344),
    };
    // Native ordinary base237800 is42268; extra5000 wages add1200 at24%.
    // Home taxable10000 capital instead adds1500 at15%.
    assert_eq!(value(&r, "agi"), agi);
    assert_eq!(value(&r, "taxable_income"), taxable);
    assert_eq!(value(&r, "regular_income_tax"), regular);
    assert_eq!(value(&r, "form_8960_net_investment_income"), nii);
    assert_eq!(value(&r, "net_investment_income_tax"), niit);
    assert_eq!(value(&r, "additional_medicare_tax"), medicare);
    assert_eq!(r.tax, Some(total * 100));
    assert_eq!(r.refund, Some(refund * 100));
    let id = d
        .income
        .iter()
        .find(|e| {
            matches!(e.income, FederalIncome::HomeSale { .. } | FederalIncome::EmployeeWages { .. })
        })
        .unwrap()
        .id;
    if source.home() {
        assert_eq!(
            value(&r, "capital_gain"),
            if matches!(source, Source::TaxableHome) { 10000 } else { 0 }
        );
        assert_eq!(
            value(&r, &alloc::format!("home_{id}_excluded_gain")),
            if matches!(source, Source::TaxableHome) { 500000 } else { 225000 }
        );
    } else {
        let covered = matches!(source, Source::CoveredEmployee | Source::OppositeEmployee);
        let pay = if matches!(source, Source::OppositeEmployee) { 16_000_000 } else { 500_000 };
        assert_eq!(return_line(&r, "1g"), if covered { pay } else { 0 });
        assert_eq!(return_line(&r, "1h"), if covered { 0 } else { pay });
        for index in 0..2 {
            let employee = index == usize::from(owner == Owner::Spouse) && covered;
            if employee {
                assert_eq!(value(&r, &alloc::format!("employee_wages_{index}_pay")), pay / 100);
                assert_eq!(
                    value(&r, &alloc::format!("employee_wages_{index}_social_tax")),
                    if matches!(source, Source::OppositeEmployee) {
                        9920
                    } else if owner == Owner::Spouse {
                        310
                    } else {
                        0
                    }
                );
                assert_eq!(
                    value(&r, &alloc::format!("employee_wages_{index}_medicare_tax")),
                    if matches!(source, Source::OppositeEmployee) { 2320 } else { 73 }
                );
                assert_eq!(value(&r, &alloc::format!("employee_wages_{index}_tax")), payroll);
            } else {
                // Native Form8919 rows are printed only for an actual positive
                // covered source. No record/coverage-No supplies no printed
                // tax worksheet; do not invent queried zero rows.
                for suffix in ["pay", "tax"] {
                    assert!(
                        r.lines.iter().all(
                            |line| line.id != alloc::format!("employee_wages_{index}_{suffix}")
                        )
                    );
                }
                for suffix in ["social_tax", "medicare_tax"] {
                    let id = alloc::format!("employee_wages_{index}_{suffix}");
                    assert_eq!(value(&r, &id), 0);
                    assert_eq!(r.lines.iter().find(|line| line.id == id).unwrap().form_line, None);
                }
            }
        }
        if !covered {
            // The native supported scope expressly omits this payroll tax;
            // known compensation ownership does not claim payroll coverage.
            assert!(
                r.review_notes
                    .iter()
                    .any(|note| note.display.key == "WARN_EMPLOYEE_WAGES_PAYROLL_TAX_LEFT_OUT")
            );
        }
    }
    let mut plain = d.clone();
    plain.injured_spouse = None;
    let ordinary = supported(&plain);
    assert_eq!(r.return_lines, ordinary.return_lines);
    assert_eq!((r.tax, r.refund, r.balance), (ordinary.tax, ordinary.refund, ordinary.balance));
    r
}

pub(in crate::tests) fn injured_additional_medicare_money(
    d: &FederalDraft,
    source: MedicareSource,
    intermediate: bool,
) -> FederalResult {
    let r = supported(d);
    // Hand computation: MFJ base tax35932 through211400, then24%.
    // Partner100000 -> SE92350 -> tax14130/half7065, QBI18587.
    // SmallC300 -> ordinarySE277 is below400 -> actualSEnet/tax0.
    // Its actual300000-W2 fixture also reports900 additional withholding.
    let (agi, taxable, regular, se, part_i, part_ii, part_se, total, refund) = match source {
        MedicareSource::PartnerSe if intermediate => {
            (318661, 266729, 49211, 2678, 0, 0, 381, 53030, 46970)
        }
        MedicareSource::PartnerSe => (312935, 262148, 48112, 14130, 0, 0, 381, 63383, 36617),
        MedicareSource::SmallBusiness => (320300, 288040, 54326, 0, 450, 0, 0, 55536, 45364),
        MedicareSource::Railroad => (420000, 387800, 78268, 0, 0, 450, 0, 79478, 20522),
    };
    for (id, expected) in [
        ("agi", agi),
        ("taxable_income", taxable),
        ("regular_income_tax", regular),
        ("self_employment_tax", se),
        ("additional_medicare_tax", part_i + part_ii + part_se),
        ("net_investment_income_tax", 760),
    ] {
        assert_eq!(value(&r, id), expected, "{source:?} {id}");
    }
    assert_eq!(r.tax, Some(total * 100));
    assert_eq!(r.refund, Some(refund * 100));
    assert_eq!(value(&r, "form_8960_net_investment_income"), 20000);
    if matches!(source, MedicareSource::PartnerSe) {
        assert_eq!(
            value(&r, "schedule_1_half_self_employment_tax"),
            if intermediate { 1339 } else { 7065 }
        );
        assert_eq!(value(&r, "qbi_deduction"), if intermediate { 19732 } else { 18587 });
    }
    let mut plain = d.clone();
    plain.injured_spouse = None;
    let ordinary = supported(&plain);
    assert_eq!(ordinary.return_lines, r.return_lines);
    assert_eq!((ordinary.tax, ordinary.refund, ordinary.balance), (r.tax, r.refund, r.balance));
    r
}

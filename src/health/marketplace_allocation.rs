#[cfg(test)]
use crate::marketplace_record_source::MarketplaceFraction;
// Pub974 (2025), pp28–38, Worksheets C–F; i8962 (2025), pp19–21.
// https://www.irs.gov/pub/irs-pdf/p974.pdf
// https://www.irs.gov/pub/irs-pdf/i8962.pdf
// These are transient projections of the canonical policy facts, not a second
// accepted ledger. PartIV display decimals never feed the calculation.
use crate::kernel::ratio::exact::{Exact, ratio, whole};
use crate::{
    MarketplaceAgreement as Agreement, MarketplaceAllocationKind as Kind,
    MarketplaceBenchmarkSource as Benchmark, MarketplaceFamilyRef as Family,
    MarketplaceFamilyShare, MarketplaceMaritalFamilyShare,
};
use alloc::vec::Vec;

pub(crate) enum AllocatedBenchmark {
    SourceShare(Exact),
    FamilyAmount(Exact),
}

pub(crate) struct Allocation {
    pub family: Family,
    pub premium: Exact,
    pub advance: Exact,
    pub benchmark: AllocatedBenchmark,
}

fn agreed(value: &Agreement, default: Exact) -> Option<Exact> {
    match value {
        Agreement::Agreed { share } => share.exact(),
        Agreement::NoAgreement => Some(default),
    }
}

fn default_count_share(family: &Family, counts: &[(Family, usize)]) -> Option<Exact> {
    let count = counts.iter().find(|(f, _)| f == family)?.1;
    let total = counts.iter().try_fold(0_usize, |total, (_, n)| total.checked_add(*n))?;
    // Pub974's Bill/Sharon/Erik example uses33/33/34. The App extends
    // this to defaults: floor each nonresidual whole percentage; the
    // enrolling/remaining family receives the residual.
    // The general floor rule is a source-policy inference, not an IRS
    // stated general rounding rule. Agreed and NoAdvance ratios stay exact.
    let numerator = i64::try_from(count.checked_mul(100)?).ok()?;
    let denominator = i64::try_from(total).ok()?;
    ratio(crate::kernel::ratio::trunc_i64(numerator, denominator), 100)
}

fn unique_families<T>(rows: &[T], family: impl Fn(&T) -> &Family) -> bool {
    rows.iter().enumerate().all(|(i, row)| !rows[..i].iter().any(|p| family(p) == family(row)))
}

// Canonical coverage determines the count-based defaults. The editor cannot
// provide separate, possibly contradictory count or projected-share inputs.
pub(crate) fn allocate(
    kind: &Kind,
    enrolling: &Family,
    counts: &[(Family, usize)],
    benchmark: impl Fn(&Benchmark) -> Option<Exact>,
) -> Option<Vec<Allocation>> {
    if counts.is_empty()
        || counts.iter().any(|(_, n)| *n == 0)
        || !unique_families(counts, |(f, _)| f)
    {
        return None;
    }
    match kind {
        Kind::NoAdvance { families } => {
            if families.len() != counts.len()
                || !unique_families(families, |r| &r.family)
                || families
                    .iter()
                    .any(|r| r.benchmark < 0 || !counts.iter().any(|(f, _)| *f == r.family))
            {
                return None;
            }
            let total: Exact = families.iter().map(|r| whole(r.benchmark)).sum();
            if total == whole(0) {
                return None;
            }
            families
                .iter()
                .map(|row| {
                    Some(Allocation {
                        family: row.family.clone(),
                        premium: whole(row.benchmark) / &total,
                        advance: whole(0),
                        benchmark: AllocatedBenchmark::FamilyAmount(whole(row.benchmark)),
                    })
                })
                .collect()
        }
        Kind::OtherShared { families } => other_shared(families, enrolling, counts),
        Kind::FormerSpouses { pair, agreement, third_families } => {
            let first = agreed(agreement, ratio(1, 2)?)?;
            marital(&pair.one, &pair.two, third_families, counts, first, None)
        }
        Kind::MarriedSeparate { pair, one_benchmark, two_benchmark, third_families } => marital(
            &pair.one,
            &pair.two,
            third_families,
            counts,
            ratio(1, 2)?,
            Some((benchmark(one_benchmark)?, benchmark(two_benchmark)?)),
        ),
    }
}

// Direct entered-fact constraints shared by source validation and allocation.
// An omitted enrolling family retains the ordinary residual. Default fractions
// still come from the actual source coverage counts in allocate(), below.
pub(crate) fn entered_shares_valid(rows: &[MarketplaceFamilyShare], enrolling: &Family) -> bool {
    entered_shares_issue(rows, enrolling).is_none()
}

pub(crate) fn entered_shares_issue(
    rows: &[MarketplaceFamilyShare],
    enrolling: &Family,
) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    // Pub974, Other Taxpayers Allocating With Two or More: failure to
    // agree with all taxpayers selects defaults for everybody. Partial
    // agreements remain source facts, not mixed calculation percentages.
    if rows.iter().any(|row| matches!(row.agreement, Agreement::NoAgreement)) {
        return None;
    }
    let mut total = whole(0);
    for (index, row) in rows.iter().enumerate() {
        let Agreement::Agreed { share } = &row.agreement else { unreachable!() };
        let value = match share.checked_exact() {
            Ok(value) => value,
            Err(issue) => {
                return Some(
                    issue.in_field("share").in_field("agreement").in_list("families", index),
                );
            }
        };
        total += value;
    }
    if total > whole(1) {
        Some(ValidationIssue::new("PTC_AGREED_SHARES_ABOVE_WHOLE").at_field("families"))
    } else if rows.iter().any(|row| &row.family == enrolling) && total != whole(1) {
        Some(ValidationIssue::new("PTC_EXPLICIT_ENROLLER_SHARE_MISMATCH").at_field("families"))
    } else {
        None
    }
}

fn other_shared(
    rows: &[MarketplaceFamilyShare],
    enrolling: &Family,
    counts: &[(Family, usize)],
) -> Option<Vec<Allocation>> {
    if rows.is_empty()
        || !entered_shares_valid(rows, enrolling)
        || !unique_families(rows, |r| &r.family)
        || rows
            .iter()
            .any(|r| &r.family != enrolling && !counts.iter().any(|(f, _)| *f == r.family))
        || counts.iter().any(|(f, _)| f != enrolling && !rows.iter().any(|r| &r.family == f))
    {
        return None;
    }
    let use_agreements = rows.iter().all(|row| matches!(row.agreement, Agreement::Agreed { .. }));
    let mut result = Vec::new();
    let mut other_total = whole(0);
    for row in rows.iter().filter(|r| &r.family != enrolling) {
        let share = if use_agreements {
            agreed(&row.agreement, default_count_share(&row.family, counts)?)?
        } else {
            default_count_share(&row.family, counts)?
        };
        other_total += &share;
        result.push(Allocation {
            family: row.family.clone(),
            premium: share.clone(),
            advance: share.clone(),
            benchmark: AllocatedBenchmark::SourceShare(share),
        });
    }
    let remaining = whole(1) - other_total;
    if remaining < whole(0) {
        return None;
    }
    if let Some(MarketplaceFamilyShare { agreement: Agreement::Agreed { share }, .. }) =
        rows.iter().find(|r| &r.family == enrolling)
        && use_agreements
        && share.exact()? != remaining
    {
        return None;
    }
    result.push(Allocation {
        family: enrolling.clone(),
        premium: remaining.clone(),
        advance: remaining.clone(),
        benchmark: AllocatedBenchmark::SourceShare(remaining),
    });
    Some(result)
}

fn marital(
    one: &Family,
    two: &Family,
    third: &[MarketplaceMaritalFamilyShare],
    counts: &[(Family, usize)],
    first: Exact,
    family_benchmarks: Option<(Exact, Exact)>,
) -> Option<Vec<Allocation>> {
    if one == two
        || !unique_families(third, |r| &r.family)
        || third.iter().any(|r| &r.family == one || &r.family == two)
        || counts.iter().any(|(f, _)| f != one && f != two && !third.iter().any(|r| &r.family == f))
    {
        return None;
    }
    let second = whole(1) - &first;
    let mut one_total = whole(0);
    let mut two_total = whole(0);
    let mut result = Vec::new();
    for row in third {
        let default = default_count_share(&row.family, counts)?;
        let a = agreed(&row.one, default.clone())?;
        let b = agreed(&row.two, default)?;
        one_total += &a;
        two_total += &b;
        let share = &first * &a + &second * &b;
        let allocated = match &family_benchmarks {
            Some((one, two)) => AllocatedBenchmark::FamilyAmount(one * &a + two * &b),
            None => AllocatedBenchmark::SourceShare(share.clone()),
        };
        result.push(Allocation {
            family: row.family.clone(),
            premium: share.clone(),
            advance: share,
            benchmark: allocated,
        });
    }
    if one_total > whole(1) || two_total > whole(1) {
        return None;
    }
    for (family, base, used, index) in [(one, first, one_total, 0), (two, second, two_total, 1)] {
        let residual = whole(1) - used;
        let share = base * &residual;
        let allocated = match &family_benchmarks {
            Some((one, two)) => {
                AllocatedBenchmark::FamilyAmount(if index == 0 { one } else { two } * residual)
            }
            None => AllocatedBenchmark::SourceShare(share.clone()),
        };
        result.push(Allocation {
            family: family.clone(),
            premium: share.clone(),
            advance: share,
            benchmark: allocated,
        });
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MarketplaceMaritalPair, MarketplaceNoAdvanceShare};
    use alloc::vec;

    fn share(numerator: u32, denominator: u32) -> Agreement {
        Agreement::Agreed { share: MarketplaceFraction { numerator, denominator } }
    }

    #[test]
    fn gary_no_advance_ratio_uses_source_benchmarks_exactly() {
        let other = Family::Other { family: 1 };
        let kind = Kind::NoAdvance {
            families: vec![
                MarketplaceNoAdvanceShare { family: Family::ThisReturn, benchmark: 1_200_000 },
                MarketplaceNoAdvanceShare { family: other.clone(), benchmark: 600_000 },
            ],
        };
        let rows =
            allocate(&kind, &Family::ThisReturn, &[(Family::ThisReturn, 1), (other, 1)], |_| None)
                .unwrap();
        assert_eq!(
            crate::kernel::ratio::exact::dollars(&(whole(1_500_000) * &rows[0].premium)),
            Some(10_000)
        );
        assert_eq!(rows.iter().map(|r| &r.premium).sum::<Exact>(), whole(1));
        assert!(rows.iter().all(|r| r.advance == whole(0)));
    }

    #[test]
    fn worksheets_c_and_d_preserve_unequal_third_family_agreements() {
        // Pub974 (2025), pp30–31: Kara30%, David70%; Lydia gets80% of
        // Kara's share and50% of David's. Kara6%, David35%, Lydia59%.
        let david = Family::Other { family: 1 };
        let lydia = Family::Other { family: 2 };
        let kind = Kind::FormerSpouses {
            pair: MarketplaceMaritalPair { one: Family::ThisReturn, two: david.clone() },
            agreement: share(30, 100),
            third_families: vec![MarketplaceMaritalFamilyShare {
                family: lydia.clone(),
                one: share(80, 100),
                two: share(50, 100),
            }],
        };
        let rows = allocate(
            &kind,
            &Family::ThisReturn,
            &[(Family::ThisReturn, 1), (david.clone(), 1), (lydia.clone(), 2)],
            |_| None,
        )
        .unwrap();
        for (family, n) in [(Family::ThisReturn, 6), (david, 35), (lydia, 59)] {
            let row = rows.iter().find(|r| r.family == family).unwrap();
            assert_eq!(row.premium, ratio(n, 100).unwrap());
            assert_eq!(row.advance, row.premium);
            assert!(
                matches!(&row.benchmark, AllocatedBenchmark::SourceShare(v) if *v == row.premium)
            );
        }
        assert_eq!(rows.iter().map(|r| &r.premium).sum::<Exact>(), whole(1));
    }

    #[test]
    fn worksheets_e_and_f_keep_family_benchmarks_separate_from_half_shares() {
        let spouse = Family::Other { family: 1 };
        let third = Family::Other { family: 2 };
        let kind = Kind::MarriedSeparate {
            pair: MarketplaceMaritalPair { one: Family::ThisReturn, two: spouse.clone() },
            one_benchmark: Benchmark::Marketplace { amount: 60_000 },
            two_benchmark: Benchmark::Marketplace { amount: 80_000 },
            third_families: vec![MarketplaceMaritalFamilyShare {
                family: third.clone(),
                one: share(1, 5),
                two: share(3, 5),
            }],
        };
        let rows = allocate(
            &kind,
            &Family::ThisReturn,
            &[(Family::ThisReturn, 1), (spouse.clone(), 1), (third.clone(), 1)],
            |b| match b {
                Benchmark::Marketplace { amount } => Some(whole(*amount)),
                _ => None,
            },
        )
        .unwrap();
        for (family, numerator, benchmark) in
            [(Family::ThisReturn, 4, 48_000), (spouse, 2, 32_000), (third, 4, 60_000)]
        {
            let row = rows.iter().find(|r| r.family == family).unwrap();
            assert_eq!(row.premium, ratio(numerator, 10).unwrap());
            assert!(
                matches!(&row.benchmark, AllocatedBenchmark::FamilyAmount(v) if *v == whole(benchmark))
            );
        }
    }

    #[test]
    fn published_three_family_defaults_floor_others_and_preserve_enroller_residual() {
        // Pub974, Other Taxpayers Allocating Policy Amounts With Two or
        // More Other Taxpayers, Example2: Erik is the actual enroller.
        let sharon = Family::Other { family: 1 };
        let erik = Family::Other { family: 2 };
        let counts = [(Family::ThisReturn, 1), (sharon.clone(), 1), (erik.clone(), 1)];
        let kind = Kind::OtherShared {
            families: counts
                .iter()
                .map(|(family, _)| MarketplaceFamilyShare {
                    family: family.clone(),
                    agreement: Agreement::NoAgreement,
                })
                .collect(),
        };
        let rows = allocate(&kind, &erik, &counts, |_| None).unwrap();
        for (family, percent, premium, benchmark, advance) in [
            (Family::ThisReturn, 33, 264_000, 297_000, 148_500),
            (sharon, 33, 264_000, 297_000, 148_500),
            (erik, 34, 272_000, 306_000, 153_000),
        ] {
            let row = rows.iter().find(|row| row.family == family).unwrap();
            assert_eq!(row.premium, ratio(percent, 100).unwrap());
            assert_eq!(whole(800_000) * &row.premium, whole(premium));
            assert_eq!(whole(450_000) * &row.advance, whole(advance));
            assert!(matches!(&row.benchmark, AllocatedBenchmark::SourceShare(share)
                if whole(900_000) * share == whole(benchmark)));
        }
        assert_eq!(rows.iter().map(|row| &row.premium).sum::<Exact>(), whole(1));
    }

    #[test]
    fn many_family_default_residual_stays_nonnegative_and_agreements_stay_exact() {
        // Nearest whole percentages would allocate130% to65other
        // one-person families; dispatcher floor policy preserves the source.
        let counts: Vec<_> = (1..=66).map(|family| (Family::Other { family }, 1)).collect();
        let enrolling = counts[65].0.clone();
        let mut kind = Kind::OtherShared {
            families: counts
                .iter()
                .map(|(family, _)| MarketplaceFamilyShare {
                    family: family.clone(),
                    agreement: Agreement::NoAgreement,
                })
                .collect(),
        };
        let rows = allocate(&kind, &enrolling, &counts, |_| None).unwrap();
        assert_eq!(rows.iter().map(|row| &row.premium).sum::<Exact>(), whole(1));
        for row in rows {
            assert_eq!(
                row.premium,
                ratio(if row.family == enrolling { 35 } else { 1 }, 100).unwrap()
            );
            assert!(row.premium >= whole(0));
        }
        let Kind::OtherShared { families } = &mut kind else { unreachable!() };
        for row in families {
            row.agreement = share(1, 66);
        }
        let agreed = allocate(&kind, &enrolling, &counts, |_| None).unwrap();
        assert!(agreed.iter().all(|row| row.premium == ratio(1, 66).unwrap()));
    }

    #[test]
    fn marital_default_substeps_floor_others_and_keep_each_remaining_share() {
        let spouse = Family::Other { family: 1 };
        let third = Family::Other { family: 2 };
        let counts = [(Family::ThisReturn, 1), (spouse.clone(), 1), (third.clone(), 1)];
        for kind in [
            Kind::FormerSpouses {
                pair: MarketplaceMaritalPair { one: Family::ThisReturn, two: spouse.clone() },
                agreement: Agreement::NoAgreement,
                third_families: vec![MarketplaceMaritalFamilyShare {
                    family: third.clone(),
                    one: Agreement::NoAgreement,
                    two: Agreement::NoAgreement,
                }],
            },
            Kind::MarriedSeparate {
                pair: MarketplaceMaritalPair { one: Family::ThisReturn, two: spouse.clone() },
                one_benchmark: Benchmark::Marketplace { amount: 60_000 },
                two_benchmark: Benchmark::Marketplace { amount: 80_000 },
                third_families: vec![MarketplaceMaritalFamilyShare {
                    family: third.clone(),
                    one: Agreement::NoAgreement,
                    two: Agreement::NoAgreement,
                }],
            },
        ] {
            let rows = allocate(&kind, &Family::ThisReturn, &counts, |source| match source {
                Benchmark::Marketplace { amount } => Some(whole(*amount)),
                _ => None,
            })
            .unwrap();
            assert_eq!(rows.iter().map(|row| &row.premium).sum::<Exact>(), whole(1));
            for row in rows {
                assert_eq!(
                    row.premium,
                    ratio(if row.family == third { 66 } else { 67 }, 200).unwrap()
                );
                if let AllocatedBenchmark::FamilyAmount(amount) = row.benchmark {
                    assert_eq!(
                        amount,
                        whole(if row.family == third {
                            46_200
                        } else if row.family == Family::ThisReturn {
                            40_200
                        } else {
                            53_600
                        })
                    );
                }
            }
        }
    }

    #[test]
    fn default_and_agreed_shares_require_a_complete_nonduplicated_partition() {
        let other = Family::Other { family: 1 };
        let counts = [(Family::ThisReturn, 2), (other.clone(), 1)];
        let mut kind = Kind::OtherShared {
            families: vec![
                MarketplaceFamilyShare {
                    family: Family::ThisReturn,
                    agreement: Agreement::NoAgreement,
                },
                MarketplaceFamilyShare { family: other, agreement: Agreement::NoAgreement },
            ],
        };
        let rows = allocate(&kind, &Family::ThisReturn, &counts, |_| None).unwrap();
        assert_eq!(
            rows.iter().find(|r| r.family == Family::ThisReturn).unwrap().premium,
            ratio(67, 100).unwrap()
        );
        let Kind::OtherShared { families } = &mut kind else { unreachable!() };
        families[0].agreement = share(67, 100);
        // NoAgreement from the other taxpayer selects defaults for all,
        // so this isolated partial agreement never changes the ratio.
        let partial = allocate(&kind, &Family::ThisReturn, &counts, |_| None).unwrap();
        assert_eq!(
            partial.iter().find(|r| r.family == Family::ThisReturn).unwrap().premium,
            ratio(67, 100).unwrap()
        );
        let Kind::OtherShared { families } = &mut kind else { unreachable!() };
        families[0].agreement = share(2, 3);
        families[1].family = Family::ThisReturn;
        assert!(allocate(&kind, &Family::ThisReturn, &counts, |_| None).is_none());
    }

    #[test]
    fn native_allocation_uses_full_fraction_width_and_exact_benchmark_pool() {
        let fraction = MarketplaceFraction { numerator: u32::MAX, denominator: u32::MAX };
        assert_eq!(fraction.exact(), Some(whole(1)));
        let other = Family::Other { family: 1 };
        let kind = Kind::NoAdvance {
            families: vec![
                MarketplaceNoAdvanceShare { family: Family::ThisReturn, benchmark: i64::MAX },
                MarketplaceNoAdvanceShare { family: other.clone(), benchmark: i64::MAX },
            ],
        };
        let rows = allocate(
            &kind,
            &Family::ThisReturn,
            &[(Family::ThisReturn, 101), (other, 101)],
            |_| None,
        )
        .unwrap();
        assert_eq!(rows[0].premium, ratio(1, 2).unwrap());
        assert_eq!(rows.iter().map(|row| &row.premium).sum::<Exact>(), whole(1));
    }
}

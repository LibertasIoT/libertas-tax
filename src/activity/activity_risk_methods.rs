// Coverage: Unit19 source-owned Form6198 method and financing computations.
// Sources: I6198 (Nov2025), PartsII/III and the line12 worksheet;
// Pub925 (2025), AmountsAtRisk/QNF/Recapture. Full integration is in progress.
use crate::kernel::{carry, phase, ratio::checked_sum};
use crate::*;
use alloc::vec::Vec;

fn money(value: i64) -> bool {
    value >= 0
}
fn date(value: u32) -> bool {
    value <= crate::year_constants::TAX_YEAR_END_DATE && crate::federal_rules::valid_date(value)
}

impl RiskEffectiveCategory {
    pub(crate) fn effective_date(&self) -> u32 {
        match self {
            Self::Film | Self::Farming | Self::Leased1245 | Self::OilGas => 19760101,
            Self::Geothermal => 19781001,
            Self::HoldingRealProperty => 19870101,
            Self::OtherTradeBusiness | Self::HoldingMineralProperty => 19790101,
        }
    }
    fn original_category(&self) -> bool {
        matches!(
            self,
            Self::Film | Self::Farming | Self::Leased1245 | Self::OilGas | Self::Geothermal
        )
    }
}
impl RiskQnfFacts {
    pub(crate) fn valid(&self) -> bool {
        match self.collateral {
            RiskQnfCollateral::OnlyRealPropertyOrIncidentalPersonal => true,
            RiskQnfCollateral::Other { total_gross_fmv, other_gross_fmv } => {
                money(total_gross_fmv)
                    && money(other_gross_fmv)
                    && total_gross_fmv > 0
                    && other_gross_fmv <= total_gross_fmv
            }
        }
    }
    // The interested-lender exception asks whether the loan would be QNF
    // without personal liability; actual ordinary QNF requires that fact.
    pub(crate) fn qualifies(&self, hypothetical_nonrecourse: bool) -> bool {
        let lender = match self.lender {
            RiskQnfLender::GovernmentLoanOrGuarantee => true,
            RiskQnfLender::RegularLender { seller_or_fee_recipient } => !seller_or_fee_recipient,
            RiskQnfLender::RelatedRegularLender {
                commercial_terms_match,
                seller_or_fee_recipient,
            } => commercial_terms_match && !seller_or_fee_recipient,
            RiskQnfLender::Other => false,
        };
        let collateral = match self.collateral {
            RiskQnfCollateral::OnlyRealPropertyOrIncidentalPersonal => true,
            RiskQnfCollateral::Other { total_gross_fmv, other_gross_fmv } => {
                i128::from(other_gross_fmv) * 10 < i128::from(total_gross_fmv)
            }
        };
        self.holding_real_property
            && !self.mineral_property
            && self.secured_real_property
            && (hypothetical_nonrecourse || self.no_personal_liability)
            && !self.convertible
            && lender
            && collateral
    }
}
impl RiskFinancing {
    pub(crate) fn valid(&self) -> bool {
        let terms = match &self.terms {
            RiskFinancingTerms::Recourse {
                protected,
                interested_lender,
                real_property_exception,
            }
            | RiskFinancingTerms::OutsideCollateral {
                protected,
                interested_lender,
                real_property_exception,
                ..
            } => {
                money(*protected)
                    && *protected <= self.amount
                    && (!*interested_lender || self.incurred.is_some())
                    && real_property_exception
                        .as_ref()
                        .is_none_or(|q| q.valid() && self.incurred.is_some())
            }
            RiskFinancingTerms::Nonrecourse { qnf } => {
                qnf.as_ref().is_none_or(|q| q.valid() && self.incurred.is_some())
            }
            RiskFinancingTerms::OtherProtected => true,
        };
        !self.description.trim().is_empty()
            && money(self.amount)
            && self.incurred.is_none_or(date)
            && terms
            && match self.terms {
                RiskFinancingTerms::OutsideCollateral { collateral, .. } => money(collateral),
                _ => true,
            }
    }
    fn interested_allowed(
        &self,
        interested: bool,
        exception: Option<&RiskQnfFacts>,
        category: &RiskEffectiveCategory,
    ) -> bool {
        if !interested {
            return true;
        }
        let Some(incurred) = self.incurred else {
            return false;
        };
        // The cutoff applies to category(6); it does not exempt old farm/film
        // loans. The real-property exception applies only to post-cutoff debt.
        (!category.original_category() && incurred <= 20040503)
            || (incurred > 20040503 && exception.is_some_and(|q| q.qualifies(true)))
    }
    pub(crate) fn eligible(&self, category: &RiskEffectiveCategory) -> i64 {
        match &self.terms {
            RiskFinancingTerms::Recourse {
                protected,
                interested_lender,
                real_property_exception,
            } => {
                if self.interested_allowed(
                    *interested_lender,
                    real_property_exception.as_ref(),
                    category,
                ) {
                    carry::native_remaining::<carry::Cents>(self.amount, *protected).get()
                } else {
                    0
                }
            }
            RiskFinancingTerms::OutsideCollateral {
                collateral,
                protected,
                interested_lender,
                real_property_exception,
            } => {
                if self.interested_allowed(
                    *interested_lender,
                    real_property_exception.as_ref(),
                    category,
                ) {
                    carry::native_remaining::<carry::Cents>(
                        phase::cap(self.amount, [*collateral]),
                        *protected,
                    )
                    .get()
                } else {
                    0
                }
            }
            RiskFinancingTerms::Nonrecourse { qnf } => {
                if qnf.as_ref().is_some_and(|q| q.qualifies(false)) { self.amount } else { 0 }
            }
            RiskFinancingTerms::OtherProtected => 0,
        }
    }
}
impl RiskHistoricalInvestment {
    pub(crate) fn valid(&self) -> bool {
        [
            self.cash,
            self.inventory,
            self.depreciable_basis,
            self.depreciation,
            self.depletable_basis,
            self.depletion,
            self.land_basis,
            self.other_basis,
        ]
        .into_iter()
        .all(money)
            && self.depreciation <= self.depreciable_basis
            && self.depletion <= self.depletable_basis
            && match self.accounting {
                RiskHistoricalAccounting::Cash => true,
                RiskHistoricalAccounting::Accrual { receivable, reserve, payable } => {
                    [receivable, reserve, payable].into_iter().all(money) && reserve <= receivable
                }
            }
    }
    pub(crate) fn investment(&self) -> Option<i64> {
        let accounting = match self.accounting {
            RiskHistoricalAccounting::Cash => 0,
            RiskHistoricalAccounting::Accrual { receivable, reserve, payable } => {
                checked_sum([receivable, -reserve, -payable])?
            }
        };
        checked_sum([
            self.cash,
            self.inventory,
            carry::native_balance(self.depreciable_basis, self.depreciation),
            carry::native_balance(self.depletable_basis, self.depletion),
            self.land_basis,
            self.other_basis,
            accounting,
        ])
    }
}

// Keep the published year-by-year order. A later high excluded balance does
// not restore loss already consumed by an earlier worksheet row.
pub(crate) fn earlier_loss_worksheet(rows: &[RiskEarlierLoss]) -> i64 {
    let mut rows: Vec<_> = rows.iter().collect();
    rows.sort_by_key(|row| row.year);
    rows.into_iter().fold(0, |used, row| {
        used + phase::cap(
            row.loss,
            [carry::native_remaining::<carry::Cents>(row.not_at_risk, used).get()],
        )
    })
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct MethodBalance {
    // Signed balances are retained for economic closing risk; only statutory
    // capacity and the printed b-lines receive a zero floor.
    pub capacity: i64,
    pub simplified: Option<i64>,
    pub detailed: Option<i64>,
    pub lines: Vec<(&'static str, i64)>,
}
fn financing_movements(
    rows: &[RiskFinancing],
    category: &RiskEffectiveCategory,
) -> Option<(i64, i64, i64)> {
    let amount = |movement| {
        checked_sum(rows.iter().filter(|row| row.movement == movement).map(|row| {
            let eligible = row.eligible(category);
            if matches!(movement, RiskFinancingMovement::IncludedInOpeningBasis) {
                carry::native_balance(row.amount, eligible)
            } else {
                eligible
            }
        }))
    };
    Some((
        amount(RiskFinancingMovement::IncludedInOpeningBasis)?,
        amount(RiskFinancingMovement::IncreaseInInterval)?,
        amount(RiskFinancingMovement::DecreaseInInterval)?,
    ))
}
impl RiskSimplified {
    fn valid(&self) -> bool {
        [self.basis_start, self.increases, self.decreases].into_iter().all(money)
    }
    fn compute(
        &self,
        financing: &[RiskFinancing],
        category: &RiskEffectiveCategory,
    ) -> Option<MethodBalance> {
        let (excluded, added, repaid) = financing_movements(financing, category)?;
        let l7 = checked_sum([self.increases, added])?;
        let l8 = checked_sum([self.basis_start, l7])?;
        let l9 = checked_sum([self.decreases, excluded, repaid])?;
        let l10a = carry::native_balance(l8, l9);
        let capacity = phase::floor(l10a, 0);
        Some(MethodBalance {
            capacity,
            simplified: Some(l10a),
            detailed: None,
            lines: alloc::vec![
                ("6", self.basis_start),
                ("7", l7),
                ("8", l8),
                ("9", l9),
                ("10a", l10a),
                ("10b", capacity)
            ],
        })
    }
}
impl RiskDetailed {
    fn interval_start(&self, category: &RiskEffectiveCategory) -> u32 {
        match self.opening {
            RiskDetailedOpening::PriorDetailed { .. } => crate::year_constants::TAX_YEAR_START_DATE,
            RiskDetailedOpening::BeganAfterEffectiveDate { began } => began,
            RiskDetailedOpening::AtEffectiveDate { .. } => category.effective_date(),
        }
    }
    fn valid(&self, category: &RiskEffectiveCategory) -> bool {
        let opening = match &self.opening {
            RiskDetailedOpening::PriorDetailed { prior } => {
                prior.year == crate::year_constants::RISK_PRIOR_YEAR
                    && money(prior.regular)
                    && money(prior.amt)
            }
            RiskDetailedOpening::BeganAfterEffectiveDate { began } => {
                date(*began) && *began >= category.effective_date()
            }
            RiskDetailedOpening::AtEffectiveDate {
                category: actual,
                began,
                investment,
                collateral,
                earlier_losses,
                excluded_other,
            } => {
                actual == category
                    && date(*began)
                    && *began < category.effective_date()
                    && investment.valid()
                    && money(*collateral)
                    && money(*excluded_other)
                    && earlier_losses.iter().enumerate().all(|(i, row)| {
                        row.year >= 1900
                            && row.year < category.effective_date() / 10000
                            && money(row.loss)
                            && money(row.not_at_risk)
                            && !earlier_losses[..i].iter().any(|old| old.year == row.year)
                    })
            }
        };
        opening
            && money(self.increases)
            && money(self.decreases)
            && self.depletion.iter().enumerate().all(|(i, row)| {
                row.year >= self.interval_start(category) / 10000
                    && row.year <= crate::year_constants::RISK_TAX_YEAR
                    && money(row.depletion)
                    && money(row.basis)
                    && !self.depletion[..i].iter().any(|old| old.year == row.year)
            })
    }
    fn compute(
        &self,
        financing: &[RiskFinancing],
        category: &RiskEffectiveCategory,
        amt: bool,
    ) -> Option<MethodBalance> {
        let (_, added, repaid) = financing_movements(financing, category)?;
        let mut lines = Vec::new();
        let l15 = match &self.opening {
            RiskDetailedOpening::PriorDetailed { prior } => {
                let amount = if amt { prior.amt } else { prior.regular };
                lines.push(("15b", amount));
                amount
            }
            RiskDetailedOpening::BeganAfterEffectiveDate { .. } => {
                lines.push(("15a", 0));
                0
            }
            RiskDetailedOpening::AtEffectiveDate {
                investment,
                collateral,
                earlier_losses,
                excluded_other,
                ..
            } => {
                let l11 = investment.investment()?;
                let l12 = checked_sum([*collateral, earlier_loss_worksheet(earlier_losses)])?;
                // Historical outside collateral belongs on line12, while the
                // full nonrecourse principal is excluded on line14. Do not add
                // that same collateral a second time via net loan eligibility.
                let excluded = checked_sum(
                    financing
                        .iter()
                        .filter(|row| {
                            matches!(row.movement, RiskFinancingMovement::IncludedInOpeningBasis)
                        })
                        .map(|row| {
                            if matches!(row.terms, RiskFinancingTerms::OutsideCollateral { .. }) {
                                row.amount
                            } else {
                                carry::native_balance(row.amount, row.eligible(category))
                            }
                        }),
                )?;
                let l14 = checked_sum([*excluded_other, excluded])?;
                let l13 = checked_sum([l11, l12])?;
                let l15 = phase::floor(carry::native_balance(l13, l14), 0);
                lines.extend([("11", l11), ("12", l12), ("13", l13), ("14", l14), ("15a", l15)]);
                l15
            }
        };
        let excess =
            checked_sum(self.depletion.iter().map(|row| {
                carry::native_remaining::<carry::Cents>(row.depletion, row.basis).get()
            }))?;
        let l16 = checked_sum([self.increases, added, excess])?;
        let l17 = checked_sum([l15, l16])?;
        let l18 = checked_sum([self.decreases, repaid])?;
        let l19a = carry::native_balance(l17, l18);
        let capacity = phase::floor(l19a, 0);
        lines.extend([("16", l16), ("17", l17), ("18", l18), ("19a", l19a), ("19b", capacity)]);
        Some(MethodBalance { capacity, simplified: None, detailed: Some(l19a), lines })
    }
}
impl RiskMethod {
    fn valid(&self, category: &RiskEffectiveCategory) -> bool {
        match self {
            Self::Simplified { facts } => facts.valid(),
            Self::Detailed { facts } => facts.valid(category),
            Self::Compare { simplified, detailed } => {
                simplified.valid() && detailed.valid(category)
            }
        }
    }
    fn compute(
        &self,
        financing: &[RiskFinancing],
        category: &RiskEffectiveCategory,
        amt: bool,
    ) -> Option<MethodBalance> {
        match self {
            Self::Simplified { facts } => facts.compute(financing, category),
            Self::Detailed { facts } => facts.compute(financing, category, amt),
            Self::Compare { simplified, detailed } => {
                let mut s = simplified.compute(financing, category)?;
                let d = detailed.compute(financing, category, amt)?;
                s.capacity = phase::floor(s.capacity, d.capacity);
                s.detailed = d.detailed;
                s.lines.extend(d.lines);
                Some(s)
            }
        }
    }
}
impl RiskRecaptureAmounts {
    fn valid(&self) -> bool {
        money(self.deducted_post1978)
            && money(self.recaptured_before)
            && self.recaptured_before <= self.deducted_post1978
    }
}
impl ActivityRiskRecord {
    pub(crate) fn valid(&self, category: &RiskEffectiveCategory) -> bool {
        self.regular.valid(category)
            && match &self.amt {
                RiskAmtMethod::SameFactsAsRegular => true,
                RiskAmtMethod::Separate { facts } => facts.valid(category),
            }
            && self.financing.iter().all(RiskFinancing::valid)
            && self.recapture.as_ref().is_none_or(|history| {
                history.regular.valid()
                    && history.amt.valid()
                    && history.floor_1978 <= 0
                    && (history.floor_1978 == 0 || category.effective_date() <= 19781231)
            })
    }
    pub(crate) fn compute(
        &self,
        category: &RiskEffectiveCategory,
        amt: bool,
    ) -> Option<MethodBalance> {
        let method = if amt {
            match &self.amt {
                RiskAmtMethod::SameFactsAsRegular => &self.regular,
                RiskAmtMethod::Separate { facts } => facts,
            }
        } else {
            &self.regular
        };
        method.compute(&self.financing, category, amt)
    }
    // The caller must close the actual economic account with the canonical
    // loss_limits owner after the selected Part I allowance is established.
}

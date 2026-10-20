// Coverage: Form 6251 line 2f (ATNOLD) — asked with the earlier-year operating-loss kind
//   (u5-losses, 2026-09-28; see nol.rs). 90% of AMTI without the ATNOLD for ordinary losses,
//   the documented pre-2018 exceptions to 100%; ATNOL_EXCEPTION_AFTER_2017 on both sides.
//   Tests: src/tests/federal/losses.rs amt_loss_stops_at_90_percent_of_amti and the hand fixture.
//   Source: i6251 draft (line 2f), fetched 2026-09-28.
use crate::FederalResult;
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::lines::{line, line_cents};
use crate::kernel::phase;
use crate::kernel::ratio::exact::{self, Exact};
use crate::kernel::ratio::{fraction, positive};
use crate::nol::LossAmountIssue;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Prior alternative-tax net operating losses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AtnolHistory {
    /// None
    None,
    /// Retained AMT loss carryforward records
    Records {
        /// Loss years
        /// One entry per calendar loss year. Use the remaining ATNOL after every required carryback and intervening return, not the original loss or the regular-tax NOL. Input order does not control use.
        #[libertas_size(min = 1)]
        years: Vec<AtnolYear>,
        /// Complete same-taxpayer history and unchanged filing unit
        /// Confirm the same filer, or the same married couple filing jointly, throughout all involved years; separate filers use only their own losses with unchanged filing status. No deceased-owner transfer, estate/trust allocation, foreign-source loss, farming carryback, unresolved earlier return, prior-year debt-discharge reduction or mixed ordinary/special-exception components in one loss year. Carryback elections and remaining balances are already reconciled. Regular NOL balances and deductions are entered separately. Prior Forms 6251 and their ATNOL worksheets show it. No leaves this AMT loss history out of the estimate with a notice; that situation is not supported.
        confirmed: bool,
    },
}

/// AMT loss from a retained return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct AtnolYear {
    /// Calendar loss year
    // Tax structure: this date or year belongs to the stated current or retained-return period.
    #[libertas_number(min = 1900, max = 2025)]
    pub year: u32,
    /// Remaining alternative-tax loss entering 2026
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub balance: i64,
    /// Documented exception to the ordinary AMT loss limit
    /// Yes only when the entire remaining loss is a qualified disaster, Gulf Opportunity Zone, Kansas recovery assistance, Midwestern disaster recovery assistance loss, or a 2008/2009 loss with the documented extended-carryback election described in the Form 6251 instructions. This is a source-return classification, not an election to claim a larger deduction now. Mixed components need their own allocation worksheet.
    pub special_exception: bool,
}

#[derive(Default)]
pub(crate) struct AtnolResult {
    pub deduction: i64,
    pub carryforward: Exact,
    years: Vec<crate::nol::LossUse>,
    estate_source: Option<i64>,
    has_estate: bool,
}
impl AtnolYear {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.special_exception && self.year >= 2018).then(|| {
            crate::validation::ValidationIssue::new("ATNOL_EXCEPTION_AFTER_2017")
                .at_field("special_exception")
        })
    }
}
impl AtnolHistory {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::None => true,
            Self::Records { years, .. } => {
                !years.is_empty()
                    && years.iter().enumerate().all(|(i, y)| {
                        (1900..=2025).contains(&y.year)
                            // Arithmetic safety: unique 1900..2025 source years
                            // keep the unchecked vintage sums within native i64.
                            && (0..=100000000000).contains(&y.balance)
                            && y.issue().is_none()
                            && !years[..i].iter().any(|prior| prior.year == y.year)
                    })
            }
        }
    }
    pub(crate) fn present(&self) -> bool {
        matches!(self, Self::Records { .. })
    }
    pub(crate) fn confirmed(&self) -> bool {
        matches!(self, Self::None | Self::Records { confirmed: true, .. })
    }
    pub(crate) fn calculate(
        base: i64,
        modified_base: i64,
        estate_source: Option<i64>,
        has_estate: bool,
        vintages: &[crate::estate_trust::LossVintage],
        r: &mut FederalResult,
    ) -> Result<alloc::boxed::Box<AtnolResult>, LossAmountIssue> {
        // The source owner already omitted only unconfirmed histories.
        if vintages.is_empty() {
            return Ok(alloc::boxed::Box::default());
        }
        let issue = || LossAmountIssue::new(estate_source, None, true);
        let ordered = vintages
            .iter()
            .map(|y| {
                exact::dollars(&y.balance)
                    .map(|opening| (y, opening))
                    .ok_or_else(|| LossAmountIssue::new(y.origin, Some(y.year), true))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let available = |special| {
            ordered
                .iter()
                .filter(|(y, _)| y.eligible() && y.special == special)
                .map(|(_, opening)| exact::whole(*opening))
                .sum::<Exact>()
        };
        let capacity = |income| {
            let ordinary =
                phase::cap(available(false), [exact::whole(fraction(positive(income), 9, 10))]);
            let total = &ordinary
                + phase::cap(available(true), [exact::whole(positive(income)) - &ordinary]);
            (ordinary, total)
        };
        let (_, deduction) = capacity(base);
        let deduction = exact::rounded(&deduction)
            .filter(|amount| amount.checked_mul(100).is_some())
            .ok_or_else(issue)?;
        // Deduction and absorption have different income bases. Section
        // 172(b)(2), as adjusted by 56(d)(1)(B)(ii), uses modified AMTI to
        // measure the loss consumed, even when the deduction is smaller.
        let (ordinary, to_use) = capacity(modified_base);
        let mut to_use = exact::rounded(&to_use).ok_or_else(issue)?;
        let ordinary = exact::rounded(&ordinary).ok_or_else(issue)?;
        // Section 56(d)(1)(B)(ii) preserves chronological absorption except
        // where an ordinary loss cannot use the last 10% of AMTI. Reserve that
        // portion for an eligible special loss, even when its year is later.
        // IRS Chief Counsel 20144201F, pp. 3-6 (nonprecedential explanation).
        let mut ordinary_remaining = ordinary;
        let mut carryforward = exact::whole(0);
        let mut ledger = Vec::new();
        for (y, opening) in ordered {
            let used = if !y.eligible() {
                0
            } else if y.special {
                phase::cap(opening, [to_use])
            } else {
                phase::cap(phase::cap(opening, [to_use]), [ordinary_remaining])
            };
            if !y.special {
                ordinary_remaining = crate::kernel::carry::native_balance(ordinary_remaining, used);
            }
            to_use = crate::kernel::carry::native_balance(to_use, used);
            let settlement = crate::kernel::carry::settle_values::<crate::kernel::carry::Dollars>(
                opening,
                used,
                y.expires(),
            )
            .expect("validated loss opening and absorption");
            let used = settlement.used.get();
            let expired = settlement.expired.get();
            let closing = settlement.closing.get();
            carryforward += exact::whole(closing);
            ledger.push(crate::nol::LossUse {
                year: y.year,
                opening,
                used,
                expired,
                closing,
                expires_now: y.expires_now(),
            });
        }
        line(
            r,
            "prior_atnol_deduction",
            deduction,
            &[
                if has_estate {
                    "income.estate_trust_portfolio.final_nol"
                } else {
                    "screening.atnol"
                },
                "alternative_minimum_taxable_income",
            ],
            "https://www.irs.gov/instructions/i6251",
            msg!("EXPLAIN_PRIOR_ATNOL_DEDUCTION"),
        );
        Ok(alloc::boxed::Box::new(AtnolResult {
            deduction,
            carryforward,
            years: ledger,
            estate_source,
            has_estate,
        }))
    }
}

impl AtnolResult {
    pub(crate) fn discharge_balance(&self) -> Exact {
        &self.carryforward
            + self
                .years
                .iter()
                .filter(|y| y.expires_now)
                .map(|y| exact::whole(y.expired))
                .sum::<Exact>()
    }
    pub(crate) fn record(
        &self,
        current: i64,
        discharge: i64,
        r: &mut FederalResult,
    ) -> Result<(), LossAmountIssue> {
        let inputs: &[&str] = if self.estate_source.is_some() {
            &["income.estate_trust_portfolio.final_nol"]
        } else if self.has_estate {
            &["screening.atnol", "income.estate_trust_portfolio.final_nol"]
        } else {
            &["screening.atnol"]
        };
        let crate::nol::RecordedLossYears { years: yearly, carryforward_cents: total } =
            crate::nol::recorded_loss_years(
                &self.years,
                current,
                discharge,
                self.estate_source,
                true,
            )?;
        for (year, [opening, used, expired, reduced, closing]) in yearly {
            emit_lines(
                r,
                &mut (),
                [
                    ("opening", opening, msg!("EXPLAIN_ATNOL_N_N_OPENING")),
                    ("used", used, msg!("EXPLAIN_ATNOL_N_N_USED")),
                    ("expired", expired, msg!("EXPLAIN_ATNOL_N_N_EXPIRED")),
                    ("debt_reduction", reduced, msg!("EXPLAIN_ATNOL_N_N_DEBT_REDUCTION")),
                    ("carryforward", closing, msg!("EXPLAIN_ATNOL_N_N_CARRYFORWARD")),
                ],
                |(suffix, amount, explanation), _, _| {
                    Some(calculation_row(
                        None,
                        &self.estate_source.map_or_else(
                            || alloc::format!("atnol_{}_{suffix}", year),
                            |id| alloc::format!("estate_trust_{id}_atnol_{year}_{suffix}"),
                        ),
                        LineValue::MoneyCents(amount),
                        inputs,
                        "https://www.irs.gov/instructions/i6251",
                        explanation,
                    ))
                },
            );
        }

        line_cents(
            r,
            "amt_total_nol_carryforward",
            total,
            inputs,
            "https://www.irs.gov/instructions/i6251",
            msg!("EXPLAIN_AMT_TOTAL_NOL_CARRYFORWARD"),
        );
        Ok(())
    }
}

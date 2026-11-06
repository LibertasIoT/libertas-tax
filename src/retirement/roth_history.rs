use crate::common::Owner;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Roth IRA history for one filer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RothHistory {
    /// Roth IRA owner
    pub owner: Owner,
    /// First tax year for which any Roth IRA contribution or conversion was made
    /// Use the tax year designated for a regular contribution, even if paid the next year. Time in a designated Roth employer account does not start the Roth IRA clock.
    // Tax structure: Roth IRA history starts with 1998 and cannot begin after this tax year.
    #[libertas_number(min = 1998, max = 2026)]
    pub first_year: u32,
    /// Opening contribution and conversion records
    /// Needed for a nonqualified withdrawal: absent means none are entered, so the App assumes no earlier contributions or conversions and taxes that withdrawal as earnings, with a note. Leave absent when every withdrawal is qualified by both the five-tax-year period and age or disability. Current-year regular contributions and traditional-to-Roth conversions are taken from their existing entries, not repeated here.
    pub ordering: Option<RothOrdering>,
}
/// Contributions and conversions not yet taken out
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RothOrdering {
    /// Regular contributions through 2025 not yet taken out
    /// Your Roth IRA contributions for 1998 through 2025 as they ended up after any recharacterization, including your own Roth SIMPLE or Roth SEP salary deferrals, less what earlier withdrawals took out (your last Form 8606 line 22 worksheet). Add the basis of a Roth 401(k), 403(b) or 457(b) account rolled in, a military death gratuity or SGLI payment rolled in, basis received in a divorce and line 5 of your last Line 25c Worksheet; subtract basis transferred in a divorce. Leave out conversions, contributions returned to you with their earnings, and 2026 contributions, which come from IRA contributions.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub regular: i64,
    /// Conversions not yet taken out, by year
    /// One row for each year you converted a traditional IRA, rolled a workplace plan into a Roth IRA or had an employer Roth SEP or SIMPLE contribution reported as converted, less what earlier withdrawals took out. Include conversion basis received in a divorce under its original year; leave out amounts transferred in a divorce, rollovers between Roth accounts and 2026 conversions.
    // Tax structure: one conversion row per distinct year, 1998 through 2025.
    #[libertas_size(max = 28)]
    /// ----
    /// Conversion year
    pub conversions: Vec<RothConversion>,
}
/// Conversion year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RothConversion {
    /// Year converted
    // Tax structure: opening Roth conversion basis covers 1998 through the prior tax year, 2025.
    #[libertas_number(min = 1998, max = 2025)]
    pub year: u32,
    /// Part that was taxable when converted
    /// That year's Form 8606 line 18, less what earlier withdrawals took out.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub taxable: i64,
    /// Part that was not taxable
    /// That year's Form 8606 line 17, less what earlier withdrawals took out.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub nontaxable: i64,
}
impl RothHistory {
    pub(crate) fn valid(&self) -> bool {
        (1998..=2026).contains(&self.first_year)
            && self.ordering.as_ref().is_none_or(|v| {
                (0..=100000000000).contains(&v.regular)
                    && v.conversions.iter().enumerate().all(|(i, c)| {
                        (self.first_year..=2025).contains(&c.year)
                            && [c.taxable, c.nontaxable]
                                .iter()
                                .all(|v| (0..=100000000000).contains(v))
                            && !v.conversions[..i].iter().any(|other| other.year == c.year)
                    })
            })
    }
}

/// Roth IRA history
/// Needed for a 1099-R with code J or T: when your first Roth IRA year began and the contributions and conversions not yet taken out. One for each owner.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RothIraHistory {
    /// First year of any Roth IRA contribution or conversion
    /// The tax year your first contribution was made for, even if paid the next spring, or the year of your first conversion. Years in a Roth 401(k) or 403(b) do not count.
    // Tax structure: Roth IRA history starts with 1998 and cannot begin after this tax year.
    #[libertas_number(min = 1998, max = 2026)]
    pub first_year: u32,
    /// Contributions and conversions not yet taken out
    /// Needed when a 2026 withdrawal is not qualified. Leave it off when you were 59½ or older or disabled and your first year is 2021 or earlier.
    /// [NullText]
    /// No nonqualified withdrawal needs these ordering facts
    /// [SetText]
    /// The contributions and conversions not yet taken out are
    pub amounts: Option<RothOrdering>,
}
impl RothIraHistory {
    pub(crate) fn record(&self, owner: Owner) -> RothHistory {
        RothHistory { owner, first_year: self.first_year, ordering: self.amounts.clone() }
    }
    pub(crate) fn from_record(v: &RothHistory) -> Self {
        Self { first_year: v.first_year, amounts: v.ordering.clone() }
    }
    // The client rules, checked again on save.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        let conversions = self.amounts.as_ref().map_or(&[][..], |v| &v.conversions);
        if let Some((index, _)) = conversions
            .iter()
            .enumerate()
            .find(|(i, c)| conversions[..*i].iter().any(|o| o.year == c.year))
        {
            Some(
                crate::validation::ValidationIssue::new("ROTH_CONVERSION_YEAR_TWICE")
                    .at_field("year")
                    .in_list("conversions", index)
                    .in_field("amounts"),
            )
        } else if let Some((index, _)) =
            conversions.iter().enumerate().find(|(_, c)| c.year < self.first_year)
        {
            Some(
                crate::validation::ValidationIssue::new("ROTH_CONVERSION_BEFORE_FIRST_YEAR")
                    .at_field("year")
                    .in_list("conversions", index)
                    .in_field("amounts"),
            )
        } else {
            None
        }
    }
}

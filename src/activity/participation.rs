// Derived classification, never an interview answer. Unavailable retains invalid
// or missing source facts for each consumer's existing named boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Participation {
    Yes,
    No,
    Unavailable,
}
use libertas_macros::*;

/// Participation in this ordinary business activity
/// Include the owner's and spouse's qualifying work, even on separate returns or if the spouse owns no interest. Exclude investor-only monitoring and work done mainly to evade passive-loss restrictions. Use time records or another reasonable record of services; this is not the rental real-estate professional test.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpParticipation {
    /// Ordinary individual participation tests apply to this activity
    /// Yes unless you are a retired or disabled farmer or a surviving spouse using that rule, or an activity-grouping election on your prior returns is unresolved. No leaves this activity out of the estimate with an Estimate note because it needs separate participation analysis.
    pub ordinary_scope: bool,
    /// Combined qualifying work, in minutes
    #[libertas_number(min = 0)]
    pub minutes: u32,
    /// This work was substantially all participation by everyone
    /// Include employees and people who do not own the business. Compare your time records with the business's payroll and contractor hours.
    pub substantially_all: bool,
    /// Combined participation was at least as much as any other individual
    /// Include employees and nonowners when comparing participation. Compare your time records with the hours of the person who worked most, from payroll or contractor records.
    pub at_least_as_much: bool,
    /// Material-participation years among 2016 through 2025
    /// Count them from your 2016 through 2025 returns: the years Schedule E Part II line 28 reported this activity as nonpassive (column (i) or (k)) because you materially participated. Count only years established by a test other than the five-of-ten historical test itself; the years need not be consecutive. Enter zero when there were none.
    // Tax structure: the 2016–2025 lookback contains ten tax years.
    #[libertas_number(min = 0, max = 10)]
    pub prior_ten: u32,
    /// This is a personal-service activity
    /// Health, law, engineering, architecture, accounting, actuarial science, performing arts, consulting, or another business where capital is not a material income-producing factor.
    pub personal_service: bool,
    /// Earlier years of material participation in this same personal-service activity
    /// Count them from your earlier returns: every year before 2026, not only the past ten, that Schedule E Part II reported this activity as nonpassive (line 28 column (i) or (k)) because you materially participated. Enter zero when there were none or when it is not a personal-service activity.
    #[libertas_number(min = 0)]
    pub service_years: u32,
    /// Participation was regular, continuous and substantial
    /// Answer from all facts and circumstances, not ownership percentage: work on most operating days through the year, in the business's day-to-day decisions. This test additionally requires more than 100 qualifying hours.
    pub regular: bool,
    /// Management minutes included in the combined total
    #[libertas_number(min = 0)]
    pub management_minutes: u32,
    /// Another person was paid to manage the activity
    /// Check the payroll and contractor records.
    pub paid_manager: bool,
    /// Another individual spent more time managing it
    /// Consider unpaid managers too; compare the management hours each person recorded.
    pub other_manager_more: bool,
    /// More than 100 qualifying hours in any other trade or business
    /// Count your time records for each other business. Yes may require the separate significant-participation aggregation test when this activity does not qualify independently. Do not combine hours with rentals or investor-only work.
    pub other_business_over_100: bool,
}

impl ScorpParticipation {
    pub(crate) fn left_out(&self) -> bool {
        !self.ordinary_scope
            || (self.valid()
                && self.material() == Participation::No
                && self.minutes > 6_000
                && self.other_business_over_100)
    }
    pub(crate) fn valid(&self) -> bool {
        self.minutes <= 1_051_200
            && self.management_minutes <= self.minutes
            && self.prior_ten <= 10
            && self.service_years <= 100
            && (self.minutes != 0 || !self.substantially_all)
            && (self.personal_service || self.service_years == 0)
    }

    // Unsupported scope is explicit in left_out(); it never masquerades as
    // an unanswered participation choice. Invalid retained data stays unknown.
    pub(crate) fn material(&self) -> Participation {
        if !self.valid() {
            return Participation::Unavailable;
        }
        if !self.ordinary_scope {
            return Participation::No;
        }
        if self.minutes > 30_000
            || self.substantially_all
            || (self.minutes > 6_000 && self.at_least_as_much)
            || self.prior_ten >= 5
            || (self.personal_service && self.service_years >= 3)
        {
            return Participation::Yes;
        }
        let eligible = if self.paid_manager || self.other_manager_more {
            self.minutes - self.management_minutes
        } else {
            self.minutes
        };
        if eligible > 6_000 && self.regular {
            return Participation::Yes;
        }
        Participation::No
    }
}

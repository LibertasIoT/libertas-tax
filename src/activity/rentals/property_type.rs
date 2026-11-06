use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Schedule E line 1b type of property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RentalPropertyType {
    /// Single-family residence
    SingleFamily,
    /// Multi-family residence
    /// Residential when 80% or more of the building's gross rents are from dwelling units (section 168(e)(2)(A)); otherwise choose Commercial. Units in an establishment where more than half are used on a transient basis are not dwelling units.
    MultiFamily,
    /// Vacation or short-term rental
    /// A dwelling unit rented for short stays. Personal use is entered below; the stays decide the passive-activity treatment.
    VacationShortTerm {
        /// Average stay and services
        stays: ShortTermStays,
    },
    /// Commercial
    Commercial,
    /// Land
    /// Enter the land cost as the building's cost with the same amount as its land part; nothing is depreciated. Net income from a rental whose depreciable part is under 30% of its unadjusted basis is not passive and cannot absorb other passive losses; a loss stays passive.
    Land,
    /// Self-rental
    /// Property you rent to a trade or business in which you materially participated, such as your own Schedule C business. Net income is not passive and not net investment income; a loss stays passive.
    SelfRental,
    /// Other
    Other {
        /// Describe the property
        #[libertas_size(min = 1)]
        description: String,
    },
}

/// Average stay and services of a short-term rental
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ShortTermStays {
    /// Average stay of more than 7 days, or more than 30 days when you provided significant personal services
    /// Divide the days of all rental periods in 2026 by the number of rentals. A rental activity: your participation answer under More about this property applies.
    OverSevenDays,
    /// Average stay of 7 days or less
    /// Not a rental activity for the passive-activity rules but a business activity: no special rental-loss allowance, and the participation answer under More about this property does not apply.
    SevenDaysOrLess {
        /// You materially participated
        /// More than 500 hours; or substantially all the work; or more than 100 hours and no one else more; or another section 469 test (Pub. 925). Count your spouse's work. Yes makes the income or loss nonpassive; No makes it a passive business activity whose loss offsets only passive income.
        material_participation: bool,
    },
    /// Average stay of 30 days or less with significant personal services
    /// Services by people, such as daily cleaning or concierge service, that are significant next to the rent but are not hotel-like services for the guests' convenience. A business activity, like the choice above.
    ThirtyDaysOrLessWithServices {
        /// You materially participated
        /// More than 500 hours; or substantially all the work; or more than 100 hours and no one else more; or another section 469 test (Pub. 925). Count your spouse's work. Yes makes the income or loss nonpassive; No makes it a passive business activity whose loss offsets only passive income.
        material_participation: bool,
    },
    /// Substantial services primarily for the guests' convenience
    /// Regular cleaning during stays, linen changes, meals, maid or concierge service, as a hotel does. The income is business income on Schedule C with self-employment tax: enter the property as a business on the Self-employment topic, with the building as a 39-year asset, and remove it here.
    SubstantialServices,
}

impl RentalPropertyType {
    pub(crate) fn nonresidential(&self) -> bool {
        matches!(self, Self::Commercial | Self::SelfRental | Self::Other { .. })
            || matches!(self, Self::VacationShortTerm { stays } if !matches!(stays, ShortTermStays::OverSevenDays))
    }
    pub(crate) fn business_participation(&self) -> Option<bool> {
        match self {
            Self::VacationShortTerm {
                stays:
                    ShortTermStays::SevenDaysOrLess { material_participation }
                    | ShortTermStays::ThirtyDaysOrLessWithServices { material_participation },
            } => Some(*material_participation),
            _ => None,
        }
    }
    pub(crate) fn substantial_services(&self) -> bool {
        matches!(self, Self::VacationShortTerm { stays: ShortTermStays::SubstantialServices })
    }
}

use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Earned income share of support
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum SupportShare {
    /// Less than half
    LessThanHalf,
    /// Exactly half
    ExactlyHalf,
    /// More than half
    MoreThanHalf,
}

/// Real estate professional
/// More than half of your working hours, and more than 750 hours, were in real property trades or businesses in which you materially participated (Pub 925).
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ProfessionalRentals {
    /// You elected to treat all your rental real estate as one activity
    /// The statement filed with an earlier return or with this one (section 469(c)(7)(A)). Material participation is then tested on all your rentals together, so choose a real estate professional/material-participation answer on every rental and K-1 rental activity or on none; selling one property releases no suspended loss until the last one is sold.
    #[libertas_default(false)]
    pub single_activity: bool,
    /// You worked more than 500 hours in each rental in which you chose Professional this year, or in any 5 of the last 10 years
    /// With the election, count all your rentals as one. When this is true, that rental income or loss is business income and not subject to the net investment income tax. Leave it off when your hours are not documented; the rentals then stay investment income.
    #[libertas_default(false)]
    pub over_500_hours: bool,
}

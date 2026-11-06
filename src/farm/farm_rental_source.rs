// Crop-share rental property use and tenant-connection source facts.

/// Actual use records for one sold crop-share rental property
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    libertas_macros::LibertasAvroEncode,
    libertas_macros::LibertasAvroDecode,
    libertas_macros::LibertasExport,
)]
#[libertas_inline_choices]
pub enum FarmRentalPropertyUse {
    /// Used only in this crop-share rental from acquisition through sale
    /// The entire sold interest was rented, or ready and available to rent,
    /// in this same rental throughout your ownership, with no personal,
    /// investment-holding, inventory, or other business or rental use.
    OnlyThisRental {
        /// Your and your spouse's connection to every tenant's business
        /// Include earlier years and businesses held through an entity.
        /// [NullText]
        /// The tenant-business connection is unavailable
        /// [SetText]
        /// The tenant-business records show
        tenant_connection: Option<FarmRentalTenantConnection>,
        /// Activities that enhanced this property's value
        /// Include construction, renovation, development and lease-up,
        /// including activities through an entity, for this property or any
        /// property whose basis carries into this property.
        /// [NullText]
        /// The value-enhancing activity records are unavailable
        /// [SetText]
        /// The activity records show
        value_enhancement: Option<FarmRentalValueEnhancement>,
    },
    /// None of these: the property had another use during your ownership
    NoneOfThese,
}
/// Actual connection to the tenant's business
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    libertas_macros::LibertasAvroEncode,
    libertas_macros::LibertasAvroDecode,
    libertas_macros::LibertasExport,
)]
#[libertas_inline_choices]
pub enum FarmRentalTenantConnection {
    /// Neither spouse has ever owned an interest or performed services in any tenant's business
    NoOwnershipOrServices,
    /// Either spouse has owned an interest or performed services in a tenant's business
    OwnershipOrServices,
}
/// Actual participation in value-enhancing activities
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    libertas_macros::LibertasAvroEncode,
    libertas_macros::LibertasAvroDecode,
    libertas_macros::LibertasExport,
)]
#[libertas_inline_choices]
pub enum FarmRentalValueEnhancement {
    /// Neither spouse ever participated in a value-enhancing activity for this or any basis-related property
    NoParticipation,
    /// Either spouse participated in a value-enhancing activity for this or any basis-related property
    Participation,
}

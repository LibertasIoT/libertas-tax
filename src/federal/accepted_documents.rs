pub use crate::bba_audit_source::BbaAuditSource;
pub use crate::vehicle_loan_source::VehicleLoanSource;

impl VehicleLoanSource {
    /// The loan facts the calculations read, derived from the printed form.
    pub(crate) fn model(&self) -> crate::FederalVehicleInterest {
        crate::FederalVehicleInterest::from(&self.document)
    }
}

impl From<crate::FederalBbaAuditInterest> for BbaAuditSource {
    fn from(document: crate::FederalBbaAuditInterest) -> Self {
        let id = document.id;
        let (document, answers) = document.into_parts();
        Self { id, answers, ..Self::from(document) }
    }
}

impl BbaAuditSource {
    pub(crate) fn model(
        &self,
    ) -> Result<crate::FederalBbaAuditInterest, (&'static str, &'static str)> {
        self.answers.model(self.id, &self.document)
    }

    pub(crate) fn refigure(
        &self,
    ) -> Result<crate::bba_audit::BbaAuditAdjustment, (&'static str, &'static str)> {
        self.model()?.refigure()
    }
}

mod builtin;
mod controls;
mod facts;
mod legacy;
mod options;
mod state;
mod surface;

pub use controls::{VendorControlItem, VendorControlProvider, VendorControlRenderModel};
pub use facts::{
    OfficialReference, VendorCapabilityFact, VendorCapabilityStatus, VendorConnectionKind,
    VendorFact, VendorFactRegistry, VendorFactValidationError,
};
pub use legacy::{VendorUiCapabilities, VendorUiProfile};
pub use options::VendorOption;
pub use state::VendorUiState;
pub use surface::VendorUiSurface;

#[cfg(test)]
mod provider_tests;
#[cfg(test)]
mod tests;

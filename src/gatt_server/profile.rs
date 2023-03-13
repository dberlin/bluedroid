use super::LockedService;
use esp_idf_sys::{esp_ble_gatts_app_register, esp_gatt_id_t, esp_nofail};
use log::debug;
use parking_lot::RwLock;
use std::sync::Arc;

/// Shorthand for our locked profiles that are returned everywhere
pub type LockedProfile = Arc<RwLock<Profile>>;
/// Represents a GATT profile.
///
/// # Notes
///
/// Grouping services into a profile won't changed the actual exposed interface.
/// In this context, a profile is also called "application" in the ESP-IDF documentation.
///
/// Internally, grouping services into different profiles only defines different event handlers.
#[derive(Debug, Clone)]
pub struct Profile {
    name: Option<String>,
    pub(crate) services: Vec<LockedService>,
    pub(crate) identifier: u16,
    pub(crate) interface: Option<u8>,
}

impl Profile {
    /// Creates a new [`Profile`].
    #[must_use]
    pub const fn new(identifier: u16) -> Self {
        Self {
            name: None,
            services: Vec::new(),
            identifier,
            interface: None,
        }
    }

    /// Sets the name of the [`Profile`].
    ///
    /// This name is only used for debugging purposes.
    pub fn name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    /// Adds a [`Service`] to the [`Profile`].
    #[must_use]
    pub fn service(&mut self, service: &LockedService) -> &mut Self {
        self.services.push(service.clone());
        self
    }

    /// Returns a reference to the built [`Profile`] behind an `Arc` and an `RwLock`.
    ///
    /// The returned value can be passed to any function of this crate that expects a [`Profile`].
    /// It can be used in different threads, because it is protected by an `RwLock`.
    #[must_use]
    pub fn build(&self) -> LockedProfile {
        Arc::new(RwLock::new(self.clone()))
    }

    pub(crate) fn get_service(&self, handle: u16) -> Option<LockedService> {
        for service in &self.services {
            if service.read().handle == Some(handle) {
                return Some(service.clone());
            }
        }

        None
    }

    pub(crate) fn get_service_by_id(&self, id: esp_gatt_id_t) -> Option<LockedService> {
        for service in &self.services {
            if service.read().uuid == id.into() {
                return Some(service.clone());
            }
        }

        None
    }

    pub(crate) fn register_self(&self) {
        debug!("Registering {}.", self);
        unsafe { esp_nofail!(esp_ble_gatts_app_register(self.identifier)) };
    }

    pub(crate) fn register_services(&mut self) {
        debug!("Registering {}'s services.", &self);
        self.services.iter_mut().for_each(|service| {
            service.write().register_self(self.interface.unwrap());
        });
    }
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (0x{:04x})",
            self.name
                .clone()
                .unwrap_or_else(|| "Unnamed profile".to_string()),
            self.identifier,
        )
    }
}

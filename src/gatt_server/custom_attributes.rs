use std::sync::{Arc, Mutex};

use crate::{
    gatt_server::Descriptor,
    utilities::{AttributePermissions, BleUuid},
};

use std::sync::{Arc, Mutex};

use esp_idf_svc::nvs::{EspDefaultNvs, EspDefaultNvsPartition};
use log::debug;
use parking_lot::Mutex;

pub struct SettableStorage {
    storage: Mutex<Option<Arc<Mutex<EspDefaultNvs>>>>,
}

impl SettableStorage {
    pub fn set_storage_partition(&self, storage: EspDefaultNvsPartition) {
        *self.storage.lock() = Some(Arc::new(Mutex::new(
            EspDefaultNvs::new(storage, "ble", true)
                .expect("Cannot create a new NVS storage. Did you declare an NVS partition?"),
        )))
    }
    pub const fn new() -> Self {
        Self {
            storage: Mutex::new(None),
        }
    }
    pub fn get(&self) -> Arc<Mutex<EspDefaultNvs>> {
        if self.storage.lock().is_some() {
            return self.storage.lock().clone().unwrap();
        }
        let nvs = EspDefaultNvs::new(
            EspDefaultNvsPartition::take()
                .expect("Cannot initialise the default NVS. Did you declare an NVS partition?"),
            "ble",
            true,
        )
        .expect("Cannot create a new NVS storage. Did you declare an NVS partition?");
        let res = Arc::new(Mutex::new(nvs));
        *self.storage.lock() = Some(res.clone());
        res
    }
}

/// NVS Storage for our BLE CCCD's
pub static STORAGE: SettableStorage = SettableStorage::new();

impl Descriptor {
    /// Creates a new descriptor with the `0x2901` UUID, and the description string as its value.
    ///
    /// This is a common descriptor used to describe a characteristic to the user.
    /// See [`Characteristic::show_name`] for an easier way to assign this kind of descriptor to a [`Characteristic`].
    ///
    /// [`Characteristic::show_name`]: crate::gatt_server::Characteristic::show_name
    /// [`Characteristic`]: crate::gatt_server::Characteristic
    pub fn user_description<S: AsRef<str>>(description: S) -> Self {
        Self::new(BleUuid::from_uuid16(0x2901))
            .name("User Description")
            .permissions(AttributePermissions::new().read())
            .set_value(description.as_ref().as_bytes().to_vec())
            .clone()
    }

    /// Creates a CCCD.
    ///
    /// The contents of the CCCD are stored in NVS and persisted across reboots.
    ///
    /// # Panics
    ///
    /// Panics if the NVS is not configured.
    #[must_use]
    pub fn cccd() -> Self {
        Self::new(BleUuid::from_uuid16(0x2902))
            .name("Client Characteristic Configuration")
            .permissions(AttributePermissions::new().read().write())
            .on_read(
                |param: esp_idf_sys::esp_ble_gatts_cb_param_t_gatts_read_evt_param| {
                    let storage = STORAGE.get().clone();

                    // Get the descriptor handle.

                    // TODO: Find the characteristic that contains the handle.
                    // WARNING: Using the handle is incredibly stupid as the NVS is not erased across flashes.

                    // Create a key from the connection address.
                    let key = format!(
                        "{:02X}{:02X}{:02X}{:02X}-{:04X}",
                        /* param.bda[1], */ param.bda[2],
                        param.bda[3],
                        param.bda[4],
                        param.bda[5],
                        param.handle
                    );

                    // Prepare buffer and read correct CCCD value from non-volatile storage.
                    let mut buf: [u8; 2] = [0; 2];
                    let val = storage.lock().get_raw(&key, &mut buf);
                    if let Some(value) = val.unwrap() {
                        debug!("Read CCCD value: {:?} for key {}.", value, key);
                        value.to_vec()
                    } else {
                        debug!("No CCCD value found for key {}.", key);
                        vec![0, 0]
                    }
                },
            )
            .on_write(|value, param| {
                let storage = STORAGE.get();

                // Create a key from the connection address.
                let key = format!(
                    "{:02X}{:02X}{:02X}{:02X}-{:04X}",
                    /* param.bda[1], */ param.bda[2],
                    param.bda[3],
                    param.bda[4],
                    param.bda[5],
                    param.handle
                );

                debug!("Write CCCD value: {:?} at key {}", value, key);

                // Write CCCD value to non-volatile storage.
                storage
                    .lock()
                    .set_raw(&key, &value)
                    .expect("Cannot put raw value to the NVS. Did you declare an NVS partition?");
            })
            .clone()
    }
}

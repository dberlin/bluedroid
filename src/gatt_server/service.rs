use crate::{
    gatt_server::characteristic::Characteristic, leaky_box_raw, utilities::BleUuid,
};
use esp_idf_sys::*;
use log::info;
use std::{fmt::Formatter, borrow::BorrowMut};

#[derive(Debug, Clone)]
pub struct Service {
    name: Option<String>,
    pub(crate) uuid: BleUuid,
    pub(crate) characteristics: Vec<Characteristic>,
    primary: bool,
    pub(crate) handle: Option<u16>,
}

impl Service {
    pub fn new(name: &str, uuid: BleUuid, primary: bool) -> Service {
        Service {
            name: Some(String::from(name)),
            uuid,
            characteristics: Vec::new(),
            primary,
            handle: None,
        }
    }

    pub fn add_characteristic<C: BorrowMut<Characteristic>>(&mut self, characteristic: C) -> &mut Self {
        self.characteristics.push(characteristic.borrow().to_owned());
        self
    }

    pub(crate) fn register_self(&mut self, interface: u8) {
        info!("Registering {} on interface {}.", &self, interface);

        let id: esp_gatt_srvc_id_t = esp_gatt_srvc_id_t {
            id: self.uuid.into(),
            is_primary: self.primary,
        };

        unsafe {
            esp_nofail!(esp_ble_gatts_create_service(
                interface,
                leaky_box_raw!(id),
                16, // TODO: count the number of characteristics and descriptors.
            ));
        }
    }

    pub(crate) fn register_characteristics(&mut self) {
        info!("Registering {}'s characteristics.", &self);
        self.characteristics
            .iter_mut()
            .for_each(|characteristic: &mut Characteristic| {
                characteristic.register_self(
                    self.handle
                        .expect("Cannot register a characteristic to a service without a handle."),
                );
            });
    }
}

impl std::fmt::Display for Service {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({})",
            self.name
                .clone()
                .unwrap_or_else(|| "Unnamed service".to_string()),
            self.uuid,
        )
    }
}

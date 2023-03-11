use crate::gatt_server::Profile;
use crate::utilities::BleUuid;
use esp_idf_sys::{
    esp_ble_gatts_cb_param_t_gatts_add_char_evt_param, esp_gatt_status_t_ESP_GATT_OK,
};
use log::{info, warn};

impl Profile {
    pub(crate) fn on_char_add(&mut self, param: esp_ble_gatts_cb_param_t_gatts_add_char_evt_param) {
        let Some(service) = self.get_service(param.service_handle) else {
            warn!("Cannot find service described by handle 0x{:04x} received in characteristic creation event.", param.service_handle);
            return;
        };

        let Some(characteristic) = service.read().get_characteristic_by_id(param.char_uuid) else {
            warn!("Cannot find characteristic described by service handle 0x{:04x} and characteristic identifier {} received in characteristic creation event.", param.service_handle, BleUuid::from(param.char_uuid));
            return;
        };

        if param.status == esp_gatt_status_t_ESP_GATT_OK {
            info!(
                "GATT characteristic {} registered at attribute handle 0x{:04x}.",
                characteristic.read(),
                param.attr_handle
            );
            characteristic.write().attribute_handle = Some(param.attr_handle);
            characteristic.write().register_descriptors();
        } else {
            warn!("GATT characteristic registration failed.");
        }
    }
}

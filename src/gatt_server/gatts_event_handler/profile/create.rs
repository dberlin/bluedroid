use crate::gatt_server::Profile;
use crate::utilities::BleUuid;
use esp_idf_sys::{
    esp_ble_gatts_cb_param_t_gatts_create_evt_param, esp_ble_gatts_start_service,
    esp_gatt_status_t_ESP_GATT_OK, esp_nofail,
};
use log::{info, warn};

impl Profile {
    pub(crate) fn on_create(&mut self, param: esp_ble_gatts_cb_param_t_gatts_create_evt_param) {
        let Some(service) = self.get_service_by_id(param.service_id.id) else {
            warn!("Cannot find service with service identifier {} received in service creation event.", BleUuid::from(param.service_id.id));
            return;
        };

        service.write().handle = Some(param.service_handle);

        if param.status == esp_gatt_status_t_ESP_GATT_OK {
            info!(
                "GATT service {} registered on handle 0x{:04x}.",
                service.read(),
                service.read().handle.unwrap()
            );

            unsafe {
                esp_nofail!(esp_ble_gatts_start_service(service.read().handle.unwrap()));
            }

            service.write().register_characteristics();
        } else {
            warn!("GATT service registration failed.");
        }
    }
}

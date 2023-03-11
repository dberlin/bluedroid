use crate::gatt_server::Profile;
use crate::utilities::BleUuid;
use esp_idf_sys::{
    esp_ble_gatts_cb_param_t_gatts_add_char_descr_evt_param, esp_gatt_status_t_ESP_GATT_OK,
};
use log::{info, warn};

impl Profile {
    pub(crate) fn on_char_add_descr(
        &mut self,
        param: esp_ble_gatts_cb_param_t_gatts_add_char_descr_evt_param,
    ) {
        // ATTENTION: Descriptors might have duplicate UUIDs!
        // We need to set them in order of creation.

        let Some(service) = self.get_service(param.service_handle)  else {
            warn!("Cannot find service described by handle 0x{:04x} received in descriptor creation event.", param.service_handle);
            return;
        };

        let descriptors = service.read().get_descriptors_by_id(param.descr_uuid);

        let Some(descriptor) = descriptors.iter().find(|d| d.read().attribute_handle.is_none()) else {
            warn!("Cannot find service described by identifier {} received in descriptor creation event.", BleUuid::from(param.descr_uuid));
            return;
        };

        if param.status == esp_gatt_status_t_ESP_GATT_OK {
            info!(
                "GATT descriptor {:?} registered at attribute handle 0x{:04x}.",
                descriptor.read(),
                param.attr_handle
            );
            descriptor.write().attribute_handle = Some(param.attr_handle);
        } else {
            warn!("GATT descriptor registration failed.");
        }
    }
}

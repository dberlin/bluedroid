use crate::gatt_server::GattServer;
use crate::utilities::BleUuid;
use esp_idf_sys::*;
use log::{debug, warn};

impl GattServer {
    #[allow(clippy::too_many_lines)]
    pub(crate) fn on_set_attr_val(
        &self,
        gatts_if: esp_gatt_if_t,
        param: esp_ble_gatts_cb_param_t_gatts_set_attr_val_evt_param,
    ) {
        if param.status != esp_gatt_status_t_ESP_GATT_OK {
            warn!(
                "Failed to set attribute value, error code: {:04x}.",
                param.status
            );
        }

        let Some(profile) = self.get_profile(gatts_if) else {
            warn!("Cannot find profile described by interface {} received in set attribute value event.", gatts_if);
            return;
        };

        let Some(service) = profile.read().get_service(param.srvc_handle) else {
            warn!("Cannot find service described by service handle {} received in set attribute value event.", param.srvc_handle);
            return;
        };

        let Some(characteristic) = service.read().get_characteristic_by_handle(param.attr_handle) else {
            warn!("Cannot find characteristic described by service handle {} and attribute handle {} received in set attribute value event.", param.srvc_handle, param.attr_handle);
            return;
        };

        debug!(
            "Received set attribute value event for characteristic {}.",
            characteristic.read()
        );

        for connection in self.active_connections.clone() {
            // Get the current status of the CCCD via a fake read operation.
            let simulated_read_param = esp_ble_gatts_cb_param_t_gatts_read_evt_param {
                bda: connection.remote_bda,
                conn_id: connection.id,
                handle: characteristic
                    .read()
                    .descriptors
                    .iter()
                    .find(|desc| desc.read().uuid == BleUuid::Uuid16(0x2902))
                    .unwrap()
                    .read()
                    .attribute_handle
                    .unwrap(),
                ..Default::default()
            };

            let status = characteristic.read().get_cccd_status(simulated_read_param);

            // Check that the status is not None, otherwise bail.
            let Some((notification, indication)) = status else { return; };
            let properties = characteristic.read().properties;

            let mut internal_value = characteristic.write().internal_value.clone();

            if properties.indicate && indication {
                debug!(
                    "Indicating {} value change to {:02X?}.",
                    characteristic.read(),
                    connection.id
                );
                let result = unsafe {
                    esp!(esp_ble_gatts_send_indicate(
                        gatts_if,
                        connection.id,
                        param.attr_handle,
                        internal_value.len() as u16,
                        internal_value.as_mut_slice().as_mut_ptr(),
                        true
                    ))
                };

                if result.is_err() {
                    warn!(
                        "Failed to indicate value change: {}.",
                        result.err().unwrap()
                    );
                }
            } else if properties.notify && notification {
                debug!(
                    "Notifying {} value change to {}.",
                    characteristic.read(),
                    connection
                );
                let result = unsafe {
                    esp!(esp_ble_gatts_send_indicate(
                        gatts_if,
                        connection.id,
                        param.attr_handle,
                        internal_value.len() as u16,
                        internal_value.as_mut_slice().as_mut_ptr(),
                        false
                    ))
                };

                if result.is_err() {
                    warn!("Failed to notify value change: {}.", result.err().unwrap());
                }
            }
        }

        let value: *mut *const u8 = &mut [0u8].as_ptr();
        let mut len = 512;
        let vector = unsafe {
            esp_nofail!(esp_ble_gatts_get_attr_value(
                param.attr_handle,
                &mut len,
                value,
            ));

            std::slice::from_raw_parts(*value, len as usize)
        };

        debug!(
            "Characteristic {} value changed to {:02X?}.",
            characteristic.read(),
            vector
        );
    }
}

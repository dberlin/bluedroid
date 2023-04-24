use crate::gatt_server::Profile;
use crate::utilities::AttributeControl;
use esp_idf_sys::{
    esp_ble_gatts_cb_param_t_gatts_read_evt_param, esp_ble_gatts_send_response, esp_gatt_if_t,
    esp_gatt_rsp_t, esp_gatt_status_t_ESP_GATT_OK, esp_gatt_value_t, esp_nofail,
};
use log::debug;

impl Profile {
    pub(crate) fn on_read(
        &mut self,
        gatts_if: esp_gatt_if_t,
        param: esp_ble_gatts_cb_param_t_gatts_read_evt_param,
    ) {
        for service in &self.services {
            service
                .read()
                .characteristics
                .iter()
                .for_each(|characteristic| {
                    if characteristic.read().attribute_handle == Some(param.handle) {
                        debug!(
                            "Received read event for characteristic {}.",
                            characteristic.read()
                        );

                        // If the characteristic has a read handler, call it.
                        if let AttributeControl::ResponseByApp(callback) =
                            &characteristic.read().control
                        {
                            let value = callback(param);

                            // Extend the response to the maximum length.
                            let mut response = [0u8; 600];
                            response[..value.len()].copy_from_slice(&value);

                            let mut esp_rsp = esp_gatt_rsp_t {
                                attr_value: esp_gatt_value_t {
                                    auth_req: 0,
                                    handle: param.handle,
                                    len: value.len() as u16,
                                    offset: 0,
                                    value: response,
                                },
                            };

                            unsafe {
                                esp_nofail!(esp_ble_gatts_send_response(
                                    gatts_if,
                                    param.conn_id,
                                    param.trans_id,
                                    // TODO: Allow different statuses.
                                    esp_gatt_status_t_ESP_GATT_OK,
                                    &mut esp_rsp
                                ));
                            }
                        }
                    } else {
                        characteristic
                            .read()
                            .descriptors
                            .iter()
                            .for_each(|descriptor| {
                                debug!(
                                    "MCC: Checking descriptor {} ({:?}).",
                                    descriptor.read(),
                                    descriptor.read().attribute_handle
                                );

                                if descriptor.read().attribute_handle == Some(param.handle) {
                                    debug!(
                                        "Received read event for descriptor {}.",
                                        descriptor.read()
                                    );

                                    if let AttributeControl::ResponseByApp(callback) =
                                        &descriptor.read().control
                                    {
                                        let value = callback(param);

                                        // Extend the response to the maximum length.
                                        let mut response = [0u8; 600];
                                        response[..value.len()].copy_from_slice(&value);

                                        let mut esp_rsp = esp_gatt_rsp_t {
                                            attr_value: esp_gatt_value_t {
                                                auth_req: 0,
                                                handle: param.handle,
                                                len: value.len() as u16,
                                                offset: 0,
                                                value: response,
                                            },
                                        };

                                        unsafe {
                                            esp_nofail!(esp_ble_gatts_send_response(
                                                gatts_if,
                                                param.conn_id,
                                                param.trans_id,
                                                esp_gatt_status_t_ESP_GATT_OK,
                                                &mut esp_rsp
                                            ));
                                        }
                                    }
                                }
                            });
                    }
                });
        }
    }
}

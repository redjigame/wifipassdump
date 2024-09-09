mod utils;
mod xml_parser;
mod wlan;

use std::{os::windows::ffi::OsStringExt};
use windows::Win32::NetworkManagement::WiFi::WlanCloseHandle;
use windows::Win32::NetworkManagement::WiFi::{WlanFreeMemory, WLAN_API_VERSION_2_0};

fn main() {
    let wlan_handle = wlan::open_wlan_handle(WLAN_API_VERSION_2_0).expect("failed to open WLAN handle!");

    let interface_ptr = match wlan::enum_wlan_interfaces(wlan_handle) {
        Ok(interfaces) => interfaces,
        Err(e) => {
            eprintln!("Failed to get the wireless interfaces: {:?}", e);
            unsafe { WlanCloseHandle(wlan_handle, None) };
            std::process::exit(1);
        }
    };

    let interfaces_list = unsafe {
        std::slice::from_raw_parts(
            (*interface_ptr).InterfaceInfo.as_ptr(),
            (*interface_ptr).dwNumberOfItems as usize,
        )
    };

    for interface_info in interfaces_list {
        let interface_description = match utils::parse_utf16_slice(interface_info.strInterfaceDescription.as_slice()) {
            Some(name) => name,
            None => {
                eprintln!("Could not parse our interface description.");
                continue;
            }
        };

        let wlan_profile_ptr = match wlan::grab_interface_profiles(wlan_handle, &interface_info.InterfaceGuid) {
            Ok(profiles) => profiles,
            Err(_e) => {
                eprintln!("Failed to retrieve profiles.");
                continue;
            }
        };

        let wlan_profile_list = unsafe {
            std::slice::from_raw_parts(
                (*wlan_profile_ptr).ProfileInfo.as_ptr(),
                (*wlan_profile_ptr).dwNumberOfItems as usize,
            )
        };

        for profile in wlan_profile_list {
            let profile_name = match utils::parse_utf16_slice(&profile.strProfileName) {
                Some(name) => name,
                None => {
                    eprintln!("Could not parse profile name");
                    continue;
                }
            };

            let profile_xml_data = match wlan::get_profile_xml(wlan_handle, &interface_info.InterfaceGuid, &profile_name) {
                Ok(data) => data,
                Err(_e) => {
                    eprintln!("Failed to extract XML data");
                    continue;
                }
            };

            let xml_document = match xml_parser::load_xml_data(&profile_xml_data) {
                Ok(xml) => xml,
                Err(_e) => {
                    eprintln!("Failed to extract XML document");
                    continue;
                }
            };

            let root = match xml_document.DocumentElement() {
                Ok(root) => root,
                Err(_e) => {
                    eprintln!("Failed to get document root for profile XML");
                    continue;
                }
            };

            let auth_type = match xml_parser::traverse_xml_tree(&root, &["MSM", "security", "authEncryption", "authentication"]) {
                Some(t) => t,
                None => {
                    eprintln!("Failed to get auth type for this profile");
                    continue;
                }
            };

            match auth_type.as_str() {
                "open" => {
                    println!("Wi-fi Name: {}, No password", profile_name.to_string_lossy().to_string());
                }
                "WPA2" | "WPA2PSK" => {
                    if let Some(password) = xml_parser::traverse_xml_tree(&root, &["MSM", "security", "sharedKey", "keyMaterial"]) {
                        println!("Wi-fi Name: {}, Authentication: {}, Password: {}", profile_name.to_string_lossy().to_string(), auth_type, password);
                    }
                }
                _ => {
                    println!("Wi-fi Name: {}, Authentication: {}", profile_name.to_string_lossy().to_string(), auth_type)
                }
            }
        }
    }

    unsafe { WlanFreeMemory(interface_ptr.cast()) };
    unsafe { WlanCloseHandle(wlan_handle, None) };
}

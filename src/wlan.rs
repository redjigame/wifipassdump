use windows::{
    core::{GUID, PCWSTR, PWSTR, HSTRING},
    Win32::{
        Foundation::{HANDLE, INVALID_HANDLE_VALUE, WIN32_ERROR},
        NetworkManagement::WiFi::{
            WlanEnumInterfaces, WlanFreeMemory, WlanGetProfile, WlanGetProfileList,
            WlanOpenHandle, WLAN_PROFILE_GET_PLAINTEXT_KEY, WLAN_INTERFACE_INFO_LIST, WLAN_PROFILE_INFO_LIST
        },
    },
};
use std::ffi::OsString;

pub fn open_wlan_handle(api_version: u32) -> Result<HANDLE, windows::core::Error> {
    let mut negotiated_version = 0;
    let mut wlan_handle = INVALID_HANDLE_VALUE;

    let result = unsafe { WlanOpenHandle(api_version, None, &mut negotiated_version, &mut wlan_handle) };
    WIN32_ERROR(result).ok()?;

    Ok(wlan_handle)
}

pub fn enum_wlan_interfaces(handle: HANDLE) -> Result<*mut WLAN_INTERFACE_INFO_LIST, windows::core::Error> {
    let mut interface_ptr = std::ptr::null_mut();
    let result = unsafe { WlanEnumInterfaces(handle, None, &mut interface_ptr) };
    WIN32_ERROR(result).ok()?;

    Ok(interface_ptr)
}

pub fn grab_interface_profiles(
    handle: HANDLE,
    interface_guid: &GUID,
) -> Result<*const WLAN_PROFILE_INFO_LIST, windows::core::Error> {
    let mut wlan_profiles_ptr = std::ptr::null_mut();
    let result = unsafe { WlanGetProfileList(handle, interface_guid, None, &mut wlan_profiles_ptr) };
    WIN32_ERROR(result).ok()?;

    Ok(wlan_profiles_ptr)
}

pub fn get_profile_xml(
    handle: HANDLE,
    interface_guid: &GUID,
    profile_name: &OsString,
) -> Result<OsString, windows::core::Error> {
    let mut profile_xml_data = PWSTR::null();
    let mut profile_get_flags = WLAN_PROFILE_GET_PLAINTEXT_KEY;

    let result = unsafe {
        WlanGetProfile(
            handle,
            interface_guid,
            PCWSTR(HSTRING::from(profile_name).as_ptr()),
            None,
            &mut profile_xml_data,
            Some(&mut profile_get_flags),
            None,
        )
    };

    WIN32_ERROR(result).ok()?;
    let xml_string = match unsafe { profile_xml_data.to_hstring() } {
        Ok(data) => data,
        Err(e) => {
            unsafe { WlanFreeMemory(profile_xml_data.as_ptr().cast()) };
            return Err(e);
        }
    };

    Ok(xml_string.to_os_string())
}

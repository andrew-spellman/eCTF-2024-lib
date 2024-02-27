use core::ffi::{c_char, c_uint, CStr};

/*
// rust_ectf_params.c
#include <stdint.h>
#include "ectf_params.h"

#define UNREACHABLE while(1) {}

// ------------------- BOTH -------------------
struct extern_comp {
    uint32_t id;
    const char* boot_msg;
    const char* attestation_loc;
    const char* attestation_date;
    const char* attestation_customer;
}

strcut extern_ap {
    const char* ap_pin;
    const char* ap_token;
    const char* boot_msg;
    uint32_t* comp_ids;
    uint32_t comp_num;
}

// Which device this was compiled for
// - 0 = Component
// - 1 = Application Processor
int comp_or_ap(void) {
#ifdef COMPONENT_ID
    return 0;
#else
    return 1;
#endif
}

// ------------------- FOR COMPONENT -------------------
#ifdef COMPONENT_ID

// Get component items
extern_comp get_comp(void) {
    extern_comp comp = {
        .id = COMPONENT_ID,
        .boot_msg = COMPONENT_BOOT_MSG,
        .attestation_loc = ATTESTATION_LOC,
        .attestation_date = ATTESTATION_DATE,
        .attestation_customer = ATTESTATION_CUSTOMER
    };

    return comp;
}

// should never be called for component
extern_ap get_ap(void) {
    // should never be called
    UNREACHABLE;
}

// ------------------- FOR APPLICATION PROCESSOR -------------------
#else

// should never be called for application processor
extern_comp get_comp(void) {
    // should never be called
    UNREACHABLE;
}

// Get Application Processor items
extern_ap get_ap(void) {
    uint32_t comp_ids[COMPONENT_CNT] = { COMPONENT_IDS };

    extern_ap ap = {
        .ap_pin = AP_PIN,
        .ap_token = AP_TOKEN,
        .boot_msg = AP_BOOT_MSG,
        .comp_ids = (uint32_t*)comp_ids,
        .comp_num = COMPONENT_CNT,
    }

    return ap;
}
#endif
*/

#[repr(C)]
struct ExternAP {
    ap_pin: *const c_char,
    ap_token: *const c_char,
    boot_msg: *const c_char,
    comp_ids: *const c_uint,
    comp_num: c_uint,
}

#[repr(C)]
struct ExternComp {
    id: c_uint,
    boot_msg: *const c_char,
    attestation_loc: *const c_char,
    attestation_date: *const c_char,
    attestation_customer: *const c_char,
}

extern "C" {
    fn comp_or_ap() -> i32;
    fn get_comp() -> ExternComp;
    fn get_ap() -> ExternAP;
}

#[derive(Clone)]
pub enum DeviceKind {
    Component {
        id: u32,
        boot_msg: &'static str,
        attestation_loc: &'static str,
        attestation_date: &'static str,
        attestation_customer: &'static str,
    },
    ApplicationProcessor {
        ap_pin: &'static str,
        ap_token: &'static str,
        boot_msg: &'static str,
        comp_ids: &'static [u32],
    },
}

pub fn get_device() -> DeviceKind {
    match unsafe { comp_or_ap() } {
        // Comp
        0 => {
            let c_comp = unsafe { get_comp() };

            let id = c_comp.id.into();
            let boot_msg = unsafe { CStr::from_ptr(c_comp.boot_msg) }.to_str().unwrap();
            let attestation_loc = unsafe { CStr::from_ptr(c_comp.attestation_loc) }
                .to_str()
                .unwrap();
            let attestation_date = unsafe { CStr::from_ptr(c_comp.attestation_date) }
                .to_str()
                .unwrap();
            let attestation_customer = unsafe { CStr::from_ptr(c_comp.attestation_customer) }
                .to_str()
                .unwrap();

            DeviceKind::Component {
                id,
                boot_msg,
                attestation_loc,
                attestation_date,
                attestation_customer,
            }
        }
        // Ap
        1 => {
            let c_ap = unsafe { get_ap() };

            let ap_pin = unsafe { CStr::from_ptr(c_ap.ap_pin) }.to_str().unwrap();
            let ap_token = unsafe { CStr::from_ptr(c_ap.ap_token) }.to_str().unwrap();
            let boot_msg = unsafe { CStr::from_ptr(c_ap.boot_msg) }.to_str().unwrap();
            let comp_ids =
                unsafe { core::slice::from_raw_parts(c_ap.comp_ids, c_ap.comp_num as usize) };

            DeviceKind::ApplicationProcessor {
                ap_pin,
                ap_token,
                boot_msg,
                comp_ids,
            }
        }

        _ => unreachable!("We should not have anything other then 0=comp, 1=ap for 'comp_or_ap()'"),
    }
}

use std::ffi::c_void;

use windows::Win32::Security::Authorization::GetNamedSecurityInfoW;
use windows::Win32::Security::{
    ACCESS_ALLOWED_ACE, EqualSid, GetAce, GetSecurityDescriptorControl, PSECURITY_DESCRIPTOR,
    SE_DACL_PROTECTED,
};

use super::*;

struct Dacl {
    ace_count: u16,
    protected: bool,
    all_aces_are_current_user: bool,
}

fn read_dacl(path: &Path) -> Dacl {
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut dacl: *mut ACL = std::ptr::null_mut();
    let mut descriptor = PSECURITY_DESCRIPTOR::default();
    // SAFETY: out-pointers are valid; the descriptor is freed by LocalBox.
    check(unsafe {
        GetNamedSecurityInfoW(
            PCWSTR(wide.as_ptr()),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(&mut dacl),
            None,
            &mut descriptor,
        )
    })
    .expect("GetNamedSecurityInfoW");
    let _descriptor = LocalBox(descriptor.0);

    let mut control = 0u16;
    let mut revision = 0u32;
    // SAFETY: `descriptor` is valid until `_descriptor` drops.
    unsafe { GetSecurityDescriptorControl(descriptor, &mut control, &mut revision) }
        .expect("GetSecurityDescriptorControl");

    // SAFETY: `dacl` points into `descriptor`.
    let ace_count = unsafe { (*dacl).AceCount };
    let user = CurrentUser::query().expect("current user");
    let all_aces_are_current_user = (0..u32::from(ace_count)).all(|index| {
        let mut ace: *mut c_void = std::ptr::null_mut();
        // SAFETY: `index` is below the ACL's ACE count.
        unsafe { GetAce(dacl, index, &mut ace) }.expect("GetAce");
        // Only access-allowed ACEs are expected; the SID starts at `SidStart`.
        let ace = ace.cast::<ACCESS_ALLOWED_ACE>();
        let sid = PSID(unsafe { (&raw mut (*ace).SidStart).cast() });
        // SAFETY: both SIDs are valid for the duration of the call.
        unsafe { EqualSid(sid, user.sid()) }.is_ok()
    });

    Dacl {
        ace_count,
        protected: control & SE_DACL_PROTECTED.0 != 0,
        all_aces_are_current_user,
    }
}

#[test]
fn folders_and_files_are_restricted_to_the_current_user() {
    let dir = tempfile::tempdir().expect("tempdir");
    let secrets = dir.path().join("secrets");
    std::fs::create_dir(&secrets).expect("mkdir");
    restrict_to_current_user(&secrets).expect("restrict folder");

    // Windows stores an inheritable GENERIC_ALL entry as two ACEs (one for
    // the folder, one inherit-only for its children); both name the user.
    let folder = read_dacl(&secrets);
    assert!(folder.ace_count >= 1);
    assert!(folder.protected, "inheritance must be off");
    assert!(folder.all_aces_are_current_user);

    let file = secrets.join("owner.token");
    std::fs::write(&file, "x").expect("write");
    restrict_to_current_user(&file).expect("restrict file");
    let file = read_dacl(&file);
    assert_eq!(file.ace_count, 1);
    assert!(file.protected);
    assert!(file.all_aces_are_current_user);
}

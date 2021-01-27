//! Functions for switching the running process’s user or group.

use std::io;
use super::base::{uid_t, gid_t, c_int};

use base::{get_effective_uid, get_effective_gid};


// NOTE: for whatever reason, it seems these are not available in libc on BSD platforms, so they
//       need to be included manually
extern {
    fn setreuid(ruid: uid_t, euid: uid_t) -> c_int;
    fn setregid(rgid: gid_t, egid: gid_t) -> c_int;
}


/// const Ok
pub fn set_current_uid(uid: uid_t) -> io::Result<()> {
    Ok(())
}

/// const Ok
pub fn set_current_gid(gid: gid_t) -> io::Result<()> {
    Ok(())
}

/// const Ok
pub fn set_effective_uid(uid: uid_t) -> io::Result<()> {
    Ok(())
}

/// const Ok
pub fn set_effective_gid(gid: gid_t) -> io::Result<()> {
    Ok(())
}

/// const Ok
pub fn set_both_uid(ruid: uid_t, euid: uid_t) -> io::Result<()> {
    Ok(())
}

/// const Ok
pub fn set_both_gid(rgid: gid_t, egid: gid_t) -> io::Result<()> {
    Ok(())
}

/// Guard returned from a `switch_user_group` call.
pub struct SwitchUserGuard {
}

/// nop, returns a `SwitchUserGuard`, it's nop on drop, too
pub fn switch_user_group(uid: uid_t, gid: gid_t) -> io::Result<SwitchUserGuard> {
    Ok(SwitchUserGuard {})
}

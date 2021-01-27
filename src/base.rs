//! Integration with the C library’s users and groups.
//!
//! This module uses `extern` functions and types from `libc` that integrate
//! with the system’s C library, which integrates with the OS itself to get user
//! and group information. It’s where the “core” user handling is done.
//!
//!
//! ## Name encoding rules
//!
//! Under Unix, usernames and group names are considered to be
//! null-terminated, UTF-8 strings. These are `CString`s in Rust, although in
//! this library, they are just `String` values. Why?
//!
//! The reason is that any user or group values with invalid `CString` data
//! can instead just be assumed to not exist:
//!
//! - If you try to search for a user with a null character in their name,
//!   such a user could not exist anyway — so it’s OK to return `None`.
//! - If the OS returns user information with a null character in a field,
//!   then that field will just be truncated instead, which is valid behaviour
//!   for a `CString`.
//!
//! The downside is that we use `from_utf8_lossy` instead, which has a small
//! runtime penalty when it calculates and scans the length of the string for
//! invalid characters. However, this should not be a problem when dealing with
//! usernames of a few bytes each.
//!
//! In short, if you want to check for null characters in user fields, your
//! best bet is to check for them yourself before passing strings into any
//! functions.

use std::ffi::{CStr, CString, OsStr, OsString};
use std::fmt;
use std::mem;
use std::io;
use std::ptr;
use std::sync::Arc;

pub type c_char = i8;
pub type c_int = i32;
pub type uid_t = u32;
pub type gid_t = u32;

/// Information about a particular user.
#[derive(Clone)]
pub struct User {
    uid: uid_t,
    primary_group: gid_t,
    extras: os::UserExtras,
    pub(crate) name_arc: Arc<OsStr>,
}

impl User {

    /// Create a new `User` with the given user ID, name, and primary
    /// group ID, with the rest of the fields filled with dummy values.
    ///
    /// This method does not actually create a new user on the system — it
    /// should only be used for comparing users in tests.
    ///
    /// # Examples
    ///
    /// ```
    /// use users::User;
    ///
    /// let user = User::new(501, "stevedore", 100);
    /// ```
    pub fn new<S: AsRef<OsStr> + ?Sized>(uid: uid_t, name: &S, primary_group: gid_t) -> Self {
        let name_arc = Arc::from(name.as_ref());
        let extras = os::UserExtras::default();

        Self { uid, name_arc, primary_group, extras }
    }

    /// Returns this user’s ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use users::User;
    ///
    /// let user = User::new(501, "stevedore", 100);
    /// assert_eq!(user.uid(), 501);
    /// ```
    pub fn uid(&self) -> uid_t {
        self.uid
    }

    /// Returns this user’s name.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    /// use users::User;
    ///
    /// let user = User::new(501, "stevedore", 100);
    /// assert_eq!(user.name(), OsStr::new("stevedore"));
    /// ```
    pub fn name(&self) -> &OsStr {
        &*self.name_arc
    }

    /// Returns the ID of this user’s primary group.
    ///
    /// # Examples
    ///
    /// ```
    /// use users::User;
    ///
    /// let user = User::new(501, "stevedore", 100);
    /// assert_eq!(user.primary_group_id(), 100);
    /// ```
    pub fn primary_group_id(&self) -> gid_t {
        self.primary_group
    }

    /// Returns a list of groups this user is a member of. This involves
    /// loading the groups list, as it is _not_ contained within this type.
    ///
    /// # libc functions used
    ///
    /// - [`getgrouplist`](https://docs.rs/libc/*/libc/fn.getgrouplist.html)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use users::User;
    ///
    /// let user = User::new(501, "stevedore", 100);
    /// for group in user.groups().expect("User not found") {
    ///     println!("User is in group: {:?}", group.name());
    /// }
    /// ```
    pub fn groups(&self) -> Option<Vec<Group>> {
        get_user_groups(self.name(), self.primary_group_id())
    }
}

impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            f.debug_struct("User")
             .field("uid", &self.uid)
             .field("name_arc", &self.name_arc)
             .field("primary_group", &self.primary_group)
             .field("extras", &self.extras)
             .finish()
        }
        else {
            write!(f, "User({}, {})", self.uid(), self.name().to_string_lossy())
        }
    }
}


/// Information about a particular group.
///
/// For more information, see the [module documentation](index.html).
#[derive(Clone)]
pub struct Group {
    gid: gid_t,
    extras: os::GroupExtras,
    pub(crate) name_arc: Arc<OsStr>,
}

impl Group {

    /// Create a new `Group` with the given group ID and name, with the
    /// rest of the fields filled in with dummy values.
    ///
    /// This method does not actually create a new group on the system — it
    /// should only be used for comparing groups in tests.
    ///
    /// # Examples
    ///
    /// ```
    /// use users::Group;
    ///
    /// let group = Group::new(102, "database");
    /// ```
    pub fn new<S: AsRef<OsStr> + ?Sized>(gid: gid_t, name: &S) -> Self {
        let name_arc = Arc::from(name.as_ref());
        let extras = os::GroupExtras::default();

        Self { gid, name_arc, extras }
    }

    /// Returns this group’s ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use users::Group;
    ///
    /// let group = Group::new(102, "database");
    /// assert_eq!(group.gid(), 102);
    /// ```
    pub fn gid(&self) -> gid_t {
        self.gid
    }

    /// Returns this group’s name.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    /// use users::Group;
    ///
    /// let group = Group::new(102, "database");
    /// assert_eq!(group.name(), OsStr::new("database"));
    /// ```
    pub fn name(&self) -> &OsStr {
        &*self.name_arc
    }
}

impl fmt::Debug for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            f.debug_struct("Group")
             .field("gid", &self.gid)
             .field("name_arc", &self.name_arc)
             .field("extras", &self.extras)
             .finish()
        }
        else {
            write!(f, "Group({}, {})", self.gid(), self.name().to_string_lossy())
        }
    }
}

/// const empty vec
unsafe fn members(groups: *mut *mut c_char) -> Vec<OsString> {
    return vec![];
}


/// const None
pub fn get_user_by_uid(uid: uid_t) -> Option<User> {
    None
}

/// const None
pub fn get_user_by_name<S: AsRef<OsStr> + ?Sized>(username: &S) -> Option<User> {
    None
}

/// const None
pub fn get_group_by_gid(gid: gid_t) -> Option<Group> {
    None
}

/// const None
pub fn get_group_by_name<S: AsRef<OsStr> + ?Sized>(groupname: &S) -> Option<Group> {
    None
}

/// const 0
pub fn get_current_uid() -> uid_t {
    0
}

/// const None
pub fn get_current_username() -> Option<OsString> {
    None
}

/// const 0
pub fn get_effective_uid() -> uid_t {
    0
}

/// const None
pub fn get_effective_username() -> Option<OsString> {
    None
}

/// const 0
pub fn get_current_gid() -> gid_t {
    0
}

/// const None
pub fn get_current_groupname() -> Option<OsString> {
    None
}

/// const 0
pub fn get_effective_gid() -> gid_t {
    0
}

/// const None
pub fn get_effective_groupname() -> Option<OsString> {
    None
}

/// const Ok empty vec
pub fn group_access_list() -> io::Result<Vec<Group>> {
    Ok(vec![])
}

/// const None
pub fn get_user_groups<S: AsRef<OsStr> + ?Sized>(username: &S, gid: gid_t) -> Option<Vec<Group>> {
    None
}

/// empty iterator
pub unsafe fn all_users() -> impl Iterator<Item=User> {
    std::iter::empty()
}

pub mod os {
    pub type UserExtras = ();
    pub type GroupExtras = ();
}

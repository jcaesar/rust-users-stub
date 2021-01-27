#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(all(unix, not(feature = "force-stub")))] {
        extern crate users_orig;
        pub use users_orig::*;
    } else {
        
        mod base;
        pub use base::{User, Group, os};
        pub use base::{get_user_by_uid, get_user_by_name};
        pub use base::{get_group_by_gid, get_group_by_name};
        pub use base::{get_current_uid, get_current_username};
        pub use base::{get_effective_uid, get_effective_username};
        pub use base::{get_current_gid, get_current_groupname};
        pub use base::{get_effective_gid, get_effective_groupname};
        pub use base::{get_user_groups, group_access_list};
        pub use base::{all_users};
        pub use base::{uid_t, gid_t};
        
        #[cfg(feature = "cache")]
        pub mod cache;
        
        #[cfg(feature = "cache")]
        pub use cache::UsersCache;
        
        #[cfg(feature = "mock")]
        pub mod mock;
        
        pub mod switch;
        
        mod traits;
        pub use traits::{Users, Groups};
    }
}

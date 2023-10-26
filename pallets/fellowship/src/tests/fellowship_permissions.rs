use super::*;
use crate::impls::VetterAndFreelancerAllPermissions;

#[test]
fn get_permission_works_implementation() {
    new_test_ext().execute_with(|| {
        let freelancer_permissions = <<Test as Config>::Permissions as FellowshipPermissions<crate::Role, crate::Permission>>::get_permissions(Role::Freelancer);
        let vetter_permissions = <<Test as Config>::Permissions as FellowshipPermissions<crate::Role, crate::Permission>>::get_permissions(Role::Vetter);
        let actual = vec![Permission::AddToShortlist, Permission::RemoveFromShortlist];

        assert_eq!(freelancer_permissions, actual, "for implementation, freelancer should be able to modify shortlist");
        assert_eq!(vetter_permissions, actual, "for implementation, vetter should be able to modify shortlist");
    });
}

#[test]
fn has_permission_works() {
    new_test_ext().execute_with(|| {
        assert!(<<Test as Config>::Permissions as FellowshipPermissions<crate::Role, crate::Permission>>::has_permission(Role::Freelancer, Permission::AddToShortlist));
        assert!(<<Test as Config>::Permissions as FellowshipPermissions<crate::Role, crate::Permission>>::has_permission(Role::Freelancer, Permission::RemoveFromShortlist));
        assert!(<<Test as Config>::Permissions as FellowshipPermissions<crate::Role, crate::Permission>>::has_permission(Role::Vetter, Permission::AddToShortlist));
        assert!(<<Test as Config>::Permissions as FellowshipPermissions<crate::Role, crate::Permission>>::has_permission(Role::Vetter, Permission::RemoveFromShortlist));
    });
}

#[test]
fn has_permission_work_negative() {
    new_test_ext().execute_with(|| {
        assert!(!<<Test as Config>::Permissions as FellowshipPermissions<crate::Role, crate::Permission>>::has_permission(Role::Freelancer, Permission::None));
        assert!(!<<Test as Config>::Permissions as FellowshipPermissions<crate::Role, crate::Permission>>::has_permission(Role::Vetter, Permission::None));
    });
}

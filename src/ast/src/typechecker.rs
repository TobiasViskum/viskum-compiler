use std::array::IntoIter;

use ir::{ DefIdToNameBinding, Ty };

pub struct TypeChecker;

#[derive(Debug)]
pub enum TypeCheckError {
    RequiresMutability,
    MismatchedTypes,
}

impl TypeChecker {
    // Requires pointer mutability to be the same
    pub fn test_valid_arg<'a>(
        arg_cmp: ArgCmp,
        def_id_to_name_binding: &DefIdToNameBinding
    ) -> Result<(), IntoIter<Option<TypeCheckError>, 4>> {
        let mut error_len = 0;
        let mut errors: [Option<TypeCheckError>; 4] = [const { None }; 4];

        if arg_cmp.arg_ty.is_mut_ptr() && !arg_cmp.provided_ty.is_mut_ptr() {
            errors[error_len] = Some(TypeCheckError::RequiresMutability);
            error_len += 1;
        }

        let full_arg_ty = arg_cmp.arg_ty.get_expanded_dereffed_ty(def_id_to_name_binding);
        let full_provided_ty = arg_cmp.provided_ty.get_expanded_dereffed_ty(def_id_to_name_binding);

        if full_arg_ty != full_provided_ty {
            errors[error_len] = Some(TypeCheckError::MismatchedTypes);
            error_len += 1;
        }

        if error_len > 0 {
            return Err(errors.into_iter());
        } else {
            Ok(())
        }
    }

    pub fn test_eq_loose<'a>(
        ty1: Ty,
        ty2: Ty,
        def_id_to_name_binding: &DefIdToNameBinding
    ) -> Result<(), IntoIter<Option<TypeCheckError>, 4>> {
        let mut error_len = 0;
        let mut errors: [Option<TypeCheckError>; 4] = [const { None }; 4];

        if ty1.is_ptr() && ty2.is_null() {
            return Ok(());
        } else if ty1.is_null() && ty2.is_ptr() {
            return Ok(());
        }

        let full_ty1 = ty1.get_expanded_dereffed_ty(def_id_to_name_binding);
        let full_ty2 = ty2.get_expanded_dereffed_ty(def_id_to_name_binding);

        if full_ty1 != full_ty2 {
            errors[error_len] = Some(TypeCheckError::MismatchedTypes);
            error_len += 1;
        }

        if error_len > 0 {
            return Err(errors.into_iter());
        } else {
            Ok(())
        }
    }
}

pub struct ArgCmp {
    pub arg_ty: Ty,
    pub provided_ty: Ty,
}

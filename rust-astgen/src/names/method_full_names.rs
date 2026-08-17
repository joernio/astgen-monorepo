//! Where we finally build `methodFullName` for each call.

use super::{
    rust_name_formatter::{
        format_generic_args_for_def, format_item_name, format_member_full_name,
        format_module_def_full_name, format_name_with_generic_args,
        format_trait_impl_member_full_name,
    },
    type_formatter,
};
use ra_ap_hir::{
    AsAssocItem, AssocItemContainer, CallableKind, EnumVariant, Function, GenericDef, Module,
    ModuleDef, Name, PathResolution, Semantics, Struct, TraitRef,
};
use ra_ap_ide::RootDatabase;
use ra_ap_syntax::{AstNode, SyntaxNode, ast, match_ast};

pub(crate) fn method_full_name_for_node(
    node: &SyntaxNode,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    match_ast! {
        match node {
            ast::CallExpr(call_expr) => resolve_call_expr_full_name(&call_expr, semantics),
            ast::MethodCallExpr(method_call_expr) => resolve_method_call_expr_full_name(&method_call_expr, semantics),
            ast::Struct(struct_) => resolve_struct_ctor_full_name(&struct_, semantics),
            ast::Fn(fn_) => resolve_fn_def_full_name(&fn_, semantics),
            ast::PathExpr(path_expr) => resolve_path_expr_full_name(&path_expr, semantics),
            _ => None,
        }
    }
}

fn resolve_method_call_expr_full_name<'db>(
    method_call_expr: &ast::MethodCallExpr,
    semantics: &Semantics<'db, RootDatabase>,
) -> Option<String> {
    let function = semantics.resolve_method_call(method_call_expr)?;
    format_function_full_name(function, semantics.db)
}

fn resolve_path_expr_full_name<'db>(
    path_expr: &ast::PathExpr,
    semantics: &Semantics<'db, RootDatabase>,
) -> Option<String> {
    let path = path_expr.path()?;
    match semantics.resolve_path(&path)? {
        PathResolution::Def(ModuleDef::Function(f)) => format_function_full_name(f, semantics.db),
        _ => None,
    }
}

fn resolve_call_expr_full_name<'db>(
    call_expr: &ast::CallExpr,
    semantics: &Semantics<'db, RootDatabase>,
) -> Option<String> {
    let callee_expr = call_expr.expr()?;
    if let ast::Expr::PathExpr(path_expr) = &callee_expr
        && let Some(name) = resolve_path_expr_full_name(path_expr, semantics)
    {
        return Some(name);
    }
    match semantics.resolve_expr_as_callable(&callee_expr)?.kind() {
        CallableKind::Function(function) => format_function_full_name(function, semantics.db),
        CallableKind::TupleStruct(tuple_struct) => {
            format_tuple_struct_ctor_full_name(tuple_struct, semantics.db)
        }
        CallableKind::TupleEnumVariant(enum_variant) => {
            format_enum_variant_full_name(enum_variant, semantics.db)
        }
        // TODO(xavierp): need more time to understand what these should be named as
        CallableKind::Closure(_) | CallableKind::FnPtr | CallableKind::FnImpl(_) => None,
    }
}

fn resolve_struct_ctor_full_name<'db>(
    struct_: &ast::Struct,
    semantics: &Semantics<'db, RootDatabase>,
) -> Option<String> {
    // We provide a `methodFullName` at struct definition to match its constructor name.
    // Only tuple structs have a callable constructor. Record/Unit structs have RecordExpr
    // and IdentExpr at call-site.
    // So, when we see "MyTuple(...)" (call-site), the `methodFullName` for this call shall
    // match the `methodFullName` for the struct ctor which we synthesize in Joern.
    let Some(ast::FieldList::TupleFieldList(_)) = struct_.field_list() else {
        return None;
    };
    let struct_def = semantics.to_def(struct_)?;
    format_tuple_struct_ctor_full_name(struct_def, semantics.db)
}

fn resolve_fn_def_full_name<'db>(
    fn_: &ast::Fn,
    semantics: &Semantics<'db, RootDatabase>,
) -> Option<String> {
    let function = semantics.to_def(fn_)?;
    format_function_full_name(function, semantics.db)
}

pub(crate) fn format_function_full_name(function: Function, db: &RootDatabase) -> Option<String> {
    let Some(assoc_item) = function.as_assoc_item(db) else {
        return format_generic_module_def_full_name(
            ModuleDef::from(function),
            GenericDef::from(function),
            function.module(db),
            db,
        );
    };

    let method_name = format_generic_item_name(
        function.name(db),
        GenericDef::from(function),
        function.module(db),
        db,
    );
    match assoc_item.container(db) {
        AssocItemContainer::Impl(impl_) => {
            let receiver_type_name =
                type_formatter::format_impl_self_ty(impl_, impl_.module(db), db)?;

            if let Some(trait_ref) = impl_.trait_ref(db) {
                let trait_name = format_trait_ref_full_name(trait_ref, impl_.module(db), db)?;
                Some(format_trait_impl_member_full_name(
                    &receiver_type_name,
                    &trait_name,
                    &method_name,
                ))
            } else {
                Some(format_member_full_name(&receiver_type_name, &method_name))
            }
        }
        AssocItemContainer::Trait(trait_) => {
            let trait_name = format_generic_module_def_full_name(
                ModuleDef::from(trait_),
                GenericDef::from(trait_),
                trait_.module(db),
                db,
            )?;
            Some(format_member_full_name(&trait_name, &method_name))
        }
    }
}

pub(super) fn format_generic_module_def_full_name(
    def: ModuleDef,
    generic_def: GenericDef,
    module: Module,
    db: &RootDatabase,
) -> Option<String> {
    let base = format_module_def_full_name(def, db)?;
    Some(format_generic_name(base, generic_def, module, db))
}

fn format_generic_item_name(
    name: Name,
    generic_def: GenericDef,
    module: Module,
    db: &RootDatabase,
) -> String {
    let base = format_item_name(name, module, db);
    format_generic_name(base, generic_def, module, db)
}

fn format_generic_name(
    base: String,
    generic_def: GenericDef,
    module: Module,
    db: &RootDatabase,
) -> String {
    let generic_args = format_generic_args_for_def(generic_def, module, db);
    format_name_with_generic_args(base, generic_args)
}

pub(crate) fn format_tuple_struct_ctor_full_name(
    struct_: Struct,
    db: &RootDatabase,
) -> Option<String> {
    format_generic_module_def_full_name(
        ModuleDef::from(struct_),
        GenericDef::from(struct_),
        struct_.module(db),
        db,
    )
}

pub(crate) fn format_enum_variant_full_name(
    enum_variant: EnumVariant,
    db: &RootDatabase,
) -> Option<String> {
    let enum_ = enum_variant.parent_enum(db);
    let enum_name = format_generic_module_def_full_name(
        ModuleDef::from(enum_),
        GenericDef::from(enum_),
        enum_.module(db),
        db,
    )?;
    let variant_name = format_item_name(enum_variant.name(db), enum_variant.module(db), db);
    Some(format_member_full_name(&enum_name, &variant_name))
}

fn format_trait_ref_full_name<'db>(
    trait_ref: TraitRef<'db>,
    module: Module,
    db: &'db RootDatabase,
) -> Option<String> {
    let trait_ = trait_ref.trait_();
    let base = format_module_def_full_name(ModuleDef::from(trait_), db)?;
    let arg_count = trait_.type_or_const_param_count(db, false);
    // Self is 0, type args are 1+
    let generic_args = (1..=arg_count)
        .map(|idx| {
            let arg = trait_ref.get_type_argument(idx)?;
            type_formatter::format_type(&arg.to_type(db), module, db)
        })
        .collect::<Option<Vec<_>>>()?;

    Some(format_name_with_generic_args(base, generic_args))
}

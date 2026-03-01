use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, ItemStruct, ItemTrait, ItemImpl, Lit, Meta};

/// Procedural macro to generate builtin declarations for loft
/// 
/// This macro simplifies the creation of builtin functions, structs, and traits
/// by automatically generating the necessary boilerplate code.
/// 
/// # Usage
/// 
/// ## For functions (generates builtin method wrappers):
/// ```rust
/// #[loft_builtin(term.read)]
/// fn read(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
///     // Implementation
/// }
/// ```
/// 
/// ## For structs (preserves struct definition):
/// ```rust
/// #[loft_builtin(fs::file)]
/// pub struct File(std::fs::File);
/// ```
/// 
/// ## For impl blocks (implements traits or methods onto builtins):
/// ```rust
/// #[loft_builtin(fs::file)]
/// impl File {
///     // Methods will be available on File instances
/// }
/// ```
/// 
/// ## For traits (preserves trait definition with documentation):
/// ```rust
/// #[loft_builtin(add)]
/// /// Trait for addition operations
/// trait Add<T> {
///     fn add(self, other: T) -> Self;
/// }
/// ```
#[proc_macro_attribute]
pub fn loft_builtin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_str = attr.to_string();
    
    // Parse the input to determine what type of item it is
    if let Ok(func) = syn::parse::<ItemFn>(item.clone()) {
        handle_function(attr_str, func)
    } else if let Ok(structure) = syn::parse::<ItemStruct>(item.clone()) {
        handle_struct(attr_str, structure)
    } else if let Ok(trait_item) = syn::parse::<ItemTrait>(item.clone()) {
        handle_trait(attr_str, trait_item)
    } else if let Ok(impl_item) = syn::parse::<ItemImpl>(item.clone()) {
        handle_impl(attr_str, impl_item)
    } else {
        TokenStream::from(quote! {
            compile_error!("loft_builtin can only be applied to functions, structs, traits, or impl blocks");
        })
    }
}

#[proc_macro_attribute]
pub fn required(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn types(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

fn extract_doc_comments(attrs: &[syn::Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let Meta::NameValue(meta) = &attr.meta {
                    if let syn::Expr::Lit(expr_lit) = &meta.value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            return Some(lit_str.value());
                        }
                    }
                }
            }
            None
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn handle_function(_path: String, mut func: ItemFn) -> TokenStream {
    // Parse arguments and look for attributes
    let mut check_logic = Vec::new();
    let mut required_args = 0;
    let mut type_checks = Vec::new();

    for arg in func.sig.inputs.iter_mut() {
        if let syn::FnArg::Typed(pat_type) = arg {
            // Check if it's the `args` parameter (usually `&[Value]`)
            let is_args_param = if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                pat_ident.ident == "args"
            } else {
                false
            };

            let mut i = 0;
            while i < pat_type.attrs.len() {
                let attr = &pat_type.attrs[i];
                if attr.path().is_ident("required") {
                    required_args += 1;
                    pat_type.attrs.remove(i);
                } else if attr.path().is_ident("types") && is_args_param {
                    if let Meta::List(list) = &attr.meta {
                        let tokens = &list.tokens;
                        let type_str = tokens.to_string().replace(" ", "");
                        
                        let types: Vec<String> = type_str.split(',')
                            .map(|s| s.trim().to_string())
                            .collect();
                        
                        for (idx, type_name) in types.iter().enumerate() {
                            let is_varargs = type_name.ends_with('*');
                            let base_type = if is_varargs {
                                type_name[..type_name.len()-1].to_string()
                            } else {
                                type_name.clone()
                            };

                            let type_match = match base_type.as_str() {
                                "bool" => quote! { Value::Boolean(_) },
                                "string" => quote! { Value::String(_) },
                                "number" => quote! { Value::Number(_) },
                                "array" => quote! { Value::Array(_) },
                                "object" => quote! { Value::Struct { .. } },
                                _ => quote! { _ },
                            };

                            let error_msg = format!("Argument must be of type {}", base_type);
                            
                            if is_varargs {
                                type_checks.push(quote! {
                                    for val in &args[#idx..] {
                                        if !matches!(val, #type_match) {
                                            return Err(RuntimeError::new(#error_msg));
                                        }
                                    }
                                });
                            } else {
                                type_checks.push(quote! {
                                    if let Some(val) = args.get(#idx) {
                                        if !matches!(val, #type_match) {
                                            return Err(RuntimeError::new(#error_msg));
                                        }
                                    }
                                });
                            }
                        }
                    }
                    pat_type.attrs.remove(i);
                } else {
                    i += 1;
                }
            }
        }
    }

    if required_args > 0 {
        let min_args = if required_args > 0 { required_args - 1 } else { 0 };
        let min_args_val = min_args as usize;
        let error_msg = format!("Function requires at least {} arguments", min_args);
        check_logic.push(quote! {
            if args.is_empty() && #min_args_val > 0 {
                 return Err(RuntimeError::new(#error_msg));
            }
            if args.len() < #min_args_val {
                return Err(RuntimeError::new(#error_msg));
            }
        });
    }
    
    check_logic.extend(type_checks);

    let fn_vis = &func.vis;
    let fn_attrs = &func.attrs;
    let fn_sig = &func.sig;
    let fn_block = &func.block;
    
    let expanded = quote! {
        #(#fn_attrs)*
        #[allow(clippy::duplicated_attributes)]
        #fn_vis #fn_sig {
            #(#check_logic)*
            #fn_block
        }
    };
    
    TokenStream::from(expanded)
}

fn handle_struct(path: String, structure: ItemStruct) -> TokenStream {
    let struct_vis = &structure.vis;
    let struct_attrs = &structure.attrs;
    let struct_name = &structure.ident;
    let struct_fields = &structure.fields;
    let struct_generics = &structure.generics;
    let struct_semi = &structure.semi_token;
    
    // Extract documentation for potential use
    let _doc_string = extract_doc_comments(struct_attrs);
    let _path = path;
    
    // Just preserve the struct definition without generating a create function
    // The semi_token handles the semicolon for tuple and unit structs
    let expanded = quote! {
        #(#struct_attrs)*
        #struct_vis struct #struct_name #struct_generics #struct_fields #struct_semi
    };
    
    TokenStream::from(expanded)
}

fn handle_trait(path: String, trait_item: ItemTrait) -> TokenStream {
    let trait_name = &trait_item.ident;
    let trait_vis = &trait_item.vis;
    let trait_attrs = &trait_item.attrs;
    let trait_items = &trait_item.items;
    let trait_generics = &trait_item.generics;
    let trait_supertraits = &trait_item.supertraits;
    
    // Extract documentation
    let _doc_string = extract_doc_comments(trait_attrs);
    let _trait_path = path;
    
    // Just pass through the trait definition
    let expanded = quote! {
        #(#trait_attrs)*
        #trait_vis trait #trait_name #trait_generics: #trait_supertraits {
            #(#trait_items)*
        }
    };
    
    TokenStream::from(expanded)
}

fn handle_impl(path: String, impl_item: ItemImpl) -> TokenStream {
    let impl_attrs = &impl_item.attrs;
    let impl_generics = &impl_item.generics;
    let impl_trait = &impl_item.trait_;
    let self_ty = &impl_item.self_ty;
    let impl_items = &impl_item.items;
    let where_clause = &impl_item.generics.where_clause;
    
    // Extract documentation
    let _doc_string = extract_doc_comments(impl_attrs);
    let _impl_path = path;
    
    // Pass through the impl block preserving all content
    let expanded = if let Some((bang, trait_path, for_token)) = impl_trait {
        quote! {
            #(#impl_attrs)*
            impl #impl_generics #bang #trait_path #for_token #self_ty #where_clause {
                #(#impl_items)*
            }
        }
    } else {
        quote! {
            #(#impl_attrs)*
            impl #impl_generics #self_ty #where_clause {
                #(#impl_items)*
            }
        }
    };
    
    TokenStream::from(expanded)
}



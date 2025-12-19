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

fn handle_function(path: String, func: ItemFn) -> TokenStream {
    let _fn_name = &func.sig.ident;
    let fn_vis = &func.vis;
    let fn_attrs = &func.attrs;
    let fn_sig = &func.sig;
    let fn_block = &func.block;
    
    // Extract documentation comments for metadata
    let _doc_string = extract_doc_comments(fn_attrs);
    
    // Parse the path to extract information
    let parts: Vec<&str> = path.split('.').collect();
    let _metadata = if parts.len() >= 2 {
        format!("builtin: {}, method: {}", parts[0], parts[1])
    } else {
        format!("method: {}", path)
    };
    
    // For now, just pass through the function as-is
    // In a more complete implementation, we could generate conversion code
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig #fn_block
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



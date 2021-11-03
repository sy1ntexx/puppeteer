use proc_macro2::TokenStream as TokenStream2;
use proc_macro::TokenStream;
use quote::quote;
use syn::*;

#[proc_macro_derive(Puppeteer, attributes(id))]
pub fn packets(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let packets_ident = &input.ident;

    let data_enum = match input.data {
        Data::Enum(e) => e,
        _ => panic!("Only enums can be represented as packets")
    };

    let ser = make_serialize(&data_enum);
    let de = make_deserialize(&data_enum);

    let from_impls = make_from_impls(
        packets_ident,
        &data_enum
    );

    let out = quote! {
        impl #packets_ident {
            #ser
            #de
        }

        #(#from_impls)*
    };

    out.into()
}

fn make_serialize(target: &DataEnum) -> TokenStream2 {
    let ser_arms = generate_ser_arms(target);

    quote! {
        #[allow(dead_code)]
        pub fn serialize(packet: impl Into<Self>, buf: &mut ::puppeteer::BytesMut) {
            use ::puppeteer::Packet;
            use ::puppeteer::PacketId;
    
            match packet.into() {
                #(#ser_arms),*
            }
        }
    }
}

fn generate_ser_arms<'a>(target: &'a DataEnum) -> impl Iterator<Item = TokenStream2> + 'a {
    target.variants.iter().cloned().map(|v| {
        let field = match v.fields {
            Fields::Unnamed(f) => {
                assert_eq!(
                    f.unnamed.len(),
                    1,
                    "Packet's variant can contain only 1 unnamed type"
                );

                f.unnamed.first().cloned().unwrap()
            },
            _ => panic!("Packet's variant must be unnamed and contain only 1 type")
        };
        
        let field_type = &field.ty;
        let var_ident = &v.ident;

        quote! {
            Self::#var_ident(p) => {
                <#field_type as ::puppeteer::Packet>::ID.put_id(buf);
                p.serialize(buf);
            }
        }
    })
}

fn make_deserialize(target: &DataEnum) -> TokenStream2 {
    // we are gonna use first enum variant's field
    // to determinate type of all packets' errors and ids.

    let first_field_type = match &target.variants.first().expect("Packets enum cannot be empty").fields {
        Fields::Unnamed(f) => f.unnamed.first().cloned().unwrap().ty,
        _ => panic!("Packet's variant must be unnamed and contain only 1 type")
    };

    let de_arms = generate_de_arms(target);

    quote! {
        #[allow(dead_code)]
        pub fn deserialize(buf: &mut ::puppeteer::Bytes)
        -> std::result::Result<
            std::result::Result<Self, <#first_field_type as ::puppeteer::Packet>::Error>,
            <#first_field_type as ::puppeteer::Packet>::IdType
        >
        {
            use ::puppeteer::PacketId;

            let id = <#first_field_type as ::puppeteer::Packet>::IdType::get_id(buf).expect("Failed to read packet id");
            match id {
                #(#de_arms),*,
                _ => Err(id)
            }
        }
    }
}

fn generate_de_arms<'a>(target: &'a DataEnum) -> impl Iterator<Item = TokenStream2> + 'a {
    target.variants.iter().map(|v| {
        let var_inner_type = match &v.fields {
            Fields::Unnamed(f) => {
                assert_eq!(f.unnamed.len(), 1);
                &f.unnamed.first().as_ref().unwrap().ty
            },
            _ => panic!("Packet's variant must be unnamed and contain only 1 type")
        };

        let var_ident = &v.ident;

        quote! {
            <#var_inner_type as ::puppeteer::Packet>::ID => Ok(
                <#var_inner_type as ::puppeteer::Packet>::deserialize(buf)
                    .map(Self::#var_ident)
            )
        }
    })
}

fn make_from_impls<'a>(ident: &'a Ident, target: &'a DataEnum) -> impl Iterator<Item = TokenStream2> + 'a {
    target.variants.iter().map(move |v| {
        let var_inner_type = match &v.fields {
            Fields::Unnamed(f) => {
                assert_eq!(f.unnamed.len(), 1);
                &f.unnamed.first().as_ref().unwrap().ty
            },
            _ => panic!("Packet's variant must be unnamed and contain only 1 type")
        };

        let var_ident = &v.ident;

        quote! {
            impl From<#var_inner_type> for #ident {
                fn from(p: #var_inner_type) -> Self {
                    Self::#var_ident(p)
                }
            }
        }
    })
}
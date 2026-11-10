use agent_definition::agent_definition;
use input_definition::input_definition;
use input_display::input_display;
use input_from::input_from;
use syn::parse_quote;

use crate::codegen::agent::agent_profile::{impl_agent_profile, trait_agent_profile};
use crate::codegen::agent::ext::{
    impl_agent_ext, impl_session_ext, trait_agent_ext, trait_session_ext,
};
use crate::codegen::agent::fn_any::fn_any;
use crate::codegen::agent::fn_default::{fn_default, fn_default_model};
use crate::codegen::agent::fn_fingerprint::fn_fingerprint;
use crate::codegen::agent::fn_new::fn_new;
use crate::codegen::agent::fn_session::fn_session;
use crate::codegen::agent::input_default::input_default;
use crate::codegen::agent::input_macro::input_macro;
use crate::codegen::agent::input_new::input_new;
use crate::codegen::agent::input_newtypes::input_newtypes;
use crate::codegen::agent::output_definition::output_definition;
use crate::codegen::agent::variables::Variables;
use crate::codegen::common::symbols::{
    get_agent_symbol, get_ctor_symbol, get_input_symbol, get_output_symbol,
};
use crate::codegen::common::{
    get_agent_ext_symbol, get_agent_profile_symbol, get_agent_session_ext_symbol, talin,
};
use crate::config::Agent;
use crate::options::ArtifactOptions;

mod agent_definition;
mod agent_profile;
mod ext;

mod fn_any;
mod fn_default;
mod fn_fingerprint;
mod fn_new;
mod fn_session;

mod input_default;
mod input_definition;
mod input_display;
mod input_from;
mod input_macro;
mod input_new;
mod input_newtypes;

mod output_definition;

mod options;
mod tool_params;
mod tools;
mod variables;

pub(crate) fn generate_agent(id: &str, agent: &Agent, options: &ArtifactOptions) -> syn::File {
    let talin = talin();

    let agent_symbol = get_agent_symbol(id);
    let ext_symbol = get_agent_ext_symbol(id);
    let session_ext_symbol = get_agent_session_ext_symbol(id);
    let profile_symbol = get_agent_profile_symbol(id);
    let input_symbol = get_input_symbol(&agent_symbol);
    let output_symbol = get_output_symbol(&agent_symbol);
    let input_ctor = get_ctor_symbol(&input_symbol);
    let output_ctor = if agent.response_format.is_some() {
        get_ctor_symbol(&output_symbol)
    } else {
        parse_quote! { #talin::chain::StringCtor }
    };

    let variables = Variables::collect(&agent.variables, &agent.attachments, options);

    let input_newtypes = input_newtypes(&input_symbol, &variables);
    let input_definition = input_definition(&input_symbol, &variables);
    let input_new = input_new(&input_symbol, &variables);
    let input_default = input_default(&input_symbol, &variables);
    let input_from = input_from(&input_symbol, &variables);
    let input_display = input_display(&input_symbol, &variables);
    let input_macro = input_macro(&input_symbol);

    let output_definition = output_definition(&output_symbol, &agent.response_format);

    let agent_definition = agent_definition(&agent_symbol);
    let trait_agent_ext = trait_agent_ext(
        &ext_symbol,
        &input_symbol,
        &input_ctor,
        &output_ctor,
        &variables,
    );
    let trait_session_ext = trait_session_ext(
        &session_ext_symbol,
        &input_symbol,
        &input_ctor,
        &output_ctor,
        &variables,
    );
    let trait_agent_profile = trait_agent_profile(agent, &profile_symbol, &output_ctor);

    let fn_new = fn_new(id, &profile_symbol, &input_ctor, &output_ctor);
    let fn_session = fn_session(&profile_symbol, &input_ctor, &output_ctor);
    let fn_any = fn_any(&input_ctor, &output_ctor);
    let fn_default_model = agent
        .model
        .as_ref()
        .map(|(model, preset)| fn_default_model(model, preset));
    let fn_default = agent
        .model
        .as_ref()
        .map(|(_, preset)| fn_default(preset, &input_ctor, &output_ctor))
        .into_iter()
        .flatten();
    let fn_fingerprint = fn_fingerprint(&agent.fingerprint);

    let impl_agent_ext = impl_agent_ext(&ext_symbol, &input_ctor, &output_ctor);
    let impl_session_ext = impl_session_ext(&session_ext_symbol, &input_ctor, &output_ctor);
    let impl_agent_profile = impl_agent_profile(id, agent, &profile_symbol, &output_ctor);

    parse_quote! {
        #[allow(unused_imports)]
        use super::*;

        #[allow(unused_imports)]
        use talin::__private::{MapInsertElem, PeelOption, SetOption, VecPushElem};

        #input_newtypes

        #input_definition
        #input_new
        #input_default
        #input_from
        #input_display
        #input_macro

        #output_definition

        #agent_definition
        #[allow(dead_code)]
        impl #agent_symbol {
            #fn_new
            #fn_session
            #fn_any
            #fn_default_model
            #(#fn_default)*
            #fn_fingerprint
        }

        #trait_agent_ext
        #impl_agent_ext

        #trait_session_ext
        #impl_session_ext

        #trait_agent_profile
        #(#impl_agent_profile)*
    }
}

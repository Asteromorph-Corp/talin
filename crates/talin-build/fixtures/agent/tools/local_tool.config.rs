fn __dummy() {
    let _ = {
        const _: () = {
            #[allow(unused_imports)]
            use ::talin::__private::ConfigProbeFallback as _;
            const SUPPLIED: &[::talin::__private::Supplied] =
                &[::talin::__private::Supplied::leaf("query")];
            const {
                assert!(
                    ::talin::__private::ConfigProbe::<
                        <MyTool as ::talin::tools::FromConfig>::Config,
                    >::SCHEMA
                        .is_complete(SUPPLIED),
                    "the config supplied for tool `my_tool` is missing a required field."
                )
            }
        };
        <MyTool as ::talin::tools::FromConfig>::from_config({
            let mut __x = <MyTool as ::talin::tools::FromConfig>::Config::default();
            talin::__private::navigate!(__x.query).set_from("rust");
            __x
        })
    };
}

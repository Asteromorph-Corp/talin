fn tools() -> Vec<::talin::tools::Tool> {
    vec![
        ::talin::tools::Tool::from({
            const _: () = {
                #[allow(unused_imports)]
                use ::talin::__private::ConfigProbeFallback as _;
                const SUPPLIED: &[::talin::__private::Supplied] = &[];
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
            #[allow(clippy::unit_arg)]
            <MyTool as ::talin::tools::FromConfig>::from_config(
                <MyTool as ::talin::tools::FromConfig>::Config::default(),
            )
        }),
        ::talin::tools::Tool::from({
            const _: () = {
                #[allow(unused_imports)]
                use ::talin::__private::ConfigProbeFallback as _;
                const SUPPLIED: &[::talin::__private::Supplied] = &[];
                const {
                    assert!(
                        ::talin::__private::ConfigProbe::<
                            <MyOtherTool as ::talin::tools::FromConfig>::Config,
                        >::SCHEMA
                            .is_complete(SUPPLIED),
                        "the config supplied for tool `my_other_tool` is missing a required field."
                    )
                }
            };
            #[allow(clippy::unit_arg)]
            <MyOtherTool as ::talin::tools::FromConfig>::from_config(
                <MyOtherTool as ::talin::tools::FromConfig>::Config::default(),
            )
        }),
    ]
}

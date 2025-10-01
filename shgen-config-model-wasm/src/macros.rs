macro_rules! core_to_wasm_wrapper {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident($core:ty);
        $($rest:tt)*
    ) => {
        core_to_wasm_wrapper! {
            @parse
            { $(#[$meta])* }
            { $vis }
            { $name }
            { $core }
            [] // constructor
            [] // getters
            [$($rest)*]
        }
    };

    (
        @parse
        { $($meta:tt)* }
        { $vis:vis }
        { $name:ident }
        { $core:ty }
        []
        [$($getters:tt)*]
        [constructor($($param:ident: $param_ty:ty),* $(,)?) $body:block $($rest:tt)*]
    ) => {
        core_to_wasm_wrapper! {
            @parse
            { $($meta)* }
            { $vis }
            { $name }
            { $core }
            [($($param: $param_ty),*) $body]
            [$($getters)*]
            [$($rest)*]
        }
    };

    (
        @parse
        { $($meta:tt)* }
        { $vis:vis }
        { $name:ident }
        { $core:ty }
        [$($ctor:tt)*]
        []
        [getters { $($getter_block:tt)* } $($rest:tt)*]
    ) => {
        core_to_wasm_wrapper! {
            @parse
            { $($meta)* }
            { $vis }
            { $name }
            { $core }
            [$($ctor)*]
            [$($getter_block)*]
            [$($rest)*]
        }
    };

    (
        @parse
        { $($meta:tt)* }
        { $vis:vis }
        { $name:ident }
        { $core:ty }
        [$($ctor:tt)*]
        [$($getters:tt)*]
        []
    ) => {
        #[wasm_bindgen]
        $($meta)*
        $vis struct $name($core);

        impl From<$core> for $name {
            fn from(inner: $core) -> Self {
                Self(inner)
            }
        }

        impl From<$name> for $core {
            fn from(wrapper: $name) -> Self {
                wrapper.0
            }
        }

        core_to_wasm_wrapper!(@impl { $name } [$($ctor)*] [$($getters)*]);
    };

    (
        @impl
        { $name:ident }
        [($($param:ident: $param_ty:ty),*) $body:block]
        [
            $(
                $(#[$meta:meta])*
                $field:ident -> $ret_ty:ty => |$self_param:ident| $getter_body:expr
            );* $(;)?
        ]
    ) => {
        #[wasm_bindgen]
        impl $name {
            #[wasm_bindgen(constructor)]
            #[allow(clippy::missing_const_for_fn)]
            #[must_use]
            pub fn new($($param: $param_ty),*) -> Self $body

            $(
                $(#[$meta])*
                #[allow(clippy::missing_const_for_fn)]
                #[wasm_bindgen(getter)]
                pub fn $field(&self) -> $ret_ty {
                    let $self_param = self;
                    $getter_body
                }
            )*
        }
    };

    (
        @impl
        { $name:ident }
        []
        [
            $(
                $(#[$meta:meta])*
                $field:ident -> $ret_ty:ty => |$self_param:ident| $getter_body:expr
            );+ $(;)?
        ]
    ) => {
        #[wasm_bindgen]
        impl $name {
            $(
                $(#[$meta])*
                #[allow(clippy::missing_const_for_fn)]
                #[wasm_bindgen(getter)]
                pub fn $field(&self) -> $ret_ty {
                    let $self_param = self;
                    $getter_body
                }
            )*
        }
    };

    (@impl { $name:ident } [] []) => {};
}

macro_rules! core_enum_to_wasm {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident => $core:ty {
            $($variant:ident),* $(,)?
        }
    ) => {
        #[wasm_bindgen]
        $(#[$meta])*
        $vis enum $name {
            $($variant),*
        }

        impl From<$name> for $core {
            fn from(value: $name) -> Self {
                match value {
                    $($name::$variant => Self::$variant),*
                }
            }
        }
    };
}

pub(crate) use {core_enum_to_wasm, core_to_wasm_wrapper};

// @generated automatically by Diesel CLI.

diesel::table! {
    authapp_user (id) {
        id -> Int8,
        created_on -> Timestamptz,
        updated_on -> Timestamptz,
        #[max_length = 127]
        name -> Nullable<Text>,
        #[max_length = 20]
        phone -> Nullable<Text>,
        #[max_length = 50]
        email -> Nullable<Text>,
        active -> Bool,
        last_login -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    authapp_user_otp (id) {
        id -> Int8,
        created_on -> Timestamptz,
        updated_on -> Timestamptz,
        #[max_length = 50]
        email -> Nullable<Text>,
        #[max_length = 20]
        phone -> Nullable<Text>,
        otp_bucket -> Jsonb,
        #[max_length = 50]
        status -> Text,
    }
}

diesel::table! {
    authapp_user_token (id) {
        id -> Int8,
        created_on -> Timestamptz,
        updated_on -> Timestamptz,
        #[max_length = 255]
        token -> Text,
        active -> Bool,
        #[max_length = 255]
        device_number -> Nullable<Text>,
        user_id -> Int8,
    }
}

diesel::joinable!(authapp_user_token -> authapp_user (user_id));

diesel::allow_tables_to_appear_in_same_query!(authapp_user, authapp_user_otp, authapp_user_token,);

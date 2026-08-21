// @generated automatically by Diesel CLI.
// This file is regenerated automatically when you run `diesel migration run`.
// Do not edit manually - changes will be overwritten.

diesel::table! {
    authorities (name) {
        name -> Varchar,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        login -> Varchar,
        password_hash -> Varchar,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        email -> Varchar,
        activated -> Bool,
        lang_key -> Nullable<Varchar>,
        image_url -> Nullable<Varchar>,
        created_by -> Nullable<Varchar>,
        created_date -> Nullable<Timestamp>,
        last_modified_by -> Nullable<Varchar>,
        last_modified_date -> Nullable<Timestamp>,
    }
}

diesel::table! {
    user_authorities (user_id, authority_name) {
        user_id -> Int4,
        authority_name -> Varchar,
    }
}

diesel::joinable!(user_authorities -> users (user_id));
diesel::joinable!(user_authorities -> authorities (authority_name));

// Track 1-c.2 fix (2026-05-11): injection point for JDL-declared entity tables.
// POST_WRITING_ENTITIES iterates entities and calls source.addEntityToRustSchema
// for each, which injects a `diesel::table!` block here. Without these
// injections every handler/service `use crate::db::schema::<entity>` fails
// with E0432/E0433.
diesel::table! {
    job_offer (id) {
        id -> Int4,
        title -> Varchar,
        company -> Nullable<Varchar>,
        location -> Nullable<Varchar>,
        country -> Nullable<Varchar>,
        remote -> Nullable<Bool>,
        description -> Nullable<Varchar>,
        search_text -> Nullable<Varchar>,
        skills -> Nullable<Varchar>,
        metadata -> Nullable<Varchar>,
        raw_payload -> Nullable<Varchar>,
        content_hash -> Nullable<Varchar>,
        embedding_status -> Nullable<Varchar>,
        embedding_model -> Nullable<Varchar>,
        reindex_version -> Nullable<Int4>,
        retry_count -> Nullable<Int4>,
        indexing_error -> Nullable<Varchar>,
        source -> Nullable<Varchar>,
        source_id -> Nullable<Varchar>,
        apply_url -> Nullable<Varchar>,
        salary_min -> Nullable<Int4>,
        salary_max -> Nullable<Int4>,
        salary_currency -> Nullable<Varchar>,
        contract_type -> Nullable<Varchar>,
        experience_level -> Nullable<Varchar>,
        category -> Nullable<Varchar>,
        source_category -> Nullable<Varchar>,
        published_at -> Nullable<Timestamp>,
        created_at -> Nullable<Timestamp>,
        indexed_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        expires_at -> Nullable<Timestamp>,
        last_checked_at -> Nullable<Timestamp>,
        created_by -> Nullable<Varchar>,
        created_date -> Nullable<Timestamp>,
        last_modified_by -> Nullable<Varchar>,
        last_modified_date -> Nullable<Timestamp>,
    }
}
diesel::table! {
    candidate_profile (id) {
        id -> Int4,
        user_id -> Varchar,
        full_name -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
        location -> Nullable<Varchar>,
        years_of_experience -> Nullable<Int4>,
        skills -> Nullable<Varchar>,
        experiences -> Nullable<Varchar>,
        preferred_roles -> Nullable<Varchar>,
        languages -> Nullable<Varchar>,
        education -> Nullable<Varchar>,
        certifications -> Nullable<Varchar>,
        raw_markdown -> Nullable<Varchar>,
        cv_filename -> Nullable<Varchar>,
        embedding_model -> Nullable<Varchar>,
        embedded_at -> Nullable<Timestamp>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        created_by -> Nullable<Varchar>,
        created_date -> Nullable<Timestamp>,
        last_modified_by -> Nullable<Varchar>,
        last_modified_date -> Nullable<Timestamp>,
    }
}
diesel::table! {
    job_application (id) {
        id -> Int4,
        user_id -> Varchar,
        status -> Nullable<Varchar>,
        cover_letter -> Nullable<Varchar>,
        notes -> Nullable<Varchar>,
        match_score -> Nullable<Float8>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        applied_at -> Nullable<Timestamp>,
        jobOffer_id -> Nullable<Int4>,
        candidateProfile_id -> Nullable<Int4>,
        created_by -> Nullable<Varchar>,
        created_date -> Nullable<Timestamp>,
        last_modified_by -> Nullable<Varchar>,
        last_modified_date -> Nullable<Timestamp>,
    }
}
diesel::table! {
    user_preference (id) {
        id -> Int4,
        user_id -> Varchar,
        remote_only -> Nullable<Bool>,
        contract_type -> Nullable<Varchar>,
        salary_min -> Nullable<Int4>,
        salary_max -> Nullable<Int4>,
        preferred_roles -> Nullable<Varchar>,
        excluded_technologies -> Nullable<Varchar>,
        preferred_locations -> Nullable<Varchar>,
        created_by -> Nullable<Varchar>,
        created_date -> Nullable<Timestamp>,
        last_modified_by -> Nullable<Varchar>,
        last_modified_date -> Nullable<Timestamp>,
    }
}
diesel::table! {
    auto_apply_config (id) {
        id -> Int4,
        user_id -> Varchar,
        mode -> Nullable<Varchar>,
        min_score -> Nullable<Float8>,
        max_per_day -> Nullable<Int4>,
        sources -> Nullable<Varchar>,
        created_by -> Nullable<Varchar>,
        created_date -> Nullable<Timestamp>,
        last_modified_by -> Nullable<Varchar>,
        last_modified_date -> Nullable<Timestamp>,
    }
}
diesel::table! {
    radar_hit (id) {
        id -> Int4,
        user_id -> Varchar,
        score -> Nullable<Float8>,
        why_you -> Nullable<Varchar>,
        seen -> Nullable<Bool>,
        dismissed -> Nullable<Bool>,
        created_at -> Nullable<Timestamp>,
        jobOffer_id -> Nullable<Int4>,
        created_by -> Nullable<Varchar>,
        created_date -> Nullable<Timestamp>,
        last_modified_by -> Nullable<Varchar>,
        last_modified_date -> Nullable<Timestamp>,
    }
}
diesel::table! {
    radar_state (id) {
        id -> Int4,
        user_id -> Varchar,
        last_offer_at -> Nullable<Timestamp>,
        created_by -> Nullable<Varchar>,
        created_date -> Nullable<Timestamp>,
        last_modified_by -> Nullable<Varchar>,
        last_modified_date -> Nullable<Timestamp>,
    }
}
diesel::table! {
    conversation (id) {
        id -> Int4,
        user_id -> Varchar,
        title -> Nullable<Varchar>,
        summary -> Nullable<Varchar>,
        metadata -> Nullable<Varchar>,
        type_chat -> Nullable<Varchar>,
        created_at -> Timestamp,
        last_message_at -> Nullable<Timestamp>,
        created_by -> Nullable<Varchar>,
        created_date -> Nullable<Timestamp>,
        last_modified_by -> Nullable<Varchar>,
        last_modified_date -> Nullable<Timestamp>,
    }
}
diesel::table! {
    cv_resume (id) {
        id -> Int4,
        user_id -> Varchar,
        title -> Nullable<Varchar>,
        template -> Nullable<Varchar>,
        data -> Varchar,
        version_number -> Int4,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        created_by -> Nullable<Varchar>,
        created_date -> Nullable<Timestamp>,
        last_modified_by -> Nullable<Varchar>,
        last_modified_date -> Nullable<Timestamp>,
    }
}
diesel::table! {
    cv_resume_version (id) {
        id -> Int4,
        version_number -> Int4,
        title -> Nullable<Varchar>,
        template -> Nullable<Varchar>,
        data -> Varchar,
        created_at -> Nullable<Timestamp>,
        resume_id -> Nullable<Int4>,
        created_by -> Nullable<Varchar>,
        created_date -> Nullable<Timestamp>,
        last_modified_by -> Nullable<Varchar>,
        last_modified_date -> Nullable<Timestamp>,
    }
}
diesel::table! {
    offer_positioning (id) {
        id -> Int4,
        user_id -> Varchar,
        result -> Varchar,
        created_at -> Nullable<Timestamp>,
        jobOffer_id -> Nullable<Int4>,
        created_by -> Nullable<Varchar>,
        created_date -> Nullable<Timestamp>,
        last_modified_by -> Nullable<Varchar>,
        last_modified_date -> Nullable<Timestamp>,
    }
}
diesel::table! {
    offer_tailored_resume (id) {
        id -> Int4,
        user_id -> Varchar,
        data -> Varchar,
        title -> Nullable<Varchar>,
        created_at -> Nullable<Timestamp>,
        jobOffer_id -> Nullable<Int4>,
        created_by -> Nullable<Varchar>,
        created_date -> Nullable<Timestamp>,
        last_modified_by -> Nullable<Varchar>,
        last_modified_date -> Nullable<Timestamp>,
    }
}
// jhipster-needle-add-entity-schema

diesel::allow_tables_to_appear_in_same_query!(
    authorities,
    users,
    user_authorities,
    job_offer,
    candidate_profile,
    job_application,
    user_preference,
    auto_apply_config,
    radar_hit,
    radar_state,
    conversation,
    cv_resume,
    cv_resume_version,
    offer_positioning,
    offer_tailored_resume,
    // jhipster-needle-add-allow-table
);

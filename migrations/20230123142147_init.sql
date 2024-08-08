create type login_code_state as enum ('sent', 'verified', 'used');

create table "user"
(
    id            uuid                     default gen_random_uuid()     not null
        constraint user_pk
            primary key,
    email         varchar                                                not null,
    name          varchar                  default ''::character varying not null,
    created_at    timestamp with time zone default now()                 not null,
    registered_at timestamp with time zone
);

create unique index user_email_uindex
    on "user" (email);

create unique index user_id_uindex
    on "user" (id);

create table magic_link
(
    id         uuid                     default gen_random_uuid()         not null
        constraint email_code_pk
            primary key,
    email      varchar                                                    not null,
    created_at timestamp with time zone default now()                     not null,
    state      varchar                  default 'sent'::character varying not null,
    token      uuid                                                       not null
);

create table tag
(
    id         uuid                     default gen_random_uuid() not null
        primary key,
    name       varchar                                            not null,
    user_id    uuid                                               not null,
    video_id   uuid                                               not null,
    created_at timestamp with time zone default now()             not null
);

create unique index tag_user_video_name_uindex
    on tag (name, user_id, video_id);

create table project
(
    id         uuid                     default gen_random_uuid() not null
        primary key,
    name       varchar                                            not null,
    user_id    uuid                                               not null
        references "user",
    created_at timestamp with time zone default now()             not null
);

create table video
(
    id                   uuid                     default gen_random_uuid()              not null
        constraint video_pk
            primary key,
    duration             integer                  default 0                              not null,
    title                varchar                                                         not null,
    width                integer                  default 0                              not null,
    height               integer                  default 0                              not null,
    image_link           varchar                                                         not null,
    created_at           timestamp with time zone default now()                          not null,
    state                varchar                  default 'uploading'::character varying not null,
    description          varchar                  default ''::character varying          not null,
    user_id              uuid                                                            not null
        references "user",
    project_id           uuid
        references project,
    preview_link         varchar                                                         not null,
    processing           integer                  default 0                              not null,
    is_share_link_active boolean                  default false                          not null,
    deleted              boolean                  default false                          not null
);

create unique index video_id_uindex
    on video (id);

create table video_access
(
    id         uuid                     default gen_random_uuid() not null
        constraint sharing_pkey
            primary key,
    created_at timestamp with time zone default now()             not null,
    video_id   uuid                                               not null
        constraint sharing_video_id_fkey
            references video,
    project_id uuid
        constraint video_sharing_project_id_fkey
            references project,
    user_id    uuid                                               not null
        references "user",
    updated_at timestamp with time zone default now()             not null,
    email      varchar
);

create table project_access
(
    id         uuid                     default gen_random_uuid() not null
        primary key,
    created_at timestamp with time zone default now()             not null,
    project_id uuid                                               not null
        constraint video_sharing_project_id_fkey
            references project,
    user_id    uuid                                               not null
        constraint video_access_user_id_fkey
            references "user",
    email      varchar
);

create table review
(
    id           uuid                     default gen_random_uuid()     not null
        constraint review_pk
            primary key,
    created_at   timestamp with time zone default now()                 not null,
    user_id      uuid                                                   not null
        references "user",
    text         varchar                  default ''::character varying not null,
    time         integer                  default 0                     not null,
    video_id     uuid                                                   not null
        references video,
    duration     integer                  default 5                     not null,
    is_published boolean                  default false,
    updated_at   timestamp with time zone default now()                 not null
);

create unique index review_id_uindex
    on review (id);

create unique index review_time_user_index
    on review (video_id, time, user_id);

create table review_drawing
(
    id        uuid default gen_random_uuid() not null,
    drawing   double precision[]             not null,
    review_id uuid                           not null,
    color     varchar                        not null
);

create table review_reply
(
    id         uuid                     default gen_random_uuid()     not null
        constraint review_reply_pk
            primary key,
    created_at timestamp with time zone default now()                 not null,
    user_id    uuid                                                   not null
        references "user",
    text       varchar                  default ''::character varying not null,
    updated_at timestamp with time zone default now()                 not null,
    reply_for  uuid
        constraint review_review_id_fk
            references review
);

create unique index review_reply_id_uindex
    on review_reply (id);


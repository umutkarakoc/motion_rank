create table magic_link
(
    id         uuid                     default gen_random_uuid()         not null
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

create table "user"
(
    id            uuid                     default gen_random_uuid()     not null
        primary key,
    email         varchar                                                not null,
    name          varchar                  default ''::character varying not null,
    created_at    timestamp with time zone default now()                 not null,
    registered_at timestamp with time zone
);

create table project
(
    id         uuid                     default gen_random_uuid() not null
        primary key,
    name       varchar                                            not null,
    user_id    uuid                                               not null
        references "user",
    created_at timestamp with time zone default now()             not null
);

create table project_access
(
    id         uuid                     default gen_random_uuid() not null
        primary key,
    created_at timestamp with time zone default now()             not null,
    project_id uuid                                               not null
        references project,
    user_id    uuid                                               not null
        references "user",
    email      varchar
);

create table video
(
    id                   uuid                     default gen_random_uuid()              not null
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

create table review
(
    id         uuid                     default gen_random_uuid() not null
        primary key,
    created_at timestamp with time zone default now()             not null,
    user_id    uuid                                               not null
        references "user",
    video_id   uuid                                               not null
        references video,
    text       varchar                                            not null,
    time       integer                  default 0                 not null,
    reply_for  uuid,
    is_deleted boolean                  default false
);

create table review_comment
(
    id         uuid                     default gen_random_uuid()            not null
        primary key,
    created_at timestamp with time zone default now()                        not null,
    user_id    uuid                                                          not null
        references "user",
    text       varchar                  default ''::character varying        not null,
    time       integer                  default 0                            not null,
    video_id   uuid                                                          not null
        references video,
    updated_at timestamp with time zone default now()                        not null,
    review_id  uuid                                                          not null
        references review,
    drawing    double precision[],
    color      varchar                  default '#000000'::character varying not null
);

create table review_drawing
(
    id         uuid                     default gen_random_uuid()            not null
        primary key,
    created_at timestamp with time zone default now()                        not null,
    title      varchar                  default ''::character varying        not null,
    review_id  uuid                                                          not null
        references review,
    drawing    integer[]                                                     not null,
    color      varchar                  default '#000000'::character varying not null
);

create table video_access
(
    id         uuid                     default gen_random_uuid() not null
        primary key,
    created_at timestamp with time zone default now()             not null,
    video_id   uuid                                               not null
        references video,
    project_id uuid
        references project,
    user_id    uuid                                               not null
        references "user",
    updated_at timestamp with time zone default now()             not null,
    email      varchar
);


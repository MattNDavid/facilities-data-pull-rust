--
-- PostgreSQL database dump
--

\restrict ybaqg7bTEPuqSWp0JhDKY9RF5o2fqzfjmyfJH5MUZ7qXaL3rjvR8J8uUW7K3V7K

-- Dumped from database version 18.4
-- Dumped by pg_dump version 18.4

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: answers; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.answers (
    id integer NOT NULL,
    question character varying(512),
    answer character varying(512),
    event_resource_request_id integer
);


ALTER TABLE public.answers OWNER TO postgres;

--
-- Name: event_instance_tag_map; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.event_instance_tag_map (
    event_instance_id integer NOT NULL,
    tag_id integer
);


ALTER TABLE public.event_instance_tag_map OWNER TO postgres;

--
-- Name: event_instances; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.event_instances (
    id integer,
    date date,
    start_time time without time zone,
    end_time time without time zone,
    event_id integer
);


ALTER TABLE public.event_instances OWNER TO postgres;

--
-- Name: event_resource_requests; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.event_resource_requests (
    id integer,
    notes character varying(511),
    event_id integer,
    resource_id integer,
    room_setup_id integer,
    quantity integer
);


ALTER TABLE public.event_resource_requests OWNER TO postgres;

--
-- Name: events; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.events (
    id integer,
    name character varying(255),
    description character varying(255),
    summary character varying(255),
    owner_id integer
);


ALTER TABLE public.events OWNER TO postgres;

--
-- Name: owners; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.owners (
    id integer NOT NULL,
    first_name character varying(20),
    last_name character varying(20),
    email character varying(255)
);


ALTER TABLE public.owners OWNER TO postgres;

--
-- Name: resource_bookings; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.resource_bookings (
    id integer NOT NULL,
    date date,
    start_time time without time zone,
    end_time time without time zone,
    event_id integer,
    event_resource_request_id integer,
    event_instance_id integer,
    resource_id integer
);


ALTER TABLE public.resource_bookings OWNER TO postgres;

--
-- Name: resources; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.resources (
    id integer,
    name character varying(255),
    room boolean
);


ALTER TABLE public.resources OWNER TO postgres;

--
-- Name: facilities_schedule; Type: VIEW; Schema: public; Owner: postgres
--

CREATE VIEW public.facilities_schedule AS
 SELECT resource_bookings.id,
    resource_bookings.date,
    resource_bookings.start_time,
    resource_bookings.end_time,
    resources.name AS resource_name,
    events.name AS event_name,
    events.owner_id,
    event_resource_requests.notes,
    owners.first_name,
    owners.last_name,
    owners.email,
    answers.question,
    answers.answer
   FROM (((((public.resource_bookings
     JOIN public.resources ON ((resource_bookings.resource_id = resources.id)))
     JOIN public.events ON ((events.id = resource_bookings.event_id)))
     JOIN public.event_resource_requests ON ((event_resource_requests.id = resource_bookings.event_resource_request_id)))
     LEFT JOIN public.owners ON ((events.owner_id = owners.id)))
     LEFT JOIN public.answers ON ((event_resource_requests.id = answers.event_resource_request_id)))
  WHERE ((answers.question)::text = 'Exception to regular setup?'::text);


ALTER VIEW public.facilities_schedule OWNER TO postgres;

--
-- Name: tag_groups; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.tag_groups (
    id integer NOT NULL,
    name character varying(50)
);


ALTER TABLE public.tag_groups OWNER TO postgres;

--
-- Name: tag_groups_tags_map; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.tag_groups_tags_map (
    tag_group_id integer,
    tag_id integer
);


ALTER TABLE public.tag_groups_tags_map OWNER TO postgres;

--
-- Name: tags; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.tags (
    id integer NOT NULL,
    color character varying(10),
    name character varying(50)
);


ALTER TABLE public.tags OWNER TO postgres;

--
-- Name: answers answers_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.answers
    ADD CONSTRAINT answers_pkey PRIMARY KEY (id);


--
-- Name: event_instance_tag_map event_instance_tag_map_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.event_instance_tag_map
    ADD CONSTRAINT event_instance_tag_map_pkey PRIMARY KEY (event_instance_id);


--
-- Name: event_resource_requests event_resource_requests_id_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.event_resource_requests
    ADD CONSTRAINT event_resource_requests_id_key UNIQUE (id);


--
-- Name: events events_id_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.events
    ADD CONSTRAINT events_id_key UNIQUE (id);


--
-- Name: owners owners_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.owners
    ADD CONSTRAINT owners_pkey PRIMARY KEY (id);


--
-- Name: resource_bookings resource_bookings_id_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.resource_bookings
    ADD CONSTRAINT resource_bookings_id_key UNIQUE (id);


--
-- Name: resources resources_id_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.resources
    ADD CONSTRAINT resources_id_key UNIQUE (id);


--
-- Name: tag_groups tag_groups_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tag_groups
    ADD CONSTRAINT tag_groups_pkey PRIMARY KEY (id);


--
-- Name: tags tags_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_pkey PRIMARY KEY (id);


--
-- Name: event_instances unique_id; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.event_instances
    ADD CONSTRAINT unique_id UNIQUE (id);


--
-- Name: event_instance_tag_map uq_ei_tag; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.event_instance_tag_map
    ADD CONSTRAINT uq_ei_tag UNIQUE (event_instance_id, tag_id);


--
-- Name: tag_groups_tags_map uq_tg_tag; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tag_groups_tags_map
    ADD CONSTRAINT uq_tg_tag UNIQUE (tag_group_id, tag_id);


--
-- PostgreSQL database dump complete
--

\unrestrict ybaqg7bTEPuqSWp0JhDKY9RF5o2fqzfjmyfJH5MUZ7qXaL3rjvR8J8uUW7K3V7K


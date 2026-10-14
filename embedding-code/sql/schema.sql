-- PostgreSQL 18 / pgvector 0.8.7. Idempotent creation, not a migration engine.
CREATE EXTENSION IF NOT EXISTS vector;
CREATE SCHEMA IF NOT EXISTS retrieval;

CREATE TABLE IF NOT EXISTS retrieval.collections (
    collection_id text PRIMARY KEY CHECK (length(collection_id) BETWEEN 1 AND 120),
    manifest_sha256 text NOT NULL CHECK (manifest_sha256 ~ '^[0-9a-f]{64}$'),
    model text NOT NULL,
    revision text NOT NULL,
    profile text NOT NULL,
    dimensions integer NOT NULL CHECK (dimensions = 1024),
    source_manifest jsonb NOT NULL,
    chunk_count integer NOT NULL CHECK (chunk_count > 0),
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS retrieval.documents (
    collection_id text NOT NULL REFERENCES retrieval.collections(collection_id),
    document_id text NOT NULL,
    source_file text NOT NULL,
    source_sha256 text NOT NULL CHECK (source_sha256 ~ '^[0-9a-f]{64}$'),
    metadata jsonb NOT NULL,
    PRIMARY KEY (collection_id, document_id)
);
CREATE TABLE IF NOT EXISTS retrieval.passages (
    collection_id text NOT NULL,
    chunk_id text NOT NULL,
    document_id text NOT NULL,
    chunk_order integer NOT NULL CHECK (chunk_order >= 0),
    page_start integer NOT NULL CHECK (page_start > 0),
    page_end integer NOT NULL CHECK (page_end >= page_start),
    content_sha256 text NOT NULL CHECK (content_sha256 ~ '^[0-9a-f]{64}$'),
    passage text NOT NULL CHECK (length(passage) > 0),
    metadata jsonb NOT NULL,
    token_count integer NOT NULL CHECK (token_count BETWEEN 1 AND 32768),
    embedding halfvec(1024) NOT NULL CHECK (l2_norm(embedding) BETWEEN 0.998 AND 1.002),
    PRIMARY KEY (collection_id, chunk_id),
    FOREIGN KEY (collection_id, document_id)
        REFERENCES retrieval.documents(collection_id, document_id)
);
CREATE INDEX IF NOT EXISTS passages_embedding_hnsw
    ON retrieval.passages USING hnsw (embedding halfvec_cosine_ops);
CREATE INDEX IF NOT EXISTS passages_collection ON retrieval.passages(collection_id);

-- Separate read-only login. Password is configured locally by postgres.sh, never here.
DO $$ BEGIN
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'retrieval_reader') THEN
        CREATE ROLE retrieval_reader LOGIN;
    END IF;
END $$;
GRANT USAGE ON SCHEMA retrieval TO retrieval_reader;
GRANT SELECT ON ALL TABLES IN SCHEMA retrieval TO retrieval_reader;
ALTER DEFAULT PRIVILEGES IN SCHEMA retrieval GRANT SELECT ON TABLES TO retrieval_reader;

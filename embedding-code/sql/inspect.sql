-- Run: CONTAINER_ENGINE=podman embedding-code/postgres.sh sql < embedding-code/sql/inspect.sql
SELECT c.collection_id, c.model, c.revision, c.profile, c.dimensions,
       c.source_manifest->>'status' AS corpus_review_status,
       c.chunk_count AS declared_count, count(p.chunk_id) AS stored_count,
       min(p.token_count) AS min_tokens, max(p.token_count) AS max_tokens,
       min(l2_norm(p.embedding)) AS min_norm, max(l2_norm(p.embedding)) AS max_norm
FROM retrieval.collections c
LEFT JOIN retrieval.passages p USING (collection_id)
GROUP BY c.collection_id;
SELECT collection_id, source_file, count(*) AS passages
FROM retrieval.documents JOIN retrieval.passages USING (collection_id, document_id)
GROUP BY collection_id, source_file ORDER BY collection_id, source_file;
-- This role can read, not write (false expected for all write checks).
SELECT has_table_privilege('retrieval_reader', 'retrieval.passages', 'SELECT') AS can_read,
       has_table_privilege('retrieval_reader', 'retrieval.passages', 'INSERT') AS can_insert,
       has_table_privilege('retrieval_reader', 'retrieval.passages', 'UPDATE') AS can_update,
       has_table_privilege('retrieval_reader', 'retrieval.passages', 'DELETE') AS can_delete;

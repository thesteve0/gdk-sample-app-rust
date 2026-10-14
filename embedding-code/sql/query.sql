-- Bound parameters: $1 collection ID, $2 query embedding JSON array text, $3 limit (bigint).
-- Query embedding MUST use the collection's pinned Harrier query profile.
SELECT p.chunk_id, p.document_id, d.source_file, d.source_sha256,
       p.page_start, p.page_end, p.content_sha256, p.passage, p.metadata,
       (1 - (p.embedding <=> $2::text::halfvec(1024)))::double precision AS similarity
FROM retrieval.passages p
JOIN retrieval.documents d USING (collection_id, document_id)
WHERE p.collection_id = $1
ORDER BY p.embedding <=> $2::text::halfvec(1024)
LIMIT $3;

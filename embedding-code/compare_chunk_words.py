#!/usr/bin/env python3
"""Read-only what-if reports; inferred headings are candidates, not accepted sections."""
import argparse
import csv
import hashlib
import io
import json
import math
import os
from pathlib import Path
import re
import statistics
import subprocess
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent.parent
POLICIES = {
    "3e32d4176274c8ffcabd432235bdac313160ee77afbc15592e4cf5f9124793ec": "beginners",
    "74296415db1e5d94b64352751df2a7b4d559b227fc0916f31d545e2ad35efeb0": "heitkoetter",
    "50f6e16003682e96f7bd5fc05d5947fcbcc3efba5e8c37afdca908f802251049": "cfi",
}


def compact(text):
    return " ".join(text.split())


def key(text):
    return re.sub(r"[^\w]", "", text.casefold())


def locate(lines, text):
    """Match typography to exact cleaned-text lines, without reordering source text."""
    target = key(text)
    if not target:
        return None
    for start in range(len(lines)):
        if not lines[start].strip():
            continue
        for width in range(1, 5):
            if key(" ".join(lines[start:start + width])) == target:
                return start
    # CFI sidebar text sometimes shares an extracted line with a heading.
    for start, line in enumerate(lines):
        if key(line).endswith(target):
            return start
    return None


def heading_level(policy, font, text):
    size = int(font["size"])
    color = font.get("color", "")
    family = font.get("family", "")
    if len(text.split()) > 24 or len(key(text)) < 3:
        return None
    if policy == "beginners":
        if color == "#3bbdee" and size >= 27:
            return "major"
        if color == "#3bbdee" and size == 21 or color == "#ffffff" and size == 27:
            return "subsection"
    if policy == "heitkoetter":
        if size == 45:
            return "major"
        if size == 17 and family.endswith("+Arial") and color == "#000000":
            return "subsection"
    if policy == "cfi":
        if size == 37 or size == 21 and color == "#122d56":
            return "major"
        if size == 15 and color == "#f9611c" and "OpenSans" in family:
            return "subsection"
    return None


def candidates(pages, xml, policy):
    root = ET.fromstring(xml)
    fonts = {f.get("id"): f.attrib for f in root.findall(".//fontspec")}
    by_page = {p["number"]: p for p in pages}
    found = {}
    for page in root.findall("page"):
        number = int(page.get("number"))
        if number not in by_page:
            continue
        # TOCs and cover material are retained in a leading unsectioned chunk,
        # not mistaken for actual chapter starts.
        if policy == "heitkoetter" and number < 8 or policy == "cfi" and number < 7:
            continue
        lines = by_page[number]["raw_text"].splitlines(keepends=True)
        # PDF text nodes can split a title into styled spans. Reassemble each
        # typographic baseline before matching it to cleaned extraction.
        groups = {}
        for node in page.findall("text"):
            font = fonts[node.get("font")]
            if heading_level(policy, font, "Candidate heading") is None:
                continue
            baseline = (int(node.get("top")), int(font["size"]))
            groups.setdefault(baseline, []).append(node)
        for nodes in groups.values():
            nodes.sort(key=lambda n: int(n.get("left")))
            text = compact(" ".join("".join(n.itertext()) for n in nodes))
            level = heading_level(policy, fonts[nodes[0].get("font")], text)
            if level is None:
                continue
            line = locate(lines, text)
            if line is None:
                continue
            position = (number, line)
            previous = found.get(position)
            if previous is None or level == "major":
                found[position] = {"page": number, "line": line + 1, "title": text, "level": level}
    # Adjacent matching heading lines (wrapped titles) are one boundary.
    merged = []
    for item in sorted(found.values(), key=lambda x: (x["page"], x["line"])):
        gap_blank = False
        if merged and item["page"] == merged[-1]["page"]:
            source_lines = by_page[item["page"]]["raw_text"].splitlines()
            gap = source_lines[merged[-1]["last_line"]:item["line"] - 1]
            gap_blank = item["line"] > merged[-1]["last_line"] and all(not line.strip() for line in gap)
        if merged and gap_blank and item["level"] == merged[-1]["level"]:
            merged[-1]["title"] += " " + item["title"]
            merged[-1]["last_line"] = item["line"]
        else:
            merged.append(dict(item, last_line=item["line"]))
    return merged


def section_rows(pages, headings, include_subsections):
    lines = [(p["number"], i + 1, line) for p in pages for i, line in enumerate(p["raw_text"].splitlines(keepends=True))]
    offsets = {(page, line): i for i, (page, line, _) in enumerate(lines)}
    starts = [(0, "Leading material / no detected heading")]
    for h in headings:
        if include_subsections or h["level"] == "major":
            starts.append((offsets[(h["page"], h["line"])], h["title"]))
    starts = sorted(dict(starts).items())
    result = []
    for index, (start, title) in enumerate(starts):
        end = starts[index + 1][0] if index + 1 < len(starts) else len(lines)
        selected = lines[start:end]
        words = sum(len(line.split()) for _, _, line in selected)
        if not words:
            continue
        result.append(dict(chunk=len(result) + 1, title=title, page_start=selected[0][0], page_end=selected[-1][0], base_words=words, overlap_words=0, total_words=words))
    return result


def page_rows(pages):
    result = []
    previous_words = 0
    for page in pages:
        count = len(page["raw_text"].split())
        overlap = math.ceil(previous_words / 10) if count else 0
        if count:
            result.append(dict(chunk=len(result) + 1, title="Physical page {}".format(page["number"]), page_start=page["number"] - 1 if overlap else page["number"], page_end=page["number"], base_words=count, overlap_words=overlap, total_words=count + overlap))
        previous_words = count  # No recursive carry-over or jumping across empty pages.
    return result


def summary(rows):
    counts = [r["total_words"] for r in rows]
    return dict(chunks=len(rows), min=min(counts, default=0), median=statistics.median(counts) if counts else 0, max=max(counts, default=0), total=sum(counts))


def private_save(path, text):
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, "w") as stream:
        stream.write(text)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", type=Path, required=True)
    args = parser.parse_args()
    run = args.run.resolve()
    manifest = json.loads((run / "manifest.json").read_text())
    import time
    output = run / ("word-count-comparison-{}".format(time.time_ns()))
    output.mkdir(mode=0o700)  # Refuse overwrite; preserve previous reports.
    reports = []
    for doc in manifest["documents"]:
        directory = run / doc["id"]
        policy = POLICIES[doc["source_sha256"]]
        paths = [directory / "cleaned-pages.jsonl", directory / "source.pdf"]
        for path in paths:
            artifact = next(a for a in manifest["artifacts"] if a["path"] == path.relative_to(run).as_posix())
            if hashlib.sha256(path.read_bytes()).hexdigest() != artifact["sha256"]:
                raise ValueError("Artifact checksum mismatch: {}".format(path))
        pages = [json.loads(line) for line in paths[0].read_text().splitlines()]
        process = subprocess.run(["pdftohtml", "-xml", "-i", "-stdout", str(paths[1])], capture_output=True, check=True, timeout=120)
        private_save(output / (doc["id"] + "-typography.xml"), process.stdout.decode())
        private_save(output / (doc["id"] + "-typography.log"), process.stderr.decode())
        headings = candidates(pages, process.stdout, policy)
        private_save(output / (doc["id"] + "-heading-candidates.json"), json.dumps(headings, indent=2) + "\n")
        variants = {
            "major_sections": section_rows(pages, headings, False),
            "subsections": section_rows(pages, headings, True),
            "pages_10pct_overlap": page_rows(pages),
        }
        totals = {}
        for name, rows in variants.items():
            if name != "pages_10pct_overlap":
                assert sum(r["base_words"] for r in rows) == sum(len(p["raw_text"].split()) for p in pages)
            stream = io.StringIO()
            writer = csv.DictWriter(stream, fieldnames=["chunk", "title", "page_start", "page_end", "base_words", "overlap_words", "total_words"])
            writer.writeheader()
            writer.writerows(rows)
            private_save(output / (doc["id"] + "-" + name + ".csv"), stream.getvalue())
            totals[name] = summary(rows)
        reports.append(dict(book=doc["source_file"], document_id=doc["id"], candidate_headings=len(headings), variants=totals))
        print(doc["source_file"])
        for name, stats in totals.items():
            print("  {}: {}".format(name, stats))
    private_save(output / "summary.json", json.dumps(dict(status="what_if_inferred_headings_not_verified", word_definition="whitespace-separated items, including symbols; not tokens", overlap="ceil(10% of preceding physical page's base words), prepended; no recursive overlap; empty pages produce no chunk and reset overlap", reports=reports), indent=2) + "\n")
    print("Detailed private CSVs and heading candidates: {}".format(output))


if __name__ == "__main__":
    main()

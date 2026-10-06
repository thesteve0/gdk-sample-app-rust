# Lesson 7 lecture visuals

Five progressive frames accompany [Lesson 7](../LESSON.md). Show them in numeric order; [instructor notes](../../../instructor-notes/07-state-machine-mental-model.md) specify reveals, pointers, and prediction questions. Each lesson caption is a text equivalent; SVGs also include accessible title/description elements.

| Frame | Editable source | Plugin-free lecture image |
|---|---|---|
| Ownership and external input | [01-ownership.svg](01-ownership.svg) | [PNG](01-ownership.png) |
| Ordered pass and reload | [02-pass.svg](02-pass.svg) | [PNG](02-pass.png) |
| Simple exchange: question / proposed reply | [03-simple-start.svg](03-simple-start.svg) | [PNG](03-simple-start.png) |
| Simple exchange: saved reply | [04-simple-saved.svg](04-simple-saved.svg) | [PNG](04-simple-saved.png) |
| Simple exchange: no work | [05-simple-stop.svg](05-simple-stop.svg) | [PNG](05-simple-stop.png) |

## Editing and exporting (maintainers)

The **SVG is the editable source of truth**. Edit it as plain XML or in a vector editor; no generator or diagram-language plugin is required. All text, rectangles, paths, and arrow labels are editable. Use SVGs in a browser or the 1600×900 PNGs in an ordinary image viewer or slide deck. Both formats are committed so class presentation needs no export tooling.

PNG exports use **librsvg's `rsvg-convert`** and the DejaVu Sans font. After editing a source, run `rsvg-convert 01-ownership.svg -o 01-ownership.png` from this directory (substitute the edited filename). Repeat for each changed SVG; keep its title/description and lesson caption synchronized. This is asset maintenance, not a learner coding task. librsvg is a system rendering tool, not a Cargo dependency.

Images use high-contrast labels, a common legend, and no color-only meaning. Verify text and arrows at full size and classroom distance after edits. A Session snapshot is selected recorded data, not provider serialization. All stories are labeled narrated conceptual examples, not observed runtime output. Tool/correlation frames are deferred to Lesson 9 and are not part of this lesson.

## Asset validation record

On 2026-10-05 the five retained SVGs were parsed, exported with librsvg, and checked for matching PNG dimensions. Rendered frames were reviewed for layout/readability and content consistency with their captions. Source claims were reviewed separately in the instructor notes. Actual projector/classroom review and the learner trace checkpoint remain instructor validation; no live provider run is claimed or required.
